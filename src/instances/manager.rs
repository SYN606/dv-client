//! Multi-Instance Management Subsystem.
//!
//! Provides isolated Minecraft directories, version mappings, custom memory limits,
//! mod loader assignments, and persistent `instance.json` definitions.

use crate::config::LauncherConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InstanceError {
    #[error("I/O error during instance operation: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Instance '{0}' not found")]
    NotFound(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstanceConfig {
    pub id: String,
    pub name: String,
    pub minecraft_version: String,
    #[serde(default = "default_loader")]
    pub mod_loader: String, // "vanilla", "fabric", "quilt", "neoforge", "forge"
    pub loader_version: Option<String>,
    pub custom_java_path: Option<PathBuf>,
    #[serde(default = "default_min_memory")]
    pub min_memory_mb: u32,
    #[serde(default = "default_max_memory")]
    pub max_memory_mb: u32,
    #[serde(default = "default_window_width")]
    pub window_width: u32,
    #[serde(default = "default_window_height")]
    pub window_height: u32,
    #[serde(default)]
    pub custom_jvm_args: Vec<String>,
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default = "current_timestamp")]
    pub created_at: u64,
    pub last_played: Option<u64>,
}

fn default_loader() -> String {
    "fabric".to_string()
}
fn default_min_memory() -> u32 {
    1024
}
fn default_max_memory() -> u32 {
    4096
}
fn default_window_width() -> u32 {
    1280
}
fn default_window_height() -> u32 {
    720
}
fn default_icon() -> String {
    "grass_block".to_string()
}
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl InstanceConfig {
    pub fn new(
        name: &str,
        minecraft_version: &str,
        mod_loader: &str,
        loader_version: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            minecraft_version: minecraft_version.to_string(),
            mod_loader: mod_loader.to_string(),
            loader_version,
            custom_java_path: None,
            min_memory_mb: default_min_memory(),
            max_memory_mb: default_max_memory(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            custom_jvm_args: Vec::new(),
            icon: default_icon(),
            created_at: current_timestamp(),
            last_played: None,
        }
    }

    pub fn instance_dir<P: AsRef<Path>>(&self, instances_dir: P) -> PathBuf {
        instances_dir.as_ref().join(&self.id)
    }

    pub fn game_dir<P: AsRef<Path>>(&self, instances_dir: P) -> PathBuf {
        self.instance_dir(instances_dir).join(".minecraft")
    }
}

pub struct InstanceManager {
    pub instances_dir: PathBuf,
    pub instances: HashMap<String, InstanceConfig>,
}

impl InstanceManager {
    pub fn new<P: AsRef<Path>>(instances_dir: P) -> Result<Self, InstanceError> {
        let dir = instances_dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir)?;

        let mut mgr = Self {
            instances_dir: dir,
            instances: HashMap::new(),
        };

        mgr.load_all()?;

        if mgr.instances.is_empty() {
            mgr.create_instance("Fabric 1.21.4", "1.21.4", "fabric", None)?;
        }

