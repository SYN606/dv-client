#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Digital Vigital (DV) — High-Performance Native Desktop Minecraft Launcher
//!
//! Powered by an independent Rust application engine and native Slint desktop UI.
//! Eliminates external Python runtimes, insecure XOR credentials, and unauthorized telemetry.

use dv_launcher::accounts::MicrosoftAuthClient;
use dv_launcher::branding;
use dv_launcher::config::{pick_directory, ConfigManager};
use dv_launcher::discord::DiscordRpcHandle;
use dv_launcher::gpu_detector::GpuDetector;
use dv_launcher::instances::InstanceManager;
use dv_launcher::java_detector::JavaDetector;
use dv_launcher::minecraft::{
    DownloadProgress, InstallationPipeline, LaunchArgumentBuilder, LaunchContext,
    ProcessState, ProcessSupervisor,
};
use dv_launcher::loaders::ModrinthClient;
use dv_launcher::ui::{
    AccountItem, DownloadItem, GpuItem, InstanceItem, JavaItem, MainWindow, ModItem, OnlineModItem,
};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel, Weak};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting {} v{}", branding::APP_NAME, branding::APP_VERSION);

    // Register Wayland app_id and verify desktop integration (XDG .desktop & icon)
    let _ = slint::set_xdg_app_id(dv_launcher::desktop::DESKTOP_APP_ID);
    dv_launcher::desktop::ensure_desktop_integration();

    // Initialize core subsystems
    let config_mgr = Arc::new(Mutex::new(ConfigManager::new::<&str>(None)?));
    let (instances_dir, base_data_dir) = {
        let mgr = config_mgr.lock().await;
        (mgr.config.instances_dir(), mgr.config.base_path().to_path_buf())
    };

    let instance_mgr = Arc::new(Mutex::new(InstanceManager::new(&instances_dir)?));
    let auth_client = Arc::new(Mutex::new(MicrosoftAuthClient::new(&base_data_dir)));
    let supervisor = Arc::new(ProcessSupervisor::new());
    let is_launching = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let discord_rpc = Arc::new(DiscordRpcHandle::new(None));

    let enable_discord = {
        let mgr = config_mgr.lock().await;
        mgr.config.privacy.enable_discord_rpc
    };
    if enable_discord {
        discord_rpc.set_in_launcher();
    }

    // Auto-detect Java runtimes using java_detector
    let detected_javas = JavaDetector::scan_system();
    let best_java = detected_javas
        .iter()
        .find(|j| j.major_version >= 21)
        .or_else(|| detected_javas.first())
        .cloned();

    // Auto-detect system GPUs using gpu_detector
    let detected_gpus = GpuDetector::detect_gpus();

    let main_window = MainWindow::new()?;

    // Populate UI initial state
    let java_display = if let Some(ref j) = best_java {
        j.display_name.clone()
    } else {
        "Java Not Found on PATH (Configure in Settings)".to_string()
    };
    main_window.set_java_runtime_display(SharedString::from(&java_display));

    // Populate detected Javas list in UI
    let java_items: Vec<JavaItem> = detected_javas
        .iter()
        .map(|j| JavaItem {
            path: SharedString::from(j.path.to_string_lossy().to_string()),
            vendor: SharedString::from(&j.vendor),
            version: SharedString::from(&j.full_version),
            display_name: SharedString::from(&j.display_name),
        })
        .collect();
    main_window.set_java_list(ModelRc::new(VecModel::from(java_items)));

    // Populate detected GPUs list in UI
    let gpu_items: Vec<GpuItem> = detected_gpus
        .iter()
        .map(|g| GpuItem {
            id: SharedString::from(&g.id),
            name: SharedString::from(&g.name),
            is_discrete: g.is_discrete,
        })
        .collect();
    main_window.set_gpu_list(ModelRc::new(VecModel::from(gpu_items)));

    // Load initial settings values into UI
    {
        let mgr = config_mgr.lock().await;
        main_window.set_min_memory_mb(mgr.config.default_min_memory_mb as f32);
        main_window.set_max_memory_mb(mgr.config.default_max_memory_mb as f32);
        main_window.set_window_width(mgr.config.default_window_width as i32);
        main_window.set_window_height(mgr.config.default_window_height as i32);
        main_window.set_enable_discord_rpc(mgr.config.privacy.enable_discord_rpc);
        main_window.set_launcher_root_dir_input(SharedString::from(
            mgr.config.data_dir.to_string_lossy().to_string(),
        ));
        main_window.set_profiles_dir_input(SharedString::from(
            mgr.config.instances_dir().to_string_lossy().to_string(),
        ));

        let current_gpu_id = mgr
            .config
            .selected_gpu
            .clone()
            .unwrap_or_else(|| "auto".to_string());
        let current_gpu_name = detected_gpus
            .iter()
            .find(|g| g.id == current_gpu_id)
            .map(|g| g.name.clone())
            .unwrap_or_else(|| "Automatic (System Default)".to_string());
        main_window.set_selected_gpu_id(SharedString::from(&current_gpu_id));
        main_window.set_selected_gpu_display(SharedString::from(&current_gpu_name));

        if let Some(ref custom_j) = mgr.config.default_java_path {
            main_window.set_custom_java_path(SharedString::from(
                custom_j.to_string_lossy().to_string(),
            ));
        }

        let preset = mgr.config.performance_preset.clone();
        let preset_disp = match preset.as_str() {
            "aikar" => "Aikar's Flags (High FPS & Zero Stutter)",
            "shenandoah" => "Shenandoah GC (Ultra-Low Latency)",
            "zgc" => "Generational ZGC (Java 21+ Modern)",
            _ => "Vanilla / Default JVM",
        };
        main_window.set_performance_preset(SharedString::from(&preset));
        main_window.set_performance_preset_display(SharedString::from(preset_disp));

        main_window.set_custom_jvm_args_input(SharedString::from(&mgr.config.custom_jvm_args));
        main_window.set_on_launch_action(SharedString::from(&mgr.config.on_launch_action));
        let on_launch_disp = match mgr.config.on_launch_action.as_str() {
            "close" => "Close Launcher Completely",
            "keep_open" => "Keep Launcher Open",
            _ => "Hide Launcher (Minimizes RAM)",
        };
        main_window.set_on_launch_action_display(SharedString::from(on_launch_disp));
    }

    // Refresh instances list in UI
    dispatch_refresh_instances(main_window.as_weak(), &instance_mgr, &config_mgr).await;
    // Refresh accounts list in UI
    dispatch_refresh_accounts(main_window.as_weak(), &auth_client, &config_mgr).await;
    // Refresh mods list in UI
    dispatch_refresh_mods(main_window.as_weak(), &instance_mgr, &config_mgr).await;

    // Dynamically fetch full official version manifest from Mojang in background
    {
        let ui_w = main_window.as_weak();
        let cfg_clone = {
            let cm = config_mgr.lock().await;
            cm.config.clone()
        };
        tokio::spawn(async move {
            let p_client = dv_launcher::minecraft::PistonMetaClient::new(
                cfg_clone.shared_versions_dir().join("meta_cache"),
            );
            if let Ok(manifest) = p_client.get_version_manifest(false).await {
                let releases: Vec<SharedString> = manifest
                    .versions
                    .iter()
                    .filter(|v| v.version_type == "release")
                    .map(|v| SharedString::from(&v.id))
                    .collect();

                if !releases.is_empty() {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_w.upgrade() {
                            ui.set_version_list(ModelRc::new(VecModel::from(releases)));
                        }
                    });
                }
            }
        });
    }

    // Connect supervisor output stream to Slint UI Console
    {
        let mut out_rx = supervisor.output_tx.subscribe();
        let ui_weak = main_window.as_weak();
        tokio::spawn(async move {
            while let Ok(line) = out_rx.recv().await {
                let line_clone = line.clone();
                let ui_w = ui_weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_w.upgrade() {
                        let mut current = ui.get_console_output().to_string();
                        current.push_str(&line_clone);
                        if current.len() > 65536 {
                            current = current.split_off(current.len() - 32768);
                        }
                        ui.set_console_output(SharedString::from(current));
                    }
                });
            }
        });
    }

    // Connect supervisor process state stream to Slint UI status indicators & Discord RPC
    {
        let mut state_rx = supervisor.state_tx.subscribe();
        let ui_weak = main_window.as_weak();
        let rpc = discord_rpc.clone();
        let inst_a = instance_mgr.clone();
        let cfg_a = config_mgr.clone();
        let is_launch_c = is_launching.clone();

        tokio::spawn(async move {
            while let Ok(state) = state_rx.recv().await {
                let ui_w = ui_weak.clone();
                let rpc_c = rpc.clone();
                let inst_c = inst_a.clone();
                let cfg_c = cfg_a.clone();
                let is_launch_inner = is_launch_c.clone();

                match &state {
                    ProcessState::Running => {
                        let im = inst_c.lock().await;
                        let cm = cfg_c.lock().await;
                        let ver = im
                            .get_active_instance(&cm.config)
                            .map(|i| i.minecraft_version.clone())
                            .unwrap_or_else(|| "1.21.4".to_string());
                        if cm.config.privacy.enable_discord_rpc {
                            rpc_c.set_playing(&ver);
                        }
                        let on_launch = cm.config.on_launch_action.clone();
                        drop(cm);
                        drop(im);

                        if on_launch == "close" {
                            info!("Exiting launcher completely per Settings (On-Launch: Close)");
                            std::process::exit(0);
                        } else if on_launch == "hide" {
                            info!("Minimizing background memory overhead: hiding launcher window during gameplay");
                            let ui_hide = ui_w.clone();
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_hide.upgrade() {
                                    let _ = ui.window().hide();
                                }
                            });
                        }
                    }
                    ProcessState::Exited(_) | ProcessState::Crashed(_) | ProcessState::Stopped => {
                        let cm = cfg_c.lock().await;
                        if cm.config.privacy.enable_discord_rpc {
                            rpc_c.set_in_launcher();
                        } else {
                            rpc_c.clear();
                        }
                        drop(cm);

                        let ui_show = ui_w.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_show.upgrade() {
                                let _ = ui.window().set_minimized(false);
                                let _ = ui.window().show();
                            }
                        });
                    }
                    _ => {}
                }

                is_launch_inner.store(false, std::sync::atomic::Ordering::SeqCst);

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_w.upgrade() {
                        ui.set_is_launching(false);
                        ui.set_is_downloading(false);

                        match state {
                            ProcessState::Starting => {
                                ui.set_is_running(true);
                                ui.set_game_status(SharedString::from("Starting Minecraft..."));
                            }
                            ProcessState::Running => {
                                ui.set_is_running(true);
                                ui.set_game_status(SharedString::from("Minecraft Running"));
                            }
                            ProcessState::Exited(code) => {
                                ui.set_is_running(false);
                                ui.set_game_status(SharedString::from(format!("Exited (code {})", code)));
                            }
                            ProcessState::Crashed(code) => {
                                ui.set_is_running(false);
                                ui.set_game_status(SharedString::from(format!("Crashed (code {})", code)));
                            }
                            ProcessState::Stopped => {
                                ui.set_is_running(false);
                                ui.set_game_status(SharedString::from("Stopped"));
                            }
                        }
                    }
                });
            }
        });
    }

    // Callback: PLAY CLICKED
    {
        let cfg_arc = config_mgr.clone();
        let inst_arc = instance_mgr.clone();
        let auth_arc = auth_client.clone();
        let sup_arc = supervisor.clone();
        let ui_weak = main_window.as_weak();
        let java_candidate = best_java.clone();
        let is_launch_arc = is_launching.clone();

        main_window.on_play_clicked(move || {
            let cfg = cfg_arc.clone();
            let inst_mgr = inst_arc.clone();
            let auth = auth_arc.clone();
            let sup = sup_arc.clone();
            let ui_w = ui_weak.clone();
            let java_opt = java_candidate.clone();
            let is_launch = is_launch_arc.clone();

            // Strict single-instance guard: only one instance can run or launch at a time
            if sup.is_active() || is_launch.swap(true, std::sync::atomic::Ordering::SeqCst) {
                warn!("Launch request ignored: An instance is already downloading, launching, or running.");
                let ui_err = ui_w.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_err.upgrade() {
                        ui.set_game_status(SharedString::from("Only one instance can run at a time"));
                    }
                });
                return;
            }

            // Immediately signal download/launch preparation on the UI
            let ui_start = ui_w.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_start.upgrade() {
                    ui.set_is_launching(true);
                    ui.set_is_downloading(true);
                    ui.set_download_progress(0.0);
                    ui.set_game_status(SharedString::from("Checking & Downloading assets..."));
                }
            });

            tokio::spawn(async move {
                let (active_inst, config_clone) = {
                    let im = inst_mgr.lock().await;
                    let cm = cfg.lock().await;
                    let inst = im.get_active_instance(&cm.config).cloned();
                    (inst, cm.config.clone())
                };

                let inst = match active_inst {
                    Some(i) => i,
                    None => {
                        error!("No active instance found to launch");
                        is_launch.store(false, std::sync::atomic::Ordering::SeqCst);
                        let ui_err = ui_w.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_err.upgrade() {
                                ui.set_is_launching(false);
                                ui.set_is_downloading(false);
                                ui.set_game_status(SharedString::from("No Active Profile Selected"));
                            }
                        });
                        return;
                    }
                };

                let account = {
                    let a = auth.lock().await;
                    a.get_active_account(config_clone.active_account_id.as_deref())
                };

                let account = match account {
                    Some(acc) => acc,
                    None => {
                        error!("Launch aborted: No active account configured");
                        is_launch.store(false, std::sync::atomic::Ordering::SeqCst);
                        let ui_err = ui_w.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_err.upgrade() {
                                ui.set_is_launching(false);
                                ui.set_is_downloading(false);
                                ui.set_game_status(SharedString::from("Account Required: Configure player name in Accounts"));
                                ui.set_active_tab(4);
                            }
                        });
                        return;
                    }
                };

                let java_path = inst
                    .custom_java_path
                    .clone()
                    .or_else(|| config_clone.default_java_path.clone())
                    .or_else(|| java_opt.map(|j| j.path))
                    .unwrap_or_else(|| PathBuf::from("java"));

                let pipeline = InstallationPipeline::new(&config_clone);
                let inst_dir = inst.instance_dir(config_clone.instances_dir());
                let game_dir = inst.game_dir(config_clone.instances_dir());

                // Ensure mods directory is present and synced between instance and game directory
                dv_launcher::instances::sync_instance_mods(&inst_dir, &game_dir);

                let ui_for_prog = ui_w.clone();
                let pkg_res = pipeline
                    .install_version(
                        &inst.minecraft_version,
                        &inst_dir,
                        &config_clone,
                        Some(move |prog: DownloadProgress| {
                            let ui_cb = ui_for_prog.clone();
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_cb.upgrade() {
                                    handle_download_progress(&ui, &prog);
                                    ui.set_is_downloading(true);
                                    ui.set_is_launching(true);
                                }
                            });
                        }),
                        Some(|status: &str| {
                            info!("[Install Pipeline] {}", status);
                        }),
                    )
                    .await;

                let version_pkg = match pkg_res {
                    Ok(pkg) => pkg,
                    Err(e) => {
                        error!("Installation failed before launch: {}", e);
                        is_launch.store(false, std::sync::atomic::Ordering::SeqCst);
                        let ui_err = ui_w.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_err.upgrade() {
                                ui.set_is_launching(false);
                                ui.set_is_downloading(false);
                                ui.set_game_status(SharedString::from("Install Error"));
                            }
                        });
                        return;
                    }
                };

                // Transition to launching state (download finished)
                let ui_launching = ui_w.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_launching.upgrade() {
                        ui.set_is_downloading(false);
                        ui.set_is_launching(true);
                        ui.set_game_status(SharedString::from("Launching Minecraft JVM..."));
                    }
                });

                // Resolve and install mod loader libraries if active (Fabric, Forge, OptiFine)
                let loader_client = dv_launcher::loaders::ModLoaderClient::new();
                let loader_info = match loader_client
                    .resolve_and_download_loader(
                        &inst.mod_loader,
                        &inst.minecraft_version,
                        inst.loader_version.as_deref(),
                        &config_clone.shared_libraries_dir(),
                    )
                    .await
                {
                    Ok(info) => info,
                    Err(e) => {
                        warn!("Failed to resolve mod loader '{}': {}", inst.mod_loader, e);
                        dv_launcher::loaders::ModLoaderLaunchInfo::default()
                    }
                };

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

                // Ensure universal player skin & cape resource pack is synchronized and enabled in game options
                let _ = dv_launcher::skins::ensure_in_game_skin_pack(&game_dir, &account.username);

                let launch_ctx = LaunchContext {
                    version_package: version_pkg,
                    instance_dir: inst_dir.clone(),
                    game_dir: game_dir.clone(),
                    assets_dir: config_clone.shared_assets_dir(),
                    libraries_dir: config_clone.shared_libraries_dir(),
                    client_jar_path: client_jar,
                    java_executable: java_path,
                    player_name: account.username,
                    player_uuid: account.id,
                    access_token: account.access_token.unwrap_or_else(|| "0".to_string()),
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
                let gpu_id = config_clone.selected_gpu.unwrap_or_else(|| "auto".to_string());
                let gpu_env = GpuDetector::get_env_vars_for_gpu(&gpu_id);
                if let Err(e) = sup.launch_with_env(command_args, game_dir, gpu_env).await {
                    error!("Launch failed: {}", e);
                    is_launch.store(false, std::sync::atomic::Ordering::SeqCst);
                    let ui_err = ui_w.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_err.upgrade() {
                            ui.set_is_launching(false);
                            ui.set_is_downloading(false);
                            ui.set_game_status(SharedString::from(format!("Launch failed: {}", e)));
                        }
                    });
                }
            });
        });
    }

    // Callback: TERMINATE CLICKED
    {
        let sup_arc = supervisor.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_terminate_clicked(move || {
            let sup = sup_arc.clone();
            let ui_t = ui_weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_t.upgrade() {
                    ui.set_game_status(SharedString::from("Stopping game..."));
                }
            });
            tokio::spawn(async move {
                sup.terminate().await;
            });
        });
    }

    // Callback: VERIFY FILES CLICKED
    {
        let cfg_arc = config_mgr.clone();
        let inst_arc = instance_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_install_clicked(move || {
            let cfg = cfg_arc.clone();
            let inst_mgr = inst_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                let (active_inst, config_clone) = {
                    let im = inst_mgr.lock().await;
                    let cm = cfg.lock().await;
                    let inst = im.get_active_instance(&cm.config).cloned();
                    (inst, cm.config.clone())
                };

                if let Some(inst) = active_inst {
                    let pipeline = InstallationPipeline::new(&config_clone);
                    let ui_for_prog = ui_w.clone();

                    let ui_start = ui_w.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_start.upgrade() {
                            ui.set_is_downloading(true);
                            ui.set_download_status_text(SharedString::from("Verifying client JAR & libraries..."));
                            ui.set_game_status(SharedString::from("Verifying integrity..."));
                        }
                    });

                    let res = pipeline
                        .verify_version_files(
                            &inst.minecraft_version,
                            &config_clone,
                            Some(move |prog: DownloadProgress| {
                                let ui_cb = ui_for_prog.clone();
                                let _ = slint::invoke_from_event_loop(move || {
                                    if let Some(ui) = ui_cb.upgrade() {
                                        handle_download_progress(&ui, &prog);
                                    }
                                });
                            }),
                            Some(|msg: &str| {
                                info!("[File Verification] {}", msg);
                            }),
                        )
                        .await;

                    let ui_end = ui_w.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_end.upgrade() {
                            ui.set_is_downloading(false);
                            match res {
                                Ok((verified, repaired)) => {
                                    ui.set_game_status(SharedString::from(format!(
                                        "Verified {} files ({} repaired)",
                                        verified, repaired
                                    )));
                                }
                                Err(e) => {
                                    ui.set_game_status(SharedString::from(format!("Verification failed: {}", e)));
                                }
                            }
                        }
                    });
                }
            });
        });
    }

    // Callback: REPAIR ASSETS CLICKED
    {
        let cfg_arc = config_mgr.clone();
        let inst_arc = instance_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_repair_clicked(move || {
            let cfg = cfg_arc.clone();
            let inst_mgr = inst_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                let (active_inst, config_clone) = {
                    let im = inst_mgr.lock().await;
                    let cm = cfg.lock().await;
                    let inst = im.get_active_instance(&cm.config).cloned();
                    (inst, cm.config.clone())
                };

                if let Some(inst) = active_inst {
                    let pipeline = InstallationPipeline::new(&config_clone);
                    let ui_for_prog = ui_w.clone();

                    let ui_start = ui_w.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_start.upgrade() {
                            ui.set_is_downloading(true);
                            ui.set_download_status_text(SharedString::from("Checking & repairing sound/texture assets..."));
                            ui.set_game_status(SharedString::from("Repairing assets..."));
                        }
                    });

                    let res = pipeline
                        .repair_version_assets(
                            &inst.minecraft_version,
                            &config_clone,
                            Some(move |prog: DownloadProgress| {
                                let ui_cb = ui_for_prog.clone();
                                let _ = slint::invoke_from_event_loop(move || {
                                    if let Some(ui) = ui_cb.upgrade() {
                                        handle_download_progress(&ui, &prog);
                                    }
                                });
                            }),
                            Some(|msg: &str| {
                                info!("[Asset Repair] {}", msg);
                            }),
                        )
                        .await;

                    let ui_end = ui_w.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_end.upgrade() {
                            ui.set_is_downloading(false);
                            match res {
                                Ok((verified, repaired)) => {
                                    ui.set_game_status(SharedString::from(format!(
                                        "Assets OK: {} verified ({} repaired)",
                                        verified, repaired
                                    )));
                                }
                                Err(e) => {
                                    ui.set_game_status(SharedString::from(format!("Asset repair failed: {}", e)));
                                }
                            }
                        }
                    });
                }
            });
        });
    }

    // Callback: OPEN ACTIVE PROFILE FOLDER
    {
        let inst_arc = instance_mgr.clone();
        let cfg_arc = config_mgr.clone();

        main_window.on_open_profile_folder_clicked(move || {
            let inst_mgr = inst_arc.clone();
            let cfg = cfg_arc.clone();
            tokio::spawn(async move {
                let dir_opt = {
                    let im = inst_mgr.lock().await;
                    let cm = cfg.lock().await;
                    im.get_active_instance(&cm.config).map(|i| i.game_dir(cm.config.instances_dir()))
                };
                if let Some(dir) = dir_opt {
                    let _ = std::fs::create_dir_all(&dir);
                    let _ = dv_launcher::instances::open_mods_folder(&dir);
                }
            });
        });
    }

    // Callback: OPEN INSTANCE FOLDER BY ID
    {
        let inst_arc = instance_mgr.clone();
        let cfg_arc = config_mgr.clone();

        main_window.on_open_instance_folder_clicked(move |id: SharedString| {
            let inst_mgr = inst_arc.clone();
            let cfg = cfg_arc.clone();
            let id_str = id.to_string();
            tokio::spawn(async move {
                let dir_opt = {
                    let im = inst_mgr.lock().await;
                    let cm = cfg.lock().await;
                    im.get_instance(&id_str).map(|i| i.game_dir(cm.config.instances_dir()))
                };
                if let Some(dir) = dir_opt {
                    let _ = std::fs::create_dir_all(&dir);
                    let _ = dv_launcher::instances::open_mods_folder(&dir);
                }
            });
        });
    }

    // Callback: DUPLICATE INSTANCE
    {
        let inst_arc = instance_mgr.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_duplicate_instance_clicked(move |id: SharedString| {
            let inst_mgr = inst_arc.clone();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();
            let id_str = id.to_string();

            tokio::spawn(async move {
                {
                    let mut im = inst_mgr.lock().await;
                    if let Some(src) = im.get_instance(&id_str) {
                        let dup_name = format!("{} (Copy)", src.name);
                        let _ = im.duplicate_instance(&id_str, &dup_name);
                    }
                }
                dispatch_refresh_instances(ui_w.clone(), &inst_mgr, &cfg).await;
                dispatch_refresh_mods(ui_w, &inst_mgr, &cfg).await;
            });
        });
    }

    // Callback: CREATE INSTANCE
    {
        let inst_arc = instance_mgr.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_create_instance_clicked(move || {
            let (name, ver, loader) = if let Some(ui) = ui_weak.upgrade() {
                (
                    ui.get_new_instance_name().to_string(),
                    ui.get_new_instance_version().to_string(),
                    ui.get_new_instance_loader().to_string(),
                )
            } else {
                return;
            };

            let loader = if loader.trim().is_empty() {
                "fabric".to_string()
            } else {
                loader
            };

            let inst_mgr = inst_arc.clone();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                {
                    let mut im = inst_mgr.lock().await;
                    let _ = im.create_instance(&name, &ver, &loader, None);
                }
                dispatch_refresh_instances(ui_w.clone(), &inst_mgr, &cfg).await;
                dispatch_refresh_mods(ui_w, &inst_mgr, &cfg).await;
            });
        });
    }

    // Callback: DELETE INSTANCE
    {
        let inst_arc = instance_mgr.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_delete_instance_clicked(move |id: SharedString| {
            let inst_mgr = inst_arc.clone();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();
            let id_str = id.to_string();

            tokio::spawn(async move {
                {
                    let mut im = inst_mgr.lock().await;
                    let _ = im.delete_instance(&id_str);
                }
                dispatch_refresh_instances(ui_w.clone(), &inst_mgr, &cfg).await;
                dispatch_refresh_mods(ui_w, &inst_mgr, &cfg).await;
            });
        });
    }

    // Callback: SELECT INSTANCE
    {
        let inst_arc = instance_mgr.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_select_instance_clicked(move |id: SharedString| {
            let inst_mgr = inst_arc.clone();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();
            let id_str = id.to_string();

            tokio::spawn(async move {
                {
                    let mut cm = cfg.lock().await;
                    cm.config.active_instance_id = Some(id_str);
                    let _ = cm.save();
                }
                dispatch_refresh_instances(ui_w.clone(), &inst_mgr, &cfg).await;
                dispatch_refresh_mods(ui_w, &inst_mgr, &cfg).await;
            });
        });
    }

    // Callback: QUICK VERSION SWITCHER (Requirement 1)
    {
        let inst_arc = instance_mgr.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_select_quick_version(move |ver: SharedString| {
            let ver_str = ver.to_string();
            let inst_mgr = inst_arc.clone();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                {
                    let mut im = inst_mgr.lock().await;
                    let mut cm = cfg.lock().await;
                    if let Some(active_id) = cm.config.active_instance_id.clone() {
                        if let Some(inst) = im.instances.get_mut(&active_id) {
                            inst.minecraft_version = ver_str.clone();
                            let inst_clone = inst.clone();
                            let _ = im.save_instance(&inst_clone);
                        }
                    }
                    let _ = cm.save();
                }
                dispatch_refresh_instances(ui_w.clone(), &inst_mgr, &cfg).await;
                dispatch_refresh_mods(ui_w, &inst_mgr, &cfg).await;
            });
        });
    }

    // Callback: SELECT ACTIVE VERSION (Dropdown)
    {
        let inst_arc = instance_mgr.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_select_active_version(move |ver: SharedString| {
            let ver_str = ver.to_string();
            let inst_mgr = inst_arc.clone();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                {
                    let mut im = inst_mgr.lock().await;
                    let mut cm = cfg.lock().await;
                    if let Some(active_id) = cm.config.active_instance_id.clone() {
                        if let Some(inst) = im.instances.get_mut(&active_id) {
                            inst.minecraft_version = ver_str.clone();
                            let inst_clone = inst.clone();
                            let _ = im.save_instance(&inst_clone);
                        }
                    }
                    let _ = cm.save();
                }
                dispatch_refresh_instances(ui_w.clone(), &inst_mgr, &cfg).await;
                dispatch_refresh_mods(ui_w, &inst_mgr, &cfg).await;
            });
        });
    }

    // Callback: SELECT ACTIVE LOADER (Dropdown: Fabric, Forge, OptiFine, Vanilla)
    {
        let inst_arc = instance_mgr.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_select_active_loader(move |loader: SharedString| {
            let loader_str = loader.to_string();
            let inst_mgr = inst_arc.clone();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                {
                    let mut im = inst_mgr.lock().await;
                    let mut cm = cfg.lock().await;
                    if let Some(active_id) = cm.config.active_instance_id.clone() {
                        if let Some(inst) = im.instances.get_mut(&active_id) {
                            inst.mod_loader = loader_str.to_lowercase();
                            let inst_clone = inst.clone();
                            let _ = im.save_instance(&inst_clone);
                        }
                    }
                    let _ = cm.save();
                }
                dispatch_refresh_instances(ui_w.clone(), &inst_mgr, &cfg).await;
                dispatch_refresh_mods(ui_w, &inst_mgr, &cfg).await;
            });
        });
    }

    // Callbacks: MOD MENU (Installed Mods & Modrinth Online Discovery)
    {
        let inst_arc = instance_mgr.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        let im_c1 = inst_arc.clone();
        let cf_c1 = cfg_arc.clone();
        let ui_c1 = ui_weak.clone();
        main_window.on_refresh_mods_clicked(move || {
            let im = im_c1.clone();
            let cf = cf_c1.clone();
            let ui = ui_c1.clone();
            tokio::spawn(async move {
                dispatch_refresh_mods(ui, &im, &cf).await;
            });
        });

        let im_c2 = inst_arc.clone();
        let cf_c2 = cfg_arc.clone();
        main_window.on_open_mods_folder_clicked(move || {
            let im = im_c2.clone();
            let cf = cf_c2.clone();
            tokio::spawn(async move {
                let inst_mgr = im.lock().await;
                let cfg_mgr = cf.lock().await;
                if let Some(inst) = inst_mgr.get_active_instance(&cfg_mgr.config) {
                    let mods_dir = inst.game_dir(cfg_mgr.config.instances_dir()).join("mods");
                    let _ = dv_launcher::instances::open_mods_folder(&mods_dir);
                }
            });
        });

        let im_c3 = inst_arc.clone();
        let cf_c3 = cfg_arc.clone();
        let ui_c3 = ui_weak.clone();
        main_window.on_toggle_mod_clicked(move |file_name: SharedString| {
            let im = im_c3.clone();
            let cf = cf_c3.clone();
            let ui = ui_c3.clone();
            let name_str = file_name.to_string();
            tokio::spawn(async move {
                {
                    let inst_mgr = im.lock().await;
                    let cfg_mgr = cf.lock().await;
                    if let Some(inst) = inst_mgr.get_active_instance(&cfg_mgr.config) {
                        let mods_dir = inst.game_dir(cfg_mgr.config.instances_dir()).join("mods");
                        let _ = dv_launcher::instances::toggle_mod(&mods_dir, &name_str);
                    }
                }
                dispatch_refresh_mods(ui, &im, &cf).await;
            });
        });

        let im_c4 = inst_arc.clone();
        let cf_c4 = cfg_arc.clone();
        let ui_c4 = ui_weak.clone();
        main_window.on_delete_mod_clicked(move |file_name: SharedString| {
            let im = im_c4.clone();
            let cf = cf_c4.clone();
            let ui = ui_c4.clone();
            let name_str = file_name.to_string();
            tokio::spawn(async move {
                {
                    let inst_mgr = im.lock().await;
                    let cfg_mgr = cf.lock().await;
                    if let Some(inst) = inst_mgr.get_active_instance(&cfg_mgr.config) {
                        let mods_dir = inst.game_dir(cfg_mgr.config.instances_dir()).join("mods");
                        let _ = dv_launcher::instances::delete_mod(&mods_dir, &name_str);
                    }
                }
                dispatch_refresh_mods(ui, &im, &cf).await;
            });
        });

        let im_c5 = inst_arc.clone();
        let cf_c5 = cfg_arc.clone();
        let ui_c5 = ui_weak.clone();
        main_window.on_filter_installed_mods(move |_query| {
            let im = im_c5.clone();
            let cf = cf_c5.clone();
            let ui = ui_c5.clone();
            tokio::spawn(async move {
                dispatch_refresh_mods(ui, &im, &cf).await;
            });
        });

        let im_c6 = inst_arc.clone();
        let cf_c6 = cfg_arc.clone();
        let ui_c6 = ui_weak.clone();
        main_window.on_import_mod_file_clicked(move || {
            let im = im_c6.clone();
            let cf = cf_c6.clone();
            let ui = ui_c6.clone();
            tokio::spawn(async move {
                let picked_file = tokio::task::spawn_blocking(|| {
                    rfd::FileDialog::new()
                        .add_filter("Minecraft Mod (.jar)", &["jar"])
                        .set_title("Select Mod (.jar) to Import")
                        .pick_file()
                })
                .await
                .ok()
                .flatten();

                if let Some(src_path) = picked_file {
                    {
                        let inst_mgr = im.lock().await;
                        let cfg_mgr = cf.lock().await;
                        if let Some(inst) = inst_mgr.get_active_instance(&cfg_mgr.config) {
                            let mods_dir = inst.game_dir(cfg_mgr.config.instances_dir()).join("mods");
                            let _ = dv_launcher::instances::import_mod_file(&mods_dir, &src_path);
                        }
                    }
                    dispatch_refresh_mods(ui, &im, &cf).await;
                }
            });
        });

        let im_c7 = inst_arc.clone();
        let cf_c7 = cfg_arc.clone();
        let ui_c7 = ui_weak.clone();
        main_window.on_search_online_mods_clicked(move |query: SharedString| {
            let query_str = query.to_string();
            let im = im_c7.clone();
            let cf = cf_c7.clone();
            let ui = ui_c7.clone();

            tokio::spawn(async move {
                let _ = slint::invoke_from_event_loop({
                    let ui_w = ui.clone();
                    move || {
                        if let Some(u) = ui_w.upgrade() {
                            u.set_is_searching_online_mods(true);
                        }
                    }
                });

                let (mc_ver, loader, mods_dir) = {
                    let inst_mgr = im.lock().await;
                    let cfg_mgr = cf.lock().await;
                    if let Some(inst) = inst_mgr.get_active_instance(&cfg_mgr.config) {
                        (
                            inst.minecraft_version.clone(),
                            inst.mod_loader.clone(),
                            inst.game_dir(cfg_mgr.config.instances_dir()).join("mods"),
                        )
                    } else {
                        ("1.21.4".to_string(), "fabric".to_string(), PathBuf::new())
                    }
                };

                let modrinth = ModrinthClient::new();
                let results = modrinth
                    .search_mods(&query_str, Some(&mc_ver), Some(&loader))
                    .await
                    .unwrap_or_default();

                let installed_files: Vec<String> = if mods_dir.exists() {
                    dv_launcher::instances::scan_mods(&mods_dir)
                        .into_iter()
                        .map(|m| m.file_name.to_lowercase())
                        .collect()
                } else {
                    Vec::new()
                };

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(u) = ui.upgrade() {
                        let online_items: Vec<OnlineModItem> = results
                            .into_iter()
                            .map(|m| {
                                let is_installed = installed_files.iter().any(|f| {
                                    f.contains(&m.slug.to_lowercase())
                                        || f.contains(&m.title.to_lowercase().replace(' ', "-"))
                                });
                                let dl_display = if m.downloads >= 1_000_000 {
                                    format!("{:.1}M", m.downloads as f64 / 1_000_000.0)
                                } else if m.downloads >= 1_000 {
                                    format!("{:.1}K", m.downloads as f64 / 1_000.0)
                                } else {
                                    format!("{}", m.downloads)
                                };

                                let (icon, has_icon) = if let Some(ref p) = m.cached_icon_path {
                                    match slint::Image::load_from_path(p) {
                                        Ok(img) => (img, true),
                                        Err(_) => (slint::Image::default(), false),
                                    }
                                } else {
                                    (slint::Image::default(), false)
                                };
                                let initial = m.title.chars().next().unwrap_or('M').to_uppercase().to_string();

                                OnlineModItem {
                                    project_id: SharedString::from(m.project_id),
                                    title: SharedString::from(m.title),
                                    description: SharedString::from(m.description),
                                    author: SharedString::from(m.author),
                                    downloads: SharedString::from(dl_display),
                                    slug: SharedString::from(m.slug),
                                    is_installed,
                                    icon,
                                    has_icon,
                                    initial: SharedString::from(initial),
                                }
                            })
                            .collect();

                        u.set_is_searching_online_mods(false);
                        u.set_online_mod_list(ModelRc::new(VecModel::from(online_items)));
                    }
                });
            });
        });

        let im_c8 = inst_arc.clone();
        let cf_c8 = cfg_arc.clone();
        let ui_c8 = ui_weak.clone();
        main_window.on_install_online_mod_clicked(move |slug: SharedString| {
            let slug_str = slug.to_string();
            let im = im_c8.clone();
            let cf = cf_c8.clone();
            let ui = ui_c8.clone();

            tokio::spawn(async move {
                let (mc_ver, loader, mods_dir) = {
                    let inst_mgr = im.lock().await;
                    let cfg_mgr = cf.lock().await;
                    if let Some(inst) = inst_mgr.get_active_instance(&cfg_mgr.config) {
                        (
                            inst.minecraft_version.clone(),
                            inst.mod_loader.clone(),
                            inst.game_dir(cfg_mgr.config.instances_dir()).join("mods"),
                        )
                    } else {
                        return;
                    }
                };

                let modrinth = ModrinthClient::new();
                match modrinth.install_mod(&slug_str, &mc_ver, &loader, &mods_dir).await {
                    Ok(jar_name) => {
                        info!("Successfully installed online mod {} ({})", slug_str, jar_name);
                    }
                    Err(e) => {
                        error!("Failed to install mod {}: {}", slug_str, e);
                    }
                }

                dispatch_refresh_mods(ui.clone(), &im, &cf).await;

                // Update online item is_installed state
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(u) = ui.upgrade() {
                        let current: Vec<OnlineModItem> = u
                            .get_online_mod_list()
                            .iter()
                            .map(|mut item| {
                                if item.slug == slug_str.as_str() {
                                    item.is_installed = true;
                                }
                                item
                            })
                            .collect();
                        u.set_online_mod_list(ModelRc::new(VecModel::from(current)));
                    }
                });
            });
        });
    }

    // Callbacks: DOWNLOAD MANAGER (Requirement 5)
    {
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_open_cache_folder_clicked(move || {
            let cf = cfg_arc.clone();
            tokio::spawn(async move {
                let cm = cf.lock().await;
                let dir = cm.config.shared_versions_dir();
                let _ = dv_launcher::instances::open_mods_folder(&dir);
            });
        });

        let ui_clear = ui_weak.clone();
        main_window.on_clear_downloads_history_clicked(move || {
            if let Some(ui) = ui_clear.upgrade() {
                ui.set_download_history(ModelRc::new(VecModel::from(vec![])));
                ui.set_downloads_completed_count(0);
                ui.set_downloads_total_count(0);
                ui.set_download_status_text(SharedString::from("Idle"));
                ui.set_download_progress(0.0);
            }
        });
    }

    // Callback: ADD OFFLINE ACCOUNT
    {
        let auth_arc = auth_client.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_add_offline_account_clicked(move || {
            let username = if let Some(ui) = ui_weak.upgrade() {
                ui.get_offline_username_input().to_string()
            } else {
                return;
            };

            let auth = auth_arc.clone();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                let mut a = auth.lock().await;
                match a.create_offline_account(&username) {
                    Ok(acc) => {
                        let mut cm = cfg.lock().await;
                        cm.config.active_account_id = Some(acc.id);
                        let _ = cm.save();
                        let ui_succ = ui_w.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_succ.upgrade() {
                                ui.set_offline_username_input(SharedString::from(""));
                                ui.set_username_hint(SharedString::from("Account configured successfully ✓"));
                                ui.set_is_username_valid(false);
                            }
                        });
                    }
                    Err(e) => {
                        let err_str = e.to_string();
                        let ui_fail = ui_w.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_fail.upgrade() {
                                ui.set_username_hint(SharedString::from(err_str));
                                ui.set_is_username_valid(false);
                            }
                        });
                    }
                }
                drop(a);
                dispatch_refresh_accounts(ui_w, &auth, &cfg).await;
            });
        });
    }

    // Callback: VALIDATE OFFLINE USERNAME
    {
        let ui_weak = main_window.as_weak();
        main_window.on_validate_offline_username(move |name: SharedString| {
            let name_str = name.to_string();
            if let Some(ui) = ui_weak.upgrade() {
                if name_str.is_empty() {
                    ui.set_username_hint(SharedString::from(""));
                    ui.set_is_username_valid(false);
                } else {
                    match MicrosoftAuthClient::validate_offline_username(&name_str) {
                        Ok(()) => {
                            ui.set_username_hint(SharedString::from("Valid Minecraft username ✓"));
                            ui.set_is_username_valid(true);
                        }
                        Err(err) => {
                            ui.set_username_hint(SharedString::from(err));
                            ui.set_is_username_valid(false);
                        }
                    }
                }
            }
        });
    }

    // Callback: START MS AUTH
    {
        let auth_arc = auth_client.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_start_ms_auth_clicked(move || {
            let auth = auth_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                let mut a = auth.lock().await;
                let url = a.generate_login_url();
                info!("Generated Microsoft OAuth URL: {}", url);

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_w.upgrade() {
                        ui.set_auth_login_url(SharedString::from(&url));
                        let mut out = ui.get_console_output().to_string();
                        out.push_str(&format!(
                            "\n[Microsoft Auth] Open this URL in browser to authenticate:\n{}\nPaste the returned code into the field above and click 'Confirm Login'.\n",
                            url
                        ));
                        ui.set_console_output(SharedString::from(out));
                        ui.set_active_tab(5); // Switch to console view
                    }
                });
            });
        });
    }

    // Callback: COMPLETE MS AUTH
    {
        let auth_arc = auth_client.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_complete_ms_auth_clicked(move || {
            let code = if let Some(ui) = ui_weak.upgrade() {
                ui.get_auth_code_input().to_string()
            } else {
                return;
            };

            let auth = auth_arc.clone();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                let mut a = auth.lock().await;
                match a.complete_login_with_code(&code).await {
                    Ok(acc) => {
                        let mut cm = cfg.lock().await;
                        cm.config.active_account_id = Some(acc.id);
                        let _ = cm.save();
                    }
                    Err(e) => {
                        error!("Microsoft authentication failed: {}", e);
                    }
                }
                drop(a);
                dispatch_refresh_accounts(ui_w, &auth, &cfg).await;
            });
        });
    }

    // Callback: LOGOUT ACCOUNT
    {
        let auth_arc = auth_client.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_logout_account_clicked(move |id: SharedString| {
            let auth = auth_arc.clone();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();
            let id_str = id.to_string();

            tokio::spawn(async move {
                {
                    let mut a = auth.lock().await;
                    let _ = a.logout(&id_str);
                }
                dispatch_refresh_accounts(ui_w, &auth, &cfg).await;
            });
        });
    }

    // Callback: CHANGE SKIN (.png file picker)
    {
        let auth_arc = auth_client.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_change_skin_clicked(move || {
            let auth = auth_arc.clone();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                let picked_file = tokio::task::spawn_blocking(|| {
                    rfd::FileDialog::new()
                        .add_filter("Minecraft Skin (*.png)", &["png"])
                        .set_title("Select Minecraft Skin (.png)")
                        .pick_file()
                })
                .await
                .ok()
                .flatten();

                if let Some(src_path) = picked_file {
                    let (acc_opt, instances_dir) = {
                        let a = auth.lock().await;
                        let cm = cfg.lock().await;
                        let info = a
                            .get_active_account(cm.config.active_account_id.as_deref())
                            .map(|acc| (acc.username.clone(), acc.account_type == "microsoft"));
                        (info, cm.config.instances_dir())
                    };

                    if let Some((active_user, is_ms)) = acc_opt {
                        let _ = dv_launcher::skins::save_custom_skin(&active_user, &src_path);
                        dv_launcher::skins::sync_universal_skin_to_instances(&instances_dir, &active_user);
                        dispatch_refresh_player_appearance(ui_w, &active_user, is_ms).await;
                    }
                }
            });
        });
    }

    // Callback: CHANGE CAPE (.png file picker)
    {
        let auth_arc = auth_client.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_change_cape_clicked(move || {
            let auth = auth_arc.clone();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                let picked_file = tokio::task::spawn_blocking(|| {
                    rfd::FileDialog::new()
                        .add_filter("Minecraft Cape (*.png)", &["png"])
                        .set_title("Select Minecraft Cape (.png)")
                        .pick_file()
                })
                .await
                .ok()
                .flatten();

                if let Some(src_path) = picked_file {
                    let (acc_opt, instances_dir) = {
                        let a = auth.lock().await;
                        let cm = cfg.lock().await;
                        let info = a
                            .get_active_account(cm.config.active_account_id.as_deref())
                            .map(|acc| (acc.username.clone(), acc.account_type == "microsoft"));
                        (info, cm.config.instances_dir())
                    };

                    if let Some((active_user, is_ms)) = acc_opt {
                        let _ = dv_launcher::skins::save_custom_cape(&active_user, &src_path);
                        dv_launcher::skins::sync_universal_skin_to_instances(&instances_dir, &active_user);
                        dispatch_refresh_player_appearance(ui_w, &active_user, is_ms).await;
                    }
                }
            });
        });
    }

    // Callback: RESET SKIN
    {
        let auth_arc = auth_client.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_reset_skin_clicked(move || {
            let auth = auth_arc.clone();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                let (acc_opt, instances_dir) = {
                    let a = auth.lock().await;
                    let cm = cfg.lock().await;
                    let info = a
                        .get_active_account(cm.config.active_account_id.as_deref())
                        .map(|acc| (acc.username.clone(), acc.account_type == "microsoft"));
                    (info, cm.config.instances_dir())
                };

                if let Some((active_user, is_ms)) = acc_opt {
                    dv_launcher::skins::reset_player_skin(&active_user);
                    dv_launcher::skins::sync_universal_skin_to_instances(&instances_dir, &active_user);
                    dispatch_refresh_player_appearance(ui_w, &active_user, is_ms).await;
                }
            });
        });
    }

    // Callback: RESET CAPE
    {
        let auth_arc = auth_client.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_reset_cape_clicked(move || {
            let auth = auth_arc.clone();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                let (acc_opt, instances_dir) = {
                    let a = auth.lock().await;
                    let cm = cfg.lock().await;
                    let info = a
                        .get_active_account(cm.config.active_account_id.as_deref())
                        .map(|acc| (acc.username.clone(), acc.account_type == "microsoft"));
                    (info, cm.config.instances_dir())
                };

                if let Some((active_user, is_ms)) = acc_opt {
                    dv_launcher::skins::reset_player_cape(&active_user);
                    dv_launcher::skins::sync_universal_skin_to_instances(&instances_dir, &active_user);
                    dispatch_refresh_player_appearance(ui_w, &active_user, is_ms).await;
                }
            });
        });
    }

    // Callback: SELECT CAPE PRESET
    {
        let auth_arc = auth_client.clone();
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_select_cape_preset(move |_preset: SharedString| {
            let auth = auth_arc.clone();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                let (acc_opt, instances_dir) = {
                    let a = auth.lock().await;
                    let cm = cfg.lock().await;
                    let info = a
                        .get_active_account(cm.config.active_account_id.as_deref())
                        .map(|acc| (acc.username.clone(), acc.account_type == "microsoft"));
                    (info, cm.config.instances_dir())
                };

                if let Some((active_user, is_ms)) = acc_opt {
                    dv_launcher::skins::ensure_preset_capes();
                    dv_launcher::skins::sync_universal_skin_to_instances(&instances_dir, &active_user);
                    dispatch_refresh_player_appearance(ui_w, &active_user, is_ms).await;
                }
            });
        });
    }

    // Callback: CLEAR CONSOLE
    {
        let ui_weak = main_window.as_weak();
        main_window.on_clear_console_clicked(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_console_output(SharedString::from(""));
            }
        });
    }

    // Callback: BROWSE JAVA (NATIVE FILE PICKER VIA RFD)
    {
        let ui_weak = main_window.as_weak();
        let cfg_arc = config_mgr.clone();

        main_window.on_browse_java_clicked(move || {
            let ui_w = ui_weak.clone();
            let cfg = cfg_arc.clone();

            tokio::spawn(async move {
                let selected = tokio::task::spawn_blocking(JavaDetector::pick_java_with_dialog)
                    .await
                    .ok()
                    .flatten();

                if let Some(selected) = selected {
                    let path_str = selected.path.to_string_lossy().to_string();
                    let disp = selected.display_name.clone();

                    let mut cm = cfg.lock().await;
                    cm.config.default_java_path = Some(selected.path.clone());
                    let _ = cm.save();

                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_w.upgrade() {
                            ui.set_custom_java_path(SharedString::from(&path_str));
                            ui.set_java_runtime_display(SharedString::from(&disp));
                        }
                    });
                }
            });
        });
    }

    // Callback: SELECT JAVA FROM DETECTED LIST
    {
        let ui_weak = main_window.as_weak();
        let cfg_arc = config_mgr.clone();

        main_window.on_select_java_runtime(move |path: SharedString| {
            let path_buf = PathBuf::from(path.as_str());
            let ui_w = ui_weak.clone();
            let cfg = cfg_arc.clone();

            tokio::spawn(async move {
                if let Some(info) = JavaDetector::inspect_executable(&path_buf) {
                    let mut cm = cfg.lock().await;
                    cm.config.default_java_path = Some(path_buf.clone());
                    let _ = cm.save();

                    let disp = info.display_name.clone();
                    let p_str = path.clone();

                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_w.upgrade() {
                            ui.set_custom_java_path(p_str);
                            ui.set_java_runtime_display(SharedString::from(disp));
                        }
                    });
                }
            });
        });
    }

    // Callback: BROWSE LAUNCHER ROOT DIRECTORY
    {
        let ui_weak = main_window.as_weak();
        let cfg_arc = config_mgr.clone();

        main_window.on_browse_root_dir_clicked(move || {
            let current_dir = {
                let ui = ui_weak.upgrade();
                ui.map(|u| PathBuf::from(u.get_launcher_root_dir_input().as_str()))
            };
            let ui_w = ui_weak.clone();
            let cfg = cfg_arc.clone();

            tokio::spawn(async move {
                let picked = tokio::task::spawn_blocking(move || {
                    pick_directory("Select Launcher Root Directory", current_dir.as_deref())
                })
                .await
                .ok()
                .flatten();

                if let Some(dir) = picked {
                    let path_str = dir.to_string_lossy().to_string();
                    let mut cm = cfg.lock().await;
                    cm.config.data_dir = dir.clone();
                    let _ = cm.config.ensure_directories();
                    let _ = cm.save();

                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_w.upgrade() {
                            ui.set_launcher_root_dir_input(SharedString::from(&path_str));
                        }
                    });
                }
            });
        });
    }

    // Callback: BROWSE PROFILES / INSTANCES DIRECTORY
    {
        let ui_weak = main_window.as_weak();
        let cfg_arc = config_mgr.clone();

        main_window.on_browse_profiles_dir_clicked(move || {
            let current_dir = {
                let ui = ui_weak.upgrade();
                ui.map(|u| PathBuf::from(u.get_profiles_dir_input().as_str()))
            };
            let ui_w = ui_weak.clone();
            let cfg = cfg_arc.clone();

            tokio::spawn(async move {
                let picked = tokio::task::spawn_blocking(move || {
                    pick_directory("Select Game Profiles Directory", current_dir.as_deref())
                })
                .await
                .ok()
                .flatten();

                if let Some(dir) = picked {
                    let path_str = dir.to_string_lossy().to_string();
                    let mut cm = cfg.lock().await;
                    cm.config.custom_instances_dir = Some(dir.clone());
                    let _ = cm.config.ensure_directories();
                    let _ = cm.save();

                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_w.upgrade() {
                            ui.set_profiles_dir_input(SharedString::from(&path_str));
                        }
                    });
                }
            });
        });
    }

    // Callback: OPEN ROOT DIR IN FILE EXPLORER
    {
        let cfg_arc = config_mgr.clone();
        main_window.on_open_root_dir_clicked(move || {
            let cf = cfg_arc.clone();
            tokio::spawn(async move {
                let cm = cf.lock().await;
                let dir = cm.config.base_path();
                let _ = dv_launcher::instances::open_mods_folder(dir);
            });
        });
    }

    // Callback: OPEN PROFILES DIR IN FILE EXPLORER
    {
        let cfg_arc = config_mgr.clone();
        main_window.on_open_profiles_dir_clicked(move || {
            let cf = cfg_arc.clone();
            tokio::spawn(async move {
                let cm = cf.lock().await;
                let dir = cm.config.instances_dir();
                let _ = dv_launcher::instances::open_mods_folder(&dir);
            });
        });
    }

    // Callback: RESET SETTINGS TO DEFAULTS
    {
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_reset_settings_clicked(move || {
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                let default_cfg = dv_launcher::config::LauncherConfig::default();
                let mut cm = cfg.lock().await;
                cm.config.default_min_memory_mb = default_cfg.default_min_memory_mb;
                cm.config.default_max_memory_mb = default_cfg.default_max_memory_mb;
                cm.config.default_window_width = default_cfg.default_window_width;
                cm.config.default_window_height = default_cfg.default_window_height;
                cm.config.privacy.enable_discord_rpc = default_cfg.privacy.enable_discord_rpc;
                cm.config.performance_preset = default_cfg.performance_preset.clone();
                cm.config.custom_jvm_args = default_cfg.custom_jvm_args.clone();
                cm.config.on_launch_action = default_cfg.on_launch_action.clone();
                let _ = cm.save();

                let def_min = default_cfg.default_min_memory_mb as f32;
                let def_max = default_cfg.default_max_memory_mb as f32;
                let def_w = default_cfg.default_window_width as i32;
                let def_h = default_cfg.default_window_height as i32;
                let def_rpc = default_cfg.privacy.enable_discord_rpc;
                let def_act = default_cfg.on_launch_action.clone();

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_w.upgrade() {
                        ui.set_min_memory_mb(def_min);
                        ui.set_max_memory_mb(def_max);
                        ui.set_window_width(def_w);
                        ui.set_window_height(def_h);
                        ui.set_enable_discord_rpc(def_rpc);
                        ui.set_custom_jvm_args_input(SharedString::from(""));
                        ui.set_on_launch_action(SharedString::from(&def_act));
                        ui.set_on_launch_action_display(SharedString::from("Hide Launcher (Minimizes RAM)"));
                        ui.set_settings_save_status(SharedString::from("Reset to Defaults ✓"));
                    }
                });
            });
        });
    }

    // Callback: SELECT GPU PREFERENCE
    {
        let ui_weak = main_window.as_weak();
        let cfg_arc = config_mgr.clone();

        main_window.on_select_gpu_preference(move |id: SharedString, name: SharedString| {
            let ui_w = ui_weak.clone();
            let cfg = cfg_arc.clone();
            let id_str = id.to_string();
            let name_str = name.to_string();

            tokio::spawn(async move {
                let mut cm = cfg.lock().await;
                cm.config.selected_gpu = Some(id_str.clone());
                let _ = cm.save();

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_w.upgrade() {
                        ui.set_selected_gpu_id(SharedString::from(&id_str));
                        ui.set_selected_gpu_display(SharedString::from(&name_str));
                    }
                });
            });
        });
    }

    // Callback: SAVE SETTINGS
    {
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();
        let rpc = discord_rpc.clone();

        main_window.on_save_settings_clicked(move || {
            let (min_mem, max_mem, width, height, discord_rpc, custom_java, custom_jvm, on_launch, root_dir, profiles_dir, perf_preset) =
                if let Some(ui) = ui_weak.upgrade() {
                    (
                        ui.get_min_memory_mb() as u32,
                        ui.get_max_memory_mb() as u32,
                        ui.get_window_width() as u32,
                        ui.get_window_height() as u32,
                        ui.get_enable_discord_rpc(),
                        ui.get_custom_java_path().to_string(),
                        ui.get_custom_jvm_args_input().to_string(),
                        ui.get_on_launch_action().to_string(),
                        ui.get_launcher_root_dir_input().to_string(),
                        ui.get_profiles_dir_input().to_string(),
                        ui.get_performance_preset().to_string(),
                    )
                } else {
                    return;
                };

            let cfg = cfg_arc.clone();
            let rpc_c = rpc.clone();
            let ui_w = ui_weak.clone();
            tokio::spawn(async move {
                let mut cm = cfg.lock().await;
                cm.config.default_min_memory_mb = min_mem;
                cm.config.default_max_memory_mb = max_mem;
                cm.config.default_window_width = width;
                cm.config.default_window_height = height;
                cm.config.privacy.enable_discord_rpc = discord_rpc;
                cm.config.performance_preset = perf_preset;
                cm.config.custom_jvm_args = custom_jvm;
                if !on_launch.is_empty() {
                    cm.config.on_launch_action = on_launch;
                }

                if !root_dir.trim().is_empty() {
                    cm.config.data_dir = PathBuf::from(root_dir.trim());
                }
                if !profiles_dir.trim().is_empty() {
                    cm.config.custom_instances_dir = Some(PathBuf::from(profiles_dir.trim()));
                }
                cm.config.default_java_path = if custom_java.trim().is_empty() {
                    None
                } else {
                    Some(PathBuf::from(custom_java.trim()))
                };

                let _ = cm.config.ensure_directories();
                let _ = cm.save();

                if discord_rpc {
                    rpc_c.set_in_launcher();
                } else {
                    rpc_c.clear();
                }

                info!("Client preferences and directories saved successfully.");

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_w.upgrade() {
                        ui.set_settings_save_status(SharedString::from("Settings Saved ✓"));
                    }
                });
            });
        });
    }

    // Callback: SELECT PERFORMANCE PRESET
    {
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_select_performance_preset(move |val: SharedString| {
            let val_str = val.to_string();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();

            let (preset_key, display_name) = if val_str.contains("Aikar") {
                ("aikar", "Aikar's Flags (High FPS & Zero Stutter)")
            } else if val_str.contains("Shenandoah") {
                ("shenandoah", "Shenandoah GC (Ultra-Low Latency)")
            } else if val_str.contains("ZGC") {
                ("zgc", "Generational ZGC (Java 21+ Modern)")
            } else {
                ("default", "Vanilla / Default JVM")
            };

            tokio::spawn(async move {
                {
                    let mut cm = cfg.lock().await;
                    cm.config.performance_preset = preset_key.to_string();
                    let _ = cm.save();
                }
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_w.upgrade() {
                        ui.set_performance_preset(SharedString::from(preset_key));
                        ui.set_performance_preset_display(SharedString::from(display_name));
                    }
                });
            });
        });
    }

    // Callback: SELECT ON-LAUNCH ACTION
    {
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_select_on_launch_action(move |val: SharedString| {
            let val_str = val.to_string();
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();

            let (action_key, display_name) = match val_str.as_str() {
                "close" => ("close", "Close Launcher Completely"),
                "keep_open" => ("keep_open", "Keep Launcher Open"),
                _ => ("hide", "Hide Launcher (Minimizes RAM)"),
            };

            tokio::spawn(async move {
                {
                    let mut cm = cfg.lock().await;
                    cm.config.on_launch_action = action_key.to_string();
                    let _ = cm.save();
                }
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_w.upgrade() {
                        ui.set_on_launch_action(SharedString::from(action_key));
                        ui.set_on_launch_action_display(SharedString::from(display_name));
                    }
                });
            });
        });
    }

    // Callback: CLEAR ASSET CACHE
    {
        let cfg_arc = config_mgr.clone();
        let ui_weak = main_window.as_weak();

        main_window.on_clear_asset_cache_clicked(move || {
            let cfg = cfg_arc.clone();
            let ui_w = ui_weak.clone();

            tokio::spawn(async move {
                let assets_dir = {
                    let cm = cfg.lock().await;
                    cm.config.shared_assets_dir()
                };
                let _ = std::fs::remove_dir_all(&assets_dir);
                let _ = std::fs::create_dir_all(&assets_dir);
                info!("Shared assets cache cleared.");

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_w.upgrade() {
                        ui.set_settings_save_status(SharedString::from("Asset Cache Purged ✓"));
                    }
                });
            });
        });
    }

    // Callback: OPEN CACHES FOLDER
    {
        let cfg_arc = config_mgr.clone();

        main_window.on_open_caches_folder_clicked(move || {
            let cfg = cfg_arc.clone();
            tokio::spawn(async move {
                let assets_dir = {
                    let cm = cfg.lock().await;
                    cm.config.shared_assets_dir()
                };
                let _ = std::fs::create_dir_all(&assets_dir);
                let _ = dv_launcher::instances::open_mods_folder(&assets_dir);
            });
        });
    }

    // Callback: OPEN DISCORD
    main_window.on_open_discord_clicked(move || {
        open_browser(branding::DISCORD_URL);
    });

    // Callback: OPEN WEBSITE
    main_window.on_open_website_clicked(move || {
        open_browser(branding::WEBSITE_URL);
    });

    // Run Slint native event loop
    main_window.run()?;
    Ok(())
}

fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = std::process::Command::new("rundll32");
        cmd.args(["url.dll,FileProtocolHandler", url]);
        cmd.creation_flags(0x08000000);
        let _ = cmd.spawn();
    }
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}

async fn dispatch_refresh_instances(
    ui_weak: Weak<MainWindow>,
    inst_arc: &Arc<Mutex<InstanceManager>>,
    cfg_arc: &Arc<Mutex<ConfigManager>>,
) {
    let im = inst_arc.lock().await;
    let cm = cfg_arc.lock().await;

    let items: Vec<InstanceItem> = im
        .instances
        .values()
        .map(|inst| InstanceItem {
            id: SharedString::from(&inst.id),
            name: SharedString::from(&inst.name),
            mc_version: SharedString::from(&inst.minecraft_version),
            mod_loader: SharedString::from(&inst.mod_loader),
            last_played: SharedString::from(
                inst.last_played
                    .map(|t| format!("{}", t))
                    .unwrap_or_else(|| "Never".to_string()),
            ),
        })
        .collect();

    let active_info = im.get_active_instance(&cm.config).map(|a| {
        let loader_title = match a.mod_loader.to_lowercase().as_str() {
            "fabric" => "Fabric",
            "forge" => "Forge",
            "optifine" => "OptiFine",
            "vanilla" => "Vanilla",
            _ => "Fabric",
        };
        (
            a.name.clone(),
            a.minecraft_version.clone(),
            loader_title.to_string(),
        )
    });

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            if let Some((name, ver, loader)) = active_info {
                let ver_shared = SharedString::from(&ver);
                let current_models: Vec<SharedString> = ui.get_version_list().iter().collect();
                if !current_models.contains(&ver_shared) {
                    let mut updated = vec![ver_shared.clone()];
                    updated.extend(current_models);
                    ui.set_version_list(ModelRc::new(VecModel::from(updated)));
                }

                ui.set_active_instance_name(SharedString::from(name));
                ui.set_active_mc_version(ver_shared);
                ui.set_active_mod_loader(SharedString::from(loader));
            }
            ui.set_instance_list(ModelRc::new(VecModel::from(items)));
        }
    });
}

