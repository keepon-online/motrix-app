use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    // Basic settings
    pub locale: String,
    pub theme: Theme,
    pub download_dir: PathBuf,
    pub auto_start: bool,
    pub start_hidden: bool,
    pub hide_on_close: bool,
    pub notify_on_complete: bool,
    pub auto_clear_completed: bool,
    pub resume_all_when_app_launched: bool,

    // Download settings
    pub max_concurrent_downloads: u32,
    pub max_connection_per_server: u32,
    pub split: u32,
    pub min_split_size: String,
    pub max_download_limit: String,
    pub max_upload_limit: String,

    // BT settings
    pub bt_listen_port: u16,
    pub dht_listen_port: u16,
    pub enable_upnp: bool,
    pub seed_ratio: f32,
    pub seed_time: u32,
    pub bt_tracker: String,
    pub tracker_source: Vec<String>,
    pub bt_force_encryption: bool,
    pub bt_require_crypto: bool,
    pub pause_metadata: bool,

    // BT advanced settings
    pub bt_save_metadata: bool,
    pub bt_load_saved_metadata: bool,
    pub bt_remove_unselected_file: bool,
    pub bt_detach_seed_only: bool,

    // Advanced settings
    pub user_agent: String,
    pub rpc_port: u16,
    pub rpc_secret: String,
    pub max_overall_download_limit: String,
    pub max_overall_upload_limit: String,
    pub allow_overwrite: bool,
    pub auto_file_renaming: bool,
    pub continue_download: bool,
    pub follow_metalink: String,

    // Proxy settings
    pub proxy_enabled: bool,
    pub proxy_type: ProxyType,
    pub proxy_host: String,
    pub proxy_port: u16,
    pub proxy_username: String,
    pub proxy_password: String,
    pub no_proxy: String,

    // Tracker auto-sync
    pub last_tracker_update: u64,

    // Behavior settings
    pub keep_window_state: bool,
    pub new_task_show_downloading: bool,
    pub no_confirm_before_delete_task: bool,
    pub auto_check_update: bool,
    pub last_check_update_time: u64,

    // Protocol handlers
    pub default_magnet_client: bool,
    pub default_thunder_client: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Auto,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    #[default]
    Http,
    Https,
    Socks5,
}

impl Default for AppConfig {
    fn default() -> Self {
        let download_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("."));

        Self {
            locale: "en".to_string(),
            theme: Theme::Auto,
            download_dir,
            auto_start: false,
            start_hidden: false,
            hide_on_close: true,
            notify_on_complete: true,
            auto_clear_completed: false,
            resume_all_when_app_launched: true,

            max_concurrent_downloads: 10,
            max_connection_per_server: 16,
            split: 16,
            min_split_size: "1M".to_string(),
            max_download_limit: "0".to_string(),
            max_upload_limit: "0".to_string(),

            bt_listen_port: 21301,
            dht_listen_port: 21302,
            enable_upnp: true,
            seed_ratio: 1.0,
            seed_time: 60,
            bt_tracker: String::new(),
            tracker_source: vec![
                "https://raw.githubusercontent.com/ngosang/trackerslist/master/trackers_all.txt".to_string(),
                "https://raw.githubusercontent.com/XIU2/TrackersListCollection/master/all.txt".to_string(),
                "https://raw.githubusercontent.com/DeSireFire/animeTrackerList/master/AT_all.txt".to_string(),
            ],
            bt_force_encryption: false,
            bt_require_crypto: false,
            pause_metadata: false,

            bt_save_metadata: true,
            bt_load_saved_metadata: true,
            bt_remove_unselected_file: false,
            bt_detach_seed_only: false,

            user_agent: format!("Motrix/{}", env!("CARGO_PKG_VERSION")),
            rpc_port: 16800,
            rpc_secret: uuid::Uuid::new_v4().to_string(),
            max_overall_download_limit: "0".to_string(),
            max_overall_upload_limit: "0".to_string(),
            allow_overwrite: false,
            auto_file_renaming: true,
            continue_download: true,
            follow_metalink: "true".to_string(),

            proxy_enabled: false,
            proxy_type: ProxyType::Http,
            proxy_host: String::new(),
            proxy_port: 1080,
            proxy_username: String::new(),
            proxy_password: String::new(),
            no_proxy: String::new(),

            last_tracker_update: 0,

            keep_window_state: true,
            new_task_show_downloading: true,
            no_confirm_before_delete_task: false,
            auto_check_update: true,
            last_check_update_time: 0,

            default_magnet_client: false,
            default_thunder_client: false,
        }
    }
}

impl AppConfig {
    /// Load config from the Tauri store, persisting defaults on first launch.
    /// Consolidates the config-loading pattern used across multiple modules.
    pub fn load_from_store(store: &tauri_plugin_store::Store<tauri::Wry>) -> Self {
        if let Some(config_val) = store.get("config") {
            serde_json::from_value(config_val.clone()).unwrap_or_else(|e| {
                tracing::warn!("Failed to deserialize config, regenerating defaults: {}", e);
                let default_config = Self::default();
                store.set("config", serde_json::to_value(&default_config).expect("AppConfig serialization cannot fail"));
                let _ = store.save();
                default_config
            })
        } else {
            let default_config = Self::default();
            store.set("config", serde_json::to_value(&default_config).expect("AppConfig serialization cannot fail"));
            let _ = store.save();
            default_config
        }
    }

