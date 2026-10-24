use crate::state::LauncherInfo;
use dv_launcher::branding;

#[tauri::command]
pub fn get_launcher_info() -> LauncherInfo {
    LauncherInfo {
        name: branding::APP_NAME.to_string(),
        version: branding::APP_VERSION.to_string(),
        author: "Digital Vigital".to_string(),
        repo_url: "https://github.com/SYN606/dv-client".to_string(),
    }
}

#[tauri::command]
pub async fn open_folder(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| e.to_string())?;
    Ok(())
}
