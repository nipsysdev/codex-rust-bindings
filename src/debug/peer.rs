//! Peer debugging operations
//!
//! This module contains peer-specific debugging operations.

use crate::callback::{c_callback, with_libcodex_lock, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{codex_peer_debug, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use crate::p2p::types::PeerRecord;
use libc::c_void;

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

    with_libcodex_lock(|| {
        let c_peer_id = string_to_c_string(peer_id);

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_peer_debug(
                node.ctx() as *mut _,
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

        Ok(())
    })?;

    // Wait for the operation to complete
    let peer_json = future.wait()?;

    // Parse the peer JSON
    let peer: PeerRecord = serde_json::from_str(&peer_json).map_err(|e| {
        CodexError::library_error(format!("Failed to parse peer debug info: {}", e))
    })?;

    Ok(peer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConnectionQuality;

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
