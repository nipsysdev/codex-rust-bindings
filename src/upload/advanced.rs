//! Advanced upload operations
//!
//! This module contains high-level upload operations like upload_reader and upload_file.

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{codex_upload_file, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use crate::upload::basic::{upload_cancel, upload_chunk, upload_finalize, upload_init};
use crate::upload::types::{UploadOptions, UploadProgress, UploadResult};
use libc::c_void;
use std::io::Read;
use std::path::Path;

/// Upload data from a reader
///
/// # Arguments
///
/// * `node` - The Codex node
/// * `options` - Upload options
/// * `reader` - The reader to read data from
///
/// # Returns
///
/// The result of the upload operation
pub async fn upload_reader<R>(
    node: &CodexNode,
    options: UploadOptions,
    mut reader: R,
) -> Result<UploadResult>
where
    R: Read,
{
    options.validate()?;

    let start_time = std::time::Instant::now();
    let chunk_size = options.chunk_size.unwrap_or(1024 * 1024); // 1 MB default

    // Initialize the upload
    let session_id = upload_init(node, &options).await?;

    // Read and upload chunks
    let mut buffer = vec![0u8; chunk_size];
    let mut total_bytes = 0;
    let mut chunk_count = 0;

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(n) => {
                total_bytes += n;
                chunk_count += 1;

                // Upload the chunk
                upload_chunk(node, &session_id, &buffer[..n]).await?;

                // Call progress callback if provided
                if let Some(ref callback) = options.on_progress {
                    let progress = UploadProgress::new_chunked(
                        total_bytes,
                        None, // Unknown total size
                        chunk_count,
                        chunk_count, // Current chunk is also the total so far
                    );
                    callback(progress);
                }
            }
            Err(e) => {
                // Cancel the upload on error
                let _ = upload_cancel(node, &session_id).await;
                return Err(CodexError::from(e));
            }
        }
    }

    // Finalize the upload
    let cid = upload_finalize(node, &session_id).await?;

    let duration = start_time.elapsed();

    Ok(UploadResult::new(cid, total_bytes)
        .chunks(chunk_count)
        .duration_ms(duration.as_millis() as u64)
        .verified(options.verify))
}

/// Upload a file
///
/// # Arguments
///
/// * `node` - The Codex node
/// * `options` - Upload options (must have filepath set)
///
/// # Returns
///
/// The result of the upload operation
pub async fn upload_file(node: &CodexNode, options: UploadOptions) -> Result<UploadResult> {
    if options.filepath.is_none() {
        return Err(CodexError::invalid_parameter(
            "filepath",
            "File path must be specified for file upload",
        ));
    }

    let filepath = options.filepath.as_ref().unwrap();

    if !Path::new(filepath).exists() {
        return Err(CodexError::invalid_parameter(
            "filepath",
            format!("File does not exist: {}", filepath.display()),
        ));
    }

    let start_time = std::time::Instant::now();

    // Get file size for progress tracking
    let file_size = std::fs::metadata(filepath)?.len() as usize;

    // Initialize upload session first
    let session_id = upload_init(node, &options).await?;

    // Create a callback future for the file upload operation
    let future = CallbackFuture::new();

    let c_session_id = string_to_c_string(&session_id);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_upload_file(
            node.ctx as *mut _,
            c_session_id,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_session_id);
    }

    if result != 0 {
        // Cancel the upload on error
        let _ = upload_cancel(node, &session_id).await;
        return Err(CodexError::library_error("Failed to upload file"));
    }

    // Wait for the operation to complete
    let cid = future.await?;

    let duration = start_time.elapsed();

    Ok(UploadResult::new(cid, file_size)
        .duration_ms(duration.as_millis() as u64)
        .verified(options.verify))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upload::types::UploadStrategy;

    #[test]
    fn test_upload_progress_callback() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let call_count = std::sync::Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let callback = move |progress: UploadProgress| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            println!(
                "Upload progress: {} bytes, {}%",
                progress.bytes_uploaded,
                progress.percentage * 100.0
            );
        };

        // Test the callback with some progress
        let progress = UploadProgress::new(500, Some(1000));
        callback(progress);

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_upload_result_creation() {
        let result = UploadResult::new("QmExample".to_string(), 1024)
            .chunks(4)
            .duration_ms(5000)
            .verified(true);

        assert_eq!(result.cid, "QmExample");
        assert_eq!(result.size, 1024);
        assert_eq!(result.chunks, Some(4));
        assert_eq!(result.duration_ms, 5000);
        assert!(result.verified);
    }

    #[test]
    fn test_upload_strategy_serialization() {
        let strategy = UploadStrategy::Chunked;
        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: UploadStrategy = serde_json::from_str(&json).unwrap();

        assert_eq!(strategy, deserialized);
    }

    #[test]
    fn test_upload_options_default() {
        let options = UploadOptions::default();

        assert_eq!(options.filepath, None);
        assert_eq!(options.chunk_size, Some(1024 * 1024));
        assert_eq!(options.strategy, UploadStrategy::Auto);
        assert!(options.on_progress.is_none());
        assert!(options.verify);
        assert!(options.metadata.is_none());
        assert_eq!(options.timeout, Some(300));
    }

    #[test]
    fn test_upload_progress_percentage() {
        // Test with known total
        let progress = UploadProgress::new(500, Some(1000));
        assert_eq!(progress.percentage, 0.5);

        // Test with zero total
        let progress = UploadProgress::new(500, Some(0));
        assert_eq!(progress.percentage, 0.0);

        // Test with unknown total
        let progress = UploadProgress::new(500, None);
        assert_eq!(progress.percentage, 0.0);

        // Test with bytes exceeding total (should cap at 1.0)
        let progress = UploadProgress::new(1500, Some(1000));
        assert_eq!(progress.percentage, 1.0);
    }

    #[test]
    fn test_upload_progress_chunked() {
        let progress = UploadProgress::new_chunked(250, Some(1000), 1, 4);

        assert_eq!(progress.bytes_uploaded, 250);
        assert_eq!(progress.total_bytes, Some(1000));
        assert_eq!(progress.percentage, 0.25);
        assert_eq!(progress.current_chunk, Some(1));
        assert_eq!(progress.total_chunks, Some(4));
    }
}
