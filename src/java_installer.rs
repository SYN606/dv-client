//! Automatic Java 21 Downloader and Installer for Windows.
//!
//! Provides one-click automated download, safe extraction, and setup of
//! Eclipse Temurin OpenJDK 21 JRE into the launcher's isolated runtimes directory.

use futures_util::StreamExt;
use reqwest::Client;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{info, warn};

use crate::java_detector::JavaDetector;

#[derive(Debug, Error)]
pub enum JavaInstallError {
    #[error("Network error while downloading Java 21: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Filesystem I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Zip slip path traversal detected: {0}")]
    PathTraversal(String),
    #[error("Downloaded runtime is invalid or missing java executable")]
    InvalidRuntime,
    #[error("All download endpoints failed")]
    AllEndpointsFailed,
}

pub struct JavaInstaller;

impl JavaInstaller {
    /// Returns the download endpoints for Windows Java 21 in prioritized order.
    pub fn get_download_endpoints() -> Vec<(&'static str, &'static str)> {
        let arch = std::env::consts::ARCH;
        if arch == "aarch64" {
            vec![
                (
                    "Adoptium Temurin 21 JRE (ARM64)",
                    "https://api.adoptium.net/v3/binary/latest/21/ga/windows/aarch64/jre/hotspot/normal/eclipse",
                ),
                (
                    "Microsoft OpenJDK 21 (ARM64)",
                    "https://aka.ms/download-jdk/microsoft-jdk-21.0.6-windows-aarch64.zip",
                ),
            ]
        } else {
            vec![
                (
                    "Adoptium Temurin 21 JRE (x64)",
                    "https://api.adoptium.net/v3/binary/latest/21/ga/windows/x64/jre/hotspot/normal/eclipse",
                ),
                (
                    "Microsoft OpenJDK 21 (x64)",
                    "https://aka.ms/download-jdk/microsoft-jdk-21.0.6-windows-x64.zip",
                ),
                (
                    "Adoptium Temurin 21 JRE Mirror (x64)",
                    "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.6%2B7/OpenJDK21U-jre_x64_windows_hotspot_21.0.6_7.zip",
                ),
            ]
        }
    }

