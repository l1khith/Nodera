use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{ProjectError, Result};

/// An external project registered with Nodera.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRegistryEntry {
    pub id: String,
    pub name: String,
    pub root: PathBuf,
    pub project_type: String,
    pub last_updated: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRegistryData {
    pub version: u32,
    pub projects: Vec<ProjectRegistryEntry>,
}

impl Default for ProjectRegistryData {
    fn default() -> Self {
        Self {
            version: 1,
            projects: Vec::new(),
        }
    }
}

pub struct ProjectRegistry;

impl ProjectRegistry {
    /// Resolves the filesystem path to the user's global `projects_registry.json`.
    pub fn registry_path() -> PathBuf {
        if let Some(custom) = std::env::var_os("NODERA_PROJECTS_REGISTRY_PATH") {
            return PathBuf::from(custom);
        }

        if cfg!(test) || std::env::var_os("NODERA_TEST").is_some() {
            return std::env::temp_dir().join("nodera_test_projects_registry.json");
        }

        let base_dir = std::env::var_os("APPDATA")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        base_dir.join(".nodera").join("projects_registry.json")
    }

    /// Loads the project registry.
    pub fn load() -> Result<ProjectRegistryData> {
        let path = Self::registry_path();
        if !path.exists() {
            return Ok(ProjectRegistryData::default());
        }

        let content = fs::read_to_string(&path).map_err(|e| ProjectError::Io {
            path: path.clone(),
            source: e,
        })?;

        serde_json::from_str(&content).map_err(|e| ProjectError::CorruptedState {
            path,
            reason: format!("Failed to parse projects registry: {e}"),
        })
    }

    /// Saves the project registry.
    pub fn save(data: &ProjectRegistryData) -> Result<()> {
        let path = Self::registry_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| ProjectError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let json = serde_json::to_string_pretty(data).map_err(|e| ProjectError::Serialization {
            reason: format!("Failed to serialize projects registry: {e}"),
        })?;

        fs::write(&path, json).map_err(|e| ProjectError::Io { path, source: e })
    }

    /// Registers or updates an external project entry.
    pub fn register(entry: ProjectRegistryEntry) -> Result<()> {
        let mut data = Self::load()?;
        data.projects
            .retain(|p| p.id != entry.id && p.root != entry.root);
        data.projects.push(entry);
        Self::save(&data)
    }

    /// Unregisters a project by ID or canonical root path.
    pub fn unregister(id_or_path: &str) -> Result<()> {
        let mut data = Self::load()?;
        let target_path = PathBuf::from(id_or_path);
        data.projects
            .retain(|p| p.id != id_or_path && p.root != target_path);
        Self::save(&data)
    }

    /// Returns all registered projects.
    pub fn list() -> Result<Vec<ProjectRegistryEntry>> {
        let data = Self::load()?;
        Ok(data.projects)
    }

    /// Finds a registered project by its stable ID.
    pub fn find_by_id(id: &str) -> Result<Option<ProjectRegistryEntry>> {
        let data = Self::load()?;
        Ok(data.projects.into_iter().find(|p| p.id == id))
    }

    /// Finds a registered project by its canonical root path.
    pub fn find_by_root(root: &Path) -> Result<Option<ProjectRegistryEntry>> {
        let data = Self::load()?;
        Ok(data.projects.into_iter().find(|p| p.root == root))
    }

    /// Prunes entries whose root directories no longer exist on disk.
    pub fn prune_missing() -> Result<usize> {
        let mut data = Self::load()?;
        let initial_len = data.projects.len();
        data.projects.retain(|p| p.root.exists());
        let pruned = initial_len - data.projects.len();
        if pruned > 0 {
            Self::save(&data)?;
        }
        Ok(pruned)
    }
}
