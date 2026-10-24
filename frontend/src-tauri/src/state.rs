use dv_launcher::accounts::MicrosoftAuthClient;
use dv_launcher::config::ConfigManager;
use dv_launcher::discord::DiscordRpcHandle;
use dv_launcher::instances::InstanceManager;
use dv_launcher::minecraft::ProcessSupervisor;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub config_mgr: Arc<Mutex<ConfigManager>>,
    pub instance_mgr: Arc<Mutex<InstanceManager>>,
    pub auth_client: Arc<Mutex<MicrosoftAuthClient>>,
    pub supervisor: Arc<ProcessSupervisor>,
    pub discord_rpc: Arc<DiscordRpcHandle>,
    pub is_launching: Arc<AtomicBool>,
    pub base_data_dir: PathBuf,
}

#[derive(Serialize)]
pub struct LauncherInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub repo_url: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InstanceDto {
    pub id: String,
    pub name: String,
    pub mc_version: String,
    pub mod_loader: String,
    pub last_played: Option<u64>,
    pub min_memory_mb: u32,
    pub max_memory_mb: u32,
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AccountDto {
    pub id: String,
    pub username: String,
    pub account_type: String, // "microsoft" or "offline"
    pub avatar_url: Option<String>,
    pub last_login: u64,
    pub is_active: bool,
}

#[derive(Serialize)]
pub struct JavaDto {
    pub path: String,
    pub vendor: String,
    pub version: String,
    pub major_version: u32,
    pub display_name: String,
    pub is_recommended: bool,
}

#[derive(Serialize)]
pub struct GpuDto {
    pub id: String,
    pub name: String,
    pub is_discrete: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SettingsDto {
    pub default_min_memory_mb: u32,
    pub default_max_memory_mb: u32,
    pub default_window_width: u32,
    pub default_window_height: u32,
    pub selected_gpu: Option<String>,
    pub custom_jvm_args: String,
    pub performance_preset: String,
    pub on_launch_action: String,
    pub enable_discord_rpc: bool,
    pub data_dir: String,
    pub instances_dir: String,
    pub custom_java_path: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct DownloadProgressDto {
    pub current: u64,
    pub total: u64,
    pub percent: f32,
    pub status: String,
    pub speed_bytes_per_sec: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MinecraftVersionDto {
    pub id: String,
    pub version_type: String, // "release", "snapshot", "old_beta", "old_alpha"
    pub release_time: String,
    pub is_latest_release: bool,
    pub is_latest_snapshot: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerSkinPayload {
    pub skin_data_url: String,
    pub cape_data_url: Option<String>,
}
