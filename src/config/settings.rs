//! Configuration and Directory Management Subsystem.
//!
//! Provides schema-validated, persistent JSON configuration for Digital Vigital (DV) Launcher.
//! Resolves platform-specific directories for Linux XDG and Windows APPDATA.

use crate::branding;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error during configuration management: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse JSON configuration: {0}")]
    Json(#[from] serde_json::Error),
}

/// Resolves the platform-native base data directory for Digital Vigital (DV) Launcher.
pub fn get_default_app_data_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("DigitalVigital");
        }
        if let Some(home) = dirs::home_dir() {
            return home.join("AppData").join("Roaming").join("DigitalVigital");
        }
        PathBuf::from(".digitalvigital")
    } else if cfg!(target_os = "macos") {
        if let Some(home) = dirs::home_dir() {
            return home
                .join("Library")
                .join("Application Support")
                .join("DigitalVigital");
        }
        PathBuf::from(".digitalvigital")
    } else {
        // Linux / XDG
        if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
            return PathBuf::from(xdg_data).join("digitalvigital");
        }
        if let Some(home) = dirs::home_dir() {
            return home.join(".local").join("share").join("digitalvigital");
        }
        PathBuf::from(".digitalvigital")
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PrivacySettings {
    #[serde(default)]
    pub enable_telemetry: bool,
    #[serde(default)]
    pub enable_crash_reporting: bool,
    #[serde(default)]
    pub enable_discord_rpc: bool,
    #[serde(default)]
    pub discord_show_version: bool,
    #[serde(default)]
    pub discord_show_modpack: bool,
    #[serde(default)]
    pub discord_show_elapsed_time: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadSettings {
    #[serde(default = "default_concurrent_downloads")]
    pub max_concurrent_downloads: usize,
    #[serde(default = "default_timeout_seconds")]
    pub request_timeout_seconds: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_true")]
    pub use_http2: bool,
}

fn default_concurrent_downloads() -> usize {
    8
}
fn default_timeout_seconds() -> u64 {
    25
}
fn default_max_retries() -> u32 {
    3
}
fn default_true() -> bool {
    true
}

impl Default for DownloadSettings {
    fn default() -> Self {
        Self {
            max_concurrent_downloads: default_concurrent_downloads(),
            request_timeout_seconds: default_timeout_seconds(),
            max_retries: default_max_retries(),
            use_http2: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppearanceSettings {
    #[serde(default = "default_theme")]
    pub theme_mode: String,
    #[serde(default = "default_accent")]
    pub accent_color: String,
    #[serde(default = "default_scale")]
    pub ui_scale: f32,
    #[serde(default)]
    pub reduced_motion: bool,
}

fn default_theme() -> String {
    "dark".to_string()
}
fn default_accent() -> String {
    branding::COLOR_ACCENT_CYAN.to_string()
}
fn default_scale() -> f32 {
    1.0
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme_mode: default_theme(),
            accent_color: default_accent(),
            ui_scale: default_scale(),
            reduced_motion: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LauncherConfig {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    pub custom_minecraft_dir: Option<PathBuf>,
    pub custom_instances_dir: Option<PathBuf>,
    pub default_java_path: Option<PathBuf>,
    pub selected_gpu: Option<String>,
    #[serde(default = "default_max_memory")]
    pub default_max_memory_mb: u32,
    #[serde(default = "default_min_memory")]
    pub default_min_memory_mb: u32,
    #[serde(default = "default_window_width")]
    pub default_window_width: u32,
    #[serde(default = "default_window_height")]
    pub default_window_height: u32,
    pub active_instance_id: Option<String>,
    pub active_account_id: Option<String>,

    #[serde(default = "default_performance_preset")]
    pub performance_preset: String,

    #[serde(default)]
    pub privacy: PrivacySettings,
    #[serde(default)]
    pub downloads: DownloadSettings,
    #[serde(default)]
    pub appearance: AppearanceSettings,
}

fn default_performance_preset() -> String {
    "aikar".to_string()
}

fn default_version() -> String {
    branding::APP_VERSION.to_string()
}
fn default_data_dir() -> PathBuf {
    get_default_app_data_dir()
}
fn default_max_memory() -> u32 {
    4096
}
fn default_min_memory() -> u32 {
    1024
}
fn default_window_width() -> u32 {
    1280
}
fn default_window_height() -> u32 {
    720
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            data_dir: default_data_dir(),
            custom_minecraft_dir: None,
            custom_instances_dir: None,
            default_java_path: None,
            selected_gpu: Some("auto".to_string()),
            default_max_memory_mb: default_max_memory(),
            default_min_memory_mb: default_min_memory(),
            default_window_width: default_window_width(),
            default_window_height: default_window_height(),
            active_instance_id: None,
            active_account_id: None,
            performance_preset: default_performance_preset(),
            privacy: PrivacySettings::default(),
            downloads: DownloadSettings::default(),
            appearance: AppearanceSettings::default(),
        }
    }
}

impl LauncherConfig {
    pub fn base_path(&self) -> &Path {
        &self.data_dir
    }

    pub fn instances_dir(&self) -> PathBuf {
        if let Some(ref custom) = self.custom_instances_dir {
            custom.clone()
        } else {
            self.data_dir.join("instances")
        }
    }

    pub fn shared_assets_dir(&self) -> PathBuf {
        self.data_dir.join("assets")
    }

    pub fn shared_libraries_dir(&self) -> PathBuf {
        self.data_dir.join("libraries")
    }

    pub fn shared_versions_dir(&self) -> PathBuf {
        self.data_dir.join("versions")
    }

    pub fn runtimes_dir(&self) -> PathBuf {
        self.data_dir.join("runtimes")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.data_dir.join("logs")
    }

    pub fn ensure_directories(&self) -> Result<(), std::io::Error> {
        for dir in [
            self.base_path().to_path_buf(),
            self.instances_dir(),
            self.shared_assets_dir(),
            self.shared_libraries_dir(),
            self.shared_versions_dir(),
            self.runtimes_dir(),
            self.logs_dir(),
        ] {
            fs::create_dir_all(dir)?;
        }
        Ok(())
    }
}

/// Open a native directory picker dialog using rfd.
pub fn pick_directory(title: &str, default_path: Option<&Path>) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new().set_title(title);
    if let Some(p) = default_path {
        dialog = dialog.set_directory(p);
    }
    dialog.pick_folder()
}

#[derive(Clone, Debug)]
pub struct ConfigManager {
    config_file: PathBuf,
    pub config: LauncherConfig,
}

impl ConfigManager {
    pub fn new<P: AsRef<Path>>(config_file: Option<P>) -> Result<Self, ConfigError> {
        let file_path = match config_file {
            Some(p) => p.as_ref().to_path_buf(),
            None => get_default_app_data_dir().join("config.json"),
        };

        let config = if file_path.exists() {
            match fs::read_to_string(&file_path) {
                Ok(content) => match serde_json::from_str::<LauncherConfig>(&content) {
                    Ok(cfg) => {
                        let _ = cfg.ensure_directories();
                        cfg
                    }
                    Err(e) => {
                        warn!(
                            "Error parsing config at {:?}: {}, resetting to defaults",
                            file_path, e
                        );
                        let cfg = LauncherConfig::default();
                        let _ = cfg.ensure_directories();
                        cfg
                    }
                },
                Err(e) => {
                    warn!("Failed reading config {:?}: {}", file_path, e);
                    let cfg = LauncherConfig::default();
                    let _ = cfg.ensure_directories();
                    cfg
                }
            }
        } else {
            let cfg = LauncherConfig::default();
            let _ = cfg.ensure_directories();
            cfg
        };

        let mut mgr = Self {
            config_file: file_path,
            config,
        };
        mgr.save()?;
        Ok(mgr)
    }

    /// Save configuration atomically using a temporary file.
    pub fn save(&mut self) -> Result<(), ConfigError> {
        if let Some(parent) = self.config_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp_path = self.config_file.with_extension("tmp");
        let json_data = serde_json::to_string_pretty(&self.config)?;

        {
            let mut file = File::create(&tmp_path)?;
            file.write_all(json_data.as_bytes())?;
            file.flush()?;
        }

        fs::rename(&tmp_path, &self.config_file)?;
        info!("Saved configuration to {:?}", self.config_file);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config_creation() {
        let temp = tempdir().unwrap();
        let conf_file = temp.path().join("config.json");
        let mgr = ConfigManager::new(Some(&conf_file)).unwrap();

        assert!(!mgr.config.privacy.enable_telemetry);
        assert!(!mgr.config.privacy.enable_crash_reporting);
        assert!(!mgr.config.privacy.enable_discord_rpc);
        assert_eq!(mgr.config.downloads.max_concurrent_downloads, 8);
        assert_eq!(mgr.config.appearance.theme_mode, "dark");
        assert!(conf_file.exists());
    }

    #[test]
    fn test_custom_config_saving() {
        let temp = tempdir().unwrap();
        let conf_file = temp.path().join("custom_config.json");
        let mut mgr = ConfigManager::new(Some(&conf_file)).unwrap();

        mgr.config.default_max_memory_mb = 8192;
        mgr.config.privacy.enable_discord_rpc = true;
        mgr.save().unwrap();

        let mgr2 = ConfigManager::new(Some(&conf_file)).unwrap();
        assert_eq!(mgr2.config.default_max_memory_mb, 8192);
        assert!(mgr2.config.privacy.enable_discord_rpc);
    }
}
