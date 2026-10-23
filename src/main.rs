#![cfg_attr(not(debug_assertions), windows_subsystem = "console")]

//! Digital Vigital (DV) - Native Minecraft Engine & CLI Runner
//!
//! Provides CLI diagnostics, instance management, and launcher inspection.
//! Desktop GUI is powered by Tauri v2 + Svelte in `frontend/src-tauri`.

use dv_launcher::branding;
use dv_launcher::config::ConfigManager;
use dv_launcher::gpu_detector::GpuDetector;
use dv_launcher::instances::InstanceManager;
use dv_launcher::java_detector::JavaDetector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("==================================================");
    println!("  {} v{}", branding::APP_NAME, branding::APP_VERSION);
    println!("  High-Performance Native Minecraft Launcher Engine");
    println!("==================================================");

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        return Ok(());
    }

    let config_mgr = match ConfigManager::new::<&str>(None) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            return Ok(());
        }
    };

    println!("\n[Configuration]");
    println!("  Data Base Path: {}", config_mgr.config.base_path().display());
    println!("  Instances Dir:  {}", config_mgr.config.instances_dir().display());

    let inst_mgr = match InstanceManager::new(config_mgr.config.instances_dir()) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error loading instances: {}", e);
            return Ok(());
        }
    };

    println!("\n[Profiles / Instances ({})]", inst_mgr.instances.len());
    for inst in inst_mgr.instances.values() {
        let active_tag = if config_mgr.config.active_instance_id.as_deref() == Some(&inst.id) {
            " [ACTIVE]"
        } else {
            ""
        };
        println!(
            "  * {}{} (Minecraft {}, Loader: {}, RAM: {}MB-{}MB)",
            inst.name,
            active_tag,
            inst.minecraft_version,
            inst.mod_loader,
            inst.min_memory_mb,
            inst.max_memory_mb
        );
    }

    println!("\n[Hardware & Runtimes]");
    let javas = JavaDetector::scan_system();
    println!("  Detected Java Installations: {}", javas.len());
    for j in &javas {
        println!("   - Java {} ({}) -> {}", j.major_version, j.vendor, j.path.display());
    }

    let gpus = GpuDetector::detect_gpus();
    println!("  Detected GPUs: {}", gpus.len());
    for g in &gpus {
        let disc = if g.is_discrete { "Discrete" } else { "Integrated" };
        println!("   - [{}] {}", disc, g.name);
    }

    println!("\nLaunch desktop GUI via: DigitalVigital-TauriV2-Portable.exe");
    println!("==================================================");

    Ok(())
}
