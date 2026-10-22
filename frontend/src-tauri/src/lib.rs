use dv_launcher::accounts::MicrosoftAuthClient;
use dv_launcher::branding;
use dv_launcher::config::ConfigManager;
use dv_launcher::discord::DiscordRpcHandle;
use dv_launcher::gpu_detector::GpuDetector;
use dv_launcher::instances::{
    scan_mods, toggle_mod as toggle_instance_mod, InstanceManager, ModEntry,
};
use dv_launcher::java_detector::JavaDetector;
use dv_launcher::java_installer::JavaInstaller;
use dv_launcher::loaders::{ModrinthClient, ModrinthMod};
use dv_launcher::minecraft::ProcessSupervisor;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tracing::{info, warn};

pub struct AppState {
    pub config_mgr: Arc<Mutex<ConfigManager>>,
    pub instance_mgr: Arc<Mutex<InstanceManager>>,
    pub auth_client: Arc<Mutex<MicrosoftAuthClient>>,
    pub supervisor: Arc<ProcessSupervisor>,
    pub discord_rpc: Arc<DiscordRpcHandle>,
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

#[tauri::command]
fn get_launcher_info() -> LauncherInfo {
    LauncherInfo {
        name: branding::APP_NAME.to_string(),
        version: branding::APP_VERSION.to_string(),
        author: "Digital Vigital".to_string(),
        repo_url: "https://github.com/SYN606/dv-client".to_string(),
    }
}

#[tauri::command]
async fn get_instances(state: State<'_, AppState>) -> Result<Vec<InstanceDto>, String> {
    let mgr = state.instance_mgr.lock().await;
    let mut list: Vec<InstanceDto> = mgr
        .instances
        .values()
        .map(|inst| InstanceDto {
            id: inst.id.clone(),
            name: inst.name.clone(),
            mc_version: inst.minecraft_version.clone(),
            mod_loader: inst.mod_loader.clone(),
            last_played: inst.last_played,
        })
        .collect();

    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(list)
}

#[tauri::command]
async fn create_instance(
    name: String,
    mc_version: String,
    loader: String,
    state: State<'_, AppState>,
) -> Result<InstanceDto, String> {
    let mut mgr = state.instance_mgr.lock().await;
    let inst = mgr
        .create_instance(&name, &mc_version, &loader, None)
        .map_err(|e| e.to_string())?;

    Ok(InstanceDto {
        id: inst.id.clone(),
        name: inst.name.clone(),
        mc_version: inst.minecraft_version.clone(),
        mod_loader: inst.mod_loader.clone(),
        last_played: inst.last_played,
    })
}

#[tauri::command]
async fn delete_instance(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut mgr = state.instance_mgr.lock().await;
    mgr.delete_instance(&id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn detect_java() -> Result<Vec<JavaDto>, String> {
    let installations = JavaDetector::scan_system();
    Ok(installations
        .into_iter()
        .map(|j| JavaDto {
            path: j.path.to_string_lossy().to_string(),
            vendor: j.vendor,
            version: j.full_version,
            major_version: j.major_version,
            display_name: j.display_name,
            is_recommended: j.major_version >= 21,
        })
        .collect())
}

#[tauri::command]
async fn install_java(major_version: u32, state: State<'_, AppState>) -> Result<String, String> {
    let runtimes_dir = {
        let cm = state.config_mgr.lock().await;
        cm.config.runtimes_dir()
    };

    let path = JavaInstaller::install_java_version(major_version, &runtimes_dir, |_, _| {})
        .await
        .map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn detect_gpus() -> Result<Vec<GpuDto>, String> {
    let gpus = GpuDetector::detect_gpus();
    Ok(gpus
        .into_iter()
        .map(|g| GpuDto {
            id: g.id,
            name: g.name,
            is_discrete: g.is_discrete,
        })
        .collect())
}

#[tauri::command]
async fn get_installed_mods(
    instance_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ModEntry>, String> {
    let mgr = state.instance_mgr.lock().await;
    let inst = mgr
        .instances
        .get(&instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;

    let mods_dir = inst.game_dir(&mgr.instances_dir).join("mods");
    if !mods_dir.exists() {
        return Ok(Vec::new());
    }

    Ok(scan_mods(&mods_dir))
}

#[tauri::command]
async fn toggle_mod(
    instance_id: String,
    filename: String,
    _enabled: bool,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let mgr = state.instance_mgr.lock().await;
    let inst = mgr
        .instances
        .get(&instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;

    let mods_dir = inst.game_dir(&mgr.instances_dir).join("mods");
    toggle_instance_mod(&mods_dir, &filename).map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_modrinth(
    query: String,
    loader: Option<String>,
    game_version: Option<String>,
) -> Result<Vec<ModrinthMod>, String> {
    let client = ModrinthClient::new();
    client
        .search_mods(&query, loader.as_deref(), game_version.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();
    info!("Starting DV Client Tauri v2 Engine");

    let config_mgr = match ConfigManager::new::<&str>(None) {
        Ok(c) => Arc::new(Mutex::new(c)),
        Err(e) => {
            warn!("Failed to load config: {}", e);
            return;
        }
    };

    let base_data_dir = {
        let mgr = config_mgr.blocking_lock();
        mgr.config.base_path().to_path_buf()
    };

    let instances_dir = {
        let mgr = config_mgr.blocking_lock();
        mgr.config.instances_dir()
    };

    let instance_mgr = match InstanceManager::new(&instances_dir) {
        Ok(i) => Arc::new(Mutex::new(i)),
        Err(e) => {
            warn!("Failed to init instances: {}", e);
            return;
        }
    };

    let auth_client = Arc::new(Mutex::new(MicrosoftAuthClient::new(&base_data_dir)));
    let supervisor = Arc::new(ProcessSupervisor::new());
    let discord_rpc = Arc::new(DiscordRpcHandle::new(None));

    let app_state = AppState {
        config_mgr,
        instance_mgr,
        auth_client,
        supervisor,
        discord_rpc,
        base_data_dir,
    };

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_launcher_info,
            get_instances,
            create_instance,
            delete_instance,
            detect_java,
            install_java,
            detect_gpus,
            get_installed_mods,
            toggle_mod,
            search_modrinth,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
