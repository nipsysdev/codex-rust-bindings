//! P2P connection operations
//!
//! This module contains connection management operations: connect and disconnect.

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{codex_connect, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use libc::{c_char, c_void};

/// Connect to a peer in the Codex network
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `peer_id` - The peer ID to connect to
/// * `peer_addresses` - List of multiaddresses for the peer
///
/// # Returns
///
/// Ok(()) if the connection was successful, or an error
pub async fn connect(node: &CodexNode, peer_id: &str, peer_addresses: &[String]) -> Result<()> {
    if peer_id.is_empty() {
        return Err(CodexError::invalid_parameter(
            "peer_id",
            "Peer ID cannot be empty",
        ));
    }

    if peer_addresses.is_empty() {
        return Err(CodexError::invalid_parameter(
            "peer_addresses",
            "At least one peer address must be provided",
        ));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_peer_id = string_to_c_string(peer_id);

    // Convert addresses to C array
    let c_addresses: Vec<*mut c_char> = peer_addresses
        .iter()
        .map(|addr| string_to_c_string(addr))
        .collect();

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_connect(
            node.ctx() as *mut _,
            c_peer_id,
            c_addresses.as_ptr() as *mut *mut c_char,
            c_addresses.len(),
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_peer_id);
        for addr in c_addresses {
            free_c_string(addr);
        }
    }

    if result != 0 {
        return Err(CodexError::p2p_error("Failed to connect to peer"));
    }

    // Wait for the operation to complete
    future.await?;

    Ok(())
}

/// Connect to multiple peers concurrently
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `peer_connections` - List of (peer_id, addresses) tuples
///
/// # Returns
///
/// A vector of results, one for each connection attempt
pub async fn connect_to_multiple(
    node: &CodexNode,
    peer_connections: Vec<(String, Vec<String>)>,
) -> Vec<Result<()>> {
    let mut results = Vec::with_capacity(peer_connections.len());

    for (peer_id, addresses) in peer_connections {
        let result = connect(node, &peer_id, &addresses).await;
        results.push(result);
    }

    results
}

/// Validate a peer ID format
///
/// # Arguments
///
/// * `peer_id` - The peer ID to validate
///
/// # Returns
///
/// Ok(()) if the peer ID is valid, or an error
pub fn validate_peer_id(peer_id: &str) -> Result<()> {
    if peer_id.is_empty() {
        return Err(CodexError::invalid_parameter(
            "peer_id",
            "Peer ID cannot be empty",
        ));
    }

    // Basic peer ID validation - peer IDs should have a reasonable length
    if peer_id.len() < 10 {
        return Err(CodexError::invalid_parameter(
            "peer_id",
            "Peer ID is too short",
        ));
    }

    if peer_id.len() > 100 {
        return Err(CodexError::invalid_parameter(
            "peer_id",
            "Peer ID is too long",
        ));
    }

    // Check for valid peer ID prefixes
    let valid_prefixes = vec![
        "12D3KooW", // libp2p Ed25519
        "Qm",       // CIDv0
        "bafy",     // CIDv1 raw
        "bafk",     // CIDv1 dag-pb
    ];

    let has_valid_prefix = valid_prefixes
        .iter()
        .any(|&prefix| peer_id.starts_with(prefix));

    if !has_valid_prefix {
        return Err(CodexError::invalid_parameter(
            "peer_id",
            "Peer ID has invalid format or prefix",
        ));
    }

    Ok(())
}

/// Validate multiaddresses
///
/// # Arguments
///
/// * `addresses` - The addresses to validate
///
/// # Returns
///
/// Ok(()) if all addresses are valid, or an error
pub fn validate_addresses(addresses: &[String]) -> Result<()> {
    if addresses.is_empty() {
        return Err(CodexError::invalid_parameter(
            "addresses",
            "At least one address must be provided",
        ));
    }

    for (i, address) in addresses.iter().enumerate() {
        if address.is_empty() {
            return Err(CodexError::invalid_parameter(
                &format!("addresses[{}]", i),
                "Address cannot be empty",
            ));
        }

        // Basic multiaddress validation
        if !address.starts_with('/') {
            return Err(CodexError::invalid_parameter(
                &format!("addresses[{}]", i),
                "Address must start with '/'",
            ));
        }

        // Check for valid protocols
        let valid_protocols = vec![
            "/ip4", "/ip6", "/dns4", "/dns6", "/dnsaddr", "/tcp", "/udp", "/quic", "/ws", "/wss",
            "/p2p", "/ipfs",
        ];

        let has_valid_protocol = valid_protocols
            .iter()
            .any(|&protocol| address.contains(protocol));

        if !has_valid_protocol {
            return Err(CodexError::invalid_parameter(
                &format!("addresses[{}]", i),
                "Address contains invalid protocol",
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_peer_id() {
        // Valid peer IDs
        let valid_peer_ids = vec![
            "12D3KooWExamplePeer123456789",
            "QmSomePeerId123456789",
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ];

        for peer_id in valid_peer_ids {
            assert!(
                validate_peer_id(peer_id).is_ok(),
                "Peer ID {} should be valid",
                peer_id
            );
        }

        // Invalid peer IDs
        let long_string = "X".repeat(101);
        let invalid_peer_ids = vec![
            "",
            "short",
            "12D3KooW",   // Too short even with valid prefix
            &long_string, // Too long
            "InvalidPrefix123456789",
        ];

        for peer_id in invalid_peer_ids {
            assert!(
                validate_peer_id(peer_id).is_err(),
                "Peer ID {} should be invalid",
                peer_id
            );
        }
    }

    #[test]
    fn test_validate_addresses() {
        // Valid addresses
        let valid_addresses = vec![
            vec!["/ip4/192.168.1.100/tcp/8080".to_string()],
            vec!["/ip6/::1/tcp/8080".to_string()],
            vec!["/dns4/example.com/tcp/8080".to_string()],
            vec![
                "/ip4/192.168.1.100/tcp/8080".to_string(),
                "/ip4/192.168.1.100/udp/8080/quic".to_string(),
            ],
        ];

        for addresses in valid_addresses {
            assert!(validate_addresses(&addresses).is_ok());
        }

        // Invalid addresses
        let invalid_addresses = vec![
            vec![],                                        // Empty
            vec!["".to_string()],                          // Empty string
            vec!["invalid-address".to_string()],           // Doesn't start with /
            vec!["/invalid/protocol/address".to_string()], // Invalid protocol
        ];

        for addresses in invalid_addresses {
            assert!(validate_addresses(&addresses).is_err());
        }
    }
}
