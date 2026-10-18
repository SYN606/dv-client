//! Integration tests for token sanitization and log scrubbing.

use dv_launcher::security::{sanitize_cmd_args, sanitize_text};

#[test]
fn test_sanitize_bearer_token() {
    let raw = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.doNotLeakThis";
    let scrubbed = sanitize_text(raw);
    assert!(!scrubbed.contains("doNotLeakThis"));
    assert!(scrubbed.contains("[REDACTED"));
}

#[test]
fn test_sanitize_minecraft_access_token() {
    let raw = "java -Xmx4G net.minecraft.client.main.Main --username Steve --accessToken eyJlongTokenHere.secret.signature";
    let scrubbed = sanitize_text(raw);
    assert!(!scrubbed.contains("eyJlongTokenHere"));
    assert!(scrubbed.contains("[REDACTED"));
}

#[test]
fn test_sanitize_command_args_list() {
    let args = vec![
        "java.exe".to_string(),
        "-Xmx4096M".to_string(),
        "net.minecraft.client.main.Main".to_string(),
        "--username".to_string(),
        "DVPlayer".to_string(),
        "--accessToken".to_string(),
        "SuperSecretAccessToken12345".to_string(),
        "--version".to_string(),
        "1.21.4".to_string(),
    ];
    let scrubbed = sanitize_cmd_args(&args);
    assert_eq!(scrubbed[0], "java.exe");
    assert_eq!(scrubbed[4], "DVPlayer");
    assert_eq!(scrubbed[5], "--accessToken");
    assert_eq!(scrubbed[6], "[REDACTED_PARAMETER]");
    assert!(!scrubbed.join(" ").contains("SuperSecretAccessToken12345"));
}

#[test]
fn test_clean_text_unchanged() {
    let text = "Loading world 'CyberRealm' with 48 mods...";
    assert_eq!(sanitize_text(text), text);
}
