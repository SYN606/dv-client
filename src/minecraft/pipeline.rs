//! Minecraft End-to-End Installation & Asset Pipeline.
//!
//! Orchestrates the download, cryptographic verification, and local setup of:
//! 1. Version package JSON metadata
//! 2. Client game JAR
//! 3. Platform-specific libraries
//! 4. LWJGL native binaries (extracted with Zip Slip safety)
//! 5. Asset index JSON & individual asset objects

use super::downloads::{AsyncDownloader, DownloadProgress, DownloadTask};
use super::manifest::{evaluate_rules, VersionPackage};
use super::metadata::PistonMetaClient;
use crate::config::LauncherConfig;
use crate::security::paths::extract_natives;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::info;

pub const MOJANG_ASSET_BASE: &str = "https://resources.download.minecraft.net";

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Metadata error: {0}")]
    Meta(#[from] super::metadata::MetaError),
    #[error("Download error: {0}")]
    Download(#[from] super::downloads::DownloadError),
    #[error("Extraction error: {0}")]
    Extraction(#[from] crate::security::paths::ExtractionError),
    #[error("I/O error during installation: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
}

pub struct InstallationPipeline {
    downloader: AsyncDownloader,
    meta_client: PistonMetaClient,
}

impl InstallationPipeline {
    pub fn new(config: &LauncherConfig) -> Self {
        let downloader = AsyncDownloader::new(
            config.downloads.max_concurrent_downloads,
            config.downloads.request_timeout_seconds,
            config.downloads.max_retries,
        );
        let meta_client = PistonMetaClient::new(config.shared_versions_dir().join("meta_cache"));

        Self {
            downloader,
            meta_client,
        }
    }

    /// Complete end-to-end installation of a Minecraft version into an instance.
    pub async fn install_version<P: AsRef<Path>, F, S>(
        &self,
        version_id: &str,
        instance_dir: P,
        config: &LauncherConfig,
        progress_cb: Option<F>,
        status_cb: Option<S>,
    ) -> Result<VersionPackage, PipelineError>
    where
        F: Fn(DownloadProgress) + Send + Sync + 'static,
        S: Fn(&str) + Send + Sync + 'static,
    {
        let set_status = |msg: &str| {
            if let Some(ref cb) = status_cb {
                cb(msg);
            }
            info!("{}", msg);
        };

        set_status(&format!(
            "Resolving official version metadata for {}...",
            version_id
        ));
        let package = self
            .meta_client
            .get_version_package(version_id, false)
            .await?;

        let version_dir = config.shared_versions_dir().join(version_id);
        fs::create_dir_all(&version_dir)?;

        let instance_dir = instance_dir.as_ref();
        let natives_dir = instance_dir.join("natives");
        fs::create_dir_all(&natives_dir)?;

        let mut download_tasks: Vec<DownloadTask> = Vec::new();

        // 1. Client JAR
        let client_jar_dest = version_dir.join(format!("{}.jar", version_id));
        download_tasks.push(DownloadTask {
            url: package.downloads.client.url.clone(),
            destination: client_jar_dest,
            sha1: Some(package.downloads.client.sha1.clone()),
            sha256: None,
            expected_size: Some(package.downloads.client.size),
            display_name: Some(format!("Client JAR ({}.jar)", version_id)),
        });

        // 2. Libraries & Native Archives
        let mut native_archives_to_extract: Vec<PathBuf> = Vec::new();
        let current_os_key = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "osx"
        } else {
            "linux"
        };

        for lib in &package.libraries {
            if !evaluate_rules(lib.rules.as_deref()) {
                continue;
            }

            if let Some(ref downloads) = lib.downloads {
                // Standard library JAR
                if let Some(ref art) = downloads.artifact {
                    let rel_path_opt = art
                        .path
                        .as_deref()
                        .or_else(|| art.url.split("libraries.minecraft.net/").nth(1));

                    if let Some(rel_path) = rel_path_opt {
                        let dest = config.shared_libraries_dir().join(rel_path);
                        download_tasks.push(DownloadTask {
                            url: art.url.clone(),
                            destination: dest,
                            sha1: Some(art.sha1.clone()),
                            sha256: None,
                            expected_size: Some(art.size),
                            display_name: Some(lib.name.clone()),
                        });
                    }
                }

                // Native library classifiers
                if let (Some(ref natives_map), Some(ref classifiers)) =
                    (&lib.natives, &downloads.classifiers)
                {
                    if let Some(classifier_tpl) = natives_map.get(current_os_key) {
                        let arch_str = if cfg!(target_pointer_width = "64") {
                            "64"
                        } else {
                            "32"
                        };
                        let classifier_key = classifier_tpl.replace("${arch}", arch_str);
                        if let Some(native_art) = classifiers.get(&classifier_key) {
                            let rel_path_opt = native_art.path.as_deref().or_else(|| {
                                native_art.url.split("libraries.minecraft.net/").nth(1)
                            });

                            if let Some(rel_path) = rel_path_opt {
                                let dest = config.shared_libraries_dir().join(rel_path);
                                download_tasks.push(DownloadTask {
                                    url: native_art.url.clone(),
                                    destination: dest.clone(),
                                    sha1: Some(native_art.sha1.clone()),
                                    sha256: None,
                                    expected_size: Some(native_art.size),
                                    display_name: Some(format!("Natives ({})", classifier_key)),
                                });
                                native_archives_to_extract.push(dest);
                            }
                        }
                    }
                }
            }
        }

        // 3. Asset Index
        set_status(&format!(
            "Fetching asset index for {}...",
            package.asset_index.id
        ));
        let index_dir = config.shared_assets_dir().join("indexes");
        fs::create_dir_all(&index_dir)?;
        let index_file = index_dir.join(format!("{}.json", package.asset_index.id));

        if !index_file.exists()
            || !crate::security::hashes::verify_file_sha1(&index_file, &package.asset_index.sha1)
        {
            let task = DownloadTask {
                url: package.asset_index.url.clone(),
                destination: index_file.clone(),
                sha1: Some(package.asset_index.sha1.clone()),
                sha256: None,
                expected_size: Some(package.asset_index.size),
                display_name: Some(format!("Asset Index ({}.json)", package.asset_index.id)),
            };
            self.downloader.download_file(&task).await?;
        }

        // Parse asset objects from index
        if let Ok(content) = fs::read_to_string(&index_file) {
            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                if let Some(objects) = val.get("objects").and_then(|o| o.as_object()) {
                    for (name, obj) in objects {
                        if let (Some(hash), Some(size)) = (
                            obj.get("hash").and_then(|h| h.as_str()),
                            obj.get("size").and_then(|s| s.as_u64()),
                        ) {
                            if hash.len() >= 2 {
                                let prefix = &hash[..2];
                                let obj_url = format!("{}/{}/{}", MOJANG_ASSET_BASE, prefix, hash);
                                let obj_dest = config
                                    .shared_assets_dir()
                                    .join("objects")
                                    .join(prefix)
                                    .join(hash);

                                download_tasks.push(DownloadTask {
                                    url: obj_url,
                                    destination: obj_dest,
                                    sha1: Some(hash.to_string()),
                                    sha256: None,
                                    expected_size: Some(size),
                                    display_name: Some(name.clone()),
                                });
                            }
                        }
                    }
                }
            }
        }

        // 4. Download all artifacts with progress reporting
        set_status(&format!(
            "Downloading {} required components...",
            download_tasks.len()
        ));
        self.downloader
            .download_all(download_tasks, progress_cb)
            .await?;

        // 5. Extract native libraries safely
        set_status(&format!(
            "Extracting native libraries into {:?}...",
            natives_dir
        ));
        for archive in &native_archives_to_extract {
            if archive.exists() {
                let _ = extract_natives(archive, &natives_dir, &[]);
            }
        }

        set_status("Installation, verification, and extraction complete!");
        Ok(package)
    }

    /// Verifies all game files (client JAR and required libraries) against Mojang SHA-1 hashes.
    /// Missing or corrupted files are immediately re-downloaded. Returns (total_files_checked, repaired_count).
    pub async fn verify_version_files<F, S>(
        &self,
        version_id: &str,
        config: &LauncherConfig,
        progress_cb: Option<F>,
        status_cb: Option<S>,
    ) -> Result<(usize, usize), PipelineError>
    where
        F: Fn(DownloadProgress) + Send + Sync + 'static,
        S: Fn(&str) + Send + Sync + 'static,
    {
        let set_status = |msg: &str| {
            if let Some(ref cb) = status_cb {
                cb(msg);
            }
            info!("{}", msg);
        };

        set_status(&format!(
            "Verifying client package for Minecraft {}...",
            version_id
        ));
        let package = self
            .meta_client
            .get_version_package(version_id, false)
            .await?;

        let version_dir = config.shared_versions_dir().join(version_id);
        fs::create_dir_all(&version_dir)?;

        let mut download_tasks: Vec<DownloadTask> = Vec::new();
        let mut total_files_checked = 0;

        // Check Client JAR
        total_files_checked += 1;
        let client_jar_dest = version_dir.join(format!("{}.jar", version_id));
        let client_valid = client_jar_dest.exists()
            && crate::security::hashes::verify_file_sha1(
                &client_jar_dest,
                &package.downloads.client.sha1,
            );
        if !client_valid {
            download_tasks.push(DownloadTask {
                url: package.downloads.client.url.clone(),
                destination: client_jar_dest,
                sha1: Some(package.downloads.client.sha1.clone()),
                sha256: None,
                expected_size: Some(package.downloads.client.size),
                display_name: Some(format!("Client JAR ({}.jar)", version_id)),
            });
        }

        // Check Libraries
        for lib in &package.libraries {
            if !evaluate_rules(lib.rules.as_deref()) {
                continue;
            }
            if let Some(ref downloads) = lib.downloads {
                if let Some(ref art) = downloads.artifact {
                    total_files_checked += 1;
                    let rel_path_opt = art
                        .path
                        .as_deref()
                        .or_else(|| art.url.split("libraries.minecraft.net/").nth(1));

                    if let Some(rel_path) = rel_path_opt {
                        let dest = config.shared_libraries_dir().join(rel_path);
                        let is_valid = dest.exists()
                            && crate::security::hashes::verify_file_sha1(&dest, &art.sha1);
                        if !is_valid {
                            download_tasks.push(DownloadTask {
                                url: art.url.clone(),
                                destination: dest,
                                sha1: Some(art.sha1.clone()),
                                sha256: None,
                                expected_size: Some(art.size),
                                display_name: Some(lib.name.clone()),
                            });
                        }
                    }
                }
            }
        }

        let repaired_count = download_tasks.len();
        if !download_tasks.is_empty() {
            set_status(&format!(
                "Found {} missing or corrupted files. Re-downloading...",
                repaired_count
            ));
            self.downloader
                .download_all(download_tasks, progress_cb)
                .await?;
            set_status(&format!(
                "Verification complete: Checked {} files, repaired {} corrupted/missing files.",
                total_files_checked, repaired_count
            ));
        } else {
            set_status(&format!(
                "Verification complete: All {} files verified and intact (100% valid SHA-1)!",
                total_files_checked
            ));
        }

        Ok((total_files_checked, repaired_count))
    }

    /// Verifies and repairs the asset index and all asset objects (sounds, textures, fonts, languages).
    /// Returns (total_assets_checked, repaired_count).
    pub async fn repair_version_assets<F, S>(
        &self,
        version_id: &str,
        config: &LauncherConfig,
        progress_cb: Option<F>,
        status_cb: Option<S>,
    ) -> Result<(usize, usize), PipelineError>
    where
        F: Fn(DownloadProgress) + Send + Sync + 'static,
        S: Fn(&str) + Send + Sync + 'static,
    {
        let set_status = |msg: &str| {
            if let Some(ref cb) = status_cb {
                cb(msg);
            }
            info!("{}", msg);
        };

        set_status(&format!(
            "Resolving asset index for Minecraft {}...",
            version_id
        ));
        let package = self
            .meta_client
            .get_version_package(version_id, false)
            .await?;

        let index_dir = config.shared_assets_dir().join("indexes");
        fs::create_dir_all(&index_dir)?;
        let index_file = index_dir.join(format!("{}.json", package.asset_index.id));

        if !index_file.exists()
            || !crate::security::hashes::verify_file_sha1(&index_file, &package.asset_index.sha1)
        {
            let task = DownloadTask {
                url: package.asset_index.url.clone(),
                destination: index_file.clone(),
                sha1: Some(package.asset_index.sha1.clone()),
                sha256: None,
                expected_size: Some(package.asset_index.size),
                display_name: Some(format!("Asset Index ({}.json)", package.asset_index.id)),
            };
            self.downloader.download_file(&task).await?;
        }

        let mut download_tasks = Vec::new();
        let mut total_assets_checked = 0;

        if let Ok(content) = fs::read_to_string(&index_file) {
            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                if let Some(objects) = val.get("objects").and_then(|o| o.as_object()) {
                    for (name, obj) in objects {
                        if let (Some(hash), Some(size)) = (
                            obj.get("hash").and_then(|h| h.as_str()),
                            obj.get("size").and_then(|s| s.as_u64()),
                        ) {
                            if hash.len() >= 2 {
                                total_assets_checked += 1;
                                let prefix = &hash[..2];
                                let obj_dest = config
                                    .shared_assets_dir()
                                    .join("objects")
                                    .join(prefix)
                                    .join(hash);

                                let is_valid = obj_dest.exists()
                                    && obj_dest
                                        .metadata()
                                        .map(|m| m.len() == size)
                                        .unwrap_or(false)
                                    && crate::security::hashes::verify_file_sha1(&obj_dest, hash);

                                if !is_valid {
                                    let obj_url =
                                        format!("{}/{}/{}", MOJANG_ASSET_BASE, prefix, hash);
                                    download_tasks.push(DownloadTask {
                                        url: obj_url,
                                        destination: obj_dest,
                                        sha1: Some(hash.to_string()),
                                        sha256: None,
                                        expected_size: Some(size),
                                        display_name: Some(name.clone()),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        let repaired_count = download_tasks.len();
        if !download_tasks.is_empty() {
            set_status(&format!(
                "Repairing {} missing or corrupted assets...",
                repaired_count
            ));
            self.downloader
                .download_all(download_tasks, progress_cb)
                .await?;
            set_status(&format!(
                "Asset repair complete: Checked {} assets, restored {} files.",
                total_assets_checked, repaired_count
            ));
        } else {
            set_status(&format!(
                "Asset check complete: All {} game assets intact and verified!",
                total_assets_checked
            ));
        }

        Ok((total_assets_checked, repaired_count))
    }
}
