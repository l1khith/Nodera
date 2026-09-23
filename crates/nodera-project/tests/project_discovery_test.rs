use std::fs;
use std::path::Path;

use nodera_project::{
    InitOptions, InitStatus, ProjectDiscovery, ProjectError, ProjectInitializer, ProjectKind,
    SourceRootKind,
};
use tempfile::tempdir;

#[test]
fn test_single_package_detection_and_metadata() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    let cargo_toml = r#"
[package]
name = "sample-tool"
version = "1.2.3"
edition = "2021"
"#;
    fs::write(root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src").join("main.rs"), "fn main() {}").unwrap();

    let project = ProjectDiscovery::discover(root).expect("discovery failed");

    assert_eq!(project.name, "sample-tool");
    assert_eq!(project.version.as_deref(), Some("1.2.3"));
    assert_eq!(project.edition.as_deref(), Some("2021"));
    assert_eq!(project.kind, ProjectKind::SinglePackage);
    assert!(!project.is_workspace());
    assert_eq!(project.packages.len(), 1);

    let pkg = &project.packages[0];
    assert_eq!(pkg.name, "sample-tool");
    assert_eq!(pkg.source_roots.len(), 1);
    assert_eq!(pkg.source_roots[0].kind, SourceRootKind::Src);
    assert_eq!(pkg.rust_file_count(), 1);
}

#[test]
fn test_workspace_detection_and_glob_discovery() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    let workspace_toml = r#"
[workspace]
members = [
    "crates/*",
    "apps/cli",
]

[workspace.package]
version = "0.5.0"
edition = "2024"
"#;
    fs::write(root.join("Cargo.toml"), workspace_toml).unwrap();

    // Member 1: crates/core
    let core_dir = root.join("crates").join("core");
    fs::create_dir_all(core_dir.join("src")).unwrap();
    fs::write(
        core_dir.join("Cargo.toml"),
        r#"[package]
name = "my-core"
version.workspace = true
edition.workspace = true
"#,
    )
    .unwrap();
    fs::write(core_dir.join("src").join("lib.rs"), "pub fn run() {}").unwrap();

    // Member 2: crates/parser
    let parser_dir = root.join("crates").join("parser");
    fs::create_dir_all(parser_dir.join("src")).unwrap();
    fs::write(
        parser_dir.join("Cargo.toml"),
        r#"[package]
name = "my-parser"
version.workspace = true
edition.workspace = true
"#,
    )
    .unwrap();
    fs::write(parser_dir.join("src").join("lib.rs"), "pub fn parse() {}").unwrap();

    // Member 3: apps/cli (explicit path)
    let cli_dir = root.join("apps").join("cli");
    fs::create_dir_all(cli_dir.join("src")).unwrap();
    fs::write(
        cli_dir.join("Cargo.toml"),
        r#"[package]
name = "my-cli"
version.workspace = true
edition.workspace = true
"#,
    )
    .unwrap();
    fs::write(cli_dir.join("src").join("main.rs"), "fn main() {}").unwrap();

    let project = ProjectDiscovery::discover(root).expect("discovery failed");

    assert_eq!(project.kind, ProjectKind::Workspace);
    assert!(project.is_workspace());
    assert_eq!(project.edition.as_deref(), Some("2024"));
    assert_eq!(project.version.as_deref(), Some("0.5.0"));
    assert_eq!(project.packages.len(), 3);

    let pkg_names: Vec<&str> = project.packages.iter().map(|p| p.name.as_str()).collect();
    assert!(pkg_names.contains(&"my-core"));
    assert!(pkg_names.contains(&"my-parser"));
    assert!(pkg_names.contains(&"my-cli"));

    assert_eq!(project.total_rust_files(), 3);
}

