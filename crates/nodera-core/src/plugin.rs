use crate::error::{NoderaError, PluginError, Result, ValidationError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Permissions requested by a Nodera plugin to perform actions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PluginPermission {
    /// Permission to read note files and frontmatter.
    ReadNotes,
    /// Permission to write, edit, or create note files.
    WriteNotes,
    /// Permission to register custom commands executable via Command Palette.
    Commands,
    /// Permission to trigger UI status notifications.
    UiNotice,
}

/// Metadata specification parsed from `.nodera/plugins/<id>/plugin.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginManifest {
    /// Unique identifier for the plugin (e.g., "word-count-pro", "auto-tagger").
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Semantic version string (e.g., "1.0.0").
    pub version: String,
    /// Brief description of the plugin's functionality.
    #[serde(default)]
    pub description: String,
    /// Author or contributor name.
    #[serde(default)]
    pub author: String,
    /// Entrypoint script or rule configuration file relative to plugin folder (e.g. "main.json" or "plugin.lua").
    pub main: String,
    /// Declared permissions required by this plugin.
    #[serde(default)]
    pub permissions: Vec<PluginPermission>,
    /// Whether this plugin is currently enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// Actionable custom command registered by a plugin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginCommand {
    pub plugin_id: String,
    pub command_id: String,
    pub name: String,
    pub description: String,
}

/// Lifecycle hooks dispatched to plugins by the application shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginHook<'a> {
    VaultOpened {
        vault_name: &'a str,
    },
    NoteSaved {
        relative_path: &'a Path,
        content: &'a str,
    },
    ExecuteCommand {
        command_id: &'a str,
        args: &'a [String],
    },
}

/// Result returned from executing a plugin hook or command.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HookExecutionResult {
    /// Status or notification messages generated for the user.
    pub notices: Vec<String>,
    /// Optional transformed note content if a modifier plugin updated text.
    pub modified_content: Option<String>,
    /// Output payload from a custom command.
    pub command_output: Option<String>,
}

/// Host environment that discovers, validates, and manages plugins for a vault.
#[derive(Debug, Clone, Default)]
pub struct PluginManager {
    plugins: HashMap<String, PluginManifest>,
    plugin_dirs: HashMap<String, PathBuf>,
    registered_commands: Vec<PluginCommand>,
}

