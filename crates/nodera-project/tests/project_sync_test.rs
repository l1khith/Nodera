use std::fs;
use tempfile::tempdir;

use nodera_project::{
    InitOptions, ProjectError, ProjectGraph, ProjectIndex, ProjectInitializer, ProjectSynchronizer,
};

#[test]
fn test_update_uninitialized_project_fails() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("uninitialized");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "uninit"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    let err = ProjectSynchronizer::update(&root).unwrap_err();
    match err {
        ProjectError::NotInitialized { path } => {
            assert_eq!(path, root);
        }
        other => panic!("Expected NotInitialized error, got: {other:?}"),
    }
}

#[test]
fn test_update_incremental_lifecycle() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("fungame");
    fs::create_dir_all(root.join("src")).unwrap();

    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "fungame"
version = "0.1.0"
edition = "2024"
"#,
    )
    .unwrap();

    fs::write(
        root.join("src").join("main.rs"),
        r#"
fn main() {
    println!("Hello, Fungame!");
}
"#,
    )
    .unwrap();

    // 1. Initial Init
    let init_res = ProjectInitializer::init(&root, &InitOptions::default()).unwrap();
    assert_eq!(init_res.symbols_parsed, 1); // fn main

    // Verify initial update when nothing changed reports empty changes
    let sync_no_changes = ProjectSynchronizer::update(&root).unwrap();
    assert!(sync_no_changes.changes.is_empty());
    assert!(!sync_no_changes.was_rebuilt);

    // 2. Modify src/main.rs to add Enemy
    fs::write(
        root.join("src").join("main.rs"),
        r#"
pub struct Enemy {
    pub health: u32,
}

fn main() {
    let _e = Enemy { health: 100 };
}
"#,
    )
    .unwrap();

    let sync_modified = ProjectSynchronizer::update(&root).unwrap();
    assert_eq!(sync_modified.changes.modified.len(), 1);
    assert_eq!(sync_modified.changes.added.len(), 0);
    assert_eq!(sync_modified.changes.deleted.len(), 0);

    let graph_path = root.join(".nodera").join("graph").join("project.json");
    let graph = ProjectGraph::load_from_file(&graph_path).unwrap();
    assert!(graph.nodes.iter().any(|n| n.name == "Enemy"));

    let index_dir = root.join(".nodera").join("index");
    let index = ProjectIndex::open_or_create(&index_dir).unwrap();
    let enemy_results = index.search_symbols("Enemy", 5).unwrap();
    assert_eq!(enemy_results.len(), 1);
    assert_eq!(enemy_results[0].name, "Enemy");

    // 3. Add a new file: src/player.rs
    fs::write(
        root.join("src").join("player.rs"),
        r#"
pub struct Player {
    pub name: String,
}
"#,
    )
    .unwrap();

    let sync_added = ProjectSynchronizer::update(&root).unwrap();
    assert_eq!(sync_added.changes.added.len(), 1);
    assert!(sync_added.changes.added[0].ends_with("player.rs"));

    let graph = ProjectGraph::load_from_file(&graph_path).unwrap();
    assert!(graph.nodes.iter().any(|n| n.name == "Player"));

    let player_results = index.search_symbols("Player", 5).unwrap();
    assert_eq!(player_results.len(), 1);

    // 4. Delete src/player.rs
    fs::remove_file(root.join("src").join("player.rs")).unwrap();

    let sync_deleted = ProjectSynchronizer::update(&root).unwrap();
    assert_eq!(sync_deleted.changes.deleted.len(), 1);
    assert_eq!(sync_deleted.changes.deleted[0], "src/player.rs");

    let graph = ProjectGraph::load_from_file(&graph_path).unwrap();
    assert!(!graph.nodes.iter().any(|n| n.name == "Player"));

    let player_results_after = index.search_symbols("Player", 5).unwrap();
    assert!(player_results_after.is_empty());
}

#[test]
fn test_update_recovery_when_graph_deleted() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("recover_proj");
    fs::create_dir_all(root.join("src")).unwrap();

    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "recover_proj"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(
        root.join("src").join("lib.rs"),
        "pub fn core_computation() -> i32 { 42 }",
    )
    .unwrap();

    ProjectInitializer::init(&root, &InitOptions::default()).unwrap();

    let graph_path = root.join(".nodera").join("graph").join("project.json");
    assert!(graph_path.exists());

    // Delete derived graph artifact
    fs::remove_file(&graph_path).unwrap();
    assert!(!graph_path.exists());

    // Run update -> must detect missing derived state and rebuild safely
    let sync_res = ProjectSynchronizer::update(&root).unwrap();
    assert!(sync_res.was_rebuilt);
    assert!(graph_path.exists());

    let graph = ProjectGraph::load_from_file(&graph_path).unwrap();
    assert!(graph.nodes.iter().any(|n| n.name == "core_computation"));
}
