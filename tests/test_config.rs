//! Integration tests for Configuration & Directory Resolution.

use dv_launcher::config::ConfigManager;
use tempfile::tempdir;

#[test]
fn test_default_config() {
    let temp = tempdir().unwrap();
    let conf_file = temp.path().join("test_config.json");
    let mgr = ConfigManager::new(Some(&conf_file)).unwrap();

    assert!(!mgr.config.privacy.enable_telemetry);
    assert!(!mgr.config.privacy.enable_crash_reporting);
    assert!(!mgr.config.privacy.enable_discord_rpc);
    assert_eq!(mgr.config.downloads.max_concurrent_downloads, 8);
    assert_eq!(mgr.config.appearance.theme_mode, "dark");

    // Verify file saved
    assert!(conf_file.exists());
    let data = std::fs::read_to_string(&conf_file).unwrap();
    assert!(data.contains("\"version\": \"1.0.0\""));
}

#[test]
fn test_custom_config_saving() {
    let temp = tempdir().unwrap();
    let conf_file = temp.path().join("custom_config.json");
    let mut mgr = ConfigManager::new(Some(&conf_file)).unwrap();

    mgr.config.default_max_memory_mb = 8192;
    mgr.config.privacy.enable_discord_rpc = true;
    mgr.save().unwrap();

    // Re-read
    let mgr2 = ConfigManager::new(Some(&conf_file)).unwrap();
    assert_eq!(mgr2.config.default_max_memory_mb, 8192);
    assert!(mgr2.config.privacy.enable_discord_rpc);
}
