//! Minecraft game installation, runtime discovery, metadata, and execution subsystem.

pub mod downloads;
pub mod java;
pub mod launch;
pub mod manifest;
pub mod metadata;
pub mod pipeline;
pub mod process;

pub use downloads::{AsyncDownloader, DownloadError, DownloadProgress, DownloadTask};
pub use java::{JavaDiscovery, JavaRuntimeInfo};
pub use launch::{get_performance_jvm_flags, LaunchArgumentBuilder, LaunchContext};
pub use manifest::{
    evaluate_rules, AssetIndexRef, DownloadArtifact, DownloadRule, LibraryEntry, ManifestLatest,
    VersionDownloads, VersionManifest, VersionPackage, VersionSummary,
};
pub use metadata::{MetaError, PistonMetaClient};
pub use pipeline::{InstallationPipeline, PipelineError};
pub use process::{ProcessError, ProcessState, ProcessSupervisor};
