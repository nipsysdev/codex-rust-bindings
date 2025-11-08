//! Chunk operations integration test for the Codex Rust bindings
//!
//! This test demonstrates how to use chunk-based upload and download:
//! - Upload using chunk-by-chunk approach
//! - Download using chunk-by-chunk approach
//! - Handle resumable uploads and downloads

use codex_rust_bindings::{
    download_cancel, download_chunk, download_init, upload_cancel, upload_chunk, upload_finalize,
    upload_init, CodexConfig, CodexNode, LogLevel, UploadOptions,
};
use tempfile::tempdir;

#[tokio::test]
async fn test_chunk_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let _ = env_logger::try_init();

    println!("Codex Rust Bindings - Chunk Operations Test");
    println!("===========================================");

    // Create a temporary directory for our test
    let temp_dir = tempdir()?;

    // Create a minimal Codex configuration
    println!("Creating Codex configuration...");
    let config = CodexConfig::new()
        .log_level(LogLevel::Error)
        .data_dir(temp_dir.path().join("codex_data"))
        .storage_quota(100 * 1024 * 1024) // 100 MB
        .block_retries(3000)
        .discovery_port(8094);

    // Create and start a Codex node
    println!("Creating and starting Codex node...");
    let mut node = CodexNode::new(config)?;
    node.start()?;
    println!("Node started successfully!");

    // Test data to upload
    let test_data = b"Hello, Codex! This is a test file for chunk-based upload. ";
    let test_data2 = b"It contains multiple chunks that will be uploaded separately. ";
    let test_data3 = b"This demonstrates the chunk-based upload functionality.";

    println!("\n=== Chunk-based Upload ===");
    println!(
        "Total data size: {} bytes",
        test_data.len() + test_data2.len() + test_data3.len()
    );

    // Initialize upload
    println!("Initializing upload...");
    let upload_options = UploadOptions::new().filepath("test_chunks.txt");

    let session_id = upload_init(&node, &upload_options).await?;
    println!("Upload session created: {}", session_id);

    // Upload chunks
    println!("Uploading chunks...");

    println!("  Uploading chunk 1 ({} bytes)...", test_data.len());
    upload_chunk(&node, &session_id, test_data).await?;
    println!("  ✓ Chunk 1 uploaded");

    println!("  Uploading chunk 2 ({} bytes)...", test_data2.len());
    upload_chunk(&node, &session_id, test_data2).await?;
    println!("  ✓ Chunk 2 uploaded");

    println!("  Uploading chunk 3 ({} bytes)...", test_data3.len());
    upload_chunk(&node, &session_id, test_data3).await?;
    println!("  ✓ Chunk 3 uploaded");

    // Finalize upload
    println!("Finalizing upload...");
    let cid = upload_finalize(&node, &session_id).await?;
    println!("✓ Upload finalized!");
    println!("  CID: {}", cid);

    // Verify the content exists
    println!("\n=== Verifying Upload ===");
    let exists = codex_rust_bindings::exists(&node, &cid).await?;
    assert!(exists, "Content should exist after upload");
    println!("Content exists: {}", exists);

    // Get manifest information
    let manifest = codex_rust_bindings::fetch(&node, &cid).await?;
    println!("Manifest information:");
    println!("  CID: {}", manifest.cid);
    println!("  Size: {} bytes", manifest.dataset_size);
    println!("  Block size: {} bytes", manifest.block_size);
    println!("  Filename: {}", manifest.filename);

    // Test chunk-based download
    println!("\n=== Chunk-based Download ===");

    // Initialize download
    println!("Initializing download...");
    let download_options = codex_rust_bindings::DownloadOptions::new(&cid);
    download_init(&node, &cid, &download_options).await?;
    println!("Download initialized for CID: {}", cid);

    // Download chunks
    println!("Downloading chunks...");
    let mut downloaded_data = Vec::new();
    let mut chunk_count = 0;

    // Download chunks until we get an error (indicating no more chunks)
    loop {
        match download_chunk(&node, &cid).await {
            Ok(chunk) => {
                chunk_count += 1;
                println!("  Downloaded chunk {} ({} bytes)", chunk_count, chunk.len());
                downloaded_data.extend_from_slice(&chunk);

                // Stop after a reasonable number of chunks to avoid infinite loop
                if chunk_count >= 10 {
                    println!("  Stopping after 10 chunks to avoid infinite loop");
                    break;
                }
            }
            Err(e) => {
                println!("  Download completed or error: {}", e);
                break;
            }
        }

        // Stop if we've downloaded all expected data
        if downloaded_data.len() >= manifest.dataset_size {
            println!("  All expected data downloaded");
            break;
        }
    }

    // Cancel download session
    println!("Canceling download session...");
    download_cancel(&node, &cid).await?;
    println!("✓ Download session canceled");

    // Verify downloaded data
    println!("\n=== Verifying Downloaded Data ===");
    let mut expected_data = Vec::new();
    expected_data.extend_from_slice(test_data);
    expected_data.extend_from_slice(test_data2);
    expected_data.extend_from_slice(test_data3);

    println!("Expected data size: {} bytes", expected_data.len());
    println!("Downloaded data size: {} bytes", downloaded_data.len());

    assert!(
        downloaded_data.len() >= expected_data.len(),
        "Should download at least expected data size"
    );

    let downloaded_str = String::from_utf8_lossy(&downloaded_data[..expected_data.len()]);
    let expected_str = String::from_utf8_lossy(&expected_data);

    assert_eq!(
        downloaded_str, expected_str,
        "Downloaded data should match expected data"
    );
    println!("✓ Downloaded data matches expected data!");

    // Test upload cancellation
    println!("\n=== Testing Upload Cancellation ===");

    // Initialize another upload
    let cancel_options = UploadOptions::new().filepath("cancel_test.txt");

    let cancel_session_id = upload_init(&node, &cancel_options).await?;
    println!("Created cancel test session: {}", cancel_session_id);

    // Upload one chunk
    let cancel_data = b"This upload will be canceled";
    upload_chunk(&node, &cancel_session_id, cancel_data).await?;
    println!("Uploaded one chunk for cancellation test");

    // Cancel the upload
    println!("Canceling upload...");
    upload_cancel(&node, &cancel_session_id).await?;
    println!("✓ Upload canceled");

    // Try to finalize - should fail
    println!("Attempting to finalize canceled upload...");
    let finalize_result = upload_finalize(&node, &cancel_session_id).await;
    assert!(
        finalize_result.is_err(),
        "Finalize should fail after cancellation"
    );
    println!("✓ Finalize correctly failed after cancellation");

    // Test with very small chunks
    println!("\n=== Testing Small Chunks ===");

    let small_options = UploadOptions::new().filepath("small_chunks.txt");

    let small_session_id = upload_init(&node, &small_options).await?;
    println!("Created small chunks session: {}", small_session_id);

    // Upload very small chunks
    let small_chunks = [b"A", b"B", b"C", b"D", b"E"];

    for (i, chunk) in small_chunks.iter().enumerate() {
        upload_chunk(&node, &small_session_id, *chunk).await?;
        println!(
            "  Uploaded small chunk {}: '{}' (1 byte)",
            i + 1,
            String::from_utf8_lossy(&chunk[..])
        );
    }

    let small_cid = upload_finalize(&node, &small_session_id).await?;
    println!("✓ Small chunks upload finalized: {}", small_cid);

    // Get final storage information
    println!("\n=== Final Storage Information ===");
    let space_info = codex_rust_bindings::space(&node).await?;
    println!("Storage usage:");
    println!("  Used: {} bytes", space_info.quota_used_bytes);
    println!(
        "  Available: {} bytes",
        space_info.quota_max_bytes - space_info.quota_used_bytes
    );
    println!("  Total blocks: {}", space_info.total_blocks);

    // List all manifests
    let manifests = codex_rust_bindings::manifests(&node).await?;
    println!("Total manifests: {}", manifests.len());
    assert!(
        manifests.len() >= 2,
        "Should have at least 2 manifests (main test and small chunks)"
    );

    for (i, manifest) in manifests.iter().enumerate() {
        println!(
            "  {}: CID={}, Size={} bytes",
            i + 1,
            manifest.cid,
            manifest.dataset_size
        );
    }

    // Stop and destroy the node
    println!("\n=== Cleanup ===");
    node.stop()?;
    node.destroy()?;
    println!("Node stopped and destroyed.");

    println!("\nChunk operations test completed successfully!");
    Ok(())
}
