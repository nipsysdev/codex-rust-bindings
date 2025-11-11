//! Streaming download operations
//!
//! This module contains streaming download implementation.

use crate::callback::{c_callback, CallbackFuture};
use crate::download::basic::download_init;
use crate::download::types::{
    DownloadOptions, DownloadProgress, DownloadResult, DownloadStreamOptions,
};
use crate::error::{CodexError, Result};
use crate::ffi::{codex_download_stream, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use libc::c_void;
use std::io::Write;
use std::ptr;
use std::sync::Arc;

/// Stream download content to a file or writer
///
/// # Arguments
///
/// * `node` - The Codex node
/// * `cid` - The content ID to download
/// * `options` - Download stream options
///
/// # Returns
///
/// The result of the download operation
pub async fn download_stream(
    node: &CodexNode,
    cid: &str,
    options: DownloadStreamOptions,
) -> Result<DownloadResult> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    options.validate()?;

    let start_time = std::time::Instant::now();
    let chunk_size = options.chunk_size.unwrap_or(1024 * 1024); // 1 MB default

    // Use a shared container to store the downloaded data
    use std::sync::Mutex;
    let total_bytes = Arc::new(Mutex::new(0usize));
    let total_bytes_clone = total_bytes.clone();

    // Create file handle if filepath is specified
    let file_handle = if let Some(ref filepath) = options.filepath {
        match std::fs::File::create(filepath) {
            Ok(file) => Some(Arc::new(Mutex::new(Some(file)))),
            Err(e) => {
                return Err(CodexError::Io(e));
            }
        }
    } else {
        None
    };

    // Create a callback future for the operation (similar to Go's bridge context)
    let future = CallbackFuture::new();
    let context = future.context.clone();

    // Set up a progress callback to capture the downloaded data and write it
    let file_handle_clone = file_handle.clone();

    // For the writer, we need to handle it differently since we can't store it directly
    // We'll use a channel to send the data to be written
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let tx_clone = tx.clone();
    let writer_task = if options.writer.is_some() {
        let mut writer = options.writer.unwrap();
        Some(std::thread::spawn(move || {
            while let Ok(chunk) = rx.recv() {
                if let Err(e) = writer.write_all(&chunk) {
                    eprintln!("Failed to write to writer: {}", e);
                    break;
                }
            }
        }))
    } else {
        None
    };

    context.set_progress_callback(move |_len, chunk| {
        if let Some(chunk_bytes) = chunk {
            let mut total = total_bytes_clone.lock().unwrap();
            *total += chunk_bytes.len();

            // Write to file if specified
            if let Some(ref file_handle) = file_handle_clone {
                if let Some(ref mut file) = file_handle.lock().unwrap().as_mut() {
                    use std::io::Write;
                    if let Err(e) = file.write_all(chunk_bytes) {
                        eprintln!("Failed to write to file: {}", e);
                    }
                }
            }

            // Send to writer thread if available
            if let Err(_) = tx_clone.send(chunk_bytes.to_vec()) {
                eprintln!("Failed to send data to writer thread");
            }
        }
    });

    let c_cid = string_to_c_string(cid);
    let c_filepath = if let Some(ref filepath) = options.filepath {
        string_to_c_string(filepath.to_str().unwrap_or(""))
    } else {
        ptr::null_mut()
    };

    // Initialize the download first (required by the C library)
    let download_options = DownloadOptions::new(cid)
        .chunk_size(chunk_size)
        .timeout(options.timeout.unwrap_or(300))
        .verify(options.verify);

    download_init(node, cid, &download_options).await?;

    // Call the C function to stream the download
    let result = unsafe {
        codex_download_stream(
            node.ctx() as *mut _,
            c_cid,
            chunk_size,
            options.local,
            c_filepath,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_cid);
        if !c_filepath.is_null() {
            free_c_string(c_filepath);
        }
    }

    if result != 0 {
        return Err(CodexError::download_error("Failed to download stream"));
    }

    // Wait for the operation to complete
    future.await?;

    // Close the channel to signal the writer thread to finish
    drop(tx);

    // Wait for the writer thread to finish
    if let Some(handle) = writer_task {
        if let Err(e) = handle.join() {
            eprintln!("Writer thread failed: {:?}", e);
        }
    }

    // Flush file handles
    if let Some(file_handle) = file_handle {
        if let Some(ref mut file) = file_handle.lock().unwrap().as_mut() {
            use std::io::Write;
            if let Err(e) = file.flush() {
                eprintln!("Failed to flush file: {}", e);
            }
        }
    }

    let duration = start_time.elapsed();
    let bytes_downloaded = *total_bytes.lock().unwrap();

    let mut result = DownloadResult::new(cid.to_string(), bytes_downloaded)
        .duration_ms(duration.as_millis() as u64)
        .verified(options.verify);

    if let Some(filepath) = options.filepath {
        result = result.filepath(filepath);
    }

    Ok(result)
}

