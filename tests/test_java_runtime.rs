//! Integration tests for Java runtime discovery and inspection.

use dv_launcher::minecraft::JavaDiscovery;
use std::path::PathBuf;

#[test]
fn test_parse_modern_jdk21_output() {
    let output = r#"
openjdk version "21.0.2" 2024-01-16
OpenJDK Runtime Environment Temurin-21.0.2+13 (build 21.0.2+13)
OpenJDK 64-Bit Server VM Temurin-21.0.2+13 (build 21.0.2+13, mixed mode, sharing)
"#;
    let info = JavaDiscovery::parse_java_version_output(PathBuf::from("/usr/bin/java"), output).unwrap();
    assert_eq!(info.major_version, 21);
    assert_eq!(info.full_version, "21.0.2");
    assert!(info.is_64bit);
    assert_eq!(info.vendor, "Eclipse Temurin");
}

#[test]
fn test_parse_legacy_java8_output() {
    let output = r#"
java version "1.8.0_351"
Java(TM) SE Runtime Environment (build 1.8.0_351-b10)
Java HotSpot(TM) 64-Bit Server VM (build 25.351-b10, mixed mode)
"#;
    let info = JavaDiscovery::parse_java_version_output(PathBuf::from("C:\\Java\\bin\\java.exe"), output).unwrap();
    assert_eq!(info.major_version, 8);
    assert_eq!(info.full_version, "1.8.0_351");
    assert!(info.is_64bit);
}

#[test]
fn test_invalid_java_path() {
    let bogus = PathBuf::from("C:/NonExistentPath/java.exe");
    let info = JavaDiscovery::inspect_java_executable(bogus);
    assert!(info.is_none());
}
