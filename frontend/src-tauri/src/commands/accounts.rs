use crate::state::{AccountDto, AppState};
use tauri::State;

#[tauri::command]
pub async fn get_accounts(state: State<'_, AppState>) -> Result<Vec<AccountDto>, String> {
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
pub async fn select_active_account(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut cm = state.config_mgr.lock().await;
    cm.config.active_account_id = Some(id);
    cm.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn add_offline_account(
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
pub async fn logout_account(id: String, state: State<'_, AppState>) -> Result<(), String> {
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
pub async fn start_microsoft_login(state: State<'_, AppState>) -> Result<String, String> {
    let mut auth = state.auth_client.lock().await;
    let url = auth.generate_login_url();
    let _ = open::that(&url);
    Ok(url)
}

#[tauri::command]
pub async fn complete_microsoft_login(
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
