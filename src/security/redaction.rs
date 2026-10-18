//! In-flight token sanitization and log scrubbing.
//!
//! Scrubs OAuth access tokens, JWTs, and sensitive command-line parameters
//! before output is displayed in the UI console or written to disk.

use regex::Regex;
use std::sync::LazyLock;

static BEARER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(Bearer\s+)([A-Za-z0-9\-._~+/]+=*)").expect("Invalid regex"));

static ACCESS_TOKEN_PARAM_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(--accessToken\s+)([^\s]+)").expect("Invalid regex"));

static JWT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"eyJ[A-Za-z0-9-_]+\.eyJ[A-Za-z0-9-_]+\.[A-Za-z0-9-_]+").expect("Invalid regex")
});

/// Sanitize sensitive authentication tokens and JWTs from log or console text.
pub fn sanitize_text(text: &str) -> String {
    let scrubbed = BEARER_REGEX.replace_all(text, "$1[REDACTED_BEARER_TOKEN]");
    let scrubbed = ACCESS_TOKEN_PARAM_REGEX.replace_all(&scrubbed, "$1[REDACTED_TOKEN]");
    let scrubbed = JWT_REGEX.replace_all(&scrubbed, "[REDACTED_JWT]");
    scrubbed.into_owned()
}

/// Sanitize command-line arguments array before logging or displaying.
pub fn sanitize_cmd_args(args: &[String]) -> Vec<String> {
    let mut scrubbed = Vec::with_capacity(args.len());
    let mut redact_next = false;

    let sensitive_flags = [
        "--accessToken",
        "--uuid",
        "--xuid",
        "-Dminecraft.launcher.token",
    ];

    for arg in args {
        if redact_next {
            scrubbed.push("[REDACTED_PARAMETER]".to_string());
            redact_next = false;
            continue;
        }

        if sensitive_flags
            .iter()
            .any(|flag| arg.eq_ignore_ascii_case(flag))
        {
            scrubbed.push(arg.clone());
            redact_next = true;
            continue;
        }

        // Apply general text sanitization for inline flags like --accessToken=XYZ
        scrubbed.push(sanitize_text(arg));
    }

    scrubbed
}

#[cfg(test)]
mod tests {
    use super::*;

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
            "java".to_string(),
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
        assert_eq!(scrubbed[0], "java");
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
}
