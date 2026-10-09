use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::tempdir;

use nodera_core::Vault;
use nodera_desktop::state::GraphSettings;
use nodera_desktop::watcher::{coalesce_events, WatcherEvent, WriteSuppressor};
use nodera_desktop::AppState;

/// Scenario 1: Renaming a note with incoming and outgoing links
/// (verifying SQLite + in-memory LinkGraph consistency).
#[test]
fn test_scenario_1_rename_note_with_incoming_and_outgoing_links() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().to_path_buf();

    let _vault = Vault::create(&vault_path, Some("Test Vault".to_string())).unwrap();
    let mut state = AppState::default();
    state.open_vault(&vault_path).unwrap();

    // 1. Create three notes: Alpha, Beta, Gamma
    state
        .create_note_with_content(
            "Gamma",
            Some(""),
            "Destination note content for Gamma.",
        )
        .unwrap();
    state
        .create_note_with_content(
            "Beta",
            Some(""),
            "Beta content linking to [[Gamma]].",
        )
        .unwrap();
    state
        .create_note_with_content(
            "Alpha",
            Some(""),
            "Alpha content linking to [[Beta]] and [[Gamma]].",
        )
        .unwrap();

    let alpha_path = PathBuf::from("Alpha.md");
    let beta_path = PathBuf::from("Beta.md");
    let gamma_path = PathBuf::from("Gamma.md");

    // Verify initial in-memory graph
    let beta_outgoing = state.link_graph.get_outgoing_links(&beta_path);
    assert_eq!(beta_outgoing.len(), 1);
    assert_eq!(beta_outgoing[0].target, "Gamma");

    let note_paths = state.note_paths();
    let gamma_backlinks = state.link_graph.get_backlinks(&gamma_path, &note_paths);
    assert!(gamma_backlinks.contains(&alpha_path));
    assert!(gamma_backlinks.contains(&beta_path));

    let beta_backlinks = state.link_graph.get_backlinks(&beta_path, &note_paths);
    assert!(beta_backlinks.contains(&alpha_path));

    // Verify SQLite index has Beta
    {
        let idx_arc = state.vault_index.as_ref().unwrap();
        let idx = idx_arc.lock().unwrap();
        let results = idx.search("Beta", 10).unwrap();
        assert!(results.iter().any(|r| r.path == "Beta.md"));
    }

    // 2. Rename Beta to Delta
    state.rename_note(&beta_path, "Delta").unwrap();
    let delta_path = PathBuf::from("Delta.md");

    // Verify filesystem state
    assert!(!vault_path.join("Beta.md").exists());
    assert!(vault_path.join("Delta.md").exists());

    // Verify in-memory LinkGraph:
    // - Beta.md must be purged from outgoing and incoming indices
    assert!(state.link_graph.get_outgoing_links(&beta_path).is_empty());
    let note_paths_after = state.note_paths();
    assert!(!note_paths_after.contains(&beta_path));
    assert!(note_paths_after.contains(&delta_path));

    // - Delta.md must inherit outgoing links to Gamma
    let delta_outgoing = state.link_graph.get_outgoing_links(&delta_path);
    assert_eq!(delta_outgoing.len(), 1);
    assert_eq!(delta_outgoing[0].target, "Gamma");

    // - Gamma backlinks must now list Delta.md, not Beta.md
    let gamma_backlinks_after = state.link_graph.get_backlinks(&gamma_path, &note_paths_after);
    assert!(gamma_backlinks_after.contains(&alpha_path));
    assert!(gamma_backlinks_after.contains(&delta_path));
    assert!(!gamma_backlinks_after.contains(&beta_path));

    // Verify SQLite index reflects rename
    {
        let idx_arc = state.vault_index.as_ref().unwrap();
        let idx = idx_arc.lock().unwrap();
        let results = idx.search("Beta", 10).unwrap();
        assert!(!results.iter().any(|r| r.path == "Beta.md"));
        let delta_results = idx.search("Delta", 10).unwrap();
        assert!(delta_results.iter().any(|r| r.path == "Delta.md"));

        let gamma_sqlite_backlinks = idx.query_backlinks("Gamma").unwrap();
        assert!(gamma_sqlite_backlinks.iter().any(|p| p.contains("Delta")));
        assert!(!gamma_sqlite_backlinks.iter().any(|p| p.contains("Beta")));
    }
}