async fn dispatch_refresh_player_appearance(
    ui_weak: Weak<MainWindow>,
    username: &str,
    is_microsoft: bool,
) {
    let u_name = username.to_string();
    let skin_path = dv_launcher::skins::get_or_fetch_player_skin(&u_name, is_microsoft).await;
    let u_name2 = u_name.clone();
    let cape_path = tokio::task::spawn_blocking(move || {
        dv_launcher::skins::get_or_create_cape_preview(&u_name2, None)
    })
    .await
    .ok()
    .flatten();

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            if let Ok(skin_img) = slint::Image::load_from_path(&skin_path) {
                ui.set_player_skin_image(skin_img);
                ui.set_has_player_skin(true);
                let stem = skin_path.file_stem().unwrap_or_default().to_string_lossy();
                let skin_display = if stem.contains("universal") {
                    "Universal Custom Skin (All Players)".to_string()
                } else if stem.contains("steve") {
                    "Default Steve (Universal)".to_string()
                } else {
                    format!("Universal ({})", stem)
                };
                ui.set_player_skin_name(SharedString::from(skin_display));
            } else {
                ui.set_has_player_skin(false);
            }

            if let Some(ref cp) = cape_path {
                if let Ok(cape_img) = slint::Image::load_from_path(cp) {
                    ui.set_player_cape_image(cape_img);
                    ui.set_has_player_cape(true);
                    let stem = cp.file_stem().unwrap_or_default().to_string_lossy();
                    let cape_display = if stem.contains("universal") {
                        "Custom Cape".to_string()
                    } else {
                        format!("Custom Cape ({})", stem)
                    };
                    ui.set_player_cape_name(SharedString::from(cape_display));
                } else {
                    ui.set_has_player_cape(false);
                    ui.set_player_cape_name(SharedString::from("None"));
                }
            } else {
                ui.set_has_player_cape(false);
                ui.set_player_cape_name(SharedString::from("None"));
            }
        }
    });
}

