//! Aria2 engine management, RPC client, and event handling
//!
//! This module is organized into:
//! - `client` — Aria2Client and RPC communication
//! - `events` — Event types and notification handling
//! - Engine lifecycle functions (this file)

pub mod client;
pub mod events;

pub use client::Aria2Client;

use crate::{Error, Result};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::RwLock;

/// Global aria2 client instance
static ARIA2_CLIENT: RwLock<Option<Arc<Aria2Client>>> = RwLock::const_new(None);

/// Global aria2 child process handle (must be kept alive to prevent process from being killed)
/// Uses std::sync::Mutex instead of tokio::sync::Mutex because CommandChild may be !Send
/// on some platforms (Windows). A synchronous lock avoids holding !Send types across await points.
static ARIA2_PROCESS: std::sync::Mutex<Option<tauri_plugin_shell::process::CommandChild>> =
    std::sync::Mutex::new(None);

/// True when the engine is being intentionally shut down
static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

/// True while `init_engine` is running (prevents concurrent initialization)
static INITIALIZING: AtomicBool = AtomicBool::new(false);

/// True when the engine is waiting to restart after an unexpected termination
static RECOVERING: AtomicBool = AtomicBool::new(false);

/// Crash restart counter — caps restart attempts to prevent infinite loops
static RESTART_COUNT: AtomicU32 = AtomicU32::new(0);
const MAX_RESTARTS: u32 = 3;
const CLIENT_WAIT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const CLIENT_WAIT_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);
const ENGINE_IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const ENGINE_IDLE_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);
const ARIA2_SIDECAR_NAME: &str = "motrix-aria2c";

fn engine_is_busy() -> bool {
    INITIALIZING.load(Ordering::SeqCst) || RECOVERING.load(Ordering::SeqCst)
}

fn should_attempt_recovery_restart() -> bool {
    !SHUTTING_DOWN.load(Ordering::SeqCst) && RECOVERING.load(Ordering::SeqCst)
}

async fn wait_for_engine_idle_inner(
    timeout: std::time::Duration,
    poll_interval: std::time::Duration,
) -> Result<()> {
    let started = tokio::time::Instant::now();
    while engine_is_busy() {
        if started.elapsed() >= timeout {
            return Err(Error::Custom(
                "Timed out waiting for engine to become idle".into(),
            ));
        }
        tokio::time::sleep(poll_interval).await;
    }
    Ok(())
}

pub async fn wait_for_engine_idle() -> Result<()> {
    wait_for_engine_idle_inner(ENGINE_IDLE_TIMEOUT, ENGINE_IDLE_POLL_INTERVAL).await
}

#[cfg(test)]
async fn wait_for_engine_idle_for_tests(
    timeout: std::time::Duration,
    poll_interval: std::time::Duration,
) -> Result<()> {
    wait_for_engine_idle_inner(timeout, poll_interval).await
}

pub fn report_engine_failure(app: &AppHandle, message: &str) {
    RECOVERING.store(false, Ordering::SeqCst);
    let _ = app.emit("aria2-error", message.to_string());
    let _ = app.emit("aria2-connection", "terminated");
}

/// Kill any orphaned aria2c processes listening on the given port
fn kill_orphaned_aria2c(port: u16) {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("cmd")
            .args([
                "/C",
                &format!("netstat -ano | findstr \"LISTENING\" | findstr \":{port}\""),
            ])
            .output();
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(pid_str) = line.split_whitespace().last() {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        if pid > 0 {
                            tracing::info!(
                                "Killing orphaned process on port {} (PID: {})",
                                port,
                                pid
                            );
                            let _ = std::process::Command::new("taskkill")
                                .args(["/F", "/PID", &pid.to_string()])
                                .output();
                        }
                    }
                }
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let output = std::process::Command::new("lsof")
            .args(["-ti", &format!(":{port}")])
            .output();
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for pid_str in stdout.split_whitespace() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    if pid > 0 {
                        tracing::info!("Killing orphaned process on port {} (PID: {})", port, pid);
                        let _ = std::process::Command::new("kill")
                            .args(["-9", &pid.to_string()])
                            .output();
                    }
                }
            }
        }
    }
}