/// Scenario 2: Deleting a linked note (verifying links to it become unresolved without corrupting other links).
#[test]
fn test_scenario_2_delete_linked_note_unresolved_links() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().to_path_buf();

    let _vault = Vault::create(&vault_path, Some("Test Vault".to_string())).unwrap();
    let mut state = AppState::default();
    state.open_vault(&vault_path).unwrap();

    // 1. Create Target and Source linking to Target
    state
        .create_note_with_content("Target", Some(""), "# Target Note Content")
        .unwrap();
    state
        .create_note_with_content(
            "Source",
            Some(""),
            "# Source Note\n\nLinks to [[Target]].",
        )
        .unwrap();

    let source_path = PathBuf::from("Source.md");
    let target_path = PathBuf::from("Target.md");

    let note_paths = state.note_paths();
    let target_backlinks = state.link_graph.get_backlinks(&target_path, &note_paths);
    assert!(target_backlinks.contains(&source_path));

    // 2. Delete Target note permanently
    state.delete_note_permanently(&target_path).unwrap();

    // Verify filesystem state
    assert!(!vault_path.join("Target.md").exists());

    // Verify in-memory LinkGraph:
    // - Target.md is removed from note paths
    let note_paths_after = state.note_paths();
    assert!(!note_paths_after.contains(&target_path));
    assert!(note_paths_after.contains(&source_path));

    // - Source note's outgoing link is still intact in its content/outgoing list
    let source_outgoing = state.link_graph.get_outgoing_links(&source_path);
    assert_eq!(source_outgoing.len(), 1);
    assert_eq!(source_outgoing[0].target, "Target");

    // - Target has no backlinks
    let target_backlinks_after = state.link_graph.get_backlinks(&target_path, &note_paths_after);
    assert!(target_backlinks_after.is_empty());

    // - Graph visualization with default filter reflects Target as unresolved node
    let graph_data = state.get_full_graph_data_with_settings(&GraphSettings::default());

    let has_unresolved_target = graph_data
        .nodes
        .iter()
        .any(|n| n.is_unresolved && n.id.contains("Target"));
    assert!(has_unresolved_target, "Deleted note target should become an unresolved node");

    // SQLite index no longer contains Target
    {
        let idx_arc = state.vault_index.as_ref().unwrap();
        let idx = idx_arc.lock().unwrap();
        let results = idx.search("Target Note", 10).unwrap();
        assert!(!results.iter().any(|r| r.path == "Target.md"));
    }
}

/// Scenario 3: Creating and editing a Markdown file externally via apply_watcher_events.
#[test]
fn test_scenario_3_create_and_edit_externally_via_apply_watcher_events() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().to_path_buf();

    let _vault = Vault::create(&vault_path, Some("Test Vault".to_string())).unwrap();
    let mut state = AppState::default();
    state.open_vault(&vault_path).unwrap();

    // Create Initial note
    state
        .create_note_with_content("Initial", Some(""), "# Initial Note")
        .unwrap();

    // 1. Externally write External.md linking to Initial
    let external_abs = vault_path.join("External.md");
    fs::write(
        &external_abs,
        "# External Note\n\nExternal reference to [[Initial]].",
    )
    .unwrap();

    // Notify state via WatcherEvent::Created
    state
        .apply_watcher_events(&[WatcherEvent::Created(external_abs.clone())])
        .unwrap();

    let external_rel = PathBuf::from("External.md");
    let initial_rel = PathBuf::from("Initial.md");

    // Assert state knows about External.md
    assert!(state.note_paths().contains(&external_rel));
    let ext_outgoing = state.link_graph.get_outgoing_links(&external_rel);
    assert_eq!(ext_outgoing.len(), 1);
    assert_eq!(ext_outgoing[0].target, "Initial");

    let note_paths = state.note_paths();
    let initial_backlinks = state.link_graph.get_backlinks(&initial_rel, &note_paths);
    assert!(initial_backlinks.contains(&external_rel));

    // 2. Externally modify External.md to remove the link
    fs::write(
        &external_abs,
        "# External Note\n\nUpdated without any links now.",
    )
    .unwrap();

    state
        .apply_watcher_events(&[WatcherEvent::Modified(external_abs)])
        .unwrap();

    // Outgoing links updated
    let ext_outgoing_after = state.link_graph.get_outgoing_links(&external_rel);
    assert!(ext_outgoing_after.is_empty());

    // Backlinks updated
    let note_paths_after = state.note_paths();
    let initial_backlinks_after = state.link_graph.get_backlinks(&initial_rel, &note_paths_after);
    assert!(!initial_backlinks_after.contains(&external_rel));
}

