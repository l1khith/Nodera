use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Categorizes whether a Rust project is a single standalone crate or a Cargo workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    SinglePackage,
    Workspace,
}

impl std::fmt::Display for ProjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SinglePackage => write!(f, "Rust Package"),
            Self::Workspace => write!(f, "Rust Workspace"),
        }
    }
}

/// Identifies the semantic purpose of a discovered source root directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceRootKind {
    Src,
    Tests,
    Examples,
    Benches,
    Other,
}

/// A discovered source root directory within a Cargo package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRoot {
    pub path: PathBuf,
    pub kind: SourceRootKind,
}

/// Represents a single Cargo package (crate) discovered within the project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoPackage {
    pub name: String,
    pub version: Option<String>,
    pub edition: Option<String>,
    pub manifest_path: PathBuf,
    pub package_root: PathBuf,
    pub source_roots: Vec<SourceRoot>,
    pub source_files: Vec<PathBuf>,
}

impl CargoPackage {
    pub fn rust_file_count(&self) -> usize {
        self.source_files.len()
    }
}

/// Canonical typed representation of an inspected Rust project or workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustProject {
    pub root: PathBuf,
    pub name: String,
    pub kind: ProjectKind,
    pub edition: Option<String>,
    pub version: Option<String>,
    pub manifest_path: PathBuf,
    pub packages: Vec<CargoPackage>,
}

impl RustProject {
    /// Returns the total count of discovered Rust source files across all packages.
    pub fn total_rust_files(&self) -> usize {
        self.packages.iter().map(|p| p.rust_file_count()).sum()
    }

    /// Returns an iterator over all discovered Rust source file paths.
    pub fn all_rust_files(&self) -> Vec<&Path> {
        self.packages
            .iter()
            .flat_map(|p| p.source_files.iter().map(|f| f.as_path()))
            .collect()
    }

    /// True if the project is a multi-crate Cargo workspace.
    pub fn is_workspace(&self) -> bool {
        self.kind == ProjectKind::Workspace
    }

    /// Returns the total package (crate) count.
    pub fn package_count(&self) -> usize {
        self.packages.len()
    }
}

/// Persistent configuration stored at `.nodera/project.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectMetaConfig,
    #[serde(default = "ArtifactsConfig::default")]
    pub artifacts: ArtifactsConfig,
    #[serde(default = "StateConfig::default")]
    pub state: StateConfig,
    #[serde(default)]
    pub rust: Option<RustMetaConfig>,
}

fn default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMetaConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: String,
    #[serde(default)]
    pub root: PathBuf,
    #[serde(default = "default_manifest")]
    pub manifest: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub kind: Option<ProjectKind>,
    pub version: Option<String>,
    pub edition: Option<String>,
}

fn default_manifest() -> String {
    "Cargo.toml".to_string()
}

fn default_language() -> String {
    "rust".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactsConfig {
    pub graph: String,
    pub index: String,
}

impl Default for ArtifactsConfig {
    fn default() -> Self {
        Self {
            graph: ".nodera/graph/project.json".to_string(),
            index: ".nodera/index".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateConfig {
    pub initialized_at: String,
    pub last_updated_at: String,
}

impl Default for StateConfig {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            initialized_at: now.clone(),
            last_updated_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustMetaConfig {
    pub manifest: String,
    #[serde(default)]
    pub is_workspace: bool,
}

/// Generates a deterministic stable project ID based on project name and canonical root path.
pub fn generate_stable_project_id(root: &Path, name: &str) -> String {
    use sha2::{Digest, Sha256};
    let canonical = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let norm = canonical
        .to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches("//?/")
        .to_lowercase();
    let mut hasher = Sha256::new();
    hasher.update(norm.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    let clean_name = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>();
    format!("{}-{}", clean_name.trim_matches('-'), &hash[..10])
}

impl ProjectConfig {
    /// Constructs a configuration from a discovered project model.
    pub fn from_project(project: &RustProject) -> Self {
        let id = generate_stable_project_id(&project.root, &project.name);
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            project: ProjectMetaConfig {
                schema_version: 1,
                id,
                name: project.name.clone(),
                project_type: "rust".to_string(),
                root: project.root.clone(),
                manifest: "Cargo.toml".to_string(),
                language: "rust".to_string(),
                kind: Some(project.kind),
                version: project.version.clone(),
                edition: project.edition.clone(),
            },
            artifacts: ArtifactsConfig::default(),
            state: StateConfig {
                initialized_at: now.clone(),
                last_updated_at: now,
            },
            rust: Some(RustMetaConfig {
                manifest: "Cargo.toml".to_string(),
                is_workspace: project.is_workspace(),
            }),
        }
    }
}
