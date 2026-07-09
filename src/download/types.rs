//! Types for download operations

use crate::error::{Result, StorageError};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

/// Progress information for download operations
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Number of bytes downloaded
    pub bytes_downloaded: usize,
    /// Total number of bytes to download (if known)
    pub total_bytes: Option<usize>,
    /// Progress percentage (0.0 to 1.0)
    pub percentage: f64,
    /// Current chunk being downloaded (if applicable)
    pub current_chunk: Option<usize>,
    /// Total number of chunks (if applicable)
    pub total_chunks: Option<usize>,
    /// Download speed in bytes per second (if available)
    pub speed_bps: Option<f64>,
}

impl DownloadProgress {
    /// Create new download progress
    pub fn new(bytes_downloaded: usize, total_bytes: Option<usize>) -> Self {
        let percentage = if let Some(total) = total_bytes {
            if total > 0 {
                bytes_downloaded as f64 / total as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        Self {
            bytes_downloaded,
            total_bytes,
            percentage: percentage.min(1.0),
            current_chunk: None,
            total_chunks: None,
            speed_bps: None,
        }
    }

    /// Create new chunked download progress
    pub fn new_chunked(
        bytes_downloaded: usize,
        total_bytes: Option<usize>,
        current_chunk: usize,
        total_chunks: usize,
    ) -> Self {
        let mut progress = Self::new(bytes_downloaded, total_bytes);
        progress.current_chunk = Some(current_chunk);
        progress.total_chunks = Some(total_chunks);
        progress
    }

    /// Set the download speed
    pub fn with_speed(mut self, speed_bps: f64) -> Self {
        self.speed_bps = Some(speed_bps);
        self
    }
}

/// Options for download operations
pub struct DownloadOptions {
    /// Content ID (CID) to download
    pub cid: String,
    /// Chunk size for chunked downloads (in bytes)
    pub chunk_size: Option<usize>,
    /// Progress callback function
    pub on_progress: Option<Box<dyn Fn(DownloadProgress) + Send + Sync>>,
    /// Timeout for the download operation (in seconds)
    pub timeout: Option<u64>,
    /// Whether to verify the download after completion
    pub verify: bool,
}

impl std::fmt::Debug for DownloadOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadOptions")
            .field("cid", &self.cid)
            .field("chunk_size", &self.chunk_size)
            .field("on_progress", &self.on_progress.is_some())
            .field("timeout", &self.timeout)
            .field("verify", &self.verify)
            .finish()
    }
}

impl Clone for DownloadOptions {
    fn clone(&self) -> Self {
        Self {
            cid: self.cid.clone(),
            chunk_size: self.chunk_size,
            on_progress: None, // Cannot clone callback
            timeout: self.timeout,
            verify: self.verify,
        }
    }
}

impl serde::Serialize for DownloadOptions {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DownloadOptions", 5)?;
        state.serialize_field("cid", &self.cid)?;
        state.serialize_field("chunk_size", &self.chunk_size)?;
        state.serialize_field("timeout", &self.timeout)?;
        state.serialize_field("verify", &self.verify)?;
        state.end()
    }
}

impl DownloadOptions {
    /// Create new download options
    pub fn new(cid: impl Into<String>) -> Self {
        Self {
            cid: cid.into(),
            chunk_size: Some(64 * 1024), // 64 KiB default (nim MaxBlockSize is 512 KiB)
            on_progress: None,
            timeout: Some(300), // 5 minutes default
            verify: true,
        }
    }

    /// Set the chunk size for chunked downloads
    pub fn chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = Some(size);
        self
    }

    /// Set the progress callback
    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(DownloadProgress) + Send + Sync + 'static,
    {
        self.on_progress = Some(Box::new(callback));
        self
    }

    /// Set the timeout
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set whether to verify the download
    pub fn verify(mut self, verify: bool) -> Self {
        self.verify = verify;
        self
    }

    /// Validate the download options
    pub fn validate(&self) -> Result<()> {
        if self.cid.is_empty() {
            return Err(StorageError::invalid_parameter(
                "cid",
                "CID cannot be empty",
            ));
        }

        if let Some(chunk_size) = self.chunk_size {
            if chunk_size == 0 {
                return Err(StorageError::invalid_parameter(
                    "chunk_size",
                    "Chunk size must be greater than 0",
                ));
            }
        }

        if let Some(timeout) = self.timeout {
            if timeout == 0 {
                return Err(StorageError::invalid_parameter(
                    "timeout",
                    "Timeout must be greater than 0",
                ));
            }
        }

        Ok(())
    }
}

