//! Manifest download operations
//!
//! This module contains manifest-related download operations.

use crate::callback::{c_callback, CallbackFuture};
use crate::download::types::Manifest;
use crate::error::{CodexError, Result};
use crate::ffi::{codex_download_manifest, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use libc::c_void;

/// Download manifest information for a content
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `cid` - The content ID to get manifest for
///
/// # Returns
///
/// The manifest information for the content
pub async fn download_manifest(node: &CodexNode, cid: &str) -> Result<Manifest> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_cid = string_to_c_string(cid);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_download_manifest(
            node.ctx as *mut _,
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
        return Err(CodexError::download_error("Failed to download manifest"));
    }

    // Wait for the operation to complete
    let manifest_json = future.await?;

    // Parse the manifest JSON
    let manifest: Manifest = serde_json::from_str(&manifest_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse manifest: {}", e)))?;

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::config::CodexConfig;
    use serde_json::json;

    #[test]
    fn test_manifest_serialization() {
        let manifest = Manifest {
            cid: "QmExample".to_string(),
            size: 1024,
            blocks: 4,
            created: "2023-01-01T00:00:00Z".to_string(),
            accessed: Some("2023-01-02T00:00:00Z".to_string()),
            content_type: Some("text/plain".to_string()),
            metadata: Some(json!({"key": "value"})),
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: Manifest = serde_json::from_str(&json).unwrap();

        assert_eq!(manifest.cid, deserialized.cid);
        assert_eq!(manifest.size, deserialized.size);
        assert_eq!(manifest.blocks, deserialized.blocks);
        assert_eq!(manifest.created, deserialized.created);
        assert_eq!(manifest.accessed, deserialized.accessed);
        assert_eq!(manifest.content_type, deserialized.content_type);
        assert_eq!(manifest.metadata, deserialized.metadata);
    }
}
