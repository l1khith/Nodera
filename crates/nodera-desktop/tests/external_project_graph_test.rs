use std::fs;
use tempfile::tempdir;

use nodera_core::Vault;
use nodera_desktop::state::{ActiveView, AppState};
use nodera_project::{InitOptions, ProjectInitializer};

#[test]
fn test_external_project_selection_and_graph_domain_separation() {
    let tmp = tempdir().unwrap();

    // 1. Setup a Markdown Vault with 2 notes
    let vault_dir = tmp.path().join("MyVault");
    let _vault = Vault::create(&vault_dir, Some("My Vault".to_string())).unwrap();

    let mut state = AppState::default();
    state.open_vault(vault_dir.clone()).unwrap();

    state.create_note("NoteA", None).unwrap();
    state.update_editor_content("# Note A\nLinks to [[NoteB]].".to_string());
    state.save_active_note().unwrap();

    state.create_note("NoteB", None).unwrap();
    state.update_editor_content("# Note B\nTarget note.".to_string());
    state.save_active_note().unwrap();

    // Verify initial Vault graph
    let vault_graph = state.get_full_graph_data();
    assert_eq!(vault_graph.nodes.len(), 2);
    assert_eq!(vault_graph.edges.len(), 1);
    assert!(vault_graph.nodes.iter().any(|n| n.label == "NoteA"));
    assert!(vault_graph.nodes.iter().any(|n| n.label == "NoteB"));

    // 2. Setup an external Rust project (completely outside the vault)
    let project_dir = tmp.path().join("fungame");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(
        project_dir.join("Cargo.toml"),
        r#"[package]
name = "fungame"
version = "0.1.0"
edition = "2024"
"#,
    )
    .unwrap();
    fs::write(
        project_dir.join("src").join("main.rs"),
        r#"
pub struct Enemy {
    pub health: u32,
}

pub fn spawn_enemy() -> Enemy {
    Enemy { health: 100 }
}
"#,
    )
    .unwrap();

    // Initialize external project
    let init_res = ProjectInitializer::init(&project_dir, &InitOptions::default()).unwrap();
    assert_eq!(init_res.project.name, "fungame");

    // 3. Verify that initializing the project did NOT modify the live vault
    let vault_graph_after_init = state.get_full_graph_data();
    assert_eq!(vault_graph_after_init.nodes.len(), 2);
    assert_eq!(vault_graph_after_init.edges.len(), 1);

    // 4. Select the external project in desktop state
    state.refresh_registered_projects();
    state.select_project(&init_res.project_id).unwrap();

    assert!(state.active_project.is_some());
    assert_eq!(state.active_view, ActiveView::Graph);

    // 5. Verify that active graph is now the Project Graph
    let project_graph_data = state.get_full_graph_data();
    assert!(project_graph_data
        .nodes
        .iter()
        .any(|n| n.label.contains("Enemy")));
    assert!(project_graph_data
        .nodes
        .iter()
        .any(|n| n.label.contains("spawn_enemy")));

    // Verify NO Markdown notes leaked into the project graph
    assert!(!project_graph_data.nodes.iter().any(|n| n.label == "NoteA"));
    assert!(!project_graph_data.nodes.iter().any(|n| n.label == "NoteB"));

    // 6. Close the project and return to Vault Mode
    state.close_project();
    assert!(state.active_project.is_none());

    // 7. Verify that vault graph is restored and 100% intact
    let restored_vault_graph = state.get_full_graph_data();
    assert_eq!(restored_vault_graph.nodes.len(), 2);
    assert_eq!(restored_vault_graph.edges.len(), 1);
    assert!(restored_vault_graph
        .nodes
        .iter()
        .any(|n| n.label == "NoteA"));
    assert!(restored_vault_graph
        .nodes
        .iter()
        .any(|n| n.label == "NoteB"));
    assert!(!restored_vault_graph
        .nodes
        .iter()
        .any(|n| n.label.contains("Enemy")));
}
