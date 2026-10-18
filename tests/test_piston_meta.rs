//! Integration tests for Mojang Piston version manifest deserialization.

use dv_launcher::minecraft::VersionManifest;

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
