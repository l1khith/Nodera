use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{ProjectError, Result};
use crate::manifest::RawManifest;
use crate::model::{CargoPackage, ProjectKind, RustProject, SourceRoot, SourceRootKind};

/// Authoritative scanner for Rust packages, Cargo workspaces, and source files.
pub struct ProjectDiscovery;

impl ProjectDiscovery {
    /// Inspects the target directory, verifies it is a valid Rust project, and discovers
    /// all packages, source roots, and `.rs` files.
    pub fn discover(target_path: impl AsRef<Path>) -> Result<RustProject> {
        let path = target_path.as_ref();
        if !path.exists() {
            return Err(ProjectError::NotADirectory {
                path: path.to_path_buf(),
            });
        }
        if !path.is_dir() {
            return Err(ProjectError::NotADirectory {
                path: path.to_path_buf(),
            });
        }

        let canonical_root = fs::canonicalize(path).map_err(|e| ProjectError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        let root_manifest_path = canonical_root.join("Cargo.toml");
        if !root_manifest_path.exists() {
            return Err(ProjectError::NotARustProject {
                path: canonical_root,
            });
        }

        let manifest_content =
            fs::read_to_string(&root_manifest_path).map_err(|e| ProjectError::Io {
                path: root_manifest_path.clone(),
                source: e,
            })?;

        let manifest = RawManifest::parse(&manifest_content, &root_manifest_path)?;

        let (kind, project_name, packages) = if manifest.is_workspace() {
            let default_name = manifest
                .package_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    canonical_root
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("workspace")
                        .to_string()
                });

            let workspace_default_edition = manifest.resolved_edition();
            let workspace_default_version = manifest.resolved_version();

            let mut discovered_packages = Vec::new();

            // 1. Root package if the root manifest also contains [package]
            if manifest.package.is_some() {
                if let Ok(pkg) = Self::discover_package(
                    &canonical_root,
                    &canonical_root,
                    workspace_default_edition,
                    workspace_default_version,
                ) {
                    discovered_packages.push(pkg);
                }
            }

            // 2. Discover packages declared in workspace members
            for member_pattern in manifest.workspace_members() {
                let pattern = member_pattern.trim();
                if pattern.ends_with("/*") {
                    let base_dir = pattern.trim_end_matches("/*").trim_end_matches('/');
                    let base_path = canonical_root.join(base_dir);
                    if base_path.is_dir() {
                        if let Ok(entries) = fs::read_dir(&base_path) {
                            let mut sub_entries: Vec<PathBuf> = entries
                                .filter_map(|e| e.ok().map(|ent| ent.path()))
                                .filter(|p| p.is_dir())
                                .collect();
                            sub_entries.sort();

                            for sub_dir in sub_entries {
                                if sub_dir.join("Cargo.toml").exists() {
                                    if let Ok(pkg) = Self::discover_package(
                                        &sub_dir,
                                        &canonical_root,
                                        workspace_default_edition,
                                        workspace_default_version,
                                    ) {
                                        discovered_packages.push(pkg);
                                    }
                                }
                            }
                        }
                    }
                } else if pattern.ends_with("/**") {
                    let base_dir = pattern.trim_end_matches("/**").trim_end_matches('/');
                    let base_path = canonical_root.join(base_dir);
                    Self::discover_packages_recursively(
                        &base_path,
                        &canonical_root,
                        workspace_default_edition,
                        workspace_default_version,
                        &mut discovered_packages,
                    );
                } else {
                    let member_path = canonical_root.join(pattern);
                    if member_path.join("Cargo.toml").exists() {
                        if let Ok(pkg) = Self::discover_package(
                            &member_path,
                            &canonical_root,
                            workspace_default_edition,
                            workspace_default_version,
                        ) {
                            discovered_packages.push(pkg);
                        }
                    }
                }
            }

            (ProjectKind::Workspace, default_name, discovered_packages)
        } else {
            let pkg = Self::discover_package(&canonical_root, &canonical_root, None, None)?;
            let name = pkg.name.clone();
            (ProjectKind::SinglePackage, name, vec![pkg])
        };

        let edition = manifest
            .resolved_edition()
            .map(|s| s.to_string())
            .or_else(|| packages.first().and_then(|p| p.edition.clone()));

