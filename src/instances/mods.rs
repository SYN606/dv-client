//! Mod management subsystem for Minecraft instances.
//!
//! Provides scanning, enabling/disabling (.jar <-> .jar.disabled), and deletion
//! of client-side mods in the instance's .minecraft/mods directory.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModEntry {
    pub file_name: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub file_size_bytes: u64,
    pub file_size_display: String,
    pub is_enabled: bool,
    pub icon_path: Option<PathBuf>,
}

/// Directory where extracted local mod icons and Modrinth mod logos are cached.
pub fn get_mod_icon_cache_dir() -> PathBuf {
    let cache_base = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("digitalvigital")
        .join("mod_icons");
    let _ = fs::create_dir_all(&cache_base);
    cache_base
}

fn save_validated_icon_png(buf: &[u8], cache_path: &Path) -> bool {
    if buf.len() >= 4 && buf[..4] == [0x89, 0x50, 0x4E, 0x47] {
        return fs::write(cache_path, buf).is_ok();
    }
    // Attempt decoding and converting other image formats (e.g. WebP, JPEG) to PNG
    if let Ok(img) = image::load_from_memory(buf) {
        return img.save_with_format(cache_path, image::ImageFormat::Png).is_ok();
    }
    false
}

fn extract_icon_from_archive<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    icon_candidate: Option<&str>,
    cache_path: &Path,
) -> Option<PathBuf> {
    if cache_path.exists() && fs::metadata(cache_path).map(|m| m.len() > 0).unwrap_or(false) {
        return Some(cache_path.to_path_buf());
    }

    // Try candidate path first if specified
    if let Some(candidate) = icon_candidate {
        let cleaned = candidate.trim_start_matches('/');
        if let Ok(mut f) = archive.by_name(cleaned) {
            let mut buf = Vec::new();
            if std::io::Read::read_to_end(&mut f, &mut buf).is_ok()
                && !buf.is_empty()
                && save_validated_icon_png(&buf, cache_path)
            {
                return Some(cache_path.to_path_buf());
            }
        }
    }

    // Common standard icon paths
    let common_paths = [
        "icon.png",
        "pack.png",
        "logo.png",
        "assets/icon.png",
        "assets/logo.png",
    ];

    for p in common_paths {
        if let Ok(mut f) = archive.by_name(p) {
            let mut buf = Vec::new();
            if std::io::Read::read_to_end(&mut f, &mut buf).is_ok()
                && !buf.is_empty()
                && save_validated_icon_png(&buf, cache_path)
            {
                return Some(cache_path.to_path_buf());
            }
        }
    }

    // Scan for any png in assets that ends with /icon.png or /logo.png
    let archive_len = archive.len();
    for i in 0..archive_len {
        if let Ok(mut f) = archive.by_index(i) {
            let name = f.name();
            if (name.ends_with("/icon.png") || name.ends_with("/logo.png")) && name.starts_with("assets/") {
                let mut buf = Vec::new();
                if std::io::Read::read_to_end(&mut f, &mut buf).is_ok()
                    && !buf.is_empty()
                    && save_validated_icon_png(&buf, cache_path)
                {
                    return Some(cache_path.to_path_buf());
                }
            }
        }
    }

    None
}

