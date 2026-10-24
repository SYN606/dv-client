use crate::state::AppState;
use dv_launcher::instances::{
    scan_mods, toggle_mod as toggle_instance_mod, ModEntry,
};
use dv_launcher::loaders::{CurseForgeClient, CurseForgeMod, ModrinthClient, ModrinthMod};
use tauri::State;

#[tauri::command]
pub async fn get_installed_mods(
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
pub async fn toggle_mod(
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
pub async fn search_modrinth(
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
pub async fn delete_mod(
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
pub async fn install_modrinth_mod(
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
pub async fn search_curseforge(
    query: Option<String>,
    loader: Option<String>,
    game_version: Option<String>,
    category_id: Option<u32>,
    sort_by: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<CurseForgeMod>, String> {
    let client = CurseForgeClient::new();
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
pub async fn install_curseforge_mod(
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

    let client = CurseForgeClient::new();
    client
        .install_mod(mod_id, &mc_ver, &loader, &mods_dir)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_mods_folder(instance_id: String, state: State<'_, AppState>) -> Result<(), String> {
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
pub async fn import_local_mod(
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
pub async fn bulk_toggle_mods(
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
pub async fn uninstall_modrinth_mod(
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
