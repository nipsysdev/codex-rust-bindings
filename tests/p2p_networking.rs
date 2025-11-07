//! P2P networking integration test for the Codex Rust bindings
//!
//! This test demonstrates how to use P2P operations:
//! - Connect to peers
//! - Get peer information
//! - Debug peer connections

use codex_rust_bindings::{CodexConfig, CodexNode, LogLevel};
use tempfile::tempdir;

#[tokio::test]
async fn test_p2p_networking() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("Codex Rust Bindings - P2P Networking Test");
    println!("==========================================");

    // Create a temporary directory for our test
    let temp_dir = tempdir()?;

    // Create a minimal Codex configuration
    println!("Creating Codex configuration...");
    let config = CodexConfig::new()
        .log_level(LogLevel::Error)
        .data_dir(temp_dir.path().join("codex_data"))
        .max_peers(50)
        .block_retries(3000);

    // Create and start a Codex node
    println!("Creating and starting Codex node...");
    let mut node = CodexNode::new(config)?;
    node.start()?;
    println!("Node started successfully!");

    // Get our node's peer ID
    println!("\n=== Node Information ===");
    let peer_id = node.peer_id()?;
    println!("Our peer ID: {}", peer_id);

    let version = node.version()?;
    println!("Node version: {}", version);

    let repo = node.repo()?;
    println!("Repository path: {}", repo);

    // Test P2P operations
    println!("\n=== P2P Operations ===");

    // Get our own peer ID using the P2P function
    let our_peer_id = codex_rust_bindings::get_peer_id(&node).await?;
    println!("Peer ID from P2P function: {}", our_peer_id);
    assert_eq!(peer_id, our_peer_id, "Peer IDs should match");

    // Test connecting to a peer (this will likely fail since it's a test peer)
    println!("\n=== Testing Peer Connection ===");
    let test_peer_id = "12D3KooWExamplePeer123456789";
    let test_addresses = vec![
        "/ip4/192.168.1.100/tcp/8080".to_string(),
        "/ip4/192.168.1.100/udp/8080/quic".to_string(),
        "/ip6/::1/tcp/8080".to_string(),
    ];

    println!("Attempting to connect to peer: {}", test_peer_id);
    for (i, addr) in test_addresses.iter().enumerate() {
        println!("  Address {}: {}", i + 1, addr);
    }

    let connect_result = codex_rust_bindings::connect(&node, test_peer_id, &test_addresses).await;
    match connect_result {
        Ok(()) => println!("✓ Successfully connected to peer"),
        Err(e) => println!("✗ Failed to connect to peer: {}", e),
    }

    // Test getting peer information
    println!("\n=== Testing Peer Information ===");
    let peer_info_result = codex_rust_bindings::get_peer_info(&node, test_peer_id).await;
    match peer_info_result {
        Ok(peer_info) => {
            println!("✓ Successfully retrieved peer information:");
            println!("  Peer ID: {}", peer_info.id);
            println!("  Connected: {}", peer_info.connected);
            println!("  Addresses: {:?}", peer_info.addresses);
            println!("  Protocols: {:?}", peer_info.protocols);
            if let Some(direction) = &peer_info.direction {
                println!("  Direction: {}", direction);
            }
            if let Some(latency) = peer_info.latency_ms {
                println!("  Latency: {} ms", latency);
            }
            if let Some(user_agent) = &peer_info.user_agent {
                println!("  User Agent: {}", user_agent);
            }
            if let Some(last_seen) = &peer_info.last_seen {
                println!("  Last Seen: {}", last_seen);
            }
        }
        Err(e) => println!("✗ Failed to get peer information: {}", e),
    }

    // Test with various peer ID formats
    println!("\n=== Testing Various Peer ID Formats ===");
    let test_peer_ids = vec![
        "12D3KooWExamplePeer123456789",
        "QmSomePeerId123456789",
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    ];

    for peer_id in test_peer_ids {
        println!("Testing peer ID: {}", peer_id);
        let peer_info_result = codex_rust_bindings::get_peer_info(&node, peer_id).await;
        match peer_info_result {
            Ok(_) => println!("  ✓ Successfully retrieved peer info"),
            Err(_) => println!("  ✗ Failed to retrieve peer info (expected for test peer)"),
        }
    }

    // Test invalid parameters
    println!("\n=== Testing Invalid Parameters ===");

    // Empty peer ID for connection
    println!("Testing connection with empty peer ID...");
    let empty_peer_result = codex_rust_bindings::connect(&node, "", &test_addresses).await;
    assert!(empty_peer_result.is_err(), "Should fail with empty peer ID");
    println!("  ✓ Correctly failed with empty peer ID");

    // Empty addresses for connection
    println!("Testing connection with empty addresses...");
    let empty_addr_result = codex_rust_bindings::connect(&node, test_peer_id, &[]).await;
    assert!(
        empty_addr_result.is_err(),
        "Should fail with empty addresses"
    );
    println!("  ✓ Correctly failed with empty addresses");

    // Empty peer ID for peer info
    println!("Testing peer info with empty peer ID...");
    let empty_info_result = codex_rust_bindings::get_peer_info(&node, "").await;
    assert!(empty_info_result.is_err(), "Should fail with empty peer ID");
    println!("  ✓ Correctly failed with empty peer ID");

    // Test concurrent P2P operations
    println!("\n=== Testing Concurrent P2P Operations ===");
    let peer_id_future1 = codex_rust_bindings::get_peer_id(&node);
    let peer_info_future1 = codex_rust_bindings::get_peer_info(&node, "12D3KooWTestPeer1");
    let peer_info_future2 = codex_rust_bindings::get_peer_info(&node, "12D3KooWTestPeer2");

    let (peer_id_result, peer_info_result1, peer_info_result2) =
        tokio::join!(peer_id_future1, peer_info_future1, peer_info_future2);

    println!("Concurrent operations results:");
    assert!(peer_id_result.is_ok(), "get_peer_id should succeed");
    println!("  ✓ get_peer_id: {}", peer_id_result.unwrap());

    match peer_info_result1 {
        Ok(_) => println!("  ✓ get_peer_info (peer1): succeeded"),
        Err(_) => println!("  ✗ get_peer_info (peer1): failed (expected)"),
    }

    match peer_info_result2 {
        Ok(_) => println!("  ✓ get_peer_info (peer2): succeeded"),
        Err(_) => println!("  ✗ get_peer_info (peer2): failed (expected)"),
    }

    // Stop and destroy the node
    println!("\n=== Cleanup ===");
    node.stop()?;
    node.destroy()?;
    println!("Node stopped and destroyed.");

    println!("\nP2P networking test completed successfully!");
    Ok(())
}
