//! Upload session management for Storage
//!
//! This module provides low-level session management operations for uploads.
//! These functions handle the lifecycle of upload sessions including initialization,
//! finalization, and cancellation.

use crate::callback::{c_callback, with_libstorage_lock, CallbackFuture};
use crate::error::{Result, StorageError};
use crate::ffi::{
    storage_upload_cancel, storage_upload_finalize, storage_upload_init, string_to_c_string,
};
use crate::node::lifecycle::StorageNode;
use crate::upload::types::UploadOptions;

/// Initialize an upload session
///
/// Creates a new upload session with the specified options. Returns a session ID
/// that can be used for subsequent chunk uploads.
///
/// # Arguments
///
/// * `node` - The Storage node to use for the upload
/// * `options` - Upload configuration options
///
/// # Returns
///
/// A session ID string that identifies this upload session
pub async fn upload_init(node: &StorageNode, options: &UploadOptions) -> Result<String> {
    options.validate()?;

    let future = CallbackFuture::new();
    let context_ptr = future.context_ptr();

    let filepath_str = options
        .filepath
        .as_ref()
        .and_then(|p| p.to_str())
        .unwrap_or("");

    let chunk_size = options.chunk_size.unwrap_or(64 * 1024);

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            let c_filepath = string_to_c_string(filepath_str);

            storage_upload_init(
                ctx as *mut _,
                c_filepath.as_ptr(),
                chunk_size,
                Some(c_callback),
                context_ptr.as_ptr(),
            )
        })
    });

    if result != 0 {
        return Err(StorageError::upload_error("Failed to initialize upload"));
    }

    let session_id = future.await?;
    Ok(session_id)
}

/// Finalize an upload session
///
/// Completes an upload session and returns the content ID (CID) of the uploaded data.
/// This should be called after all chunks have been uploaded.
///
/// # Arguments
///
/// * `node` - The Storage node used for the upload
/// * `session_id` - The session ID returned by `upload_init`
///
/// # Returns
///
/// The CID of the uploaded content
pub async fn upload_finalize(node: &StorageNode, session_id: &str) -> Result<String> {
    if session_id.is_empty() {
        return Err(StorageError::invalid_parameter(
            "session_id",
            "Session ID cannot be empty",
        ));
    }

    let future = CallbackFuture::new();
    let context_ptr = future.context_ptr();

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            let c_session_id = string_to_c_string(session_id);

            storage_upload_finalize(
                ctx as *mut _,
                c_session_id.as_ptr(),
                Some(c_callback),
                context_ptr.as_ptr(),
            )
        })
    });

    if result != 0 {
        return Err(StorageError::upload_error("Failed to finalize upload"));
    }

    let cid = future.await?;
    Ok(cid)
}

/// Cancel an upload session
///
/// Cancels an ongoing upload session and cleans up any resources associated with it.
/// This should be called if an upload needs to be aborted.
///
/// # Arguments
///
/// * `node` - The Storage node used for the upload
/// * `session_id` - The session ID returned by `upload_init`
pub async fn upload_cancel(node: &StorageNode, session_id: &str) -> Result<()> {
    if session_id.is_empty() {
        return Err(StorageError::invalid_parameter(
            "session_id",
            "Session ID cannot be empty",
        ));
    }

    let future = CallbackFuture::new();
    let context_ptr = future.context_ptr();

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            let c_session_id = string_to_c_string(session_id);

            storage_upload_cancel(
                ctx as *mut _,
                c_session_id.as_ptr(),
                Some(c_callback),
                context_ptr.as_ptr(),
            )
        })
    });

    if result != 0 {
        return Err(StorageError::upload_error("Failed to cancel upload"));
    }

    future.await?;
    Ok(())
}
