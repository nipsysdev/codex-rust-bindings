//! Node configuration structures for Storage

use crate::error::{Result, StorageError};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Log level for the Storage node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Notice,
    Warn,
    Error,
    Fatal,
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

/// Log format for the Storage node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Auto,
    Colors,
    NoColors,
    Json,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RepoKind {
    #[default]
    Fs,
    Sqlite,
    LevelDb,
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

/// Network preset to connect to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkPreset {
    #[serde(rename = "logos.test")]
    LogosTest,
    #[serde(rename = "logos.dev")]
    LogosDev,
}

impl std::fmt::Display for NetworkPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkPreset::LogosTest => write!(f, "logos.test"),
            NetworkPreset::LogosDev => write!(f, "logos.dev"),
        }
    }
}

/// Configuration for a Storage node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
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

    /// The directory where storage will store configuration and data
    #[serde(rename = "data-dir", default, skip_serializing_if = "Option::is_none")]
    pub data_dir: Option<PathBuf>,

    /// IP address to listen on for remote peer connections (default: 0.0.0.0)
    #[serde(rename = "listen-ip", default, skip_serializing_if = "Option::is_none")]
    pub listen_ip: Option<String>,

    /// TCP port to listen on for remote peer connections (default: 0 = random)
    #[serde(
        rename = "listen-port",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub listen_port: Option<u16>,

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

    /// The network preset to connect to (overridden by bootstrap_nodes when set)
    #[serde(rename = "network", default, skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkPreset>,

    /// Specifies one or more bootstrap nodes to use when connecting to the network
    /// (when set, overrides the network preset)
    #[serde(
        rename = "bootstrap-node",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub bootstrap_nodes: Vec<String>,

    /// Do not bootstrap the node at all (default: false)
    #[serde(
        rename = "no-bootstrap-node",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub no_bootstrap_node: Option<bool>,

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

    /// Node agent string which is used as identifier in network (default: "Storage")
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
        skip_serializing_if = "Option::is_none"
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

    /// Log file path (default: "" - no log file)
    #[serde(rename = "log-file", default, skip_serializing_if = "Option::is_none")]
    pub log_file: Option<PathBuf>,

    /// Route DHT provider lookups through the Mix protocol via the dht_mix_proxies.
    /// Hides the requester's identity from the proxy (default: false)
    #[serde(
        rename = "mix-enabled",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mix_enabled: Option<bool>,

    /// Path to the Mix relay pool JSON file
    #[serde(rename = "mix-pool", default, skip_serializing_if = "Option::is_none")]
    pub mix_pool: Option<PathBuf>,

    /// Inline JSON content of the Mix relay pool (takes precedence over mix_pool)
    #[serde(
        rename = "mix-pool-json",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mix_pool_json: Option<String>,

    /// Peers used as dht-proxy destinations when Mix is enabled
    #[serde(
        rename = "dht-mix-proxy",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub dht_mix_proxies: Vec<String>,

    /// Max concurrent DHT proxy lookups handled by this node
    /// (omit to use the protocol default)
    #[serde(
        rename = "dht-proxy-max-inflight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub dht_proxy_max_in_flight: Option<u32>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            log_level: Some(LogLevel::Info),
            log_format: Some(LogFormat::Auto),
            metrics_enabled: Some(false),
            metrics_address: Some("127.0.0.1".to_string()),
            metrics_port: Some(8008),
            data_dir: None,
            listen_ip: None,
            listen_port: None,
            nat: Some("any".to_string()),
            discovery_port: Some(8090),
            net_priv_key_file: None,
            network: None,
            bootstrap_nodes: vec![],
            no_bootstrap_node: None,
            max_peers: Some(160),
            num_threads: Some(0),
            agent_string: Some("Storage".to_string()),
            repo_kind: Some(RepoKind::Fs),
            storage_quota: Some(20 * 1024 * 1024 * 1024), // 20 GiB
            block_ttl: Some(30 * 24 * 60 * 60),           // 30 days in seconds
            block_maintenance_interval: Some(10 * 60),    // 10 minutes in seconds
            block_maintenance_number_of_blocks: Some(1000),
            block_retries: Some(3000),
            log_file: None,
            mix_enabled: None,
            mix_pool: None,
            mix_pool_json: None,
            dht_mix_proxies: vec![],
            dht_proxy_max_in_flight: None,
        }
    }
}

