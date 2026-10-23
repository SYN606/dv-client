use dv_launcher::accounts::MicrosoftAuthClient;
use dv_launcher::branding;
use dv_launcher::config::ConfigManager;
use dv_launcher::discord::DiscordRpcHandle;
use dv_launcher::gpu_detector::GpuDetector;
use dv_launcher::instances::{
    scan_mods, sync_instance_mods, toggle_mod as toggle_instance_mod, InstanceManager, ModEntry,
};
use dv_launcher::java_detector::JavaDetector;
use dv_launcher::java_installer::JavaInstaller;
use dv_launcher::loaders::{ModLoaderClient, ModLoaderLaunchInfo, ModrinthClient, ModrinthMod};
use dv_launcher::minecraft::{
    DownloadProgress, InstallationPipeline, LaunchArgumentBuilder, LaunchContext, ProcessState,
    ProcessSupervisor,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex;
use tracing::info;

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

// ---------------------------------------------------------------------------
// System & App Commands
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone)]
pub struct MinecraftVersionDto {
    pub id: String,
    pub version_type: String, // "release", "snapshot", "old_beta", "old_alpha"
    pub release_time: String,
    pub is_latest_release: bool,
    pub is_latest_snapshot: bool,
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
async fn get_all_minecraft_versions(
    state: State<'_, AppState>,
) -> Result<Vec<MinecraftVersionDto>, String> {
    let cfg = {
        let cm = state.config_mgr.lock().await;
        cm.config.clone()
    };

    let meta_client = dv_launcher::minecraft::PistonMetaClient::new(
        cfg.shared_versions_dir().join("meta_cache"),
    );

    match meta_client.get_version_manifest(false).await {
        Ok(manifest) => {
            let latest_rel = manifest.latest.release;
            let latest_snap = manifest.latest.snapshot;
            let list = manifest
                .versions
                .into_iter()
                .map(|v| MinecraftVersionDto {
                    is_latest_release: v.id == latest_rel,
                    is_latest_snapshot: v.id == latest_snap,
                    id: v.id,
                    version_type: v.version_type,
                    release_time: v.release_time,
                })
                .collect();
            Ok(list)
        }
        Err(e) => {
            tracing::warn!("Failed to fetch Mojang manifest: {}. Using fallback list.", e);
            let fallback_releases = [
                "1.21.4", "1.21.3", "1.21.2", "1.21.1", "1.21",
                "1.20.6", "1.20.4", "1.20.2", "1.20.1", "1.20",
                "1.19.4", "1.19.3", "1.19.2", "1.19.1", "1.19",
                "1.18.2", "1.18.1", "1.18", "1.17.1", "1.17",
                "1.16.5", "1.16.4", "1.16.3", "1.16.2", "1.16.1", "1.16",
                "1.15.2", "1.14.4", "1.13.2", "1.12.2", "1.11.2", "1.10.2",
                "1.9.4", "1.8.9", "1.7.10", "1.6.4", "1.5.2"
            ];
            let list = fallback_releases
                .iter()
                .map(|id| MinecraftVersionDto {
                    id: id.to_string(),
                    version_type: "release".to_string(),
                    release_time: String::new(),
                    is_latest_release: *id == "1.21.4",
                    is_latest_snapshot: false,
                })
                .collect();
            Ok(list)
        }
    }
}

// ---------------------------------------------------------------------------
// Instance Management Commands
// ---------------------------------------------------------------------------

#[tauri::command]
async fn get_instances(state: State<'_, AppState>) -> Result<Vec<InstanceDto>, String> {
    let mgr = state.instance_mgr.lock().await;
    let cm = state.config_mgr.lock().await;
    let active_id = cm.config.active_instance_id.as_deref();

    let mut list: Vec<InstanceDto> = mgr
        .instances
        .values()
        .map(|inst| InstanceDto {
            id: inst.id.clone(),
            name: inst.name.clone(),
            mc_version: inst.minecraft_version.clone(),
            mod_loader: inst.mod_loader.clone(),
            last_played: inst.last_played,
            min_memory_mb: inst.min_memory_mb,
            max_memory_mb: inst.max_memory_mb,
            is_active: active_id == Some(&inst.id),
        })
        .collect();

    list.sort_by_key(|a| a.name.to_lowercase());
    Ok(list)
}

#[tauri::command]
async fn select_active_instance(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut cm = state.config_mgr.lock().await;
    cm.config.active_instance_id = Some(id);
    cm.save().map_err(|e| e.to_string())?;
    Ok(())
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

    // Auto set as active instance
    let mut cm = state.config_mgr.lock().await;
    cm.config.active_instance_id = Some(inst.id.clone());
    let _ = cm.save();

    Ok(InstanceDto {
        id: inst.id.clone(),
        name: inst.name.clone(),
        mc_version: inst.minecraft_version.clone(),
        mod_loader: inst.mod_loader.clone(),
        last_played: inst.last_played,
        min_memory_mb: inst.min_memory_mb,
        max_memory_mb: inst.max_memory_mb,
        is_active: true,
    })
}

#[tauri::command]
async fn delete_instance(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut mgr = state.instance_mgr.lock().await;
    mgr.delete_instance(&id).map_err(|e| e.to_string())?;

    let mut cm = state.config_mgr.lock().await;
    if cm.config.active_instance_id.as_deref() == Some(&id) {
        cm.config.active_instance_id = None;
        let _ = cm.save();
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Account Management Commands
// ---------------------------------------------------------------------------

#[tauri::command]
async fn get_accounts(state: State<'_, AppState>) -> Result<Vec<AccountDto>, String> {
    let auth = state.auth_client.lock().await;
    let cm = state.config_mgr.lock().await;
    let active_id = cm.config.active_account_id.as_deref();

    let mut list: Vec<AccountDto> = auth
        .accounts
        .values()
        .map(|acc| AccountDto {
            id: acc.id.clone(),
            username: acc.username.clone(),
            account_type: acc.account_type.clone(),
            avatar_url: acc.avatar_url.clone(),
            last_login: acc.last_login,
            is_active: active_id == Some(&acc.id),
        })
        .collect();

    list.sort_by_key(|a| std::cmp::Reverse(a.last_login));
    Ok(list)
}

#[tauri::command]
async fn select_active_account(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut cm = state.config_mgr.lock().await;
    cm.config.active_account_id = Some(id);
    cm.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn add_offline_account(
    username: String,
    state: State<'_, AppState>,
) -> Result<AccountDto, String> {
    let mut auth = state.auth_client.lock().await;
    let acc = auth
        .create_offline_account(&username)
        .map_err(|e| e.to_string())?;

    let mut cm = state.config_mgr.lock().await;
    cm.config.active_account_id = Some(acc.id.clone());
    let _ = cm.save();

    Ok(AccountDto {
        id: acc.id.clone(),
        username: acc.username.clone(),
        account_type: acc.account_type.clone(),
        avatar_url: acc.avatar_url.clone(),
        last_login: acc.last_login,
        is_active: true,
    })
}

#[tauri::command]
async fn logout_account(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut auth = state.auth_client.lock().await;
    auth.logout(&id).map_err(|e| e.to_string())?;

    let mut cm = state.config_mgr.lock().await;
    if cm.config.active_account_id.as_deref() == Some(&id) {
        cm.config.active_account_id = None;
        let _ = cm.save();
    }
    Ok(())
}

#[tauri::command]
async fn start_microsoft_login(state: State<'_, AppState>) -> Result<String, String> {
    let mut auth = state.auth_client.lock().await;
    let url = auth.generate_login_url();
    let _ = open::that(&url);
    Ok(url)
}

#[tauri::command]
async fn complete_microsoft_login(
    code: String,
    state: State<'_, AppState>,
) -> Result<AccountDto, String> {
    let mut auth = state.auth_client.lock().await;
    let acc = auth
        .complete_login_with_code(&code)
        .await
        .map_err(|e| e.to_string())?;

    let mut cm = state.config_mgr.lock().await;
    cm.config.active_account_id = Some(acc.id.clone());
    let _ = cm.save();

    Ok(AccountDto {
        id: acc.id.clone(),
        username: acc.username.clone(),
        account_type: acc.account_type.clone(),
        avatar_url: acc.avatar_url.clone(),
        last_login: acc.last_login,
        is_active: true,
    })
}

// ---------------------------------------------------------------------------
// Java Runtime Commands
// ---------------------------------------------------------------------------

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
async fn install_java(
    major_version: u32,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let runtimes_dir = {
        let cm = state.config_mgr.lock().await;
        cm.config.runtimes_dir()
    };

    let handle = app_handle.clone();
    let progress_cb = move |percent: f32, status: &str| {
        let _ = handle.emit(
            "java-install-progress",
            serde_json::json!({
                "status": status,
                "percent": percent,
            }),
        );
    };

    let path = JavaInstaller::install_java_version(major_version, &runtimes_dir, progress_cb)
        .await
        .map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn pick_custom_java() -> Result<Option<JavaDto>, String> {
    let picked = tokio::task::spawn_blocking(JavaDetector::pick_java_with_dialog)
        .await
        .map_err(|e| e.to_string())?;

    Ok(picked.map(|j| JavaDto {
        path: j.path.to_string_lossy().to_string(),
        vendor: j.vendor,
        version: j.full_version,
        major_version: j.major_version,
        display_name: j.display_name,
        is_recommended: j.major_version >= 21,
    }))
}

// ---------------------------------------------------------------------------
// Hardware & Settings Commands
// ---------------------------------------------------------------------------

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
async fn get_settings(state: State<'_, AppState>) -> Result<SettingsDto, String> {
    let cm = state.config_mgr.lock().await;
    let cfg = &cm.config;
    Ok(SettingsDto {
        default_min_memory_mb: cfg.default_min_memory_mb,
        default_max_memory_mb: cfg.default_max_memory_mb,
        default_window_width: cfg.default_window_width,
        default_window_height: cfg.default_window_height,
        selected_gpu: cfg.selected_gpu.clone(),
        custom_jvm_args: cfg.custom_jvm_args.clone(),
        performance_preset: cfg.performance_preset.clone(),
        on_launch_action: cfg.on_launch_action.clone(),
        enable_discord_rpc: cfg.privacy.enable_discord_rpc,
        data_dir: cfg.data_dir.to_string_lossy().to_string(),
        instances_dir: cfg.instances_dir().to_string_lossy().to_string(),
        custom_java_path: cfg
            .default_java_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
    })
}

#[tauri::command]
async fn save_settings(settings: SettingsDto, state: State<'_, AppState>) -> Result<(), String> {
    let mut cm = state.config_mgr.lock().await;
    cm.config.default_min_memory_mb = settings.default_min_memory_mb;
    cm.config.default_max_memory_mb = settings.default_max_memory_mb;
    cm.config.default_window_width = settings.default_window_width;
    cm.config.default_window_height = settings.default_window_height;
    cm.config.selected_gpu = settings.selected_gpu;
    cm.config.custom_jvm_args = settings.custom_jvm_args;
    cm.config.performance_preset = settings.performance_preset;
    cm.config.on_launch_action = settings.on_launch_action;
    cm.config.privacy.enable_discord_rpc = settings.enable_discord_rpc;
    cm.config.default_java_path = settings.custom_java_path.map(PathBuf::from);

    cm.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn open_folder(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Mod Management Commands
// ---------------------------------------------------------------------------

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
    query: Option<String>,
    loader: Option<String>,
    game_version: Option<String>,
    category: Option<String>,
    sort_by: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<ModrinthMod>, String> {
    let client = ModrinthClient::new();
    let q = query.unwrap_or_default();
    let lim = limit.unwrap_or(30);
    client
        .search_mods_advanced(
            &q,
            game_version.as_deref(),
            loader.as_deref(),
            category.as_deref(),
            sort_by.as_deref(),
            lim,
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_mod(
    instance_id: String,
    filename: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.instance_mgr.lock().await;
    let inst = mgr
        .instances
        .get(&instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;

    let mods_dir = inst.game_dir(&mgr.instances_dir).join("mods");
    let target = mods_dir.join(&filename);
    if target.exists() {
        std::fs::remove_file(&target).map_err(|e| format!("Failed to delete mod: {}", e))?;
    }
    let alt_name = if filename.ends_with(".disabled") {
        filename.trim_end_matches(".disabled").to_string()
    } else {
        format!("{}.disabled", filename)
    };
    let alt_target = mods_dir.join(alt_name);
    if alt_target.exists() {
        let _ = std::fs::remove_file(&alt_target);
    }
    Ok(())
}

#[tauri::command]
async fn install_modrinth_mod(
    instance_id: String,
    project_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mgr = state.instance_mgr.lock().await;
    let inst = mgr
        .instances
        .get(&instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;

    let mods_dir = inst.game_dir(&mgr.instances_dir).join("mods");
    let mc_ver = inst.minecraft_version.clone();
    let loader = inst.mod_loader.clone();

    let client = ModrinthClient::new();
    client
        .install_mod(&project_id, &mc_ver, &loader, &mods_dir)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_curseforge(
    query: Option<String>,
    loader: Option<String>,
    game_version: Option<String>,
    category_id: Option<u32>,
    sort_by: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<dv_launcher::loaders::CurseForgeMod>, String> {
    let client = dv_launcher::loaders::CurseForgeClient::new();
    let q = query.unwrap_or_default();
    let lim = limit.unwrap_or(30);
    client
        .search_mods(
            &q,
            game_version.as_deref(),
            loader.as_deref(),
            category_id,
            sort_by.as_deref(),
            lim,
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_curseforge_mod(
    instance_id: String,
    mod_id: u64,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mgr = state.instance_mgr.lock().await;
    let inst = mgr
        .instances
        .get(&instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;

    let mods_dir = inst.game_dir(&mgr.instances_dir).join("mods");
    let mc_ver = inst.minecraft_version.clone();
    let loader = inst.mod_loader.clone();

    let client = dv_launcher::loaders::CurseForgeClient::new();
    client
        .install_mod(mod_id, &mc_ver, &loader, &mods_dir)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_mods_folder(instance_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mgr = state.instance_mgr.lock().await;
    let inst = mgr
        .instances
        .get(&instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;

    let mods_dir = inst.game_dir(&mgr.instances_dir).join("mods");
    let _ = std::fs::create_dir_all(&mods_dir);
    open::that(&mods_dir).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn import_local_mod(
    instance_id: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let mgr = state.instance_mgr.lock().await;
    let inst = mgr
        .instances
        .get(&instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;

    let mods_dir = inst.game_dir(&mgr.instances_dir).join("mods");
    let file = tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Select Minecraft Mod (.jar)")
            .add_filter("Minecraft Mod Jar (*.jar)", &["jar"])
            .pick_file()
    })
    .await
    .map_err(|e| e.to_string())?;

    if let Some(src) = file {
        let dest = dv_launcher::instances::import_mod_file(&mods_dir, &src)
            .map_err(|e| format!("Failed to import mod file: {}", e))?;
        let name = dest
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "mod.jar".to_string());
        Ok(Some(name))
    } else {
        Ok(None)
    }
}

#[tauri::command]
async fn bulk_toggle_mods(
    instance_id: String,
    enable_all: bool,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let mgr = state.instance_mgr.lock().await;
    let inst = mgr
        .instances
        .get(&instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;

    let mods_dir = inst.game_dir(&mgr.instances_dir).join("mods");
    if !mods_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(&mods_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let filename = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            if enable_all {
                if filename.ends_with(".jar.disabled") {
                    let new_name = filename.trim_end_matches(".disabled");
                    if std::fs::rename(&path, mods_dir.join(new_name)).is_ok() {
                        count += 1;
                    }
                } else if filename.ends_with(".disabled") {
                    let new_name = format!("{}.jar", filename.trim_end_matches(".disabled"));
                    if std::fs::rename(&path, mods_dir.join(new_name)).is_ok() {
                        count += 1;
                    }
                }
            } else if filename.ends_with(".jar") {
                let new_name = format!("{}.disabled", filename);
                if std::fs::rename(&path, mods_dir.join(new_name)).is_ok() {
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}

#[tauri::command]
async fn uninstall_modrinth_mod(
    instance_id: String,
    slug: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let mgr = state.instance_mgr.lock().await;
    let inst = mgr
        .instances
        .get(&instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;

    let mods_dir = inst.game_dir(&mgr.instances_dir).join("mods");
    if !mods_dir.exists() {
        return Ok(false);
    }

    let clean_slug = slug.to_lowercase().replace('-', "_");
    let slug_dash = slug.to_lowercase().replace('_', "-");
    let mut removed = false;

    if let Ok(entries) = std::fs::read_dir(&mods_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_lowercase(),
                None => continue,
            };

            if (name.contains(&clean_slug) || name.contains(&slug_dash))
                && (name.ends_with(".jar") || name.ends_with(".disabled"))
            {
                if std::fs::remove_file(entry.path()).is_ok() {
                    removed = true;
                }
            }
        }
    }

    Ok(removed)
}

// ---------------------------------------------------------------------------
// Skins & Capes Commands
// ---------------------------------------------------------------------------

#[tauri::command]
async fn pick_skin_file() -> Result<Option<String>, String> {
    let file = tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Select Minecraft Skin PNG (64x64)")
            .add_filter("Skin PNG", &["png"])
            .pick_file()
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(file.map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
async fn pick_cape_file() -> Result<Option<String>, String> {
    let file = tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Select Minecraft Cape PNG (64x32)")
            .add_filter("Cape PNG", &["png"])
            .pick_file()
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(file.map(|p| p.to_string_lossy().to_string()))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerSkinPayload {
    pub skin_data_url: String,
    pub cape_data_url: Option<String>,
}

fn fast_base64_encode(data: &[u8]) -> String {
    const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        out.push(B64[(b0 >> 2) as usize] as char);
        out.push(B64[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(B64[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(B64[(b2 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[tauri::command]
async fn get_player_skin(username: Option<String>) -> Result<PlayerSkinPayload, String> {
    let skin_path = dv_launcher::skins::get_active_skin_path(username.as_deref());
    let skin_bytes = std::fs::read(&skin_path).map_err(|e| format!("Failed to read skin file: {}", e))?;
    let skin_data_url = format!("data:image/png;base64,{}", fast_base64_encode(&skin_bytes));

    let cape_data_url = if let Some(cape_path) = dv_launcher::skins::get_active_cape_path(username.as_deref()) {
        if let Ok(cape_bytes) = std::fs::read(&cape_path) {
            Some(format!("data:image/png;base64,{}", fast_base64_encode(&cape_bytes)))
        } else {
            None
        }
    } else {
        None
    };

    Ok(PlayerSkinPayload {
        skin_data_url,
        cape_data_url,
    })
}

#[tauri::command]
async fn reset_skin(username: String) -> Result<PlayerSkinPayload, String> {
    dv_launcher::skins::reset_player_skin(&username);
    get_player_skin(Some(username)).await
}

#[tauri::command]
async fn reset_cape(username: String) -> Result<PlayerSkinPayload, String> {
    dv_launcher::skins::reset_player_cape(&username);
    get_player_skin(Some(username)).await
}

#[tauri::command]
async fn apply_skin(username: String, file_path: String) -> Result<PlayerSkinPayload, String> {
    dv_launcher::skins::save_custom_skin(&username, &file_path).map_err(|e| e.to_string())?;
    get_player_skin(Some(username)).await
}

#[tauri::command]
async fn apply_cape(username: String, file_path: String) -> Result<PlayerSkinPayload, String> {
    dv_launcher::skins::save_custom_cape(&username, &file_path).map_err(|e| e.to_string())?;
    get_player_skin(Some(username)).await
}

// ---------------------------------------------------------------------------
// Game Launch & Lifecycle Commands
// ---------------------------------------------------------------------------

#[tauri::command]
async fn launch_game(
    instance_id: Option<String>,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if state.supervisor.is_active() {
        return Err("A Minecraft game instance is already running.".to_string());
    }

    if state.is_launching.load(Ordering::SeqCst) {
        return Err("Launch process is already underway.".to_string());
    }

    state.is_launching.store(true, Ordering::SeqCst);

    let (inst, config_clone) = {
        let mgr = state.instance_mgr.lock().await;
        let cm = state.config_mgr.lock().await;
        let target_id = instance_id.or_else(|| cm.config.active_instance_id.clone());
        let target = match target_id {
            Some(ref id) => mgr.instances.get(id).cloned(),
            None => None,
        };
        (target, cm.config.clone())
    };

    let inst = match inst {
        Some(i) => i,
        None => {
            state.is_launching.store(false, Ordering::SeqCst);
            return Err(
                "No profile selected. Please select or create an instance first.".to_string(),
            );
        }
    };

    let account = {
        let auth = state.auth_client.lock().await;
        let cm = state.config_mgr.lock().await;
        auth.get_active_account(cm.config.active_account_id.as_deref())
            .or_else(|| auth.accounts.values().next().cloned())
            .unwrap_or_else(|| {
                // Fallback offline player
                dv_launcher::accounts::MinecraftAccount {
                    id: "00000000-0000-0000-0000-000000000000".to_string(),
                    username: "Player".to_string(),
                    account_type: "offline".to_string(),
                    avatar_url: None,
                    last_login: 0,
                    access_token: Some("0".to_string()),
                }
            })
    };

    let req_ver = JavaDetector::required_java_major_version(&inst.minecraft_version);
    let detected_javas = JavaDetector::scan_system();
    let custom_path = inst
        .custom_java_path
        .as_ref()
        .or(config_clone.default_java_path.as_ref());

    let java_path = if let Some(p) = custom_path.filter(|p| p.exists()) {
        p.clone()
    } else if let Some(j) = JavaDetector::find_best_java(&detected_javas, &inst.minecraft_version) {
        j.path.clone()
    } else {
        state.is_launching.store(false, Ordering::SeqCst);
        let req_name = if req_ver == 8 { "Java 8" } else { "Java 21" };
        return Err(format!(
            "Required Java version ({}) not found for Minecraft {}. Please install it via Settings.",
            req_name, inst.minecraft_version
        ));
    };

    let inst_dir = inst.instance_dir(config_clone.instances_dir());
    let game_dir = inst.game_dir(config_clone.instances_dir());
    let _ = std::fs::create_dir_all(&game_dir);

    // Sync instance mods
    sync_instance_mods(&inst_dir, &game_dir);
    let _ = dv_launcher::skins::ensure_in_game_skin_pack(&game_dir, &account.username);

    let handle_for_prog = app_handle.clone();
    let _ = handle_for_prog.emit("launch-status", "Verifying and downloading game assets...");

    let pipeline = InstallationPipeline::new(&config_clone);
    let pkg_res = pipeline
        .install_version(
            &inst.minecraft_version,
            &inst_dir,
            &config_clone,
            Some(move |prog: DownloadProgress| {
                let _ = handle_for_prog.emit(
                    "download-progress",
                    serde_json::json!({
                        "current": prog.completed_items,
                        "total": prog.total_items,
                        "percent": prog.percent,
                        "status": prog.current_item,
                        "bytes_downloaded": prog.bytes_downloaded,
                        "total_bytes": prog.total_bytes,
                    }),
                );
            }),
            Some(|status: &str| {
                info!("[Tauri Pipeline] {}", status);
            }),
        )
        .await;

    let version_pkg = match pkg_res {
        Ok(pkg) => pkg,
        Err(e) => {
            state.is_launching.store(false, Ordering::SeqCst);
            let _ = app_handle.emit("launch-status", format!("Installation error: {}", e));
            return Err(format!("Download failed: {}", e));
        }
    };

    let _ = app_handle.emit("launch-status", "Resolving mod loader libraries...");

    let loader_client = ModLoaderClient::new();
    let loader_info = loader_client
        .resolve_and_download_loader(
            &inst.mod_loader,
            &inst.minecraft_version,
            inst.loader_version.as_deref(),
            &config_clone.shared_libraries_dir(),
        )
        .await
        .unwrap_or_else(|_| ModLoaderLaunchInfo::default());

    let mut combined_jvm_args = inst.custom_jvm_args.clone();
    if !config_clone.custom_jvm_args.trim().is_empty() {
        for arg in config_clone.custom_jvm_args.split_whitespace() {
            combined_jvm_args.push(arg.to_string());
        }
    }
    combined_jvm_args.extend(loader_info.extra_jvm_args);

    let client_jar = config_clone
        .shared_versions_dir()
        .join(&inst.minecraft_version)
        .join(format!("{}.jar", inst.minecraft_version));

    let access_token = {
        let mut a = state.auth_client.lock().await;
        match a.get_or_refresh_token(&account.id).await {
            Ok(tok) => tok,
            Err(_) => account
                .access_token
                .clone()
                .unwrap_or_else(|| "0".to_string()),
        }
    };

    let launch_ctx = LaunchContext {
        version_package: version_pkg,
        instance_dir: inst_dir.clone(),
        game_dir: game_dir.clone(),
        assets_dir: config_clone.shared_assets_dir(),
        libraries_dir: config_clone.shared_libraries_dir(),
        client_jar_path: client_jar,
        java_executable: java_path,
        player_name: account.username.clone(),
        player_uuid: account.id.clone(),
        access_token,
        user_type: if account.account_type == "microsoft" {
            "msa".to_string()
        } else {
            "offline".to_string()
        },
        xuid: "".to_string(),
        min_memory_mb: inst.min_memory_mb,
        max_memory_mb: inst.max_memory_mb,
        window_width: inst.window_width,
        window_height: inst.window_height,
        custom_jvm_args: Some(combined_jvm_args),
        performance_preset: Some(config_clone.performance_preset.clone()),
        mod_loader: inst.mod_loader.clone(),
        extra_classpath: loader_info.extra_libraries,
        main_class_override: loader_info.main_class,
        extra_game_args: loader_info.extra_game_args,
    };

    let command_args = LaunchArgumentBuilder::build_command_args(&launch_ctx);
    let gpu_id = config_clone
        .selected_gpu
        .unwrap_or_else(|| "auto".to_string());
    GpuDetector::configure_windows_gpu_preference(&launch_ctx.java_executable, &gpu_id);
    let gpu_env = GpuDetector::get_env_vars_for_gpu(&gpu_id);

    let _ = app_handle.emit("launch-status", "Launching Minecraft JVM...");

    let sup = state.supervisor.clone();
    match sup.launch_with_env(command_args, game_dir, gpu_env).await {
        Ok(_) => {
            state.is_launching.store(false, Ordering::SeqCst);
            let _ = app_handle.emit("launch-status", "Game is running!");

            // Update last_played on instance
            {
                let mgr = state.instance_mgr.lock().await;
                if let Some(mut inst_mut) = mgr.instances.get(&inst.id).cloned() {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    inst_mut.last_played = Some(now);
                    let _ = mgr.save_instance(&inst_mut);
                }
            }

            // Window on-launch action
            if config_clone.on_launch_action == "hide" {
                if let Some(win) = app_handle.get_webview_window("main") {
                    let _ = win.hide();
                }
            } else if config_clone.on_launch_action == "close" {
                app_handle.exit(0);
            }

            Ok(())
        }
        Err(e) => {
            state.is_launching.store(false, Ordering::SeqCst);
            let _ = app_handle.emit("launch-status", format!("Launch failed: {}", e));
            Err(format!("Execution failed: {}", e))
        }
    }
}

#[tauri::command]
async fn stop_game(state: State<'_, AppState>) -> Result<(), String> {
    state.supervisor.terminate().await;
    Ok(())
}

#[tauri::command]
async fn get_game_status(state: State<'_, AppState>) -> Result<String, String> {
    if state.supervisor.is_active() {
        Ok("running".to_string())
    } else if state.is_launching.load(Ordering::SeqCst) {
        Ok("launching".to_string())
    } else {
        Ok("idle".to_string())
    }
}

// ---------------------------------------------------------------------------
// Log Management Commands
// ---------------------------------------------------------------------------

#[tauri::command]
async fn get_latest_logs(
    instance_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let (inst, cfg) = {
        let mgr = state.instance_mgr.lock().await;
        let cm = state.config_mgr.lock().await;
        let target_id = instance_id.or_else(|| cm.config.active_instance_id.clone());
        let target = match target_id {
            Some(ref id) => mgr.instances.get(id).cloned(),
            None => None,
        };
        (target, cm.config.clone())
    };

    if let Some(inst) = inst {
        let log_file = inst
            .game_dir(cfg.instances_dir())
            .join("logs")
            .join("latest.log");
        if log_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&log_file) {
                return Ok(content);
            }
        }
    }
    Ok(String::new())
}

#[tauri::command]
async fn open_logs_folder(
    instance_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (inst, cfg) = {
        let mgr = state.instance_mgr.lock().await;
        let cm = state.config_mgr.lock().await;
        let target_id = instance_id.or_else(|| cm.config.active_instance_id.clone());
        let target = match target_id {
            Some(ref id) => mgr.instances.get(id).cloned(),
            None => None,
        };
        (target, cm.config.clone())
    };

    let logs_dir = if let Some(inst) = inst {
        inst.game_dir(cfg.instances_dir()).join("logs")
    } else {
        cfg.base_path().join("logs")
    };

    let _ = std::fs::create_dir_all(&logs_dir);
    open::that(&logs_dir).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn export_logs(content: String) -> Result<Option<String>, String> {
    let file = tokio::task::spawn_blocking(move || {
        let now = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let default_name = format!("minecraft_log_{}.log", now);
        rfd::FileDialog::new()
            .set_title("Export Minecraft Log")
            .set_file_name(&default_name)
            .add_filter("Log Files (*.log, *.txt)", &["log", "txt"])
            .save_file()
            .map(|p| {
                let _ = std::fs::write(&p, content);
                p.to_string_lossy().to_string()
            })
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(file)
}

// ---------------------------------------------------------------------------
// App Entry Point & Setup
// ---------------------------------------------------------------------------

fn append_startup_log(step: &str) {
    use std::io::Write;
    let log_path = dirs::data_dir()
        .map(|d| d.join("DigitalVigital").join("startup.log"))
        .unwrap_or_else(|| std::path::PathBuf::from("dv_startup.log"));
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(f, "[Tauri Engine] {}", step);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    append_startup_log("2. app_lib::run() entered");
    info!("Starting DV Client Tauri v2 Engine");

    append_startup_log("3. Initializing ConfigManager");
    let (config_mgr, base_data_dir, instances_dir) = match ConfigManager::new::<&str>(None) {
        Ok(c) => {
            let base = c.config.base_path().to_path_buf();
            let inst = c.config.instances_dir();
            append_startup_log(&format!(
                "Config loaded. Base: {:?}, Instances: {:?}",
                base, inst
            ));
            (Arc::new(Mutex::new(c)), base, inst)
        }
        Err(e) => {
            append_startup_log(&format!(
                "Config load error: {}. Using default fallback.",
                e
            ));
            let fallback_dir = dirs::data_dir()
                .map(|d| d.join("DigitalVigital"))
                .unwrap_or_else(std::env::temp_dir);
            let _ = std::fs::create_dir_all(&fallback_dir);
            let fallback_cfg = ConfigManager::new(Some(fallback_dir.join("config.json")))
                .unwrap_or_else(|_| {
                    let cfg = dv_launcher::config::LauncherConfig::default();
                    let file_path = fallback_dir.join("config.json");
                    let _ = std::fs::write(&file_path, serde_json::to_string(&cfg).unwrap());
                    ConfigManager::new(Some(file_path)).unwrap()
                });
            let base = fallback_cfg.config.base_path().to_path_buf();
            let inst = fallback_cfg.config.instances_dir();
            (Arc::new(Mutex::new(fallback_cfg)), base, inst)
        }
    };

    append_startup_log("4. Initializing InstanceManager");
    let instance_mgr = match InstanceManager::new(&instances_dir) {
        Ok(i) => {
            append_startup_log(&format!(
                "InstanceManager initialized with {} instances",
                i.instances.len()
            ));
            Arc::new(Mutex::new(i))
        }
        Err(e) => {
            append_startup_log(&format!(
                "InstanceManager init error: {}. Using fallback.",
                e
            ));
            let fallback_dir = base_data_dir.join("instances");
            let _ = std::fs::create_dir_all(&fallback_dir);
            let i = InstanceManager::new(&fallback_dir).unwrap_or_else(|_| InstanceManager {
                instances_dir: fallback_dir,
                instances: std::collections::HashMap::new(),
            });
            Arc::new(Mutex::new(i))
        }
    };

    append_startup_log("5. Initializing AuthClient & ProcessSupervisor");
    let auth_client = Arc::new(Mutex::new(MicrosoftAuthClient::new(&base_data_dir)));
    let supervisor = Arc::new(ProcessSupervisor::new());
    let discord_rpc = Arc::new(DiscordRpcHandle::new(None));
    let is_launching = Arc::new(AtomicBool::new(false));

    let sup_for_logs = supervisor.clone();
    let sup_for_state = supervisor.clone();

    let app_state = AppState {
        config_mgr,
        instance_mgr,
        auth_client,
        supervisor,
        discord_rpc,
        is_launching,
        base_data_dir,
    };

    append_startup_log("6. Building Tauri Application Window");

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            append_startup_log("7. Tauri setup() callback executing");
            let handle = app.handle().clone();

            // Background task: stream supervisor process stdout & stderr to Tauri frontend
            let handle_logs = handle.clone();
            let mut log_rx = sup_for_logs.output_tx.subscribe();
            tauri::async_runtime::spawn(async move {
                while let Ok(line) = log_rx.recv().await {
                    let _ = handle_logs.emit("game-log", &line);
                }
            });

            // Background task: stream process state transitions
            let handle_state = handle.clone();
            let mut state_rx = sup_for_state.state_tx.subscribe();
            tauri::async_runtime::spawn(async move {
                while let Ok(st) = state_rx.recv().await {
                    let (st_name, restore_win) = match st {
                        ProcessState::Stopped => ("stopped", true),
                        ProcessState::Starting => ("starting", false),
                        ProcessState::Running => ("running", false),
                        ProcessState::Exited(_) => ("finished", true),
                        ProcessState::Crashed(_) => ("crashed", true),
                    };

                    let _ = handle_state.emit("game-state-change", st_name);

                    // If game finished or crashed, restore window if it was hidden
                    if restore_win {
                        if let Some(win) = handle_state.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                }
            });

            append_startup_log("8. Tauri setup() finished, webview starting");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_launcher_info,
            get_all_minecraft_versions,
            get_instances,
            select_active_instance,
            create_instance,
            delete_instance,
            get_accounts,
            select_active_account,
            add_offline_account,
            logout_account,
            start_microsoft_login,
            complete_microsoft_login,
            detect_java,
            install_java,
            pick_custom_java,
            detect_gpus,
            get_settings,
            save_settings,
            open_folder,
            get_installed_mods,
            toggle_mod,
            delete_mod,
            bulk_toggle_mods,
            import_local_mod,
            uninstall_modrinth_mod,
            search_modrinth,
            install_modrinth_mod,
            search_curseforge,
            install_curseforge_mod,
            open_mods_folder,
            pick_skin_file,
            pick_cape_file,
            get_player_skin,
            apply_skin,
            apply_cape,
            reset_skin,
            reset_cape,
            launch_game,
            stop_game,
            get_game_status,
            get_latest_logs,
            open_logs_folder,
            export_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
