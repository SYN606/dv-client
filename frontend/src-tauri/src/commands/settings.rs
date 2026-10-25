use crate::state::{AppState, GpuDto, JavaDto, SettingsDto};
use dv_launcher::gpu_detector::GpuDetector;
use dv_launcher::java_detector::JavaDetector;
use dv_launcher::java_installer::JavaInstaller;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn detect_java() -> Result<Vec<JavaDto>, String> {
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
pub async fn install_java(
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
pub async fn pick_custom_java() -> Result<Option<JavaDto>, String> {
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

#[tauri::command]
pub async fn detect_gpus() -> Result<Vec<GpuDto>, String> {
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
pub async fn get_settings(state: State<'_, AppState>) -> Result<SettingsDto, String> {
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
pub async fn pick_game_directory() -> Result<Option<String>, String> {
    let picked = tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Select Digital Vigital Game & Data Directory")
            .pick_folder()
            .map(|p| p.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(picked)
}

#[tauri::command]
pub async fn reset_game_directory(state: State<'_, AppState>) -> Result<SettingsDto, String> {
    let default_dir = dv_launcher::config::get_default_app_data_dir();
    {
        let mut cm = state.config_mgr.lock().await;
        cm.config.data_dir = default_dir.clone();
        let new_inst_dir = cm.config.instances_dir();
        let _ = std::fs::create_dir_all(&default_dir);
        let _ = std::fs::create_dir_all(&new_inst_dir);
        cm.save().map_err(|e| e.to_string())?;

        let mut im = state.instance_mgr.lock().await;
        im.instances_dir = new_inst_dir;
        let _ = im.load_all();
    }

    get_settings(state).await
}

#[tauri::command]
pub async fn pick_instances_directory() -> Result<Option<String>, String> {
    let picked = tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Select Digital Vigital Instances Directory")
            .pick_folder()
            .map(|p| p.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(picked)
}

#[tauri::command]
pub async fn reset_instances_directory(state: State<'_, AppState>) -> Result<SettingsDto, String> {
    {
        let mut cm = state.config_mgr.lock().await;
        cm.config.custom_instances_dir = None;
        let new_inst_dir = cm.config.instances_dir();
        let _ = std::fs::create_dir_all(&new_inst_dir);
        cm.save().map_err(|e| e.to_string())?;

        let mut im = state.instance_mgr.lock().await;
        im.instances_dir = new_inst_dir;
        let _ = im.load_all();
    }

    get_settings(state).await
}

#[tauri::command]
pub async fn save_settings(settings: SettingsDto, state: State<'_, AppState>) -> Result<(), String> {
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
    if settings.enable_discord_rpc {
        if state.supervisor.is_active() {
            state.discord_rpc.set_playing("Minecraft");
        } else {
            state.discord_rpc.set_in_launcher();
        }
    } else {
        state.discord_rpc.clear();
    }
    cm.config.default_java_path = settings.custom_java_path.map(PathBuf::from);

    let trimmed_dir = settings.data_dir.trim();
    if !trimmed_dir.is_empty() {
        let new_path = PathBuf::from(trimmed_dir);
        if new_path != cm.config.data_dir {
            let _ = std::fs::create_dir_all(&new_path);
            cm.config.data_dir = new_path;
        }
    }

    let trimmed_inst = settings.instances_dir.trim();
    if !trimmed_inst.is_empty() {
        let inst_path = PathBuf::from(trimmed_inst);
        if inst_path == cm.config.data_dir.join("instances") {
            cm.config.custom_instances_dir = None;
        } else {
            cm.config.custom_instances_dir = Some(inst_path);
        }
    } else {
        cm.config.custom_instances_dir = None;
    }

    let new_inst_dir = cm.config.instances_dir();
    let _ = std::fs::create_dir_all(&new_inst_dir);
    {
        let mut im = state.instance_mgr.lock().await;
        if im.instances_dir != new_inst_dir {
            im.instances_dir = new_inst_dir;
            let _ = im.load_all();
        }
    }

    cm.save().map_err(|e| e.to_string())?;
    Ok(())
}