/// Options for streaming downloads
pub struct DownloadStreamOptions {
    /// Content ID (CID) to download
    pub cid: String,
    /// Path to save the downloaded file (optional)
    pub filepath: Option<PathBuf>,
    /// Writer to write the downloaded data to (optional)
    pub writer: Option<Box<dyn Write + Send>>,
    /// Chunk size for streaming (in bytes)
    pub chunk_size: Option<usize>,
    /// Progress callback function
    pub on_progress: Option<Box<dyn Fn(DownloadProgress) + Send + Sync>>,
    /// Whether to download locally only (don't fetch from network)
    pub local: bool,
    /// Expected dataset size (for progress tracking)
    pub dataset_size: Option<usize>,
    /// Whether to auto-detect dataset size
    pub dataset_size_auto: bool,
    /// Timeout for the download operation (in seconds)
    pub timeout: Option<u64>,
    /// Whether to verify the download after completion
    pub verify: bool,
}

impl std::fmt::Debug for DownloadStreamOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadStreamOptions")
            .field("cid", &self.cid)
            .field("filepath", &self.filepath)
            .field("writer", &self.writer.is_some())
            .field("chunk_size", &self.chunk_size)
            .field("on_progress", &self.on_progress.is_some())
            .field("local", &self.local)
            .field("dataset_size", &self.dataset_size)
            .field("dataset_size_auto", &self.dataset_size_auto)
            .field("timeout", &self.timeout)
            .field("verify", &self.verify)
            .finish()
    }
}

impl Clone for DownloadStreamOptions {
    fn clone(&self) -> Self {
        Self {
            cid: self.cid.clone(),
            filepath: self.filepath.clone(),
            writer: None, // Cannot clone writer
            chunk_size: self.chunk_size,
            on_progress: None, // Cannot clone callback
            local: self.local,
            dataset_size: self.dataset_size,
            dataset_size_auto: self.dataset_size_auto,
            timeout: self.timeout,
            verify: self.verify,
        }
    }
}

impl DownloadStreamOptions {
    /// Create new download stream options
    pub fn new(cid: impl Into<String>) -> Self {
        Self {
            cid: cid.into(),
            filepath: None,
            writer: None,
            chunk_size: Some(64 * 1024), // 64 KiB default (nim MaxBlockSize is 512 KiB)
            on_progress: None,
            local: false,
            dataset_size: None,
            dataset_size_auto: true,
            timeout: Some(300), // 5 minutes default
            verify: true,
        }
    }

    /// Set the file path to save to
    pub fn filepath<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.filepath = Some(path.into());
        self
    }

    /// Set the writer to write to
    pub fn writer<W: Write + Send + 'static>(mut self, writer: W) -> Self {
        self.writer = Some(Box::new(writer));
        self
    }

    /// Set the chunk size
    pub fn chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = Some(size);
        self
    }

    /// Set the progress callback
    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(DownloadProgress) + Send + Sync + 'static,
    {
        self.on_progress = Some(Box::new(callback));
        self
    }

    /// Set whether to download locally only
    pub fn local(mut self, local: bool) -> Self {
        self.local = local;
        self
    }

    /// Set the expected dataset size
    pub fn dataset_size(mut self, size: usize) -> Self {
        self.dataset_size = Some(size);
        self
    }

    /// Set whether to auto-detect dataset size
    pub fn dataset_size_auto(mut self, auto: bool) -> Self {
        self.dataset_size_auto = auto;
        self
    }

    /// Set the timeout
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set whether to verify the download
    pub fn verify(mut self, verify: bool) -> Self {
        self.verify = verify;
        self
    }

    /// Validate the download stream options
    pub fn validate(&self) -> Result<()> {
        if self.cid.is_empty() {
            return Err(StorageError::invalid_parameter(
                "cid",
                "CID cannot be empty",
            ));
        }

        if self.filepath.is_none() && self.writer.is_none() {
            return Err(StorageError::invalid_parameter(
                "filepath/writer",
                "Either filepath or writer must be specified",
            ));
        }

        if let Some(chunk_size) = self.chunk_size {
            if chunk_size == 0 {
                return Err(StorageError::invalid_parameter(
                    "chunk_size",
                    "Chunk size must be greater than 0",
                ));
            }
        }

        if let Some(timeout) = self.timeout {
            if timeout == 0 {
                return Err(StorageError::invalid_parameter(
                    "timeout",
                    "Timeout must be greater than 0",
                ));
            }
        }

        Ok(())
    }
}

