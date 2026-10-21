//! High-Performance Asynchronous Verified Downloader.
//!
//! Provides bounded concurrency, in-flight streaming SHA-1/SHA-256 verification,
//! exponential backoff retries, and atomic file swaps.

use crate::branding;
use crate::security::hashes::{verify_file_sha1, verify_file_sha256};
use futures_util::StreamExt;
use reqwest::Client;
use sha1::{Digest as Sha1Digest, Sha1};
use sha2::Sha256;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::Semaphore;
use tracing::warn;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Network transfer error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("I/O error during download: {0}")]
    Io(#[from] std::io::Error),
    #[error("SHA-1 mismatch for {url}: expected {expected}, computed {actual}")]
    Sha1Mismatch {
        url: String,
        expected: String,
        actual: String,
    },
    #[error("SHA-256 mismatch for {url}: expected {expected}, computed {actual}")]
    Sha256Mismatch {
        url: String,
        expected: String,
        actual: String,
    },
    #[error("File size mismatch for {url}: expected {expected} bytes, got {actual} bytes")]
    SizeMismatch {
        url: String,
        expected: u64,
        actual: u64,
    },
    #[error("Download operation cancelled by user")]
    Cancelled,
}

#[derive(Clone, Debug)]
pub struct DownloadTask {
    pub url: String,
    pub destination: PathBuf,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub expected_size: Option<u64>,
    pub display_name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DownloadProgress {
    pub current_item: String,
    pub completed_items: usize,
    pub total_items: usize,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub percent: f32,
}

#[derive(Clone)]
pub struct AsyncDownloader {
    pub concurrency: usize,
    pub timeout: Duration,
    pub max_retries: u32,
    client: Client,
    cancelled: Arc<AtomicBool>,
}

impl AsyncDownloader {
    pub fn new(concurrency: usize, timeout_secs: u64, max_retries: u32) -> Self {
        let client = Client::builder()
            .user_agent(branding::USER_AGENT)
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("Failed to initialize reqwest client for AsyncDownloader");

        Self {
            concurrency,
            timeout: Duration::from_secs(timeout_secs),
            max_retries,
            client,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn reset_cancellation(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Download a single artifact with in-flight hash verification and atomic file swap.
    pub async fn download_file(&self, task: &DownloadTask) -> Result<bool, DownloadError> {
        if self.is_cancelled() {
            return Err(DownloadError::Cancelled);
        }

        // Check if destination exists and matches both size and SHA-1
        if task.destination.exists() {
            let matches_size = if let Some(size) = task.expected_size {
                if let Ok(meta) = fs::metadata(&task.destination) {
                    meta.len() == size
                } else {
                    false
                }
            } else {
                true
            };

            if matches_size {
                if let Some(ref sha1) = task.sha1 {
                    if verify_file_sha1(&task.destination, sha1) {
                        return Ok(true); // Already verified, skip download
                    }
                } else if let Some(ref sha256) = task.sha256 {
                    if verify_file_sha256(&task.destination, sha256) {
                        return Ok(true);
                    }
                } else {
                    return Ok(true);
                }
            }
        }

        if let Some(parent) = task.destination.parent() {
            fs::create_dir_all(parent)?;
        }

        let pid = std::process::id();
        let tmp_dest = task.destination.with_extension(format!("tmp.{}", pid));

        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            if self.is_cancelled() {
                let _ = fs::remove_file(&tmp_dest);
                return Err(DownloadError::Cancelled);
            }

            match self.execute_stream_download(task, &tmp_dest).await {
                Ok(()) => {
                    // Verification success -> Atomic swap
                    if task.destination.exists() {
                        let _ = fs::remove_file(&task.destination);
                    }
                    fs::rename(&tmp_dest, &task.destination)?;
                    return Ok(true);
                }
                Err(e) => {
                    warn!(
                        "Download attempt {}/{} failed for {}: {}",
                        attempt, self.max_retries, task.url, e
                    );
                    let _ = fs::remove_file(&tmp_dest);
                    last_error = Some(e);

                    if attempt < self.max_retries {
                        let backoff = Duration::from_millis(500 * (1 << (attempt - 1)));
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or(DownloadError::Cancelled))
    }

    async fn execute_stream_download(
        &self,
        task: &DownloadTask,
        tmp_dest: &Path,
    ) -> Result<(), DownloadError> {
        let resp = self
            .client
            .get(&task.url)
            .send()
            .await?
            .error_for_status()?;

        let raw_file = File::create(tmp_dest)?;
        let mut file = BufWriter::with_capacity(128 * 1024, raw_file);
        let mut stream = resp.bytes_stream();

        let mut sha1_hasher = task.sha1.as_ref().map(|_| Sha1::new());
        let mut sha256_hasher = task.sha256.as_ref().map(|_| Sha256::new());
        let mut bytes_written: u64 = 0;

        while let Some(chunk_result) = stream.next().await {
            if self.is_cancelled() {
                return Err(DownloadError::Cancelled);
            }

            let chunk = chunk_result?;
            file.write_all(&chunk)?;
            bytes_written += chunk.len() as u64;

            if let Some(ref mut h) = sha1_hasher {
                h.update(&chunk);
            }
            if let Some(ref mut h) = sha256_hasher {
                h.update(&chunk);
            }
        }

        file.flush()?;

        // Verify size
        if let Some(expected_size) = task.expected_size {
            if bytes_written != expected_size {
                return Err(DownloadError::SizeMismatch {
                    url: task.url.clone(),
                    expected: expected_size,
                    actual: bytes_written,
                });
            }
        }

        // Verify SHA-1
        if let Some(ref expected_sha1) = task.sha1 {
            if let Some(hasher) = sha1_hasher {
                let actual = hex::encode(hasher.finalize());
                if !actual.eq_ignore_ascii_case(expected_sha1) {
                    return Err(DownloadError::Sha1Mismatch {
                        url: task.url.clone(),
                        expected: expected_sha1.clone(),
                        actual,
                    });
                }
            }
        }

        // Verify SHA-256
        if let Some(ref expected_sha256) = task.sha256 {
            if let Some(hasher) = sha256_hasher {
                let actual = hex::encode(hasher.finalize());
                if !actual.eq_ignore_ascii_case(expected_sha256) {
                    return Err(DownloadError::Sha256Mismatch {
                        url: task.url.clone(),
                        expected: expected_sha256.clone(),
                        actual,
                    });
                }
            }
        }

        Ok(())
    }

    /// Execute concurrent download of tasks with bounded concurrency.
    pub async fn download_all<F>(
        &self,
        tasks: Vec<DownloadTask>,
        progress_callback: Option<F>,
    ) -> Result<bool, DownloadError>
    where
        F: Fn(DownloadProgress) + Send + Sync + 'static,
    {
        self.reset_cancellation();
        let total_items = tasks.len();
        if total_items == 0 {
            return Ok(true);
        }

        let total_bytes: u64 = tasks.iter().map(|t| t.expected_size.unwrap_or(0)).sum();
        let completed_items = Arc::new(AtomicUsize::new(0));
        let bytes_downloaded = Arc::new(AtomicU64::new(0));
        let progress_cb = progress_callback.map(Arc::new);

        let semaphore = Arc::new(Semaphore::new(self.concurrency));
        let mut handles = Vec::with_capacity(total_items);

        for task in tasks {
            let sem = Arc::clone(&semaphore);
            let dl = self.clone();
            let c_items = Arc::clone(&completed_items);
            let b_dl = Arc::clone(&bytes_downloaded);
            let cb = progress_cb.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.map_err(|_| DownloadError::Cancelled)?;
                if dl.is_cancelled() {
                    return Err(DownloadError::Cancelled);
                }

                dl.download_file(&task).await?;

                let completed = c_items.fetch_add(1, Ordering::SeqCst) + 1;
                let task_bytes = task.expected_size.unwrap_or(0);
                let bytes = b_dl.fetch_add(task_bytes, Ordering::SeqCst) + task_bytes;

                if let Some(ref callback) = cb {
                    let percent = (completed as f32 / total_items as f32) * 100.0;
                    callback(DownloadProgress {
                        current_item: task.display_name.unwrap_or_else(|| {
                            task.destination
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string()
                        }),
                        completed_items: completed,
                        total_items,
                        bytes_downloaded: bytes,
                        total_bytes,
                        percent,
                    });
                }

                Ok::<(), DownloadError>(())
            });

            handles.push(handle);
        }

        for handle in handles {
            match handle.await {
                Ok(res) => res?,
                Err(_) => return Err(DownloadError::Cancelled),
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_skip_already_downloaded_file() {
        let temp = tempdir().unwrap();
        let existing = temp.path().join("already_done.jar");
        let data = b"Existing verified content";
        fs::write(&existing, data).unwrap();

        let sha1 = hex::encode(Sha1::digest(data));

        let task = DownloadTask {
            url: "https://mock.invalid/file.jar".to_string(),
            destination: existing.clone(),
            sha1: Some(sha1),
            sha256: None,
            expected_size: Some(data.len() as u64),
            display_name: Some("test.jar".to_string()),
        };

        let downloader = AsyncDownloader::new(4, 10, 3);
        let result = downloader.download_file(&task).await.unwrap();
        assert!(result);
    }
}
