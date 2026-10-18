//! Microsoft & Minecraft Account Authentication Subsystem.
//!
//! Handles Microsoft OAuth exchange, Xbox Live/XSTS/Minecraft Services tokens,
//! offline development accounts, profile persistence, and OS keyring storage.

use super::oauth::{
    PkceChallenge, DEFAULT_AZURE_CLIENT_ID, MC_AUTH_ENDPOINT, MC_PROFILE_ENDPOINT,
    REDIRECT_URI, TOKEN_ENDPOINT, XBL_AUTH_ENDPOINT, XSTS_AUTH_ENDPOINT,
};
use crate::branding;
use crate::security::CredentialVault;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Network error during authentication: {0}")]
    Network(#[from] reqwest::Error),
    #[error("I/O error during account persistence: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Credential vault error: {0}")]
    Vault(#[from] crate::security::credentials::CredentialError),
    #[error("Authentication protocol error: {0}")]
    Protocol(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MinecraftAccount {
    pub id: String, // Minecraft UUID
    pub username: String,
    #[serde(default = "default_account_type")]
    pub account_type: String, // "microsoft" or "offline"
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub last_login: u64,

    // In-memory volatile access token (never serialized to disk)
    #[serde(skip)]
    pub access_token: Option<String>,
}

fn default_account_type() -> String {
    "microsoft".to_string()
}

#[derive(Serialize, Deserialize)]
struct AccountsFilePayload {
    accounts: Vec<MinecraftAccount>,
}

pub struct MicrosoftAuthClient {
    pub client_id: String,
    pub accounts_file: PathBuf,
    pub accounts: HashMap<String, MinecraftAccount>,
    vault: CredentialVault,
    client: Client,
    current_pkce: Option<PkceChallenge>,
}

impl MicrosoftAuthClient {
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Self {
        let accounts_file = data_dir.as_ref().join("accounts.json");
        let client = Client::builder()
            .user_agent(branding::USER_AGENT)
            .timeout(Duration::from_secs(20))
            .build()
            .expect("Failed to initialize reqwest client for auth");

        let mut auth = Self {
            client_id: DEFAULT_AZURE_CLIENT_ID.to_string(),
            accounts_file,
            accounts: HashMap::new(),
            vault: CredentialVault::default(),
            client,
            current_pkce: None,
        };

        auth.load_accounts();
        auth
    }

    /// Load non-sensitive account metadata from disk.
    pub fn load_accounts(&mut self) {
        self.accounts.clear();
        if self.accounts_file.exists() {
            if let Ok(content) = fs::read_to_string(&self.accounts_file) {
                if let Ok(payload) = serde_json::from_str::<AccountsFilePayload>(&content) {
                    for acc in payload.accounts {
                        self.accounts.insert(acc.id.clone(), acc);
                    }
                }
            }
        }
    }

    /// Persist non-sensitive account metadata to disk (tokens are strictly excluded).
    pub fn save_accounts(&self) -> Result<(), AuthError> {
        if let Some(parent) = self.accounts_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let payload = AccountsFilePayload {
            accounts: self.accounts.values().cloned().collect(),
        };

        let json_data = serde_json::to_string_pretty(&payload)?;
        let mut file = File::create(&self.accounts_file)?;
        file.write_all(json_data.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// Generate a login URL with a fresh PKCE challenge.
    pub fn generate_login_url(&mut self) -> String {
        let pkce = PkceChallenge::generate();
        let url = pkce.build_login_url(&self.client_id);
        self.current_pkce = Some(pkce);
        url
    }

    /// Complete Microsoft authentication flow with received authorization code.
    pub async fn complete_login_with_code(&mut self, auth_code: &str) -> Result<MinecraftAccount, AuthError> {
        let pkce = self
            .current_pkce
            .take()
            .ok_or_else(|| AuthError::Protocol("No active PKCE challenge found".to_string()))?;

        // 1. Exchange OAuth code for MS token & refresh token
        let token_params = [
            ("client_id", self.client_id.as_str()),
            ("grant_type", "authorization_code"),
            ("code", auth_code),
            ("redirect_uri", REDIRECT_URI),
            ("code_verifier", pkce.verifier.as_str()),
        ];

        let ms_resp = self
            .client
            .post(TOKEN_ENDPOINT)
            .form(&token_params)
            .send()
            .await?
            .error_for_status()?;

        let ms_data: serde_json::Value = ms_resp.json().await?;
        let ms_access_token = ms_data["access_token"]
            .as_str()
            .ok_or_else(|| AuthError::Protocol("Missing access_token in MS response".to_string()))?;
        let refresh_token = ms_data.get("refresh_token").and_then(|r| r.as_str());

        // 2. Exchange for XBL token
        let xbl_payload = json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={}", ms_access_token)
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        });

        let xbl_resp = self
            .client
            .post(XBL_AUTH_ENDPOINT)
            .json(&xbl_payload)
            .send()
            .await?
            .error_for_status()?;

        let xbl_data: serde_json::Value = xbl_resp.json().await?;
        let xbl_token = xbl_data["Token"]
            .as_str()
            .ok_or_else(|| AuthError::Protocol("Missing XBL Token".to_string()))?;
        let user_hash = xbl_data["DisplayClaims"]["xui"][0]["uhs"]
            .as_str()
            .ok_or_else(|| AuthError::Protocol("Missing XBL user hash".to_string()))?;

        // 3. Exchange for XSTS token
        let xsts_payload = json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbl_token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        });

        let xsts_resp = self
            .client
            .post(XSTS_AUTH_ENDPOINT)
            .json(&xsts_payload)
            .send()
            .await?
            .error_for_status()?;

        let xsts_data: serde_json::Value = xsts_resp.json().await?;
        let xsts_token = xsts_data["Token"]
            .as_str()
            .ok_or_else(|| AuthError::Protocol("Missing XSTS Token".to_string()))?;

        // 4. Exchange for Minecraft Services Bearer Token
        let mc_payload = json!({
            "identityToken": format!("XBL3.0 x={};{}", user_hash, xsts_token)
        });

        let mc_resp = self
            .client
            .post(MC_AUTH_ENDPOINT)
            .json(&mc_payload)
            .send()
            .await?
            .error_for_status()?;

        let mc_data: serde_json::Value = mc_resp.json().await?;
        let mc_access_token = mc_data["access_token"]
            .as_str()
            .ok_or_else(|| AuthError::Protocol("Missing Minecraft access token".to_string()))?;

        // 5. Fetch Profile (UUID & Gamertag)
        let profile_resp = self
            .client
            .get(MC_PROFILE_ENDPOINT)
            .bearer_auth(mc_access_token)
            .send()
            .await?
            .error_for_status()?;

        let profile: serde_json::Value = profile_resp.json().await?;
        let account_id = profile["id"]
            .as_str()
            .ok_or_else(|| AuthError::Protocol("Missing player ID".to_string()))?
            .to_string();
        let username = profile["name"]
            .as_str()
            .ok_or_else(|| AuthError::Protocol("Missing player name".to_string()))?
            .to_string();

        // Securely store refresh token in OS Keyring
        if let Some(ref_token) = refresh_token {
            let _ = self.vault.save_refresh_token(&account_id, ref_token);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let account = MinecraftAccount {
            id: account_id.clone(),
            username,
            account_type: "microsoft".to_string(),
            avatar_url: Some(format!("https://crafthead.net/avatar/{}", account_id)),
            last_login: now,
            access_token: Some(mc_access_token.to_string()),
        };

        self.accounts.insert(account.id.clone(), account.clone());
        self.save_accounts()?;

        info!("Successfully authenticated Microsoft account: {}", account.username);
        Ok(account)
    }

    /// Create an offline development profile for local testing without network authentication.
    pub fn create_offline_account(&mut self, username: &str) -> Result<MinecraftAccount, AuthError> {
        let digest = md5_hash(format!("OfflinePlayer:{}", username).as_bytes());
        let hex_str = hex::encode(digest);
        let formatted_uuid = format!(
            "{}-{}-{}-{}-{}",
            &hex_str[0..8],
            &hex_str[8..12],
            &hex_str[12..16],
            &hex_str[16..20],
            &hex_str[20..32]
        );

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let account = MinecraftAccount {
            id: formatted_uuid.clone(),
            username: username.to_string(),
            account_type: "offline".to_string(),
            avatar_url: Some(format!("https://crafthead.net/avatar/{}", formatted_uuid)),
            last_login: now,
            access_token: Some("0".to_string()),
        };

        self.accounts.insert(account.id.clone(), account.clone());
        self.save_accounts()?;

        info!("Created offline account: {} ({})", account.username, account.id);
        Ok(account)
    }

    /// Purge stored credentials from OS keyring and delete account profile.
    pub fn logout(&mut self, account_id: &str) -> Result<bool, AuthError> {
        let _ = self.vault.delete_credentials(account_id);
        if self.accounts.remove(account_id).is_some() {
            self.save_accounts()?;
            info!("Logged out and purged account ID: {}", account_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_account(&self, account_id: &str) -> Option<&MinecraftAccount> {
        self.accounts.get(account_id)
    }

    pub fn get_active_account(&mut self, active_id: Option<&str>) -> MinecraftAccount {
        if let Some(id) = active_id {
            if let Some(acc) = self.accounts.get(id) {
                return acc.clone();
            }
        }
        if let Some(acc) = self.accounts.values().next() {
            return acc.clone();
        }
        // Fallback default
        self.create_offline_account("DVPlayer")
            .unwrap_or_else(|_| MinecraftAccount {
                id: "00000000-0000-0000-0000-000000000000".to_string(),
                username: "DVPlayer".to_string(),
                account_type: "offline".to_string(),
                avatar_url: None,
                last_login: 0,
                access_token: Some("0".to_string()),
            })
    }
}

/// Simple MD5 implementation for standard offline Minecraft player UUID derivation.
fn md5_hash(data: &[u8]) -> [u8; 16] {
    // Standard MD5 constants and round operations
    let mut state = [0x67452301u32, 0xefcdab89u32, 0x98badcfeu32, 0x10325476u32];
    let len = data.len();
    let mut padded = data.to_vec();
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }
    let bit_len = (len as u64) * 8;
    padded.extend_from_slice(&bit_len.to_le_bytes());

    for chunk in padded.chunks(64) {
        let mut w = [0u32; 16];
        for (i, w_val) in w.iter_mut().enumerate() {
            *w_val = u32::from_le_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];

        macro_rules! step {
            ($f:expr, $a:expr, $b:expr, $c:expr, $d:expr, $x:expr, $s:expr, $ac:expr) => {
                $a = $a.wrapping_add($f).wrapping_add($x).wrapping_add($ac);
                $a = $a.rotate_left($s).wrapping_add($b);
            };
        }

        // Round 1
        step!((b & c) | (!b & d), a, b, c, d, w[0], 7, 0xd76aa478);
        step!((a & b) | (!a & c), d, a, b, c, w[1], 12, 0xe8c7b756);
        step!((d & a) | (!d & b), c, d, a, b, w[2], 17, 0x242070db);
        step!((c & d) | (!c & a), b, c, d, a, w[3], 22, 0xc1bdceee);
        step!((b & c) | (!b & d), a, b, c, d, w[4], 7, 0xf57c0faf);
        step!((a & b) | (!a & c), d, a, b, c, w[5], 12, 0x4787c62a);
        step!((d & a) | (!d & b), c, d, a, b, w[6], 17, 0xa8304613);
        step!((c & d) | (!c & a), b, c, d, a, w[7], 22, 0xfd469501);
        step!((b & c) | (!b & d), a, b, c, d, w[8], 7, 0x698098d8);
        step!((a & b) | (!a & c), d, a, b, c, w[9], 12, 0x8b44f7af);
        step!((d & a) | (!d & b), c, d, a, b, w[10], 17, 0xffff5bb1);
        step!((c & d) | (!c & a), b, c, d, a, w[11], 22, 0x895cd7be);
        step!((b & c) | (!b & d), a, b, c, d, w[12], 7, 0x6b901122);
        step!((a & b) | (!a & c), d, a, b, c, w[13], 12, 0xfd987193);
        step!((d & a) | (!d & b), c, d, a, b, w[14], 17, 0xa679438e);
        step!((c & d) | (!c & a), b, c, d, a, w[15], 22, 0x49b40821);

        // Round 2
        step!((b & d) | (c & !d), a, b, c, d, w[1], 5, 0xf61e2562);
        step!((a & c) | (b & !c), d, a, b, c, w[6], 9, 0xc040b340);
        step!((d & b) | (a & !b), c, d, a, b, w[11], 14, 0x265e5a51);
        step!((c & a) | (d & !a), b, c, d, a, w[0], 20, 0xe9b6c7aa);
        step!((b & d) | (c & !d), a, b, c, d, w[5], 5, 0xd62f105d);
        step!((a & c) | (b & !c), d, a, b, c, w[10], 9, 0x02441453);
        step!((d & b) | (a & !b), c, d, a, b, w[15], 14, 0xd8a1e681);
        step!((c & a) | (d & !a), b, c, d, a, w[4], 20, 0xe7d3fbc8);
        step!((b & d) | (c & !d), a, b, c, d, w[9], 5, 0x21e1cde6);
        step!((a & c) | (b & !c), d, a, b, c, w[14], 9, 0xc33707d6);
        step!((d & b) | (a & !b), c, d, a, b, w[3], 14, 0xf4d50d87);
        step!((c & a) | (d & !a), b, c, d, a, w[8], 20, 0x455a14ed);
        step!((b & d) | (c & !d), a, b, c, d, w[13], 5, 0xa9e3e905);
        step!((a & c) | (b & !c), d, a, b, c, w[2], 9, 0xfcefa3f8);
        step!((d & b) | (a & !b), c, d, a, b, w[7], 14, 0x676f02d9);
        step!((c & a) | (d & !a), b, c, d, a, w[12], 20, 0x8d2a4c8a);

        // Round 3
        step!(b ^ c ^ d, a, b, c, d, w[5], 4, 0xfffa3942);
        step!(a ^ b ^ c, d, a, b, c, w[8], 11, 0x8771f681);
        step!(d ^ a ^ b, c, d, a, b, w[11], 16, 0x6d9d6122);
        step!(c ^ d ^ a, b, c, d, a, w[14], 23, 0xfde5380c);
        step!(b ^ c ^ d, a, b, c, d, w[1], 4, 0xa4beea44);
        step!(a ^ b ^ c, d, a, b, c, w[4], 11, 0x4bdecfa9);
        step!(d ^ a ^ b, c, d, a, b, w[7], 16, 0xf6bb4b60);
        step!(c ^ d ^ a, b, c, d, a, w[10], 23, 0xbebfbc70);
        step!(b ^ c ^ d, a, b, c, d, w[13], 4, 0x289b7ec6);
        step!(a ^ b ^ c, d, a, b, c, w[0], 11, 0xeaa127fa);
        step!(d ^ a ^ b, c, d, a, b, w[3], 16, 0xd4ef3085);
        step!(c ^ d ^ a, b, c, d, a, w[6], 23, 0x04881d05);
        step!(b ^ c ^ d, a, b, c, d, w[9], 4, 0xd9d4d039);
        step!(a ^ b ^ c, d, a, b, c, w[12], 11, 0xe6db99e5);
        step!(d ^ a ^ b, c, d, a, b, w[15], 16, 0x1fa27cf8);
        step!(c ^ d ^ a, b, c, d, a, w[2], 23, 0xc4ac5665);

        // Round 4
        step!(c ^ (b | !d), a, b, c, d, w[0], 6, 0xf4292244);
        step!(b ^ (a | !c), d, a, b, c, w[7], 10, 0x432aff97);
        step!(a ^ (d | !b), c, d, a, b, w[14], 15, 0xab9423a7);
        step!(d ^ (c | !a), b, c, d, a, w[5], 21, 0xfc93a039);
        step!(c ^ (b | !d), a, b, c, d, w[12], 6, 0x655b59c3);
        step!(b ^ (a | !c), d, a, b, c, w[3], 10, 0x8f0ccc92);
        step!(a ^ (d | !b), c, d, a, b, w[10], 15, 0xffeff47d);
        step!(d ^ (c | !a), b, c, d, a, w[1], 21, 0x85845dd1);
        step!(c ^ (b | !d), a, b, c, d, w[8], 6, 0x6fa87e4f);
        step!(b ^ (a | !c), d, a, b, c, w[15], 10, 0xfe2ce6e0);
        step!(a ^ (d | !b), c, d, a, b, w[6], 15, 0xa3014314);
        step!(d ^ (c | !a), b, c, d, a, w[13], 21, 0x4e0811a1);
        step!(c ^ (b | !d), a, b, c, d, w[4], 6, 0xf7537e82);
        step!(b ^ (a | !c), d, a, b, c, w[11], 10, 0xbd3af235);
        step!(a ^ (d | !b), c, d, a, b, w[2], 15, 0x2ad7d2bb);
        step!(d ^ (c | !a), b, c, d, a, w[9], 21, 0xeb86d391);

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
    }

    let mut res = [0u8; 16];
    for (i, val) in state.iter().enumerate() {
        let b = val.to_le_bytes();
        res[i * 4..i * 4 + 4].copy_from_slice(&b);
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_offline_account_creation_and_logout() {
        let temp = tempdir().unwrap();
        let mut client = MicrosoftAuthClient::new(temp.path());
        let acc = client.create_offline_account("CyberTester").unwrap();

        assert_eq!(acc.username, "CyberTester");
        assert_eq!(acc.account_type, "offline");
        assert_eq!(acc.id.len(), 36); // Valid UUID representation
        assert!(client.accounts.contains_key(&acc.id));

        // Test logout / purge
        let logged_out = client.logout(&acc.id).unwrap();
        assert!(logged_out);
        assert!(!client.accounts.contains_key(&acc.id));
    }
}
