//! High-level file upload operations for Storage
//!
//! This module provides convenient high-level functions for uploading files
//! and readers to the Storage network. These functions handle the complete
//! upload lifecycle including session management and chunking.

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{Result, StorageError};
use crate::ffi::{storage_upload_file, string_to_c_string};
use crate::node::lifecycle::StorageNode;
use crate::upload::types::{UploadOptions, UploadProgress, UploadResult};
use std::io::Read;
use std::path::Path;

/// Upload a file from the filesystem
///
/// High-level function that uploads a file from the filesystem to the Storage network.
/// This function handles the complete upload process including file validation,
/// session creation, and progress tracking.
///
/// # Arguments
///
/// * `node` - The Storage node to use for the upload
/// * `options` - Upload options including file path and configuration
///
/// # Returns
///
/// An `UploadResult` containing the CID and upload statistics
///
/// # Errors
///
/// Returns an error if:
/// - No file path is specified in options
/// - The file doesn't exist
/// - The upload fails for any reason
pub async fn upload_file(node: &StorageNode, options: UploadOptions) -> Result<UploadResult> {
    if options.filepath.is_none() {
        return Err(StorageError::invalid_parameter(
            "filepath",
            "File path must be specified for file upload",
        ));
    }

    let filepath = options.filepath.as_ref().unwrap();

    if !Path::new(filepath).exists() {
        return Err(StorageError::invalid_parameter(
            "filepath",
            format!("File does not exist: {}", filepath.display()),
        ));
    }

    let start_time = std::time::Instant::now();

    let file_size = std::fs::metadata(filepath)?.len() as usize;

    let session_id = upload_init_sync(node, &options)?;

    let future = CallbackFuture::new();
    let context_ptr = future.context_ptr();

    let result = unsafe {
        node.with_ctx_locked(|ctx| {
            let c_session_id = string_to_c_string(&session_id);

            storage_upload_file(
                ctx as *mut _,
                c_session_id.as_ptr(),
                Some(c_callback),
                context_ptr.as_ptr(),
            )
        })
    };

    if result != 0 {
        let _ = upload_cancel_sync(node, &session_id);
        return Err(StorageError::library_error("Failed to upload file"));
    }

    let cid = future.await?;

    let duration = start_time.elapsed();

    Ok(UploadResult::new(cid, file_size)
        .duration_ms(duration.as_millis() as u64)
        .verified(options.verify))
}

/// Upload data from any Read implementation
///
/// High-level function that uploads data from any type that implements Read.
/// This is useful for uploading data from memory, network streams, or custom sources.
/// The function handles chunking the data and tracking progress.
///
/// # Arguments
///
/// * `node` - The Storage node to use for the upload
/// * `options` - Upload options including chunk size and progress callbacks
/// * `reader` - Any type that implements Read
///
/// # Returns
///
/// An `UploadResult` containing the CID and upload statistics
///
/// # Errors
///
/// Returns an error if:
/// - The reader fails
/// - The upload fails for any reason
pub async fn upload_reader<R>(
    node: &StorageNode,
    options: UploadOptions,
    reader: R,
) -> Result<UploadResult>
where
    R: Read + Send + 'static,
{
    let node = node.clone();

    tokio::task::spawn_blocking(move || {
        options.validate()?;

        let start_time = std::time::Instant::now();
        let chunk_size = options.chunk_size.unwrap_or(64 * 1024);

        let session_id = upload_init_sync(&node, &options)?;

        let mut buffer = vec![0u8; chunk_size];
        let mut total_bytes = 0;
        let mut chunk_count = 0;
        let mut reader = reader;

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    total_bytes += n;
                    chunk_count += 1;

                    upload_chunk_sync(&node, &session_id, &buffer[..n])?;

                    if let Some(ref callback) = options.on_progress {
                        let progress = UploadProgress::new_chunked(
                            total_bytes,
                            None,
                            chunk_count,
                            chunk_count,
                        );
                        callback(progress);
                    }
                }
                Err(e) => {
                    let _ = upload_cancel_sync(&node, &session_id);
                    return Err(StorageError::from(e));
                }
            }
        }

        let cid = upload_finalize_sync(&node, &session_id)?;

        let duration = start_time.elapsed();

        Ok(UploadResult::new(cid, total_bytes)
            .chunks(chunk_count)
            .duration_ms(duration.as_millis() as u64)
            .verified(options.verify))
    })
    .await?
}

