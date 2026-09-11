use nodera_core::VaultEntry;
use nodera_desktop::AppState;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_knowledge_graph_and_backlinks_workflow() {
    let tmp = tempdir().expect("tempdir");
    let vault_path = tmp.path().join("KnowledgeVault");

    let mut state = AppState::default();
    state
        .create_vault(&vault_path, Some("Knowledge Vault".to_string()))
        .expect("create vault");

    // 1. Create Note A (Concept)
    state.create_note("Rust", None).expect("create Note A");
    state.update_editor_content(
        "---\ntitle: Rust Language\ntags: [systems, programming]\n---\n# Rust\n\nA modern systems programming language."
            .to_string(),
    );
    state.save_active_note().expect("save Note A");

    // 2. Create Note B linking to Note A
    state
        .create_note("Concurrency", None)
        .expect("create Note B");
    state.update_editor_content(
        "# Concurrency\n\n[[Rust]] provides fearless concurrency.\nSee also [[Async]] for details.\n\n- [ ] Study threads\n- [ ] Study channels"
            .to_string(),
    );
    state.save_active_note().expect("save Note B");

    // 3. Create Note C also linking to Note A
    state
        .create_note("Memory Safety", None)
        .expect("create Note C");
    state.update_editor_content(
        "# Memory Safety\n\nGuaranteed at compile-time by [[Rust]] borrow checker.".to_string(),
    );
    state.save_active_note().expect("save Note C");

    // 4. Open Note A and check backlinks (should contain Concurrency and Memory Safety)
    let rust_path = state
        .entries
        .iter()
        .find(|e: &&VaultEntry| e.name() == "Rust")
        .expect("find Rust")
        .relative_path()
        .to_path_buf();

    state.select_note(&rust_path).expect("select Rust note");
    let backlinks = state.get_current_backlinks();
    assert_eq!(backlinks.len(), 2, "Rust should have 2 incoming backlinks");

    let backlink_names: Vec<String> = backlinks
        .iter()
        .map(|p: &PathBuf| p.file_stem().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(backlink_names.contains(&"Concurrency".to_string()));
    assert!(backlink_names.contains(&"Memory Safety".to_string()));

    // 5. Check outgoing links of Concurrency note
    let concurrency_path = state
        .entries
        .iter()
        .find(|e: &&VaultEntry| e.name() == "Concurrency")
        .expect("find Concurrency")
        .relative_path()
        .to_path_buf();

    state
        .select_note(&concurrency_path)
        .expect("select Concurrency");
    let outgoing = state.get_current_outgoing_links();
    assert_eq!(outgoing.len(), 2);

    // [[Rust]] should be resolved
    let rust_link = outgoing.iter().find(|(l, _)| l.target == "Rust").unwrap();
    assert!(
        rust_link.1.is_some(),
        "[[Rust]] must be resolved to existing note"
    );

    // [[Async]] should be unresolved initially
    let async_link = outgoing.iter().find(|(l, _)| l.target == "Async").unwrap();
    assert!(async_link.1.is_none(), "[[Async]] is not yet created");

    // 6. Click [[Async]] to open/create it
    state
        .open_or_create_target("Async")
        .expect("open or create Async");
    assert_eq!(state.active_note.as_ref().unwrap().title, "Async");

    // 7. Verify task toggling in Concurrency
    state
        .select_note(&concurrency_path)
        .expect("select Concurrency again");
    // Line 6 is "- [ ] Study threads"
    state.toggle_task_at_line(6).expect("toggle task");
    assert!(state.editor_content.contains("- [x] Study threads"));
}
