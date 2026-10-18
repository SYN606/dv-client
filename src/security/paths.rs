// ! Safe archive extraction and filesystem boundary enforcement.
// !
// ! Provides strict Zip Slip / Path Traversal prevention when unpacking
// ! Minecraft native libraries and modpack archives.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use thiserror::Error;
use zip::ZipArchive;

#[derive(Debug, Error)]
pub enum ExtractionError {
    #[error("Archive file not found: {0}")]
    NotFound(PathBuf),
    #[error("I/O error during extraction: {0}")]
    Io(#[from] io::Error),
    #[error("ZIP format error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Zip Slip path traversal detected! Member '{member}' attempts to escape destination '{destination}'")]
    PathTraversal {
        member: String,
        destination: PathBuf,
    },
}

/// Safely extracts native library archives while preventing Zip Slip attacks.
pub fn extract_natives<P: AsRef<Path>, Q: AsRef<Path>>(
    archive_path: P,
    destination_dir: Q,
    exclude_patterns: &[&str],
) -> Result<Vec<PathBuf>, ExtractionError> {
    let archive_path = archive_path.as_ref();
    let destination_dir = destination_dir.as_ref();

    if !archive_path.exists() {
        return Err(ExtractionError::NotFound(archive_path.to_path_buf()));
    }

    fs::create_dir_all(destination_dir)?;
    let dest_canonical = dunce::canonicalize(destination_dir)?;
    let mut extracted_files = Vec::new();

    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    let default_excludes = ["META-INF/", ".git/"];
    let excludes: Vec<&str> = if exclude_patterns.is_empty() {
        default_excludes.to_vec()
    } else {
        exclude_patterns.to_vec()
    };

    for i in 0..archive.len() {
        let mut member = archive.by_index(i)?;
        let name = member.name().to_string();

        // Check if member should be excluded (e.g., META-INF)
        if excludes.iter().any(|pattern| name.starts_with(pattern)) {
            continue;
        }

        // Construct target path and canonicalize parent to detect Zip Slip
        let enclosed_path = match member.enclosed_name() {
            Some(path) => path.to_owned(),
            None => {
                return Err(ExtractionError::PathTraversal {
                    member: name,
                    destination: dest_canonical,
                });
            }
        };

        let target_path = dest_canonical.join(&enclosed_path);

        // Security check: verify that target path begins with dest_canonical
        if !target_path.starts_with(&dest_canonical) {
            return Err(ExtractionError::PathTraversal {
                member: name,
                destination: dest_canonical,
            });
        }

        if member.is_dir() {
            fs::create_dir_all(&target_path)?;
            continue;
        }

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut output_file = File::create(&target_path)?;
        let mut buffer = [0u8; 65536];
        loop {
            let bytes_read = member.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            io::Write::write_all(&mut output_file, &buffer[..bytes_read])?;
        }

        extracted_files.push(target_path);
    }

    Ok(extracted_files)
}

#[cfg(test)]
mod tests {
    use super::*;
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

            zip.start_file("natives/windows/lwjgl.dll", options)
                .unwrap();
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

            // Try to add traversal member
            let _ = zip.start_file("../escaped_system_file.dll", options);
            let _ = zip.write_all(b"evil exploit payload");
            let _ = zip.finish();
        }

        let result = extract_natives(&malicious_archive, &dest_dir, &[]);
        assert!(result.is_err(), "Extraction must fail on path traversal");
        assert!(!temp.path().join("escaped_system_file.dll").exists());
    }
}