async fn dispatch_refresh_accounts(
    ui_weak: Weak<MainWindow>,
    auth_arc: &Arc<Mutex<MicrosoftAuthClient>>,
    cfg_arc: &Arc<Mutex<ConfigManager>>,
) {
    let auth = auth_arc.lock().await;
    let cm = cfg_arc.lock().await;

    let active_acc = auth.get_active_account(cm.config.active_account_id.as_deref());
    let (active_user, is_ms) = match active_acc {
        Some(ref acc) => (acc.username.clone(), acc.account_type == "microsoft"),
        None => (String::new(), false),
    };

    let items: Vec<AccountItem> = auth
        .accounts
        .values()
        .map(|acc| AccountItem {
            id: SharedString::from(&acc.id),
            username: SharedString::from(&acc.username),
            account_type: SharedString::from(&acc.account_type),
            last_login: SharedString::from(format!("{}", acc.last_login)),
        })
        .collect();

    let ui_w2 = ui_weak.clone();
    let active_user_disp = active_user.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_w2.upgrade() {
            ui.set_active_username(SharedString::from(&active_user_disp));
            ui.set_active_account_type(SharedString::from(if active_user_disp.is_empty() {
                "Not Configured"
            } else if is_ms {
                "Microsoft Account"
            } else {
                "Offline Profile"
            }));
            ui.set_account_list(ModelRc::new(VecModel::from(items)));
        }
    });

    // Refresh player skin and cape preview
    if !active_user.is_empty() {
        dispatch_refresh_player_appearance(ui_weak, &active_user, is_ms).await;
    } else {
        let ui_w3 = ui_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_w3.upgrade() {
                ui.set_has_player_skin(false);
                ui.set_has_player_cape(false);
                ui.set_player_skin_name(SharedString::from("No Skin Configured"));
                ui.set_player_cape_name(SharedString::from("None"));
            }
        });
    }
}

