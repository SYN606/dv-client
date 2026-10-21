//! Java Runtime Detector and File Picker.
//!
//! Automatically discovers installed JREs/JDKs across standard OS locations
//! and provides native file dialog selection for custom runtimes using `rfd`.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JavaInstallation {
    pub path: PathBuf,
    pub vendor: String,
    pub full_version: String,
    pub major_version: u32,
    pub is_64bit: bool,
    pub display_name: String,
}

pub struct JavaDetector;

impl JavaDetector {
    /// Inspect an individual java executable and parse its version metadata.
    pub fn inspect_executable<P: AsRef<Path>>(path: P) -> Option<JavaInstallation> {
        let path = path.as_ref();
        if !path.exists() {
            return None;
        }

        let mut cmd = Command::new(path);
        cmd.arg("-version");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW suppresses console window flash
        }

        let output = cmd.output().ok()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let full_output = format!("{}\n{}", stderr, stdout);

        let clean_path = dunce::simplified(path).to_path_buf();
        Self::parse_version_output(clean_path, &full_output)
    }

    /// Parses the raw string output of `java -version`.
    pub fn parse_version_output(path: PathBuf, output: &str) -> Option<JavaInstallation> {
        if output.trim().is_empty() {
            return None;
        }

        let is_64bit =
            output.contains("64-Bit") || output.contains("x86_64") || output.contains("amd64");

        let mut full_version = "Unknown".to_string();
        let mut major_version: u32 = 0;

        for line in output.lines() {
            if line.contains("version \"") {
                if let Some(start) = line.find("version \"") {
                    let rest = &line[start + 9..];
                    if let Some(end) = rest.find('"') {
                        let ver_str = &rest[..end];
                        full_version = ver_str.to_string();

                        if ver_str.starts_with("1.") {
                            // Legacy: 1.8 -> 8
                            if let Some(second) = ver_str.split('.').nth(1) {
                                major_version = second.parse().unwrap_or(8);
                            }
                        } else {
                            // Modern: 17.0.2 -> 17, 21.0.1 -> 21
                            if let Some(first) = ver_str.split('.').next() {
                                major_version = first.parse().unwrap_or(0);
                            }
                        }
                    }
                }
            }
        }

        if major_version == 0 {
            return None;
        }

        let vendor = if output.contains("Adoptium") || output.contains("Temurin") {
            "Eclipse Temurin"
        } else if output.contains("Microsoft") {
            "Microsoft OpenJDK"
        } else if output.contains("Zulu") {
            "Azul Zulu"
        } else if output.contains("BellSoft") || output.contains("Liberica") {
            "BellSoft Liberica"
        } else if output.contains("Corretto") {
            "Amazon Corretto"
        } else if output.contains("GraalVM") {
            "GraalVM"
        } else if output.contains("Semeru") {
            "IBM Semeru"
        } else if output.contains("Red Hat") {
            "Red Hat OpenJDK"
        } else if output.contains("SapMachine") {
            "SAP SapMachine"
        } else if output.contains("OpenJDK") {
            "OpenJDK"
        } else if output.contains("Oracle") {
            "Oracle"
        } else {
            "Java"
        }
        .to_string();

        let display_name = format!("{} Java {} (v{})", vendor, major_version, full_version);

        Some(JavaInstallation {
            path,
            vendor,
            full_version,
            major_version,
            is_64bit,
            display_name,
        })
    }

    /// Auto-detect all compatible Java runtimes on the current operating system.
    pub fn scan_system() -> Vec<JavaInstallation> {
        let mut runtimes = Vec::new();
        let mut candidate_paths = Vec::new();

        // 1. Scan system PATH
        if let Ok(path_var) = std::env::var("PATH") {
            let exe_name = if cfg!(target_os = "windows") {
                "java.exe"
            } else {
                "java"
            };

            for dir in std::env::split_paths(&path_var) {
                let candidate = dunce::simplified(&dir.join(exe_name)).to_path_buf();
                if candidate.exists() && !candidate_paths.contains(&candidate) {
                    candidate_paths.push(candidate);
                }
            }
        }

        // 2. Scan OS-specific JVM directories
        #[cfg(target_os = "windows")]
        Self::scan_windows_java(&mut candidate_paths);

        #[cfg(not(target_os = "windows"))]
        Self::scan_unix_java(&mut candidate_paths);

        for path in candidate_paths {
            if let Some(info) = Self::inspect_executable(&path) {
                // Avoid duplicates by canonical path where possible
                let canonical =
                    dunce::canonicalize(&info.path).unwrap_or_else(|_| info.path.clone());
                if !runtimes.iter().any(|r: &JavaInstallation| {
                    dunce::canonicalize(&r.path).unwrap_or_else(|_| r.path.clone()) == canonical
                }) {
                    info!(
                        "Discovered Java runtime: {} (v{}) at {:?}",
                        info.vendor, info.full_version, info.path
                    );
                    runtimes.push(info);
                }
            }
        }

        // Sort descending by major version (21, 17, 8)
        runtimes.sort_by_key(|a| std::cmp::Reverse(a.major_version));
        runtimes
    }

    #[cfg(target_os = "windows")]
    fn scan_windows_java(candidate_paths: &mut Vec<PathBuf>) {
        let mut win_roots: Vec<PathBuf> = Vec::new();

        // Check JAVA_HOME & JDK_HOME directly
        for env_key in &["JAVA_HOME", "JDK_HOME"] {
            if let Ok(home) = std::env::var(env_key) {
                let java_exe = dunce::simplified(&PathBuf::from(home).join("bin").join("java.exe"))
                    .to_path_buf();
                if java_exe.exists() && !candidate_paths.contains(&java_exe) {
                    candidate_paths.push(java_exe);
                }
            }
        }

        // Standard Program Files paths
        let prog_files =
            std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".to_string());
        let prog_files_x86 = std::env::var("ProgramFiles(x86)")
            .unwrap_or_else(|_| r"C:\Program Files (x86)".to_string());

        for base in &[&prog_files, &prog_files_x86] {
            let base_path = Path::new(base);
            win_roots.push(base_path.join("Java"));
            win_roots.push(base_path.join("Eclipse Adoptium"));
            win_roots.push(base_path.join("Microsoft"));
            win_roots.push(base_path.join("BellSoft"));
            win_roots.push(base_path.join("Zulu"));
            win_roots.push(base_path.join("Amazon Corretto"));
            win_roots.push(base_path.join("GraalVM"));
            win_roots.push(base_path.join("Semeru"));
            win_roots.push(base_path.join("RedHat"));
            win_roots.push(base_path.join("SapMachine"));
        }

        // User directory IDE runtimes (.jdks / .gradle)
        if let Some(home) = dirs::home_dir() {
            win_roots.push(home.join(".jdks"));
            win_roots.push(home.join(".gradle").join("jdks"));
        }

        // Official Minecraft Launcher and DV Launcher runtimes on Windows
        if let Ok(app_data) = std::env::var("APPDATA") {
            win_roots.push(PathBuf::from(&app_data).join(".minecraft").join("runtime"));
            win_roots.push(PathBuf::from(&app_data).join("DigitalVigital").join("runtimes"));
        }
        if let Some(data) = dirs::data_dir() {
            win_roots.push(data.join("DigitalVigital").join("runtimes"));
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            win_roots.push(
                PathBuf::from(local_app_data)
                    .join("Packages")
                    .join("Microsoft.4297127D64EC6_8wekyb3d8bbwe")
                    .join("LocalCache")
                    .join("Local")
                    .join("runtime"),
            );
        }

        // Scan Windows Registry for official JDK installations (Adoptium, Oracle, Microsoft)
        let reg_keys = [
            r"HKLM\SOFTWARE\JavaSoft\JDK",
            r"HKLM\SOFTWARE\JavaSoft\Java Development Kit",
            r"HKLM\SOFTWARE\JavaSoft\Java Runtime Environment",
            r"HKLM\SOFTWARE\Eclipse Adoptium\JDK",
            r"HKLM\SOFTWARE\Microsoft\JDK",
        ];
        for key in reg_keys {
            use std::os::windows::process::CommandExt;
            let mut cmd = Command::new("reg");
            cmd.args(["query", key, "/s", "/v", "JavaHome"]);
            cmd.creation_flags(0x08000000);
            if let Ok(output) = cmd.output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        if line.contains("JavaHome") && line.contains("REG_SZ") {
                            if let Some(val) = line.split("REG_SZ").nth(1) {
                                let home_path = val.trim();
                                let java_exe = dunce::simplified(
                                    &PathBuf::from(home_path).join("bin").join("java.exe"),
                                )
                                .to_path_buf();
                                if java_exe.exists() && !candidate_paths.contains(&java_exe) {
                                    candidate_paths.push(java_exe);
                                }
                            }
                        }
                    }
                }
            }
        }

        for root in win_roots {
            if root.exists() {
                if let Ok(entries) = std::fs::read_dir(&root) {
                    for entry in entries.flatten() {
                        let java_exe =
                            dunce::simplified(&entry.path().join("bin").join("java.exe"))
                                .to_path_buf();
                        if java_exe.exists() && !candidate_paths.contains(&java_exe) {
                            candidate_paths.push(java_exe.clone());
                        }
                        // Deep scan subdirectories for runtime bundles (e.g. java-runtime-gamma/windows/...)
                        if let Ok(sub_entries) = std::fs::read_dir(entry.path()) {
                            for sub in sub_entries.flatten() {
                                let sub_java =
                                    dunce::simplified(&sub.path().join("bin").join("java.exe"))
                                        .to_path_buf();
                                if sub_java.exists() && !candidate_paths.contains(&sub_java) {
                                    candidate_paths.push(sub_java);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn scan_unix_java(candidate_paths: &mut Vec<PathBuf>) {
        // Linux & Unix standard JVM directories
        let mut linux_roots = vec![
            PathBuf::from("/usr/lib/jvm"),
            PathBuf::from("/usr/java"),
            PathBuf::from("/opt/java"),
            PathBuf::from("/etc/alternatives"),
        ];
        if let Some(home) = dirs::home_dir() {
            linux_roots.push(home.join(".sdkman").join("candidates").join("java"));
            linux_roots.push(home.join(".asdf").join("installs").join("java"));
            linux_roots.push(home.join(".local").join("share").join("DigitalVigital").join("runtimes"));
        }
        if let Some(data) = dirs::data_dir() {
            linux_roots.push(data.join("DigitalVigital").join("runtimes"));
        }

        for root in linux_roots {
            if root.exists() {
                if let Ok(entries) = std::fs::read_dir(&root) {
                    for entry in entries.flatten() {
                        let bin = entry.path().join("bin").join("java");
                        if bin.exists() && !candidate_paths.contains(&bin) {
                            candidate_paths.push(bin);
                        } else if entry.path().file_name().is_some_and(|n| n == "java")
                            && entry.path().is_file()
                            && !candidate_paths.contains(&entry.path())
                        {
                            candidate_paths.push(entry.path());
                        }
                    }
                }
            }
        }
    }

    /// Prompt user with native file dialog to manually select a Java executable.
    pub fn pick_java_with_dialog() -> Option<JavaInstallation> {
        #[allow(unused_mut)]
        let mut dialog =
            rfd::FileDialog::new().set_title("Select Java Executable (java / java.exe)");

        #[cfg(target_os = "windows")]
        {
            dialog = dialog.add_filter("Java Executable (*.exe)", &["exe"]);
        }

        if let Some(path) = dialog.pick_file() {
            Self::inspect_executable(path)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_output_jdk21() {
        let output = r#"
openjdk version "21.0.2" 2024-01-16
OpenJDK Runtime Environment Temurin-21.0.2+13 (build 21.0.2+13)
OpenJDK 64-Bit Server VM Temurin-21.0.2+13 (build 21.0.2+13, mixed mode, sharing)
"#;
        let info =
            JavaDetector::parse_version_output(PathBuf::from("/usr/bin/java"), output).unwrap();
        assert_eq!(info.major_version, 21);
        assert_eq!(info.full_version, "21.0.2");
        assert!(info.is_64bit);
        assert_eq!(info.vendor, "Eclipse Temurin");
        assert!(info.display_name.contains("Temurin Java 21"));
    }

    #[test]
    fn test_parse_version_output_legacy_jdk8() {
        let output = r#"
java version "1.8.0_351"
Java(TM) SE Runtime Environment (build 1.8.0_351-b10)
Java HotSpot(TM) 64-Bit Server VM (build 25.351-b10, mixed mode)
"#;
        let info =
            JavaDetector::parse_version_output(PathBuf::from("C:\\Java\\bin\\java.exe"), output)
                .unwrap();
        assert_eq!(info.major_version, 8);
        assert_eq!(info.full_version, "1.8.0_351");
        assert!(info.is_64bit);
    }

    #[test]
    fn test_invalid_executable_returns_none() {
        let bogus = PathBuf::from("/non/existent/bogus/java");
        assert!(JavaDetector::inspect_executable(bogus).is_none());
    }
}
