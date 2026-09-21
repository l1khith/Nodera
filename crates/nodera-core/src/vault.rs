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

/// Starter vault template presets for different knowledge workflow paradigms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StarterVaultPreset {
    #[default]
    Empty,
    Zettelkasten,
    PersonalKnowledge,
    Research,
    Student,
}

impl StarterVaultPreset {
    pub fn display_name(&self) -> &'static str {
        match self {
            StarterVaultPreset::Empty => "Empty Vault",
            StarterVaultPreset::Zettelkasten => "Zettelkasten Knowledge Vault",
            StarterVaultPreset::PersonalKnowledge => "Personal Knowledge (PARA)",
            StarterVaultPreset::Research => "Academic Research Vault",
            StarterVaultPreset::Student => "Student & Learning Vault",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            StarterVaultPreset::Empty => "Clean vault with standard Notes and Projects folders.",
            StarterVaultPreset::Zettelkasten => "Slip-box system with 00 Inbox, 01 Fleeting, 02 Literature, 03 Permanent, and 04 Index.",
            StarterVaultPreset::PersonalKnowledge => "Projects, Areas, Resources, and Archives system.",
            StarterVaultPreset::Research => "Literature review, raw source notes, synthesized manuscripts, and citations.",
            StarterVaultPreset::Student => "Courses, readings, lecture notes, and study guides.",
        }
    }

    pub fn folder_structure(&self) -> &'static [&'static str] {
        match self {
            StarterVaultPreset::Empty => DEFAULT_FOLDERS,
            StarterVaultPreset::Zettelkasten => &[
                "00 Inbox",
                "01 Fleeting",
                "02 Literature",
                "03 Permanent",
                "04 Index",
                "Templates",
                "Attachments",
            ],
            StarterVaultPreset::PersonalKnowledge => &[
                "00 Inbox",
                "01 Projects",
                "02 Areas",
                "03 Resources",
                "04 Archives",
                "Attachments",
            ],
            StarterVaultPreset::Research => &[
                "00 Inbox",
                "01 Sources",
                "02 Notes",
                "03 Manuscripts",
                "Templates",
                "Attachments",
            ],
            StarterVaultPreset::Student => &[
                "00 Inbox",
                "01 Courses",
                "02 Readings",
                "03 Lectures",
                "04 Exams",
                "Attachments",
            ],
        }
    }

    pub fn default_folder(&self) -> &'static str {
        match self {
            StarterVaultPreset::Empty => "Notes",
            _ => "00 Inbox",
        }
    }
}

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
    /// Creates a new vault with default empty layout.
    pub fn create(path: impl AsRef<Path>, name: Option<String>) -> Result<Self> {
        Self::create_with_preset(path, name, StarterVaultPreset::Empty)
    }

    /// Creates a new vault initialized according to the chosen starter preset.
    pub fn create_with_preset(
        path: impl AsRef<Path>,
        name: Option<String>,
        preset: StarterVaultPreset,
    ) -> Result<Self> {
        let root = path.as_ref().to_path_buf();
        info!(path = %root.display(), preset = ?preset, "Creating new vault");

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

        // Create preset subdirectories
        for folder in preset.folder_structure() {
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
            default_folder: preset.default_folder().to_string(),
        };

        let config_path = nodera_dir.join(CONFIG_FILE_NAME);
        let config_json =
            serde_json::to_string_pretty(&config).map_err(|source| VaultError::ConfigError {
                reason: format!("Failed to serialize config: {source}"),
            })?;

        fs::write(&config_path, config_json).map_err(|source| VaultError::ConfigError {
            reason: format!("Failed to write config file: {source}"),
        })?;

        // Write starter files based on chosen preset
        match preset {
            StarterVaultPreset::Zettelkasten => {
                let welcome = "---\ntitle: \"Welcome to Zettelkasten\"\ntype: rough\ntags:\n  - welcome\n  - inbox\n---\n# Welcome to your Zettelkasten Vault\n\nThis vault is configured for the Luhmann Zettelkasten method:\n- **00 Inbox**: Capture fleeting ideas, quick thoughts, and raw notes.\n- **01 Fleeting**: Thoughts awaiting elaboration.\n- **02 Literature**: Notes taken while consuming books, papers, articles, and videos.\n- **03 Permanent**: Atomic, synthesized notes in your own words, connected via `[[wikilinks]]`.\n- **04 Index**: High-level maps of content (MOC).\n";
                let _ = fs::write(canonical_root.join("00 Inbox").join("Welcome.md"), welcome);

                let index = "---\ntitle: \"Zettelkasten Master Index\"\ntype: index\ntags:\n  - index\n  - moc\n---\n# Zettelkasten Master Index\n\n> Central Map of Content (MOC) and topic index for this vault.\n\n## Core Concepts & Permanent Notes\n- [[Welcome]]\n\n## Literature & Sources\n- \n\n## Fleeting Thoughts in Progress\n- \n";
                let _ = fs::write(canonical_root.join("04 Index").join("Index.md"), index);

                let template = "---\ntitle: \"{{title}}\"\ntype: permanent\ncreated: \"{{date}}\"\ntags:\n  - permanent\n---\n# {{title}}\n\n## Core Idea\n\n## Context & Explanation\n\n## Connections & References\n- \n";
                let _ = fs::write(
                    canonical_root.join("Templates").join("Permanent Note.md"),
                    template,
                );
            }
            StarterVaultPreset::PersonalKnowledge => {
                let welcome = "---\ntitle: \"Welcome to Personal Knowledge\"\ntype: rough\ntags:\n  - welcome\n  - inbox\n---\n# Welcome to your Personal Knowledge Vault (PARA)\n\nOrganized using the PARA methodology:\n- **00 Inbox**: Quick thoughts and unsorted notes.\n- **01 Projects**: Active efforts with clear deadlines and outcomes.\n- **02 Areas**: Ongoing spheres of responsibility (Health, Career, Finance).\n- **03 Resources**: Topics of interest, references, and useful materials.\n- **04 Archives**: Inactive or completed projects.\n";
                let _ = fs::write(canonical_root.join("00 Inbox").join("Welcome.md"), welcome);
            }
            StarterVaultPreset::Research => {
                let welcome = "---\ntitle: \"Welcome to Academic Research\"\ntype: rough\ntags:\n  - research\n  - welcome\n---\n# Welcome to your Research Vault\n\n- **00 Inbox**: Raw literature queries and quick observations.\n- **01 Sources**: Summaries of papers, books, datasets, and citation notes.\n- **02 Notes**: Atomic synthesized conceptual notes.\n- **03 Manuscripts**: Drafts, papers, and presentations.\n";
                let _ = fs::write(canonical_root.join("00 Inbox").join("Welcome.md"), welcome);
            }
            StarterVaultPreset::Student => {
                let welcome = "---\ntitle: \"Welcome to Student Vault\"\ntype: rough\ntags:\n  - learning\n  - student\n---\n# Welcome to your Student Vault\n\n- **00 Inbox**: Quick capture during lectures and reading sessions.\n- **01 Courses**: Course syllabi and semester overviews.\n- **02 Readings**: Assigned textbook and paper notes.\n- **03 Lectures**: Dated lecture notes and discussions.\n- **04 Exams**: Study guides, flashcard concepts, and practice problems.\n";
                let _ = fs::write(canonical_root.join("00 Inbox").join("Welcome.md"), welcome);
            }
            StarterVaultPreset::Empty => {}
        }

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
    fn test_create_with_preset_zettelkasten() {
        let tmp = tempdir().unwrap();
        let vault_path = tmp.path().join("ZettelVault");

        let vault = Vault::create_with_preset(
            &vault_path,
            Some("Zettel".to_string()),
            StarterVaultPreset::Zettelkasten,
        )
        .unwrap();
        assert_eq!(vault.config().name, "Zettel");
        assert_eq!(vault.config().default_folder, "00 Inbox");

        assert!(vault.root().join("00 Inbox").is_dir());
        assert!(vault.root().join("01 Fleeting").is_dir());
        assert!(vault.root().join("02 Literature").is_dir());
        assert!(vault.root().join("03 Permanent").is_dir());
        assert!(vault.root().join("04 Index").is_dir());
        assert!(vault.root().join("04 Index").join("Index.md").is_file());
        assert!(vault.root().join("00 Inbox").join("Welcome.md").is_file());
        assert!(vault
            .root()
            .join("Templates")
            .join("Permanent Note.md")
            .is_file());
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
