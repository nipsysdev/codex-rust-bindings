use crate::callback::{c_callback, with_libstorage_lock, CallbackFuture};
use crate::error::{Result, StorageError};
use crate::ffi::{
    c_str_to_string, storage_close, storage_destroy, storage_get_metrics, storage_new,
    storage_peer_id, storage_repo, storage_revision, storage_spr, storage_start, storage_stop,
    storage_toggle_private_queries, storage_version, string_to_c_string, SendSafePtr,
};
use crate::node::config::StorageConfig;
use libc::c_void;
use std::ptr;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct StorageNode {
    inner: Arc<Mutex<StorageNodeInner>>,
}

struct StorageNodeInner {
    ctx: *mut c_void,
    started: bool,
}

unsafe impl Send for StorageNodeInner {}
unsafe impl Sync for StorageNodeInner {}

unsafe impl Send for StorageNode {}
unsafe impl Sync for StorageNode {}

impl StorageNode {
    /// Create a new Storage node
    ///
    /// # Example
    ///
    /// ```no_run
    /// use storage_bindings::{LogLevel, StorageConfig, StorageNode};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = StorageConfig::new()
    ///         .log_level(LogLevel::Info)
    ///         .data_dir("./storage");
    ///
    ///     let node = StorageNode::new(config).await?;
    ///     node.start().await?;
    ///
    ///     let peer_id = node.peer_id().await?;
    ///     println!("Peer ID: {}", peer_id);
    ///
    ///     node.stop().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(config: StorageConfig) -> Result<Self> {
        let json_config = config.to_json()?;

        // Use a blocking task to avoid capturing the CallbackFuture in the async block
        let node_ctx_ptr = tokio::task::spawn_blocking(move || {
            // Create the CallbackFuture inside the blocking task
            let future = CallbackFuture::new();
            let context_id = future.context_id();

            // Call storage_new inside the lock, using the context ID
            let node_ctx = with_libstorage_lock(|| {
                let c_json_config = string_to_c_string(&json_config);

                unsafe {
                    let node_ctx = storage_new(
                        c_json_config.as_ptr(),
                        Some(c_callback),
                        context_id as *mut c_void,
                    );

                    // c_json_config is automatically freed when it goes out of scope here

                    if node_ctx.is_null() {
                        return Err(StorageError::node_error("new", "Failed to create node"));
                    }

                    Ok(node_ctx)
                }
            })?;

            // Wrap the raw pointer in SendSafePtr to make it Send
            let node_ctx_ptr = unsafe { SendSafePtr::new(node_ctx) };

            // Wait for the callback to complete
            let rt = tokio::runtime::Handle::current();
            let result = rt.block_on(future)?;

            Ok::<(SendSafePtr<c_void>, String), StorageError>((node_ctx_ptr, result))
        })
        .await
        .map_err(|e| StorageError::node_error("new", format!("Blocking task failed: {}", e)))??;

        // Extract the raw pointer from SendSafePtr
        let node_ctx = unsafe { node_ctx_ptr.0.as_ptr() };