/// Synchronous version of upload_init for internal use
fn upload_init_sync(node: &StorageNode, options: &UploadOptions) -> Result<String> {
    options.validate()?;

    let future = CallbackFuture::new();

    let filepath_str = options
        .filepath
        .as_ref()
        .and_then(|p| p.to_str())
        .unwrap_or("");

    let chunk_size = options.chunk_size.unwrap_or(64 * 1024);
    let context_ptr = future.context_ptr();

    let result = crate::callback::with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            let c_filepath = crate::ffi::string_to_c_string(filepath_str);

            crate::ffi::storage_upload_init(
                ctx as *mut _,
                c_filepath.as_ptr(),
                chunk_size,
                Some(crate::callback::c_callback),
                context_ptr.as_ptr(),
            )
        })
    });

    if result != 0 {
        return Err(StorageError::upload_error("Failed to initialize upload"));
    }

    let session_id =
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(future))?;
    Ok(session_id)
}

/// Synchronous version of upload_chunk for internal use
fn upload_chunk_sync(node: &StorageNode, session_id: &str, chunk: &[u8]) -> Result<()> {
    if session_id.is_empty() {
        return Err(StorageError::invalid_parameter(
            "session_id",
            "Session ID cannot be empty",
        ));
    }

    if chunk.is_empty() {
        return Err(StorageError::invalid_parameter(
            "chunk",
            "Chunk cannot be empty",
        ));
    }

    let future = CallbackFuture::new();

    let chunk_ptr = chunk.as_ptr() as *mut u8;
    let chunk_len = chunk.len();
    let context_ptr = future.context_ptr();

    let result = crate::callback::with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            let c_session_id = crate::ffi::string_to_c_string(session_id);

            crate::ffi::storage_upload_chunk(
                ctx as *mut _,
                c_session_id.as_ptr(),
                chunk_ptr,
                chunk_len,
                Some(crate::callback::c_callback),
                context_ptr.as_ptr(),
            )
        })
    });

    if result != 0 {
        return Err(StorageError::upload_error("Failed to upload chunk"));
    }

    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(future))?;
    Ok(())
}

/// Synchronous version of upload_finalize for internal use
fn upload_finalize_sync(node: &StorageNode, session_id: &str) -> Result<String> {
    if session_id.is_empty() {
        return Err(StorageError::invalid_parameter(
            "session_id",
            "Session ID cannot be empty",
        ));
    }

    let future = CallbackFuture::new();

    let context_ptr = future.context_ptr();

    let result = crate::callback::with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            let c_session_id = crate::ffi::string_to_c_string(session_id);

            crate::ffi::storage_upload_finalize(
                ctx as *mut _,
                c_session_id.as_ptr(),
                Some(crate::callback::c_callback),
                context_ptr.as_ptr(),
            )
        })
    });

    if result != 0 {
        return Err(StorageError::upload_error("Failed to finalize upload"));
    }

    let cid = tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(future))?;
    Ok(cid)
}

/// Synchronous version of upload_cancel for internal use
fn upload_cancel_sync(node: &StorageNode, session_id: &str) -> Result<()> {
    if session_id.is_empty() {
        return Err(StorageError::invalid_parameter(
            "session_id",
            "Session ID cannot be empty",
        ));
    }

    let future = CallbackFuture::new();

    let context_ptr = future.context_ptr();

    let result = crate::callback::with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            let c_session_id = crate::ffi::string_to_c_string(session_id);

            crate::ffi::storage_upload_cancel(
                ctx as *mut _,
                c_session_id.as_ptr(),
                Some(crate::callback::c_callback),
                context_ptr.as_ptr(),
            )
        })
    });

    if result != 0 {
        return Err(StorageError::upload_error("Failed to cancel upload"));
    }

    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(future))?;
    Ok(())
}
