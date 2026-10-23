//! Mod-Loader Metadata and Version Resolution Adapters.
//!
//! Queries official upstream distribution APIs for Fabric, Quilt, and NeoForge.

pub mod modrinth;
pub use modrinth::{ModrinthClient, ModrinthMod};

pub mod curseforge;
pub use curseforge::{CurseForgeClient, CurseForgeMod};

use crate::branding;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tracing::warn;

pub const FABRIC_META_BASE: &str = "https://meta.fabricmc.net/v2";
pub const QUILT_META_BASE: &str = "https://meta.quiltmc.org/v3";
pub const NEOFORGE_MAVEN_META: &str =
    "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Network error fetching loader metadata: {0}")]
    Network(#[from] reqwest::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O error preparing loader libraries: {0}")]
    Io(#[from] std::io::Error),
    #[error("Loader error: {0}")]
    Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoaderVersionInfo {
    pub loader_name: String,
    pub version: String,
    pub game_version: String,
    pub stable: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModLoaderLaunchInfo {
    pub main_class: Option<String>,
    pub extra_libraries: Vec<std::path::PathBuf>,
    pub extra_jvm_args: Vec<String>,
    pub extra_game_args: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FabricLibrary {
    pub name: String,
    pub url: Option<String>,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub sha512: Option<String>,
    pub size: Option<u64>,
}

impl FabricLibrary {
    pub fn maven_relative_path(&self) -> Option<String> {
        let parts: Vec<&str> = self.name.split(':').collect();
        if parts.len() >= 3 {
            let group = parts[0].replace('.', "/");
            let artifact = parts[1];
            let version = parts[2];
            let jar_name = format!("{}-{}.jar", artifact, version);
            Some(format!("{}/{}/{}/{}", group, artifact, version, jar_name))
        } else {
            None
        }
    }

    pub fn download_url(&self) -> Option<String> {
        let rel = self.maven_relative_path()?;
        let base = self.url.as_deref().unwrap_or("https://maven.fabricmc.net/");
        let base_clean = if base.ends_with('/') {
            base.to_string()
        } else {
            format!("{}/", base)
        };
        Some(format!("{}{}", base_clean, rel))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FabricProfileArguments {
    pub jvm: Option<Vec<String>>,
    pub game: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FabricProfile {
    pub id: String,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    pub arguments: Option<FabricProfileArguments>,
    pub libraries: Vec<FabricLibrary>,
}

#[derive(Clone)]
pub struct ModLoaderClient {
    client: Client,
}

impl Default for ModLoaderClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ModLoaderClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(branding::USER_AGENT)
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to initialize reqwest client for ModLoaderClient");

        Self { client }
    }

    /// Query compatible Fabric loader versions for a specific Minecraft version.
    pub async fn get_fabric_loaders(
        &self,
        mc_version: &str,
    ) -> Result<Vec<LoaderVersionInfo>, LoaderError> {
        let url = format!("{}/versions/loader/{}", FABRIC_META_BASE, mc_version);
        let resp = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to query Fabric loaders for {}: {}", mc_version, e);
                return Ok(Vec::new());
            }
        };

        if resp.status().as_u16() == 404 {
            return Ok(Vec::new());
        }

        let data: Vec<serde_json::Value> = resp.json().await?;
        let mut results = Vec::new();

        for item in data {
            if let Some(loader) = item.get("loader") {
                if let Some(ver) = loader.get("version").and_then(|v| v.as_str()) {
                    let stable = loader
                        .get("stable")
                        .and_then(|s| s.as_bool())
                        .unwrap_or(true);
                    results.push(LoaderVersionInfo {
                        loader_name: "fabric".to_string(),
                        version: ver.to_string(),
                        game_version: mc_version.to_string(),
                        stable,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Query compatible Quilt loader versions for a specific Minecraft version.
    pub async fn get_quilt_loaders(
        &self,
        mc_version: &str,
    ) -> Result<Vec<LoaderVersionInfo>, LoaderError> {
        let url = format!("{}/versions/loader/{}", QUILT_META_BASE, mc_version);
        let resp = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to query Quilt loaders for {}: {}", mc_version, e);
                return Ok(Vec::new());
            }
        };

        if resp.status().as_u16() == 404 {
            return Ok(Vec::new());
        }

        let data: Vec<serde_json::Value> = resp.json().await?;
        let mut results = Vec::new();

        for item in data {
            if let Some(loader) = item.get("loader") {
                if let Some(ver) = loader.get("version").and_then(|v| v.as_str()) {
                    let stable = loader
                        .get("stable")
                        .and_then(|s| s.as_bool())
                        .unwrap_or(true);
                    results.push(LoaderVersionInfo {
                        loader_name: "quilt".to_string(),
                        version: ver.to_string(),
                        game_version: mc_version.to_string(),
                        stable,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Query released NeoForge versions.
    pub async fn get_neoforge_versions(&self) -> Result<Vec<String>, LoaderError> {
        let resp = match self.client.get(NEOFORGE_MAVEN_META).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to query NeoForge releases: {}", e);
                return Ok(Vec::new());
            }
        };

        let data: serde_json::Value = resp.json().await?;
        let mut versions = Vec::new();

        if let Some(arr) = data.get("versions").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(v_str) = item.as_str() {
                    versions.push(v_str.to_string());
                }
            }
        }

        Ok(versions)
    }

    /// Retrieve Quilt profile JSON containing mainClass and libraries for launching.
    pub async fn get_quilt_profile(
        &self,
        mc_version: &str,
        loader_version: Option<&str>,
    ) -> Result<FabricProfile, LoaderError> {
        let version_to_use = match loader_version {
            Some(v) => v.to_string(),
            None => {
                let loaders = self.get_quilt_loaders(mc_version).await?;
                let selected = loaders
                    .iter()
                    .find(|l| l.stable)
                    .or_else(|| loaders.first())
                    .map(|l| l.version.clone())
                    .unwrap_or_else(|| "0.27.1".to_string());
                selected
            }
        };

        let url = format!(
            "{}/versions/loader/{}/{}/profile/json",
            QUILT_META_BASE, mc_version, version_to_use
        );
        let resp = self.client.get(&url).send().await?.error_for_status()?;
        let profile: FabricProfile = resp.json().await?;
        Ok(profile)
    }

    /// Retrieve Fabric profile JSON containing mainClass and libraries for launching.
    pub async fn get_fabric_profile(
        &self,
        mc_version: &str,
        loader_version: Option<&str>,
    ) -> Result<FabricProfile, LoaderError> {
        let version_to_use = match loader_version {
            Some(v) => v.to_string(),
            None => {
                let loaders = self.get_fabric_loaders(mc_version).await?;
                let selected = loaders
                    .iter()
                    .find(|l| l.stable)
                    .or_else(|| loaders.first())
                    .map(|l| l.version.clone())
                    .unwrap_or_else(|| "0.16.10".to_string());
                selected
            }
        };

        let url = format!(
            "{}/versions/loader/{}/{}/profile/json",
            FABRIC_META_BASE, mc_version, version_to_use
        );
        let resp = self.client.get(&url).send().await?.error_for_status()?;
        let profile: FabricProfile = resp.json().await?;
        Ok(profile)
    }

    /// Resolve and download mod loader dependencies into libraries_dir, returning launch arguments.
    pub async fn resolve_and_download_loader(
        &self,
        loader_name: &str,
        mc_version: &str,
        loader_version: Option<&str>,
        libraries_dir: &std::path::Path,
    ) -> Result<ModLoaderLaunchInfo, LoaderError> {
        let name_lower = loader_name.trim().to_lowercase();
        let mut info = ModLoaderLaunchInfo::default();

        match name_lower.as_str() {
            "fabric" => {
                let profile = self.get_fabric_profile(mc_version, loader_version).await?;
                info.main_class = Some(profile.main_class.clone());

                if let Some(ref args) = profile.arguments {
                    if let Some(ref jvm) = args.jvm {
                        info.extra_jvm_args.extend(jvm.clone());
                    }
                    if let Some(ref game) = args.game {
                        info.extra_game_args.extend(game.clone());
                    }
                }
                if info.extra_jvm_args.is_empty() {
                    info.extra_jvm_args
                        .push("-DFabricMcEmu= net.minecraft.client.main.Main ".to_string());
                }

                // Download libraries into shared libraries dir
                for lib in &profile.libraries {
                    if let (Some(rel_path), Some(dl_url)) =
                        (lib.maven_relative_path(), lib.download_url())
                    {
                        let target_path = libraries_dir.join(&rel_path);
                        if !target_path.exists() {
                            if let Some(parent) = target_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            if let Ok(resp) = self.client.get(&dl_url).send().await {
                                if resp.status().is_success() {
                                    if let Ok(bytes) = resp.bytes().await {
                                        let _ = std::fs::write(&target_path, bytes);
                                    }
                                }
                            }
                        }
                        if target_path.exists() {
                            info.extra_libraries.push(target_path);
                        }
                    }
                }
            }
            "quilt" => {
                let profile = self.get_quilt_profile(mc_version, loader_version).await?;
                info.main_class = Some(profile.main_class.clone());

                if let Some(ref args) = profile.arguments {
                    if let Some(ref jvm) = args.jvm {
                        info.extra_jvm_args.extend(jvm.clone());
                    }
                    if let Some(ref game) = args.game {
                        info.extra_game_args.extend(game.clone());
                    }
                }

                // Download Quilt libraries into shared libraries dir
                for lib in &profile.libraries {
                    if let Some(rel_path) = lib.maven_relative_path() {
                        let dl_url = lib.download_url().unwrap_or_else(|| {
                            format!("https://maven.quiltmc.org/repository/release/{}", rel_path)
                        });
                        let target_path = libraries_dir.join(&rel_path);
                        if !target_path.exists() {
                            if let Some(parent) = target_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            if let Ok(resp) = self.client.get(&dl_url).send().await {
                                if resp.status().is_success() {
                                    if let Ok(bytes) = resp.bytes().await {
                                        let _ = std::fs::write(&target_path, bytes);
                                    }
                                }
                            }
                        }
                        if target_path.exists() {
                            info.extra_libraries.push(target_path);
                        }
                    }
                }
            }
            "optifine" => {
                // Search for OptiFine JAR in the libraries directory
                let mut found_optifine = false;
                if let Ok(entries) = std::fs::read_dir(libraries_dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.to_lowercase().contains("optifine") && name.ends_with(".jar") {
                            info.extra_libraries.push(entry.path());
                            found_optifine = true;
                            break;
                        }
                    }
                }

                if found_optifine {
                    info.extra_game_args.push("--tweakClass".to_string());
                    info.extra_game_args
                        .push("optifine.OptiFineTweaker".to_string());
                } else {
                    warn!("OptiFine was selected, but no OptiFine JAR was found in the libraries directory. To prevent ClassNotFoundException, launching standard game jar.");
                }
            }
            "forge" => {
                info.extra_jvm_args
                    .push("-Dforge.logging.console.level=info".to_string());
                warn!("Forge profile selected. Modern Forge requires running the Forge client installer for this instance.");
            }
            "neoforge" => {
                info.extra_jvm_args
                    .push("-Dneoforge.logging.console.level=info".to_string());
                warn!("NeoForge profile selected. NeoForge requires running the NeoForge client installer for this instance.");
            }
            _ => {
                // vanilla - standard launcher arguments
            }
        }

        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fabric_library_maven_resolution() {
        let lib = FabricLibrary {
            name: "net.fabricmc:fabric-loader:0.16.10".to_string(),
            url: Some("https://maven.fabricmc.net/".to_string()),
            md5: None,
            sha1: None,
            sha256: None,
            sha512: None,
            size: None,
        };

        assert_eq!(
            lib.maven_relative_path().unwrap(),
            "net/fabricmc/fabric-loader/0.16.10/fabric-loader-0.16.10.jar"
        );
        assert_eq!(
            lib.download_url().unwrap(),
            "https://maven.fabricmc.net/net/fabricmc/fabric-loader/0.16.10/fabric-loader-0.16.10.jar"
        );
    }

    #[test]
    fn test_parse_fabric_profile_json() {
        let sample = r#"{
            "id": "fabric-loader-0.16.10-1.21.4",
            "inheritsFrom": "1.21.4",
            "mainClass": "net.fabricmc.loader.impl.launch.knot.KnotClient",
            "arguments": {
                "jvm": ["-DFabricMcEmu= net.minecraft.client.main.Main "]
            },
            "libraries": [
                {
                    "name": "net.fabricmc:fabric-loader:0.16.10",
                    "url": "https://maven.fabricmc.net/"
                }
            ]
        }"#;

        let profile: FabricProfile = serde_json::from_str(sample).unwrap();
        assert_eq!(
            profile.main_class,
            "net.fabricmc.loader.impl.launch.knot.KnotClient"
        );
        assert_eq!(profile.libraries.len(), 1);
        assert_eq!(
            profile.libraries[0].name,
            "net.fabricmc:fabric-loader:0.16.10"
        );
    }
}