        Ok(StorageNode {
            inner: Arc::new(Mutex::new(StorageNodeInner {
                ctx: node_ctx,
                started: false,
            })),
        })
    }

    /// Start the Storage node
    ///
    /// # Errors
    ///
    /// Returns an error if the node is already started.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use storage_bindings::{LogLevel, StorageConfig, StorageNode};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = StorageConfig::new()
    ///         .log_level(LogLevel::Info)
    ///         .data_dir("./storage");
    ///
    ///     let node = StorageNode::new(config).await?;
    ///     node.start().await?;
    ///
    ///     let peer_id = node.peer_id().await?;
    ///     println!("Peer ID: {}", peer_id);
    ///
    ///     node.stop().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn start(&self) -> Result<()> {
        let node = self.clone();

        {
            let inner = node.inner.lock().unwrap();
            if inner.started {
                return Err(StorageError::node_error("start", "Node is already started"));
            }
        }

        let future = CallbackFuture::new();
        let context_ptr = future.context_ptr();

        let ctx = {
            let inner = node.inner.lock().unwrap();
            inner.ctx as *mut _
        };

        let result = unsafe { storage_start(ctx, Some(c_callback), context_ptr.as_ptr()) };

        if result != 0 {
            return Err(StorageError::node_error("start", "Failed to start node"));
        }

        let _result = future.await?;

        {
            let mut inner = node.inner.lock().unwrap();
            inner.started = true;
        }

        Ok(())
    }

    pub async fn start_async(&self) -> Result<()> {
        self.start().await
    }

    /// Stop the Storage node
    ///
    /// # Errors
    ///
    /// Returns an error if the node is not started.
    pub async fn stop(&self) -> Result<()> {
        let node = self.clone();

        {
            let inner = node.inner.lock().unwrap();
            if !inner.started {
                return Err(StorageError::node_error("stop", "Node is not started"));
            }
        }

        let future = CallbackFuture::new();
        let context_ptr = future.context_ptr();

        let ctx = {
            let inner = node.inner.lock().unwrap();
            inner.ctx as *mut _
        };

        let result = unsafe { storage_stop(ctx, Some(c_callback), context_ptr.as_ptr()) };

        if result != 0 {
            return Err(StorageError::node_error("stop", "Failed to stop node"));
        }

        let _result = future.await?;

        {
            let mut inner = node.inner.lock().unwrap();
            inner.started = false;
        }

        Ok(())
    }

    pub async fn stop_async(&self) -> Result<()> {
        self.stop().await
    }

    /// Close the Storage node
    ///
    /// This method closes the node and releases resources. The node must be
    /// stopped before it can be closed.
    ///
    /// # Errors
    ///
    /// Returns an error if the node is still started.
    pub async fn close(&self) -> Result<()> {
        let node = self.clone();

        {
            let inner = node.inner.lock().unwrap();
            if inner.started {
                return Err(StorageError::node_error(
                    "close",
                    "Node must be stopped before closing",
                ));
            }
        }

        let future = CallbackFuture::new();
        let context_ptr = future.context_ptr();

        let ctx = {
            let inner = node.inner.lock().unwrap();
            inner.ctx as *mut _
        };

        let result = unsafe { storage_close(ctx, Some(c_callback), context_ptr.as_ptr()) };

        if result != 0 {
            return Err(StorageError::node_error("close", "Failed to close node"));
        }

        future.await?;
        Ok(())
    }

    pub async fn close_async(&self) -> Result<()> {
        self.close().await
    }

    /// Destroy the Storage node
    ///
    /// This method destroys the node and releases all resources. The node must be
    /// stopped before it can be destroyed. This method will automatically call
    /// close() before destroying the node.
    ///
    /// # Errors
    ///
    /// Returns an error if the node is still started or if there are multiple
    /// references to the node.
    pub async fn destroy(self) -> Result<()> {
        if Arc::strong_count(&self.inner) != 1 {
            return Err(StorageError::node_error(
                "destroy",
                "Cannot destroy: multiple references exist",
            ));
        }

        {
            let inner = self.inner.lock().unwrap();
            if inner.started {
                return Err(StorageError::node_error("destroy", "Node is still started"));
            }
        }

        let future = CallbackFuture::new();
        let context_ptr = future.context_ptr();

        let ctx = {
            let inner = self.inner.lock().unwrap();
            inner.ctx as *mut _
        };

        let result = unsafe { storage_close(ctx, Some(c_callback), context_ptr.as_ptr()) };

        if result != 0 {
            return Err(StorageError::node_error("destroy", "Failed to close node"));
        }

        future.await?;

        // storage_destroy is synchronous and takes only the context.
        let result = unsafe { storage_destroy(ctx) };

        // Null the context before reporting the failure, so Drop does not free it again.
        {
            let mut inner = self.inner.lock().unwrap();
            inner.ctx = ptr::null_mut();
        }

        if result != 0 {
            return Err(StorageError::node_error(
                "destroy",
                "Failed to destroy node",
            ));
        }

        Ok(())
    }

    pub async fn destroy_async(self) -> Result<()> {
        self.destroy().await
    }

    /// Get the version of the Storage node
    ///
    /// # Example
    ///
    /// ```no_run
    /// use storage_bindings::{LogLevel, StorageConfig, StorageNode};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = StorageConfig::new()
    ///         .log_level(LogLevel::Info)
    ///         .data_dir("./storage");
    ///
    ///     let node = StorageNode::new(config).await?;
    ///     node.start().await?;
    ///
    ///     let version = node.version().await?;
    ///     println!("Version: {}", version);
    ///
    ///     node.stop().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn version(&self) -> Result<String> {
        let ctx = {
            let inner = self.inner.lock().unwrap();
            inner.ctx as *mut _
        };

        // storage_version is synchronous and returns a heap string the caller must free.
        let c_ptr = with_libstorage_lock(|| unsafe { storage_version(ctx) });
        if c_ptr.is_null() {
            return Err(StorageError::node_error("version", "Failed to get version"));
        }

        let version = unsafe { c_str_to_string(c_ptr) };
        unsafe { libc::free(c_ptr as *mut c_void) };

        version.map_err(|e| StorageError::node_error("version", format!("Invalid UTF-8: {}", e)))
    }

    /// Get the revision of the Storage node
    pub async fn revision(&self) -> Result<String> {
        let ctx = {
            let inner = self.inner.lock().unwrap();
            inner.ctx as *mut _
        };

        // storage_revision is synchronous and returns a heap string the caller must free.
        let c_ptr = with_libstorage_lock(|| unsafe { storage_revision(ctx) });
        if c_ptr.is_null() {
            return Err(StorageError::node_error(
                "revision",
                "Failed to get revision",
            ));
        }

        let revision = unsafe { c_str_to_string(c_ptr) };
        unsafe { libc::free(c_ptr as *mut c_void) };

        revision.map_err(|e| StorageError::node_error("revision", format!("Invalid UTF-8: {}", e)))
    }

    /// Get the repository path of the Storage node
    pub async fn repo(&self) -> Result<String> {
        let node = self.clone();
        let future = CallbackFuture::new();
        let context_ptr = future.context_ptr();

        let ctx = {
            let inner = node.inner.lock().unwrap();
            inner.ctx as *mut _
        };

        let result = unsafe { storage_repo(ctx, Some(c_callback), context_ptr.as_ptr()) };

        if result != 0 {
            return Err(StorageError::node_error("repo", "Failed to get repo path"));
        }

        future.await
    }

    /// Get the SPR (Storage Provider Record) of the Storage node
    pub async fn spr(&self) -> Result<String> {
        let node = self.clone();
        let future = CallbackFuture::new();
        let context_ptr = future.context_ptr();

        let ctx = {
            let inner = node.inner.lock().unwrap();
            inner.ctx as *mut _
        };

        let result = unsafe { storage_spr(ctx, Some(c_callback), context_ptr.as_ptr()) };

        if result != 0 {
            return Err(StorageError::node_error("spr", "Failed to get SPR"));
        }

        future.await
    }

    /// Get the peer ID of the Storage node
    ///
    /// # Example
    ///
    /// ```no_run
    /// use storage_bindings::{LogLevel, StorageConfig, StorageNode};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = StorageConfig::new()
    ///         .log_level(LogLevel::Info)
    ///         .data_dir("./storage");
    ///
    ///     let node = StorageNode::new(config).await?;
    ///     node.start().await?;
    ///
    ///     let peer_id = node.peer_id().await?;
    ///     println!("Peer ID: {}", peer_id);
    ///
    ///     node.stop().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn peer_id(&self) -> Result<String> {
        let node = self.clone();
        let future = CallbackFuture::new();
        let context_ptr = future.context_ptr();

        let ctx = {
            let inner = node.inner.lock().unwrap();
            inner.ctx as *mut _
        };

        let result = unsafe { storage_peer_id(ctx, Some(c_callback), context_ptr.as_ptr()) };

        if result != 0 {
            return Err(StorageError::node_error("peer_id", "Failed to get peer ID"));
        }

        future.await
    }

    /// Get the node metrics in the Logos openmetrics-compatible format
    /// (https://github.com/logos-co/openmetrics-module).
    pub async fn get_metrics(&self) -> Result<String> {
        let future = CallbackFuture::new();
        let context_ptr = future.context_ptr();

        let ctx = {
            let inner = self.inner.lock().unwrap();
            inner.ctx as *mut _
        };

        let result = unsafe { storage_get_metrics(ctx, Some(c_callback), context_ptr.as_ptr()) };

        if result != 0 {
            return Err(StorageError::node_error(
                "get_metrics",
                "Failed to get metrics",
            ));
        }

        future.await
    }

    /// Toggle routing of DHT queries over the Logos mix network.
    ///
    /// When enabled, all subsequent DHT queries are tunnelled over Mix; this
    /// affects queries only, not advertisements.
    ///
    /// Enabling requires Mix to be configured: `mix_enabled` true and at least
    /// one `dht_mix_proxies` set (see [`StorageConfig`]). Otherwise enabling
    /// fails with an error. Disabling is always allowed.
    ///
    /// This is a temporary API and will likely be removed before mainnet.
    ///
    /// On success, returns the previous toggle state (true = private queries
    /// were already enabled).
    pub async fn toggle_private_queries(&self, enabled: bool) -> Result<bool> {
        let future = CallbackFuture::new();
        let context_ptr = future.context_ptr();

        let ctx = {
            let inner = self.inner.lock().unwrap();
            inner.ctx as *mut _
        };

        let result = unsafe {
            storage_toggle_private_queries(ctx, enabled, Some(c_callback), context_ptr.as_ptr())
        };

        if result != 0 {
            return Err(StorageError::node_error(
                "toggle_private_queries",
                "Failed to toggle private queries",
            ));
        }

        let previous = future.await?;
        Ok(previous == "true")
    }

    pub fn is_started(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.started
    }

    #[allow(dead_code)]
    pub(crate) fn ctx(&self) -> *mut c_void {
        let inner = self.inner.lock().unwrap();
        inner.ctx
    }

    pub(crate) fn with_ctx<F, R>(&self, f: F) -> R
    where
        F: FnOnce(*mut c_void) -> R,
    {
        let inner = self.inner.lock().unwrap();
        f(inner.ctx)
    }

    pub(crate) fn with_ctx_locked<F, R>(&self, f: F) -> R
    where
        F: FnOnce(*mut c_void) -> R,
    {
        with_libstorage_lock(|| {
            let inner = self.inner.lock().unwrap();
            f(inner.ctx)
        })
    }
}

impl Drop for StorageNode {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) == 1 {
            let mut inner = self.inner.lock().unwrap();

            // Stop the node if it's started
            if !inner.ctx.is_null() && inner.started {
                unsafe {
                    storage_stop(inner.ctx as *mut _, None, ptr::null_mut());
                }
                inner.started = false;
            }

            // Close the node
            if !inner.ctx.is_null() {
                unsafe {
                    storage_close(inner.ctx as *mut _, None, ptr::null_mut());
                }
            }

            // Destroy the node
            if !inner.ctx.is_null() {
                unsafe {
                    storage_destroy(inner.ctx as *mut _);
                }
                inner.ctx = ptr::null_mut();
            }
        }
    }
}
