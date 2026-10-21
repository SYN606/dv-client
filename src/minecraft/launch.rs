//! Minecraft JVM & Game Execution Argument Builder.
//!
//! Assembles classpath entries, JVM arguments, and interpolates version-specific
//! game parameters for modern and legacy versions.

use super::manifest::{evaluate_rules, VersionPackage};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct LaunchContext {
    pub version_package: VersionPackage,
    pub instance_dir: PathBuf,
    pub game_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub libraries_dir: PathBuf,
    pub client_jar_path: PathBuf,
    pub java_executable: PathBuf,

    pub player_name: String,
    pub player_uuid: String,
    pub access_token: String,
    pub user_type: String,
    pub xuid: String,

    pub min_memory_mb: u32,
    pub max_memory_mb: u32,
    pub window_width: u32,
    pub window_height: u32,
    pub custom_jvm_args: Option<Vec<String>>,
    pub performance_preset: Option<String>,

    pub mod_loader: String,
    pub extra_classpath: Vec<PathBuf>,
    pub main_class_override: Option<String>,
    pub extra_game_args: Vec<String>,
}

/// Returns GC and performance optimization flags based on the chosen preset.
pub fn get_performance_jvm_flags(preset: &str) -> Vec<String> {
    match preset.to_lowercase().as_str() {
        "aikar" => vec![
            "-XX:+UseG1GC".to_string(),
            "-XX:+ParallelRefProcEnabled".to_string(),
            "-XX:MaxGCPauseMillis=200".to_string(),
            "-XX:+UnlockExperimentalVMOptions".to_string(),
            "-XX:+DisableExplicitGC".to_string(),
            "-XX:+AlwaysPreTouch".to_string(),
            "-XX:G1NewSizePercent=30".to_string(),
            "-XX:G1MaxNewSizePercent=40".to_string(),
            "-XX:G1ReservePercent=20".to_string(),
            "-XX:G1HeapWastePercent=5".to_string(),
            "-XX:G1MixedGCCountTarget=4".to_string(),
            "-XX:InitiatingHeapOccupancyPercent=15".to_string(),
            "-XX:G1MixedGCLiveThresholdPercent=90".to_string(),
            "-XX:G1RSetUpdatingPauseTimePercent=5".to_string(),
            "-XX:SurvivorRatio=32".to_string(),
            "-XX:+PerfDisableSharedMem".to_string(),
            "-XX:MaxTenuringThreshold=1".to_string(),
        ],
        "shenandoah" => vec![
            "-XX:+UnlockExperimentalVMOptions".to_string(),
            "-XX:+UseShenandoahGC".to_string(),
            "-XX:ShenandoahGCMode=normal".to_string(),
            "-XX:+AlwaysPreTouch".to_string(),
            "-XX:+DisableExplicitGC".to_string(),
        ],
        "zgc" => vec![
            "-XX:+UnlockExperimentalVMOptions".to_string(),
            "-XX:+UseZGC".to_string(),
            "-XX:+ZGenerational".to_string(),
            "-XX:+AlwaysPreTouch".to_string(),
            "-XX:+DisableExplicitGC".to_string(),
        ],
        _ => Vec::new(),
    }
}

pub struct LaunchArgumentBuilder;

impl LaunchArgumentBuilder {
    /// Compose the full classpath including all applicable libraries, mod loader JARs, and the game client JAR.
    pub fn build_classpath(ctx: &LaunchContext) -> String {
        let mut cp_elements: Vec<PathBuf> = Vec::new();

        // 1. Extra loader libraries (e.g. Fabric, KnotClient, Mixin, ASM) prepended first
        for extra in &ctx.extra_classpath {
            if extra.exists() && !cp_elements.contains(extra) {
                cp_elements.push(extra.clone());
            }
        }

        // 2. Standard game libraries
        for lib in &ctx.version_package.libraries {
            if !evaluate_rules(lib.rules.as_deref()) {
                continue;
            }

            if let Some(ref downloads) = lib.downloads {
                if let Some(ref art) = downloads.artifact {
                    let rel_path_opt = art
                        .path
                        .as_deref()
                        .or_else(|| art.url.split("libraries.minecraft.net/").nth(1));

                    if let Some(rel_path) = rel_path_opt {
                        let path = ctx.libraries_dir.join(rel_path);
                        if path.exists() && !cp_elements.contains(&path) {
                            cp_elements.push(path);
                        }
                    }
                }
            } else {
                // Maven coordinate resolution (e.g. org.ow2.asm:asm:9.6)
                let parts: Vec<&str> = lib.name.split(':').collect();
                if parts.len() >= 3 {
                    let group_path = parts[0].replace('.', "/");
                    let artifact = parts[1];
                    let version = parts[2];
                    let jar_name = format!("{}-{}.jar", artifact, version);
                    let path = ctx
                        .libraries_dir
                        .join(group_path)
                        .join(artifact)
                        .join(version)
                        .join(jar_name);
                    if path.exists() && !cp_elements.contains(&path) {
                        cp_elements.push(path);
                    }
                }
            }
        }

        // 3. Append client JAR last
        if !cp_elements.contains(&ctx.client_jar_path) {
            cp_elements.push(ctx.client_jar_path.clone());
        }

        let sep = if cfg!(target_os = "windows") {
            ";"
        } else {
            ":"
        };
        let paths_str: Vec<String> = cp_elements
            .into_iter()
            .map(|p| dunce::simplified(&p).to_string_lossy().to_string())
            .collect();
        paths_str.join(sep)
    }

