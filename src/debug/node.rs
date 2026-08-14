use crate::callback::{c_callback, with_libstorage_lock, CallbackFuture};
use crate::error::{Result, StorageError};
use crate::ffi::{storage_debug, storage_log_level, string_to_c_string};
use crate::node::lifecycle::StorageNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    pub id: String,
    pub addrs: Vec<String>,
    pub spr: String,
    #[serde(rename = "providerAddresses")]
    pub provider_addresses: Vec<String>,
    pub table: DiscoveryTable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryTable {
    #[serde(rename = "localNode")]
    pub local_node: LocalNodeInfo,
    pub nodes: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalNodeInfo {
    #[serde(rename = "nodeId")]
    pub node_id: String,
    #[serde(rename = "peerId")]
    pub peer_id: String,
    pub record: String,
    pub address: String,
    pub seen: bool,
}

impl DebugInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn peer_id(&self) -> &str {
        &self.id
    }

    pub fn address_count(&self) -> usize {
        self.addrs.len()
    }

    pub fn provider_address_count(&self) -> usize {
        self.provider_addresses.len()
    }

    pub fn discovery_node_count(&self) -> usize {
        self.table.nodes.len()
    }

    pub fn is_healthy(&self) -> bool {
        !self.id.is_empty()
            && !self.addrs.is_empty()
            && !self.spr.is_empty()
            && !self.table.local_node.node_id.is_empty()
    }

    pub fn health_status(&self) -> &'static str {
        if self.is_healthy() {
            "Healthy"
        } else {
            "Unhealthy"
        }
    }
}

impl Default for DebugInfo {
    fn default() -> Self {
        Self {
            id: String::new(),
            addrs: Vec::new(),
            spr: String::new(),
            provider_addresses: Vec::new(),
            table: DiscoveryTable {
                local_node: LocalNodeInfo {
                    node_id: String::new(),
                    peer_id: String::new(),
                    record: String::new(),
                    address: String::new(),
                    seen: false,
                },
                nodes: Vec::new(),
            },
        }
    }
}

pub async fn debug(node: &StorageNode) -> Result<DebugInfo> {
    let future = CallbackFuture::new();
    let context_ptr = future.context_ptr();

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| storage_debug(ctx as *mut _, Some(c_callback), context_ptr.as_ptr()))
    });

    if result != 0 {
        return Err(StorageError::library_error("Failed to get debug info"));
    }

    let debug_json = future.await?;

    let debug_info: DebugInfo = serde_json::from_str(&debug_json)
        .map_err(|e| StorageError::library_error(format!("Failed to parse debug info: {}", e)))?;

    Ok(debug_info)
}

pub async fn update_log_level(node: &StorageNode, log_level: LogLevel) -> Result<()> {
    let future = CallbackFuture::new();
    let context_ptr = future.context_ptr();

    let c_log_level = string_to_c_string(&log_level.to_string());

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            storage_log_level(
                ctx as *mut _,
                c_log_level.as_ptr(),
                Some(c_callback),
                context_ptr.as_ptr(),
            )
        })
    });

    if result != 0 {
        return Err(StorageError::library_error("Failed to update log level"));
    }

    future.await?;

    Ok(())
}
