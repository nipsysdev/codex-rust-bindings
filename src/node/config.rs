//! Node configuration structures for Codex

use crate::error::{CodexError, Result};
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;

/// Serialize u64 as string for compatibility with C library
/// Note: storage-quota must be a string due to C library requirements
fn serialize_u64_as_string<S>(
    value: &Option<u64>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(v) => serializer.serialize_str(&v.to_string()),
        None => serializer.serialize_none(),
    }
}

/// Deserialize u64 from string for compatibility with C library
fn deserialize_u64_from_string<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;
    use std::str::FromStr;

    struct U64StringVisitor;

    impl<'de> Visitor<'de> for U64StringVisitor {
        type Value = Option<u64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing a u64 or null")
        }

        fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            u64::from_str(value)
                .map(Some)
                .map_err(|_| E::custom(format!("invalid u64 string: {}", value)))
        }

        fn visit_none<E>(self) -> std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_str(self)
        }
    }

    deserializer.deserialize_option(U64StringVisitor)
}

/// Log level for the Codex node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Notice,
    Warn,
    Error,
    Fatal,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Notice => write!(f, "notice"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
            LogLevel::Fatal => write!(f, "fatal"),
        }
    }
}

/// Log format for the Codex node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Auto,
    Colors,
    NoColors,
    Json,
}

impl Default for LogFormat {
    fn default() -> Self {
        LogFormat::Auto
    }
}

impl std::fmt::Display for LogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogFormat::Auto => write!(f, "auto"),
            LogFormat::Colors => write!(f, "colors"),
            LogFormat::NoColors => write!(f, "nocolors"),
            LogFormat::Json => write!(f, "json"),
        }
    }
}

/// Repository kind for storage backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepoKind {
    Fs,
    Sqlite,
    LevelDb,
}

impl Default for RepoKind {
    fn default() -> Self {
        RepoKind::Fs
    }
}

impl std::fmt::Display for RepoKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoKind::Fs => write!(f, "fs"),
            RepoKind::Sqlite => write!(f, "sqlite"),
            RepoKind::LevelDb => write!(f, "leveldb"),
        }
    }
}

