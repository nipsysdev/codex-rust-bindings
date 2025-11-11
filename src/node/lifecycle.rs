//! Node lifecycle management for Codex
//!
//! This module provides the main CodexNode struct and methods for
//! managing the lifecycle of a Codex node.

use crate::callback::{c_callback, with_libcodex_lock, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{
    codex_close, codex_destroy, codex_new, codex_peer_id, codex_repo, codex_revision, codex_spr,
    codex_start, codex_stop, codex_version, free_c_string, string_to_c_string,
};
use crate::node::config::CodexConfig;
use libc::c_void;
use std::ptr;
use std::sync::{Arc, Mutex};

/// A Codex node that can interact with the Codex network
///
/// This struct is thread-safe and can be safely shared across threads.
/// The underlying C library is not thread-safe, so all operations are
/// serialized through a global mutex.
#[derive(Clone)]
pub struct CodexNode {
    /// Shared state containing the C context and started flag
    inner: Arc<Mutex<CodexNodeInner>>,
}

/// Inner state of CodexNode
struct CodexNodeInner {
    /// Pointer to the C context
    ctx: *mut c_void,
    /// Whether the node is currently started
    started: bool,
}

unsafe impl Send for CodexNode {}
unsafe impl Sync for CodexNode {}

impl CodexNode {
    /// Create a new Codex node with the provided configuration
    ///
    /// The node is not started automatically; you need to call `start()`
    /// to start it.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the node
    ///
    /// # Returns
    ///
    /// A new CodexNode instance
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use codex_bindings::{CodexNode, CodexConfig};
    ///
    /// let config = CodexConfig::default();
    /// let mut node = CodexNode::new(config)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(config: CodexConfig) -> Result<Self> {
        with_libcodex_lock(|| {
            let json_config = config.to_json()?;
            let c_json_config = string_to_c_string(&json_config);

            // Create a callback future for the operation
            let future = CallbackFuture::new();

            let node_ctx = unsafe {
                // Call the C function with the context pointer directly
                let node_ctx = codex_new(
                    c_json_config,
                    Some(c_callback),
                    future.context_ptr() as *mut c_void,
                );

                // Clean up
                free_c_string(c_json_config);

                if node_ctx.is_null() {
                    return Err(CodexError::node_error("new", "Failed to create node"));
                }

                node_ctx
            };

            // Wait for the operation to complete
            let _result = future.wait()?;

            Ok(CodexNode {
                inner: Arc::new(Mutex::new(CodexNodeInner {
                    ctx: node_ctx,
                    started: false,
                })),
            })
        })
    }

    /// Start the Codex node
    ///
    /// This method starts the node and connects it to the Codex network.
    ///
    /// # Returns
    ///
    /// Ok(()) if the node was started successfully, or an error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use codex_bindings::{CodexNode, CodexConfig};
    ///
    /// let config = CodexConfig::default();
    /// let mut node = CodexNode::new(config)?;
    /// node.start()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn start(&mut self) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        if inner.started {
            return Err(CodexError::node_error("start", "Node is already started"));
        }

        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_start(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("start", "Failed to start node"));
        }

        // Wait for the operation to complete
        let _result = future.wait()?;

        inner.started = true;
        Ok(())
    }

    /// Start the Codex node asynchronously
    ///
    /// This is the async version of `start()`.
    pub async fn start_async(&self) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        if inner.started {
            return Err(CodexError::node_error(
                "start_async",
                "Node is already started",
            ));
        }

        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_start(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error(
                "start_async",
                "Failed to start node",
            ));
        }

        // Wait for the operation to complete
        let _result = future.await?;

        inner.started = true;
        Ok(())
    }

    /// Stop the Codex node
    ///
    /// This method stops the node and disconnects it from the Codex network.
    ///
    /// # Returns
    ///
    /// Ok(()) if the node was stopped successfully, or an error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use codex_bindings::{CodexNode, CodexConfig};
    ///
    /// let config = CodexConfig::default();
    /// let mut node = CodexNode::new(config)?;
    /// node.start()?;
    /// node.stop()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn stop(&mut self) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        if !inner.started {
            return Err(CodexError::node_error("stop", "Node is not started"));
        }

        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_stop(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("stop", "Failed to stop node"));
        }

        inner.started = false;
        Ok(())
    }

    /// Stop the Codex node asynchronously
    ///
    /// This is the async version of `stop()`.
    pub async fn stop_async(&self) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        if !inner.started {
            return Err(CodexError::node_error("stop_async", "Node is not started"));
        }

        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_stop(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("stop_async", "Failed to stop node"));
        }

        // Wait for the operation to complete
        let _result = future.await?;

        inner.started = false;
        Ok(())
    }

    /// Destroy the Codex node, freeing all resources
    ///
    /// The node must be stopped before calling this method.
    ///
    /// # Returns
    ///
    /// Ok(()) if the node was destroyed successfully, or an error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use codex_bindings::{CodexNode, CodexConfig};
    ///
    /// let config = CodexConfig::default();
    /// let mut node = CodexNode::new(config)?;
    /// node.start()?;
    /// node.stop()?;
    /// node.destroy()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn destroy(self) -> Result<()> {
        // Check if we're the sole owner
        if Arc::strong_count(&self.inner) != 1 {
            return Err(CodexError::node_error(
                "destroy",
                "Cannot destroy: multiple references exist",
            ));
        }

        let mut inner = self.inner.lock().unwrap();
        if inner.started {
            return Err(CodexError::node_error("destroy", "Node is still started"));
        }

        // First close the node - this needs to complete before destroy
        let future = CallbackFuture::new();

        // Call the C function to close the node
        let result = unsafe {
            codex_close(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("destroy", "Failed to close node"));
        }

        // Wait for the close operation to complete
        future.wait()?;

        // Now destroy the node - this is synchronous and doesn't use the callback
        // According to the Go bindings, we don't check the return value of destroy
        unsafe {
            codex_destroy(
                inner.ctx as *mut _,
                None, // No callback needed for destroy
                ptr::null_mut(),
            )
        };

        inner.ctx = ptr::null_mut();
        Ok(())
    }

    /// Get the version of the Codex node
    pub fn version(&self) -> Result<String> {
        let inner = self.inner.lock().unwrap();

        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_version(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("version", "Failed to get version"));
        }

        // Wait for the operation to complete
        let version = future.wait()?;

        Ok(version)
    }

    /// Get the revision of the Codex node
    pub fn revision(&self) -> Result<String> {
        let inner = self.inner.lock().unwrap();

        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_revision(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("revision", "Failed to get revision"));
        }

        // Wait for the operation to complete
        let revision = future.wait()?;

        Ok(revision)
    }

    /// Get the path of the data directory
    pub fn repo(&self) -> Result<String> {
        let inner = self.inner.lock().unwrap();

        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_repo(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("repo", "Failed to get repo path"));
        }

        // Wait for the operation to complete
        let repo = future.wait()?;

        Ok(repo)
    }

    /// Get the SPR (Storage Provider Reputation) of the node
    pub fn spr(&self) -> Result<String> {
        let inner = self.inner.lock().unwrap();

        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_spr(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("spr", "Failed to get SPR"));
        }

        // Wait for the operation to complete
        let spr = future.wait()?;

        Ok(spr)
    }

    /// Get the peer ID of the node
    pub fn peer_id(&self) -> Result<String> {
        let inner = self.inner.lock().unwrap();

        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_peer_id(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("peer_id", "Failed to get peer ID"));
        }

        // Wait for the operation to complete
        let peer_id = future.wait()?;

        Ok(peer_id)
    }

    /// Check if the node is started
    pub fn is_started(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.started
    }

    /// Get the raw context pointer (for internal use)
    #[allow(dead_code)]
    pub(crate) fn ctx(&self) -> *mut c_void {
        let inner = self.inner.lock().unwrap();
        inner.ctx
    }
}

impl Drop for CodexNode {
    fn drop(&mut self) {
        // Only cleanup if we're the last reference
        if Arc::strong_count(&self.inner) == 1 {
            let mut inner = self.inner.lock().unwrap();
            if !inner.ctx.is_null() && inner.started {
                // Try to stop the node if it's still started
                let _ = unsafe {
                    codex_stop(inner.ctx as *mut _, None, ptr::null_mut());
                };
                inner.started = false;
            }

            if !inner.ctx.is_null() {
                // Try to destroy the node if it's not already destroyed
                let _ = unsafe {
                    codex_destroy(inner.ctx as *mut _, None, ptr::null_mut());
                };
                inner.ctx = ptr::null_mut();
            }
        }
    }
}
