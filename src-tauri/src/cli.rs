//! Command line argument parsing utilities

/// Parse argv for downloadable URLs and torrent file paths
pub fn parse_args(argv: &[String]) -> Vec<String> {
    argv.iter()
        .skip(1) // Skip program name
        .filter(|arg| is_downloadable_url(arg) || is_torrent_file(arg))
        .cloned()
        .collect()
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
