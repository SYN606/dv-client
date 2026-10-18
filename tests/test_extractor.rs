//! Integration tests for SafeExtractor and Zip Slip prevention.

use dv_launcher::security::extract_natives;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use zip::write::SimpleFileOptions;

#[test]
fn test_safe_extraction_success() {
    let temp = tempdir().unwrap();
    let archive_path = temp.path().join("valid.zip");
    let dest_dir = temp.path().join("extracted");

    {
        let file = File::create(&archive_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        zip.start_file("natives/windows/lwjgl.dll", options).unwrap();
        zip.write_all(b"fake binary data").unwrap();

        zip.start_file("META-INF/MANIFEST.MF", options).unwrap();
        zip.write_all(b"Manifest-Version: 1.0").unwrap();
        zip.finish().unwrap();
    }

    let extracted = extract_natives(&archive_path, &dest_dir, &[]).unwrap();
    assert_eq!(extracted.len(), 1);
    assert!(dest_dir.join("natives/windows/lwjgl.dll").exists());
    assert!(!dest_dir.join("META-INF/MANIFEST.MF").exists());
}

#[test]
fn test_zip_slip_path_traversal_blocked() {
    let temp = tempdir().unwrap();
    let malicious_archive = temp.path().join("evil_slip.zip");
    let dest_dir = temp.path().join("sandbox");

    {
        let file = File::create(&malicious_archive).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        let _ = zip.start_file("../escaped_system_file.dll", options);
        let _ = zip.write_all(b"evil exploit payload");
        let _ = zip.finish();
    }

    let result = extract_natives(&malicious_archive, &dest_dir, &[]);
    assert!(result.is_err(), "Zip Slip path traversal must be rejected");
    assert!(!temp.path().join("escaped_system_file.dll").exists());
}
