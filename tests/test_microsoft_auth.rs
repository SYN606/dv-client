//! Integration tests for Microsoft OAuth PKCE flow and account management.

use dv_launcher::accounts::{MicrosoftAuthClient, PkceChallenge};
use tempfile::tempdir;

#[test]
fn test_pkce_url_generation() {
    let pkce = PkceChallenge::generate();
    let url = pkce.build_login_url("00000000402b5328");

    assert!(url.contains("https://login.live.com/oauth20_authorize.srf"));
    assert!(url.contains("code_challenge="));
    assert!(url.contains("code_challenge_method=S256"));
    assert!(url.contains("response_type=code"));
    assert!(pkce.verifier.len() > 40);
}

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