/// Scenario 4: Renaming a file externally (WatcherEvent::Renamed via apply_watcher_events).
#[test]
fn test_scenario_4_external_rename_via_apply_watcher_events() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().to_path_buf();

    let _vault = Vault::create(&vault_path, Some("Test Vault".to_string())).unwrap();
    let mut state = AppState::default();
    state.open_vault(&vault_path).unwrap();

    state
        .create_note_with_content("Target", Some(""), "# Target Note")
        .unwrap();
    state
        .create_note_with_content(
            "Source",
            Some(""),
            "# Source Note\n\nPoints to [[Target]].",
        )
        .unwrap();

    let old_abs = vault_path.join("Source.md");
    let new_abs = vault_path.join("RenamedSource.md");

    // Physically rename on disk
    fs::rename(&old_abs, &new_abs).unwrap();

    // Apply WatcherEvent::Renamed
    state
        .apply_watcher_events(&[WatcherEvent::Renamed {
            old_path: old_abs,
            new_path: new_abs,
        }])
        .unwrap();

    let old_rel = PathBuf::from("Source.md");
    let new_rel = PathBuf::from("RenamedSource.md");
    let target_rel = PathBuf::from("Target.md");

    // Verify old path removed and new path present
    let paths = state.note_paths();
    assert!(!paths.contains(&old_rel));
    assert!(paths.contains(&new_rel));

    // Verify link graph outgoing and incoming consistency
    let new_outgoing = state.link_graph.get_outgoing_links(&new_rel);
    assert_eq!(new_outgoing.len(), 1);
    assert_eq!(new_outgoing[0].target, "Target");

    let target_backlinks = state.link_graph.get_backlinks(&target_rel, &paths);
    assert!(target_backlinks.contains(&new_rel));
    assert!(!target_backlinks.contains(&old_rel));
}

/// Scenario 5: Duplicate watcher events debouncing and coalescing (coalesce_events).
#[test]
fn test_scenario_5_duplicate_watcher_events_coalescing() {
    let p1 = PathBuf::from("/vault/Note1.md");
    let p2 = PathBuf::from("/vault/Note2.md");
    let p3 = PathBuf::from("/vault/Transient.md");

    // Multiple rapid modifies of p1 should collapse to a single Modify
    let events = vec![
        WatcherEvent::Modified(p1.clone()),
        WatcherEvent::Modified(p1.clone()),
        WatcherEvent::Modified(p1.clone()),
    ];
    let coalesced = coalesce_events(events);
    assert_eq!(coalesced.len(), 1);
    assert_eq!(coalesced[0], WatcherEvent::Modified(p1.clone()));

    // Created followed by Modified should stay Created
    let events2 = vec![
        WatcherEvent::Created(p2.clone()),
        WatcherEvent::Modified(p2.clone()),
        WatcherEvent::Modified(p2.clone()),
    ];
    let coalesced2 = coalesce_events(events2);
    assert_eq!(coalesced2.len(), 1);
    assert_eq!(coalesced2[0], WatcherEvent::Created(p2.clone()));

    // Created then Deleted should vanish (transient temporary file)
    let events3 = vec![
        WatcherEvent::Created(p3.clone()),
        WatcherEvent::Modified(p3.clone()),
        WatcherEvent::Deleted(p3.clone()),
    ];
    let coalesced3 = coalesce_events(events3);
    assert!(coalesced3.is_empty(), "Transient file creation and deletion must be cancelled out");

    // Modified followed by Deleted should collapse to Deleted
    let events4 = vec![
        WatcherEvent::Modified(p1.clone()),
        WatcherEvent::Deleted(p1.clone()),
    ];
    let coalesced4 = coalesce_events(events4);
    assert_eq!(coalesced4.len(), 1);
    assert_eq!(coalesced4[0], WatcherEvent::Deleted(p1.clone()));
}

/// Scenario 6: Nodera's own writes suppressed via WriteSuppressor (no reindex loops).
#[test]
fn test_scenario_6_write_suppressor_prevents_reindex_loops() {
    let suppressor = WriteSuppressor::new();
    let p = PathBuf::from("/vault/ActiveNote.md");

    // Initially not suppressed
    assert!(!suppressor.is_suppressed(&p));

    // Suppress for 200ms
    suppressor.suppress(&p, Duration::from_millis(200));
    assert!(suppressor.is_suppressed(&p));

    // Fast check within suppression duration
    std::thread::sleep(Duration::from_millis(50));
    assert!(suppressor.is_suppressed(&p));

    // Wait until expiration
    std::thread::sleep(Duration::from_millis(200));
    assert!(!suppressor.is_suppressed(&p));

    // Also verify AppState integration
    let dir = tempdir().unwrap();
    let vault_path = dir.path().to_path_buf();
    let _vault = Vault::create(&vault_path, Some("Test Vault".to_string())).unwrap();
    let mut state = AppState::default();
    state.open_vault(&vault_path).unwrap();

    state.create_note_with_content("SelfWrite", Some(""), "# Testing Self-Write").unwrap();
    let full_note_path = vault_path.join("SelfWrite.md");
    // State's internal suppressor should have suppressed this path
    assert!(state.write_suppressor.is_suppressed(&full_note_path));
}