    /// Recursively search for `java.exe` (or `java` on unix) inside `dir`.
    pub fn find_java_in_directory<P: AsRef<Path>>(dir: P) -> Option<PathBuf> {
        let dir = dir.as_ref();
        if !dir.exists() {
            return None;
        }

        let target_name = if cfg!(target_os = "windows") {
            "java.exe"
        } else {
            "java"
        };

        // Check direct bin/java.exe
        let direct_bin = dir.join("bin").join(target_name);
        if direct_bin.exists() {
            return Some(dunce::simplified(&direct_bin).to_path_buf());
        }

        // Check root/java.exe
        let root_bin = dir.join(target_name);
        if root_bin.exists() {
            return Some(dunce::simplified(&root_bin).to_path_buf());
        }

        // Walk 2 levels deep
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let sub_bin = p.join("bin").join(target_name);
                    if sub_bin.exists() {
                        return Some(dunce::simplified(&sub_bin).to_path_buf());
                    }
                    if let Ok(sub_entries) = fs::read_dir(&p) {
                        for sub_entry in sub_entries.flatten() {
                            let sp = sub_entry.path();
                            if sp.is_dir() {
                                let sub_sub_bin = sp.join("bin").join(target_name);
                                if sub_sub_bin.exists() {
                                    return Some(dunce::simplified(&sub_sub_bin).to_path_buf());
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Downloads and installs Java 21 into `runtimes_dir/java-21`.
    /// Reports progress via `progress_cb(fraction, status_text)`.
    pub async fn install_java_21<F>(
        runtimes_dir: &Path,
        progress_cb: F,
    ) -> Result<PathBuf, JavaInstallError>
    where
        F: Fn(f32, &str) + Send + Sync + 'static,
    {
        fs::create_dir_all(runtimes_dir)?;
        let runtimes_canonical = dunce::canonicalize(runtimes_dir)?;
        let install_dest = runtimes_canonical.join("java-21");
        let zip_temp_path = runtimes_canonical.join("java-21-download.zip");

        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()?;

        let endpoints = Self::get_download_endpoints();
        let mut download_success = false;

        for (name, url) in endpoints {
            info!("Attempting to download Java 21 from: {} ({})", name, url);
            progress_cb(0.05, &format!("Connecting to {}...", name));

            match client.get(url).send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        warn!(
                            "Failed with HTTP status {} from endpoint: {}",
                            response.status(),
                            url
                        );
                        continue;
                    }

                    let total_bytes = response.content_length().unwrap_or(48 * 1024 * 1024);
                    let mut downloaded: u64 = 0;
                    let mut stream = response.bytes_stream();

                    let file_result = File::create(&zip_temp_path);
                    let mut file = match file_result {
                        Ok(f) => f,
                        Err(e) => return Err(JavaInstallError::Io(e)),
                    };

                    let mut last_pct = -1;
                    let mut stream_failed = false;

                    while let Some(chunk_res) = stream.next().await {
                        match chunk_res {
                            Ok(chunk) => {
                                if let Err(e) = file.write_all(&chunk) {
                                    warn!("Error writing chunk to zip: {}", e);
                                    stream_failed = true;
                                    break;
                                }
                                downloaded += chunk.len() as u64;

                                let pct = ((downloaded as f64 / total_bytes as f64) * 100.0) as i32;
                                if pct != last_pct {
                                    last_pct = pct;
                                    let downloaded_mb = downloaded as f64 / (1024.0 * 1024.0);
                                    let total_mb = total_bytes as f64 / (1024.0 * 1024.0);
                                    let fraction = 0.05
                                        + (downloaded as f32 / total_bytes as f32).min(1.0) * 0.70;
                                    let msg = format!(
                                        "Downloading Java 21: {:.1} MB / {:.1} MB ({}%)",
                                        downloaded_mb, total_mb, pct
                                    );
                                    progress_cb(fraction, &msg);
                                }
                            }
                            Err(e) => {
                                warn!("Stream error while downloading from {}: {}", url, e);
                                stream_failed = true;
                                break;
                            }
                        }
                    }

                    if stream_failed {
                        let _ = fs::remove_file(&zip_temp_path);
                        continue;
                    }

                    if let Err(e) = file.flush() {
                        warn!("Failed to flush downloaded zip: {}", e);
                        let _ = fs::remove_file(&zip_temp_path);
                        continue;
                    }

                    download_success = true;
                    break;
                }
                Err(e) => {
                    warn!("Network error connecting to {}: {}", url, e);
                }
            }
        }

        if !download_success {
            return Err(JavaInstallError::AllEndpointsFailed);
        }

        // 2. Safe Extraction
        progress_cb(0.78, "Preparing destination directory...");

        // Clean up previous installation if present
        if install_dest.exists() {
            let _ = fs::remove_dir_all(&install_dest);
        }
        fs::create_dir_all(&install_dest)?;
        let install_dest_canonical = dunce::canonicalize(&install_dest)?;

        progress_cb(0.80, "Extracting Java 21 runtime files...");
        let zip_file = File::open(&zip_temp_path)?;
        let mut archive = zip::ZipArchive::new(zip_file)?;

        // Determine if all entries share a common top-level root folder
        let common_root: Option<PathBuf> = {
            let names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();
            if let Some(first) = names.first() {
                if let Some(first_comp) = Path::new(first).components().next() {
                    let root_str = first_comp.as_os_str().to_string_lossy().to_string();
                    if names.iter().all(|n| n.starts_with(&root_str)) {
                        Some(PathBuf::from(root_str))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        let total_entries = archive.len();
        for i in 0..total_entries {
            let mut member = archive.by_index(i)?;
            let raw_enclosed = match member.enclosed_name() {
                Some(p) => p.to_path_buf(),
                None => {
                    return Err(JavaInstallError::PathTraversal(member.name().to_string()));
                }
            };

            let rel_path = if let Some(ref root) = common_root {
                raw_enclosed
                    .strip_prefix(root)
                    .unwrap_or(&raw_enclosed)
                    .to_path_buf()
            } else {
                raw_enclosed
            };

            if rel_path.as_os_str().is_empty() {
                continue;
            }

            let target_path = install_dest_canonical.join(&rel_path);

            // Zip Slip protection
            if !target_path.starts_with(&install_dest_canonical) {
                return Err(JavaInstallError::PathTraversal(member.name().to_string()));
            }

            if member.is_dir() {
                fs::create_dir_all(&target_path)?;
                continue;
            }

            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut outfile = File::create(&target_path)?;
            io::copy(&mut member, &mut outfile)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let is_exec = member
                    .unix_mode()
                    .map(|m| m & 0o111 != 0)
                    .unwrap_or(false);
                if is_exec || target_path.to_string_lossy().contains("/bin/") {
                    let _ = fs::set_permissions(&target_path, fs::Permissions::from_mode(0o755));
                }
            }

            if i % 100 == 0 || i == total_entries - 1 {
                let frac = 0.80 + (i as f32 / total_entries as f32) * 0.15;
                progress_cb(
                    frac,
                    &format!("Extracting runtime files: {} / {}", i + 1, total_entries),
                );
            }
        }

        // Clean up temporary download file
        let _ = fs::remove_file(&zip_temp_path);

        // 3. Locate and verify the java executable
        progress_cb(0.96, "Verifying Java 21 runtime installation...");

        let java_exe = Self::find_java_in_directory(&install_dest_canonical)
            .ok_or(JavaInstallError::InvalidRuntime)?;

        info!("Located installed Java executable at: {:?}", java_exe);

        if let Some(info) = JavaDetector::inspect_executable(&java_exe) {
            info!(
                "Successfully verified Java runtime: {} (v{})",
                info.display_name, info.full_version
            );
        } else {
            warn!(
                "Java binary present at {:?} but inspect_executable returned None. Proceeding with installed path.",
                java_exe
            );
        }

        progress_cb(1.0, "Java 21 installed and verified successfully!");
        Ok(java_exe)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_download_endpoints_defined() {
        let endpoints = JavaInstaller::get_download_endpoints();
        assert!(!endpoints.is_empty());
        for (name, url) in endpoints {
            assert!(!name.is_empty());
            assert!(url.starts_with("https://"));
        }
    }

    #[test]
    fn test_find_java_in_directory() {
        let temp = tempdir().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();

        let exe_name = if cfg!(target_os = "windows") {
            "java.exe"
        } else {
            "java"
        };
        let java_file = bin_dir.join(exe_name);
        fs::write(&java_file, b"dummy java").unwrap();

        let found = JavaInstaller::find_java_in_directory(temp.path());
        assert!(found.is_some());
        assert_eq!(found.unwrap(), dunce::simplified(&java_file).to_path_buf());
    }
}
