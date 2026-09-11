use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};
use tracing::{debug, info};

use crate::error::{Result, VaultError};

pub const CURRENT_VAULT_VERSION: u32 = 1;
pub const NODERA_DIR_NAME: &str = ".nodera";
pub const CONFIG_FILE_NAME: &str = "config.json";

/// Standard directory layout for a Nodera vault.
pub const DEFAULT_FOLDERS: &[&str] = &["Notes", "Projects", "Books", "Attachments"];

/// Persistent configuration stored inside `<vault>/.nodera/config.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultConfig {
    pub version: u32,
    pub name: String,
    pub default_folder: String,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            version: CURRENT_VAULT_VERSION,
            name: "My Vault".to_string(),
            default_folder: "Notes".to_string(),
        }
    }
}

/// Represents an opened or created local vault.
#[derive(Debug, Clone)]
pub struct Vault {
    root: PathBuf,
    config: VaultConfig,
}

impl Vault {
    /// Creates a new vault at the specified root path.
    ///
    /// Initializes standard directories (`Notes`, `Projects`, `Books`, `Attachments`, `.nodera`)
    /// and writes the vault configuration.
    pub fn create(path: impl AsRef<Path>, name: Option<String>) -> Result<Self> {
        let root = path.as_ref().to_path_buf();
        info!(path = %root.display(), "Creating new vault");

        if root.exists() {
            if !root.is_dir() {
                return Err(VaultError::NotADirectory { path: root }.into());
            }

            // If .nodera already exists and contains config.json, treat as already existing vault
            let nodera_dir = root.join(NODERA_DIR_NAME);
            if nodera_dir.join(CONFIG_FILE_NAME).exists() {
                return Err(VaultError::AlreadyExists { path: root }.into());
            }
        } else {
            fs::create_dir_all(&root).map_err(|source| VaultError::InvalidStructure {
                path: root.clone(),
                reason: format!("Failed to create root directory: {source}"),
            })?;
        }

        // Canonicalize root for robust traversal prevention
        let canonical_root =
            fs::canonicalize(&root).map_err(|source| VaultError::InvalidStructure {
                path: root.clone(),
                reason: format!("Failed to canonicalize root: {source}"),
            })?;

        // Create standard subdirectories
        for folder in DEFAULT_FOLDERS {
            let folder_path = canonical_root.join(folder);
            if !folder_path.exists() {
                fs::create_dir_all(&folder_path).map_err(|source| {
                    VaultError::InvalidStructure {
                        path: folder_path,
                        reason: format!("Failed to create folder '{folder}': {source}"),
                    }
                })?;
            }
        }

        // Create .nodera directory
        let nodera_dir = canonical_root.join(NODERA_DIR_NAME);
        if !nodera_dir.exists() {
            fs::create_dir_all(&nodera_dir).map_err(|source| VaultError::InvalidStructure {
                path: nodera_dir.clone(),
                reason: format!("Failed to create .nodera directory: {source}"),
            })?;
        }

        let vault_name = name.unwrap_or_else(|| {
            canonical_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("My Vault")
                .to_string()
        });

        let config = VaultConfig {
            version: CURRENT_VAULT_VERSION,
            name: vault_name,
            default_folder: "Notes".to_string(),
        };

        let config_path = nodera_dir.join(CONFIG_FILE_NAME);
        let config_json =
            serde_json::to_string_pretty(&config).map_err(|source| VaultError::ConfigError {
                reason: format!("Failed to serialize config: {source}"),
            })?;

        fs::write(&config_path, config_json).map_err(|source| VaultError::ConfigError {
            reason: format!("Failed to write config file: {source}"),
        })?;

        debug!(path = %canonical_root.display(), "Vault created successfully");
        Ok(Self {
            root: canonical_root,
            config,
        })
    }

    /// Opens an existing vault at the given path.
    ///
    /// If `.nodera/config.json` exists, loads and verifies configuration.
    /// If opening an existing folder without `.nodera`, initializes `.nodera` and default configuration
    /// so user notes remain intact without loss.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let raw_path = path.as_ref();
        if !raw_path.exists() {
            return Err(VaultError::NotFound {
                path: raw_path.to_path_buf(),
            }
            .into());
        }

        if !raw_path.is_dir() {
            return Err(VaultError::NotADirectory {
                path: raw_path.to_path_buf(),
            }
            .into());
        }

        let canonical_root =
            fs::canonicalize(raw_path).map_err(|source| VaultError::InvalidStructure {
                path: raw_path.to_path_buf(),
                reason: format!("Failed to canonicalize vault path: {source}"),
            })?;

        let nodera_dir = canonical_root.join(NODERA_DIR_NAME);
        let config_path = nodera_dir.join(CONFIG_FILE_NAME);

