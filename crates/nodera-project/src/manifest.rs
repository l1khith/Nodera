use std::path::Path;

use serde::Deserialize;

use crate::error::{ProjectError, Result};

/// Raw deserialized representation of a Cargo.toml manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct RawManifest {
    pub package: Option<RawPackage>,
    pub workspace: Option<RawWorkspace>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawPackage {
    pub name: Option<String>,
    pub version: Option<StringOrInherited>,
    pub edition: Option<StringOrInherited>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawWorkspace {
    pub members: Option<Vec<String>>,
    #[serde(rename = "default-members")]
    pub default_members: Option<Vec<String>>,
    pub package: Option<RawWorkspacePackage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawWorkspacePackage {
    pub version: Option<String>,
    pub edition: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum StringOrInherited {
    String(String),
    Inherited { workspace: bool },
}

impl StringOrInherited {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            Self::Inherited { .. } => None,
        }
    }
}

impl RawManifest {
    /// Parses a manifest string into a `RawManifest`.
    pub fn parse(content: &str, path: &Path) -> Result<Self> {
        toml::from_str(content).map_err(|e| ProjectError::ManifestParseError {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })
    }

    /// True if the manifest defines a `[workspace]`.
    pub fn is_workspace(&self) -> bool {
        self.workspace.is_some()
    }

    /// Package name, if declared in `[package]`.
    pub fn package_name(&self) -> Option<&str> {
        self.package.as_ref().and_then(|p| p.name.as_deref())
    }

    /// Package version from `[package]` or inherited from `[workspace.package]`.
    pub fn resolved_version(&self) -> Option<&str> {
        if let Some(pkg) = &self.package {
            if let Some(v) = &pkg.version {
                if let Some(s) = v.as_str() {
                    return Some(s);
                }
            }
        }
        self.workspace
            .as_ref()
            .and_then(|w| w.package.as_ref())
            .and_then(|wp| wp.version.as_deref())
    }

    /// Package edition from `[package]` or inherited from `[workspace.package]`.
    pub fn resolved_edition(&self) -> Option<&str> {
        if let Some(pkg) = &self.package {
            if let Some(e) = &pkg.edition {
                if let Some(s) = e.as_str() {
                    return Some(s);
                }
            }
        }
        self.workspace
            .as_ref()
            .and_then(|w| w.package.as_ref())
            .and_then(|wp| wp.edition.as_deref())
    }

    /// List of declared workspace members.
    pub fn workspace_members(&self) -> &[String] {
        self.workspace
            .as_ref()
            .and_then(|w| w.members.as_deref())
            .unwrap_or(&[])
    }
}