impl PluginManager {
    /// Creates an empty plugin manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Discovers all installed plugins within `<vault_root>/.nodera/plugins/`.
    pub fn discover_from_vault(&mut self, vault_root: &Path) -> Result<usize> {
        let plugins_dir = vault_root.join(".nodera").join("plugins");
        if !plugins_dir.exists() {
            return Ok(0);
        }

        let mut discovered = 0;
        let read_dir = std::fs::read_dir(&plugins_dir).map_err(|e| {
            NoderaError::Validation(ValidationError::InvalidPath {
                path: plugins_dir.clone(),
                reason: format!("Failed to read plugins directory: {e}"),
            })
        })?;

        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("plugin.json");
                if manifest_path.is_file() {
                    match self.load_plugin_manifest(&manifest_path) {
                        Ok(manifest) => {
                            let id = manifest.id.clone();
                            self.plugin_dirs.insert(id.clone(), path);
                            self.register_plugin_commands(&manifest);
                            self.plugins.insert(id, manifest);
                            discovered += 1;
                        }
                        Err(e) => {
                            warn!(
                                "Failed to load plugin manifest at {}: {e}",
                                manifest_path.display()
                            );
                        }
                    }
                }
            }
        }

        info!(
            "Discovered {discovered} plugin(s) in {}",
            plugins_dir.display()
        );
        Ok(discovered)
    }

    /// Loads and validates a `plugin.json` manifest from disk.
    fn load_plugin_manifest(&self, manifest_path: &Path) -> Result<PluginManifest> {
        let content = std::fs::read_to_string(manifest_path).map_err(|e| {
            NoderaError::Plugin(PluginError::ManifestError {
                path: manifest_path.to_path_buf(),
                reason: format!("Cannot read manifest: {e}"),
            })
        })?;

        let manifest: PluginManifest = serde_json::from_str(&content).map_err(|e| {
            NoderaError::Plugin(PluginError::ManifestError {
                path: manifest_path.to_path_buf(),
                reason: format!("Malformed plugin manifest JSON: {e}"),
            })
        })?;

        // Validate plugin ID
        if manifest.id.trim().is_empty()
            || !manifest
                .id
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(NoderaError::Plugin(PluginError::InvalidId {
                id: manifest.id.clone(),
                reason: "Plugin ID must be non-empty alphanumeric with hyphens/underscores"
                    .to_string(),
            }));
        }

        Ok(manifest)
    }

    fn register_plugin_commands(&mut self, manifest: &PluginManifest) {
        if manifest.has_permission(&PluginPermission::Commands) {
            // Default command registration for the plugin
            self.registered_commands.push(PluginCommand {
                plugin_id: manifest.id.clone(),
                command_id: format!("{}:run", manifest.id),
                name: format!("{}: Run", manifest.name),
                description: format!("Execute {}", manifest.name),
            });
        }
    }

    /// Returns a list of all discovered plugins.
    pub fn list_plugins(&self) -> Vec<&PluginManifest> {
        self.plugins.values().collect()
    }

    /// Returns a specific plugin by its ID.
    pub fn get_plugin(&self, id: &str) -> Option<&PluginManifest> {
        self.plugins.get(id)
    }

    /// Returns all registered commands from enabled plugins.
    pub fn get_commands(&self) -> Vec<PluginCommand> {
        self.registered_commands
            .iter()
            .filter(|cmd| {
                self.plugins
                    .get(&cmd.plugin_id)
                    .map(|p| p.enabled)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// Enables or disables a plugin by ID.
    pub fn set_plugin_enabled(&mut self, id: &str, enabled: bool) -> Result<()> {
        if let Some(p) = self.plugins.get_mut(id) {
            p.enabled = enabled;
            debug!("Plugin '{}' enabled status set to {}", id, enabled);
            Ok(())
        } else {
            Err(NoderaError::Plugin(PluginError::NotFound {
                id: id.to_string(),
            }))
        }
    }

    /// Dispatches a lifecycle hook across all active plugins.
    pub fn dispatch_hook(&self, hook: &PluginHook) -> HookExecutionResult {
        let mut result = HookExecutionResult::default();

        for manifest in self.plugins.values() {
            if !manifest.enabled {
                continue;
            }

            match hook {
                PluginHook::VaultOpened { vault_name } => {
                    if manifest.has_permission(&PluginPermission::UiNotice) {
                        result.notices.push(format!(
                            "[{}] Loaded for vault '{}'",
                            manifest.name, vault_name
                        ));
                    }
                }
                PluginHook::NoteSaved {
                    relative_path,
                    content,
                } => {
                    if manifest.has_permission(&PluginPermission::ReadNotes) {
                        debug!(
                            "Plugin '{}' inspected note '{}' ({} bytes)",
                            manifest.id,
                            relative_path.display(),
                            content.len()
                        );
                    }
                }
                PluginHook::ExecuteCommand { command_id, args } => {
                    if manifest.has_permission(&PluginPermission::Commands) {
                        let expected = format!("{}:run", manifest.id);
                        if *command_id == expected || *command_id == manifest.id {
                            result.command_output = Some(format!(
                                "Plugin '{}' executed command '{}' with args: {:?}",
                                manifest.name, command_id, args
                            ));
                            if manifest.has_permission(&PluginPermission::UiNotice) {
                                result
                                    .notices
                                    .push(format!("Executed plugin '{}'", manifest.name));
                            }
                        }
                    }
                }
            }
        }

        result
    }
}

impl PluginManifest {
    /// Checks whether this plugin has declared a specific permission.
    pub fn has_permission(&self, perm: &PluginPermission) -> bool {
        self.permissions.contains(perm)
    }

    /// Returns a set of all unique permissions.
    pub fn permission_set(&self) -> HashSet<PluginPermission> {
        self.permissions.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_plugin_manifest_parsing_and_permissions() {
        let json = r#"{
            "id": "readability-scorer",
            "name": "Readability Scorer",
            "version": "1.2.0",
            "description": "Calculates Flesch-Kincaid index for notes",
            "author": "Alice",
            "main": "index.js",
            "permissions": ["read_notes", "commands", "ui_notice"],
            "enabled": true
        }"#;

        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.id, "readability-scorer");
        assert_eq!(manifest.name, "Readability Scorer");
        assert!(manifest.has_permission(&PluginPermission::ReadNotes));
        assert!(manifest.has_permission(&PluginPermission::Commands));
        assert!(manifest.has_permission(&PluginPermission::UiNotice));
        assert!(!manifest.has_permission(&PluginPermission::WriteNotes));
    }

    #[test]
    fn test_plugin_discovery_and_hook_dispatch() {
        let tmp = tempdir().unwrap();
        let vault_root = tmp.path();

        let plugin_dir = vault_root
            .join(".nodera")
            .join("plugins")
            .join("word-counter");
        std::fs::create_dir_all(&plugin_dir).unwrap();

        let manifest_content = r#"{
            "id": "word-counter",
            "name": "Word Counter",
            "version": "1.0.0",
            "main": "main.json",
            "permissions": ["read_notes", "commands", "ui_notice"]
        }"#;
        std::fs::write(plugin_dir.join("plugin.json"), manifest_content).unwrap();

        let mut manager = PluginManager::new();
        let count = manager.discover_from_vault(vault_root).unwrap();
        assert_eq!(count, 1);

        let plugins = manager.list_plugins();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "Word Counter");

        // Test commands registration
        let cmds = manager.get_commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].command_id, "word-counter:run");

        // Test hook dispatch
        let res = manager.dispatch_hook(&PluginHook::VaultOpened {
            vault_name: "TestVault",
        });
        assert_eq!(res.notices.len(), 1);
        assert!(res.notices[0].contains("Word Counter"));

        // Test command execution
        let cmd_res = manager.dispatch_hook(&PluginHook::ExecuteCommand {
            command_id: "word-counter:run",
            args: &["arg1".to_string()],
        });
        assert!(cmd_res.command_output.is_some());
        assert!(cmd_res.command_output.unwrap().contains("Word Counter"));

        // Test disabling
        manager.set_plugin_enabled("word-counter", false).unwrap();
        assert!(manager.get_commands().is_empty());
    }
}
