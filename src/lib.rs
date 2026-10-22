//! Digital Vigital (DV) Launcher Engine Library.

pub mod accounts;
pub mod branding;
pub mod config;
pub mod desktop;
pub mod discord;
pub mod gpu_detector;
pub mod instances;
pub mod java_detector;
pub mod java_installer;
pub mod loaders;
pub mod minecraft;
pub mod security;
pub mod skins;
#[cfg(feature = "slint-ui")]
pub mod ui;
