//! Intelligent Minecraft Crash and Error Diagnostics Subsystem.
//!
//! Analyzes JVM output, stderr streams, exit codes, and Mojang crash reports
//! to automatically deduce root causes and provide actionable recommendations.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrashAnalysis {
    pub exit_code: i32,
    pub category: String,
    pub headline: String,
    pub explanation: String,
    pub recommended_action: String,
    pub log_excerpt: String,
    pub crash_report_path: Option<String>,
}

impl CrashAnalysis {
    /// Formats a clean ANSI-highlighted summary for real-time terminal display.
    pub fn to_ansi_summary(&self) -> String {
        format!(
            "\n\
            \x1b[31;1m======================================================================\x1b[0m\n\
            \x1b[31;1m[DV ERROR DETECTOR] ⚠️ Minecraft Process Terminated (Exit Code: {})\x1b[0m\n\
            \x1b[33;1mCategory:   \x1b[0m {}\n\
            \x1b[37;1mIssue:      \x1b[0m {}\n\
            \x1b[36;1mExplanation:\x1b[0m {}\n\
            \x1b[32;1mSuggested Fix:\x1b[0m {}\n\
            {}\
            \x1b[31;1m======================================================================\x1b[0m\n",
            self.exit_code,
            self.category,
            self.headline,
            self.explanation,
            self.recommended_action,
            if let Some(ref cr) = self.crash_report_path {
                format!("\x1b[35;1mCrash Report:\x1b[0m {}\n", cr)
            } else {
                String::new()
            }
        )
    }
}

pub struct ErrorDetector;

