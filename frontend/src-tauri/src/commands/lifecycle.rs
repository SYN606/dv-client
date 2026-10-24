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
        let req_name = match req_ver {
            8 => "Java 8",
            16 | 17 => "Java 17",
            _ => "Java 21",
        };
        return Err(format!(
            "Required Java version ({}) not found for Minecraft {}. Please install it via Settings.",
            req_name, inst.minecraft_version
        ));
    };

    // Strict validation: Verify the chosen Java binary meets Minecraft's minimum major version requirement
    if let Some(inspected) = JavaDetector::inspect_executable(&java_path) {
        if inspected.major_version < req_ver {
            state.is_launching.store(false, Ordering::SeqCst);
            return Err(format!(
                "Incompatible Java version: Selected runtime '{}' is Java {}, but Minecraft {} strictly requires Java {} (or newer). Please select or install Java {} in Settings.",
                java_path.display(),
                inspected.major_version,
                inst.minecraft_version,
                req_ver,
                req_ver
            ));
        }
    }

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
    let app_handle_bg = app_handle.clone();
    let bg_game_dir = game_dir.clone();
    let on_launch = config_clone.on_launch_action.clone();
    let is_launching = state.is_launching.clone();

    // Spawn supervisor execution in background task
    tokio::spawn(async move {
        let launch_res = sup.launch_with_env(command_args, bg_game_dir, gpu_env).await;
        is_launching.store(false, Ordering::SeqCst);

        match launch_res {
            Ok(0) => {
                let _ = app_handle_bg.emit("game-state-change", "finished");
                let _ = app_handle_bg.emit("launch-status", "Game process exited normally.");
            }
            Ok(code) => {
                let analysis = sup.get_last_crash_analysis().await;
                let _ = app_handle_bg.emit("game-state-change", "crashed");
                if let Some(ref a) = analysis {
                    let _ = app_handle_bg.emit("game-crashed", serde_json::to_value(a).unwrap_or_default());
                    let _ = app_handle_bg.emit("launch-status", format!("Crash: {}", a.headline));
                } else {
                    let _ = app_handle_bg.emit("launch-status", format!("Process crashed with code {}", code));
                }

                // If window was hidden on launch, restore it on crash
                if on_launch == "hide" {
                    if let Some(win) = app_handle_bg.get_webview_window("main") {
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                }
            }
            Err(e) => {
                let _ = app_handle_bg.emit("game-state-change", "crashed");
                let _ = app_handle_bg.emit("launch-status", format!("Launch failed: {}", e));
            }
        }
    });

    // Startup health check: wait briefly to detect immediate early crashes
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;

    // Check if the process crashed immediately during startup
    if !state.supervisor.is_active() {
        state.is_launching.store(false, Ordering::SeqCst);
        if let Some(analysis) = state.supervisor.get_last_crash_analysis().await {
            let _ = app_handle.emit("game-state-change", "crashed");
            let _ = app_handle.emit("game-crashed", serde_json::to_value(&analysis).unwrap_or_default());
            return Err(format!(
                "[{}]: {}. Suggested Fix: {}",
                analysis.category, analysis.headline, analysis.recommended_action
            ));
        } else {
            return Err("Minecraft process terminated immediately upon spawning. Check Live Console for details.".to_string());
        }
    }

    state.is_launching.store(false, Ordering::SeqCst);
    let _ = app_handle.emit("launch-status", "Minecraft is currently running");
    let _ = app_handle.emit("game-state-change", "running");

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

#[tauri::command]
pub async fn stop_game(state: State<'_, AppState>) -> Result<(), String> {
    state.supervisor.terminate().await;
    Ok(())
}

#[tauri::command]
pub async fn get_last_crash_analysis(
    state: State<'_, AppState>,
) -> Result<Option<dv_launcher::minecraft::CrashAnalysis>, String> {
    Ok(state.supervisor.get_last_crash_analysis().await)
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

pub const DISCORD_CRASH_WEBHOOK_URL: &str =
    "https://discord.com/api/webhooks/1563426682088722504/G5FHous4s_1Hfvge2Wjf9zaqSbHvdqxm73H0BZBfRowmPbh-qBQn_WlZ1HXTCjNd0eaY";

#[tauri::command]
pub async fn report_crash(
    logs: String,
    crash_snippet: Option<String>,
    instance_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    // 1. Resolve Account Info
    let (username, account_type) = {
        let auth = state.auth_client.lock().await;
        let cm = state.config_mgr.lock().await;
        let active_id = cm.config.active_account_id.as_deref();
        if let Some(acc) = active_id.and_then(|id| auth.accounts.get(id)) {
            (acc.username.clone(), acc.account_type.clone())
        } else if let Some(first) = auth.accounts.values().next() {
            (first.username.clone(), first.account_type.clone())
        } else {
            ("Offline Player".to_string(), "offline".to_string())
        }
    };

    // 2. Resolve Instance & Profile Info
    let (inst_name, mc_ver, mod_loader, ram_str, java_str, disk_logs) = {
        let mgr = state.instance_mgr.lock().await;
        let cm = state.config_mgr.lock().await;
        let target_id = instance_id.or_else(|| cm.config.active_instance_id.clone());
        let target = match target_id {
            Some(ref id) => mgr.instances.get(id).cloned(),
            None => None,
        };

        if let Some(inst) = target {
            let loader_str = if let Some(ver) = &inst.loader_version {
                format!("{} {}", inst.mod_loader, ver)
            } else {
                inst.mod_loader.clone()
            };
            let ram = format!("{} MB - {} MB", inst.min_memory_mb, inst.max_memory_mb);
            let java = inst
                .custom_java_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "Default Managed Java".to_string());

            let mut disk_content = String::new();
            let log_file = inst
                .game_dir(cm.config.instances_dir())
                .join("logs")
                .join("latest.log");
            if log_file.exists() {
                if let Ok(c) = std::fs::read_to_string(&log_file) {
                    disk_content = c;
                }
            }

            (
                inst.name.clone(),
                inst.minecraft_version.clone(),
                loader_str,
                ram,
                java,
                disk_content,
            )
        } else {
            (
                "Default Profile".to_string(),
                "Unknown".to_string(),
                "vanilla".to_string(),
                "2048 MB - 4096 MB".to_string(),
                "Default Java".to_string(),
                String::new(),
            )
        }
    };

    // 3. Resolve GPU Info
    let gpu_name = {
        let cm = state.config_mgr.lock().await;
        let gpus = GpuDetector::detect_gpus();
        if let Some(sel) = cm.config.selected_gpu.as_deref() {
            gpus.iter()
                .find(|g| g.id == sel)
                .map(|g| g.name.clone())
                .unwrap_or_else(|| sel.to_string())
        } else {
            gpus.iter()
                .find(|g| g.is_discrete)
                .or_else(|| gpus.first())
                .map(|g| g.name.clone())
                .unwrap_or_else(|| "Standard GPU".to_string())
        }
    };

    // 4. Resolve Logs & Snippet
    let full_logs = if !logs.trim().is_empty() {
        logs
    } else if !disk_logs.trim().is_empty() {
        disk_logs
    } else {
        "No live or disk logs available at report time.".to_string()
    };

    let snippet_text = if let Some(snip) = crash_snippet.filter(|s| !s.trim().is_empty()) {
        snip
    } else {
        let lines: Vec<&str> = full_logs.lines().collect();
        let error_lines: Vec<&str> = lines
            .iter()
            .copied()
            .filter(|l| {
                l.contains("ERROR")
                    || l.contains("FATAL")
                    || l.contains("Exception")
                    || l.contains("CrashReport")
                    || l.contains("at ")
            })
            .take(30)
            .collect();

        if !error_lines.is_empty() {
            error_lines.join("\n")
        } else {
            lines
                .iter()
                .rev()
                .take(25)
                .rev()
                .copied()
                .collect::<Vec<_>>()
                .join("\n")
        }
    };

    let truncated_snippet = if snippet_text.len() > 1800 {
        format!(
            "{}...\n[Truncated - see attached crash.log for full output]",
            &snippet_text[..1800]
        )
    } else if snippet_text.trim().is_empty() {
        "No crash stack trace snippet available.".to_string()
    } else {
        snippet_text
    };

    // 5. Build Discord Embed
    let os_info = format!("{} ({})", std::env::consts::OS, std::env::consts::ARCH);
    let launcher_ver = format!("Digital Vigital v{}", dv_launcher::branding::APP_VERSION);
    let now_iso = chrono::Utc::now().to_rfc3339();

    let embed = serde_json::json!({
        "title": "🚨 Minecraft Crash & Diagnostics Report",
        "description": format!("```text\n{}\n```", truncated_snippet),
        "color": 14753096, // 0xE11D48 (Rose 600)
        "fields": [
            {
                "name": "👤 Player / User",
                "value": format!("**{}** (`{}`)", username, account_type),
                "inline": true
            },
            {
                "name": "💻 Operating System",
                "value": os_info,
                "inline": true
            },
            {
                "name": "🎮 Launcher Version",
                "value": launcher_ver,
                "inline": true
            },
            {
                "name": "📦 Profile & Mod Loader",
                "value": format!("**{}** (MC {} • {})", inst_name, mc_ver, mod_loader),
                "inline": true
            },
            {
                "name": "🧠 Memory Allocated",
                "value": ram_str,
                "inline": true
            },
            {
                "name": "☕ Java Runtime",
                "value": if java_str.len() > 40 {
                    format!("...{}", &java_str[java_str.len() - 37..])
                } else {
                    java_str
                },
                "inline": true
            },
            {
                "name": "🖥️ Graphics / GPU",
                "value": gpu_name,
                "inline": false
            }
        ],
        "footer": {
            "text": "DV Launcher • Automated Diagnostics & Crash Dispatcher"
        },
        "timestamp": now_iso
    });

    let payload_json = serde_json::json!({
        "embeds": [embed]
    });

    // 6. Build Multipart and Send via reqwest
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let form_file = reqwest::multipart::Part::bytes(full_logs.into_bytes())
        .file_name("crash.log")
        .mime_str("text/plain")
        .map_err(|e| format!("Failed to encode crash log attachment: {}", e))?;

    let form = reqwest::multipart::Form::new()
        .text("payload_json", serde_json::to_string(&payload_json).unwrap_or_default())
        .part("files[0]", form_file);

    let resp = client
        .post(DISCORD_CRASH_WEBHOOK_URL)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Failed to send crash report to webhook: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Webhook error (HTTP {}): {}", status, body));
    }

    info!(
        "Crash report successfully dispatched to diagnostics webhook for user: {}",
        username
    );
    Ok(true)
}