        Ok(mgr)
    }

    /// Load all instances from disk by reading each folder's `instance.json`.
    pub fn load_all(&mut self) -> Result<(), InstanceError> {
        self.instances.clear();

        if let Ok(entries) = fs::read_dir(&self.instances_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let conf_file = entry.path().join("instance.json");
                    if conf_file.exists() {
                        if let Ok(content) = fs::read_to_string(&conf_file) {
                            match serde_json::from_str::<InstanceConfig>(&content) {
                                Ok(inst) => {
                                    self.instances.insert(inst.id.clone(), inst);
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to parse instance JSON at {:?}: {}",
                                        conf_file, e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Create and persist a new isolated instance.
    pub fn create_instance(
        &mut self,
        name: &str,
        minecraft_version: &str,
        mod_loader: &str,
        loader_version: Option<String>,
    ) -> Result<InstanceConfig, InstanceError> {
        let inst = InstanceConfig::new(name, minecraft_version, mod_loader, loader_version);
        self.save_instance(&inst)?;
        self.instances.insert(inst.id.clone(), inst.clone());
        info!("Created instance '{}' ({})", inst.name, inst.id);
        Ok(inst)
    }

    /// Save instance configuration to its dedicated directory.
    pub fn save_instance(&self, inst: &InstanceConfig) -> Result<(), InstanceError> {
        let inst_dir = inst.instance_dir(&self.instances_dir);
        let game_dir = inst.game_dir(&self.instances_dir);

        fs::create_dir_all(&inst_dir)?;
        fs::create_dir_all(&game_dir)?;

        let conf_file = inst_dir.join("instance.json");
        let json_data = serde_json::to_string_pretty(inst)?;
        let mut file = File::create(conf_file)?;
        file.write_all(json_data.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// Delete an instance directory and remove it from manager.
    pub fn delete_instance(&mut self, id: &str) -> Result<bool, InstanceError> {
        if let Some(inst) = self.instances.remove(id) {
            let inst_dir = inst.instance_dir(&self.instances_dir);
            if inst_dir.exists() {
                let _ = fs::remove_dir_all(&inst_dir);
            }
            info!("Deleted instance '{}' ({})", inst.name, id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Clone an existing instance with a newly generated UUID and custom name.
    pub fn duplicate_instance(
        &mut self,
        src_id: &str,
        new_name: &str,
    ) -> Result<InstanceConfig, InstanceError> {
        let src = self
            .instances
            .get(src_id)
            .ok_or_else(|| InstanceError::NotFound(src_id.to_string()))?;

        let src_name = src.name.clone();
        let mut cloned = src.clone();
        cloned.id = Uuid::new_v4().to_string();
        cloned.name = new_name.to_string();
        cloned.created_at = current_timestamp();
        cloned.last_played = None;

        self.save_instance(&cloned)?;
        self.instances.insert(cloned.id.clone(), cloned.clone());
        info!(
            "Duplicated instance '{}' -> '{}' ({})",
            src_name, cloned.name, cloned.id
        );
        Ok(cloned)
    }

    pub fn get_instance(&self, id: &str) -> Option<&InstanceConfig> {
        self.instances.get(id)
    }

    pub fn get_active_instance<'a>(
        &'a self,
        config: &'a LauncherConfig,
    ) -> Option<&'a InstanceConfig> {
        if let Some(ref active_id) = config.active_instance_id {
            if let Some(inst) = self.instances.get(active_id) {
                return Some(inst);
            }
        }
        self.instances.values().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_instance_creation_and_isolation() {
        let temp = tempdir().unwrap();
        let mut mgr = InstanceManager::new(temp.path()).unwrap();

        let inst = mgr
            .create_instance(
                "Test Cyber World",
                "1.20.4",
                "fabric",
                Some("0.15.7".to_string()),
            )
            .unwrap();

        assert_eq!(inst.name, "Test Cyber World");
        assert_eq!(inst.minecraft_version, "1.20.4");
        assert_eq!(inst.mod_loader, "fabric");
        assert!(inst.instance_dir(temp.path()).exists());
        assert!(inst.game_dir(temp.path()).exists());
        assert!(inst
            .instance_dir(temp.path())
            .join("instance.json")
            .exists());

        // Duplicate
        let cloned = mgr.duplicate_instance(&inst.id, "Cloned World").unwrap();
        assert_ne!(cloned.id, inst.id);
        assert_eq!(cloned.name, "Cloned World");
        assert_eq!(cloned.minecraft_version, "1.20.4");

        // Delete cloned
        assert!(mgr.delete_instance(&cloned.id).unwrap());
        assert!(!cloned.instance_dir(temp.path()).exists());

        // Delete original
        assert!(mgr.delete_instance(&inst.id).unwrap());
        assert!(!inst.instance_dir(temp.path()).exists());
    }
}