impl ErrorDetector {
    /// Inspect recent logs, stderr lines, and disk crash reports to diagnose a process failure.
    pub fn analyze(
        exit_code: i32,
        recent_lines: &[String],
        game_dir: &Path,
    ) -> CrashAnalysis {
        let combined_text = recent_lines.join("\n");
        let crash_report = Self::find_recent_crash_report(game_dir);

        // 1. Check for Java Bytecode Version Mismatch (UnsupportedClassVersionError)
        if combined_text.contains("UnsupportedClassVersionError")
            || combined_text.contains("has been compiled by a more recent version of the Java Runtime")
        {
            let (compiled_ver, current_ver) = Self::parse_bytecode_versions(&combined_text);
            return CrashAnalysis {
                exit_code,
                category: "Java Version Mismatch".to_string(),
                headline: "Unsupported Java Runtime Version".to_string(),
                explanation: format!(
                    "Minecraft or a loaded mod requires {} (class version {}), but is running under {} (class version {}).",
                    Self::class_version_to_java_name(compiled_ver),
                    compiled_ver,
                    Self::class_version_to_java_name(current_ver),
                    current_ver
                ),
                recommended_action: "Open Settings -> Java and select Java 21 (or click 'Install Java 21' to automatically download Temurin OpenJDK 21).".to_string(),
                log_excerpt: Self::extract_relevant_snippet(&recent_lines, "UnsupportedClassVersionError"),
                crash_report_path: crash_report.map(|p| p.to_string_lossy().to_string()),
            };
        }

        // 2. Check for Missing Main Class or ClassNotFoundException
        if combined_text.contains("Could not find or load main class")
            || (combined_text.contains("ClassNotFoundException") && recent_lines.len() < 25)
        {
            let missing_class = combined_text
                .lines()
                .find(|l| l.contains("Could not find or load main class") || l.contains("ClassNotFoundException"))
                .map(|l| l.trim().to_string())
                .unwrap_or_else(|| "Main class not found".to_string());

            return CrashAnalysis {
                exit_code,
                category: "Launch Configuration Error".to_string(),
                headline: "Main Game Class Not Found".to_string(),
                explanation: format!(
                    "The JVM was unable to locate the entrypoint class ({}) on the classpath.",
                    missing_class
                ),
                recommended_action: "Ensure your instance profile has compatible game libraries downloaded, or click 'Repair' in Profile Settings.".to_string(),
                log_excerpt: Self::extract_relevant_snippet(&recent_lines, "main class"),
                crash_report_path: crash_report.map(|p| p.to_string_lossy().to_string()),
            };
        }

        // 3. Check for Invalid JVM Options / Unrecognized VM Options
        if combined_text.contains("Unrecognized option:")
            || combined_text.contains("Unrecognized VM option")
            || combined_text.contains("Could not create the Java Virtual Machine")
        {
            let bad_opt = combined_text
                .lines()
                .find(|l| l.contains("Unrecognized") || l.contains("Could not create"))
                .map(|l| l.trim().to_string())
                .unwrap_or_else(|| "Unrecognized JVM argument".to_string());

            return CrashAnalysis {
                exit_code,
                category: "JVM Arguments Error".to_string(),
                headline: "Invalid Java Virtual Machine Option".to_string(),
                explanation: format!(
                    "The JVM refused to start due to invalid or unsupported command-line arguments: {}",
                    bad_opt
                ),
                recommended_action: "Open Settings and review your Custom JVM Arguments. Remove deprecated or unrecognized flags.".to_string(),
                log_excerpt: Self::extract_relevant_snippet(&recent_lines, "Unrecognized"),
                crash_report_path: crash_report.map(|p| p.to_string_lossy().to_string()),
            };
        }

        // 4. Check for Out Of Memory Errors
        if combined_text.contains("OutOfMemoryError")
            || combined_text.contains("Could not reserve enough space for")
            || combined_text.contains("unable to create new native thread")
        {
            return CrashAnalysis {
                exit_code,
                category: "Out Of Memory".to_string(),
                headline: "Insufficient RAM Allocation".to_string(),
                explanation: "Minecraft ran out of available memory, or the JVM was unable to allocate the requested heap space on this system.".to_string(),
                recommended_action: "Adjust the Maximum Memory allocation slider in Profile Settings (recommend 4096MB - 8192MB). Ensure 64-bit Java is installed.".to_string(),
                log_excerpt: Self::extract_relevant_snippet(&recent_lines, "OutOfMemory"),
                crash_report_path: crash_report.map(|p| p.to_string_lossy().to_string()),
            };
        }

        // 5. Check for Fabric Mod Set Incompatibilities
        if combined_text.contains("Incompatible mod set found!")
            || combined_text.contains("net.fabricmc.loader.impl.FormattedException")
        {
            let reason = Self::extract_fabric_problem(&combined_text);
            return CrashAnalysis {
                exit_code,
                category: "Fabric Mod Conflict".to_string(),
                headline: "Incompatible Fabric Mods Detected".to_string(),
                explanation: reason.clone(),
                recommended_action: "Open the Mod Studio tab, locate the incompatible or missing dependencies listed above, and update or disable them.".to_string(),
                log_excerpt: reason,
                crash_report_path: crash_report.map(|p| p.to_string_lossy().to_string()),
            };
        }

        // 6. Check for Forge / NeoForge Mod Loading Errors
        if combined_text.contains("net.minecraftforge.fml.LoadingFailedException")
            || combined_text.contains("ModLoadingException")
            || combined_text.contains("Missing or unsupported mandatory dependencies")
        {
            return CrashAnalysis {
                exit_code,
                category: "Forge Mod Conflict".to_string(),
                headline: "Forge Mod Loading Failed".to_string(),
                explanation: "Forge or NeoForge encountered incompatible mod versions or missing mandatory dependencies.".to_string(),
                recommended_action: "Open the Mod Studio tab to verify required Forge library dependencies and remove conflicting mods.".to_string(),
                log_excerpt: Self::extract_relevant_snippet(&recent_lines, "ModLoadingException"),
                crash_report_path: crash_report.map(|p| p.to_string_lossy().to_string()),
            };
        }

        // 7. Check for Graphics / OpenGL / Driver Crashes
        if combined_text.contains("GLFW error 65542")
            || combined_text.contains("Pixel format not accelerated")
            || combined_text.contains("WGL: The driver does not appear to support OpenGL")
            || combined_text.contains("org.lwjgl.LWJGLException")
        {
            return CrashAnalysis {
                exit_code,
                category: "Graphics Driver Error".to_string(),
                headline: "OpenGL / Display Driver Failure".to_string(),
                explanation: "The graphics driver failed to initialize an accelerated OpenGL window for Minecraft.".to_string(),
                recommended_action: "Update your GPU display drivers (NVIDIA / AMD / Intel) to the latest version, or change GPU Selection in Settings.".to_string(),
                log_excerpt: Self::extract_relevant_snippet(&recent_lines, "GLFW"),
                crash_report_path: crash_report.map(|p| p.to_string_lossy().to_string()),
            };
        }

        // 8. Windows Native Access Violation (0xC0000005) or Missing DLL (0xC0000135)
        if exit_code as u32 == 0xC0000005 || exit_code == -1073741819 {
            return CrashAnalysis {
                exit_code,
                category: "Native Access Violation".to_string(),
                headline: "Memory Access Violation (0xC0000005)".to_string(),
                explanation: "A native Minecraft library or driver attempted to access protected memory. Often caused by outdated GPU drivers or incompatible overlay hooks (e.g. RivaTuner, GeForce Experience).".to_string(),
                recommended_action: "Disable third-party screen recording/overlay utilities and update your graphics drivers.".to_string(),
                log_excerpt: Self::extract_relevant_snippet(&recent_lines, "EXCEPTION_ACCESS_VIOLATION"),
                crash_report_path: crash_report.map(|p| p.to_string_lossy().to_string()),
            };
        }

        if exit_code as u32 == 0xC0000135 || exit_code == -1073741515 {
            return CrashAnalysis {
                exit_code,
                category: "Missing DLL Dependency".to_string(),
                headline: "Required Windows DLL Not Found (0xC0000135)".to_string(),
                explanation: "A native library required by Java or Minecraft could not find a required Windows system DLL.".to_string(),
                recommended_action: "Download and install the Microsoft Visual C++ 2015-2022 Redistributable (x64) from Microsoft.".to_string(),
                log_excerpt: "Process terminated immediately due to STATUS_DLL_NOT_FOUND (0xC0000135)".to_string(),
                crash_report_path: crash_report.map(|p| p.to_string_lossy().to_string()),
            };
        }

        // 9. If a crash report exists, read its top description
        if let Some(ref cr_path) = crash_report {
            if let Ok(content) = fs::read_to_string(cr_path) {
                let desc = content
                    .lines()
                    .find(|l| l.starts_with("Description:"))
                    .map(|l| l.trim().to_string())
                    .unwrap_or_else(|| "Crash report generated by Minecraft".to_string());

                return CrashAnalysis {
                    exit_code,
                    category: "Game Crash".to_string(),
                    headline: desc,
                    explanation: "Minecraft crashed during execution. Detailed diagnostics were recorded in the official crash report.".to_string(),
                    recommended_action: "Click 'Report Crash' to submit diagnostics to Discord, or inspect the crash log file.".to_string(),
                    log_excerpt: content.lines().take(20).collect::<Vec<_>>().join("\n"),
                    crash_report_path: Some(cr_path.to_string_lossy().to_string()),
                };
            }
        }

        // 10. Fallback General Exit
        let snippet = recent_lines.iter().rev().take(10).cloned().collect::<Vec<_>>();
        let mut snippet_rev = snippet;
        snippet_rev.reverse();

        CrashAnalysis {
            exit_code,
            category: "Unexpected Termination".to_string(),
            headline: format!("Game process terminated with exit code {}", exit_code),
            explanation: "Minecraft stopped running unexpectedly. Review the console logs for recent exceptions or error traces.".to_string(),
            recommended_action: "Check the Live Console tab for the full log trace or click 'Report Crash' to analyze further.".to_string(),
            log_excerpt: snippet_rev.join("\n"),
            crash_report_path: None,
        }
    }

