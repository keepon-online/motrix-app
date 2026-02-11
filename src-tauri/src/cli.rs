//! Command line argument parsing utilities

use base64::Engine;

/// Parse argv for downloadable URLs and torrent file paths
pub fn parse_args(argv: &[String]) -> Vec<String> {
    argv.iter()
        .skip(1) // Skip program name
        .filter(|arg| is_downloadable_url(arg) || is_torrent_file(arg))
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
pub fn is_downloadable_url(s: &str) -> bool {
    let lower = s.to_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("ftp://")
        || lower.starts_with("magnet:")
        || lower.starts_with("thunder://")
        || lower.starts_with("motrix://")
}

/// Check if a string is a path to an existing .torrent file
pub fn is_torrent_file(s: &str) -> bool {
    s.to_lowercase().ends_with(".torrent")
        && std::path::Path::new(s).exists()
}