        let version = manifest
            .resolved_version()
            .map(|s| s.to_string())
            .or_else(|| packages.first().and_then(|p| p.version.clone()));

        Ok(RustProject {
            root: canonical_root,
            name: project_name,
            kind,
            edition,
            version,
            manifest_path: root_manifest_path,
            packages,
        })
    }

    /// Discovers a single package at `package_root`.
    fn discover_package(
        package_root: &Path,
        project_root: &Path,
        default_edition: Option<&str>,
        default_version: Option<&str>,
    ) -> Result<CargoPackage> {
        let manifest_path = package_root.join("Cargo.toml");
        if !manifest_path.exists() {
            return Err(ProjectError::ManifestNotFound {
                path: manifest_path,
            });
        }

        let content = fs::read_to_string(&manifest_path).map_err(|e| ProjectError::Io {
            path: manifest_path.clone(),
            source: e,
        })?;

        let manifest = RawManifest::parse(&content, &manifest_path)?;

        let name = manifest
            .package_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                package_root
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("package")
                    .to_string()
            });

        let version = manifest
            .resolved_version()
            .map(|s| s.to_string())
            .or_else(|| default_version.map(|s| s.to_string()));

        let edition = manifest
            .resolved_edition()
            .map(|s| s.to_string())
            .or_else(|| default_edition.map(|s| s.to_string()));

        // Discover source roots
        let mut source_roots = Vec::new();
        let candidate_roots = [
            ("src", SourceRootKind::Src),
            ("tests", SourceRootKind::Tests),
            ("examples", SourceRootKind::Examples),
            ("benches", SourceRootKind::Benches),
        ];

        for (sub_dir, kind) in candidate_roots {
            let path = package_root.join(sub_dir);
            if path.is_dir() {
                source_roots.push(SourceRoot { path, kind });
            }
        }

        // If no src directory exists, check for top-level lib.rs or main.rs
        if source_roots.is_empty()
            && (package_root.join("lib.rs").exists() || package_root.join("main.rs").exists())
        {
            source_roots.push(SourceRoot {
                path: package_root.to_path_buf(),
                kind: SourceRootKind::Src,
            });
        }

        // Discover Rust source files within source roots
        let mut source_files = Vec::new();
        for sr in &source_roots {
            Self::scan_rust_files(&sr.path, project_root, &mut source_files);
        }
        source_files.sort();

        Ok(CargoPackage {
            name,
            version,
            edition,
            manifest_path,
            package_root: package_root.to_path_buf(),
            source_roots,
            source_files,
        })
    }

    /// Recursively searches for packages containing Cargo.toml.
    fn discover_packages_recursively(
        dir: &Path,
        project_root: &Path,
        default_edition: Option<&str>,
        default_version: Option<&str>,
        packages: &mut Vec<CargoPackage>,
    ) {
        if !dir.is_dir() {
            return;
        }

        let dir_name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if is_excluded_dir(dir_name) {
            return;
        }

        if dir.join("Cargo.toml").exists() && dir != project_root {
            if let Ok(pkg) =
                Self::discover_package(dir, project_root, default_edition, default_version)
            {
                packages.push(pkg);
                return; // Stop descending deeper once a package root is found
            }
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::discover_packages_recursively(
                        &path,
                        project_root,
                        default_edition,
                        default_version,
                        packages,
                    );
                }
            }
        }
    }

    /// Recursively scans for `.rs` files under a source root, respecting exclusion rules.
    fn scan_rust_files(dir: &Path, project_root: &Path, files: &mut Vec<PathBuf>) {
        if !dir.is_dir() {
            return;
        }

        let dir_name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if is_excluded_dir(dir_name) {
            return;
        }

        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::scan_rust_files(&path, project_root, files);
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.eq_ignore_ascii_case("rs") {
                        // Ensure path stays within project boundaries
                        if path.starts_with(project_root) {
                            files.push(path);
                        }
                    }
                }
            }
        }
    }
}

/// Returns true if a directory name should be strictly excluded from scanning.
pub fn is_excluded_dir(name: &str) -> bool {
    matches!(
        name,
        "target" | ".git" | ".nodera" | "vendor" | "node_modules" | ".cargo"
    )
}
