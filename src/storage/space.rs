//! Space management operations for storage
//!
//! This module contains storage management operations: manifests and space.

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{codex_storage_list, codex_storage_space};
use crate::node::lifecycle::CodexNode;
use libc::c_void;
use serde::{Deserialize, Serialize};

/// Manifest information for a stored content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Content ID (CID) - set separately in fetch()
    #[serde(skip)]
    pub cid: String,
    /// Tree CID - root of the merkle tree
    #[serde(rename = "treeCid", default)]
    pub tree_cid: String,
    /// Dataset size - total size of all blocks
    #[serde(rename = "datasetSize")]
    pub dataset_size: usize,
    /// Block size - size of each contained block
    #[serde(rename = "blockSize")]
    pub block_size: usize,
    /// Filename - name of the file (optional)
    #[serde(default)]
    pub filename: String,
    /// Mimetype - MIME type of the file (optional)
    #[serde(default)]
    pub mimetype: String,
    /// Protected datasets have erasure coded info
    #[serde(default)]
    pub protected: bool,
}

/// Manifest with CID wrapper (as returned by storage list)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestWithCid {
    /// Content ID (CID)
    pub cid: String,
    /// Manifest data
    pub manifest: Manifest,
}

/// Storage space information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    /// Total number of blocks stored by the node
    #[serde(rename = "totalBlocks")]
    pub total_blocks: usize,
    /// Maximum storage space (in bytes) available
    #[serde(rename = "quotaMaxBytes")]
    pub quota_max_bytes: u64,
    /// Amount of storage space (in bytes) currently used
    #[serde(rename = "quotaUsedBytes")]
    pub quota_used_bytes: u64,
    /// Amount of storage reserved (in bytes) for future use
    #[serde(rename = "quotaReservedBytes")]
    pub quota_reserved_bytes: u64,
}

/// List all manifests in the storage
///
/// # Arguments
///
/// * `node` - The Codex node to use
///
/// # Returns
///
/// A vector of manifests for all stored content
pub async fn manifests(node: &CodexNode) -> Result<Vec<Manifest>> {
    // Create a callback future for the operation
    let future = CallbackFuture::new();

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_storage_list(
            node.ctx() as *mut _,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    if result != 0 {
        return Err(CodexError::storage_error(
            "manifests",
            "Failed to list manifests",
        ));
    }

    // Wait for the operation to complete
    let manifests_json = future.await?;

    // Parse the manifests JSON array
    let manifests_with_cid: Vec<ManifestWithCid> = serde_json::from_str(&manifests_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse manifests: {}", e)))?;

    // Convert to Manifest structs with CID set
    let manifests: Vec<Manifest> = manifests_with_cid
        .into_iter()
        .map(|item| {
            let mut manifest = item.manifest;
            manifest.cid = item.cid;
            manifest
        })
        .collect();

    Ok(manifests)
}

/// Get storage space information
///
/// # Arguments
///
/// * `node` - The Codex node to use
///
/// # Returns
///
/// Information about storage usage and availability
pub async fn space(node: &CodexNode) -> Result<Space> {
    // Create a callback future for the operation
    let future = CallbackFuture::new();

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_storage_space(
            node.ctx() as *mut _,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    if result != 0 {
        return Err(CodexError::storage_error(
            "space",
            "Failed to get storage space",
        ));
    }

    // Wait for the operation to complete
    let space_json = future.await?;

    // Parse the space JSON
    let space: Space = serde_json::from_str(&space_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse space info: {}", e)))?;

    Ok(space)
}
