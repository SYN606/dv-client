//! CurseForge Online Mod Discovery and 1-Click Installation Client.
//!
//! Provides search and direct downloading from CurseForge's catalog of 100,000+ Minecraft mods
//! using the official CurseForge Core API.

use crate::branding;
use crate::loaders::LoaderError;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tracing::{info, warn};

pub const CURSEFORGE_API_BASE: &str = "https://api.curseforge.com/v1";
pub const CURSEFORGE_MINECRAFT_GAME_ID: u32 = 432;
pub const CURSEFORGE_MODS_CLASS_ID: u32 = 6;
pub const CURSEFORGE_API_KEY: &str = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurseForgeMod {
    pub id: u64,
    pub name: String,
    pub slug: String,
    pub summary: String,
    pub download_count: u64,
    pub icon_url: Option<String>,
    pub author: String,
    pub categories: Vec<String>,
    pub website_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurseForgeFile {
    pub id: u64,
    pub mod_id: u64,
    pub display_name: String,
    pub file_name: String,
    pub download_url: Option<String>,
    pub file_length: u64,
    pub game_versions: Vec<String>,
}

#[derive(Clone)]
pub struct CurseForgeClient {
    client: Client,
}

impl Default for CurseForgeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl CurseForgeClient {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        if let Ok(key) = HeaderValue::from_str(CURSEFORGE_API_KEY) {
            headers.insert("x-api-key", key);
        }

        let client = Client::builder()
            .user_agent(branding::USER_AGENT)
            .default_headers(headers)
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to initialize reqwest client for CurseForgeClient");

        Self { client }
    }

    /// Convert loader name string to CurseForge modLoaderType enum integer.
    /// 0 = Any, 1 = Forge, 4 = Fabric, 5 = Quilt, 6 = NeoForge
    fn map_loader_type(loader: Option<&str>) -> u32 {
        match loader.map(|s| s.to_lowercase()).as_deref() {
            Some("fabric") => 4,
            Some("forge") => 1,
            Some("neoforge") => 6,
            Some("quilt") => 5,
            _ => 0,
        }
    }

    /// Search mods on CurseForge with filtering and sorting.
    pub async fn search_mods(
        &self,
        query: &str,
        mc_version: Option<&str>,
        loader: Option<&str>,
        category_id: Option<u32>,
        sort_by: Option<&str>,
        limit: u32,
    ) -> Result<Vec<CurseForgeMod>, LoaderError> {
        let trimmed_query = query.trim();
        let loader_type = Self::map_loader_type(loader);

        // SortField: 1=Featured, 2=Popularity, 3=LastUpdated, 4=Name, 6=TotalDownloads
        let sort_field = match sort_by.unwrap_or("downloads") {
            "updated" => 3,
            "newest" | "recent" => 3,
            "name" => 4,
            "popularity" => 2,
            _ => 6, // TotalDownloads
        };

        let mut url = format!(
            "{}/mods/search?gameId={}&classId={}&sortField={}&sortOrder=desc&pageSize={}",
            CURSEFORGE_API_BASE,
            CURSEFORGE_MINECRAFT_GAME_ID,
            CURSEFORGE_MODS_CLASS_ID,
            sort_field,
            limit.clamp(1, 50)
        );

        if !trimmed_query.is_empty() {
            url.push_str(&format!("&searchFilter={}", urlencoding_encode(trimmed_query)));
        }

        if let Some(ver) = mc_version {
            if !ver.is_empty() {
                url.push_str(&format!("&gameVersion={}", urlencoding_encode(ver)));
            }
        }

        if loader_type > 0 {
            url.push_str(&format!("&modLoaderType={}", loader_type));
        }

        if let Some(cat) = category_id {
            if cat > 0 {
                url.push_str(&format!("&categoryId={}", cat));
            }
        }

        info!("Querying CurseForge search API: {}", url);

        let resp = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to query CurseForge search: {}", e);
                return Ok(Vec::new());
            }
        };

        if !resp.status().is_success() {
            warn!("CurseForge search returned HTTP status: {}", resp.status());
            return Ok(Vec::new());
        }

        let body: serde_json::Value = resp.json().await?;
        let mut results = Vec::new();

        if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
            for item in data {
                let id = item.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                let name = item
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let slug = item
                    .get("slug")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let summary = item
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let download_count = item
                    .get("downloadCount")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let icon_url = item
                    .get("logo")
                    .and_then(|l| l.get("thumbnailUrl").or_else(|| l.get("url")))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let author = item
                    .get("authors")
                    .and_then(|a| a.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|a| a.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let categories = item
                    .get("categories")
                    .and_then(|c| c.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|c| c.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                let website_url = item
                    .get("links")
                    .and_then(|l| l.get("websiteUrl"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if id > 0 && !name.is_empty() {
                    results.push(CurseForgeMod {
                        id,
                        name,
                        slug,
                        summary,
                        download_count,
                        icon_url,
                        author,
                        categories,
                        website_url,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Download and install a CurseForge mod directly into the instance's mods folder.
    pub async fn install_mod<P: AsRef<Path>>(
        &self,
        mod_id: u64,
        mc_version: &str,
        loader: &str,
        mods_dir: P,
    ) -> Result<String, LoaderError> {
        let dir = mods_dir.as_ref();
        fs::create_dir_all(dir)?;

        let loader_type = Self::map_loader_type(Some(loader));
        let mut url = format!("{}/mods/{}/files?pageSize=20", CURSEFORGE_API_BASE, mod_id);

        if !mc_version.is_empty() {
            url.push_str(&format!("&gameVersion={}", urlencoding_encode(mc_version)));
        }
        if loader_type > 0 {
            url.push_str(&format!("&modLoaderType={}", loader_type));
        }

        info!("Fetching CurseForge mod files: {}", url);
        let resp = self.client.get(&url).send().await?.error_for_status()?;
        let body: serde_json::Value = resp.json().await?;

        let files = body.get("data").and_then(|d| d.as_array()).ok_or_else(|| {
            LoaderError::Custom(format!("No files available for CurseForge mod ID {}", mod_id))
        })?;

        if files.is_empty() {
            return Err(LoaderError::Custom(format!(
                "No compatible version found on CurseForge for mod {} on MC {} ({})",
                mod_id, mc_version, loader
            )));
        }

        // Pick latest primary / release file
        let file_obj = files
            .iter()
            .find(|f| f.get("releaseType").and_then(|v| v.as_u64()) == Some(1))
            .or_else(|| files.first())
            .ok_or_else(|| LoaderError::Custom("No downloadable files found".to_string()))?;

        let file_id = file_obj
            .get("id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| LoaderError::Custom("File object missing ID".to_string()))?;

        let file_name = file_obj
            .get("fileName")
            .and_then(|v| v.as_str())
            .unwrap_or("mod.jar")
            .to_string();

        // Check if downloadUrl is provided or needs separate fetch / edge fallback
        let download_url = if let Some(u) = file_obj.get("downloadUrl").and_then(|v| v.as_str()) {
            u.to_string()
        } else {
            // Call download-url endpoint
            let dl_url_api = format!("{}/mods/{}/files/{}/download-url", CURSEFORGE_API_BASE, mod_id, file_id);
            if let Ok(dl_resp) = self.client.get(&dl_url_api).send().await {
                if let Ok(val) = dl_resp.json::<serde_json::Value>().await {
                    if let Some(u) = val.get("data").and_then(|v| v.as_str()) {
                        u.to_string()
                    } else {
                        // Edge CDN formula fallback: https://edge.forgecdn.net/files/{part1}/{part2}/{filename}
                        let part1 = file_id / 1000;
                        let part2 = file_id % 1000;
                        format!("https://edge.forgecdn.net/files/{}/{}/{}", part1, part2, file_name)
                    }
                } else {
                    let part1 = file_id / 1000;
                    let part2 = file_id % 1000;
                    format!("https://edge.forgecdn.net/files/{}/{}/{}", part1, part2, file_name)
                }
            } else {
                let part1 = file_id / 1000;
                let part2 = file_id % 1000;
                format!("https://edge.forgecdn.net/files/{}/{}/{}", part1, part2, file_name)
            }
        };

        let dest = dir.join(&file_name);
        info!("Downloading CurseForge mod '{}' from {} into {:?}", file_name, download_url, dest);

        // Remove old duplicates of same mod before writing to prevent Fabric/Forge duplicate ID crashes
        if let Ok(entries) = fs::read_dir(dir) {
            let stem = file_name.split(['-', '_']).next().unwrap_or("").to_lowercase();
            if !stem.is_empty() && stem.len() >= 3 {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name != file_name && name.to_lowercase().starts_with(&stem) && (name.ends_with(".jar") || name.ends_with(".disabled")) {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }

        let dl_resp = self.client.get(&download_url).send().await?.error_for_status()?;
        let bytes = dl_resp.bytes().await?;
        fs::write(&dest, bytes)?;

        Ok(file_name)
    }
}

fn urlencoding_encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
