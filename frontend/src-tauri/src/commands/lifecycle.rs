use crate::state::AppState;
use dv_launcher::gpu_detector::GpuDetector;
use dv_launcher::instances::sync_instance_mods;
use dv_launcher::java_detector::JavaDetector;
use dv_launcher::loaders::{ModLoaderClient, ModLoaderLaunchInfo};
use dv_launcher::minecraft::{
    DownloadProgress, InstallationPipeline, LaunchArgumentBuilder, LaunchContext,
};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager, State};
use tracing::info;

#[tauri::command]
pub async fn launch_game(
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
pub async fn stop_game(state: State<'_, AppState>) -> Result<(), String> {
    state.supervisor.terminate().await;
    Ok(())
}

#[tauri::command]
pub async fn get_game_status(state: State<'_, AppState>) -> Result<String, String> {
    if state.supervisor.is_active() {
        Ok("running".to_string())
    } else if state.is_launching.load(Ordering::SeqCst) {
        Ok("launching".to_string())
    } else {
        Ok("idle".to_string())
    }
}

#[tauri::command]
pub async fn get_latest_logs(
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
pub async fn open_logs_folder(
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
pub async fn export_logs(content: String) -> Result<Option<String>, String> {
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
