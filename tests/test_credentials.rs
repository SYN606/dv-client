//! Integration tests for CredentialVault and OS keyring validation.

use dv_launcher::security::CredentialVault;

#[test]
fn test_credential_vault_validation() {
    let vault = CredentialVault::new("com.digitalvigital.launcher.test");
    assert!(vault.save_refresh_token("", "token").is_err());
    assert!(vault.save_refresh_token("id", "").is_err());
    assert!(vault.get_refresh_token("").is_err());
    assert!(vault.delete_credentials("").is_err());
}