/// Force kill the managed aria2c child process
pub(crate) async fn force_kill_process() {
    let mut guard = ARIA2_PROCESS.lock().unwrap();
    if let Some(child) = guard.take() {
        tracing::info!("Force killing aria2c child process");
        let _ = child.kill();
    }
}

/// Graceful shutdown: try RPC shutdown first, then force kill as fallback.
/// Safe to call multiple times — subsequent calls return immediately.
pub async fn shutdown_and_cleanup() {
    // Swap returns the OLD value; if it was already true, another call is in progress
    if SHUTTING_DOWN.swap(true, Ordering::SeqCst) {
        tracing::debug!("Shutdown already in progress, skipping duplicate call");
        return;
    }

    if let Ok(client) = get_client().await {
        let _ = client.save_session().await;
        let rpc_ok = client.shutdown().await.is_ok();
        // Stop reconnection loop AFTER RPC calls so shutdown() isn't blocked
        client.mark_shutdown();
        if !rpc_ok {
            tracing::warn!("RPC shutdown failed, force killing aria2c process");
            force_kill_process().await;
        }
    } else {
        force_kill_process().await;
    }

    crate::upnp::unmap_all().await;

    let mut guard = ARIA2_CLIENT.write().await;
    *guard = None;
    RECOVERING.store(false, Ordering::SeqCst);
}

