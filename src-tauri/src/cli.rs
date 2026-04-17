//! Command line argument parsing utilities

use base64::Engine;

/// Parse argv for downloadable URLs and torrent file paths
pub fn parse_args(argv: &[String]) -> Vec<String> {
    argv.iter()
        .skip(1) // Skip program name
        .filter(|arg| is_downloadable_url(arg) || is_torrent_file(arg) || is_metalink_file(arg))
        .map(|arg| decode_thunder_url(arg))
        .collect()
}

/// Decode thunder:// URL to real download URL
/// thunder:// format: thunder://BASE64(AA<real_url>ZZ)
fn decode_thunder_url(url: &str) -> String {
    if !url.to_lowercase().starts_with("thunder://") {
        return url.to_string();
    }
    let encoded = &url[10..]; // Skip "thunder://"
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(encoded) {
        if let Ok(s) = String::from_utf8(decoded) {
            // thunder wraps URL with "AA" prefix and "ZZ" suffix
            let trimmed = s.strip_prefix("AA").unwrap_or(&s);
            let trimmed = trimmed.strip_suffix("ZZ").unwrap_or(trimmed);
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    // Return original if decode fails
    url.to_string()
}

/// Check if a string is a downloadable URL
fn is_downloadable_url(s: &str) -> bool {
    let lower = s.to_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("ftp://")
        || lower.starts_with("magnet:")
        || lower.starts_with("thunder://")
        || lower.starts_with("motrix://")
}

/// Check if a string is a path to an existing .torrent file
fn is_torrent_file(s: &str) -> bool {
    s.to_lowercase().ends_with(".torrent") && std::path::Path::new(s).exists()
}

/// Check if a string is a path to an existing metalink file
fn is_metalink_file(s: &str) -> bool {
    let lower = s.to_lowercase();
    (lower.ends_with(".metalink") || lower.ends_with(".meta4")) && std::path::Path::new(s).exists()
}

#[cfg(test)]
mod tests {
    use super::parse_args;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_temp_file(extension: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "motrix-cli-test-{}-{}.{}",
            std::process::id(),
            unique,
            extension
        ));
        fs::write(&path, b"test").expect("temp file should be created");
        path
    }

    #[test]
    fn parse_args_accepts_platform_native_torrent_file_paths() {
        let torrent = create_temp_file("torrent");
        let torrent_arg = torrent.to_string_lossy().to_string();
        let argv = vec!["motrix".to_string(), torrent_arg.clone()];

        let parsed = parse_args(&argv);

        assert_eq!(parsed, vec![torrent_arg]);

        fs::remove_file(torrent).expect("temp file should be removed");
    }

    #[test]
    fn parse_args_ignores_missing_torrent_file_paths() {
        let missing = std::env::temp_dir().join(format!(
            "motrix-cli-missing-{}-{}.torrent",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be after epoch")
                .as_nanos()
        ));
        let argv = vec!["motrix".to_string(), missing.to_string_lossy().to_string()];

        let parsed = parse_args(&argv);

        assert!(parsed.is_empty());
    }
}
