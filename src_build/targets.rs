/// Supported Rust target triples and their corresponding platform identifiers
///
/// This mapping defines which Rust target triples are supported and what
/// platform identifier to use when downloading prebuilt binaries.
///
/// To add support for a new platform:
/// 1. Add an entry to this mapping
/// 2. Ensure the corresponding prebuilt binary exists in the logos-storage-nim GitHub releases
pub const SUPPORTED_TARGETS: &[(&str, &str)] = &[
    ("x86_64-unknown-linux-gnu", "linux-amd64"),
    ("aarch64-unknown-linux-gnu", "linux-arm64"),
    ("aarch64-apple-darwin", "darwin-arm64"),
];

/// Returns a list of all supported target triples
pub fn supported_targets() -> Vec<&'static str> {
    SUPPORTED_TARGETS
        .iter()
        .map(|(target, _)| *target)
        .collect()
}

/// Maps a Rust target triple to its platform identifier
///
/// Returns `None` if the target is not supported
pub fn map_target_to_platform(target: &str) -> Option<&'static str> {
    SUPPORTED_TARGETS
        .iter()
        .find(|(t, _)| *t == target)
        .map(|(_, platform)| *platform)
}
