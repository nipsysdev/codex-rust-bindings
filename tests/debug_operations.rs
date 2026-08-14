//! Debug operations integration test for the Storage Rust bindings
//!
//! This test demonstrates how to use debug operations:
//! - Get node debug information
//! - Update log levels
//! - Get peer debug information

use storage_bindings::debug::LogLevel;
use storage_bindings::{StorageConfig, StorageNode};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread")]
async fn test_debug_operations() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::try_init();

    let temp_dir = tempdir()?;

    let config = StorageConfig::new()
        .log_level(storage_bindings::LogLevel::Info)
        .data_dir(temp_dir.path().join("storage_data"))
        .discovery_port(8095);

    let node = StorageNode::new(config).await?;
    node.start().await?;

    // Get initial debug information
    let debug_info = storage_bindings::debug(&node).await?;
    println!("Peer ID: {}", debug_info.peer_id());
    println!("Address count: {}", debug_info.address_count());
    println!(
        "Announce address count: {}",
        debug_info.provider_address_count()
    );
    println!(
        "Discovery node count: {}",
        debug_info.discovery_node_count()
    );

    // Test updating log levels
    println!("\nTesting log level updates:");
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
        let update_result = storage_bindings::update_log_level(&node, log_level).await;
        match update_result {
            Ok(()) => {
                println!("  ✓ Log level updated to {:?}", log_level);
                let debug_info = storage_bindings::debug(&node).await?;
                println!("    Address count: {}", debug_info.address_count());
            }
            Err(e) => {
                println!("  ✗ Failed to update log level to {:?}: {}", log_level, e);
            }
        }
    }

    storage_bindings::update_log_level(&node, LogLevel::Info).await?;

    // Test peer debug information
    println!("\nTesting peer debug information:");
    let test_peer_ids = vec![
        "12D3KooWExamplePeer123456789",
        "QmSomePeerId123456789",
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        "zdj7WWeQ43G6JJvLWQWZpyHuAMq6uYWRjkBXZadLDEotRHi7T7ycf",
    ];

    for peer_id in test_peer_ids {
        let peer_record = storage_bindings::peer_debug(&node, peer_id).await;
        match peer_record {
            Ok(record) => {
                println!(
                    "  ✓ Peer debug info retrieved: ID={}, Connected={}",
                    record.id, record.connected
                );
            }
            Err(e) => {
                println!("  ✗ Failed to get peer debug info: {}", e);
            }
        }
    }

    // Test invalid peer ID
    println!("\nTesting invalid peer IDs:");
    let empty_peer_result = storage_bindings::peer_debug(&node, "").await;
    assert!(empty_peer_result.is_err(), "Should fail with empty peer ID");
    println!("  ✓ Correctly failed with empty peer ID");

    let whitespace_peer_result = storage_bindings::peer_debug(&node, "   \t\n   ").await;
    assert!(
        whitespace_peer_result.is_err(),
        "Should fail with whitespace-only peer ID"
    );
    println!("  ✓ Correctly failed with whitespace-only peer ID");

    // Test debug operations without starting node
    println!("\nTesting debug operations without starting node:");
    let config2 = StorageConfig::new()
        .log_level(storage_bindings::LogLevel::Error)
        .data_dir(temp_dir.path().join("storage_data2"))
        .discovery_port(8096);

    let node2 = StorageNode::new(config2).await?;

    let debug_info_result = storage_bindings::debug(&node2).await;
    match debug_info_result {
        Ok(info) => {
            println!(
                "  ✓ Debug info works without starting node: Peer ID={}",
                info.peer_id()
            );
        }
        Err(e) => println!("  ✗ Debug info failed without starting node: {}", e),
    }

    let update_result = storage_bindings::update_log_level(&node2, LogLevel::Debug).await;
    match update_result {
        Ok(()) => println!("  ✓ Log level update works without starting node"),
        Err(e) => println!("  ✗ Log level update failed without starting node: {}", e),
    }

    let peer_debug_result = storage_bindings::peer_debug(&node2, "12D3KooWTestPeer").await;
    match peer_debug_result {
        Ok(_) => println!("  ✓ Peer debug works without starting node"),
        Err(e) => println!("  ✗ Peer debug failed without starting node: {}", e),
    }

    node2.destroy().await?;

    // Test concurrent debug operations
    println!("\nTesting concurrent debug operations:");
    let debug_future1 = storage_bindings::debug(&node);
    let debug_future2 = storage_bindings::debug(&node);
    let peer_debug_future1 = storage_bindings::peer_debug(&node, "12D3KooWTestPeer1");
    let peer_debug_future2 = storage_bindings::peer_debug(&node, "12D3KooWTestPeer2");

    let (debug_result1, debug_result2, peer_debug_result1, peer_debug_result2) = tokio::join!(
        debug_future1,
        debug_future2,
        peer_debug_future1,
        peer_debug_future2
    );

    assert!(debug_result1.is_ok(), "debug (1) should succeed");
    println!("  ✓ debug (1): succeeded");

    assert!(debug_result2.is_ok(), "debug (2) should succeed");
    println!("  ✓ debug (2): succeeded");

    match peer_debug_result1 {
        Ok(_) => println!("  ✓ peer_debug (1): succeeded"),
        Err(_) => println!("  ✗ peer_debug (1): failed (expected for test peer)"),
    }

    match peer_debug_result2 {
        Ok(_) => println!("  ✓ peer_debug (2): succeeded"),
        Err(_) => println!("  ✗ peer_debug (2): failed (expected for test peer)"),
    }

    // Get final debug information
    println!("\nFinal node state:");
    let final_debug_info = storage_bindings::debug(&node).await?;
    println!("  Peer ID: {}", final_debug_info.peer_id());
    println!("  Address count: {}", final_debug_info.address_count());
    println!(
        "  Announce address count: {}",
        final_debug_info.provider_address_count()
    );
    println!(
        "  Discovery node count: {}",
        final_debug_info.discovery_node_count()
    );
    println!("  Health status: {}", final_debug_info.health_status());

    // Cleanup
    node.stop().await?;
    node.destroy().await?;

    Ok(())
}