/// Configuration for a Codex node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexConfig {
    /// Log level (default: INFO)
    #[serde(rename = "log-level", default, skip_serializing_if = "Option::is_none")]
    pub log_level: Option<LogLevel>,

    /// Log format (default: auto)
    #[serde(
        rename = "log-format",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub log_format: Option<LogFormat>,

    /// Enable the metrics server (default: false)
    #[serde(rename = "metrics", default, skip_serializing_if = "Option::is_none")]
    pub metrics_enabled: Option<bool>,

    /// Listening address of the metrics server (default: 127.0.0.1)
    #[serde(
        rename = "metrics-address",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub metrics_address: Option<String>,

    /// Listening HTTP port of the metrics server (default: 8008)
    #[serde(
        rename = "metrics-port",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub metrics_port: Option<u16>,

    /// The directory where codex will store configuration and data
    #[serde(rename = "data-dir", default, skip_serializing_if = "Option::is_none")]
    pub data_dir: Option<PathBuf>,

    /// Multi Addresses to listen on (default: ["/ip4/0.0.0.0/tcp/0"])
    #[serde(
        rename = "listen-addrs",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub listen_addrs: Vec<String>,

    /// Specify method to use for determining public address
    #[serde(rename = "nat", default, skip_serializing_if = "Option::is_none")]
    pub nat: Option<String>,

    /// Discovery (UDP) port (default: 8090)
    #[serde(rename = "disc-port", default, skip_serializing_if = "Option::is_none")]
    pub discovery_port: Option<u16>,

    /// Source of network (secp256k1) private key file path or name (default: "key")
    #[serde(
        rename = "net-privkey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub net_priv_key_file: Option<PathBuf>,

    /// Specifies one or more bootstrap nodes to use when connecting to the network
    #[serde(
        rename = "bootstrap-node",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub bootstrap_nodes: Vec<String>,

    /// The maximum number of peers to connect to (default: 160)
    #[serde(rename = "max-peers", default, skip_serializing_if = "Option::is_none")]
    pub max_peers: Option<u32>,

    /// Number of worker threads (default: 0 = use as many threads as there are CPU cores available)
    #[serde(
        rename = "num-threads",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub num_threads: Option<u32>,

    /// Node agent string which is used as identifier in network (default: "Codex")
    #[serde(
        rename = "agent-string",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub agent_string: Option<String>,

    /// Backend for main repo store (fs, sqlite, leveldb) (default: fs)
    #[serde(rename = "repo-kind", default, skip_serializing_if = "Option::is_none")]
    pub repo_kind: Option<RepoKind>,

    /// The size of the total storage quota dedicated to the node (default: 20 GiBs)
    #[serde(
        rename = "storage-quota",
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string"
    )]
    pub storage_quota: Option<u64>,

    /// Default block timeout in seconds - 0 disables the ttl (default: 30 days)
    #[serde(rename = "block-ttl", default, skip_serializing_if = "Option::is_none")]
    pub block_ttl: Option<u32>,

    /// Time interval in seconds - determines frequency of block maintenance cycle (default: 10 minutes)
    #[serde(rename = "block-mi", default, skip_serializing_if = "Option::is_none")]
    pub block_maintenance_interval: Option<u32>,

    /// Number of blocks to check every maintenance cycle (default: 1000)
    #[serde(rename = "block-mn", default, skip_serializing_if = "Option::is_none")]
    pub block_maintenance_number_of_blocks: Option<u32>,

    /// Number of times to retry fetching a block before giving up (default: 3000)
    #[serde(
        rename = "block-retries",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub block_retries: Option<u32>,

    /// The size of the block cache, 0 disables the cache (default: 0)
    #[serde(
        rename = "cache-size",
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string"
    )]
    pub cache_size: Option<u64>,

    /// Log file path (default: "" - no log file)
    #[serde(rename = "log-file", default, skip_serializing_if = "Option::is_none")]
    pub log_file: Option<PathBuf>,
}

impl Default for CodexConfig {
    fn default() -> Self {
        Self {
            log_level: Some(LogLevel::Info),
            log_format: Some(LogFormat::Auto),
            metrics_enabled: Some(false),
            metrics_address: Some("127.0.0.1".to_string()),
            metrics_port: Some(8008),
            data_dir: None,
            listen_addrs: vec!["/ip4/0.0.0.0/tcp/0".to_string()],
            nat: Some("any".to_string()),
            discovery_port: Some(8090),
            net_priv_key_file: None,
            bootstrap_nodes: vec![],
            max_peers: Some(160),
            num_threads: Some(0),
            agent_string: Some("Codex".to_string()),
            repo_kind: Some(RepoKind::Fs),
            storage_quota: Some(20 * 1024 * 1024 * 1024), // 20 GiB
            block_ttl: Some(30 * 24 * 60 * 60),           // 30 days in seconds
            block_maintenance_interval: Some(10 * 60),    // 10 minutes in seconds
            block_maintenance_number_of_blocks: Some(1000),
            block_retries: Some(3000),
            cache_size: Some(0),
            log_file: None,
        }
    }
}

impl CodexConfig {
    /// Create a new configuration with minimal values (compatible with C library)
    pub fn new() -> Self {
        Self {
            log_level: Some(LogLevel::Info),
            log_format: None,
            metrics_enabled: None,
            metrics_address: None,
            metrics_port: None,
            data_dir: None,
            listen_addrs: vec![],
            nat: None,
            discovery_port: None,
            net_priv_key_file: None,
            bootstrap_nodes: vec![],
            max_peers: None,
            num_threads: None,
            agent_string: None,
            repo_kind: None,
            storage_quota: None,
            block_ttl: None,
            block_maintenance_interval: None,
            block_maintenance_number_of_blocks: None,
            block_retries: None,
            cache_size: None,
            log_file: None,
        }
    }