async fn dispatch_refresh_mods(
    ui_weak: Weak<MainWindow>,
    inst_arc: &Arc<Mutex<InstanceManager>>,
    cfg_arc: &Arc<Mutex<ConfigManager>>,
) {
    let mods_dir_opt = {
        let im = inst_arc.lock().await;
        let cm = cfg_arc.lock().await;
        im.get_active_instance(&cm.config)
            .map(|inst| inst.game_dir(cm.config.instances_dir()).join("mods"))
    };

    let mods = if let Some(mods_dir) = mods_dir_opt {
        tokio::task::spawn_blocking(move || dv_launcher::instances::scan_mods(&mods_dir))
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let total_count = mods.len() as i32;

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let filter_query = ui.get_mod_search_query().to_string().to_lowercase();
            let items: Vec<ModItem> = mods
                .into_iter()
                .filter(|m| {
                    if filter_query.trim().is_empty() {
                        true
                    } else {
                        m.name.to_lowercase().contains(&filter_query)
                            || m.file_name.to_lowercase().contains(&filter_query)
                            || m.description.to_lowercase().contains(&filter_query)
                            || m.author.to_lowercase().contains(&filter_query)
                    }
                })
                .map(|m| {
                    let (icon, has_icon) = if let Some(ref p) = m.icon_path {
                        match slint::Image::load_from_path(p) {
                            Ok(img) => (img, true),
                            Err(_) => (slint::Image::default(), false),
                        }
                    } else {
                        (slint::Image::default(), false)
                    };
                    let initial = m.name.chars().next().unwrap_or('M').to_uppercase().to_string();

                    ModItem {
                        file_name: SharedString::from(m.file_name),
                        name: SharedString::from(m.name),
                        version: SharedString::from(m.version),
                        description: SharedString::from(m.description),
                        author: SharedString::from(m.author),
                        file_size: SharedString::from(m.file_size_display),
                        is_enabled: m.is_enabled,
                        icon,
                        has_icon,
                        initial: SharedString::from(initial),
                    }
                })
                .collect();

            ui.set_mods_count(total_count);
            ui.set_mod_list(ModelRc::new(VecModel::from(items)));
        }
    });
}

