//! Cryptographic hashing and verification utilities.

use sha1::{Digest as Sha1Digest, Sha1};
use sha2::Sha256;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HashError {
    #[error("I/O error during hash computation: {0}")]
    Io(#[from] io::Error),
    #[error("SHA-1 mismatch: expected {expected}, got {actual}")]
    Sha1Mismatch { expected: String, actual: String },
    #[error("SHA-256 mismatch: expected {expected}, got {actual}")]
    Sha256Mismatch { expected: String, actual: String },
    #[error("File size mismatch: expected {expected} bytes, got {actual} bytes")]
    SizeMismatch { expected: u64, actual: u64 },
}

/// Compute hex-encoded SHA-1 digest of a file.
pub fn compute_file_sha1<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 65536];

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Compute hex-encoded SHA-256 digest of a file.
pub fn compute_file_sha256<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Verify that an existing file matches the expected SHA-1 digest.
pub fn verify_file_sha1<P: AsRef<Path>>(path: P, expected_sha1: &str) -> bool {
    match compute_file_sha1(path) {
        Ok(actual) => actual.eq_ignore_ascii_case(expected_sha1),
        Err(_) => false,
    }
}

/// Verify that an existing file matches the expected SHA-256 digest.
pub fn verify_file_sha256<P: AsRef<Path>>(path: P, expected_sha256: &str) -> bool {
    match compute_file_sha256(path) {
        Ok(actual) => actual.eq_ignore_ascii_case(expected_sha256),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha1_verification() {
        let mut tmp = NamedTempFile::new().unwrap();
        let data = b"Digital Vigital high-performance download test payload";
        tmp.write_all(data).unwrap();

        let sha1 = compute_file_sha1(tmp.path()).unwrap();
        assert!(verify_file_sha1(tmp.path(), &sha1));
        assert!(!verify_file_sha1(
            tmp.path(),
            "0000000000000000000000000000000000000000"
        ));
    }

    #[test]
    fn test_sha256_verification() {
        let mut tmp = NamedTempFile::new().unwrap();
        let data = b"Digital Vigital high-performance download test payload";
        tmp.write_all(data).unwrap();

        let sha256 = compute_file_sha256(tmp.path()).unwrap();
        assert!(verify_file_sha256(tmp.path(), &sha256));
    }
}
