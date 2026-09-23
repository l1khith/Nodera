use std::fs;
use tempfile::tempdir;

use nodera_project::{InitOptions, InitStatus, ProjectGraph, ProjectIndex, ProjectInitializer};

#[test]
fn test_init_creates_project_artifacts() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("game_engine");
    fs::create_dir_all(root.join("src")).unwrap();

    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "game_engine"
version = "0.2.0"
edition = "2024"
"#,
    )
    .unwrap();

    fs::write(
        root.join("src").join("main.rs"),
        r#"
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

pub fn start_game() {
    let _v = Vector2::new(1.0, 2.0);
}
"#,
    )
    .unwrap();

    let init_res = ProjectInitializer::init(&root, &InitOptions::default()).unwrap();
    assert_eq!(init_res.status, InitStatus::Initialized);
    assert_eq!(init_res.project.name, "game_engine");

    // Verify all 4 required artifacts exist
    let nodera_dir = root.join(".nodera");
    assert!(nodera_dir.join("project.toml").exists());
    assert!(nodera_dir.join("graph").join("project.json").exists());
    assert!(nodera_dir.join("index").join("project.db").exists());
    assert!(nodera_dir.join("state").join("source_state.json").exists());

    // Verify ProjectGraph content
    let graph =
        ProjectGraph::load_from_file(&nodera_dir.join("graph").join("project.json")).unwrap();
    assert_eq!(graph.project.name, "game_engine");
    assert!(!graph.files.is_empty());
    assert!(!graph.nodes.is_empty());
    assert!(!graph.edges.is_empty());

    // Verify stable ID format
    let struct_node = graph.nodes.iter().find(|n| n.name == "Vector2").unwrap();
    assert!(struct_node
        .id
        .starts_with("sym:game_engine::src/main.rs::struct::"));
    assert_eq!(struct_node.visibility, "pub");

    let fn_node = graph.nodes.iter().find(|n| n.name == "start_game").unwrap();
    assert!(fn_node
        .id
        .starts_with("sym:game_engine::src/main.rs::function::"));

    // Verify adapter to GraphData
    let graph_data = graph.to_graph_data();
    assert_eq!(graph_data.nodes.len(), graph.nodes.len());
    assert_eq!(graph_data.edges.len(), graph.edges.len());
    assert!(graph_data.nodes.iter().any(|n| n.label.contains("Vector2")));

    // Verify SQLite ProjectIndex
    let index = ProjectIndex::open_or_create(&nodera_dir.join("index")).unwrap();
    assert!(index.symbol_count().unwrap() >= 4); // Vector2, x, y, new, start_game
    assert_eq!(index.file_count().unwrap(), 1);

    let search_results = index.search_symbols("Vector", 10).unwrap();
    assert!(!search_results.is_empty());
    assert_eq!(search_results[0].name, "Vector2");
}

#[test]
fn test_project_id_stability() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("stable_proj");
    fs::create_dir_all(root.join("src")).unwrap();

    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "stable_proj"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(root.join("src").join("lib.rs"), "pub fn test_fn() {}").unwrap();

    let id1 = nodera_project::generate_stable_project_id(&root, "stable_proj");
    let id2 = nodera_project::generate_stable_project_id(&root, "stable_proj");
    assert_eq!(
        id1, id2,
        "Generated IDs for the same path must be identical"
    );

    let init1 = ProjectInitializer::init(&root, &InitOptions::default()).unwrap();
    let init2 = ProjectInitializer::init(&root, &InitOptions::default()).unwrap();
    assert_eq!(init1.project_id, init2.project_id);
    assert_eq!(init1.project_id, id1);
}