#[test]
fn test_source_root_and_file_discovery_with_exclusions() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "exclusions-test"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    // Valid source roots
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src").join("lib.rs"), "pub mod sub;").unwrap();
    fs::write(root.join("src").join("sub.rs"), "pub fn sub() {}").unwrap();

    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(root.join("tests").join("test_one.rs"), "fn it_works() {}").unwrap();

    fs::create_dir_all(root.join("benches")).unwrap();
    fs::write(root.join("benches").join("bench_one.rs"), "fn bench() {}").unwrap();

    // Directories that MUST BE EXCLUDED
    fs::create_dir_all(root.join("target").join("debug").join("build")).unwrap();
    fs::write(
        root.join("target").join("debug").join("build.rs"),
        "// should be excluded",
    )
    .unwrap();

    fs::create_dir_all(root.join(".git").join("hooks")).unwrap();
    fs::write(root.join(".git").join("dummy.rs"), "// should be excluded").unwrap();

    fs::create_dir_all(root.join(".nodera").join("cache")).unwrap();
    fs::write(
        root.join(".nodera").join("cached.rs"),
        "// should be excluded",
    )
    .unwrap();

    let project = ProjectDiscovery::discover(root).expect("discovery failed");
    let files = project.all_rust_files();

    assert_eq!(files.len(), 4);
    for file in files {
        let path_str = file.to_string_lossy();
        assert!(!path_str.contains("target"));
        assert!(!path_str.contains(".git"));
        assert!(!path_str.contains(".nodera"));
    }
}

#[test]
fn test_missing_cargo_toml_error() {
    let tmp = tempdir().unwrap();
    let empty_dir = tmp.path();

    let err = ProjectDiscovery::discover(empty_dir).unwrap_err();
    assert!(matches!(err, ProjectError::NotARustProject { .. }));
}

#[test]
fn test_malformed_cargo_toml_error() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    fs::write(root.join("Cargo.toml"), "invalid toml syntax [[[").unwrap();

    let err = ProjectDiscovery::discover(root).unwrap_err();
    assert!(matches!(err, ProjectError::ManifestParseError { .. }));
}

#[test]
fn test_missing_workspace_member_resilience() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    // Workspace pointing to nonexistent member path
    fs::write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = [
    "nonexistent_crate",
    "real_crate",
]
"#,
    )
    .unwrap();

    let real_dir = root.join("real_crate");
    fs::create_dir_all(real_dir.join("src")).unwrap();
    fs::write(
        real_dir.join("Cargo.toml"),
        r#"[package]
name = "real-crate"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();
    fs::write(real_dir.join("src").join("lib.rs"), "").unwrap();

    // Discovery should not panic or fail; it discovers the valid member
    let project = ProjectDiscovery::discover(root).expect("discovery should be resilient");
    assert_eq!(project.packages.len(), 1);
    assert_eq!(project.packages[0].name, "real-crate");
}

#[test]
fn test_nodera_init_and_idempotency() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "init-test"
version = "1.0.0"
edition = "2021"
"#,
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src").join("main.rs"), "fn main() {}").unwrap();

    let options = InitOptions::default();

    // 1. Initial initialization
    let res1 = ProjectInitializer::init(root, &options).expect("init failed");
    assert_eq!(res1.status, InitStatus::Initialized);
    assert!(res1.nodera_dir.exists());
    assert!(res1.config_path.exists());
    assert!(res1.nodera_dir.join("state").exists());
    assert!(res1.nodera_dir.join("cache").exists());
    assert!(res1.nodera_dir.join("index").exists());

    let original_config = fs::read_to_string(&res1.config_path).unwrap();
    assert!(original_config.contains("init-test"));

    // Add custom key to project.toml to test preservation
    let customized_config = format!("{original_config}\ncustom_user_setting = true\n");
    fs::write(&res1.config_path, &customized_config).unwrap();

    // 2. Second initialization (idempotent)
    let res2 = ProjectInitializer::init(root, &options).expect("idempotent init failed");
    assert_eq!(res2.status, InitStatus::AlreadyInitialized);

    // Configuration must remain preserved!
    let preserved_config = fs::read_to_string(&res2.config_path).unwrap();
    assert!(preserved_config.contains("custom_user_setting = true"));

    // Source files must remain unchanged
    let src_content = fs::read_to_string(root.join("src").join("main.rs")).unwrap();
    assert_eq!(src_content, "fn main() {}");
}

#[test]
fn test_nodera_self_discovery() {
    // Test discovering the actual nodera workspace repository
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent().unwrap().parent().unwrap();

    let project = ProjectDiscovery::discover(repo_root).expect("failed to discover nodera repo");
    assert!(project.is_workspace());
    assert!(project.packages.len() >= 6);

    let pkg_names: Vec<&str> = project.packages.iter().map(|p| p.name.as_str()).collect();
    assert!(pkg_names.contains(&"nodera-core"));
    assert!(pkg_names.contains(&"nodera-parser-core"));
    assert!(pkg_names.contains(&"nodera-project"));
    assert!(project.total_rust_files() > 50);
}