impl StorageConfig {
    /// Create a new configuration with minimal values (compatible with C library)
    pub fn new() -> Self {
        Self {
            log_level: Some(LogLevel::Info),
            log_format: None,
            metrics_enabled: None,
            metrics_address: None,
            metrics_port: None,
            data_dir: None,
            listen_ip: None,
            listen_port: None,
            nat: None,
            discovery_port: None,
            net_priv_key_file: None,
            network: None,
            bootstrap_nodes: vec![],
            no_bootstrap_node: None,
            max_peers: None,
            num_threads: None,
            agent_string: None,
            repo_kind: None,
            storage_quota: None,
            block_ttl: None,
            block_maintenance_interval: None,
            block_maintenance_number_of_blocks: None,
            block_retries: None,
            log_file: None,
            mix_enabled: None,
            mix_pool: None,
            mix_pool_json: None,
            dht_mix_proxies: vec![],
            dht_proxy_max_in_flight: None,
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

    /// Set the listen IP address
    pub fn listen_ip<S: Into<String>>(mut self, ip: S) -> Self {
        self.listen_ip = Some(ip.into());
        self
    }

    /// Set the listen TCP port
    pub fn listen_port(mut self, port: u16) -> Self {
        self.listen_port = Some(port);
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

    /// Set the log file path
    pub fn log_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.log_file = Some(path.into());
        self
    }

    /// Set the network preset
    pub fn network(mut self, network: NetworkPreset) -> Self {
        self.network = Some(network);
        self
    }

    /// Do not bootstrap the node at all
    pub fn no_bootstrap_node(mut self, enabled: bool) -> Self {
        self.no_bootstrap_node = Some(enabled);
        self
    }

    /// Enable routing of DHT lookups through the Mix protocol
    pub fn mix_enabled(mut self, enabled: bool) -> Self {
        self.mix_enabled = Some(enabled);
        self
    }

    /// Set the path to the Mix relay pool JSON file
    pub fn mix_pool<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.mix_pool = Some(path.into());
        self
    }

    /// Set the inline JSON content of the Mix relay pool
    pub fn mix_pool_json<S: Into<String>>(mut self, json: S) -> Self {
        self.mix_pool_json = Some(json.into());
        self
    }

    /// Set the DHT mix proxy destinations
    pub fn dht_mix_proxies(mut self, proxies: Vec<String>) -> Self {
        self.dht_mix_proxies = proxies;
        self
    }

    /// Add a DHT mix proxy destination
    pub fn add_dht_mix_proxy<S: Into<String>>(mut self, proxy: S) -> Self {
        self.dht_mix_proxies.push(proxy.into());
        self
    }

    /// Set the max concurrent DHT proxy lookups handled by this node
    pub fn dht_proxy_max_in_flight(mut self, max: u32) -> Self {
        self.dht_proxy_max_in_flight = Some(max);
        self
    }

    /// Convert the configuration to a JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(StorageError::from)
    }