    /// Assemble the unquoted command argument array for direct process spawning.
    pub fn build_command_args(ctx: &LaunchContext) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();

        // 1. Java executable binary
        args.push(
            dunce::simplified(&ctx.java_executable)
                .to_string_lossy()
                .to_string(),
        );

        // 2. Memory allocations
        args.push(format!("-Xms{}M", ctx.min_memory_mb));
        args.push(format!("-Xmx{}M", ctx.max_memory_mb));

        // 3. Performance preset flags
        if let Some(ref preset) = ctx.performance_preset {
            args.extend(get_performance_jvm_flags(preset));
        }

        // 4. Custom user JVM flags
        if let Some(ref custom_args) = ctx.custom_jvm_args {
            args.extend(custom_args.clone());
        }

        let natives_dir = dunce::simplified(&ctx.instance_dir.join("natives")).to_path_buf();
        let classpath = Self::build_classpath(ctx);

        // 5. Modern Mojang arguments.jvm vs Legacy fallback
        let mut has_classpath_in_jvm = false;

        if let Some(jvm_args_val) = ctx
            .version_package
            .arguments
            .as_ref()
            .and_then(|a| a.get("jvm"))
        {
            if let Some(jvm_args_arr) = jvm_args_val.as_array() {
                for item in jvm_args_arr {
                    if let Some(arg_str) = item.as_str() {
                        let substituted = substitute_jvm_arg(
                            arg_str,
                            &natives_dir,
                            &classpath,
                            &ctx.libraries_dir,
                        );
                        if substituted.contains(&classpath) || arg_str == "-cp" {
                            has_classpath_in_jvm = true;
                        }
                        args.push(substituted);
                    } else if let Some(obj) = item.as_object() {
                        if evaluate_argument_rules(obj) {
                            if let Some(val_str) = obj.get("value").and_then(|v| v.as_str()) {
                                let substituted = substitute_jvm_arg(
                                    val_str,
                                    &natives_dir,
                                    &classpath,
                                    &ctx.libraries_dir,
                                );
                                if substituted.contains(&classpath) || val_str == "-cp" {
                                    has_classpath_in_jvm = true;
                                }
                                args.push(substituted);
                            } else if let Some(val_arr) =
                                obj.get("value").and_then(|v| v.as_array())
                            {
                                for v_item in val_arr {
                                    if let Some(v_str) = v_item.as_str() {
                                        let substituted = substitute_jvm_arg(
                                            v_str,
                                            &natives_dir,
                                            &classpath,
                                            &ctx.libraries_dir,
                                        );
                                        if substituted.contains(&classpath) || v_str == "-cp" {
                                            has_classpath_in_jvm = true;
                                        }
                                        args.push(substituted);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback for versions without arguments.jvm (1.12.2 and older) or incomplete manifests
        if !has_classpath_in_jvm {
            if !args.iter().any(|a| a.starts_with("-Djava.library.path=")) {
                args.push(format!(
                    "-Djava.library.path={}",
                    natives_dir.to_string_lossy()
                ));
            }
            if !args
                .iter()
                .any(|a| a.starts_with("-Dminecraft.launcher.brand="))
            {
                args.push("-Dminecraft.launcher.brand=DigitalVigital".to_string());
                args.push(format!(
                    "-Dminecraft.launcher.version={}",
                    crate::branding::APP_VERSION
                ));
            }
            args.push("-cp".to_string());
            args.push(classpath);
        }

        // macOS GLFW thread initialization requirement
        #[cfg(target_os = "macos")]
        if !args.iter().any(|a| a == "-XstartOnFirstThread") {
            args.push("-XstartOnFirstThread".to_string());
        }

        // 6. Main Class (supports mod loaders like KnotClient or FML)
        let main_class = ctx
            .main_class_override
            .as_deref()
            .unwrap_or(&ctx.version_package.main_class);
        args.push(main_class.to_string());

        // 8. Placeholder substitution map
        let mut substitutions: HashMap<&str, String> = HashMap::new();
        substitutions.insert("${auth_player_name}", ctx.player_name.clone());
        substitutions.insert("${version_name}", ctx.version_package.id.clone());
        substitutions.insert(
            "${game_directory}",
            dunce::simplified(&ctx.game_dir)
                .to_string_lossy()
                .to_string(),
        );
        substitutions.insert(
            "${assets_root}",
            dunce::simplified(&ctx.assets_dir)
                .to_string_lossy()
                .to_string(),
        );
        substitutions.insert(
            "${assets_index_name}",
            ctx.version_package.asset_index.id.clone(),
        );
        substitutions.insert("${auth_uuid}", ctx.player_uuid.clone());
        substitutions.insert("${auth_access_token}", ctx.access_token.clone());
        substitutions.insert("${user_type}", ctx.user_type.clone());
        substitutions.insert("${version_type}", ctx.version_package.version_type.clone());
        substitutions.insert(
            "${auth_xuid}",
            if ctx.xuid.is_empty() {
                "0".to_string()
            } else {
                ctx.xuid.clone()
            },
        );
        substitutions.insert("${clientid}", "DigitalVigital".to_string());
        substitutions.insert("${user_properties}", "{}".to_string());

        // 9. Modern arguments object
        let mut raw_game_args: Vec<String> = Vec::new();
        if let Some(ref arguments) = ctx.version_package.arguments {
            if let Some(game_args_val) = arguments.get("game") {
                if let Some(game_args) = game_args_val.as_array() {
                    for item in game_args {
                        if let Some(arg_str) = item.as_str() {
                            let mut val = arg_str.to_string();
                            for (key, replacement) in &substitutions {
                                val = val.replace(key, replacement);
                            }
                            raw_game_args.push(val);
                        } else if let Some(obj) = item.as_object() {
                            if evaluate_argument_rules(obj) {
                                if let Some(val_str) = obj.get("value").and_then(|v| v.as_str()) {
                                    let mut val = val_str.to_string();
                                    for (key, replacement) in &substitutions {
                                        val = val.replace(key, replacement);
                                    }
                                    raw_game_args.push(val);
                                } else if let Some(val_arr) =
                                    obj.get("value").and_then(|v| v.as_array())
                                {
                                    for v_item in val_arr {
                                        if let Some(v_str) = v_item.as_str() {
                                            let mut val = v_str.to_string();
                                            for (key, replacement) in &substitutions {
                                                val = val.replace(key, replacement);
                                            }
                                            raw_game_args.push(val);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else if let Some(ref mc_args) = ctx.version_package.minecraft_arguments {
            // Legacy space-separated arguments string
            for part in mc_args.split_whitespace() {
                let mut val = part.to_string();
                for (key, replacement) in &substitutions {
                    val = val.replace(key, replacement);
                }
                raw_game_args.push(val);
            }
        }

        // 10. Extra game args (e.g. loader tweak classes)
        for extra in &ctx.extra_game_args {
            raw_game_args.push(extra.clone());
        }

        // 11. Ensure --gameDir is unconditionally present so mods/saves load correctly
        if !raw_game_args.iter().any(|a| a == "--gameDir") {
            raw_game_args.push("--gameDir".to_string());
            raw_game_args.push(ctx.game_dir.to_string_lossy().to_string());
        }

        // 12. Resolution dimensions
        if ctx.window_width > 0 && ctx.window_height > 0 {
            if !raw_game_args.iter().any(|a| a == "--width") {
                raw_game_args.push("--width".to_string());
                raw_game_args.push(ctx.window_width.to_string());
            }
            if !raw_game_args.iter().any(|a| a == "--height") {
                raw_game_args.push("--height".to_string());
                raw_game_args.push(ctx.window_height.to_string());
            }
        }

        // 13. Sanitize game arguments: strip any duplicate/unsupported quick play flags and unreplaced template variables
        let clean_game_args = sanitize_game_arguments(raw_game_args);
        args.extend(clean_game_args);

        args
    }
}

/// Substitute Mojang template placeholders in JVM arguments (${natives_directory}, ${classpath}, etc.).
fn substitute_jvm_arg(
    arg: &str,
    natives_dir: &std::path::Path,
    classpath: &str,
    libraries_dir: &std::path::Path,
) -> String {
    let sep = if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    };
    arg.replace(
        "${natives_directory}",
        &dunce::simplified(natives_dir).to_string_lossy(),
    )
    .replace("${launcher_name}", "DigitalVigital")
    .replace("${launcher_version}", crate::branding::APP_VERSION)
    .replace("${classpath}", classpath)
    .replace("${classpath_separator}", sep)
    .replace(
        "${library_directory}",
        &dunce::simplified(libraries_dir).to_string_lossy(),
    )
}

/// Evaluates conditional rules on an argument object from Mojang version JSON.
fn evaluate_argument_rules(obj: &serde_json::Map<String, serde_json::Value>) -> bool {
    if let Some(rules_arr) = obj.get("rules").and_then(|r| r.as_array()) {
        for rule in rules_arr {
            // Check feature requirements (e.g. is_quick_play_*, is_demo_user)
            if let Some(features) = rule.get("features").and_then(|f| f.as_object()) {
                for (_feature_name, enabled) in features {
                    if enabled.as_bool() == Some(true) {
                        return false;
                    }
                }
            }
            if let Some(os) = rule.get("os").and_then(|o| o.as_object()) {
                let current_os = if cfg!(target_os = "windows") {
                    "windows"
                } else if cfg!(target_os = "macos") {
                    "osx"
                } else {
                    "linux"
                };

                let current_arch = if cfg!(target_arch = "x86_64") {
                    "x86_64"
                } else if cfg!(target_arch = "aarch64") {
                    "arm64"
                } else if cfg!(target_arch = "x86") {
                    "x86"
                } else {
                    "unknown"
                };

                let mut matches_rule = true;

                if let Some(os_name) = os.get("name").and_then(|n| n.as_str()) {
                    if os_name != current_os {
                        matches_rule = false;
                    }
                }

                if let Some(arch) = os.get("arch").and_then(|a| a.as_str()) {
                    let arch_matches = match arch {
                        "x86_64" | "amd64" => current_arch == "x86_64",
                        "arm64" | "aarch64" => current_arch == "arm64",
                        "x86" | "i386" | "i686" => current_arch == "x86",
                        pattern => {
                            if let Ok(re) = regex::Regex::new(pattern) {
                                re.is_match(current_arch)
                            } else {
                                pattern.eq_ignore_ascii_case(current_arch)
                            }
                        }
                    };
                    if !arch_matches {
                        matches_rule = false;
                    }
                }

                if let Some(action) = rule.get("action").and_then(|a| a.as_str()) {
                    if action == "allow" && !matches_rule {
                        return false;
                    }
                    if action == "disallow" && matches_rule {
                        return false;
                    }
                }
            }
        }
    }
    true
}

/// Strip conflicting quick play flags and un-substituted template placeholders.
fn sanitize_game_arguments(raw_args: Vec<String>) -> Vec<String> {
    let quick_play_flags = [
        "--quickPlaySingleplayer",
        "--quickPlayMultiplayer",
        "--quickPlayRealms",
        "--quickPlayPath",
    ];

    let mut clean_args = Vec::new();
    let mut skip_next = false;

    for arg in raw_args {
        if skip_next {
            skip_next = false;
            continue;
        }

        // If it's one of the quick play flags, skip it and its parameter
        if quick_play_flags.iter().any(|&f| f == arg) {
            skip_next = true;
            continue;
        }

        // Skip un-interpolated template placeholders
        if arg.starts_with("${") && arg.ends_with('}') {
            continue;
        }

        clean_args.push(arg);
    }

    clean_args
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minecraft::manifest::{AssetIndexRef, DownloadArtifact, VersionDownloads};
    use tempfile::tempdir;

    #[test]
    fn test_launch_argument_builder_modern() {
        let temp = tempdir().unwrap();
        let client_jar = temp.path().join("1.21.4.jar");
        std::fs::write(&client_jar, b"mock client jar").unwrap();

        let game_args = vec![
            serde_json::Value::String("--username".to_string()),
            serde_json::Value::String("${auth_player_name}".to_string()),
            serde_json::Value::String("--version".to_string()),
            serde_json::Value::String("${version_name}".to_string()),
            serde_json::Value::String("--uuid".to_string()),
            serde_json::Value::String("${auth_uuid}".to_string()),
            serde_json::Value::String("--accessToken".to_string()),
            serde_json::Value::String("${auth_access_token}".to_string()),
        ];

        let mut arguments = HashMap::new();
        arguments.insert("game".to_string(), serde_json::Value::Array(game_args));

        let version_pkg = VersionPackage {
            id: "1.21.4".to_string(),
            version_type: "release".to_string(),
            main_class: "net.minecraft.client.main.Main".to_string(),
            minecraft_arguments: None,
            arguments: Some(arguments),
            downloads: VersionDownloads {
                client: DownloadArtifact {
                    sha1: "abc".to_string(),
                    size: 1000,
                    url: "https://mock/client.jar".to_string(),
                    path: None,
                },
                server: None,
                client_mappings: None,
            },
            libraries: Vec::new(),
            asset_index: AssetIndexRef {
                id: "17".to_string(),
                sha1: "def".to_string(),
                size: 500,
                url: "https://mock/17.json".to_string(),
                total_size: 500,
            },
            assets: "17".to_string(),
            java_version: None,
            compliance_level: 0,
        };

        let ctx = LaunchContext {
            version_package: version_pkg,
            instance_dir: temp.path().join("instance"),
            game_dir: temp.path().join("instance").join(".minecraft"),
            assets_dir: temp.path().join("assets"),
            libraries_dir: temp.path().join("libraries"),
            client_jar_path: client_jar,
            java_executable: PathBuf::from("C:/Java/bin/java.exe"),
            player_name: "CyberHacker".to_string(),
            player_uuid: "12345678-1234-1234-1234-1234567890ab".to_string(),
            access_token: "secret_token_val".to_string(),
            user_type: "msa".to_string(),
            xuid: "".to_string(),
            min_memory_mb: 2048,
            max_memory_mb: 6144,
            window_width: 1280,
            window_height: 720,
            custom_jvm_args: None,
            performance_preset: Some("aikar".to_string()),
            mod_loader: "vanilla".to_string(),
            extra_classpath: Vec::new(),
            main_class_override: None,
            extra_game_args: Vec::new(),
        };

        let args = LaunchArgumentBuilder::build_command_args(&ctx);

        assert!(args.contains(&"-Xms2048M".to_string()));
        assert!(args.contains(&"-Xmx6144M".to_string()));
        assert!(args.contains(&"-XX:+UseG1GC".to_string()));
        assert!(args.contains(&"-Dminecraft.launcher.brand=DigitalVigital".to_string()));
        assert!(args.contains(&"-cp".to_string()));
        assert!(args.contains(&"net.minecraft.client.main.Main".to_string()));
        assert!(args.contains(&"--gameDir".to_string()));
    }

    #[test]
    fn test_launch_argument_builder_modern_arguments_jvm() {
        let temp = tempdir().unwrap();
        let client_jar = temp.path().join("1.21.4.jar");
        std::fs::write(&client_jar, b"mock client jar").unwrap();

        let jvm_args = vec![
            serde_json::Value::String("-Djava.library.path=${natives_directory}".to_string()),
            serde_json::Value::String("-Dcustom.launcher=${launcher_name}".to_string()),
            serde_json::Value::String("-cp".to_string()),
            serde_json::Value::String("${classpath}".to_string()),
        ];

        let mut arguments = HashMap::new();
        arguments.insert("jvm".to_string(), serde_json::Value::Array(jvm_args));

        let version_pkg = VersionPackage {
            id: "1.21.4".to_string(),
            version_type: "release".to_string(),
            main_class: "net.minecraft.client.main.Main".to_string(),
            minecraft_arguments: None,
            arguments: Some(arguments),
            downloads: VersionDownloads {
                client: DownloadArtifact {
                    sha1: "abc".to_string(),
                    size: 1000,
                    url: "https://mock/client.jar".to_string(),
                    path: None,
                },
                server: None,
                client_mappings: None,
            },
            libraries: Vec::new(),
            asset_index: AssetIndexRef {
                id: "17".to_string(),
                sha1: "def".to_string(),
                size: 500,
                url: "https://mock/17.json".to_string(),
                total_size: 500,
            },
            assets: "17".to_string(),
            java_version: None,
            compliance_level: 0,
        };

        let ctx = LaunchContext {
            version_package: version_pkg,
            instance_dir: temp.path().join("instance"),
            game_dir: temp.path().join("instance").join(".minecraft"),
            assets_dir: temp.path().join("assets"),
            libraries_dir: temp.path().join("libraries"),
            client_jar_path: client_jar,
            java_executable: PathBuf::from("C:/Java/bin/java.exe"),
            player_name: "Tester".to_string(),
            player_uuid: "12345678-1234-1234-1234-1234567890ab".to_string(),
            access_token: "mock_token".to_string(),
            user_type: "msa".to_string(),
            xuid: "".to_string(),
            min_memory_mb: 2048,
            max_memory_mb: 4096,
            window_width: 1280,
            window_height: 720,
            custom_jvm_args: None,
            performance_preset: None,
            mod_loader: "vanilla".to_string(),
            extra_classpath: Vec::new(),
            main_class_override: None,
            extra_game_args: Vec::new(),
        };

        let args = LaunchArgumentBuilder::build_command_args(&ctx);
        assert!(args.contains(&"-Dcustom.launcher=DigitalVigital".to_string()));
        assert!(args.iter().any(|a| a.starts_with("-Djava.library.path=")));
    }

    #[test]
    fn test_launch_argument_builder_fabric_mod_loader() {
        let temp = tempdir().unwrap();
        let client_jar = temp.path().join("1.21.4.jar");
        std::fs::write(&client_jar, b"mock jar").unwrap();

        let fabric_jar = temp.path().join("fabric-loader.jar");
        std::fs::write(&fabric_jar, b"mock fabric jar").unwrap();

        let version_pkg = VersionPackage {
            id: "1.21.4".to_string(),
            version_type: "release".to_string(),
            main_class: "net.minecraft.client.main.Main".to_string(),
            minecraft_arguments: Some("--username ${auth_player_name}".to_string()),
            arguments: None,
            downloads: VersionDownloads {
                client: DownloadArtifact {
                    sha1: "abc".to_string(),
                    size: 1000,
                    url: "https://mock/client.jar".to_string(),
                    path: None,
                },
                server: None,
                client_mappings: None,
            },
            libraries: Vec::new(),
            asset_index: AssetIndexRef {
                id: "17".to_string(),
                sha1: "def".to_string(),
                size: 500,
                url: "https://mock/17.json".to_string(),
                total_size: 500,
            },
            assets: "17".to_string(),
            java_version: None,
            compliance_level: 0,
        };

        let ctx = LaunchContext {
            version_package: version_pkg,
            instance_dir: temp.path().join("instance"),
            game_dir: temp.path().join("instance").join(".minecraft"),
            assets_dir: temp.path().join("assets"),
            libraries_dir: temp.path().join("libraries"),
            client_jar_path: client_jar,
            java_executable: PathBuf::from("java"),
            player_name: "ModdedPlayer".to_string(),
            player_uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            access_token: "0".to_string(),
            user_type: "offline".to_string(),
            xuid: "".to_string(),
            min_memory_mb: 2048,
            max_memory_mb: 4096,
            window_width: 1280,
            window_height: 720,
            custom_jvm_args: Some(vec![
                "-DFabricMcEmu= net.minecraft.client.main.Main ".to_string()
            ]),
            performance_preset: None,
            mod_loader: "fabric".to_string(),
            extra_classpath: vec![fabric_jar],
            main_class_override: Some(
                "net.fabricmc.loader.impl.launch.knot.KnotClient".to_string(),
            ),
            extra_game_args: Vec::new(),
        };

        let args = LaunchArgumentBuilder::build_command_args(&ctx);

        // Verify KnotClient is the main class
        assert!(args.contains(&"net.fabricmc.loader.impl.launch.knot.KnotClient".to_string()));
        // Verify vanilla main is NOT the entry main class
        assert_ne!(
            args[args.iter().position(|r| r == "-cp").unwrap() + 2],
            "net.minecraft.client.main.Main"
        );
        // Verify Fabric JVM emulator flag is included
        assert!(args.contains(&"-DFabricMcEmu= net.minecraft.client.main.Main ".to_string()));
        // Verify gameDir is present
        assert!(args.contains(&"--gameDir".to_string()));
        // Verify classpath contains the fabric jar
        let cp_idx = args.iter().position(|r| r == "-cp").unwrap();
        assert!(args[cp_idx + 1].contains("fabric-loader.jar"));

        let username_idx = args.iter().position(|r| r == "--username").unwrap();
        assert_eq!(args[username_idx + 1], "ModdedPlayer");
    }
}