    /// Convert to aria2 command line arguments
    pub fn to_aria2_args(&self) -> Vec<String> {
        let mut args = vec![
            format!("--dir={}", self.download_dir.display()),
            format!("--max-concurrent-downloads={}", self.max_concurrent_downloads),
            format!("--max-connection-per-server={}", self.max_connection_per_server),
            format!("--split={}", self.split),
            format!("--min-split-size={}", self.min_split_size),
            format!("--max-download-limit={}", self.max_download_limit),
            format!("--max-upload-limit={}", self.max_upload_limit),
            format!("--listen-port={}", self.bt_listen_port),
            format!("--dht-listen-port={}", self.dht_listen_port),
            format!("--seed-ratio={}", self.seed_ratio),
            format!("--seed-time={}", self.seed_time),
            format!("--user-agent={}", self.user_agent),
            format!("--rpc-listen-port={}", self.rpc_port),
            // Note: rpc-secret is passed via conf-path to avoid exposure in process list
            "--enable-rpc=true".to_string(),
            "--rpc-listen-all=false".to_string(),
            "--rpc-allow-origin-all=true".to_string(),
            "--enable-dht=true".to_string(),
            "--enable-dht6=true".to_string(),
            "--enable-peer-exchange=true".to_string(),
            "--bt-enable-lpd=true".to_string(),
            "--follow-torrent=false".to_string(),
            "--check-certificate=true".to_string(),
            format!("--max-overall-download-limit={}", self.max_overall_download_limit),
            format!("--max-overall-upload-limit={}", self.max_overall_upload_limit),
            format!("--allow-overwrite={}", self.allow_overwrite),
            format!("--auto-file-renaming={}", self.auto_file_renaming),
            format!("--continue={}", self.continue_download),
            format!("--bt-force-encryption={}", self.bt_force_encryption),
            format!("--bt-require-crypto={}", self.bt_require_crypto),
            format!("--pause-metadata={}", self.pause_metadata),
            format!("--bt-save-metadata={}", self.bt_save_metadata),
            format!("--bt-load-saved-metadata={}", self.bt_load_saved_metadata),
            format!("--bt-remove-unselected-file={}", self.bt_remove_unselected_file),
            format!("--bt-detach-seed-only={}", self.bt_detach_seed_only),
            format!("--follow-metalink={}", self.follow_metalink),
            format!("--enable-upnp={}", self.enable_upnp),
        ];

        // Add proxy settings if enabled
        if self.proxy_enabled && !self.proxy_host.is_empty() {
            let proxy_url = match self.proxy_type {
                ProxyType::Http => format!("http://{}:{}", self.proxy_host, self.proxy_port),
                ProxyType::Https => format!("https://{}:{}", self.proxy_host, self.proxy_port),
                ProxyType::Socks5 => format!("socks5://{}:{}", self.proxy_host, self.proxy_port),
            };
            args.push(format!("--all-proxy={}", proxy_url));

            if !self.proxy_username.is_empty() {
                args.push(format!("--all-proxy-user={}", self.proxy_username));
            }
            // Note: proxy password is passed via conf-path to avoid exposure in process list
            if !self.no_proxy.is_empty() {
                args.push(format!("--no-proxy={}", self.no_proxy));
            }
        }

        args
    }

    /// Convert config entries that should be persisted in aria2.conf
    pub fn to_aria2_conf_lines(&self) -> Vec<String> {
        let mut lines = vec![format!("rpc-secret={}", self.rpc_secret)];

        if self.proxy_enabled && !self.proxy_password.is_empty() {
            lines.push(format!("all-proxy-passwd={}", self.proxy_password));
        }

        // Tracker lists can become very large; keep them out of Windows spawn args.
        if !self.bt_tracker.is_empty() {
            lines.push(format!("bt-tracker={}", self.bt_tracker));
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::AppConfig;

    #[test]
    fn bt_tracker_is_not_passed_via_cli_args() {
        let mut config = AppConfig::default();
        config.bt_tracker = (0..512)
            .map(|index| format!("udp://tracker{index}.example.com:6969/announce"))
            .collect::<Vec<_>>()
            .join(",");

        let args = config.to_aria2_args();

        assert!(
            args.iter().all(|arg| !arg.starts_with("--bt-tracker=")),
            "bt-tracker should be written to aria2.conf instead of CLI args"
        );
    }

    #[test]
    fn bt_tracker_is_written_to_aria2_conf() {
        let mut config = AppConfig::default();
        config.rpc_secret = "secret".to_string();
        config.bt_tracker = "udp://tracker.example.com:6969/announce".to_string();

        let lines = config.to_aria2_conf_lines();

        assert!(
            lines
                .iter()
                .any(|line| line == "bt-tracker=udp://tracker.example.com:6969/announce"),
            "bt-tracker should be persisted in aria2.conf"
        );
    }
}
