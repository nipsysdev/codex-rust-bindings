//! Streaming upload operations
//!
//! This module contains streaming-specific upload logic and utilities.

use crate::upload::types::{UploadOptions, UploadProgress};
use std::io::Read;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};

/// A streaming upload reader that wraps any Read implementation
/// and provides progress tracking during upload operations.
pub struct StreamingUploadReader<R> {
    inner: R,
    options: UploadOptions,
    bytes_read: usize,
    total_bytes: Option<usize>,
    chunk_count: usize,
}

impl<R> StreamingUploadReader<R>
where
    R: Read,
{
    /// Create a new streaming upload reader
    ///
    /// # Arguments
    ///
    /// * `reader` - The underlying reader to wrap
    /// * `options` - Upload options with progress callback
    /// * `total_bytes` - Optional total size for progress tracking
    pub fn new(reader: R, options: UploadOptions, total_bytes: Option<usize>) -> Self {
        Self {
            inner: reader,
            options,
            bytes_read: 0,
            total_bytes,
            chunk_count: 0,
        }
    }

    /// Get the current progress
    pub fn progress(&self) -> UploadProgress {
        let percentage = if let Some(total) = self.total_bytes {
            if total > 0 {
                self.bytes_read as f64 / total as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        UploadProgress::new_chunked(
            self.bytes_read,
            self.total_bytes,
            self.chunk_count,
            self.chunk_count,
        )
        .with_percentage(percentage.min(1.0))
    }

    /// Get the number of bytes read so far
    pub fn bytes_read(&self) -> usize {
        self.bytes_read
    }

    /// Get the number of chunks processed
    pub fn chunk_count(&self) -> usize {
        self.chunk_count
    }
}

impl<R> Read for StreamingUploadReader<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = self.inner.read(buf)?;

        if bytes_read > 0 {
            self.bytes_read += bytes_read;
            self.chunk_count += 1;

            // Call progress callback if provided
            if let Some(ref callback) = self.options.on_progress {
                let progress = self.progress();
                callback(progress);
            }
        }

        Ok(bytes_read)
    }
}

/// An async version of the streaming upload reader
pub struct AsyncStreamingUploadReader<R> {
    inner: Pin<Box<R>>,
    options: UploadOptions,
    bytes_read: usize,
    total_bytes: Option<usize>,
    chunk_count: usize,
}

