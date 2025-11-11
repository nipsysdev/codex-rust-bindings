//! CRUD operations for storage
//!
//! This module contains content operations: fetch, delete, and exists.

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{
    codex_storage_delete, codex_storage_exists, codex_storage_fetch, free_c_string,
    string_to_c_string,
};
use crate::node::lifecycle::CodexNode;
use libc::c_void;

/// Fetch manifest information for a specific content
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `cid` - The content ID to fetch manifest for
///
/// # Returns
///
/// The manifest information for the specified content
pub async fn fetch(node: &CodexNode, cid: &str) -> Result<super::types::Manifest> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_cid = string_to_c_string(cid);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_storage_fetch(
            node.ctx() as *mut _,
            c_cid,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_cid);
    }

    if result != 0 {
        return Err(CodexError::storage_error(
            "fetch",
            "Failed to fetch manifest",
        ));
    }

    // Wait for the operation to complete
    let manifest_json = future.await?;

    // Parse the manifest JSON
    let manifest: super::types::Manifest = serde_json::from_str(&manifest_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse manifest: {}", e)))?;

    Ok(manifest)
}

/// Delete content from storage
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `cid` - The content ID to delete
///
/// # Returns
///
/// Ok(()) if the content was deleted successfully, or an error
pub async fn delete(node: &CodexNode, cid: &str) -> Result<()> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_cid = string_to_c_string(cid);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_storage_delete(
            node.ctx() as *mut _,
            c_cid,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_cid);
    }

    if result != 0 {
        return Err(CodexError::storage_error(
            "delete",
            "Failed to delete content",
        ));
    }

    // Wait for the operation to complete
    future.await?;

    Ok(())
}

/// Check if content exists in storage
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `cid` - The content ID to check
///
/// # Returns
///
/// true if the content exists, false otherwise
pub async fn exists(node: &CodexNode, cid: &str) -> Result<bool> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_cid = string_to_c_string(cid);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_storage_exists(
            node.ctx() as *mut _,
            c_cid,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_cid);
    }

    if result != 0 {
        return Err(CodexError::storage_error(
            "exists",
            "Failed to check if content exists",
        ));
    }

    // Wait for the operation to complete
    let exists_str = future.await?;

    // Parse the boolean result
    let exists = exists_str
        .parse::<bool>()
        .map_err(|e| CodexError::library_error(format!("Failed to parse exists result: {}", e)))?;

    Ok(exists)
}
