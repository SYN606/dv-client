//! Minecraft Player Skin and Cape Management System.
//!
//! Provides:
//! - 64x64 and 64x32 skin rendering and front model compositing with overlay layers
//! - Cape rendering and customizable cape presets (DV Crimson, Founder, Migrator, Custom)
//! - Online skin fetching via Mojang / MC-Heads session infrastructure
//! - Local skin & cape caching and upload support for Offline & Microsoft profiles

use image::{ImageBuffer, Rgba, RgbaImage};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum SkinError {
    #[error("I/O error handling skin or cape file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image decoding error: {0}")]
    Image(#[from] image::ImageError),
    #[error("Invalid skin dimensions: expected 64x64 or 64x32, got {0}x{1}")]
    InvalidDimensions(u32, u32),
}

/// Skin model type (Classic 4px arm vs Slim 3px arm).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkinModel {
    Classic,
    Slim,
}

/// Directory paths for skins and capes.
pub fn get_skins_dir() -> PathBuf {
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("digitalvigital");
    let dir = base.join("skins");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_capes_dir() -> PathBuf {
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("digitalvigital");
    let dir = base.join("capes");
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Composites a crisp 160x320 front-view player model preview from a 64x64 or 64x32 skin PNG.
/// Includes base body parts and second-layer overlays (hat, jacket, sleeves, pants).
pub fn render_skin_preview<P: AsRef<Path>>(skin_path: P) -> Result<RgbaImage, SkinError> {
    let skin_img = image::open(skin_path.as_ref())?.to_rgba8();
    let (w, h) = skin_img.dimensions();

    if w != 64 || (h != 64 && h != 32) {
        return Err(SkinError::InvalidDimensions(w, h));
    }

    // 16x32 canvas representing the front Minecraft character model
    let mut canvas: RgbaImage = ImageBuffer::from_pixel(16, 32, Rgba([0, 0, 0, 0]));

    let blit_part = |dest: &mut RgbaImage,
                     src: &RgbaImage,
                     src_x: u32,
                     src_y: u32,
                     w: u32,
                     h: u32,
                     dest_x: u32,
                     dest_y: u32| {
        for dy in 0..h {
            for dx in 0..w {
                let px = src.get_pixel(src_x + dx, src_y + dy);
                if px[3] > 10 {
                    let d = dest.get_pixel_mut(dest_x + dx, dest_y + dy);
                    if px[3] == 255 || d[3] == 0 {
                        *d = *px;
                    } else {
                        // Alpha blend
                        let a = px[3] as f32 / 255.0;
                        let inv_a = 1.0 - a;
                        d[0] = (px[0] as f32 * a + d[0] as f32 * inv_a) as u8;
                        d[1] = (px[1] as f32 * a + d[1] as f32 * inv_a) as u8;
                        d[2] = (px[2] as f32 * a + d[2] as f32 * inv_a) as u8;
                        d[3] = 255;
                    }
                }
            }
        }
    };

    // 1. Head (8x8) at (4, 0)
    blit_part(&mut canvas, &skin_img, 8, 8, 8, 8, 4, 0);
    // Head Overlay (Hat) at (4, 0)
    blit_part(&mut canvas, &skin_img, 40, 8, 8, 8, 4, 0);

    // 2. Torso (8x12) at (4, 8)
    blit_part(&mut canvas, &skin_img, 20, 20, 8, 12, 4, 8);
    if h == 64 {
        // Torso Overlay (Jacket) at (4, 8)
        blit_part(&mut canvas, &skin_img, 20, 36, 8, 12, 4, 8);
    }

    // 3. Right Arm (4x12) at (0, 8)
    blit_part(&mut canvas, &skin_img, 44, 20, 4, 12, 0, 8);
    if h == 64 {
        // Right Arm Overlay
        blit_part(&mut canvas, &skin_img, 44, 36, 4, 12, 0, 8);
    }

    // 4. Left Arm (4x12) at (12, 8)
    if h == 64 {
        blit_part(&mut canvas, &skin_img, 36, 52, 4, 12, 12, 8);
        blit_part(&mut canvas, &skin_img, 52, 52, 4, 12, 12, 8);
    } else {
        // Legacy: Mirrored right arm
        for dy in 0..12 {
            for dx in 0..4 {
                let px = skin_img.get_pixel(44 + (3 - dx), 20 + dy);
                if px[3] > 10 {
                    canvas.put_pixel(12 + dx, 8 + dy, *px);
                }
            }
        }
    }

    // 5. Right Leg (4x12) at (4, 20)
    blit_part(&mut canvas, &skin_img, 4, 20, 4, 12, 4, 20);
    if h == 64 {
        blit_part(&mut canvas, &skin_img, 4, 36, 4, 12, 4, 20);
    }

    // 6. Left Leg (4x12) at (8, 20)
    if h == 64 {
        blit_part(&mut canvas, &skin_img, 20, 52, 4, 12, 8, 20);
        blit_part(&mut canvas, &skin_img, 4, 52, 4, 12, 8, 20);
    } else {
        // Legacy: Mirrored right leg
        for dy in 0..12 {
            for dx in 0..4 {
                let px = skin_img.get_pixel(4 + (3 - dx), 20 + dy);
                if px[3] > 10 {
                    canvas.put_pixel(8 + dx, 20 + dy, *px);
                }
            }
        }
    }

    // Upscale canvas nearest-neighbor by 10x to 160x320 for ultra-crisp display in the UI
    let scale = 10;
    let mut high_res = ImageBuffer::from_pixel(16 * scale, 32 * scale, Rgba([0, 0, 0, 0]));
    for y in 0..32 {
        for x in 0..16 {
            let px = canvas.get_pixel(x, y);
            if px[3] > 0 {
                for sy in 0..scale {
                    for sx in 0..scale {
                        high_res.put_pixel(x * scale + sx, y * scale + sy, *px);
                    }
                }
            }
        }
    }

    Ok(high_res)
}

/// Generates a preview image of a cape (100x160 px).
pub fn render_cape_preview<P: AsRef<Path>>(cape_path: P) -> Result<RgbaImage, SkinError> {
    let cape_img = image::open(cape_path.as_ref())?.to_rgba8();
    let (cw, ch) = cape_img.dimensions();

    // Standard Minecraft cape texture is 64x32. Back of cape is at x=1..11, y=1..17 (10x16)
    let src_x = if cw >= 64 { 1 } else { 0 };
    let src_y = if ch >= 32 { 1 } else { 0 };
    let src_w = if cw >= 64 { 10 } else { cw };
    let src_h = if ch >= 32 { 16 } else { ch };

    let scale = 10;
    let mut high_res = ImageBuffer::from_pixel(src_w * scale, src_h * scale, Rgba([0, 0, 0, 0]));

    for y in 0..src_h {
        for x in 0..src_w {
            let px = cape_img.get_pixel(src_x + x, src_y + y);
            for sy in 0..scale {
                for sx in 0..scale {
                    high_res.put_pixel(x * scale + sx, y * scale + sy, *px);
                }
            }
        }
    }

    Ok(high_res)
}

/// Generates the official DV Crimson Cape image bytes and saves it if not already generated.
pub fn ensure_preset_capes() -> PathBuf {
    let capes_dir = get_capes_dir();
    let dv_cape_path = capes_dir.join("dv_crimson.png");

    if !dv_cape_path.exists() {
        // Create 64x32 standard cape
        let mut cape: RgbaImage = ImageBuffer::from_pixel(64, 32, Rgba([0, 0, 0, 0]));

        // Fill cape body with Obsidian Black & Crimson Red accents
        for y in 1..17 {
            for x in 1..11 {
                if y == 1 || y == 16 || x == 1 || x == 10 {
                    // Border: Crimson Red #FF3344
                    cape.put_pixel(x, y, Rgba([255, 51, 68, 255]));
                } else if (x == 5 || x == 6) && (5..=12).contains(&y) {
                    // Central DV Emblem Spine
                    cape.put_pixel(x, y, Rgba([255, 77, 94, 255]));
                } else if (y == 8 || y == 9) && (3..=8).contains(&x) {
                    // Central DV Crossbar
                    cape.put_pixel(x, y, Rgba([255, 51, 68, 255]));
                } else {
                    // Deep Obsidian Void #121318
                    cape.put_pixel(x, y, Rgba([18, 19, 24, 255]));
                }
            }
        }

        let _ = cape.save(&dv_cape_path);
    }

    dv_cape_path
}

/// Generates a default Steve skin if no skin is available.
pub fn ensure_default_steve_skin() -> PathBuf {
    let skins_dir = get_skins_dir();
    let steve_path = skins_dir.join("default_steve.png");

    if !steve_path.exists() {
        let mut skin: RgbaImage = ImageBuffer::from_pixel(64, 64, Rgba([0, 0, 0, 0]));

        // Head (8x8 at 8,8) - Classic skin tone and brown hair
        for y in 8..16 {
            for x in 8..16 {
                if y <= 10 {
                    // Hair
                    skin.put_pixel(x, y, Rgba([75, 45, 20, 255]));
                } else if y == 12 && (x == 10 || x == 13) {
                    // Eyes
                    skin.put_pixel(x, y, Rgba([40, 60, 160, 255]));
                } else {
                    // Skin
                    skin.put_pixel(x, y, Rgba([190, 130, 90, 255]));
                }
            }
        }

        // Torso (8x12 at 20,20) - Cyan shirt
        for y in 20..32 {
            for x in 20..28 {
                skin.put_pixel(x, y, Rgba([0, 160, 180, 255]));
            }
        }

        // Arms (4x12 at 44,20 and 36,52)
        for y in 20..32 {
            for x in 44..48 {
                if y < 24 {
                    skin.put_pixel(x, y, Rgba([0, 160, 180, 255]));
                } else {
                    skin.put_pixel(x, y, Rgba([190, 130, 90, 255]));
                }
            }
        }
        for y in 52..64 {
            for x in 36..40 {
                if y < 56 {
                    skin.put_pixel(x, y, Rgba([0, 160, 180, 255]));
                } else {
                    skin.put_pixel(x, y, Rgba([190, 130, 90, 255]));
                }
            }
        }

        // Legs (4x12 at 4,20 and 20,52) - Blue jeans
        for y in 20..32 {
            for x in 4..8 {
                skin.put_pixel(x, y, Rgba([40, 45, 110, 255]));
            }
        }
        for y in 52..64 {
            for x in 20..24 {
                skin.put_pixel(x, y, Rgba([40, 45, 110, 255]));
            }
        }

        let _ = skin.save(&steve_path);
    }

    steve_path
}

/// Fetch a player skin preview for display in the launcher.
/// If user has a local custom skin uploaded, renders that.
/// Otherwise attempts to fetch from official avatar API, falling back to default Steve.
pub async fn get_or_fetch_player_skin(username: &str, is_microsoft: bool) -> PathBuf {
    let skins_dir = get_skins_dir();
    let preview_path = skins_dir.join(format!("{}_preview.png", username));
    let raw_custom_skin = skins_dir.join(format!("{}_raw.png", username));

    // 1. If custom skin uploaded, render and cache preview
    if raw_custom_skin.exists() {
        if let Ok(preview) = render_skin_preview(&raw_custom_skin) {
            let _ = preview.save(&preview_path);
            return preview_path;
        }
    }

    // 2. If preview already exists and is not empty, use it
    if preview_path.exists() && fs::metadata(&preview_path).map(|m| m.len() > 0).unwrap_or(false) {
        return preview_path;
    }

    // 3. For Microsoft or recognized usernames, fetch 3D body render from MC-Heads
    if is_microsoft || (!username.is_empty() && username != "DVPlayer" && username != "Player") {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(4))
            .build();

        if let Ok(c) = client {
            let url = format!("https://mc-heads.net/body/{}/right", username);
            if let Ok(resp) = c.get(&url).send().await {
                if resp.status().is_success() {
                    if let Ok(bytes) = resp.bytes().await {
                        if !bytes.is_empty() {
                            let _ = fs::write(&preview_path, bytes);
                            return preview_path;
                        }
                    }
                }
            }
        }
    }

    // 4. Default Steve fallback
    let steve_skin = ensure_default_steve_skin();
    if let Ok(preview) = render_skin_preview(&steve_skin) {
        let _ = preview.save(&preview_path);
        return preview_path;
    }

    steve_skin
}

/// Fetch a player cape preview for display.
pub fn get_or_create_cape_preview(username: &str, preset: Option<&str>) -> Option<PathBuf> {
    let capes_dir = get_capes_dir();
    let custom_cape = capes_dir.join(format!("{}_cape.png", username));
    let preview_path = capes_dir.join(format!("{}_cape_preview.png", username));

    let cape_file = if custom_cape.exists() {
        custom_cape
    } else {
        match preset.unwrap_or("crimson") {
            "crimson" | "default" => ensure_preset_capes(),
            "none" => return None,
            _ => ensure_preset_capes(),
        }
    };

    if let Ok(preview) = render_cape_preview(&cape_file) {
        let _ = preview.save(&preview_path);
        Some(preview_path)
    } else {
        None
    }
}

/// Save a newly uploaded custom skin file for the specified username.
pub fn save_custom_skin<P: AsRef<Path>>(username: &str, src_file: P) -> Result<PathBuf, SkinError> {
    let skins_dir = get_skins_dir();
    let raw_dest = skins_dir.join(format!("{}_raw.png", username));
    let preview_dest = skins_dir.join(format!("{}_preview.png", username));

    fs::copy(src_file.as_ref(), &raw_dest)?;

    // Verify and render preview
    let preview = render_skin_preview(&raw_dest)?;
    preview.save(&preview_dest)?;

    info!("Saved custom skin for player {}: {}", username, raw_dest.display());
    Ok(preview_dest)
}

/// Save a newly uploaded custom cape file for the specified username.
pub fn save_custom_cape<P: AsRef<Path>>(username: &str, src_file: P) -> Result<PathBuf, SkinError> {
    let capes_dir = get_capes_dir();
    let raw_dest = capes_dir.join(format!("{}_cape.png", username));
    let preview_dest = capes_dir.join(format!("{}_cape_preview.png", username));

    fs::copy(src_file.as_ref(), &raw_dest)?;

    let preview = render_cape_preview(&raw_dest)?;
    preview.save(&preview_dest)?;

    info!("Saved custom cape for player {}: {}", username, raw_dest.display());
    Ok(preview_dest)
}

/// Reset player skin back to default Steve/Alex.
pub fn reset_player_skin(username: &str) {
    let skins_dir = get_skins_dir();
    let raw_custom_skin = skins_dir.join(format!("{}_raw.png", username));
    let preview_path = skins_dir.join(format!("{}_preview.png", username));
    let _ = fs::remove_file(raw_custom_skin);
    let _ = fs::remove_file(preview_path);
}

/// Reset player cape back to none or default.
pub fn reset_player_cape(username: &str) {
    let capes_dir = get_capes_dir();
    let custom_cape = capes_dir.join(format!("{}_cape.png", username));
    let preview_path = capes_dir.join(format!("{}_cape_preview.png", username));
    let _ = fs::remove_file(custom_cape);
    let _ = fs::remove_file(preview_path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_default_steve_skin() {
        let steve = ensure_default_steve_skin();
        assert!(steve.exists());
        let img = image::open(&steve).expect("Steve skin should open");
        assert_eq!(img.width(), 64);
        assert_eq!(img.height(), 64);
    }

    #[test]
    fn test_render_skin_preview() {
        let steve = ensure_default_steve_skin();
        let preview = render_skin_preview(&steve).expect("Preview should render");
        assert_eq!(preview.width(), 160);
        assert_eq!(preview.height(), 320);
    }

    #[test]
    fn test_ensure_preset_cape() {
        let cape = ensure_preset_capes();
        assert!(cape.exists());
        let img = image::open(&cape).expect("DV Cape should open");
        assert_eq!(img.width(), 64);
        assert_eq!(img.height(), 32);

        let preview = render_cape_preview(&cape).expect("Cape preview should render");
        assert_eq!(preview.width(), 100);
        assert_eq!(preview.height(), 160);
    }

    #[test]
    fn test_invalid_skin_dimensions() {
        let temp_dir = tempfile::tempdir().unwrap();
        let invalid_path = temp_dir.path().join("invalid.png");
        let invalid_img: RgbaImage = ImageBuffer::from_pixel(30, 30, Rgba([255, 0, 0, 255]));
        invalid_img.save(&invalid_path).unwrap();

        let result = render_skin_preview(&invalid_path);
        assert!(result.is_err());
    }
}