/// Initialize aria2 engine
pub async fn init_engine(app: &AppHandle) -> Result<()> {
    use crate::config::AppConfig;
    use tauri_plugin_store::StoreExt;

    // Re-entrancy guard: prevent concurrent initialization
    if INITIALIZING.swap(true, Ordering::SeqCst) {
        return Err(Error::Custom(
            "Engine initialization already in progress".into(),
        ));
    }

    // RAII guard: ensures INITIALIZING is reset on ALL exit paths (including panic/drop)
    struct InitGuard;
    impl Drop for InitGuard {
        fn drop(&mut self) {
            INITIALIZING.store(false, Ordering::SeqCst);
        }
    }
    let _init_guard = InitGuard;

    SHUTTING_DOWN.store(false, Ordering::SeqCst);
    RECOVERING.store(false, Ordering::SeqCst);
    RESTART_COUNT.store(0, Ordering::SeqCst);

    let store = app.store("config.json")?;
    let config = AppConfig::load_from_store(&store);

    let port = config.rpc_port;
    let secret = config.rpc_secret.clone();

    kill_orphaned_aria2c(port);
    start_aria2_process(app, &config)?;

    let mut client = None;
    for attempt in 1..=10 {
        if SHUTTING_DOWN.load(Ordering::SeqCst) {
            force_kill_process().await;
            return Err(Error::Custom("Engine initialization cancelled".into()));
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        if SHUTTING_DOWN.load(Ordering::SeqCst) {
            force_kill_process().await;
            return Err(Error::Custom("Engine initialization cancelled".into()));
        }

        match Aria2Client::new(app.clone(), port, secret.clone()).await {
            Ok(c) => {
                client = Some(c);
                break;
            }
            Err(e) => {
                tracing::warn!("Aria2 connection attempt {}/10 failed: {}", attempt, e);
                if attempt == 10 {
                    // Clean up the child process we started — it's useless without a client
                    force_kill_process().await;
                    return Err(e);
                }
            }
        }
    }
    let client = Arc::new(client.expect("aria2 client should be initialized after retry loop"));

    let mut guard = ARIA2_CLIENT.write().await;
    *guard = Some(client);

    tracing::info!("Aria2 engine initialized on port {}", port);

    // Map UPnP ports if enabled
    if config.enable_upnp {
        crate::upnp::set_enabled(true);
        crate::upnp::map_ports(config.bt_listen_port, config.dht_listen_port).await;
    }

    let _ = app.emit("aria2-ready", ());
    Ok(())
    // _init_guard dropped here → INITIALIZING reset to false
}

/// Start aria2 process and spawn a watchdog to detect unexpected termination
fn start_aria2_process(app: &AppHandle, config: &crate::config::AppConfig) -> Result<()> {
    use tauri_plugin_shell::ShellExt;

    let shell = app.shell();

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| Error::Custom(format!("Failed to get app data dir: {}", e)))?;
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| Error::Custom(format!("Failed to create app data dir: {}", e)))?;

    let session_path = app_data_dir.join("aria2.session");
    if !session_path.exists() {
        std::fs::File::create(&session_path)
            .map_err(|e| Error::Custom(format!("Failed to create session file: {}", e)))?;
    }

    let dht_path = app_data_dir.join("dht.dat");
    let dht6_path = app_data_dir.join("dht6.dat");

    // Write ALL configuration to aria2.conf to keep CLI args minimal.
    // This avoids Windows OS error 206 (filename too long) caused by excessive CLI arguments.
    let mut conf_lines = config.to_aria2_conf_lines();
    conf_lines.push(format!("save-session={}", session_path.display()));
    conf_lines.push(format!("input-file={}", session_path.display()));
    conf_lines.push("save-session-interval=10".to_string());
    conf_lines.push(format!("dht-file-path={}", dht_path.display()));
    conf_lines.push(format!("dht-file-path6={}", dht6_path.display()));

    let conf_path = app_data_dir.join("aria2.conf");
    std::fs::write(&conf_path, conf_lines.join("\n"))
        .map_err(|e| Error::Custom(format!("Failed to write aria2 conf: {}", e)))?;

    // Only pass --conf-path via CLI — everything else is in the conf file
    let args = vec![format!("--conf-path={}", conf_path.display())];

    let (mut rx, child) = shell
        .sidecar(ARIA2_SIDECAR_NAME)
        .map_err(|e| Error::Custom(format!("Failed to create aria2c sidecar: {}", e)))?
        .args(&args)
        .spawn()
        .map_err(|e| Error::Custom(format!("Failed to spawn aria2c: {}", e)))?;

    let mut process_guard = ARIA2_PROCESS.lock().unwrap();
    *process_guard = Some(child);

    // Spawn process watchdog to detect unexpected termination and auto-restart
    let watchdog_app = app.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                tauri_plugin_shell::process::CommandEvent::Terminated(payload) => {
                    // Use SeqCst to synchronize with shutdown_and_cleanup's swap
                    if SHUTTING_DOWN.load(Ordering::SeqCst) {
                        tracing::info!("Aria2 process terminated (intentional shutdown)");
                        return;
                    }
                    let code = payload
                        .code
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "unknown".into());
                    tracing::warn!(
                        "Aria2 process terminated unexpectedly (exit code: {})",
                        code
                    );
                    // Double-check after logging to avoid race with concurrent shutdown
                    if SHUTTING_DOWN.load(Ordering::SeqCst) {
                        tracing::info!(
                            "Shutdown started during termination handling, aborting notification"
                        );
                        return;
                    }

                    // Clear stale client so get_client() fails fast during restart
                    {
                        let mut guard = ARIA2_CLIENT.write().await;
                        *guard = None;
                    }

                    let count = RESTART_COUNT.fetch_add(1, Ordering::SeqCst);
                    if count >= MAX_RESTARTS {
                        RECOVERING.store(false, Ordering::SeqCst);
                        tracing::error!(
                            "Max restart attempts ({}) reached, giving up",
                            MAX_RESTARTS
                        );
                        let _ = watchdog_app.emit("aria2-connection", "terminated");
                        return;
                    }

                    RECOVERING.store(true, Ordering::SeqCst);
                    tracing::info!(
                        "Auto-restarting engine (attempt {}/{})",
                        count + 1,
                        MAX_RESTARTS
                    );
                    let _ = watchdog_app.emit("aria2-connection", "reconnecting");

                    // Exponential backoff: 2s, 4s, 8s
                    let delay = std::time::Duration::from_secs(2u64.pow(count + 1));
                    tokio::time::sleep(delay).await;

                    if !should_attempt_recovery_restart() {
                        tracing::info!(
                            "Skipping recovery restart because recovery is no longer active"
                        );
                        return;
                    }

                    // RECOVERING was set to gate `should_attempt_recovery_restart()` during
                    // backoff. Clear it now so `wait_for_engine_idle()` does not wait for
                    // itself (RECOVERING is included in `engine_is_busy()`).
                    RECOVERING.store(false, Ordering::SeqCst);

                    if let Err(e) = wait_for_engine_idle().await {
                        if SHUTTING_DOWN.load(Ordering::SeqCst) {
                            return;
                        }
                        tracing::error!("Engine restart preflight failed: {}", e);
                        report_engine_failure(&watchdog_app, &e.to_string());
                        return;
                    }

                    match init_engine(&watchdog_app).await {
                        Ok(()) => {
                            RESTART_COUNT.store(0, Ordering::SeqCst);
                            tracing::info!("Engine auto-restart succeeded");
                        }
                        Err(e) => {
                            RECOVERING.store(false, Ordering::SeqCst);
                            tracing::error!("Engine auto-restart failed: {}", e);
                            report_engine_failure(&watchdog_app, &e.to_string());
                            // Emit terminated if this was the last attempt
                            if RESTART_COUNT.load(Ordering::SeqCst) >= MAX_RESTARTS {
                                let _ = watchdog_app.emit("aria2-connection", "terminated");
                            }
                        }
                    }
                    return;
                }
                tauri_plugin_shell::process::CommandEvent::Stderr(line) => {
                    let text = String::from_utf8_lossy(&line);
                    tracing::debug!("aria2c stderr: {}", text.trim_end());
                }
                _ => {}
            }
        }
    });

    tracing::info!("Aria2 process started");
    Ok(())
}

