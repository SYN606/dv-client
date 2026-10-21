//! Java Runtime Discovery and Version Inspection (compatibility module).

use std::path::Path;
pub use crate::java_detector::{JavaDetector, JavaInstallation};
pub type JavaRuntimeInfo = crate::java_detector::JavaInstallation;

pub struct JavaDiscovery;

impl JavaDiscovery {
    pub fn inspect_java_executable<P: AsRef<Path>>(path: P) -> Option<JavaRuntimeInfo> {
        crate::java_detector::JavaDetector::inspect_executable(path)
    }

    pub fn scan_system_runtimes() -> Vec<JavaRuntimeInfo> {
        crate::java_detector::JavaDetector::scan_system()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_java_discovery_compatibility() {
        let bogus = PathBuf::from("/non/existent/bogus/java");
        assert!(JavaDiscovery::inspect_java_executable(bogus).is_none());
    }
}