    /// Create a configuration from a JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(StorageError::from)
    }

    /// Create a configuration from environment variables
    ///
    /// Environment variables with the `STORAGE_` prefix are supported:
    ///
    /// - `STORAGE_DATA_DIR` - Data directory path
    /// - `STORAGE_LOG_LEVEL` - Log level (trace, debug, info, notice, warn, error, fatal)
    /// - `STORAGE_LOG_FORMAT` - Log format (auto, colors, nocolors, json)
    /// - `STORAGE_STORAGE_QUOTA` - Storage quota in bytes (supports suffixes: K, M, G, T)
    /// - `STORAGE_MAX_PEERS` - Maximum number of peers
    /// - `STORAGE_DISCOVERY_PORT` - Discovery port
    /// - `STORAGE_NUM_THREADS` - Number of worker threads
    /// - `STORAGE_REPO_KIND` - Repository kind (fs, sqlite, leveldb)
    /// - `STORAGE_NAT` - NAT configuration
    /// - `STORAGE_AGENT_STRING` - Agent string
    ///
    /// # Example
    ///
    /// ```bash
    /// export STORAGE_LOG_LEVEL=debug
    /// export STORAGE_DATA_DIR=/tmp/storage
    /// export STORAGE_STORAGE_QUOTA=1G
    /// ```
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // Read environment variables
        if let Ok(data_dir) = env::var("STORAGE_DATA_DIR") {
            config.data_dir = Some(PathBuf::from(data_dir));
        }

        if let Ok(log_level) = env::var("STORAGE_LOG_LEVEL") {
            config.log_level = Some(LogLevel::from_str(&log_level).map_err(|_| {
                StorageError::config_error(format!("Invalid log level: {}", log_level))
            })?);
        }

        if let Ok(log_format) = env::var("STORAGE_LOG_FORMAT") {
            config.log_format = Some(LogFormat::from_str(&log_format).map_err(|_| {
                StorageError::config_error(format!("Invalid log format: {}", log_format))
            })?);
        }

        if let Ok(storage_quota) = env::var("STORAGE_STORAGE_QUOTA") {
            config.storage_quota = Some(parse_bytes(&storage_quota).map_err(|e| {
                StorageError::config_error(format!("Invalid storage quota: {}", e))
            })?);
        }

        if let Ok(max_peers) = env::var("STORAGE_MAX_PEERS") {
            config.max_peers =
                Some(max_peers.parse().map_err(|e| {
                    StorageError::config_error(format!("Invalid max peers: {}", e))
                })?);
        }

        if let Ok(discovery_port) = env::var("STORAGE_DISCOVERY_PORT") {
            config.discovery_port = Some(discovery_port.parse().map_err(|e| {
                StorageError::config_error(format!("Invalid discovery port: {}", e))
            })?);
        }

        if let Ok(num_threads) = env::var("STORAGE_NUM_THREADS") {
            config.num_threads =
                Some(num_threads.parse().map_err(|e| {
                    StorageError::config_error(format!("Invalid num threads: {}", e))
                })?);
        }

        if let Ok(repo_kind) = env::var("STORAGE_REPO_KIND") {
            config.repo_kind = Some(RepoKind::from_str(&repo_kind).map_err(|_| {
                StorageError::config_error(format!("Invalid repo kind: {}", repo_kind))
            })?);
        }

        if let Ok(nat) = env::var("STORAGE_NAT") {
            config.nat = Some(nat);
        }

        if let Ok(agent_string) = env::var("STORAGE_AGENT_STRING") {
            config.agent_string = Some(agent_string);
        }

        Ok(config)
    }

    /// Merge configuration from environment variables
    ///
    /// Environment variables override the current configuration values.
    pub fn merge_with_env(mut self) -> Result<Self> {
        let env_config = Self::from_env()?;

        // Override with environment values (only if set)
        if env_config.data_dir.is_some() {
            self.data_dir = env_config.data_dir;
        }
        if env_config.log_level.is_some() {
            self.log_level = env_config.log_level;
        }
        if env_config.log_format.is_some() {
            self.log_format = env_config.log_format;
        }
        if env_config.storage_quota.is_some() {
            self.storage_quota = env_config.storage_quota;
        }
        if env_config.max_peers.is_some() {
            self.max_peers = env_config.max_peers;
        }
        if env_config.discovery_port.is_some() {
            self.discovery_port = env_config.discovery_port;
        }
        if env_config.num_threads.is_some() {
            self.num_threads = env_config.num_threads;
        }
        if env_config.repo_kind.is_some() {
            self.repo_kind = env_config.repo_kind;
        }
        if env_config.nat.is_some() {
            self.nat = env_config.nat;
        }
        if env_config.agent_string.is_some() {
            self.agent_string = env_config.agent_string;
        }

        Ok(self)
    }

    /// Merge configuration from a JSON file
    ///
    /// File values override the current configuration values.
    pub fn merge_with_file(mut self, path: &Path) -> Result<Self> {
        let file_content = std::fs::read_to_string(path).map_err(|e| {
            StorageError::config_error(format!("Failed to read config file: {}", e))
        })?;

        let file_config: StorageConfig = serde_json::from_str(&file_content).map_err(|e| {
            StorageError::config_error(format!("Failed to parse config file: {}", e))
        })?;

        // Override with file values (only if set)
        if file_config.data_dir.is_some() {
            self.data_dir = file_config.data_dir;
        }
        if file_config.log_level.is_some() {
            self.log_level = file_config.log_level;
        }
        if file_config.log_format.is_some() {
            self.log_format = file_config.log_format;
        }
        if file_config.storage_quota.is_some() {
            self.storage_quota = file_config.storage_quota;
        }
        if file_config.max_peers.is_some() {
            self.max_peers = file_config.max_peers;
        }
        if file_config.discovery_port.is_some() {
            self.discovery_port = file_config.discovery_port;
        }
        if file_config.num_threads.is_some() {
            self.num_threads = file_config.num_threads;
        }
        if file_config.repo_kind.is_some() {
            self.repo_kind = file_config.repo_kind;
        }
        if file_config.nat.is_some() {
            self.nat = file_config.nat;
        }
        if file_config.agent_string.is_some() {
            self.agent_string = file_config.agent_string;
        }

        Ok(self)
    }

    /// Merge configuration from CLI arguments
    ///
    /// CLI arguments override the current configuration values.
    pub fn merge_with_cli(mut self, args: &CliArgs) -> Result<Self> {
        if let Some(log_level) = &args.log_level {
            self.log_level = Some(LogLevel::from_str(log_level).map_err(|_| {
                StorageError::config_error(format!("Invalid log level: {}", log_level))
            })?);
        }
        if let Some(data_dir) = &args.data_dir {
            self.data_dir = Some(PathBuf::from(data_dir));
        }
        if let Some(storage_quota) = &args.storage_quota {
            self.storage_quota = Some(parse_bytes(storage_quota).map_err(|e| {
                StorageError::config_error(format!("Invalid storage quota: {}", e))
            })?);
        }
        if let Some(max_peers) = args.max_peers {
            self.max_peers = Some(max_peers);
        }
        if let Some(discovery_port) = args.discovery_port {
            self.discovery_port = Some(discovery_port);
        }

        Ok(self)
    }
}

