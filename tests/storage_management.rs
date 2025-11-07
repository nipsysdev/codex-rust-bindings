//! Storage management integration test for the Codex Rust bindings
//!
//! This test demonstrates how to manage storage operations:
//! - List manifests
//! - Check storage space
//! - Fetch manifest information
//! - Delete content
//! - Check content existence

use codex_rust_bindings::{CodexConfig, CodexNode, LogLevel};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[tokio::test]
async fn test_storage_management() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("Codex Rust Bindings - Storage Management Test");
    println!("=============================================");

    // Create a temporary directory for our test
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("test_file.txt");

    // Create a test file to upload
    println!("Creating test file...");
    let mut file = File::create(&file_path)?;
    file.write_all(b"This is a test file for storage management example.")?;
    file.sync_all()?;
    println!("Test file created at: {}", file_path.display());

    // Create a minimal Codex configuration
    println!("Creating Codex configuration...");
    let config = CodexConfig::new()
        .log_level(LogLevel::Error)
        .data_dir(temp_dir.path().join("codex_data"))
        .block_retries(3000);

    // Create and start a Codex node
    println!("Creating and starting Codex node...");
    let mut node = CodexNode::new(config)?;
    node.start()?;
    println!("Node started successfully!");

    // Get initial storage information
    println!("\n=== Initial Storage Information ===");
    let space_info = codex_rust_bindings::space(&node).await?;
    println!("Storage quota: {} bytes", space_info.quota_max_bytes);
    println!("Storage used: {} bytes", space_info.quota_used_bytes);
    println!(
        "Storage reserved: {} bytes",
        space_info.quota_reserved_bytes
    );
    println!("Total blocks: {}", space_info.total_blocks);

    // List initial manifests (should be empty)
    println!("\n=== Initial Manifests ===");
    let manifests = codex_rust_bindings::manifests(&node).await?;
    println!("Number of manifests: {}", manifests.len());
    assert_eq!(manifests.len(), 0, "Should start with no manifests");

    for (i, manifest) in manifests.iter().enumerate() {
        println!(
            "  Manifest {}: CID={}, Size={} bytes",
            i, manifest.cid, manifest.dataset_size
        );
    }

    // Upload a file to have some content
    println!("\n=== Uploading Test File ===");
    let upload_options = codex_rust_bindings::UploadOptions::new()
        .filepath(&file_path)
        .on_progress(|progress| {
            println!(
                "  Upload progress: {} bytes ({}%)",
                progress.bytes_uploaded,
                (progress.percentage * 100.0) as u32
            );
        });

    let upload_result = codex_rust_bindings::upload_file(&node, upload_options).await?;
    println!("File uploaded successfully!");
    println!("  CID: {}", upload_result.cid);
    println!("  Size: {} bytes", upload_result.size);

    // Check if content exists
    println!("\n=== Checking Content Existence ===");
    let exists = codex_rust_bindings::exists(&node, &upload_result.cid).await?;
    assert!(exists, "Uploaded content should exist");
    println!("Content exists: {}", exists);

    // Check non-existent content (using a valid CID format that doesn't exist)
    let non_existent_cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
    let non_existent = codex_rust_bindings::exists(&node, non_existent_cid).await?;
    assert!(!non_existent, "Non-existent content should not exist");
    println!("Non-existent content exists: {}", non_existent);

    // Fetch manifest information
    println!("\n=== Fetching Manifest Information ===");
    let manifest = codex_rust_bindings::fetch(&node, &upload_result.cid).await?;
    println!("Manifest CID: {}", manifest.cid);
    println!("Manifest size: {} bytes", manifest.dataset_size);
    println!("Manifest block size: {} bytes", manifest.block_size);
    println!("Manifest filename: {}", manifest.filename);
    println!("Manifest mimetype: {}", manifest.mimetype);
    println!("Manifest protected: {}", manifest.protected);

    // List manifests after upload
    println!("\n=== Manifests After Upload ===");
    let manifests = codex_rust_bindings::manifests(&node).await?;
    println!("Number of manifests: {}", manifests.len());
    assert_eq!(manifests.len(), 1, "Should have 1 manifest after upload");

    for (i, manifest) in manifests.iter().enumerate() {
        println!(
            "  Manifest {}: CID={}, Size={} bytes",
            i, manifest.cid, manifest.dataset_size
        );
    }

    // Get updated storage information
    println!("\n=== Updated Storage Information ===");
    let space_info = codex_rust_bindings::space(&node).await?;
    println!("Storage quota: {} bytes", space_info.quota_max_bytes);
    println!("Storage used: {} bytes", space_info.quota_used_bytes);
    println!(
        "Storage reserved: {} bytes",
        space_info.quota_reserved_bytes
    );
    println!("Total blocks: {}", space_info.total_blocks);

    // Upload another file for more content
    println!("\n=== Uploading Second Test File ===");
    let file_path2 = temp_dir.path().join("test_file2.txt");
    let mut file2 = File::create(&file_path2)?;
    file2.write_all(b"This is a second test file for storage management.")?;
    file2.sync_all()?;

    let upload_options2 = codex_rust_bindings::UploadOptions::new().filepath(&file_path2);

    let upload_result2 = codex_rust_bindings::upload_file(&node, upload_options2).await?;
    println!("Second file uploaded successfully!");
    println!("  CID: {}", upload_result2.cid);
    println!("  Size: {} bytes", upload_result2.size);

    // List manifests after second upload
    println!("\n=== Manifests After Second Upload ===");
    let manifests = codex_rust_bindings::manifests(&node).await?;
    println!("Number of manifests: {}", manifests.len());
    assert_eq!(
        manifests.len(),
        2,
        "Should have 2 manifests after second upload"
    );

    for (i, manifest) in manifests.iter().enumerate() {
        println!(
            "  Manifest {}: CID={}, Size={} bytes",
            i, manifest.cid, manifest.dataset_size
        );
    }

    // Delete the first file
    println!("\n=== Deleting First File ===");
    codex_rust_bindings::delete(&node, &upload_result.cid).await?;
    println!("First file deleted successfully!");

    // Check if deleted content still exists
    println!("\n=== Checking Deleted Content ===");
    let exists_after_delete = codex_rust_bindings::exists(&node, &upload_result.cid).await?;
    assert!(!exists_after_delete, "Deleted content should not exist");
    println!("Deleted content exists: {}", exists_after_delete);

    // List manifests after deletion
    println!("\n=== Manifests After Deletion ===");
    let manifests = codex_rust_bindings::manifests(&node).await?;
    println!("Number of manifests: {}", manifests.len());
    assert_eq!(manifests.len(), 1, "Should have 1 manifest after deletion");

    for (i, manifest) in manifests.iter().enumerate() {
        println!(
            "  Manifest {}: CID={}, Size={} bytes",
            i, manifest.cid, manifest.dataset_size
        );
    }

    // Get final storage information
    println!("\n=== Final Storage Information ===");
    let space_info = codex_rust_bindings::space(&node).await?;
    println!("Storage quota: {} bytes", space_info.quota_max_bytes);
    println!("Storage used: {} bytes", space_info.quota_used_bytes);
    println!(
        "Storage reserved: {} bytes",
        space_info.quota_reserved_bytes
    );
    println!("Total blocks: {}", space_info.total_blocks);

    // Stop and destroy the node
    println!("\n=== Cleanup ===");
    node.stop()?;
    node.destroy()?;
    println!("Node stopped and destroyed.");

    println!("\nStorage management test completed successfully!");
    Ok(())
}
