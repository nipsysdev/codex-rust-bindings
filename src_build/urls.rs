pub const GITHUB_REPO_OWNER: &str = "logos-storage";

pub const GITHUB_REPO_NAME: &str = "logos-storage-nim";

pub const GITHUB_API_BASE: &str = "https://api.github.com/repos";

/// User agent for HTTP requests
pub const USER_AGENT: &str = "storage-rust-bindings";

/// HTTP timeout for API requests (in seconds)
pub const API_TIMEOUT_SECONDS: u64 = 30;

/// HTTP timeout for file downloads (in seconds)
pub const DOWNLOAD_TIMEOUT_SECONDS: u64 = 900; // 15 minutes

/// Download buffer size in bytes
pub const DOWNLOAD_BUFFER_SIZE: usize = 8192; // 8KB

/// Constructs the GitHub API URL for the latest release
pub fn latest_release_url() -> String {
    format!(
        "{}/{}/{}/releases/latest",
        GITHUB_API_BASE, GITHUB_REPO_OWNER, GITHUB_REPO_NAME
    )
}

/// Constructs the GitHub API URL for a specific tagged release
pub fn tagged_release_url(version: &str) -> String {
    format!(
        "{}/{}/{}/releases/tags/{}",
        GITHUB_API_BASE, GITHUB_REPO_OWNER, GITHUB_REPO_NAME, version
    )
}
