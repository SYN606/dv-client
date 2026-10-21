//! Mojang Piston Version Manifest and Version Package Data Models.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestLatest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionSummary {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String, // "release", "snapshot", "old_beta", "old_alpha"
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub sha1: String,
    #[serde(default, rename = "complianceLevel")]
    pub compliance_level: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: ManifestLatest,
    pub versions: Vec<VersionSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadArtifact {
    pub sha1: String,
    pub size: u64,
    pub url: String,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionDownloads {
    pub client: DownloadArtifact,
    pub server: Option<DownloadArtifact>,
    pub client_mappings: Option<DownloadArtifact>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetIndexRef {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
    #[serde(default, rename = "totalSize")]
    pub total_size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaVersionReq {
    #[serde(default = "default_component")]
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

fn default_component() -> String {
    "java-runtime-gamma".to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleOS {
    pub name: Option<String>,
    pub version: Option<String>,
    pub arch: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadRule {
    pub action: String, // "allow" or "disallow"
    pub os: Option<RuleOS>,
    pub features: Option<HashMap<String, bool>>,
}

/// Evaluates if a set of download rules permits inclusion on the current OS and CPU architecture.
pub fn evaluate_rules(rules: Option<&[DownloadRule]>) -> bool {
    let rules = match rules {
        Some(r) => r,
        None => return true,
    };

    if rules.is_empty() {
        return true;
    }

    let mut allowed = false;
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

    for rule in rules {
        if let Some(ref os) = rule.os {
            if let Some(ref name) = os.name {
                if name != current_os {
                    continue;
                }
            }

            if let Some(ref arch) = os.arch {
                let arch_matches = match arch.as_str() {
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
                    continue;
                }
            }
        }

        if rule.action == "allow" {
            allowed = true;
        } else if rule.action == "disallow" {
            allowed = false;
        }
    }

    allowed
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryArtifacts {
    pub artifact: Option<DownloadArtifact>,
    pub classifiers: Option<HashMap<String, DownloadArtifact>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub name: String,
    pub downloads: Option<LibraryArtifacts>,
    pub rules: Option<Vec<DownloadRule>>,
    pub natives: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionPackage {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    pub arguments: Option<HashMap<String, serde_json::Value>>,
    pub downloads: VersionDownloads,
    #[serde(default)]
    pub libraries: Vec<LibraryEntry>,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndexRef,
    pub assets: String,
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersionReq>,
    #[serde(default, rename = "complianceLevel")]
    pub compliance_level: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mock_version_manifest() {
        let mock_json = r#"{
            "latest": {
                "release": "1.21.4",
                "snapshot": "25w03a"
            },
            "versions": [
                {
                    "id": "1.21.4",
                    "type": "release",
                    "url": "https://piston-meta.mojang.com/v1/packages/abc/1.21.4.json",
                    "time": "2024-12-03T10:00:00Z",
                    "releaseTime": "2024-12-03T10:00:00Z",
                    "sha1": "0123456789abcdef0123456789abcdef01234567"
                },
                {
                    "id": "25w03a",
                    "type": "snapshot",
                    "url": "https://piston-meta.mojang.com/v1/packages/def/25w03a.json",
                    "time": "2025-01-15T12:00:00Z",
                    "releaseTime": "2025-01-15T12:00:00Z",
                    "sha1": "fedcba9876543210fedcba9876543210fedcba98"
                }
            ]
        }"#;

        let manifest: VersionManifest = serde_json::from_str(mock_json).unwrap();
        assert_eq!(manifest.latest.release, "1.21.4");
        assert_eq!(manifest.latest.snapshot, "25w03a");
        assert_eq!(manifest.versions.len(), 2);
        assert_eq!(manifest.versions[0].version_type, "release");
        assert_eq!(manifest.versions[1].version_type, "snapshot");
    }

    #[test]
    fn test_evaluate_rules() {
        let allow_all = vec![DownloadRule {
            action: "allow".to_string(),
            os: None,
            features: None,
        }];
        assert!(evaluate_rules(Some(&allow_all)));

        let disallow_current = vec![
            DownloadRule {
                action: "allow".to_string(),
                os: None,
                features: None,
            },
            DownloadRule {
                action: "disallow".to_string(),
                os: Some(RuleOS {
                    name: Some(if cfg!(target_os = "windows") {
                        "windows".to_string()
                    } else {
                        "linux".to_string()
                    }),
                    version: None,
                    arch: None,
                }),
                features: None,
            },
        ];
        assert!(!evaluate_rules(Some(&disallow_current)));

        // Test CPU architecture matching
        let wrong_arch_rule = vec![DownloadRule {
            action: "allow".to_string(),
            os: Some(RuleOS {
                name: Some(if cfg!(target_os = "windows") {
                    "windows".to_string()
                } else {
                    "linux".to_string()
                }),
                version: None,
                arch: Some("incompatible_cpu_arch_xyz".to_string()),
            }),
            features: None,
        }];
        assert!(!evaluate_rules(Some(&wrong_arch_rule)));
    }
}
