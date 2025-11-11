//! Node debugging operations
//!
//! This module contains node-specific debugging operations.

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{codex_debug, codex_log_level, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use libc::c_void;
use serde::{Deserialize, Serialize};

/// Log level for debugging
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

/// Debug information about the node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    /// Node peer ID
    pub id: String,
    /// Node addresses
    pub addrs: Vec<String>,
    /// Storage Provider Reputation (SPR)
    pub spr: String,
    /// Announce addresses
    #[serde(rename = "announceAddresses")]
    pub announce_addresses: Vec<String>,
    /// Discovery table information
    pub table: DiscoveryTable,
}

/// Discovery table information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryTable {
    /// Local node information
    #[serde(rename = "localNode")]
    pub local_node: LocalNodeInfo,
    /// Remote nodes in the table
    pub nodes: Vec<serde_json::Value>,
}

/// Local node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalNodeInfo {
    /// Node ID
    #[serde(rename = "nodeId")]
    pub node_id: String,
    /// Peer ID
    #[serde(rename = "peerId")]
    pub peer_id: String,
    /// Node record
    pub record: String,
    /// Bind address
    pub address: String,
    /// Whether the node has been seen
    pub seen: bool,
}

impl DebugInfo {
    /// Create a new debug info
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the peer ID
    pub fn peer_id(&self) -> &str {
        &self.id
    }

    /// Get the number of addresses
    pub fn address_count(&self) -> usize {
        self.addrs.len()
    }

    /// Get the number of announce addresses
    pub fn announce_address_count(&self) -> usize {
        self.announce_addresses.len()
    }

    /// Get the number of nodes in the discovery table
    pub fn discovery_node_count(&self) -> usize {
        self.table.nodes.len()
    }

    /// Check if the node is healthy
    pub fn is_healthy(&self) -> bool {
        // Basic health checks
        !self.id.is_empty()
            && !self.addrs.is_empty()
            && !self.spr.is_empty()
            && !self.table.local_node.node_id.is_empty()
    }

    /// Get health status as a string
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
            announce_addresses: Vec::new(),
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

/// Get debug information about the node
///
/// # Arguments
///
/// * `node` - The Codex node to get debug info for
///
/// # Returns
///
/// Debug information about the node
pub async fn debug(node: &CodexNode) -> Result<DebugInfo> {
    // Create a callback future for the operation
    let future = CallbackFuture::new();

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_debug(
            node.ctx() as *mut _,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    if result != 0 {
        return Err(CodexError::library_error("Failed to get debug info"));
    }

    // Wait for the operation to complete
    let debug_json = future.await?;

    // Parse the debug JSON
    let debug_info: DebugInfo = serde_json::from_str(&debug_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse debug info: {}", e)))?;

    Ok(debug_info)
}

/// Update the log level of the node
///
/// # Arguments
///
/// * `node` - The Codex node to update
/// * `log_level` - The new log level
///
/// # Returns
///
/// Ok(()) if the log level was updated successfully, or an error
pub async fn update_log_level(node: &CodexNode, log_level: LogLevel) -> Result<()> {
    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_log_level = string_to_c_string(&log_level.to_string());

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_log_level(
            node.ctx() as *mut _,
            c_log_level,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_log_level);
    }

    if result != 0 {
        return Err(CodexError::library_error("Failed to update log level"));
    }

    // Wait for the operation to complete
    future.await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Trace.to_string(), "trace");
        assert_eq!(LogLevel::Debug.to_string(), "debug");
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Notice.to_string(), "notice");
        assert_eq!(LogLevel::Warn.to_string(), "warn");
        assert_eq!(LogLevel::Error.to_string(), "error");
        assert_eq!(LogLevel::Fatal.to_string(), "fatal");
    }

    #[test]
    fn test_log_level_serialization() {
        let log_levels = vec![
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Notice,
            LogLevel::Warn,
            LogLevel::Error,
            LogLevel::Fatal,
        ];

        for log_level in log_levels {
            // Test serialization
            let json = serde_json::to_string(&log_level).unwrap();

            // Test deserialization
            let deserialized: LogLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(log_level, deserialized);
        }
    }

    #[test]
    fn test_debug_info_methods() {
        let debug_info = DebugInfo {
            id: "12D3KooWExamplePeer".to_string(),
            addrs: vec!["/ip4/127.0.0.1/tcp/8080".to_string()],
            spr: "spr:test".to_string(),
            announce_addresses: vec!["/ip4/127.0.0.1/tcp/8080".to_string()],
            table: DiscoveryTable {
                local_node: LocalNodeInfo {
                    node_id: "test_node_id".to_string(),
                    peer_id: "12D3KooWExamplePeer".to_string(),
                    record: "test_record".to_string(),
                    address: "127.0.0.1:8080".to_string(),
                    seen: true,
                },
                nodes: vec![],
            },
        };

        assert!(debug_info.is_healthy());
        assert_eq!(debug_info.health_status(), "Healthy");
        assert_eq!(debug_info.address_count(), 1);
        assert_eq!(debug_info.announce_address_count(), 1);
        assert_eq!(debug_info.discovery_node_count(), 0);

        let unhealthy_info = DebugInfo::new();
        assert!(!unhealthy_info.is_healthy());
        assert_eq!(unhealthy_info.health_status(), "Unhealthy");
    }

    #[test]
    fn test_debug_info_structure() {
        let test_debug_info = DebugInfo {
            id: "12D3KooWExamplePeer".to_string(),
            addrs: vec!["/ip4/127.0.0.1/tcp/8080".to_string()],
            spr: "spr:test".to_string(),
            announce_addresses: vec!["/ip4/127.0.0.1/tcp/8080".to_string()],
            table: DiscoveryTable {
                local_node: LocalNodeInfo {
                    node_id: "test_node_id".to_string(),
                    peer_id: "12D3KooWExamplePeer".to_string(),
                    record: "test_record".to_string(),
                    address: "127.0.0.1:8080".to_string(),
                    seen: true,
                },
                nodes: vec![],
            },
        };

        // Verify the debug info can be serialized and deserialized
        let json = serde_json::to_string(&test_debug_info).unwrap();
        let deserialized: DebugInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(test_debug_info.id, deserialized.id);
        assert_eq!(test_debug_info.addrs, deserialized.addrs);
        assert_eq!(test_debug_info.spr, deserialized.spr);
        assert_eq!(
            test_debug_info.announce_addresses,
            deserialized.announce_addresses
        );
        assert_eq!(
            test_debug_info.table.local_node.node_id,
            deserialized.table.local_node.node_id
        );
        assert_eq!(
            test_debug_info.table.local_node.peer_id,
            deserialized.table.local_node.peer_id
        );
    }
}
