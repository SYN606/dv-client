// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let log_path = dirs::data_dir()
        .map(|d| d.join("DigitalVigital").join("startup.log"))
        .unwrap_or_else(|| std::path::PathBuf::from("dv_startup.log"));

    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let p_clone = log_path.clone();
    std::panic::set_hook(Box::new(move |info| {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&p_clone) {
            let _ = writeln!(f, "[CRITICAL PANIC] {}", info);
        }
    }));

    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).truncate(true).open(&log_path) {
        let _ = writeln!(f, "=== Digital Vigital Launcher Startup ===");
        let _ = writeln!(f, "1. Process main() entered");
    }

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
                let _ = writeln!(f, "[CRITICAL] Failed to build Tokio runtime: {}", e);
            }
            return;
        }
    };
    let _guard = rt.enter();

    app_lib::run();
}
