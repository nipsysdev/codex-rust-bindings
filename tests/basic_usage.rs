//! Basic usage integration test for the Codex Rust bindings
//!
//! This test demonstrates how to create a Codex node, start it,
//! upload a file, download it, and then clean up.

use codex_rust_bindings::{
    download_stream, upload_file, CodexConfig, CodexNode, DownloadStreamOptions, LogLevel,
    UploadOptions,
};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[tokio::test]
async fn test_basic_usage() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let _ = env_logger::try_init();

    println!("Codex Rust Bindings - Basic Usage Test");
    println!("=====================================");

    // Create a temporary directory for our test
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("example.txt");
    let download_path = temp_dir.path().join("downloaded.txt");

    // Create a test file to upload
    println!("Creating test file...");
    let mut file = File::create(&file_path)?;
    file.write_all(b"Hello, Codex! This is a test file for the Rust bindings.")?;
    file.sync_all()?;
    println!("Test file created at: {}", file_path.display());

    // Create a Codex configuration
    println!("Creating Codex configuration...");
    let config = CodexConfig::new()
        .log_level(LogLevel::Info)
        .data_dir(temp_dir.path().join("codex_data"))
        .storage_quota(100 * 1024 * 1024) // 100 MB
        .max_peers(50)
        .discovery_port(8090);

    // Create a new Codex node
    println!("Creating Codex node...");
    let mut node = CodexNode::new(config)?;

    // Start the node
    println!("Starting Codex node...");
    node.start()?;
    println!("Node started successfully!");

    // Get node information
    println!("Node information:");
    println!("  Version: {}", node.version()?);
    println!("  Peer ID: {}", node.peer_id()?);
    println!("  Repository: {}", node.repo()?);

    // Upload the file
    println!("Uploading file...");
    let upload_options = UploadOptions::new()
        .filepath(&file_path)
        .on_progress(|progress| {
            println!(
                "  Upload progress: {} bytes ({}%)",
                progress.bytes_uploaded,
                (progress.percentage * 100.0) as u32
            );
        });

    let upload_result = upload_file(&node, upload_options).await?;
    println!("File uploaded successfully!");
    println!("  CID: {}", upload_result.cid);
    println!("  Size: {} bytes", upload_result.size);
    println!("  Chunks: {:?}", upload_result.chunks);
    println!("  Duration: {} ms", upload_result.duration_ms);

    // Download the file
    println!("Downloading file...");
    let download_options = DownloadStreamOptions::new(&upload_result.cid)
        .filepath(&download_path)
        .on_progress(|progress| {
            println!(
                "  Download progress: {} bytes ({}%)",
                progress.bytes_downloaded,
                (progress.percentage * 100.0) as u32
            );
        });

    let download_result = download_stream(&node, &upload_result.cid, download_options).await?;
    println!("File downloaded successfully!");
    println!("  Size: {} bytes", download_result.size);
    println!("  Duration: {} ms", download_result.duration_ms);
    println!("  Saved to: {:?}", download_result.filepath);

    // Verify the downloaded content
    println!("Verifying downloaded content...");
    let original_content = std::fs::read_to_string(&file_path)?;
    let downloaded_content = std::fs::read_to_string(&download_path)?;

    assert_eq!(
        original_content, downloaded_content,
        "Downloaded content should match original"
    );
    println!("✓ Content verification successful!");

    // Stop and destroy the node
    println!("Stopping Codex node...");
    node.stop()?;
    println!("Node stopped.");

    println!("Destroying Codex node...");
    node.destroy()?;
    println!("Node destroyed.");

    println!("Basic usage test completed successfully!");
    Ok(())
}