    fn find_recent_crash_report(game_dir: &Path) -> Option<PathBuf> {
        let reports_dir = game_dir.join("crash-reports");
        if !reports_dir.exists() {
            return None;
        }

        let mut newest_file = None;
        let mut newest_time = SystemTime::UNIX_EPOCH;

        if let Ok(entries) = fs::read_dir(reports_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("txt") {
                    if let Ok(meta) = p.metadata() {
                        if let Ok(modified) = meta.modified() {
                            if modified > newest_time {
                                newest_time = modified;
                                newest_file = Some(p);
                            }
                        }
                    }
                }
            }
        }

        // Only return if created within the last 5 minutes
        if let Some(ref _path) = newest_file {
            if let Ok(elapsed) = newest_time.elapsed() {
                if elapsed.as_secs() <= 300 {
                    return newest_file;
                }
            }
        }

        None
    }

    fn parse_bytecode_versions(text: &str) -> (u32, u32) {
        // "class file version 65.0), this version of the Java Runtime only recognizes class file versions up to 52.0"
        let mut compiled = 65;
        let mut current = 52;

        if let Some(idx) = text.find("class file version ") {
            let part = &text[idx + 19..];
            if let Some(end) = part.find('.') {
                if let Ok(v) = part[..end].parse::<u32>() {
                    compiled = v;
                }
            }
        }

        if let Some(idx) = text.find("up to ") {
            let part = &text[idx + 6..];
            if let Some(end) = part.find('.') {
                if let Ok(v) = part[..end].parse::<u32>() {
                    current = v;
                }
            }
        }

        (compiled, current)
    }

    fn class_version_to_java_name(class_ver: u32) -> &'static str {
        match class_ver {
            45..=52 => "Java 8 (1.8)",
            53 => "Java 9",
            54 => "Java 10",
            55 => "Java 11",
            56 => "Java 12",
            57 => "Java 13",
            58 => "Java 14",
            59 => "Java 15",
            60 => "Java 16",
            61 => "Java 17",
            62 => "Java 18",
            63 => "Java 19",
            64 => "Java 20",
            65 => "Java 21",
            66 => "Java 22",
            67 => "Java 23",
            68 => "Java 24",
            _ => "Java 21+",
        }
    }

    fn extract_relevant_snippet(lines: &[String], keyword: &str) -> String {
        for (idx, line) in lines.iter().enumerate() {
            if line.contains(keyword) {
                let start = idx.saturating_sub(2);
                let end = (idx + 6).min(lines.len());
                return lines[start..end].join("\n");
            }
        }
        lines.iter().rev().take(6).cloned().collect::<Vec<_>>().join("\n")
    }

    fn extract_fabric_problem(text: &str) -> String {
        let mut problem_lines = Vec::new();
        let mut capturing = false;

        for line in text.lines() {
            if line.contains("Incompatible mod set found!") || line.contains("net.fabricmc.loader.impl.FormattedException") {
                capturing = true;
            }
            if capturing {
                problem_lines.push(line.trim());
                if problem_lines.len() >= 12 || line.contains("A potential solution has been determined") {
                    if problem_lines.len() >= 15 {
                        break;
                    }
                }
            }
        }

        if problem_lines.is_empty() {
            "Incompatible Fabric mod dependencies detected.".to_string()
        } else {
            problem_lines.join("\n")
        }
    }
}