fn handle_download_progress(ui: &MainWindow, prog: &DownloadProgress) {
    let normalized = if prog.percent > 1.0 {
        prog.percent / 100.0
    } else {
        prog.percent
    };
    ui.set_download_progress(normalized.clamp(0.0, 1.0));
    ui.set_download_status_text(SharedString::from(&prog.current_item));
    ui.set_download_bytes_text(SharedString::from(format!(
        "{:.1} MB / {:.1} MB",
        prog.bytes_downloaded as f64 / 1_048_576.0,
        prog.total_bytes as f64 / 1_048_576.0
    )));
    ui.set_downloads_completed_count(prog.completed_items as i32);
    ui.set_downloads_total_count(prog.total_items as i32);

    let mut history: Vec<DownloadItem> = ui
        .get_download_history()
        .iter()
        .collect();
    if history.len() >= 25 {
        history.remove(0);
    }
    history.push(DownloadItem {
        file_name: SharedString::from(&prog.current_item),
        status: SharedString::from(if prog.percent >= 100.0 { "Verified" } else { "Downloading" }),
        size_text: SharedString::from(format!("{:.1} MB", prog.bytes_downloaded as f64 / 1_048_576.0)),
        is_done: prog.percent >= 100.0,
    });
    ui.set_download_history(ModelRc::new(VecModel::from(history)));
}
