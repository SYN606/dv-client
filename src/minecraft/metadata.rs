//! Mojang Piston Metadata HTTP Client & Disk Cache.

use super::manifest::{VersionManifest, VersionPackage};
use crate::branding;
use reqwest::Client;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tracing::{info, warn};

pub const OFFICIAL_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
pub const MANIFEST_CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

#[derive(Debug, Error)]
pub enum MetaError {
    #[error("Network request error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("I/O error during metadata cache operation: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Version '{0}' not found in official manifest")]
    VersionNotFound(String),
}

#[derive(Clone, Debug)]
pub struct PistonMetaClient {
    pub cache_dir: PathBuf,
    client: Client,
}

impl PistonMetaClient {
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Self {
        let client = Client::builder()
            .user_agent(branding::USER_AGENT)
            .timeout(Duration::from_secs(20))
            .build()
            .expect("Failed to initialize reqwest HTTP client");

        let cache_path = cache_dir.as_ref().to_path_buf();
        let _ = fs::create_dir_all(&cache_path);

        Self {
            cache_dir: cache_path,
            client,
        }
    }

    pub fn manifest_cache_file(&self) -> PathBuf {
        self.cache_dir.join("version_manifest_v2.json")
    }

    /// Retrieve the official version manifest, leveraging disk cache when fresh.
    pub async fn get_version_manifest(&self, force_refresh: bool) -> Result<VersionManifest, MetaError> {
        let cache_file = self.manifest_cache_file();

        if !force_refresh && cache_file.exists() {
            if let Ok(metadata) = fs::metadata(&cache_file) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                        if elapsed < MANIFEST_CACHE_TTL {
                            if let Ok(content) = fs::read_to_string(&cache_file) {
                                if let Ok(manifest) = serde_json::from_str::<VersionManifest>(&content) {
                                    return Ok(manifest);
                                }
                            }
                        }
                    }
                }
            }
        }

        info!("Fetching official version manifest from {}", OFFICIAL_MANIFEST_URL);
        let resp = self
            .client
            .get(OFFICIAL_MANIFEST_URL)
            .send()
            .await?
            .error_for_status()?;

        let text = resp.text().await?;
        let manifest: VersionManifest = serde_json::from_str(&text)?;

        // Cache manifest to disk
        if let Err(e) = fs::write(&cache_file, &text) {
            warn!("Failed to cache version manifest to {:?}: {}", cache_file, e);
        }

        Ok(manifest)
    }

    /// Retrieve the version package JSON for a specific version ID.
    pub async fn get_version_package(&self, version_id: &str, force_refresh: bool) -> Result<VersionPackage, MetaError> {
        let package_cache = self.cache_dir.join(format!("{}.json", version_id));

        if !force_refresh && package_cache.exists() {
            if let Ok(content) = fs::read_to_string(&package_cache) {
                if let Ok(pkg) = serde_json::from_str::<VersionPackage>(&content) {
                    return Ok(pkg);
                }
            }
        }

        let manifest = self.get_version_manifest(false).await?;
        let version_summary = manifest
            .versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| MetaError::VersionNotFound(version_id.to_string()))?;

        info!("Downloading version package for {} from {}", version_id, version_summary.url);
        let resp = self
            .client
            .get(&version_summary.url)
            .send()
            .await?
            .error_for_status()?;

        let text = resp.text().await?;
        let package: VersionPackage = serde_json::from_str(&text)?;

        if let Err(e) = fs::write(&package_cache, &text) {
            warn!("Failed to cache version package to {:?}: {}", package_cache, e);
        }

        Ok(package)
    }
}
