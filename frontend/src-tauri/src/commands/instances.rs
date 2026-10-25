use crate::state::{AppState, InstanceDto, MinecraftVersionDto};
use tauri::State;

#[tauri::command]
pub async fn get_all_minecraft_versions(
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

#[tauri::command]
pub async fn get_instances(state: State<'_, AppState>) -> Result<Vec<InstanceDto>, String> {
    let mgr = state.instance_mgr.lock().await;
    let cm = state.config_mgr.lock().await;
    let active_id = cm.config.active_instance_id.as_deref();
    let global_max = cm.config.default_max_memory_mb;

    let mut list: Vec<InstanceDto> = mgr
        .instances
        .values()
        .map(|inst| {
            let is_active = active_id == Some(&inst.id);
            let effective_max = if is_active && global_max > 0 {
                global_max
            } else if inst.max_memory_mb > 0 && inst.max_memory_mb != 4096 {
                inst.max_memory_mb
            } else if global_max > 0 {
                global_max
            } else {
                inst.max_memory_mb.max(4096)
            };
            InstanceDto {
                id: inst.id.clone(),
                name: inst.name.clone(),
                mc_version: inst.minecraft_version.clone(),
                mod_loader: inst.mod_loader.clone(),
                last_played: inst.last_played,
                min_memory_mb: inst.min_memory_mb,
                max_memory_mb: effective_max,
                is_active,
            }
        })
        .collect();

    list.sort_by_key(|a| a.name.to_lowercase());
    Ok(list)
}

#[tauri::command]
pub async fn select_active_instance(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut cm = state.config_mgr.lock().await;
    cm.config.active_instance_id = Some(id);
    cm.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn update_instance_memory(
    id: String,
    max_memory_mb: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut im = state.instance_mgr.lock().await;
    if let Some(inst) = im.instances.get_mut(&id) {
        inst.max_memory_mb = max_memory_mb;
        let inst_clone = inst.clone();
        im.save_instance(&inst_clone).map_err(|e| e.to_string())?;
    } else {
        return Err("Instance not found".to_string());
    }

    // If this is the active instance, also update the global default
    let mut cm = state.config_mgr.lock().await;
    if cm.config.active_instance_id.as_deref() == Some(&id) {
        cm.config.default_max_memory_mb = max_memory_mb;
        let _ = cm.save();
    }

    Ok(())
}

#[tauri::command]
pub async fn create_instance(
    name: String,
    mc_version: String,
    loader: String,
    state: State<'_, AppState>,
) -> Result<InstanceDto, String> {
    let mut mgr = state.instance_mgr.lock().await;
    let mut inst = mgr
        .create_instance(&name, &mc_version, &loader, None)
        .map_err(|e| e.to_string())?;

    // Auto set as active instance and apply default_max_memory_mb
    let mut cm = state.config_mgr.lock().await;
    cm.config.active_instance_id = Some(inst.id.clone());
    if cm.config.default_max_memory_mb > 0 {
        inst.max_memory_mb = cm.config.default_max_memory_mb;
        let _ = mgr.save_instance(&inst);
    }
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
pub async fn delete_instance(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut mgr = state.instance_mgr.lock().await;
    mgr.delete_instance(&id).map_err(|e| e.to_string())?;

    let mut cm = state.config_mgr.lock().await;
    if cm.config.active_instance_id.as_deref() == Some(&id) {
        cm.config.active_instance_id = None;
        let _ = cm.save();
    }
    Ok(())
}
