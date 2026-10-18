//! Integration tests for LaunchArgumentBuilder and command assembly.

use dv_launcher::minecraft::{
    AssetIndexRef, DownloadArtifact, LaunchArgumentBuilder, LaunchContext, VersionDownloads,
    VersionPackage,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_launch_argument_builder_modern() {
    let temp = tempdir().unwrap();
    let client_jar = temp.path().join("1.21.4.jar");
    std::fs::write(&client_jar, b"mock jar").unwrap();

    let game_args = vec![
        serde_json::Value::String("--username".to_string()),
        serde_json::Value::String("${auth_player_name}".to_string()),
        serde_json::Value::String("--version".to_string()),
        serde_json::Value::String("${version_name}".to_string()),
        serde_json::Value::String("--gameDir".to_string()),
        serde_json::Value::String("${game_directory}".to_string()),
        serde_json::Value::String("--assetsDir".to_string()),
        serde_json::Value::String("${assets_root}".to_string()),
        serde_json::Value::String("--assetIndex".to_string()),
        serde_json::Value::String("${assets_index_name}".to_string()),
        serde_json::Value::String("--uuid".to_string()),
        serde_json::Value::String("${auth_uuid}".to_string()),
        serde_json::Value::String("--accessToken".to_string()),
        serde_json::Value::String("${auth_access_token}".to_string()),
        serde_json::Value::String("--userType".to_string()),
        serde_json::Value::String("${user_type}".to_string()),
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

    assert!(args[0].contains("java.exe"));
    assert!(args.contains(&"-Xms2048M".to_string()));
    assert!(args.contains(&"-Xmx6144M".to_string()));
    assert!(args.contains(&"-Dminecraft.launcher.brand=DigitalVigital".to_string()));
    assert!(args.contains(&"-cp".to_string()));
    assert!(args.contains(&"net.minecraft.client.main.Main".to_string()));

    let username_idx = args.iter().position(|r| r == "--username").unwrap();
    assert_eq!(args[username_idx + 1], "CyberHacker");

    let uuid_idx = args.iter().position(|r| r == "--uuid").unwrap();
    assert_eq!(args[uuid_idx + 1], "12345678-1234-1234-1234-1234567890ab");

    let token_idx = args.iter().position(|r| r == "--accessToken").unwrap();
    assert_eq!(args[token_idx + 1], "secret_token_val");
}
