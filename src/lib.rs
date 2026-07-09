//! # Logos Storage Rust Bindings
//!
//! This crate provides safe Rust bindings to the Logos Storage C FFI API.
//!
//! ## Safety
//!
//! This crate provides a safe wrapper around the underlying C FFI API. The following
//! safety guarantees are provided:
//!
//! - All opaque pointers are properly wrapped and never dereferenced
//! - All callback data is copied immediately
//! - Proper RAII is used for resource management
//! - Thread safety is enforced via Send/Sync traits
//!
//! ## Threading Model
//!
//! The Logos Storage system uses a single worker thread pattern:
//!
//! - **Main thread**: Rust application code
//! - **Worker thread**: Nim async runtime + storage operations
//! - **Communication**: Callback-based with Future support
//!
//! All FFI calls return immediately with a status code. The actual work happens
//! asynchronously on the worker thread, and results are delivered via callbacks.
//!
//! ## Memory Management
//!
//! - Strings allocated by Nim are freed by Nim
//! - Rust copies strings if needed for long-term storage
//! - Never store callback pointers beyond callback lifetime
//! - All opaque pointers are treated as `*mut c_void` and never dereferenced
//!
//! ## Lifecycle
//!
//! The Storage node follows a strict lifecycle:
//!
//! 1. **Create** - Create a new node with [`StorageNode::new()`]
//! 2. **Start** - Start the node with [`StorageNode::start()`] or [`StorageNode::start_async()`]
//! 3. **Stop** - Stop the node with [`StorageNode::stop()`] or [`StorageNode::stop_async()`]
//! 4. **Close** - Close the node with [`StorageNode::close()`] or [`StorageNode::close_async()`]
//! 5. **Destroy** - Destroy the node with [`StorageNode::destroy()`] or [`StorageNode::destroy_async()`]
//!
//! The node can be started and stopped multiple times, but must be closed
//! before it can be destroyed. The `Drop` implementation will automatically
//! clean up resources if the node is dropped without explicit destruction.
//!
//! ## Error Handling
//!
//! All operations return a `Result<T, StorageError>`. Errors are categorized
//! into different variants:
//!
//! - `LibraryError` - Errors from the underlying C library
//! - `NodeError` - Errors from node operations
//! - `UploadError` - Errors from upload operations
//! - `DownloadError` - Errors from download operations
//! - `StorageError` - Errors from storage operations
//! - `P2PError` - Errors from P2P operations
//! - `ConfigError` - Errors from configuration
//! - `InvalidParameter` - Invalid parameter errors
//! - `Timeout` - Operation timeout errors
//! - `Cancelled` - Operation cancelled errors
//! - `MissingCallback` - Missing callback errors
//! - `NullPointer` - Null pointer errors
//!
//! ## Example
//!
//! ```no_run
//! use storage_bindings::{LogLevel, StorageConfig, StorageNode};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create configuration
//!     let config = StorageConfig::new()
//!         .log_level(LogLevel::Info)
//!         .data_dir("./storage");
//!
//!     // Create node
//!     let node = StorageNode::new(config).await?;
//!
//!     // Start node
//!     node.start().await?;
//!
//!     // Get node information
//!     println!("Peer ID: {}", node.peer_id().await?);
//!     println!("Version: {}", node.version().await?);
//!
//!     // Stop and destroy node
//!     node.stop().await?;
//!     node.destroy().await?;
//!
//!     Ok(())
//! }
//! # Ok::<(), storage_bindings::StorageError>(())
//! ```
//!
//! ## Configuration
//!
//! Configuration can be loaded from multiple sources with the following priority:
//!
//! 1. **CLI arguments** - Highest priority
//! 2. **Configuration file** - JSON file
//! 3. **Environment variables** - Variables with `STORAGE_` prefix
//! 4. **Default values** - Lowest priority
//!
//! ### Environment Variables
//!
//! - `STORAGE_DATA_DIR` - Data directory path
//! - `STORAGE_LOG_LEVEL` - Log level (trace, debug, info, notice, warn, error, fatal)
//! - `STORAGE_LOG_FORMAT` - Log format (auto, colors, nocolors, json)
//! - `STORAGE_STORAGE_QUOTA` - Storage quota in bytes (supports suffixes: K, M, G, T)
//! - `STORAGE_MAX_PEERS` - Maximum number of peers
//! - `STORAGE_DISCOVERY_PORT` - Discovery port
//! - `STORAGE_NUM_THREADS` - Number of worker threads
//! - `STORAGE_REPO_KIND` - Repository kind (fs, sqlite, leveldb)
//! - `STORAGE_NAT` - NAT configuration
//! - `STORAGE_AGENT_STRING` - Agent string
//!
//! ## Type-Safe Wrappers
//!
//! The crate provides type-safe wrappers for common storage types:
//!
//! - [`Cid`] - Content Identifier with CIDv1 validation
//! - [`PeerId`] - Peer ID with base58 validation
//! - [`MultiAddress`] - MultiAddress with format validation
//!
//! ## Testing
//!
//! The crate includes comprehensive tests:
//!
//! - Unit tests for individual components
//! - Integration tests for full workflows
//! - Memory safety tests (run with sanitizers)
//! - Thread safety tests
//!
//! Run tests with:
//!
//! ```bash
//! # Run all tests
//! cargo test
//!
//! # Run with AddressSanitizer
//! RUSTFLAGS="-Z sanitizer=address" cargo test
//!
//! # Run with ThreadSanitizer
//! RUSTFLAGS="-Z sanitizer=thread" cargo test
//! ```

pub mod callback;
pub mod error;
pub mod ffi;
pub mod types;

pub mod debug;
pub mod download;
pub mod node;
pub mod p2p;
pub mod storage;
pub mod upload;

// Re-export types
pub use types::{Cid, CidError, MultiAddrError, MultiAddress, PeerId, PeerIdError};

// Debug operations and types
pub use debug::{debug, peer_debug, update_log_level, DebugInfo};

pub use download::{
    download_cancel, download_chunk, download_init, download_manifest, download_stream,
    DownloadOptions, DownloadProgress, DownloadResult, DownloadStreamOptions,
};

pub use error::{Result, StorageError};

pub use node::{LogFormat, LogLevel, NetworkPreset, StorageConfig, StorageNode};

pub use p2p::{
    connect, connect_to_multiple, get_peer_id, get_peer_info, validate_addresses, validate_peer_id,
    ConnectionQuality, PeerInfo, PeerRecord,
};

pub use storage::{delete, exists, fetch, manifests, space, Manifest as StorageManifest, Space};

pub use upload::{
    upload_cancel, upload_chunk, upload_file, upload_finalize, upload_init, upload_reader,
    UploadOptions, UploadProgress, UploadResult, UploadStrategy,
};

pub use upload::{
    create_streaming_reader, AsyncStreamingUploadReader, StreamingUploadReader, UploadProgressExt,
};
