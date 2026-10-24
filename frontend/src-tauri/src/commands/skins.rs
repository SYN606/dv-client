use crate::state::PlayerSkinPayload;

#[tauri::command]
pub async fn pick_skin_file() -> Result<Option<String>, String> {
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
pub async fn pick_cape_file() -> Result<Option<String>, String> {
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
pub async fn get_player_skin(username: Option<String>) -> Result<PlayerSkinPayload, String> {
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
pub async fn reset_skin(username: String) -> Result<PlayerSkinPayload, String> {
    dv_launcher::skins::reset_player_skin(&username);
    get_player_skin(Some(username)).await
}

#[tauri::command]
pub async fn reset_cape(username: String) -> Result<PlayerSkinPayload, String> {
    dv_launcher::skins::reset_player_cape(&username);
    get_player_skin(Some(username)).await
}

#[tauri::command]
pub async fn apply_skin(username: String, file_path: String) -> Result<PlayerSkinPayload, String> {
    dv_launcher::skins::save_custom_skin(&username, &file_path).map_err(|e| e.to_string())?;
    get_player_skin(Some(username)).await
}

#[tauri::command]
pub async fn apply_cape(username: String, file_path: String) -> Result<PlayerSkinPayload, String> {
    dv_launcher::skins::save_custom_cape(&username, &file_path).map_err(|e| e.to_string())?;
    get_player_skin(Some(username)).await
}

fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    const B64_REV: [i8; 256] = {
        let mut table = [-1i8; 256];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0usize;
        while i < chars.len() {
            table[chars[i] as usize] = i as i8;
            i += 1;
        }
        table
    };

    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0;

    for &byte in input.as_bytes() {
        if byte == b'=' || byte.is_ascii_whitespace() {
            continue;
        }
        let val = B64_REV[byte as usize];
        if val < 0 {
            return Err("Invalid base64 character".to_string());
        }
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Ok(out)
}

#[tauri::command]
pub async fn apply_skin_base64(username: String, base64_data: String) -> Result<PlayerSkinPayload, String> {
    let clean_b64 = if let Some(idx) = base64_data.find(',') {
        &base64_data[idx + 1..]
    } else {
        &base64_data
    };

    let bytes = decode_base64(clean_b64)?;
    let temp_path = std::env::temp_dir().join(format!("dv_skin_{}.png", username));
    std::fs::write(&temp_path, &bytes).map_err(|e| e.to_string())?;
    let res = dv_launcher::skins::save_custom_skin(&username, &temp_path);
    let _ = std::fs::remove_file(&temp_path);
    res.map_err(|e| e.to_string())?;
    get_player_skin(Some(username)).await
}

#[tauri::command]
pub async fn apply_cape_base64(username: String, base64_data: String) -> Result<PlayerSkinPayload, String> {
    let clean_b64 = if let Some(idx) = base64_data.find(',') {
        &base64_data[idx + 1..]
    } else {
        &base64_data
    };

    let bytes = decode_base64(clean_b64)?;
    let temp_path = std::env::temp_dir().join(format!("dv_cape_{}.png", username));
    std::fs::write(&temp_path, &bytes).map_err(|e| e.to_string())?;
    let res = dv_launcher::skins::save_custom_cape(&username, &temp_path);
    let _ = std::fs::remove_file(&temp_path);
    res.map_err(|e| e.to_string())?;
    get_player_skin(Some(username)).await
}
