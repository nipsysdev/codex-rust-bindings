//! Download session management for Storage
//!
//! This module provides low-level session management operations for downloads.
//! These functions handle the lifecycle of download sessions including initialization
//! and cancellation.

use crate::callback::{c_callback, with_libstorage_lock, CallbackFuture};
use crate::download::types::DownloadOptions;
use crate::error::{Result, StorageError};
use crate::ffi::{storage_download_cancel, storage_download_init, string_to_c_string};
use crate::node::lifecycle::StorageNode;

/// Initialize a download session
///
/// Creates a new download session for the specified content ID (CID) with the given options.
/// This prepares the node to download content from the Storage network.
///
/// # Arguments
///
/// * `node` - The Storage node to use for the download
/// * `cid` - The content ID to download
/// * `options` - Download configuration options
///
/// # Returns
///
/// Ok(()) if the download session was initialized successfully
///
/// # Errors
///
/// Returns an error if:
/// - The CID is empty
/// - The options are invalid
/// - The download initialization fails
pub async fn download_init(node: &StorageNode, cid: &str, options: &DownloadOptions) -> Result<()> {
    if cid.is_empty() {
        return Err(StorageError::invalid_parameter(
            "cid",
            "CID cannot be empty",
        ));
    }

    options.validate()?;

    let future = CallbackFuture::new();
    let context_ptr = future.context_ptr();

    let chunk_size = options.chunk_size.unwrap_or(64 * 1024);

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            let c_cid = string_to_c_string(cid);

            storage_download_init(
                ctx as *mut _,
                c_cid.as_ptr(),
                chunk_size,
                false,
                Some(c_callback),
                context_ptr.as_ptr(),
            )
        })
    });

    if result != 0 {
        return Err(StorageError::download_error(
            "Failed to initialize download",
        ));
    }

    future.await?;

    Ok(())
}

/// Cancel a download session
///
/// Cancels an ongoing download session for the specified content ID.
/// This should be called if a download needs to be aborted.
///
/// # Arguments
///
/// * `node` - The Storage node used for the download
/// * `cid` - The content ID of the download to cancel
///
/// # Returns
///
/// Ok(()) if the download was cancelled successfully
///
/// # Errors
///
/// Returns an error if:
/// - The CID is empty
/// - The cancellation fails
pub async fn download_cancel(node: &StorageNode, cid: &str) -> Result<()> {
    if cid.is_empty() {
        return Err(StorageError::invalid_parameter(
            "cid",
            "CID cannot be empty",
        ));
    }

    let future = CallbackFuture::new();
    let context_ptr = future.context_ptr();

    let result = with_libstorage_lock(|| unsafe {
        let ctx = node.ctx();
        let c_cid = string_to_c_string(cid);

        storage_download_cancel(
            ctx as *mut _,
            c_cid.as_ptr(),
            Some(c_callback),
            context_ptr.as_ptr(),
        )
    });

    if result != 0 {
        return Err(StorageError::download_error("Failed to cancel download"));
    }

    future.await?;

    Ok(())
}

/// Synchronous version of download_init for internal use
pub(crate) fn download_init_sync(
    node: &StorageNode,
    cid: &str,
    options: &DownloadOptions,
) -> Result<()> {
    if cid.is_empty() {
        return Err(StorageError::invalid_parameter(
            "cid",
            "CID cannot be empty",
        ));
    }

    options.validate()?;

    let future = CallbackFuture::new();
    let context_ptr = future.context_ptr();

    let chunk_size = options.chunk_size.unwrap_or(64 * 1024);

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            let c_cid = string_to_c_string(cid);

            storage_download_init(
                ctx as *mut _,
                c_cid.as_ptr(),
                chunk_size,
                false,
                Some(c_callback),
                context_ptr.as_ptr(),
            )
        })
    });

    if result != 0 {
        return Err(StorageError::download_error(
            "Failed to initialize download",
        ));
    }

    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(future))?;

    Ok(())
}
