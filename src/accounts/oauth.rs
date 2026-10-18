//! Microsoft OAuth 2.0 PKCE Code Challenge Generator & Endpoints.

use sha2::{Digest, Sha256};
use url::Url;
use uuid::Uuid;

pub const DEFAULT_AZURE_CLIENT_ID: &str = "00000000402b5328";
pub const REDIRECT_URI: &str = "https://login.live.com/oauth20_desktop.srf";
pub const OAUTH_SCOPE: &str = "XboxLive.signin offline_access";

pub const AUTH_ENDPOINT: &str = "https://login.live.com/oauth20_authorize.srf";
pub const TOKEN_ENDPOINT: &str = "https://login.live.com/oauth20_token.srf";
pub const XBL_AUTH_ENDPOINT: &str = "https://user.auth.xboxlive.com/user/authenticate";
pub const XSTS_AUTH_ENDPOINT: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
pub const MC_AUTH_ENDPOINT: &str =
    "https://api.minecraftservices.com/authentication/login_with_xbox";
pub const MC_PROFILE_ENDPOINT: &str = "https://api.minecraftservices.com/minecraft/profile";

pub struct PkceChallenge {
    pub verifier: String,
    pub challenge: String,
}

impl PkceChallenge {
    /// Generate a cryptographically secure PKCE verifier and S256 code challenge.
    pub fn generate() -> Self {
        // High-entropy 64-character verifier
        let verifier = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let digest = hasher.finalize();

        // Base64-url encoding without padding
        let b64 = {
            const URL_SAFE: &[u8] =
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
            let mut s = String::new();
            let mut i = 0;
            while i < digest.len() {
                let b0 = digest[i];
                let b1 = if i + 1 < digest.len() {
                    digest[i + 1]
                } else {
                    0
                };
                let b2 = if i + 2 < digest.len() {
                    digest[i + 2]
                } else {
                    0
                };

                s.push(URL_SAFE[(b0 >> 2) as usize] as char);
                s.push(URL_SAFE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
                if i + 1 < digest.len() {
                    s.push(URL_SAFE[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
                }
                if i + 2 < digest.len() {
                    s.push(URL_SAFE[(b2 & 0x3f) as usize] as char);
                }
                i += 3;
            }
            s
        };

        Self {
            verifier,
            challenge: b64,
        }
    }

    /// Construct Microsoft OAuth login URL.
    pub fn build_login_url(&self, client_id: &str) -> String {
        let mut url = Url::parse(AUTH_ENDPOINT).expect("Invalid auth endpoint");
        url.query_pairs_mut()
            .append_pair("client_id", client_id)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", REDIRECT_URI)
            .append_pair("scope", OAUTH_SCOPE)
            .append_pair("code_challenge", &self.challenge)
            .append_pair("code_challenge_method", "S256");

        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_url_generation() {
        let pkce = PkceChallenge::generate();
        let url = pkce.build_login_url(DEFAULT_AZURE_CLIENT_ID);

        assert!(url.contains("https://login.live.com/oauth20_authorize.srf"));
        assert!(url.contains("code_challenge="));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("response_type=code"));
        assert!(pkce.verifier.len() > 40);
    }
}
