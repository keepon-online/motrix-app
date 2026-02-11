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
                "https://raw.githubusercontent.com/ngosang/trackerslist/master/trackers_best.txt".to_string(),
            ],
            bt_force_encryption: false,
            bt_require_crypto: false,
            pause_metadata: false,

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
        }
    }
}

impl AppConfig {
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
            format!("--rpc-secret={}", self.rpc_secret),
            "--enable-rpc=true".to_string(),
            "--rpc-listen-all=false".to_string(),
            "--rpc-allow-origin-all=true".to_string(),
            "--enable-dht=true".to_string(),
            "--enable-dht6=true".to_string(),
            format!("--enable-peer-exchange={}", self.enable_upnp),
            "--bt-enable-lpd=true".to_string(),
            "--follow-torrent=true".to_string(),
            "--check-certificate=false".to_string(),
            format!("--max-overall-download-limit={}", self.max_overall_download_limit),
            format!("--max-overall-upload-limit={}", self.max_overall_upload_limit),
            format!("--allow-overwrite={}", self.allow_overwrite),
            format!("--auto-file-renaming={}", self.auto_file_renaming),
            format!("--continue={}", self.continue_download),
            format!("--bt-force-encryption={}", self.bt_force_encryption),
            format!("--bt-require-crypto={}", self.bt_require_crypto),
            format!("--pause-metadata={}", self.pause_metadata),
            format!("--follow-metalink={}", self.follow_metalink),
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
            if !self.proxy_password.is_empty() {
                args.push(format!("--all-proxy-passwd={}", self.proxy_password));
            }
            if !self.no_proxy.is_empty() {
                args.push(format!("--no-proxy={}", self.no_proxy));
            }
        }

        // Add BT trackers if configured
        if !self.bt_tracker.is_empty() {
            args.push(format!("--bt-tracker={}", self.bt_tracker));
        }

        args
    }
}
