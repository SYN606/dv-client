//! Java Runtime Discovery and Version Inspection.
//!
//! Scans system PATH, standard Linux JVM directories (`/usr/lib/jvm`), and Windows
//! registry / Program Files locations to detect compatible Java runtimes (JDK 8, 17, 21, 25).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaRuntimeInfo {
    pub path: PathBuf,
    pub full_version: String,
    pub major_version: u32,
    pub is_64bit: bool,
    pub vendor: String,
}

pub struct JavaDiscovery;

impl JavaDiscovery {
    /// Inspect an individual java executable and parse its version metadata.
    pub fn inspect_java_executable<P: AsRef<Path>>(path: P) -> Option<JavaRuntimeInfo> {
        let path = path.as_ref();
        if !path.exists() {
            return None;
        }

        let output = Command::new(path).arg("-version").output().ok()?;

        // java -version writes output to stderr in standard JVMs
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let full_output = format!("{}\n{}", stderr, stdout);

        Self::parse_java_version_output(path.to_path_buf(), &full_output)
    }

    /// Parses the raw string output of `java -version`.
    pub fn parse_java_version_output(path: PathBuf, output: &str) -> Option<JavaRuntimeInfo> {
        if output.trim().is_empty() {
            return None;
        }

        let is_64bit =
            output.contains("64-Bit") || output.contains("x86_64") || output.contains("amd64");

        // Parse major version: look for version "..." pattern
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
        } else if output.contains("OpenJDK") {
            "OpenJDK"
        } else if output.contains("Oracle") {
            "Oracle"
        } else {
            "Java"
        }
        .to_string();

        Some(JavaRuntimeInfo {
            path,
            full_version,
            major_version,
            is_64bit,
            vendor,
        })
    }

    /// Auto-detect all compatible Java runtimes on the current system.
    pub fn discover_all() -> Vec<JavaRuntimeInfo> {
        let mut runtimes = Vec::new();
        let mut candidate_paths = Vec::new();

        // 1. Check system PATH
        if let Ok(path_var) = std::env::var("PATH") {
            let exe_name = if cfg!(target_os = "windows") {
                "java.exe"
            } else {
                "java"
            };

            for dir in std::env::split_paths(&path_var) {
                let candidate = dir.join(exe_name);
                if candidate.exists() && !candidate_paths.contains(&candidate) {
                    candidate_paths.push(candidate);
                }
            }
        }

        // 2. Platform-specific standard directories
        if cfg!(target_os = "windows") {
            let standard_roots = [
                r"C:\Program Files\Java",
                r"C:\Program Files\Eclipse Adoptium",
                r"C:\Program Files\Microsoft",
                r"C:\Program Files\BellSoft",
            ];
            for root_str in standard_roots {
                let root = Path::new(root_str);
                if root.exists() {
                    if let Ok(entries) = std::fs::read_dir(root) {
                        for entry in entries.flatten() {
                            let java_exe = entry.path().join("bin").join("java.exe");
                            if java_exe.exists() && !candidate_paths.contains(&java_exe) {
                                candidate_paths.push(java_exe);
                            }
                        }
                    }
                }
            }
        } else {
            // Linux / Unix standard paths
            let linux_roots = ["/usr/lib/jvm", "/usr/java", "/opt/java"];
            for root_str in linux_roots {
                let root = Path::new(root_str);
                if root.exists() {
                    if let Ok(entries) = std::fs::read_dir(root) {
                        for entry in entries.flatten() {
                            let java_bin = entry.path().join("bin").join("java");
                            if java_bin.exists() && !candidate_paths.contains(&java_bin) {
                                candidate_paths.push(java_bin);
                            }
                        }
                    }
                }
            }
        }

        for path in candidate_paths {
            if let Some(info) = Self::inspect_java_executable(&path) {
                info!(
                    "Discovered Java runtime: {} (v{}) at {:?}",
                    info.vendor, info.full_version, info.path
                );
                runtimes.push(info);
            }
        }

        runtimes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modern_java_output() {
        let output = r#"
openjdk version "21.0.2" 2024-01-16
OpenJDK Runtime Environment Temurin-21.0.2+13 (build 21.0.2+13)
OpenJDK 64-Bit Server VM Temurin-21.0.2+13 (build 21.0.2+13, mixed mode, sharing)
"#;
        let info = JavaDiscovery::parse_java_version_output(PathBuf::from("/usr/bin/java"), output)
            .unwrap();
        assert_eq!(info.major_version, 21);
        assert_eq!(info.full_version, "21.0.2");
        assert!(info.is_64bit);
        assert_eq!(info.vendor, "Eclipse Temurin");
    }

    #[test]
    fn test_parse_legacy_java_output() {
        let output = r#"
java version "1.8.0_351"
Java(TM) SE Runtime Environment (build 1.8.0_351-b10)
Java HotSpot(TM) 64-Bit Server VM (build 25.351-b10, mixed mode)
"#;
        let info = JavaDiscovery::parse_java_version_output(
            PathBuf::from("C:\\Java\\bin\\java.exe"),
            output,
        )
        .unwrap();
        assert_eq!(info.major_version, 8);
        assert_eq!(info.full_version, "1.8.0_351");
        assert!(info.is_64bit);
    }

    #[test]
    fn test_invalid_java_path() {
        let bogus = PathBuf::from("/non/existent/path/java");
        let info = JavaDiscovery::inspect_java_executable(bogus);
        assert!(info.is_none());
    }
}
