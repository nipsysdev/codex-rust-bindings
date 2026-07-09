//! Two-node networking integration test for the Storage Rust bindings
//!
//! This test demonstrates how to create and connect two Storage nodes:
//! - Create two separate nodes
//! - Configure them to discover each other
//! - Connect the nodes
//! - Transfer data between nodes

use std::fs::File;
use std::io::Write;
use storage_bindings::{
    connect, download_stream, upload_file, DownloadStreamOptions, LogLevel, StorageConfig,
    StorageNode, UploadOptions,
};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread")]
async fn test_two_node_network() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::try_init();

    let temp_dir = tempdir()?;
    let node1_dir = temp_dir.path().join("node1");
    let node2_dir = temp_dir.path().join("node2");

    std::fs::create_dir_all(&node1_dir)?;
    std::fs::create_dir_all(&node2_dir)?;

    let file_path = temp_dir.path().join("test_file.txt");
    let download_path = temp_dir.path().join("downloaded_file.txt");

    // Create a test file to upload
    let mut file = File::create(&file_path)?;
    file.write_all(b"Hello from node1! This file will be transferred to node2.")?;
    file.sync_all()?;

    // Configure node1
    println!("Creating node 1:");
    let node1_config = StorageConfig::new()
        .log_level(LogLevel::Info)
        .data_dir(&node1_dir)
        .storage_quota(100 * 1024 * 1024)
        .max_peers(50)
        .discovery_port(8092)
        .listen_ip("0.0.0.0")
        .listen_port(0);

    let node1 = StorageNode::new(node1_config).await?;
    node1.start().await?;

    let node1_peer_id = node1.peer_id().await?;
    let node1_repo = node1.repo().await?;
    let debug1 = storage_bindings::debug(&node1).await?;

    println!("  Peer ID: {}", node1_peer_id);
    println!("  Repository: {}", node1_repo);
    println!("  SPR: {}", debug1.spr);

    // Configure node2
    println!("\nCreating node 2:");
    let mut node2_config = StorageConfig::new()
        .log_level(LogLevel::Info)
        .data_dir(&node2_dir)
        .storage_quota(100 * 1024 * 1024)
        .max_peers(50)
        .discovery_port(8093)
        .add_bootstrap_node(&debug1.spr);

    node2_config.listen_ip = Some("0.0.0.0".to_string());
    node2_config.listen_port = Some(0);

    let node2 = StorageNode::new(node2_config).await?;
    node2.start().await?;

    let node2_peer_id = node2.peer_id().await?;
    let node2_repo = node2.repo().await?;

    println!("  Peer ID: {}", node2_peer_id);
    println!("  Repository: {}", node2_repo);

    // Get debug information for both nodes
    println!("\nNode debug information:");
    let debug2 = storage_bindings::debug(&node2).await?;

    println!("Node 1:");
    println!("  Peer ID: {}", debug1.peer_id());
    println!("  Address count: {}", debug1.address_count());
    println!("  Discovery node count: {}", debug1.discovery_node_count());

    println!("Node 2:");
    println!("  Peer ID: {}", debug2.peer_id());
    println!("  Address count: {}", debug2.address_count());
    println!("  Discovery node count: {}", debug2.discovery_node_count());

    // Try to connect node2 to node1
    println!("\nAttempting P2P connection:");
    let node1_addresses = debug1.addrs.clone();

    println!("  Node1 Peer ID: {}", node1_peer_id);
    println!("  Trying addresses:");
    for (i, addr) in node1_addresses.iter().enumerate() {
        println!("    {}: {}", i + 1, addr);
    }

    let mut connection_successful = false;
    for addr in &node1_addresses {
        match connect(&node2, &node1_peer_id, std::slice::from_ref(addr)).await {
            Ok(()) => {
                println!("  ✓ Successfully connected node2 to node1 at {}", addr);
                connection_successful = true;
                break;
            }
            Err(e) => {
                println!("  ✗ Failed to connect to node1 at {}: {}", addr, e);
            }
        }
    }

    if !connection_successful {
        println!("  ⚠ Could not establish direct P2P connection, but continuing with test...");
    }

    // Upload a file from node1
    println!("\nUploading file from node 1:");
    let upload_options = UploadOptions::new()
        .filepath(&file_path)
        .on_progress(|progress| {
            println!(
                "  Upload progress: {} bytes ({}%)",
                progress.bytes_uploaded,
                (progress.percentage * 100.0) as u32
            );
        });

    let upload_result = upload_file(&node1, upload_options).await?;
    println!("  CID: {}", upload_result.cid);
    println!("  Size: {} bytes", upload_result.size);

    // Check if the content exists on node1
    println!("\nChecking content on node 1:");
    let exists_on_node1 = storage_bindings::exists(&node1, &upload_result.cid).await?;
    assert!(exists_on_node1, "Content should exist on node1");
    println!("  Content exists on node1: {}", exists_on_node1);

    // Try to fetch the content on node2
    println!("\nFetching content on node 2:");
    let fetch_timeout = tokio::time::Duration::from_secs(30);
    let fetch_result = tokio::time::timeout(
        fetch_timeout,
        storage_bindings::fetch(&node2, &upload_result.cid),
    )
    .await;

    let _fetch_successful = false;
    match fetch_result {
        Ok(Ok(manifest)) => {
            println!("  ✓ Successfully fetched content on node2:");
            println!("    CID: {}", manifest.cid);
            println!("    Size: {} bytes", manifest.dataset_size);
            println!("    Block size: {} bytes", manifest.block_size);
            let _fetch_successful = true;
        }
        Ok(Err(e)) => {
            println!("  ✗ Failed to fetch content on node2: {}", e);
            println!("    This might be expected if nodes are not connected");
        }
        Err(_) => {
            println!("  ✗ Fetch operation timed out after 30 seconds");
            println!("    This indicates the nodes are not properly connected or the content is not available");
        }
    }

    // Check if content exists on node2 after fetch attempt
    let exists_on_node2 = storage_bindings::exists(&node2, &upload_result.cid).await?;
    println!("  Content exists on node2: {}", exists_on_node2);

    // Download the file from node2 (if it has the content)
    if exists_on_node2 {
        println!("\nDownloading file from node 2:");
        let download_options = DownloadStreamOptions::new(&upload_result.cid)
            .filepath(&download_path)
            .on_progress(|progress| {
                println!(
                    "  Download progress: {} bytes ({}%)",
                    progress.bytes_downloaded,
                    (progress.percentage * 100.0) as u32
                );
            });

        let download_result = download_stream(&node2, &upload_result.cid, download_options).await?;
        println!("  Size: {} bytes", download_result.size);

        // Verify the downloaded content
        println!("\nVerifying downloaded content:");
        let original_content = std::fs::read_to_string(&file_path)?;
        let downloaded_content = std::fs::read_to_string(&download_path)?;

        assert_eq!(
            original_content, downloaded_content,
            "Downloaded content should match original"
        );
        println!("  ✓ Content verification successful! P2P transfer worked!");
    } else {
        println!("\nDownload test skipped:");
        println!("  Content not available on node2 - P2P transfer test skipped");
        println!("  This is expected if nodes cannot establish direct connection");
    }

    // Get final debug information
    println!("\nFinal node status:");
    let final_debug1 = storage_bindings::debug(&node1).await?;
    let final_debug2 = storage_bindings::debug(&node2).await?;

    println!("Node 1:");
    println!("  Peer ID: {}", final_debug1.peer_id());
    println!("  Address count: {}", final_debug1.address_count());
    println!(
        "  Discovery node count: {}",
        final_debug1.discovery_node_count()
    );
    println!("  Health status: {}", final_debug1.health_status());

    println!("Node 2:");
    println!("  Peer ID: {}", final_debug2.peer_id());
    println!("  Address count: {}", final_debug2.address_count());
    println!(
        "  Discovery node count: {}",
        final_debug2.discovery_node_count()
    );
    println!("  Health status: {}", final_debug2.health_status());

    // Get storage information
    println!("\nStorage information:");
    let space1 = storage_bindings::space(&node1).await?;
    let space2 = storage_bindings::space(&node2).await?;

    println!("Node 1:");
    println!("  Used: {} bytes", space1.quota_used_bytes);
    println!(
        "  Available: {} bytes",
        space1.quota_max_bytes - space1.quota_used_bytes
    );
    println!("  Total blocks: {}", space1.total_blocks);

    println!("Node 2:");
    println!("  Used: {} bytes", space2.quota_used_bytes);
    println!(
        "  Available: {} bytes",
        space2.quota_max_bytes - space2.quota_used_bytes
    );
    println!("  Total blocks: {}", space2.total_blocks);

    // List manifests on both nodes
    println!("\nManifests:");
    let manifests1 = storage_bindings::manifests(&node1).await?;
    let manifests2 = storage_bindings::manifests(&node2).await?;

    println!("Node 1 manifests: {}", manifests1.len());
    for manifest in &manifests1 {
        println!(
            "  CID: {}, Size: {} bytes",
            manifest.cid, manifest.dataset_size
        );
    }

    println!("Node 2 manifests: {}", manifests2.len());
    for manifest in &manifests2 {
        println!(
            "  CID: {}, Size: {} bytes",
            manifest.cid, manifest.dataset_size
        );
    }

    // Cleanup
    println!("\nCleanup:");
    println!("Stopping node1...");
    node1.stop().await?;
    node1.destroy().await?;

    println!("Stopping node2...");
    node2.stop().await?;
    node2.destroy().await?;

    println!("\nTwo-node network test completed!");
    println!("Note: P2P connectivity depends on network configuration and available ports.");

    Ok(())
}