/// Get the global aria2 client
pub async fn get_client() -> Result<Arc<Aria2Client>> {
    {
        let guard = ARIA2_CLIENT.read().await;
        if let Some(client) = guard.clone() {
            return Ok(client);
        }
    }

    let should_wait = || {
        !SHUTTING_DOWN.load(Ordering::SeqCst)
            && (INITIALIZING.load(Ordering::SeqCst) || RECOVERING.load(Ordering::SeqCst))
    };

    if should_wait() {
        let started = tokio::time::Instant::now();
        while started.elapsed() < CLIENT_WAIT_TIMEOUT {
            tokio::time::sleep(CLIENT_WAIT_POLL_INTERVAL).await;

            let guard = ARIA2_CLIENT.read().await;
            if let Some(client) = guard.clone() {
                return Ok(client);
            }

            if !should_wait() {
                break;
            }
        }
    }

    let guard = ARIA2_CLIENT.read().await;
    guard
        .clone()
        .ok_or_else(|| Error::Aria2Rpc("Aria2 client not initialized".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    async fn reset_test_state() {
        SHUTTING_DOWN.store(false, Ordering::SeqCst);
        INITIALIZING.store(false, Ordering::SeqCst);
        RECOVERING.store(false, Ordering::SeqCst);
        RESTART_COUNT.store(0, Ordering::SeqCst);
        let mut guard = ARIA2_CLIENT.write().await;
        *guard = None;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_client_waits_for_initialization_window_before_failing() {
        let _lock = TEST_MUTEX.lock().unwrap();
        reset_test_state().await;
        INITIALIZING.store(true, Ordering::SeqCst);

        let early = timeout(Duration::from_millis(20), get_client()).await;
        assert!(
            early.is_err(),
            "get_client should stay pending while initialization is still in progress"
        );

        INITIALIZING.store(false, Ordering::SeqCst);

        let result = timeout(Duration::from_millis(100), get_client())
            .await
            .expect("get_client should return once initialization window closes");

        match result {
            Err(Error::Aria2Rpc(message)) => {
                assert_eq!(message, "Aria2 client not initialized");
            }
            Ok(_) => panic!("get_client unexpectedly returned a client"),
            Err(other) => panic!("get_client returned unexpected error: {other}"),
        }

        reset_test_state().await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_client_waits_for_recovery_window_before_failing() {
        let _lock = TEST_MUTEX.lock().unwrap();
        reset_test_state().await;
        RECOVERING.store(true, Ordering::SeqCst);

        let early = timeout(Duration::from_millis(20), get_client()).await;
        assert!(
            early.is_err(),
            "get_client should stay pending while engine recovery is in progress"
        );

        RECOVERING.store(false, Ordering::SeqCst);

        let result = timeout(Duration::from_millis(100), get_client())
            .await
            .expect("get_client should return once recovery window closes");

        match result {
            Err(Error::Aria2Rpc(message)) => {
                assert_eq!(message, "Aria2 client not initialized");
            }
            Ok(_) => panic!("get_client unexpectedly returned a client"),
            Err(other) => panic!("get_client returned unexpected error: {other}"),
        }

        reset_test_state().await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_client_fails_fast_when_engine_is_idle_and_uninitialized() {
        let _lock = TEST_MUTEX.lock().unwrap();
        reset_test_state().await;

        let result = timeout(Duration::from_millis(20), get_client())
            .await
            .expect("get_client should fail immediately when engine is idle");

        match result {
            Err(Error::Aria2Rpc(message)) => {
                assert_eq!(message, "Aria2 client not initialized");
            }
            Ok(_) => panic!("get_client unexpectedly returned a client"),
            Err(other) => panic!("get_client returned unexpected error: {other}"),
        }

        reset_test_state().await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn wait_for_engine_idle_blocks_until_initialization_finishes() {
        let _lock = TEST_MUTEX.lock().unwrap();
        reset_test_state().await;
        INITIALIZING.store(true, Ordering::SeqCst);

        let release = tokio::spawn(async {
            tokio::time::sleep(Duration::from_millis(30)).await;
            INITIALIZING.store(false, Ordering::SeqCst);
        });

        let result = timeout(
            Duration::from_millis(120),
            wait_for_engine_idle_for_tests(Duration::from_millis(200), Duration::from_millis(10)),
        )
        .await
        .expect("wait_for_engine_idle should complete once initialization finishes");

        assert!(result.is_ok(), "wait_for_engine_idle should succeed after initialization completes");

        release.await.expect("release task should complete");
        reset_test_state().await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn wait_for_engine_idle_times_out_when_engine_stays_busy() {
        let _lock = TEST_MUTEX.lock().unwrap();
        reset_test_state().await;
        INITIALIZING.store(true, Ordering::SeqCst);

        let result = wait_for_engine_idle_for_tests(Duration::from_millis(20), Duration::from_millis(5)).await;

        match result {
            Err(Error::Custom(message)) => {
                assert_eq!(message, "Timed out waiting for engine to become idle");
            }
            Ok(()) => panic!("wait_for_engine_idle unexpectedly succeeded"),
            Err(other) => panic!("wait_for_engine_idle returned unexpected error: {other}"),
        }

        reset_test_state().await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn recovery_restart_requires_active_recovery_and_no_shutdown() {
        let _lock = TEST_MUTEX.lock().unwrap();
        reset_test_state().await;

        RECOVERING.store(true, Ordering::SeqCst);
        SHUTTING_DOWN.store(false, Ordering::SeqCst);
        assert!(should_attempt_recovery_restart());

        SHUTTING_DOWN.store(true, Ordering::SeqCst);
        assert!(!should_attempt_recovery_restart());

        SHUTTING_DOWN.store(false, Ordering::SeqCst);
        RECOVERING.store(false, Ordering::SeqCst);
        assert!(!should_attempt_recovery_restart());

        reset_test_state().await;
    }
}