    /// Create a new configuration with all default values (for advanced use)
    pub fn with_defaults() -> Self {
        Self::default()
    }

    /// Set the log level
    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.log_level = Some(level);
        self
    }

    /// Set the data directory
    pub fn data_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.data_dir = Some(path.into());
        self
    }

    /// Set the storage quota in bytes
    pub fn storage_quota(mut self, quota: u64) -> Self {
        self.storage_quota = Some(quota);
        self
    }

    /// Add a bootstrap node
    pub fn add_bootstrap_node<S: Into<String>>(mut self, node: S) -> Self {
        self.bootstrap_nodes.push(node.into());
        self
    }

    /// Set the maximum number of peers
    pub fn max_peers(mut self, max: u32) -> Self {
        self.max_peers = Some(max);
        self
    }

    /// Set the repository kind
    pub fn repo_kind(mut self, kind: RepoKind) -> Self {
        self.repo_kind = Some(kind);
        self
    }

    /// Set the discovery port
    pub fn discovery_port(mut self, port: u16) -> Self {
        self.discovery_port = Some(port);
        self
    }

    /// Set the listen addresses
    pub fn listen_addrs(mut self, addrs: Vec<String>) -> Self {
        self.listen_addrs = addrs;
        self
    }

    /// Add a listen address
    pub fn add_listen_addr<S: Into<String>>(mut self, addr: S) -> Self {
        self.listen_addrs.push(addr.into());
        self
    }

    /// Set the log format
    pub fn log_format(mut self, format: LogFormat) -> Self {
        self.log_format = Some(format);
        self
    }

    /// Enable metrics server
    pub fn enable_metrics(mut self, enabled: bool) -> Self {
        self.metrics_enabled = Some(enabled);
        self
    }

    /// Set the metrics server address
    pub fn metrics_address<S: Into<String>>(mut self, addr: S) -> Self {
        self.metrics_address = Some(addr.into());
        self
    }

    /// Set the metrics server port
    pub fn metrics_port(mut self, port: u16) -> Self {
        self.metrics_port = Some(port);
        self
    }

    /// Set the NAT configuration
    pub fn nat<S: Into<String>>(mut self, nat: S) -> Self {
        self.nat = Some(nat.into());
        self
    }

    /// Set the network private key file path
    pub fn net_priv_key_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.net_priv_key_file = Some(path.into());
        self
    }

    /// Set the number of worker threads
    pub fn num_threads(mut self, num: u32) -> Self {
        self.num_threads = Some(num);
        self
    }

    /// Set the agent string
    pub fn agent_string<S: Into<String>>(mut self, agent: S) -> Self {
        self.agent_string = Some(agent.into());
        self
    }

    /// Set the block timeout in seconds
    pub fn block_ttl(mut self, ttl: u32) -> Self {
        self.block_ttl = Some(ttl);
        self
    }

    /// Set the block maintenance interval in seconds
    pub fn block_maintenance_interval(mut self, interval: u32) -> Self {
        self.block_maintenance_interval = Some(interval);
        self
    }

    /// Set the block maintenance number of blocks
    pub fn block_maintenance_number_of_blocks(mut self, num: u32) -> Self {
        self.block_maintenance_number_of_blocks = Some(num);
        self
    }

    /// Set the block retries
    pub fn block_retries(mut self, retries: u32) -> Self {
        self.block_retries = Some(retries);
        self
    }

    /// Set the cache size
    pub fn cache_size(mut self, size: u64) -> Self {
        self.cache_size = Some(size);
        self
    }

    /// Set the log file path
    pub fn log_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.log_file = Some(path.into());
        self
    }

    /// Convert the configuration to a JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(CodexError::from)
    }

    /// Create a configuration from a JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(CodexError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CodexConfig::default();
        assert_eq!(config.log_level, Some(LogLevel::Info));
        assert_eq!(config.log_format, Some(LogFormat::Auto));
        assert_eq!(config.metrics_enabled, Some(false));
        assert_eq!(config.max_peers, Some(160));
    }

    #[test]
    fn test_config_builder() {
        let config = CodexConfig::new()
            .log_level(LogLevel::Debug)
            .data_dir("/tmp/codex")
            .storage_quota(1024 * 1024) // 1 MB
            .max_peers(100)
            .repo_kind(RepoKind::Sqlite);

        assert_eq!(config.log_level, Some(LogLevel::Debug));
        assert_eq!(config.data_dir, Some(PathBuf::from("/tmp/codex")));
        assert_eq!(config.storage_quota, Some(1024 * 1024));
        assert_eq!(config.max_peers, Some(100));
        assert_eq!(config.repo_kind, Some(RepoKind::Sqlite));
    }

    #[test]
    fn test_json_serialization_minimal_config() {
        let config = CodexConfig::new();
        let json_str = config.to_json().expect("Failed to serialize to JSON");

        // Verify the JSON is valid
        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("Serialized JSON should be valid");

        // Minimal config - should only have log-level
        assert!(parsed.get("log-level").is_some());
        assert!(parsed.get("listen-addrs").is_none()); // Empty vector should be skipped
        assert!(parsed.get("bootstrap-node").is_none()); // Empty vector should be skipped
    }

    #[test]
    fn test_json_serialization_partial_config() {
        let config = CodexConfig::new().log_level(LogLevel::Debug);
        let json_str = config.to_json().expect("Failed to serialize to JSON");

        // Verify the JSON is valid
        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("Serialized JSON should be valid");

        // Config with log level
        assert_eq!(parsed["log-level"], "debug");
    }

    #[test]
    fn test_json_serialization_full_config() {
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir("/tmp/codex")
            .storage_quota(1024 * 1024)
            .max_peers(50)
            .add_listen_addr("/ip4/127.0.0.1/tcp/8080")
            .add_bootstrap_node("/ip4/127.0.0.1/tcp/8081");

        let json_str = config.to_json().expect("Failed to serialize to JSON");

        // Verify the JSON is valid
        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("Serialized JSON should be valid");

        // Full config
        assert_eq!(parsed["log-level"], "error");
        assert_eq!(parsed["data-dir"], "/tmp/codex");
        assert_eq!(parsed["storage-quota"], "1048576"); // Should be string
        assert_eq!(parsed["max-peers"], 50);
        assert!(parsed["listen-addrs"].is_array());
        assert!(parsed["bootstrap-node"].is_array());
    }

    #[test]
    fn test_json_deserialization_minimal() {
        let json_str = r#"{"log-level":"info"}"#;
        let config = CodexConfig::from_json(json_str).expect("Failed to deserialize from JSON");

        // Minimal JSON
        assert_eq!(config.log_level, Some(LogLevel::Info));
        assert_eq!(config.listen_addrs, Vec::<String>::new()); // Default empty
        assert_eq!(config.bootstrap_nodes, Vec::<String>::new()); // Default empty
    }

    #[test]
    fn test_json_deserialization_with_empty_vectors() {
        let json_str = r#"{"log-level":"debug","listen-addrs":[],"bootstrap-node":[]}"#;
        let config = CodexConfig::from_json(json_str).expect("Failed to deserialize from JSON");

        // JSON with empty vectors
        assert_eq!(config.log_level, Some(LogLevel::Debug));
        assert_eq!(config.listen_addrs, Vec::<String>::new());
        assert_eq!(config.bootstrap_nodes, Vec::<String>::new());
    }

    #[test]
    fn test_json_deserialization_full_config() {
        let json_str = r#"{
            "log-level":"error",
            "log-format":"json",
            "metrics":true,
            "metrics-address":"192.168.1.100",
            "metrics-port":9000,
            "data-dir":"/tmp/codex",
            "listen-addrs":["/ip4/127.0.0.1/tcp/8080"],
            "nat":"any",
            "disc-port":8090,
            "bootstrap-node":["/ip4/127.0.0.1/tcp/8081"],
            "max-peers":100,
            "num-threads":4,
            "agent-string":"TestAgent/1.0",
            "repo-kind":"sqlite",
            "storage-quota":"2147483648",
            "block-ttl":86400,
            "block-mi":600,
            "block-mn":500,
            "block-retries":1000,
            "cache-size":"1048576",
            "log-file":"/var/log/codex.log"
        }"#;

        let config = CodexConfig::from_json(json_str).expect("Failed to deserialize from JSON");

        // Full JSON
        assert_eq!(config.log_level, Some(LogLevel::Error));
        assert_eq!(config.log_format, Some(LogFormat::Json));
        assert_eq!(config.metrics_enabled, Some(true));
        assert_eq!(config.metrics_address, Some("192.168.1.100".to_string()));
        assert_eq!(config.metrics_port, Some(9000));
        assert_eq!(config.data_dir, Some(PathBuf::from("/tmp/codex")));
        assert_eq!(config.listen_addrs, vec!["/ip4/127.0.0.1/tcp/8080"]);
        assert_eq!(config.nat, Some("any".to_string()));
        assert_eq!(config.discovery_port, Some(8090));
        assert_eq!(config.bootstrap_nodes, vec!["/ip4/127.0.0.1/tcp/8081"]);
        assert_eq!(config.max_peers, Some(100));
        assert_eq!(config.num_threads, Some(4));
        assert_eq!(config.agent_string, Some("TestAgent/1.0".to_string()));
        assert_eq!(config.repo_kind, Some(RepoKind::Sqlite));
        assert_eq!(config.storage_quota, Some(2147483648));
        assert_eq!(config.block_ttl, Some(86400));
        assert_eq!(config.block_maintenance_interval, Some(600));
        assert_eq!(config.block_maintenance_number_of_blocks, Some(500));
        assert_eq!(config.block_retries, Some(1000));
        assert_eq!(config.cache_size, Some(1048576));
        assert_eq!(config.log_file, Some(PathBuf::from("/var/log/codex.log")));
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Debug.to_string(), "debug");
        assert_eq!(LogLevel::Error.to_string(), "error");
    }

    #[test]
    fn test_log_format_display() {
        assert_eq!(LogFormat::Auto.to_string(), "auto");
        assert_eq!(LogFormat::Colors.to_string(), "colors");
        assert_eq!(LogFormat::Json.to_string(), "json");
    }

    #[test]
    fn test_repo_kind_display() {
        assert_eq!(RepoKind::Fs.to_string(), "fs");
        assert_eq!(RepoKind::Sqlite.to_string(), "sqlite");
        assert_eq!(RepoKind::LevelDb.to_string(), "leveldb");
    }

    #[test]
    fn test_listen_addrs_builder() {
        let config = CodexConfig::new().listen_addrs(vec![
            "/ip4/127.0.0.1/tcp/8080".to_string(),
            "/ip4/0.0.0.0/tcp/8080".to_string(),
        ]);

        assert_eq!(config.listen_addrs.len(), 2);
        assert_eq!(config.listen_addrs[0], "/ip4/127.0.0.1/tcp/8080");
        assert_eq!(config.listen_addrs[1], "/ip4/0.0.0.0/tcp/8080");
    }

    #[test]
    fn test_add_listen_addr_builder() {
        let config = CodexConfig::new()
            .add_listen_addr("/ip4/127.0.0.1/tcp/8080")
            .add_listen_addr("/ip4/0.0.0.0/tcp/8080");

        assert_eq!(config.listen_addrs.len(), 2);
        assert_eq!(config.listen_addrs[0], "/ip4/127.0.0.1/tcp/8080");
        assert_eq!(config.listen_addrs[1], "/ip4/0.0.0.0/tcp/8080");
    }

    #[test]
    fn test_log_format_builder() {
        let config = CodexConfig::new().log_format(LogFormat::Json);
        assert_eq!(config.log_format, Some(LogFormat::Json));
    }

    #[test]
    fn test_metrics_builder() {
        let config = CodexConfig::new()
            .enable_metrics(true)
            .metrics_address("192.168.1.100")
            .metrics_port(9000);

        assert_eq!(config.metrics_enabled, Some(true));
        assert_eq!(config.metrics_address, Some("192.168.1.100".to_string()));
        assert_eq!(config.metrics_port, Some(9000));
    }

    #[test]
    fn test_nat_builder() {
        let config = CodexConfig::new().nat("any");
        assert_eq!(config.nat, Some("any".to_string()));
    }

    #[test]
    fn test_net_priv_key_file_builder() {
        let config = CodexConfig::new().net_priv_key_file("/path/to/key");
        assert_eq!(
            config.net_priv_key_file,
            Some(PathBuf::from("/path/to/key"))
        );
    }

    #[test]
    fn test_num_threads_builder() {
        let config = CodexConfig::new().num_threads(4);
        assert_eq!(config.num_threads, Some(4));
    }

    #[test]
    fn test_agent_string_builder() {
        let config = CodexConfig::new().agent_string("CustomAgent/1.0");
        assert_eq!(config.agent_string, Some("CustomAgent/1.0".to_string()));
    }

    #[test]
    fn test_block_config_builders() {
        let config = CodexConfig::new()
            .block_ttl(86400) // 1 day
            .block_maintenance_interval(600) // 10 minutes
            .block_maintenance_number_of_blocks(500)
            .block_retries(1000);

        assert_eq!(config.block_ttl, Some(86400));
        assert_eq!(config.block_maintenance_interval, Some(600));
        assert_eq!(config.block_maintenance_number_of_blocks, Some(500));
        assert_eq!(config.block_retries, Some(1000));
    }

    #[test]
    fn test_cache_size_builder() {
        let config = CodexConfig::new().cache_size(1024 * 1024); // 1 MB
        assert_eq!(config.cache_size, Some(1024 * 1024));
    }

    #[test]
    fn test_log_file_builder() {
        let config = CodexConfig::new().log_file("/var/log/codex.log");
        assert_eq!(config.log_file, Some(PathBuf::from("/var/log/codex.log")));
    }

    #[test]
    fn test_comprehensive_builder() {
        let config = CodexConfig::new()
            .log_level(LogLevel::Debug)
            .log_format(LogFormat::Json)
            .data_dir("/tmp/codex")
            .listen_addrs(vec!["/ip4/127.0.0.1/tcp/8080".to_string()])
            .enable_metrics(true)
            .metrics_address("127.0.0.1")
            .metrics_port(8080)
            .discovery_port(8090)
            .max_peers(50)
            .storage_quota(1024 * 1024 * 1024) // 1 GB
            .repo_kind(RepoKind::Sqlite)
            .nat("any")
            .agent_string("TestAgent/1.0")
            .block_ttl(86400)
            .cache_size(1024 * 1024);

        assert_eq!(config.log_level, Some(LogLevel::Debug));
        assert_eq!(config.log_format, Some(LogFormat::Json));
        assert_eq!(config.data_dir, Some(PathBuf::from("/tmp/codex")));
        assert_eq!(config.listen_addrs.len(), 1);
        assert_eq!(config.listen_addrs[0], "/ip4/127.0.0.1/tcp/8080");
        assert_eq!(config.metrics_enabled, Some(true));
        assert_eq!(config.metrics_address, Some("127.0.0.1".to_string()));
        assert_eq!(config.metrics_port, Some(8080));
        assert_eq!(config.discovery_port, Some(8090));
        assert_eq!(config.max_peers, Some(50));
        assert_eq!(config.storage_quota, Some(1024 * 1024 * 1024));
        assert_eq!(config.repo_kind, Some(RepoKind::Sqlite));
        assert_eq!(config.nat, Some("any".to_string()));
        assert_eq!(config.agent_string, Some("TestAgent/1.0".to_string()));
        assert_eq!(config.block_ttl, Some(86400));
        assert_eq!(config.cache_size, Some(1024 * 1024));
    }
}