/// Scenario 7: Index/database recovery after corruption (rebuild_vault_index).
#[test]
fn test_scenario_7_index_and_database_recovery_after_corruption() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().to_path_buf();

    let _vault = Vault::create(&vault_path, Some("Recovery Vault".to_string())).unwrap();
    let mut state = AppState::default();
    state.open_vault(&vault_path).unwrap();

    state
        .create_note_with_content(
            "Alpha",
            Some(""),
            "# Alpha Note\n\nCritical data linking to [[Beta]].",
        )
        .unwrap();
    state
        .create_note_with_content(
            "Beta",
            Some(""),
            "# Beta Note\n\nContent for Beta.",
        )
        .unwrap();

    // Drop current index handle lock before corrupting
    drop(state.vault_index.take());

    // Deliberately corrupt SQLite database file with garbage
    let sqlite_path = vault_path.join(".nodera").join("index.sqlite");
    assert!(sqlite_path.exists());
    fs::write(&sqlite_path, b"CORRUPTED_GARBAGE_HEADER_DATA_HERE").unwrap();

    // Re-open index via VaultIndex::open (which self-heals corrupted SQLite databases)
    let healed_idx = nodera_index::VaultIndex::open(&vault_path).unwrap();
    state.vault_index = Some(std::sync::Arc::new(std::sync::Mutex::new(healed_idx)));

    // Rebuild vault index and in-memory link graph from source markdown files
    let count = state.rebuild_vault_index().unwrap();
    assert_eq!(count, 2);

    // Verify search is operational
    {
        let idx_arc = state.vault_index.as_ref().unwrap();
        let idx = idx_arc.lock().unwrap();
        let res = idx.search("Critical", 5).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].title, "Alpha Note");
    }

    // Verify in-memory link graph is fully rebuilt
    let note_paths = state.note_paths();
    let beta_rel = PathBuf::from("Beta.md");
    let alpha_rel = PathBuf::from("Alpha.md");
    let backlinks = state.link_graph.get_backlinks(&beta_rel, &note_paths);
    assert!(backlinks.contains(&alpha_rel));
}

/// Scenario 8: Restarting Nodera and rebuilding consistent graph state.
#[test]
fn test_scenario_8_restarting_nodera_rebuilds_consistent_graph_state() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().to_path_buf();

    // 1. First session: Create notes with links
    {
        let _vault = Vault::create(&vault_path, Some("Persistent Vault".to_string())).unwrap();
        let mut session1 = AppState::default();
        session1.open_vault(&vault_path).unwrap();

        session1
            .create_note_with_content(
                "NodeA",
                Some(""),
                "# Node A\n\nPoints to [[NodeB]].",
            )
            .unwrap();
        session1
            .create_note_with_content(
                "NodeB",
                Some(""),
                "# Node B\n\nPoints to [[NodeC]] and [[NodeD]].",
            )
            .unwrap();
        session1
            .create_note_with_content(
                "NodeC",
                Some(""),
                "# Node C\n\nPoints to [[NodeA]].",
            )
            .unwrap();
        session1
            .create_note_with_content(
                "NodeD",
                Some(""),
                "# Node D\n\nIsolated leaf.",
            )
            .unwrap();
    }

    // 2. Second session (simulated app restart): Open brand-new AppState
    let mut session2 = AppState::default();
    session2.open_vault(&vault_path).unwrap();

    let node_a = PathBuf::from("NodeA.md");
    let node_b = PathBuf::from("NodeB.md");
    let node_c = PathBuf::from("NodeC.md");
    let node_d = PathBuf::from("NodeD.md");
    let paths = session2.note_paths();

    // Outgoing links verification
    let a_outgoing = session2.link_graph.get_outgoing_links(&node_a);
    assert_eq!(a_outgoing.len(), 1);
    assert_eq!(a_outgoing[0].target, "NodeB");

    let b_outgoing = session2.link_graph.get_outgoing_links(&node_b);
    assert_eq!(b_outgoing.len(), 2);
    assert!(b_outgoing.iter().any(|l| l.target == "NodeC"));
    assert!(b_outgoing.iter().any(|l| l.target == "NodeD"));

    // Backlinks (incoming reverse index) verification
    let d_backlinks = session2.link_graph.get_backlinks(&node_d, &paths);
    assert_eq!(d_backlinks, vec![node_b.clone()]);

    let a_backlinks = session2.link_graph.get_backlinks(&node_a, &paths);
    assert_eq!(a_backlinks, vec![node_c.clone()]);

    let b_backlinks = session2.link_graph.get_backlinks(&node_b, &paths);
    assert_eq!(b_backlinks, vec![node_a.clone()]);

    let c_backlinks = session2.link_graph.get_backlinks(&node_c, &paths);
    assert_eq!(c_backlinks, vec![node_b.clone()]);

    // Graph data verification
    let graph_data = session2.get_full_graph_data_with_settings(&GraphSettings::default());

    assert_eq!(graph_data.nodes.len(), 4);
    assert_eq!(graph_data.edges.len(), 4);
}
