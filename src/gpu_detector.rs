//! GPU Hardware Detector and Graphics Preference Configuration.
//!
//! Detects integrated and discrete GPUs (NVIDIA, AMD, Intel) and provides
//! the required process environment variables for hardware offloading.

use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::info;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuKind {
    Discrete,
    Integrated,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuDevice {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub kind: GpuKind,
    pub is_discrete: bool,
}

pub struct GpuDetector;

impl GpuDetector {
    /// Detect all available system GPUs and return list with an "Auto" option at the front.
    pub fn detect_gpus() -> Vec<GpuDevice> {
        let mut devices = Vec::new();

        // 1. Always offer Automatic / OS Default
        devices.push(GpuDevice {
            id: "auto".to_string(),
            name: "Automatic (System Default)".to_string(),
            vendor: "System".to_string(),
            kind: GpuKind::Unknown,
            is_discrete: false,
        });

        // 2. Platform-specific GPU detection
        #[cfg(target_os = "linux")]
        {
            Self::detect_linux_gpus(&mut devices);
        }

        #[cfg(target_os = "windows")]
        {
            Self::detect_windows_gpus(&mut devices);
        }

        // If no hardware adapters were detected, offer sensible fallbacks
        if devices.len() == 1 {
            devices.push(GpuDevice {
                id: "nvidia-force".to_string(),
                name: "High-Performance NVIDIA GPU (Force Offload)".to_string(),
                vendor: "NVIDIA".to_string(),
                kind: GpuKind::Discrete,
                is_discrete: true,
            });
            devices.push(GpuDevice {
                id: "amd-force".to_string(),
                name: "High-Performance AMD GPU (DRI_PRIME=1)".to_string(),
                vendor: "AMD".to_string(),
                kind: GpuKind::Discrete,
                is_discrete: true,
            });
            devices.push(GpuDevice {
                id: "integrated-force".to_string(),
                name: "Power-Saving Integrated GPU".to_string(),
                vendor: "Integrated".to_string(),
                kind: GpuKind::Integrated,
                is_discrete: false,
            });
        }

        devices
    }

    #[cfg(target_os = "linux")]
    fn detect_linux_gpus(devices: &mut Vec<GpuDevice>) {
        // Method A: Check lspci for human-readable GPU names
        if let Ok(output) = Command::new("lspci").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("VGA compatible controller") || line.contains("3D controller") {
                    let parts: Vec<&str> = line.split(':').collect();
                    let desc = if parts.len() >= 3 {
                        parts[2..].join(":").trim().to_string()
                    } else {
                        line.trim().to_string()
                    };

                    let is_nvidia = desc.to_lowercase().contains("nvidia");
                    let is_amd = desc.to_lowercase().contains("amd")
                        || desc.to_lowercase().contains("radeon");
                    let is_intel = desc.to_lowercase().contains("intel");

                    let is_discrete = is_nvidia
                        || (is_amd && !desc.to_lowercase().contains("integrated"))
                        || line.contains("3D controller");

                    let vendor = if is_nvidia {
                        "NVIDIA"
                    } else if is_amd {
                        "AMD"
                    } else if is_intel {
                        "Intel"
                    } else {
                        "Graphics"
                    }
                    .to_string();

                    let kind = if is_discrete {
                        GpuKind::Discrete
                    } else if is_intel || desc.to_lowercase().contains("integrated") {
                        GpuKind::Integrated
                    } else {
                        GpuKind::Unknown
                    };

                    let suffix = match kind {
                        GpuKind::Discrete => " (Discrete High-Performance)",
                        GpuKind::Integrated => " (Integrated Power-Saving)",
                        GpuKind::Unknown => "",
                    };

                    let id = format!(
                        "{}-{}",
                        vendor.to_lowercase(),
                        if is_discrete {
                            "discrete"
                        } else {
                            "integrated"
                        }
                    );

                    let display_name = format!("{}{}", desc, suffix);

                    if !devices.iter().any(|d| d.name == display_name) {
                        info!("Detected Linux GPU: {}", display_name);
                        devices.push(GpuDevice {
                            id,
                            name: display_name,
                            vendor,
                            kind,
                            is_discrete,
                        });
                    }
                }
            }
        }

        // Method B: Fallback to /sys/class/drm/ if lspci is not installed or returned nothing
        if devices.len() == 1 {
            let drm_path = std::path::Path::new("/sys/class/drm");
            if let Ok(entries) = std::fs::read_dir(drm_path) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with("card") && !file_name.contains('-') {
                        let vendor_file = entry.path().join("device/vendor");
                        if let Ok(vendor_hex) = std::fs::read_to_string(vendor_file) {
                            let vendor_clean = vendor_hex.trim().to_lowercase();
                            let (vendor, is_discrete, id) = if vendor_clean.contains("0x10de") {
                                ("NVIDIA", true, "nvidia-discrete")
                            } else if vendor_clean.contains("0x1002") {
                                ("AMD", true, "amd-discrete")
                            } else if vendor_clean.contains("0x8086") {
                                ("Intel", false, "intel-integrated")
                            } else {
                                ("Generic", false, "generic-gpu")
                            };

                            let name = format!(
                                "{} GPU ({})",
                                vendor,
                                if is_discrete {
                                    "Discrete High-Performance"
                                } else {
                                    "Integrated Power-Saving"
                                }
                            );

                            if !devices.iter().any(|d| d.id == id) {
                                devices.push(GpuDevice {
                                    id: id.to_string(),
                                    name,
                                    vendor: vendor.to_string(),
                                    kind: if is_discrete {
                                        GpuKind::Discrete
                                    } else {
                                        GpuKind::Integrated
                                    },
                                    is_discrete,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    /// Classify and parse a raw GPU device name string into a structured GpuDevice.
    pub fn parse_gpu_device(raw_name: &str) -> Option<GpuDevice> {
        let name = raw_name.trim();
        if name.is_empty() || name.eq_ignore_ascii_case("Name") || name.contains("DriverDesc") {
            return None;
        }

        let lower = name.to_lowercase();
        // Ignore remote display, virtual adapters, or basic display drivers
        if lower.contains("remote")
            || lower.contains("virtual")
            || lower.contains("basic display")
            || lower.contains("miracast")
        {
            return None;
        }

        let is_nvidia =
            lower.contains("nvidia") || lower.contains("geforce") || lower.contains("quadro");
        let is_amd = lower.contains("amd") || lower.contains("radeon");
        let is_intel = lower.contains("intel");

        // Intel Arc are discrete graphics cards; HD/UHD/Iris are integrated
        let is_intel_discrete =
            lower.contains("arc ") || lower.contains("arc(tm)") || lower.contains("arc a");
        let is_amd_integrated = lower.contains("vega")
            || lower.contains("integrated")
            || (lower.contains("graphics") && !lower.contains("rx") && !lower.contains("xt"));

        let is_discrete = is_nvidia || is_intel_discrete || (is_amd && !is_amd_integrated);

        let vendor = if is_nvidia {
            "NVIDIA"
        } else if is_amd {
            "AMD"
        } else if is_intel {
            "Intel"
        } else {
            "Graphics"
        };

        let kind = if is_discrete {
            GpuKind::Discrete
        } else {
            GpuKind::Integrated
        };

        let suffix = if is_discrete {
            " (Discrete High-Performance)"
        } else {
            " (Integrated Power-Saving)"
        };

        let id = format!(
            "{}-{}",
            vendor.to_lowercase(),
            if is_discrete {
                "discrete"
            } else {
                "integrated"
            }
        );

        Some(GpuDevice {
            id,
            name: format!("{}{}", name, suffix),
            vendor: vendor.to_string(),
            kind,
            is_discrete,
        })
    }

    #[cfg(target_os = "windows")]
    fn detect_windows_gpus(devices: &mut Vec<GpuDevice>) {
        use std::os::windows::process::CommandExt;
        let mut raw_names: Vec<String> = Vec::new();

        // 1. Primary Method: Instant Win32 EnumDisplayDevicesW (0ms latency, zero process spawning)
        #[repr(C)]
        #[allow(non_snake_case)]
        struct DISPLAY_DEVICEW {
            cb: u32,
            DeviceName: [u16; 32],
            DeviceString: [u16; 128],
            StateFlags: u32,
            DeviceID: [u16; 128],
            DeviceKey: [u16; 128],
        }

        #[link(name = "user32")]
        extern "system" {
            fn EnumDisplayDevicesW(
                lpDevice: *const u16,
                iDevNum: u32,
                lpDisplayDevice: *mut DISPLAY_DEVICEW,
                dwFlags: u32,
            ) -> i32;
        }

        let mut i = 0u32;
        loop {
            let mut dev: DISPLAY_DEVICEW = unsafe { std::mem::zeroed() };
            dev.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
            let success = unsafe { EnumDisplayDevicesW(std::ptr::null(), i, &mut dev, 0) };
            if success == 0 || i > 32 {
                break;
            }
            // StateFlags & 0x8 is DISPLAY_DEVICE_MIRRORING_DRIVER (virtual mirror drivers)
            if (dev.StateFlags & 8) == 0 {
                let len = dev.DeviceString.iter().position(|&c| c == 0).unwrap_or(128);
                let name = String::from_utf16_lossy(&dev.DeviceString[..len])
                    .trim()
                    .to_string();
                if !name.is_empty() && !raw_names.contains(&name) {
                    raw_names.push(name);
                }
            }
            i += 1;
        }

        // 2. Fallback Method: Windows Registry Display Class query
        if raw_names.is_empty() {
            let mut reg_cmd = Command::new("reg");
            reg_cmd.args([
                "query",
                r"HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}",
                "/s",
                "/v",
                "DriverDesc",
            ]);
            reg_cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            if let Ok(output) = reg_cmd.output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        if line.contains("DriverDesc") && line.contains("REG_SZ") {
                            if let Some(desc) = line.split("REG_SZ").nth(1) {
                                let trimmed = desc.trim();
                                if !trimmed.is_empty() {
                                    raw_names.push(trimmed.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Parse and deduplicate found devices
        for raw in raw_names {
            if let Some(device) = Self::parse_gpu_device(&raw) {
                if !devices.iter().any(|d| d.name == device.name) {
                    info!("Detected Windows GPU: {}", device.name);
                    devices.push(device);
                }
            }
        }
    }

    /// On Windows 10/11, register the Java executable with the DirectX graphics scheduler
    /// in HKCU\Software\Microsoft\DirectX\UserGpuPreferences so that the OS GPU scheduler
    /// automatically routes Minecraft to the discrete or integrated GPU.
    pub fn configure_windows_gpu_preference<P: AsRef<std::path::Path>>(java_exe: P, gpu_id: &str) {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let path = java_exe.as_ref();
            let parent = path.parent();

            // Collect both java.exe and javaw.exe so Windows DirectX scheduler covers both
            let mut paths_to_register = vec![dunce::simplified(path).to_string_lossy().to_string()];
            if let Some(p) = parent {
                for bin in ["javaw.exe", "java.exe"] {
                    let candidate = p.join(bin);
                    if candidate.exists() {
                        let c_str = dunce::simplified(&candidate).to_string_lossy().to_string();
                        if !paths_to_register.contains(&c_str) {
                            paths_to_register.push(c_str);
                        }
                    }
                }
            }

            let id_lower = gpu_id.to_lowercase();
            let is_forced_integrated = id_lower.contains("integrated") || id_lower.contains("intel-integrated");

            let use_discrete = if is_forced_integrated {
                false
            } else if id_lower.contains("discrete") || id_lower.contains("nvidia") || id_lower.contains("amd") {
                true
            } else {
                // "auto" mode: inspect hardware GPUs and route to High Performance if discrete exists
                let detected = Self::detect_gpus();
                detected.iter().any(|d| d.is_discrete)
            };

            let pref_val = if use_discrete {
                "GpuPreference=2;" // High Performance (Discrete GPU)
            } else {
                "GpuPreference=1;" // Power Saving (Integrated GPU)
            };

            for p_str in paths_to_register {
                let mut reg_cmd = Command::new("reg");
                reg_cmd.args([
                    "add",
                    r"HKCU\Software\Microsoft\DirectX\UserGpuPreferences",
                    "/v",
                    &p_str,
                    "/t",
                    "REG_SZ",
                    "/d",
                    pref_val,
                    "/f",
                ]);
                reg_cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
                let _ = reg_cmd.output();
                info!(
                    "Registered Windows DirectX GPU preference: {} -> {}",
                    p_str, pref_val
                );
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (java_exe, gpu_id);
        }
    }

    /// Retrieve the necessary environment variables for the selected GPU preference.
    pub fn get_env_vars_for_gpu(gpu_id: &str) -> Vec<(String, String)> {
        let mut vars = Vec::new();
        let id_lower = gpu_id.to_lowercase();

        let (is_nvidia, is_amd) = if id_lower == "auto" {
            let detected = Self::detect_gpus();
            let has_nv = detected
                .iter()
                .any(|d| d.is_discrete && d.vendor.to_lowercase().contains("nvidia"));
            let has_amd = detected
                .iter()
                .any(|d| d.is_discrete && d.vendor.to_lowercase().contains("amd"));
            (has_nv, has_amd)
        } else {
            (
                id_lower.contains("nvidia") || id_lower.contains("discrete"),
                id_lower.contains("amd"),
            )
        };

        if is_nvidia {
            // Linux NVIDIA PRIME offloading
            vars.push(("__NV_PRIME_RENDER_OFFLOAD".to_string(), "1".to_string()));
            vars.push((
                "__GLX_VENDOR_LIBRARY_NAME".to_string(),
                "nvidia".to_string(),
            ));
            vars.push((
                "__VK_LAYER_NV_optimus".to_string(),
                "NVIDIA_only".to_string(),
            ));
            // Windows High-Performance hybrid graphics scheduler hint
            vars.push(("SHIM_MCCOMPAT".to_string(), "0x800000001".to_string()));
        } else if is_amd {
            // Linux DRI PRIME for dedicated AMD cards
            vars.push(("DRI_PRIME".to_string(), "1".to_string()));
            // Windows AMD Radeon Driver performance optimizations
            vars.push(("GPU_MAX_ALLOC_PERCENT".to_string(), "100".to_string()));
            vars.push(("GPU_USE_SYNC_OBJECTS".to_string(), "1".to_string()));
            vars.push(("GPU_SINGLE_ALLOC_PERCENT".to_string(), "100".to_string()));
            vars.push(("GPU_MAX_HEAP_SIZE".to_string(), "100".to_string()));
        } else if id_lower.contains("integrated") || id_lower.contains("intel") {
            // Force integrated power-saving
            vars.push(("__NV_PRIME_RENDER_OFFLOAD".to_string(), "0".to_string()));
            vars.push(("DRI_PRIME".to_string(), "0".to_string()));
            vars.push(("SHIM_MCCOMPAT".to_string(), "0x000000000".to_string()));
        }

        vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_gpus_includes_auto() {
        let gpus = GpuDetector::detect_gpus();
        assert!(!gpus.is_empty());
        assert_eq!(gpus[0].id, "auto");
    }

    #[test]
    fn test_nvidia_gpu_env_vars() {
        let vars = GpuDetector::get_env_vars_for_gpu("nvidia-discrete");
        assert!(vars
            .iter()
            .any(|(k, v)| k == "__NV_PRIME_RENDER_OFFLOAD" && v == "1"));
        assert!(vars
            .iter()
            .any(|(k, v)| k == "__GLX_VENDOR_LIBRARY_NAME" && v == "nvidia"));
    }

    #[test]
    fn test_amd_gpu_env_vars() {
        let vars = GpuDetector::get_env_vars_for_gpu("amd-discrete");
        assert!(vars.iter().any(|(k, v)| k == "DRI_PRIME" && v == "1"));
    }

    #[test]
    fn test_integrated_gpu_env_vars() {
        let vars = GpuDetector::get_env_vars_for_gpu("intel-integrated");
        assert!(vars
            .iter()
            .any(|(k, v)| k == "__NV_PRIME_RENDER_OFFLOAD" && v == "0"));
    }

    #[test]
    fn test_auto_gpu_env_vars_empty() {
        let vars = GpuDetector::get_env_vars_for_gpu("auto");
        assert!(vars.is_empty());
    }

    #[test]
    fn test_parse_gpu_devices() {
        let nv = GpuDetector::parse_gpu_device("NVIDIA GeForce RTX 4080").unwrap();
        assert_eq!(nv.vendor, "NVIDIA");
        assert!(nv.is_discrete);
        assert_eq!(nv.kind, GpuKind::Discrete);

        let intel_igpu = GpuDetector::parse_gpu_device("Intel(R) Iris(R) Xe Graphics").unwrap();
        assert_eq!(intel_igpu.vendor, "Intel");
        assert!(!intel_igpu.is_discrete);
        assert_eq!(intel_igpu.kind, GpuKind::Integrated);

        let intel_arc = GpuDetector::parse_gpu_device("Intel Arc A770 Graphics").unwrap();
        assert_eq!(intel_arc.vendor, "Intel");
        assert!(intel_arc.is_discrete);

        let amd_discrete = GpuDetector::parse_gpu_device("AMD Radeon RX 7900 XTX").unwrap();
        assert_eq!(amd_discrete.vendor, "AMD");
        assert!(amd_discrete.is_discrete);

        let amd_igpu = GpuDetector::parse_gpu_device("AMD Radeon(TM) Vega 8 Graphics").unwrap();
        assert_eq!(amd_igpu.vendor, "AMD");
        assert!(!amd_igpu.is_discrete);

        assert!(GpuDetector::parse_gpu_device("Microsoft Basic Display Adapter").is_none());
        assert!(GpuDetector::parse_gpu_device("").is_none());
    }
}
