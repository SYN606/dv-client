//! Multi-instance isolation and configuration subsystem.

pub mod manager;
pub mod mods;

pub use manager::{InstanceConfig, InstanceError, InstanceManager};
pub use mods::{
    delete_mod, import_mod_file, load_mods_meta, open_mods_folder, record_installed_mod,
    scan_mods, sync_instance_mods, toggle_mod, unrecord_installed_mod, ModEntry, ModMetaRecord,
};
