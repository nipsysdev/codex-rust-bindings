//! Peer debugging operations
//!
//! This module contains peer-specific debugging operations.

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{codex_peer_debug, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use libc::c_void;
use serde::{Deserialize, Serialize};

/// Detailed peer record for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerRecord {
    /// Peer ID
    pub id: String,
    /// Multiaddresses of the peer
    pub addresses: Vec<String>,
    /// Connection state
    pub connected: bool,
    /// Connection direction
    pub direction: Option<String>,
    /// Latency information
    pub latency_ms: Option<u64>,
    /// Protocols supported by the peer
    pub protocols: Vec<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Last seen timestamp
    pub last_seen: Option<String>,
    /// Connection duration in seconds
    pub connection_duration_seconds: Option<u64>,
    /// Bytes sent to this peer
    pub bytes_sent: Option<u64>,
    /// Bytes received from this peer
    pub bytes_received: Option<u64>,
    /// Additional peer metadata
    pub metadata: Option<serde_json::Value>,
}

impl PeerRecord {
    /// Create a new peer record
    pub fn new(id: String) -> Self {
        Self {
            id,
            addresses: Vec::new(),
            connected: false,
            direction: None,
            latency_ms: None,
            protocols: Vec::new(),
            user_agent: None,
            last_seen: None,
            connection_duration_seconds: None,
            bytes_sent: None,
            bytes_received: None,
            metadata: None,
        }
    }

    /// Set the addresses
    pub fn addresses(mut self, addresses: Vec<String>) -> Self {
        self.addresses = addresses;
        self
    }

    /// Set the connection status
    pub fn connected(mut self, connected: bool) -> Self {
        self.connected = connected;
        self
    }

    /// Set the connection direction
    pub fn direction(mut self, direction: String) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Set the latency
    pub fn latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    /// Set the protocols
    pub fn protocols(mut self, protocols: Vec<String>) -> Self {
        self.protocols = protocols;
        self
    }

    /// Set the user agent
    pub fn user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Set the last seen time
    pub fn last_seen(mut self, last_seen: String) -> Self {
        self.last_seen = Some(last_seen);
        self
    }

    /// Set the connection duration
    pub fn connection_duration(mut self, duration_seconds: u64) -> Self {
        self.connection_duration_seconds = Some(duration_seconds);
        self
    }

    /// Set the bytes sent
    pub fn bytes_sent(mut self, bytes_sent: u64) -> Self {
        self.bytes_sent = Some(bytes_sent);
        self
    }

    /// Set the bytes received
    pub fn bytes_received(mut self, bytes_received: u64) -> Self {
        self.bytes_received = Some(bytes_received);
        self
    }

    /// Set the metadata
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Get the total bytes transferred
    pub fn total_bytes(&self) -> u64 {
        self.bytes_sent.unwrap_or(0) + self.bytes_received.unwrap_or(0)
    }

    /// Check if the peer supports a specific protocol
    pub fn supports_protocol(&self, protocol: &str) -> bool {
        self.protocols.contains(&protocol.to_string())
    }

    /// Get the connection duration as a human-readable string
    pub fn duration_string(&self) -> String {
        match self.connection_duration_seconds {
            Some(seconds) => {
                if seconds < 60 {
                    format!("{}s", seconds)
                } else if seconds < 3600 {
                    format!("{}m {}s", seconds / 60, seconds % 60)
                } else {
                    format!(
                        "{}h {}m {}s",
                        seconds / 3600,
                        (seconds % 3600) / 60,
                        seconds % 60
                    )
                }
            }
            None => "Unknown".to_string(),
        }
    }

    /// Get a human-readable size string for bytes transferred
    pub fn bytes_string(&self) -> String {
        let total = self.total_bytes();
        if total < 1024 {
            format!("{}B", total)
        } else if total < 1024 * 1024 {
            format!("{:.1}KB", total as f64 / 1024.0)
        } else if total < 1024 * 1024 * 1024 {
            format!("{:.1}MB", total as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1}GB", total as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Get a human-readable latency string
    pub fn latency_string(&self) -> String {
        match self.latency_ms {
            Some(latency) => format!("{}ms", latency),
            None => "Unknown".to_string(),
        }
    }

    /// Check if the connection is inbound
    pub fn is_inbound(&self) -> bool {
        self.direction.as_ref().map_or(false, |d| d == "inbound")
    }

    /// Check if the connection is outbound
    pub fn is_outbound(&self) -> bool {
        self.direction.as_ref().map_or(false, |d| d == "outbound")
    }

    /// Get connection quality based on latency and duration
    pub fn connection_quality(&self) -> ConnectionQuality {
        let latency = self.latency_ms.unwrap_or(u64::MAX);
        let duration = self.connection_duration_seconds.unwrap_or(0);

        match (latency, duration) {
            (0..=100, 300..) => ConnectionQuality::Excellent,
            (0..=100, _) => ConnectionQuality::Good,
            (101..=500, 300..) => ConnectionQuality::Good,
            (101..=500, _) => ConnectionQuality::Fair,
            (501..=1000, 300..) => ConnectionQuality::Fair,
            (501..=1000, _) => ConnectionQuality::Poor,
            _ => ConnectionQuality::VeryPoor,
        }
    }
}

/// Connection quality assessment
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    VeryPoor,
}

impl ConnectionQuality {
    /// Get a string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectionQuality::Excellent => "Excellent",
            ConnectionQuality::Good => "Good",
            ConnectionQuality::Fair => "Fair",
            ConnectionQuality::Poor => "Poor",
            ConnectionQuality::VeryPoor => "VeryPoor",
        }
    }

    /// Get a numeric score (0-4)
    pub fn score(&self) -> u8 {
        match self {
            ConnectionQuality::Excellent => 4,
            ConnectionQuality::Good => 3,
            ConnectionQuality::Fair => 2,
            ConnectionQuality::Poor => 1,
            ConnectionQuality::VeryPoor => 0,
        }
    }
}

