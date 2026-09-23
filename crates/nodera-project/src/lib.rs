pub mod discovery;
pub mod error;
pub mod graph;
pub mod index;
pub mod init;
pub mod manifest;
pub mod model;
pub mod registry;
pub mod state;
pub mod sync;

pub use discovery::ProjectDiscovery;
pub use error::{ProjectError, Result};
pub use graph::{
    FileMetrics, ProjectEdge, ProjectEdgeKind, ProjectFileNode, ProjectGraph, ProjectHeader,
    ProjectNode, ProjectNodeKind,
};
pub use index::ProjectIndex;
pub use init::{InitOptions, InitResult, InitStatus, ProjectInitializer};
pub use manifest::RawManifest;
pub use model::{
    generate_stable_project_id, ArtifactsConfig, CargoPackage, ProjectConfig, ProjectKind,
    ProjectMetaConfig, RustMetaConfig, RustProject, SourceRoot, SourceRootKind, StateConfig,
};
pub use registry::{ProjectRegistry, ProjectRegistryData, ProjectRegistryEntry};
pub use state::{compute_file_hash, ChangeSet, FileStateRecord, SourceState};
pub use sync::{ProjectSynchronizer, SyncResult};
