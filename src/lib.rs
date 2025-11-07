//! # Codex Rust Bindings
//!
//! This library provides Rust bindings for the Codex SDK, allowing you to interact
//! with the Codex network from Rust applications.
//!
//! ## Features
//!
//! - Node lifecycle management
//! - File upload and download
//! - Storage management
//! - P2P networking
//! - Debug operations
//! - Async/await support
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use codex_rust_bindings::{CodexNode, CodexConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a configuration
//!     let config = CodexConfig::default();
//!     
//!     // Create a new node
//!     let mut node = CodexNode::new(config)?;
//!     
//!     // Start the node
//!     node.start().await?;
//!     
//!     // Use the node...
//!     
//!     // Stop and destroy the node
//!     node.stop().await?;
//!     node.destroy()?;
//!     
//!     Ok(())
//! }
//! ```

pub mod callback;
pub mod error;
pub mod ffi;

pub mod debug;
pub mod download;
pub mod node;
pub mod p2p;
pub mod storage;
pub mod upload;

// Re-export the main types for convenience
pub use debug::{debug, peer_debug, update_log_level};
pub use debug::{DebugInfo, PeerRecord};
pub use download::types::DownloadResult;
pub use download::{
    download_cancel, download_chunk, download_init, download_manifest, download_stream,
};
pub use download::{DownloadOptions, DownloadProgress, DownloadStreamOptions};
pub use error::{CodexError, Result};
pub use node::{CodexConfig, CodexNode, LogFormat, LogLevel};
pub use p2p::{connect, get_peer_id, get_peer_info, PeerInfo};
pub use storage::{delete, exists, fetch, manifests, space, Manifest, Space};
pub use upload::types::UploadResult;
pub use upload::{
    upload_cancel, upload_chunk, upload_file, upload_finalize, upload_init, upload_reader,
    UploadOptions, UploadProgress,
};