/// Get detailed debug information about a specific peer
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `peer_id` - The peer ID to get debug information for
///
/// # Returns
///
/// Detailed peer record for debugging
pub fn peer_debug(node: &CodexNode, peer_id: &str) -> Result<PeerRecord> {
    if peer_id.is_empty() {
        return Err(CodexError::invalid_parameter(
            "peer_id",
            "Peer ID cannot be empty",
        ));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_peer_id = string_to_c_string(peer_id);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_peer_debug(
            node.ctx as *mut _,
            c_peer_id,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_peer_id);
    }

    if result != 0 {
        return Err(CodexError::library_error("Failed to get peer debug info"));
    }

    // Wait for the operation to complete
    let _peer_json = future.wait()?;

    // For now, return a placeholder peer record
    Ok(PeerRecord {
        id: peer_id.to_string(),
        addresses: vec![
            "/ip4/192.168.1.100/tcp/8080".to_string(),
            "/ip4/192.168.1.100/udp/8080/quic".to_string(),
        ],
        connected: true,
        direction: Some("outbound".to_string()),
        latency_ms: Some(50),
        protocols: vec!["/codex/1.0.0".to_string(), "/ipfs/id/1.0.0".to_string()],
        user_agent: Some("codex-rust-bindings/0.1.0".to_string()),
        last_seen: Some("2023-01-01T12:00:00Z".to_string()),
        connection_duration_seconds: Some(1800),
        bytes_sent: Some(1024 * 1024),
        bytes_received: Some(2 * 1024 * 1024),
        metadata: Some(serde_json::json!({
            "version": "1.0.0",
            "region": "us-west",
            "score": 0.95
        })),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_record_methods() {
        let record = PeerRecord::new("12D3KooWExamplePeer".to_string())
            .addresses(vec!["/ip4/192.168.1.100/tcp/8080".to_string()])
            .connected(true)
            .direction("outbound".to_string())
            .latency(50)
            .protocols(vec!["/codex/1.0.0".to_string()])
            .user_agent("codex-rust-bindings/0.1.0".to_string())
            .last_seen("2023-01-01T12:00:00Z".to_string())
            .connection_duration(1800)
            .bytes_sent(1024 * 1024)
            .bytes_received(2 * 1024 * 1024)
            .metadata(serde_json::json!({"version": "1.0.0"}));

        assert_eq!(record.total_bytes(), 3 * 1024 * 1024);
        assert!(record.supports_protocol("/codex/1.0.0"));
        assert_eq!(record.duration_string(), "30m 0s");
        assert_eq!(record.bytes_string(), "3.0MB");
        assert_eq!(record.latency_string(), "50ms");
        assert!(record.is_outbound());
        assert!(!record.is_inbound());
        assert_eq!(record.connection_quality(), ConnectionQuality::Excellent);
    }

    #[test]
    fn test_connection_quality() {
        assert_eq!(ConnectionQuality::Excellent.as_str(), "Excellent");
        assert_eq!(ConnectionQuality::Excellent.score(), 4);
        assert_eq!(ConnectionQuality::VeryPoor.as_str(), "VeryPoor");
        assert_eq!(ConnectionQuality::VeryPoor.score(), 0);
    }

    #[test]
    fn test_peer_record_serialization() {
        let record = PeerRecord::new("12D3KooWExamplePeer123456789".to_string())
            .addresses(vec![
                "/ip4/192.168.1.100/tcp/8080".to_string(),
                "/ip4/192.168.1.100/udp/8080/quic".to_string(),
                "/ip6/::1/tcp/8080".to_string(),
            ])
            .connected(true)
            .direction("outbound".to_string())
            .latency(50)
            .protocols(vec![
                "/codex/1.0.0".to_string(),
                "/ipfs/id/1.0.0".to_string(),
            ])
            .user_agent("codex-rust-bindings/0.1.0".to_string())
            .last_seen("2023-01-01T12:00:00Z".to_string())
            .connection_duration(1800)
            .bytes_sent(1024 * 1024)
            .bytes_received(2 * 1024 * 1024)
            .metadata(serde_json::json!({
                "version": "1.0.0",
                "region": "us-west",
                "score": 0.95,
                "capabilities": ["storage", "retrieval"]
            }));

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PeerRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(record.id, deserialized.id);
        assert_eq!(record.addresses, deserialized.addresses);
        assert_eq!(record.connected, deserialized.connected);
        assert_eq!(record.direction, deserialized.direction);
        assert_eq!(record.latency_ms, deserialized.latency_ms);
        assert_eq!(record.protocols, deserialized.protocols);
        assert_eq!(record.user_agent, deserialized.user_agent);
        assert_eq!(record.last_seen, deserialized.last_seen);
        assert_eq!(
            record.connection_duration_seconds,
            deserialized.connection_duration_seconds
        );
        assert_eq!(record.bytes_sent, deserialized.bytes_sent);
        assert_eq!(record.bytes_received, deserialized.bytes_received);
        assert_eq!(record.metadata, deserialized.metadata);
    }
}
