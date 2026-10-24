use crate::state::LauncherInfo;
use dv_launcher::branding;
use serde::{Deserialize, Serialize};
use tracing::info;

pub const GITHUB_RELEASES_API: &str =
    "https://api.github.com/repos/SYN606/dv-client/releases/latest";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateCheckResult {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_name: String,
    pub release_notes: String,
    pub published_at: Option<String>,
    pub html_url: String,
    pub download_url: Option<String>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    html_url: String,
    published_at: Option<String>,
    #[serde(default)]
    assets: Vec<GitHubAsset>,
}

fn parse_version_tuple(v: &str) -> (u64, u64, u64) {
    let clean = v.trim().trim_start_matches('v').trim_start_matches('V');
    let mut parts = clean.split('.');
    let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts
        .next()
        .and_then(|s| s.split('-').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (major, minor, patch)
}

fn is_version_newer(latest: &str, current: &str) -> bool {
    let (l_maj, l_min, l_pat) = parse_version_tuple(latest);
    let (c_maj, c_min, c_pat) = parse_version_tuple(current);
    (l_maj, l_min, l_pat) > (c_maj, c_min, c_pat)
}

#[tauri::command]
pub fn get_launcher_info() -> LauncherInfo {
    LauncherInfo {
        name: branding::APP_NAME.to_string(),
        version: branding::APP_VERSION.to_string(),
        author: "Digital Vigital".to_string(),
        repo_url: "https://github.com/SYN606/dv-client".to_string(),
    }
}

#[tauri::command]
pub async fn open_folder(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateCheckResult, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let resp = client
        .get(GITHUB_RELEASES_API)
        .header(
            "User-Agent",
            format!("DigitalVigital-Launcher/{}", branding::APP_VERSION),
        )
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("GitHub API returned HTTP {}: {}", status, body));
    }

    let release: GitHubRelease = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub release payload: {}", e))?;

    let current_ver = branding::APP_VERSION;
    let latest_tag = release.tag_name.clone();
    let has_update = is_version_newer(&latest_tag, current_ver);

    // Look for Windows installer asset or preferred download asset
    let download_url = release
        .assets
        .iter()
        .find(|a| a.name.ends_with(".exe") || a.name.ends_with(".zip"))
        .map(|a| a.browser_download_url.clone())
        .or_else(|| Some(release.html_url.clone()));

    let release_name = release
        .name
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| format!("Digital Vigital {}", latest_tag));

    let release_notes = release.body.unwrap_or_default();

    info!(
        "Update check complete: current=v{}, latest={}, has_update={}",
        current_ver, latest_tag, has_update
    );

    Ok(UpdateCheckResult {
        has_update,
        current_version: current_ver.to_string(),
        latest_version: latest_tag,
        release_name,
        release_notes,
        published_at: release.published_at,
        html_url: release.html_url,
        download_url,
    })
}
