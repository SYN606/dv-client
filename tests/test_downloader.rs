//! Integration tests for AsyncDownloader and cryptographic verification.

use dv_launcher::minecraft::{AsyncDownloader, DownloadTask};
use dv_launcher::security::verify_file_sha1;
use sha1::{Digest, Sha1};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_file_verification_sha1() {
    let temp = tempdir().unwrap();
    let test_file = temp.path().join("sample.bin");
    let test_data = b"Digital Vigital high-performance download test payload";
    fs::write(&test_file, test_data).unwrap();

    let correct_sha1 = hex::encode(Sha1::digest(test_data));
    assert!(verify_file_sha1(&test_file, &correct_sha1));
    assert!(!verify_file_sha1(&test_file, "0000000000000000000000000000000000000000"));
}

#[tokio::test]
async fn test_skip_already_downloaded_file() {
    let temp = tempdir().unwrap();
    let existing_file = temp.path().join("already_done.jar");
    let data = b"Existing verified content";
    fs::write(&existing_file, data).unwrap();

    let correct_sha1 = hex::encode(Sha1::digest(data));
    let task = DownloadTask {
        url: "https://mock.invalid/file.jar".to_string(),
        destination: existing_file.clone(),
        sha1: Some(correct_sha1),
        sha256: None,
        expected_size: Some(data.len() as u64),
        display_name: Some("test.jar".to_string()),
    };

    let downloader = AsyncDownloader::new(4, 10, 3);
    let success = downloader.download_file(&task).await.unwrap();
    assert!(success);
}
