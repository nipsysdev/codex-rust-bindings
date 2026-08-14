//! Stream download operations for Storage
//!
//! This module provides high-level streaming download functionality for the Storage network.
//! It supports downloading content directly to files, writers, or custom destinations
//! with progress tracking and verification.

use crate::callback::{c_callback, with_libstorage_lock, CallbackFuture};
use crate::download::session::download_init_sync;
use crate::download::types::{DownloadOptions, DownloadResult, DownloadStreamOptions};
use crate::error::{Result, StorageError};
use crate::ffi::{storage_download_stream, string_to_c_string};
use crate::node::lifecycle::StorageNode;
use std::io::Write;
use std::sync::{Arc, Mutex};

/// Download content as a stream to various destinations
///
/// High-level function that downloads content from the Storage network and streams it
/// to a file, writer, or custom callback. This function handles the complete download
/// process including session management, progress tracking, and error handling.
///
/// # Arguments
///
/// * `node` - The Storage node to use for the download
/// * `cid` - The content ID to download
/// * `options` - Stream download options including destination and configuration
///
/// # Returns
///
/// A `DownloadResult` containing download statistics and metadata
///
/// # Errors
///
/// Returns an error if:
/// - The CID is empty
/// - The options are invalid
/// - The download fails for any reason
pub async fn download_stream(
    node: &StorageNode,
    cid: &str,
    options: DownloadStreamOptions,
) -> Result<DownloadResult> {
    if cid.is_empty() {
        return Err(StorageError::invalid_parameter(
            "cid",
            "CID cannot be empty",
        ));
    }

    options.validate()?;

    let start_time = std::time::Instant::now();
    let chunk_size = options.chunk_size.unwrap_or(64 * 1024);

    let total_bytes = Arc::new(Mutex::new(0usize));
    let total_bytes_clone = total_bytes.clone();

    let file_handle = if let Some(ref filepath) = options.filepath {
        match std::fs::File::create(filepath) {
            Ok(file) => Some(Arc::new(Mutex::new(Some(file)))),
            Err(e) => {
                return Err(StorageError::Io(e));
            }
        }
    } else {
        None
    };

    let future = CallbackFuture::new();
    let context = future.context.clone();

    let file_handle_clone = file_handle.clone();

    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let tx_clone = tx.clone();
    let writer_task = if options.writer.is_some() {
        let mut writer = options.writer.unwrap();
        Some(std::thread::spawn(move || {
            while let Ok(chunk) = rx.recv() {
                if let Err(e) = writer.write_all(&chunk) {
                    eprintln!("Failed to write to writer: {}", e);
                    break;
                }
            }
        }))
    } else {
        None
    };

    context.set_progress_callback(move |_len, chunk| {
        if let Some(chunk_bytes) = chunk {
            let mut total = total_bytes_clone.lock().unwrap();
            *total += chunk_bytes.len();

            if let Some(ref file_handle) = file_handle_clone {
                if let Some(ref mut file) = file_handle.lock().unwrap().as_mut() {
                    if let Err(e) = file.write_all(chunk_bytes) {
                        eprintln!("Failed to write to file: {}", e);
                    }
                }
            }

            if tx_clone.send(chunk_bytes.to_vec()).is_err() {
                eprintln!("Failed to send data to writer thread");
            }
        }
    });

    let download_options = DownloadOptions::new(cid)
        .chunk_size(chunk_size)
        .timeout(options.timeout.unwrap_or(300))
        .verify(options.verify);

    download_init_sync(node, cid, &download_options)?;

    let context_ptr = future.context_ptr();
    let filepath_str = options
        .filepath
        .as_ref()
        .and_then(|p| p.to_str())
        .unwrap_or("");

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            let c_cid = string_to_c_string(cid);
            let c_filepath = string_to_c_string(filepath_str);

            storage_download_stream(
                ctx as *mut _,
                c_cid.as_ptr(),
                chunk_size,
                options.local,
                c_filepath.as_ptr(),
                Some(c_callback),
                context_ptr.as_ptr(),
            )
        })
    });

    if result != 0 {
        return Err(StorageError::download_error("Failed to download stream"));
    }

    future.await?;

    drop(tx);

    if let Some(handle) = writer_task {
        if let Err(e) = handle.join() {
            eprintln!("Writer thread failed: {:?}", e);
        }
    }

    if let Some(file_handle) = file_handle {
        if let Some(ref mut file) = file_handle.lock().unwrap().as_mut() {
            if let Err(e) = file.flush() {
                eprintln!("Failed to flush file: {}", e);
            }
        }
    }

    let duration = start_time.elapsed();
    let bytes_downloaded = *total_bytes.lock().unwrap();

    let mut result = DownloadResult::new(cid.to_string(), bytes_downloaded)
        .duration_ms(duration.as_millis() as u64)
        .verified(options.verify);

    if let Some(filepath) = options.filepath {
        result = result.filepath(filepath);
    }

    Ok(result)
}

/// Download content directly to a file
///
/// Convenience function that downloads content directly to a file.
/// This is a simplified version of `download_stream` for the common case
/// of downloading to a file.
///
/// # Arguments
///
/// * `node` - The Storage node to use for the download
/// * `cid` - The content ID to download
/// * `filepath` - The path where the file should be saved
///
/// # Returns
///
/// A `DownloadResult` containing download statistics
///
/// # Errors
///
/// Returns an error if the download fails
pub async fn download_to_file(
    node: &StorageNode,
    cid: &str,
    filepath: &std::path::Path,
) -> Result<DownloadResult> {
    let options = DownloadStreamOptions::new(cid)
        .filepath(filepath.to_path_buf())
        .local(true);

    download_stream(node, cid, options).await
}

/// Download content to a custom writer
///
/// Convenience function that downloads content to any type that implements Write.
/// This is useful for streaming to memory buffers, network streams, or custom destinations.
///
/// # Arguments
///
/// * `node` - The Storage node to use for the download
/// * `cid` - The content ID to download
/// * `writer` - Any type that implements Write
///
/// # Returns
///
/// A `DownloadResult` containing download statistics
///
/// # Errors
///
/// Returns an error if the download fails
pub async fn download_to_writer<W>(
    node: &StorageNode,
    cid: &str,
    writer: W,
) -> Result<DownloadResult>
where
    W: Write + Send + 'static,
{
    let options = DownloadStreamOptions::new(cid).writer(writer);
    download_stream(node, cid, options).await
}
