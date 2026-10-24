pub mod commands;
pub mod state;

use commands::*;
use dv_launcher::accounts::MicrosoftAuthClient;
use dv_launcher::config::ConfigManager;
use dv_launcher::discord::DiscordRpcHandle;
use dv_launcher::instances::InstanceManager;
use dv_launcher::minecraft::{ProcessState, ProcessSupervisor};
use state::AppState;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;
use tracing::info;

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
            pick_game_directory,
            reset_game_directory,
            pick_instances_directory,
            reset_instances_directory,
            open_folder,
            open_url,
            check_for_updates,
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
            report_crash,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
