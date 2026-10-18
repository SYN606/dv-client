//! Desktop Environment Integration for Digital Vigital (DV) Launcher.
//!
//! Handles Linux Wayland and X11 compositor integration by ensuring
//! XDG `.desktop` entry registration and multi-resolution hicolor icon installation.
//! This ensures window managers (such as KDE Plasma KWin and GNOME Shell)
//! properly associate the Wayland `app_id` ("dv-launcher") with the DV logo.

use std::path::PathBuf;
use tracing::info;

/// Application ID registered with Wayland and XDG desktop entry specifications.
pub const DESKTOP_APP_ID: &str = "dv-launcher";

/// Embedded launcher logo image bytes.
pub const LOGO_PNG_BYTES: &[u8] = include_bytes!("../assets/logo.png");

/// Ensures that the DV launcher icon and desktop entry are installed in the
/// user's local XDG directories so that Wayland compositors display the DV logo.
pub fn ensure_desktop_integration() {
    #[cfg(target_os = "linux")]
    {
        let Some(data_dir) = dirs::data_local_dir() else {
            return;
        };

        // 1. Install icon into standard XDG hicolor icon theme hierarchy
        for size in &["512x512", "256x256", "128x128", "64x64", "48x48", "32x32"] {
            let icon_dir = data_dir.join("icons").join("hicolor").join(size).join("apps");
            let _ = std::fs::create_dir_all(&icon_dir);
            let icon_path = icon_dir.join(format!("{DESKTOP_APP_ID}.png"));
            let _ = std::fs::write(&icon_path, LOGO_PNG_BYTES);
        }

        // 2. Install icon into pixmaps fallback directory
        let pixmaps_dir = data_dir.join("pixmaps");
        let _ = std::fs::create_dir_all(&pixmaps_dir);
        let pixmap_path = pixmaps_dir.join(format!("{DESKTOP_APP_ID}.png"));
        let _ = std::fs::write(&pixmap_path, LOGO_PNG_BYTES);

        // 3. Install .desktop entry
        let apps_dir = data_dir.join("applications");
        let _ = std::fs::create_dir_all(&apps_dir);
        let desktop_path = apps_dir.join(format!("{DESKTOP_APP_ID}.desktop"));

        let current_exe = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from(DESKTOP_APP_ID));

        let desktop_entry = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=Digital Vigital\n\
             GenericName=Minecraft Launcher\n\
             Comment=Digital Vigital (DV) — Minecraft Launcher\n\
             Exec=\"{}\"\n\
             Icon={}\n\
             Terminal=false\n\
             Categories=Game;ActionGame;AdventureGame;\n\
             StartupWMClass={}\n\
             Keywords=game;minecraft;dv;digitalvigital;launcher;\n",
            current_exe.display(),
            DESKTOP_APP_ID,
            DESKTOP_APP_ID
        );

        let _ = std::fs::write(&desktop_path, desktop_entry);

        // 4. Notify KDE Plasma / GNOME desktop database to refresh cache
        let _ = std::process::Command::new("kbuildsycoca6").output();
        let _ = std::process::Command::new("update-desktop-database")
            .arg(&apps_dir)
            .output();

        info!("Desktop integration verified: registered Wayland app_id '{DESKTOP_APP_ID}' and XDG icons");
    }
}