/// Inspect a .jar file to extract rich metadata and icon from fabric.mod.json, mcmod.info, or META-INF/mods.toml.
pub fn inspect_mod_jar<P: AsRef<Path>>(path: P) -> Option<(String, String, String, String, Option<PathBuf>)> {
    let file = fs::File::open(path.as_ref()).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    let file_stem = path.as_ref().file_stem().and_then(|s| s.to_str()).unwrap_or("mod");
    let clean_stem = file_stem.trim_end_matches(".jar").trim_end_matches(".disabled");
    let cache_path = get_mod_icon_cache_dir().join(format!("{}.png", clean_stem));

    // 1. Try fabric.mod.json
    let fabric_meta = {
        if let Ok(mut mod_json_file) = archive.by_name("fabric.mod.json") {
            let mut content = String::new();
            if std::io::Read::read_to_string(&mut mod_json_file, &mut content).is_ok() {
                serde_json::from_str::<serde_json::Value>(&content).ok()
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(val) = fabric_meta {
        let name = val.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
        let version = val.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let desc = val.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();
        let author = if let Some(authors) = val.get("authors").and_then(|a| a.as_array()) {
            authors
                .iter()
                .filter_map(|a| a.as_str().or_else(|| a.get("name").and_then(|n| n.as_str())))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            String::new()
        };

        let icon_candidate = val.get("icon").and_then(|i| {
            if let Some(s) = i.as_str() {
                Some(s.to_string())
            } else if let Some(obj) = i.as_object() {
                obj.values().filter_map(|v| v.as_str()).next().map(|s| s.to_string())
            } else {
                None
            }
        });

        let icon_path = extract_icon_from_archive(&mut archive, icon_candidate.as_deref(), &cache_path);

        if !name.is_empty() {
            return Some((name, version, desc, author, icon_path));
        }
    }

    // 2. Try mcmod.info (Forge legacy)
    let mcmod_meta = {
        if let Ok(mut mcmod_file) = archive.by_name("mcmod.info") {
            let mut content = String::new();
            if std::io::Read::read_to_string(&mut mcmod_file, &mut content).is_ok() {
                serde_json::from_str::<serde_json::Value>(&content).ok()
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(val) = mcmod_meta {
        let mod_obj = val.as_array().and_then(|arr| arr.first()).or(Some(&val));
        if let Some(m) = mod_obj {
            let name = m.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let version = m.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let desc = m.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();
            let author = m.get("authorList")
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(", "))
                .unwrap_or_default();
            let icon_candidate = m.get("logoFile").and_then(|l| l.as_str());
            let icon_path = extract_icon_from_archive(&mut archive, icon_candidate, &cache_path);

            if !name.is_empty() {
                return Some((name, version, desc, author, icon_path));
            }
        }
    }

    // 3. Fallback generic icon extraction
    let icon_path = extract_icon_from_archive(&mut archive, None, &cache_path);
    if icon_path.is_some() {
        return Some((String::new(), String::new(), String::new(), String::new(), icon_path));
    }

    None
}

/// Scan the mods directory of an instance and return all found mods (.jar and .jar.disabled).
pub fn scan_mods<P: AsRef<Path>>(mods_dir: P) -> Vec<ModEntry> {
    let dir = mods_dir.as_ref();
    let mut mods = Vec::new();

    if !dir.exists() {
        let _ = fs::create_dir_all(dir);
        return mods;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            let is_jar = file_name.ends_with(".jar");
            let is_disabled = file_name.ends_with(".jar.disabled") || file_name.ends_with(".disabled");

            if !is_jar && !is_disabled {
                continue;
            }

            let is_enabled = is_jar;
            let metadata = fs::metadata(&path).ok();
            let file_size_bytes = metadata.map(|m| m.len()).unwrap_or(0);
            let file_size_display = format_file_size(file_size_bytes);

            // Attempt to inspect deep metadata from inside the jar
            let (name, version, description, author, icon_path) = if let Some(meta) = inspect_mod_jar(&path) {
                if meta.0.is_empty() {
                    let base_name = if is_disabled {
                        file_name.trim_end_matches(".disabled").trim_end_matches(".jar")
                    } else {
                        file_name.trim_end_matches(".jar")
                    };
                    let (n, v) = parse_name_and_version(base_name);
                    (n, v, meta.2, meta.3, meta.4)
                } else {
                    meta
                }
            } else {
                let base_name = if is_disabled {
                    file_name.trim_end_matches(".disabled").trim_end_matches(".jar")
                } else {
                    file_name.trim_end_matches(".jar")
                };
                let (n, v) = parse_name_and_version(base_name);
                (n, v, String::new(), String::new(), None)
            };

            mods.push(ModEntry {
                file_name,
                name,
                version,
                description,
                author,
                file_size_bytes,
                file_size_display,
                is_enabled,
                icon_path,
            });
        }
    }

    // Sort alphabetically by name
    mods.sort_by_key(|a| a.name.to_lowercase());
    mods
}

/// Toggle a mod between enabled (.jar) and disabled (.jar.disabled).
pub fn toggle_mod<P: AsRef<Path>>(mods_dir: P, file_name: &str) -> std::io::Result<bool> {
    let dir = mods_dir.as_ref();
    let current_path = dir.join(file_name);

    if !current_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Mod file '{}' not found", file_name),
        ));
    }

    if file_name.ends_with(".jar") {
        let new_name = format!("{}.disabled", file_name);
        fs::rename(&current_path, dir.join(new_name))?;
        Ok(false) // now disabled
    } else if file_name.ends_with(".jar.disabled") {
        let new_name = file_name.trim_end_matches(".disabled");
        fs::rename(&current_path, dir.join(new_name))?;
        Ok(true) // now enabled
    } else if file_name.ends_with(".disabled") {
        let new_name = format!("{}.jar", file_name.trim_end_matches(".disabled"));
        fs::rename(&current_path, dir.join(new_name))?;
        Ok(true) // now enabled
    } else {
        Ok(false)
    }
}

/// Delete a mod from the mods folder.
pub fn delete_mod<P: AsRef<Path>>(mods_dir: P, file_name: &str) -> std::io::Result<()> {
    let target = mods_dir.as_ref().join(file_name);
    if target.exists() {
        fs::remove_file(target)?;
    }
    Ok(())
}

/// Import an external mod (.jar) file into the instance's mods directory.
pub fn import_mod_file<P: AsRef<Path>, Q: AsRef<Path>>(mods_dir: P, source_path: Q) -> std::io::Result<PathBuf> {
    let dir = mods_dir.as_ref();
    let src = source_path.as_ref();

    if !src.exists() || !src.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Source file does not exist",
        ));
    }

    fs::create_dir_all(dir)?;

    let file_name = src
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid file name"))?;

    let dest = dir.join(file_name);
    fs::copy(src, &dest)?;
    Ok(dest)
}