impl<R> AsyncStreamingUploadReader<R>
where
    R: AsyncRead + Unpin,
{
    /// Create a new async streaming upload reader
    ///
    /// # Arguments
    ///
    /// * `reader` - The underlying async reader to wrap
    /// * `options` - Upload options with progress callback
    /// * `total_bytes` - Optional total size for progress tracking
    pub fn new(reader: R, options: UploadOptions, total_bytes: Option<usize>) -> Self {
        Self {
            inner: Box::pin(reader),
            options,
            bytes_read: 0,
            total_bytes,
            chunk_count: 0,
        }
    }

    /// Get the current progress
    pub fn progress(&self) -> UploadProgress {
        let percentage = if let Some(total) = self.total_bytes {
            if total > 0 {
                self.bytes_read as f64 / total as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        UploadProgress::new_chunked(
            self.bytes_read,
            self.total_bytes,
            self.chunk_count,
            self.chunk_count,
        )
        .with_percentage(percentage.min(1.0))
    }

    /// Get the number of bytes read so far
    pub fn bytes_read(&self) -> usize {
        self.bytes_read
    }

    /// Get the number of chunks processed
    pub fn chunk_count(&self) -> usize {
        self.chunk_count
    }
}

impl<R> AsyncRead for AsyncStreamingUploadReader<R>
where
    R: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let initial_len = buf.filled().len();

        match Pin::new(&mut self.inner).poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                let bytes_read = buf.filled().len() - initial_len;

                if bytes_read > 0 {
                    self.bytes_read += bytes_read;
                    self.chunk_count += 1;

                    // Call progress callback if provided
                    if let Some(ref callback) = self.options.on_progress {
                        let progress = self.progress();
                        callback(progress);
                    }
                }

                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Extension trait for UploadProgress to support percentage setting
pub trait UploadProgressExt {
    /// Set the percentage value
    fn with_percentage(self, percentage: f64) -> Self;
}

impl UploadProgressExt for UploadProgress {
    fn with_percentage(mut self, percentage: f64) -> Self {
        self.percentage = percentage;
        self
    }
}

/// Utility function to create a streaming upload reader with automatic chunk size detection
pub fn create_streaming_reader<R>(
    reader: R,
    options: UploadOptions,
    total_size: Option<usize>,
) -> StreamingUploadReader<R>
where
    R: Read,
{
    // Adjust chunk size based on total size if provided
    let adjusted_options = if let Some(size) = total_size {
        let optimal_chunk_size = (size / 100).clamp(64 * 1024, 512 * 1024);

        let mut opts = options;
        opts.chunk_size = Some(optimal_chunk_size);
        opts
    } else {
        options
    };

    StreamingUploadReader::new(reader, adjusted_options, total_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_streaming_upload_reader() {
        let data = b"Hello, world!";
        let reader = Cursor::new(data);

        let progress_calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let progress_calls_clone = progress_calls.clone();

        let options = UploadOptions::new().on_progress(move |progress| {
            progress_calls_clone
                .lock()
                .unwrap()
                .push(progress.bytes_uploaded);
        });

        let mut streaming_reader = StreamingUploadReader::new(reader, options, Some(data.len()));

        let mut buffer = [0u8; 5];
        let bytes_read = streaming_reader.read(&mut buffer).unwrap();
        assert_eq!(bytes_read, 5);
        assert_eq!(streaming_reader.bytes_read(), 5);
        assert_eq!(streaming_reader.chunk_count(), 1);

        let bytes_read = streaming_reader.read(&mut buffer).unwrap();
        assert_eq!(bytes_read, 5);
        assert_eq!(streaming_reader.bytes_read(), 10);
        assert_eq!(streaming_reader.chunk_count(), 2);

        let bytes_read = streaming_reader.read(&mut buffer).unwrap();
        assert_eq!(bytes_read, 3);
        assert_eq!(streaming_reader.bytes_read(), 13);
        assert_eq!(streaming_reader.chunk_count(), 3);

        // Check progress callbacks were called
        let calls = progress_calls.lock().unwrap();
        assert_eq!(calls.len(), 3);
        assert_eq!(calls[0], 5);
        assert_eq!(calls[1], 10);
        assert_eq!(calls[2], 13);
    }

    #[test]
    fn test_streaming_upload_reader_progress() {
        let data = b"Hello, world!";
        let reader = Cursor::new(data);
        let options = UploadOptions::new();

        let streaming_reader = StreamingUploadReader::new(reader, options, Some(data.len()));

        let progress = streaming_reader.progress();
        assert_eq!(progress.bytes_uploaded, 0);
        assert_eq!(progress.total_bytes, Some(data.len()));
        assert_eq!(progress.percentage, 0.0);
        assert_eq!(progress.current_chunk, Some(0));
        assert_eq!(progress.total_chunks, Some(0));
    }

    #[test]
    fn test_create_streaming_reader() {
        let data = b"Hello, world!";
        let reader = Cursor::new(data);
        let options = UploadOptions::new().chunk_size(1024);

        let streaming_reader = create_streaming_reader(reader, options, Some(data.len()));

        // Should have adjusted chunk size based on total size
        assert_eq!(streaming_reader.bytes_read(), 0);
        assert_eq!(streaming_reader.chunk_count(), 0);
    }

    #[tokio::test]
    async fn test_async_streaming_upload_reader() {
        use tokio::io::AsyncReadExt;

        let data = b"Hello, world!";
        let reader = tokio::io::BufReader::new(Cursor::new(data));

        let options = UploadOptions::new();
        let mut async_reader = AsyncStreamingUploadReader::new(reader, options, Some(data.len()));

        let mut buffer = [0u8; 13];
        let bytes_read = async_reader.read(&mut buffer).await.unwrap();
        assert_eq!(bytes_read, 13);
        assert_eq!(async_reader.bytes_read, 13);
        assert_eq!(async_reader.chunk_count, 1);

        let progress = async_reader.progress();
        assert_eq!(progress.bytes_uploaded, 13);
        assert_eq!(progress.total_bytes, Some(data.len()));
        assert_eq!(progress.percentage, 1.0);
    }

    #[test]
    fn test_upload_progress_ext() {
        let progress = UploadProgress::new(500, Some(1000));
        let with_percentage = progress.with_percentage(0.75);

        assert_eq!(with_percentage.percentage, 0.75);
        assert_eq!(with_percentage.bytes_uploaded, 500);
        assert_eq!(with_percentage.total_bytes, Some(1000));
    }
}
