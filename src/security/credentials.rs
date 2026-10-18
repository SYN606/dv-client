//! Operating-System-Backed Hardware Keyring Vault.
//!
//! Secures Microsoft OAuth refresh tokens directly in:
//! - Linux: Secret Service API via DBus (`zbus`)
//! - Windows: Windows Credential Manager / DPAPI
//! - macOS: Apple Keychain Services
//!
//! Eliminates static XOR masking and plaintext storage completely.

use keyring::Entry;
use thiserror::Error;
use tracing::{error, info, warn};

pub const DEFAULT_SERVICE_NAME: &str = "com.digitalvigital.launcher";

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("Account ID and token must not be empty")]
    InvalidInput,
    #[error("Keyring backend error: {0}")]
    Backend(#[from] keyring::Error),
}

#[derive(Clone, Debug)]
pub struct CredentialVault {
    pub service_name: String,
}

impl Default for CredentialVault {
    fn default() -> Self {
        Self::new(DEFAULT_SERVICE_NAME)
    }
}

impl CredentialVault {
    pub fn new<S: Into<String>>(service_name: S) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }

    /// Save a refresh token to the platform keyring.
    pub fn save_refresh_token(&self, account_id: &str, token: &str) -> Result<(), CredentialError> {
        if account_id.trim().is_empty() || token.trim().is_empty() {
            return Err(CredentialError::InvalidInput);
        }

        let entry = Entry::new(&self.service_name, account_id)?;
        entry.set_password(token)?;
        info!(
            "Refresh token securely stored in OS keyring for account ID: {}",
            account_id
        );
        Ok(())
    }

    /// Retrieve an existing refresh token from the platform keyring.
    pub fn get_refresh_token(&self, account_id: &str) -> Result<Option<String>, CredentialError> {
        if account_id.trim().is_empty() {
            return Err(CredentialError::InvalidInput);
        }

        let entry = Entry::new(&self.service_name, account_id)?;
        match entry.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => {
                warn!(
                    "Could not read credential for account {}: {}",
                    account_id, e
                );
                Err(CredentialError::Backend(e))
            }
        }
    }

    /// Delete credentials from the platform keyring upon logout.
    pub fn delete_credentials(&self, account_id: &str) -> Result<bool, CredentialError> {
        if account_id.trim().is_empty() {
            return Err(CredentialError::InvalidInput);
        }

        let entry = Entry::new(&self.service_name, account_id)?;
        match entry.delete_credential() {
            Ok(_) => {
                info!(
                    "Deleted credentials from OS keyring for account ID: {}",
                    account_id
                );
                Ok(true)
            }
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(e) => {
                error!(
                    "Failed to delete credentials for account {}: {}",
                    account_id, e
                );
                Err(CredentialError::Backend(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_validation() {
        let vault = CredentialVault::new("com.digitalvigital.launcher.test");
        assert!(vault.save_refresh_token("", "token").is_err());
        assert!(vault.save_refresh_token("id", "").is_err());
        assert!(vault.get_refresh_token("").is_err());
        assert!(vault.delete_credentials("").is_err());
    }
}