/// Open the instance mods folder in the native OS file explorer.
pub fn open_mods_folder<P: AsRef<Path>>(mods_dir: P) -> std::io::Result<()> {
    let dir = mods_dir.as_ref();
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }

    if cfg!(target_os = "windows") {
        let clean_dir = dunce::simplified(dir);
        std::process::Command::new("explorer").arg(clean_dir).spawn()?;
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(dir).spawn()?;
    } else {
        std::process::Command::new("xdg-open").arg(dir).spawn()?;
    }

    Ok(())
}

/// Ensure both instance_dir/mods and game_dir/mods exist and synchronize any mods placed in either.
/// Prefers instant NTFS/filesystem hard links to eliminate duplicate disk space usage.
pub fn sync_instance_mods<P: AsRef<Path>, Q: AsRef<Path>>(inst_dir: P, game_dir: Q) {
    let mods1 = inst_dir.as_ref().join("mods");
    let mods2 = game_dir.as_ref().join("mods");
    let _ = fs::create_dir_all(&mods1);
    let _ = fs::create_dir_all(&mods2);

    if let Ok(entries) = fs::read_dir(&mods1) {
        for entry in entries.flatten() {
            let target = mods2.join(entry.file_name());
            if !target.exists() && entry.path().is_file() {
                if fs::hard_link(entry.path(), &target).is_err() {
                    let _ = fs::copy(entry.path(), target);
                }
            }
        }
    }

    if let Ok(entries) = fs::read_dir(&mods2) {
        for entry in entries.flatten() {
            let target = mods1.join(entry.file_name());
            if !target.exists() && entry.path().is_file() {
                if fs::hard_link(entry.path(), &target).is_err() {
                    let _ = fs::copy(entry.path(), target);
                }
            }
        }
    }
}

fn format_file_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn parse_name_and_version(base_name: &str) -> (String, String) {
    if let Some(pos) = base_name.rfind('-') {
        let candidate_ver = &base_name[pos + 1..];
        // If candidate version starts with a digit or 'v'
        if candidate_ver.starts_with('v') || candidate_ver.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return (base_name[..pos].to_string(), candidate_ver.to_string());
        }
    }
    (base_name.to_string(), "latest".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_scan_and_toggle_mods() {
        let temp = tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        fs::create_dir_all(&mods_dir).unwrap();

        let mod1 = mods_dir.join("fabric-api-0.92.0+1.20.4.jar");
        fs::write(&mod1, b"mock mod 1").unwrap();

        let mod2 = mods_dir.join("sodium-fabric-0.5.8.jar.disabled");
        fs::write(&mod2, b"mock mod 2").unwrap();

        let scanned = scan_mods(&mods_dir);
        assert_eq!(scanned.len(), 2);

        let fabric = scanned.iter().find(|m| m.name == "fabric-api").unwrap();
        assert!(fabric.is_enabled);
        assert_eq!(fabric.version, "0.92.0+1.20.4");

        let sodium = scanned.iter().find(|m| m.name == "sodium-fabric").unwrap();
        assert!(!sodium.is_enabled);

        // Toggle sodium to enabled
        let now_enabled = toggle_mod(&mods_dir, "sodium-fabric-0.5.8.jar.disabled").unwrap();
        assert!(now_enabled);
        assert!(mods_dir.join("sodium-fabric-0.5.8.jar").exists());

        // Toggle fabric to disabled
        let now_disabled = toggle_mod(&mods_dir, "fabric-api-0.92.0+1.20.4.jar").unwrap();
        assert!(!now_disabled);
        assert!(mods_dir.join("fabric-api-0.92.0+1.20.4.jar.disabled").exists());
    }

    #[test]
    fn test_import_mod_file() {
        let temp = tempdir().unwrap();
        let mods_dir = temp.path().join("instance_mods");
        let external_mod = temp.path().join("external_mod.jar");
        fs::write(&external_mod, b"fake jar bytes").unwrap();

        let imported = import_mod_file(&mods_dir, &external_mod).unwrap();
        assert!(imported.exists());
        assert_eq!(imported.file_name().unwrap(), "external_mod.jar");
    }
}
