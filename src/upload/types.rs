use crate::error::{Result, StorageError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Block size bounds enforced by libstorage (MinBlockSize / MaxBlockSize)
const MIN_BLOCK_SIZE: usize = 4 * 1024;
const MAX_BLOCK_SIZE: usize = 512 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UploadStrategy {
    Chunked,
    Stream,
    #[default]
    Auto,
}

#[derive(Debug, Clone)]
pub struct UploadProgress {
    pub bytes_uploaded: usize,
    pub total_bytes: Option<usize>,
    pub percentage: f64,
    pub current_chunk: Option<usize>,
    pub total_chunks: Option<usize>,
}

impl UploadProgress {
    pub fn new(bytes_uploaded: usize, total_bytes: Option<usize>) -> Self {
        let percentage = if let Some(total) = total_bytes {
            if total > 0 {
                bytes_uploaded as f64 / total as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        Self {
            bytes_uploaded,
            total_bytes,
            percentage: percentage.min(1.0),
            current_chunk: None,
            total_chunks: None,
        }
    }

    pub fn new_chunked(
        bytes_uploaded: usize,
        total_bytes: Option<usize>,
        current_chunk: usize,
        total_chunks: usize,
    ) -> Self {
        let mut progress = Self::new(bytes_uploaded, total_bytes);
        progress.current_chunk = Some(current_chunk);
        progress.total_chunks = Some(total_chunks);
        progress
    }
}

#[derive(Clone)]
pub struct UploadOptions {
    pub filepath: Option<PathBuf>,
    pub chunk_size: Option<usize>,
    pub strategy: UploadStrategy,
    pub on_progress: Option<Arc<dyn Fn(UploadProgress) + Send + Sync>>,
    pub verify: bool,
    pub metadata: Option<serde_json::Value>,
    pub timeout: Option<u64>,
}

impl std::fmt::Debug for UploadOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UploadOptions")
            .field("filepath", &self.filepath)
            .field("chunk_size", &self.chunk_size)
            .field("strategy", &self.strategy)
            .field("on_progress", &self.on_progress.is_some())
            .field("verify", &self.verify)
            .field("metadata", &self.metadata)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl Default for UploadOptions {
    fn default() -> Self {
        Self {
            filepath: None,
            chunk_size: Some(64 * 1024),
            strategy: UploadStrategy::Auto,
            on_progress: None,
            verify: true,
            metadata: None,
            timeout: Some(300),
        }
    }
}

impl UploadOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn filepath<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.filepath = Some(path.into());
        self
    }

    pub fn chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = Some(size);
        self
    }

    pub fn strategy(mut self, strategy: UploadStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(UploadProgress) + Send + Sync + 'static,
    {
        self.on_progress = Some(Arc::new(callback));
        self
    }

    pub fn verify(mut self, verify: bool) -> Self {
        self.verify = verify;
        self
    }

    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(chunk_size) = self.chunk_size {
            if !(MIN_BLOCK_SIZE..=MAX_BLOCK_SIZE).contains(&chunk_size)
                || !chunk_size.is_power_of_two()
            {
                return Err(StorageError::invalid_parameter(
                    "chunk_size",
                    "Chunk size must be a power of two between 4 KiB and 512 KiB",
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

#[derive(Debug, Clone)]
pub struct UploadResult {
    pub cid: String,
    pub size: usize,
    pub chunks: Option<usize>,
    pub duration_ms: u64,
    pub verified: bool,
}

impl UploadResult {
    pub fn new(cid: String, size: usize) -> Self {
        Self {
            cid,
            size,
            chunks: None,
            duration_ms: 0,
            verified: false,
        }
    }

    pub fn chunks(mut self, chunks: usize) -> Self {
        self.chunks = Some(chunks);
        self
    }

    pub fn duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    pub fn verified(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_progress() {
        let progress = UploadProgress::new(500, Some(1000));
        assert_eq!(progress.bytes_uploaded, 500);
        assert_eq!(progress.total_bytes, Some(1000));
        assert_eq!(progress.percentage, 0.5);

        let chunked = UploadProgress::new_chunked(500, Some(1000), 2, 4);
        assert_eq!(chunked.current_chunk, Some(2));
        assert_eq!(chunked.total_chunks, Some(4));
    }

    #[test]
    fn test_upload_options() {
        let options = UploadOptions::new()
            .filepath("/test/file.txt")
            .chunk_size(64 * 1024)
            .strategy(UploadStrategy::Chunked)
            .verify(false)
            .timeout(600);

        assert_eq!(options.filepath, Some(PathBuf::from("/test/file.txt")));
        assert_eq!(options.chunk_size, Some(64 * 1024));
        assert_eq!(options.strategy, UploadStrategy::Chunked);
        assert!(!options.verify);
        assert_eq!(options.timeout, Some(600));
    }

    #[test]
    fn test_upload_options_validation() {
        let mut options = UploadOptions::new();
        assert!(options.validate().is_ok());

        options.chunk_size = Some(0);
        assert!(options.validate().is_err());

        options.chunk_size = Some(1024 * 1024);
        assert!(options.validate().is_err());

        options.chunk_size = Some(100_000);
        assert!(options.validate().is_err());

        options.chunk_size = Some(64 * 1024);
        options.timeout = Some(0);
        assert!(options.validate().is_err());
    }

    #[test]
    fn test_upload_result() {
        let result = UploadResult::new("QmExample".to_string(), 1024)
            .chunks(4)
            .duration_ms(5000)
            .verified(true);

        assert_eq!(result.cid, "QmExample");
        assert_eq!(result.size, 1024);
        assert_eq!(result.chunks, Some(4));
        assert_eq!(result.duration_ms, 5000);
        assert!(result.verified);
    }
}
