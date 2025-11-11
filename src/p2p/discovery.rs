//! P2P discovery operations
//!
//! This module contains peer discovery and information operations.

use crate::callback::{c_callback, with_libcodex_lock, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{codex_peer_debug, codex_peer_id, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use crate::p2p::types::PeerRecord;
use libc::c_void;

/// Get detailed information about a specific peer
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `peer_id` - The peer ID to get information for
///
/// # Returns
///
/// Detailed peer record
pub async fn get_peer_info(node: &CodexNode, peer_id: &str) -> Result<PeerRecord> {
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
            return Err(CodexError::p2p_error("Failed to get peer info"));
        }

        Ok(())
    })?;

    // Wait for the operation to complete
    let peer_json = future.await?;

    // Parse the peer JSON
    let peer: PeerRecord = serde_json::from_str(&peer_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse peer info: {}", e)))?;

    Ok(peer)
}

/// Get the peer ID of the current node
pub async fn get_peer_id(node: &CodexNode) -> Result<String> {
    // Create a callback future for the operation
    let future = CallbackFuture::new();

    with_libcodex_lock(|| {
        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_peer_id(
                node.ctx() as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::p2p_error("Failed to get peer ID"));
        }

        Ok(())
    })?;

    // Wait for the operation to complete
    let peer_id = future.await?;

    Ok(peer_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p2p::types::PeerInfo;

    #[test]
    fn test_peer_info_serialization() {
        let peer_info = PeerInfo {
            id: "12D3KooWExamplePeer123456789".to_string(),
            addresses: vec![
                "/ip4/192.168.1.100/tcp/8080".to_string(),
                "/ip4/192.168.1.100/udp/8080/quic".to_string(),
            ],
            connected: true,
            direction: Some("outbound".to_string()),
            latency_ms: Some(50),
        };

        let json = serde_json::to_string(&peer_info).unwrap();
        let deserialized: PeerInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(peer_info.id, deserialized.id);
        assert_eq!(peer_info.addresses, deserialized.addresses);
        assert_eq!(peer_info.connected, deserialized.connected);
        assert_eq!(peer_info.direction, deserialized.direction);
        assert_eq!(peer_info.latency_ms, deserialized.latency_ms);
    }

    #[test]
    fn test_peer_record_serialization() {
        let peer_record = PeerRecord {
            id: "12D3KooWExamplePeer123456789".to_string(),
            addresses: vec![
                "/ip4/192.168.1.100/tcp/8080".to_string(),
                "/ip4/192.168.1.100/udp/8080/quic".to_string(),
            ],
            connected: true,
            direction: Some("outbound".to_string()),
            latency_ms: Some(50),
            protocols: vec!["/codex/1.0.0".to_string()],
            user_agent: Some("codex-rust-bindings/0.1.0".to_string()),
            last_seen: Some("2023-01-01T12:00:00Z".to_string()),
            connection_duration_seconds: Some(1800),
            bytes_sent: Some(1024 * 1024),
            bytes_received: Some(2 * 1024 * 1024),
            metadata: Some(serde_json::json!({"version": "1.0.0"})),
        };

        let json = serde_json::to_string(&peer_record).unwrap();
        let deserialized: PeerRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(peer_record.id, deserialized.id);
        assert_eq!(peer_record.addresses, deserialized.addresses);
        assert_eq!(peer_record.connected, deserialized.connected);
        assert_eq!(peer_record.direction, deserialized.direction);
        assert_eq!(peer_record.latency_ms, deserialized.latency_ms);
        assert_eq!(peer_record.protocols, deserialized.protocols);
        assert_eq!(peer_record.user_agent, deserialized.user_agent);
        assert_eq!(peer_record.last_seen, deserialized.last_seen);
        assert_eq!(
            peer_record.connection_duration_seconds,
            deserialized.connection_duration_seconds
        );
        assert_eq!(peer_record.bytes_sent, deserialized.bytes_sent);
        assert_eq!(peer_record.bytes_received, deserialized.bytes_received);
        assert_eq!(peer_record.metadata, deserialized.metadata);
    }
}