/// CLI arguments for configuration
#[derive(Debug, Clone, Default)]
pub struct CliArgs {
    pub log_level: Option<String>,
    pub data_dir: Option<String>,
    pub storage_quota: Option<String>,
    pub max_peers: Option<u32>,
    pub discovery_port: Option<u16>,
}

/// Parse a byte string with optional suffix (K, M, G, T)
fn parse_bytes(s: &str) -> Result<u64> {
    let s = s.trim();
    let (num, suffix) = if let Some(pos) = s.find(|c: char| !c.is_ascii_digit()) {
        (&s[..pos], &s[pos..])
    } else {
        (s, "")
    };

    let base: u64 = num
        .parse()
        .map_err(|e| StorageError::config_error(format!("Invalid number: {}", e)))?;

    let multiplier = match suffix.to_uppercase().as_str() {
        "" | "B" => 1,
        "K" | "KB" => 1024,
        "M" | "MB" => 1024 * 1024,
        "G" | "GB" => 1024 * 1024 * 1024,
        "T" | "TB" => 1024 * 1024 * 1024 * 1024,
        _ => {
            return Err(StorageError::config_error(format!(
                "Invalid suffix: {}",
                suffix
            )))
        }
    };

    Ok(base * multiplier)
}

// Add FromStr implementations for enums
impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "notice" => Ok(LogLevel::Notice),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            "fatal" => Ok(LogLevel::Fatal),
            _ => Err(format!("Invalid log level: {}", s)),
        }
    }
}

impl FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(LogFormat::Auto),
            "colors" => Ok(LogFormat::Colors),
            "nocolors" => Ok(LogFormat::NoColors),
            "json" => Ok(LogFormat::Json),
            _ => Err(format!("Invalid log format: {}", s)),
        }
    }
}

