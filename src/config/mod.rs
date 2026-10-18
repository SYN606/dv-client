//! Application configuration and persistent preferences module.

pub mod settings;

pub use settings::{
    get_default_app_data_dir, pick_directory, AppearanceSettings, ConfigError, ConfigManager,
    DownloadSettings, LauncherConfig, PrivacySettings,
};
