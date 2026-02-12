//! Platform-specific power save blocker (prevent system sleep during downloads)

use std::sync::atomic::{AtomicBool, Ordering};

static SLEEP_PREVENTED: AtomicBool = AtomicBool::new(false);

/// Prevent the system from sleeping
pub fn prevent_sleep() -> Result<(), String> {
    if SLEEP_PREVENTED.load(Ordering::SeqCst) {
        return Ok(());
    }

    platform::prevent_sleep()?;
    SLEEP_PREVENTED.store(true, Ordering::SeqCst);
    tracing::info!("System sleep prevention enabled");
    Ok(())
}

/// Allow the system to sleep again
pub fn allow_sleep() -> Result<(), String> {
    if !SLEEP_PREVENTED.load(Ordering::SeqCst) {
        return Ok(());
    }

    platform::allow_sleep()?;
    SLEEP_PREVENTED.store(false, Ordering::SeqCst);
    tracing::info!("System sleep prevention disabled");
    Ok(())
}

// --- macOS implementation ---
#[cfg(target_os = "macos")]
mod platform {
    use std::sync::Mutex;
    use core_foundation::base::TCFType;

    static ASSERTION_ID: Mutex<u32> = Mutex::new(0);

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOPMAssertionCreateWithName(
            assertion_type: *const std::ffi::c_void,
            level: u32,
            name: *const std::ffi::c_void,
            assertion_id: *mut u32,
        ) -> i32;
        fn IOPMAssertionRelease(assertion_id: u32) -> i32;
    }

    // kIOPMAssertionLevelOn = 255
    const IOPM_ASSERTION_LEVEL_ON: u32 = 255;

    pub fn prevent_sleep() -> Result<(), String> {
        unsafe {
            use std::ffi::c_void;

            // Create CFString for assertion type "PreventUserIdleSystemSleep"
            let assertion_type = core_foundation::string::CFString::new("PreventUserIdleSystemSleep");
            let name = core_foundation::string::CFString::new("Motrix is downloading");

            let mut assertion_id: u32 = 0;
            let result = IOPMAssertionCreateWithName(
                assertion_type.as_concrete_TypeRef() as *const c_void,
                IOPM_ASSERTION_LEVEL_ON,
                name.as_concrete_TypeRef() as *const c_void,
                &mut assertion_id,
            );

            if result == 0 {
                *ASSERTION_ID.lock().unwrap() = assertion_id;
                Ok(())
            } else {
                Err(format!("IOPMAssertionCreateWithName failed: {}", result))
            }
        }
    }

    pub fn allow_sleep() -> Result<(), String> {
        unsafe {
            let id = *ASSERTION_ID.lock().unwrap();
            if id != 0 {
                IOPMAssertionRelease(id);
                *ASSERTION_ID.lock().unwrap() = 0;
            }
            Ok(())
        }
    }
}

// --- Windows implementation ---
#[cfg(target_os = "windows")]
mod platform {
    // ES_CONTINUOUS | ES_SYSTEM_REQUIRED
    const ES_CONTINUOUS: u32 = 0x80000000;
    const ES_SYSTEM_REQUIRED: u32 = 0x00000001;

    extern "system" {
        fn SetThreadExecutionState(flags: u32) -> u32;
    }

    pub fn prevent_sleep() -> Result<(), String> {
        unsafe {
            let result = SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED);
            if result == 0 {
                Err("SetThreadExecutionState failed".to_string())
            } else {
                Ok(())
            }
        }
    }

    pub fn allow_sleep() -> Result<(), String> {
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS);
            Ok(())
        }
    }
}

// --- Linux implementation (D-Bus Inhibit) ---
#[cfg(target_os = "linux")]
mod platform {
    use std::sync::Mutex;

    static INHIBIT_COOKIE: Mutex<Option<u32>> = Mutex::new(None);

    pub fn prevent_sleep() -> Result<(), String> {
        // Use D-Bus to call org.freedesktop.ScreenSaver.Inhibit
        let output = std::process::Command::new("dbus-send")
            .args([
                "--session",
                "--dest=org.freedesktop.ScreenSaver",
                "--type=method_call",
                "--print-reply",
                "/org/freedesktop/ScreenSaver",
                "org.freedesktop.ScreenSaver.Inhibit",
                "string:Motrix",
                "string:Downloading files",
            ])
            .output()
            .map_err(|e| format!("Failed to call dbus-send: {}", e))?;

        if output.status.success() {
            // Parse cookie from output like "   uint32 12345"
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(cookie) = stdout
                .lines()
                .find(|l| l.contains("uint32"))
                .and_then(|l| l.split_whitespace().last())
                .and_then(|s| s.parse::<u32>().ok())
            {
                *INHIBIT_COOKIE.lock().unwrap() = Some(cookie);
            }
            Ok(())
        } else {
            // Fallback: try systemd-inhibit style (non-blocking, best effort)
            tracing::warn!("D-Bus ScreenSaver.Inhibit failed, sleep prevention may not work");
            Ok(())
        }
    }

    pub fn allow_sleep() -> Result<(), String> {
        let cookie = INHIBIT_COOKIE.lock().unwrap().take();
        if let Some(cookie) = cookie {
            let _ = std::process::Command::new("dbus-send")
                .args([
                    "--session",
                    "--dest=org.freedesktop.ScreenSaver",
                    "--type=method_call",
                    "/org/freedesktop/ScreenSaver",
                    "org.freedesktop.ScreenSaver.UnInhibit",
                    &format!("uint32:{}", cookie),
                ])
                .output();
        }
        Ok(())
    }
}