impl FromStr for RepoKind {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fs" => Ok(RepoKind::Fs),
            "sqlite" => Ok(RepoKind::Sqlite),
            "leveldb" => Ok(RepoKind::LevelDb),
            _ => Err(format!("Invalid repo kind: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = StorageConfig::default();
        assert_eq!(config.log_level, Some(LogLevel::Info));
        assert_eq!(config.log_format, Some(LogFormat::Auto));
        assert_eq!(config.metrics_enabled, Some(false));
        assert_eq!(config.max_peers, Some(160));
    }

    #[test]
    fn test_config_builder() {
        let config = StorageConfig::new()
            .log_level(LogLevel::Debug)
            .data_dir("/tmp/storage")
            .storage_quota(1024 * 1024) // 1 MB
            .max_peers(100)
            .repo_kind(RepoKind::Sqlite);

        assert_eq!(config.log_level, Some(LogLevel::Debug));
        assert_eq!(config.data_dir, Some(PathBuf::from("/tmp/storage")));
        assert_eq!(config.storage_quota, Some(1024 * 1024));
        assert_eq!(config.max_peers, Some(100));
        assert_eq!(config.repo_kind, Some(RepoKind::Sqlite));
    }

    #[test]
    fn test_json_serialization_minimal_config() {
        let config = StorageConfig::new();
        let json_str = config.to_json().expect("Failed to serialize to JSON");

        // Verify the JSON is valid
        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("Serialized JSON should be valid");

        // Minimal config - should only have log-level
        assert!(parsed.get("log-level").is_some());
        assert!(parsed.get("listen-ip").is_none()); // None should be skipped
        assert!(parsed.get("bootstrap-node").is_none()); // Empty vector should be skipped
    }

    #[test]
    fn test_json_serialization_partial_config() {
        let config = StorageConfig::new().log_level(LogLevel::Debug);
        let json_str = config.to_json().expect("Failed to serialize to JSON");

        // Verify the JSON is valid
        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("Serialized JSON should be valid");

        // Config with log level
        assert_eq!(parsed["log-level"], "debug");
    }

    #[test]
    fn test_json_serialization_full_config() {
        let config = StorageConfig::new()
            .log_level(LogLevel::Error)
            .data_dir("/tmp/storage")
            .storage_quota(1024 * 1024)
            .max_peers(50)
            .listen_ip("/ip4/127.0.0.1/tcp/8080")
            .add_bootstrap_node("/ip4/127.0.0.1/tcp/8081");

        let json_str = config.to_json().expect("Failed to serialize to JSON");

        // Verify the JSON is valid
        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("Serialized JSON should be valid");

        // Full config
        assert_eq!(parsed["log-level"], "error");
        assert_eq!(parsed["data-dir"], "/tmp/storage");
        assert_eq!(parsed["storage-quota"], 1048576);
        assert_eq!(parsed["max-peers"], 50);
        assert_eq!(parsed["listen-ip"], "/ip4/127.0.0.1/tcp/8080");
        assert!(parsed["bootstrap-node"].is_array());
    }

    #[test]
    fn test_json_deserialization_minimal() {
        let json_str = r#"{"log-level":"info"}"#;
        let config = StorageConfig::from_json(json_str).expect("Failed to deserialize from JSON");

        // Minimal JSON
        assert_eq!(config.log_level, Some(LogLevel::Info));
        assert_eq!(config.listen_ip, None); // Default empty
        assert_eq!(config.bootstrap_nodes, Vec::<String>::new()); // Default empty
    }

    #[test]
    fn test_json_deserialization_with_empty_vectors() {
        let json_str = r#"{"log-level":"debug","dht-mix-proxy":[],"bootstrap-node":[]}"#;
        let config = StorageConfig::from_json(json_str).expect("Failed to deserialize from JSON");

        // JSON with empty vectors
        assert_eq!(config.log_level, Some(LogLevel::Debug));
        assert_eq!(config.dht_mix_proxies, Vec::<String>::new());
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
            "data-dir":"/tmp/storage",
            "listen-ip":"127.0.0.1",
            "listen-port":8080,
            "nat":"any",
            "disc-port":8090,
            "network":"logos.test",
            "bootstrap-node":["/ip4/127.0.0.1/tcp/8081"],
            "no-bootstrap-node":false,
            "max-peers":100,
            "num-threads":4,
            "agent-string":"TestAgent/1.0",
            "repo-kind":"sqlite",
            "storage-quota":2147483648,
            "block-ttl":86400,
            "block-mi":600,
            "block-mn":500,
            "block-retries":1000,
            "mix-enabled":true,
            "mix-pool-json":"{\"relays\":[]}",
            "dht-mix-proxy":["spr:proxy1"],
            "dht-proxy-max-inflight":8,
            "log-file":"/var/log/storage.log"
        }"#;

        let config = StorageConfig::from_json(json_str).expect("Failed to deserialize from JSON");

        // Full JSON
        assert_eq!(config.log_level, Some(LogLevel::Error));
        assert_eq!(config.log_format, Some(LogFormat::Json));
        assert_eq!(config.metrics_enabled, Some(true));
        assert_eq!(config.metrics_address, Some("192.168.1.100".to_string()));
        assert_eq!(config.metrics_port, Some(9000));
        assert_eq!(config.data_dir, Some(PathBuf::from("/tmp/storage")));
        assert_eq!(config.listen_ip, Some("127.0.0.1".to_string()));
        assert_eq!(config.listen_port, Some(8080));
        assert_eq!(config.nat, Some("any".to_string()));
        assert_eq!(config.discovery_port, Some(8090));
        assert_eq!(config.network, Some(NetworkPreset::LogosTest));
        assert_eq!(config.bootstrap_nodes, vec!["/ip4/127.0.0.1/tcp/8081"]);
        assert_eq!(config.no_bootstrap_node, Some(false));
        assert_eq!(config.max_peers, Some(100));
        assert_eq!(config.num_threads, Some(4));
        assert_eq!(config.agent_string, Some("TestAgent/1.0".to_string()));
        assert_eq!(config.repo_kind, Some(RepoKind::Sqlite));
        assert_eq!(config.storage_quota, Some(2147483648));
        assert_eq!(config.block_ttl, Some(86400));
        assert_eq!(config.block_maintenance_interval, Some(600));
        assert_eq!(config.block_maintenance_number_of_blocks, Some(500));
        assert_eq!(config.block_retries, Some(1000));
        assert_eq!(config.mix_enabled, Some(true));
        assert_eq!(config.mix_pool_json, Some("{\"relays\":[]}".to_string()));
        assert_eq!(config.dht_mix_proxies, vec!["spr:proxy1"]);
        assert_eq!(config.dht_proxy_max_in_flight, Some(8));
        assert_eq!(config.log_file, Some(PathBuf::from("/var/log/storage.log")));
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
    fn test_listen_ip_builder() {
        let config = StorageConfig::new().listen_ip("127.0.0.1");
        assert_eq!(config.listen_ip, Some("127.0.0.1".to_string()));
    }

    #[test]
    fn test_listen_port_builder() {
        let config = StorageConfig::new().listen_port(8080);
        assert_eq!(config.listen_port, Some(8080));
    }

    #[test]
    fn test_log_format_builder() {
        let config = StorageConfig::new().log_format(LogFormat::Json);
        assert_eq!(config.log_format, Some(LogFormat::Json));
    }

    #[test]
    fn test_metrics_builder() {
        let config = StorageConfig::new()
            .enable_metrics(true)
            .metrics_address("192.168.1.100")
            .metrics_port(9000);

        assert_eq!(config.metrics_enabled, Some(true));
        assert_eq!(config.metrics_address, Some("192.168.1.100".to_string()));
        assert_eq!(config.metrics_port, Some(9000));
    }

    #[test]
    fn test_nat_builder() {
        let config = StorageConfig::new().nat("any");
        assert_eq!(config.nat, Some("any".to_string()));
    }

    #[test]
    fn test_net_priv_key_file_builder() {
        let config = StorageConfig::new().net_priv_key_file("/path/to/key");
        assert_eq!(
            config.net_priv_key_file,
            Some(PathBuf::from("/path/to/key"))
        );
    }

    #[test]
    fn test_num_threads_builder() {
        let config = StorageConfig::new().num_threads(4);
        assert_eq!(config.num_threads, Some(4));
    }

    #[test]
    fn test_agent_string_builder() {
        let config = StorageConfig::new().agent_string("CustomAgent/1.0");
        assert_eq!(config.agent_string, Some("CustomAgent/1.0".to_string()));
    }

    #[test]
    fn test_block_config_builders() {
        let config = StorageConfig::new()
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
    fn test_mix_builders() {
        let config = StorageConfig::new()
            .mix_enabled(true)
            .mix_pool_json("{\"relays\":[]}")
            .add_dht_mix_proxy("spr:proxy1")
            .dht_proxy_max_in_flight(8);

        assert_eq!(config.mix_enabled, Some(true));
        assert_eq!(config.mix_pool_json, Some("{\"relays\":[]}".to_string()));
        assert_eq!(config.dht_mix_proxies, vec!["spr:proxy1"]);
        assert_eq!(config.dht_proxy_max_in_flight, Some(8));
    }

    #[test]
    fn test_network_builder() {
        let config = StorageConfig::new().network(NetworkPreset::LogosTest);
        assert_eq!(config.network, Some(NetworkPreset::LogosTest));
    }

    #[test]
    fn test_log_file_builder() {
        let config = StorageConfig::new().log_file("/var/log/storage.log");
        assert_eq!(config.log_file, Some(PathBuf::from("/var/log/storage.log")));
    }

    #[test]
    fn test_comprehensive_builder() {
        let config = StorageConfig::new()
            .log_level(LogLevel::Debug)
            .log_format(LogFormat::Json)
            .data_dir("/tmp/storage")
            .listen_ip("127.0.0.1")
            .listen_port(8080)
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
            .mix_enabled(true);

        assert_eq!(config.log_level, Some(LogLevel::Debug));
        assert_eq!(config.log_format, Some(LogFormat::Json));
        assert_eq!(config.data_dir, Some(PathBuf::from("/tmp/storage")));
        assert_eq!(config.listen_ip, Some("127.0.0.1".to_string()));
        assert_eq!(config.listen_port, Some(8080));
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
        assert_eq!(config.mix_enabled, Some(true));
    }
}
