//! Multi-instance isolation and configuration subsystem.

pub mod manager;
pub mod mods;

pub use manager::{InstanceConfig, InstanceError, InstanceManager};
pub use mods::{
    delete_mod, import_mod_file, open_mods_folder, scan_mods, sync_instance_mods, toggle_mod,
    ModEntry,
};