/// A streaming download writer that tracks progress
pub struct StreamingDownloadWriter<W> {
    inner: W,
    bytes_written: usize,
    total_bytes: Option<usize>,
    on_progress: Option<Box<dyn Fn(DownloadProgress) + Send + Sync>>,
}

impl<W> StreamingDownloadWriter<W>
where
    W: Write,
{
    /// Create a new streaming download writer
    ///
    /// # Arguments
    ///
    /// * `writer` - The underlying writer to wrap
    /// * `total_bytes` - Optional total size for progress tracking
    /// * `on_progress` - Optional progress callback
    pub fn new(
        writer: W,
        total_bytes: Option<usize>,
        on_progress: Option<Box<dyn Fn(DownloadProgress) + Send + Sync>>,
    ) -> Self {
        Self {
            inner: writer,
            bytes_written: 0,
            total_bytes,
            on_progress,
        }
    }

    /// Get the current progress
    pub fn progress(&self) -> DownloadProgress {
        let percentage = if let Some(total) = self.total_bytes {
            if total > 0 {
                self.bytes_written as f64 / total as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        DownloadProgress::new(self.bytes_written, self.total_bytes)
            .with_percentage(percentage.min(1.0))
    }

    /// Get the number of bytes written so far
    pub fn bytes_written(&self) -> usize {
        self.bytes_written
    }
}

impl<W> Write for StreamingDownloadWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let bytes_written = self.inner.write(buf)?;

        if bytes_written > 0 {
            self.bytes_written += bytes_written;

            // Call progress callback if provided
            if let Some(ref callback) = self.on_progress {
                let progress = self.progress();
                callback(progress);
            }
        }

        Ok(bytes_written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Extension trait for DownloadProgress to support percentage setting
pub trait DownloadProgressExt {
    /// Set the percentage value
    fn with_percentage(self, percentage: f64) -> Self;
}

impl DownloadProgressExt for DownloadProgress {
    fn with_percentage(mut self, percentage: f64) -> Self {
        self.percentage = percentage;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_download_writer() {
        let buffer = Vec::new();
        let progress_calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let progress_calls_clone = progress_calls.clone();

        let on_progress = Some(Box::new(move |progress: DownloadProgress| {
            progress_calls_clone
                .lock()
                .unwrap()
                .push(progress.bytes_downloaded);
        }) as Box<dyn Fn(DownloadProgress) + Send + Sync>);

        let mut writer = StreamingDownloadWriter::new(buffer, Some(100), on_progress);

        let data = b"Hello, world!";
        writer.write_all(data).unwrap();

        assert_eq!(writer.bytes_written(), data.len());
        assert_eq!(writer.progress().bytes_downloaded, data.len());
        assert_eq!(writer.progress().total_bytes, Some(100));
        assert_eq!(writer.progress().percentage, 0.13); // 13/100 ≈ 0.13

        let calls = progress_calls.lock().unwrap();
        assert!(!calls.is_empty());
        assert_eq!(calls.last().unwrap(), &data.len());
    }

    #[test]
    fn test_download_progress_ext() {
        let progress = DownloadProgress::new(500, Some(1000));
        let with_percentage = progress.with_percentage(0.75);

        assert_eq!(with_percentage.percentage, 0.75);
        assert_eq!(with_percentage.bytes_downloaded, 500);
        assert_eq!(with_percentage.total_bytes, Some(1000));
    }
}