        let config = if config_path.exists() {
            let content =
                fs::read_to_string(&config_path).map_err(|source| VaultError::ConfigError {
                    reason: format!(
                        "Failed to read config file {}: {source}",
                        config_path.display()
                    ),
                })?;
            serde_json::from_str::<VaultConfig>(&content).map_err(|source| {
                VaultError::ConfigError {
                    reason: format!("Failed to parse config file: {source}"),
                }
            })?
        } else {
            // Folder exists but lacks .nodera metadata - gracefully initialize .nodera
            if !nodera_dir.exists() {
                fs::create_dir_all(&nodera_dir).map_err(|source| VaultError::InvalidStructure {
                    path: nodera_dir.clone(),
                    reason: format!("Failed to create .nodera directory: {source}"),
                })?;
            }

            let vault_name = canonical_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("My Vault")
                .to_string();

            let config = VaultConfig {
                version: CURRENT_VAULT_VERSION,
                name: vault_name,
                default_folder: "Notes".to_string(),
            };

            let config_json = serde_json::to_string_pretty(&config).map_err(|source| {
                VaultError::ConfigError {
                    reason: format!("Failed to serialize config: {source}"),
                }
            })?;

            fs::write(&config_path, config_json).map_err(|source| VaultError::ConfigError {
                reason: format!("Failed to write initial config: {source}"),
            })?;

            config
        };

        info!(
            path = %canonical_root.display(),
            name = %config.name,
            "Opened vault"
        );

        Ok(Self {
            root: canonical_root,
            config,
        })
    }

    /// Returns the root path of the vault.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the active vault configuration.
    pub fn config(&self) -> &VaultConfig {
        &self.config
    }

    /// Returns the path to the `.nodera` internal directory.
    pub fn nodera_dir(&self) -> PathBuf {
        self.root.join(NODERA_DIR_NAME)
    }

    /// Resolves a vault-relative path and strictly protects against path traversal.
    ///
    /// The relative path must not contain parent components (`..`) that escape the vault root.
    pub fn resolve_path(&self, relative: impl AsRef<Path>) -> Result<PathBuf> {
        let relative = relative.as_ref();

        // Check for suspicious components
        for comp in relative.components() {
            match comp {
                Component::Prefix(_) | Component::RootDir => {
                    // Absolute path attempt
                    return Err(VaultError::PathTraversal {
                        attempted_path: relative.to_path_buf(),
                        vault_root: self.root.clone(),
                    }
                    .into());
                }
                Component::ParentDir => {
                    // Deny parent dir component in relative path input
                    return Err(VaultError::PathTraversal {
                        attempted_path: relative.to_path_buf(),
                        vault_root: self.root.clone(),
                    }
                    .into());
                }
                Component::CurDir | Component::Normal(_) => {}
            }
        }

        let full_path = self.root.join(relative);
        Ok(full_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_new_vault() {
        let tmp = tempdir().unwrap();
        let vault_path = tmp.path().join("TestVault");

        let vault = Vault::create(&vault_path, Some("Test Vault".to_string())).unwrap();
        assert_eq!(vault.config().name, "Test Vault");
        assert_eq!(vault.config().version, CURRENT_VAULT_VERSION);

        // Verify subdirectories
        assert!(vault.root().join("Notes").is_dir());
        assert!(vault.root().join("Projects").is_dir());
        assert!(vault.root().join("Books").is_dir());
        assert!(vault.root().join("Attachments").is_dir());
        assert!(vault.nodera_dir().is_dir());
        assert!(vault.nodera_dir().join(CONFIG_FILE_NAME).is_file());
    }

    #[test]
    fn test_create_already_existing_vault_fails() {
        let tmp = tempdir().unwrap();
        let vault_path = tmp.path().join("ExistingVault");

        let _ = Vault::create(&vault_path, None).unwrap();
        let second_attempt = Vault::create(&vault_path, None);
        assert!(second_attempt.is_err());
    }

    #[test]
    fn test_open_existing_vault() {
        let tmp = tempdir().unwrap();
        let vault_path = tmp.path().join("OpenVault");

        let created = Vault::create(&vault_path, Some("Open Me".to_string())).unwrap();
        drop(created);

        let opened = Vault::open(&vault_path).unwrap();
        assert_eq!(opened.config().name, "Open Me");
    }

    #[test]
    fn test_open_nonexistent_vault_fails() {
        let tmp = tempdir().unwrap();
        let nonexistent = tmp.path().join("DoesNotExist");

        let result = Vault::open(&nonexistent);
        assert!(result.is_err());
        match result.err().unwrap() {
            crate::error::NoderaError::Vault(VaultError::NotFound { .. }) => {}
            other => panic!("Expected NotFound, got: {other:?}"),
        }
    }

    #[test]
    fn test_open_file_as_vault_fails() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("file.txt");
        fs::write(&file_path, "hello").unwrap();

        let result = Vault::open(&file_path);
        assert!(result.is_err());
        match result.err().unwrap() {
            crate::error::NoderaError::Vault(VaultError::NotADirectory { .. }) => {}
            other => panic!("Expected NotADirectory, got: {other:?}"),
        }
    }

    #[test]
    fn test_resolve_path_safe() {
        let tmp = tempdir().unwrap();
        let vault_path = tmp.path().join("SafeVault");
        let vault = Vault::create(&vault_path, None).unwrap();

        let resolved = vault.resolve_path("Notes/Meeting.md").unwrap();
        assert_eq!(resolved, vault.root().join("Notes/Meeting.md"));
    }

    #[test]
    fn test_resolve_path_traversal_rejected() {
        let tmp = tempdir().unwrap();
        let vault_path = tmp.path().join("SafeVault");
        let vault = Vault::create(&vault_path, None).unwrap();

        assert!(vault.resolve_path("../outside.md").is_err());
        assert!(vault.resolve_path("../../etc/passwd").is_err());
        assert!(vault.resolve_path("/absolute/path.md").is_err());
    }
}