/// Result of a download operation
#[derive(Debug, Clone)]
pub struct DownloadResult {
    /// Content ID (CID) of the downloaded content
    pub cid: String,
    /// Size of the downloaded content in bytes
    pub size: usize,
    /// Number of chunks downloaded (if applicable)
    pub chunks: Option<usize>,
    /// Time taken for the download (in milliseconds)
    pub duration_ms: u64,
    /// Whether the download was verified (if verification was requested)
    pub verified: bool,
    /// Path where the file was saved (if applicable)
    pub filepath: Option<PathBuf>,
}

impl DownloadResult {
    /// Create a new download result
    pub fn new(cid: String, size: usize) -> Self {
        Self {
            cid,
            size,
            chunks: None,
            duration_ms: 0,
            verified: false,
            filepath: None,
        }
    }

    /// Set the number of chunks
    pub fn chunks(mut self, chunks: usize) -> Self {
        self.chunks = Some(chunks);
        self
    }

    /// Set the duration
    pub fn duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Set whether the download was verified
    pub fn verified(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }

    /// Set the file path
    pub fn filepath<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.filepath = Some(path.into());
        self
    }
}

/// Manifest information for a stored content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Content ID (CID)
    pub cid: String,
    /// Size of the content in bytes
    pub size: usize,
    /// Number of blocks
    pub blocks: usize,
    /// Creation timestamp
    pub created: String,
    /// Last access timestamp
    pub accessed: Option<String>,
    /// Content type/mime type
    pub content_type: Option<String>,
    /// Custom metadata
    pub metadata: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_progress() {
        let progress = DownloadProgress::new(500, Some(1000));
        assert_eq!(progress.bytes_downloaded, 500);
        assert_eq!(progress.total_bytes, Some(1000));
        assert_eq!(progress.percentage, 0.5);

        let chunked = DownloadProgress::new_chunked(500, Some(1000), 2, 4);
        assert_eq!(chunked.current_chunk, Some(2));
        assert_eq!(chunked.total_chunks, Some(4));

        let with_speed = chunked.with_speed(1024.0);
        assert_eq!(with_speed.speed_bps, Some(1024.0));
    }

    #[test]
    fn test_download_options() {
        let options = DownloadOptions::new("QmExample")
            .chunk_size(2048)
            .timeout(600)
            .verify(false);

        assert_eq!(options.cid, "QmExample");
        assert_eq!(options.chunk_size, Some(2048));
        assert_eq!(options.timeout, Some(600));
        assert!(!options.verify);
    }

    #[test]
    fn test_download_options_validation() {
        let mut options = DownloadOptions::new("QmExample");
        assert!(options.validate().is_ok());

        options.cid = "".to_string();
        assert!(options.validate().is_err());

        options.cid = "QmExample".to_string();
        options.chunk_size = Some(0);
        assert!(options.validate().is_err());

        options.chunk_size = Some(1024);
        options.timeout = Some(0);
        assert!(options.validate().is_err());
    }

    #[test]
    fn test_download_stream_options() {
        let options = DownloadStreamOptions::new("QmExample")
            .filepath("/test/output.txt")
            .chunk_size(2048)
            .local(true)
            .dataset_size(1024)
            .dataset_size_auto(false)
            .timeout(600)
            .verify(false);

        assert_eq!(options.cid, "QmExample");
        assert_eq!(options.filepath, Some(PathBuf::from("/test/output.txt")));
        assert_eq!(options.chunk_size, Some(2048));
        assert!(options.local);
        assert_eq!(options.dataset_size, Some(1024));
        assert!(!options.dataset_size_auto);
        assert_eq!(options.timeout, Some(600));
        assert!(!options.verify);
    }

    #[test]
    fn test_download_stream_options_validation() {
        let mut options = DownloadStreamOptions::new("QmExample");
        assert!(options.validate().is_err()); // No filepath or writer

        options.filepath = Some(PathBuf::from("/test/output.txt"));
        assert!(options.validate().is_ok());

        options.cid = "".to_string();
        assert!(options.validate().is_err());

        options.cid = "QmExample".to_string();
        options.chunk_size = Some(0);
        assert!(options.validate().is_err());
    }

    #[test]
    fn test_download_result() {
        let result = DownloadResult::new("QmExample".to_string(), 1024)
            .chunks(4)
            .duration_ms(5000)
            .verified(true)
            .filepath("/test/downloaded.txt");

        assert_eq!(result.cid, "QmExample");
        assert_eq!(result.size, 1024);
        assert_eq!(result.chunks, Some(4));
        assert_eq!(result.duration_ms, 5000);
        assert!(result.verified);
        assert_eq!(result.filepath, Some(PathBuf::from("/test/downloaded.txt")));
    }
}
