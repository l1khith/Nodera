use tempfile::tempdir;

use nodera_desktop::state::{ActiveView, AppState, PaletteAction};

#[test]
fn test_search_and_global_tasks_workflow() {
    let tmp = tempdir().expect("tempdir");
    let vault_path = tmp.path().join("SearchVault");

    let mut state = AppState::default();
    state
        .create_vault(&vault_path, Some("Search & Task Vault".to_string()))
        .expect("create vault");

    // 1. Create Note 1: Project Plan
    state
        .create_note("Project Plan", None)
        .expect("create Note 1");
    let plan_path = state.active_note.as_ref().unwrap().relative_path.clone();
    state.update_editor_content(
        "# Project Plan\nPlanning the architecture for Nodera local-first desktop app.\nTags: #project #rust\n\n- [ ] Design database schema\n- [x] Implement foundation crate"
            .to_string(),
    );
    state.save_active_note().expect("save Note 1");

    // 2. Create Note 2: Tantivy Guide
    state
        .create_note("Search Guide", None)
        .expect("create Note 2");
    let search_note_path = state.active_note.as_ref().unwrap().relative_path.clone();
    state.update_editor_content(
        "# Search Guide\nTantivy provides high-performance full-text search indexing.\n\n- [ ] Benchmark query speed\n- [ ] Implement snippet highlighting"
            .to_string(),
    );
    state.save_active_note().expect("save Note 2");

    // 3. Test Full-Text Search via Tantivy
    state.execute_search("architecture");
    assert_eq!(state.search_results.len(), 1, "Should find 'Project Plan'");
    assert_eq!(state.search_results[0].title, "Project Plan");
    assert!(state.search_results[0]
        .snippet
        .to_lowercase()
        .contains("architecture"));

    state.execute_search("full-text");
    assert_eq!(state.search_results.len(), 1, "Should find 'Search Guide'");
    assert_eq!(state.search_results[0].title, "Search Guide");

    // 4. Test Global Tasks Query across all notes
    let all_tasks = state.get_vault_tasks();
    assert_eq!(all_tasks.len(), 4, "Total 4 tasks across 2 notes");

    // Filter incomplete tasks
    state.task_filter.checked = Some(false);
    let open_tasks = state.get_vault_tasks();
    assert_eq!(open_tasks.len(), 3, "3 incomplete tasks");

    // Filter completed tasks
    state.task_filter.checked = Some(true);
    let done_tasks = state.get_vault_tasks();
    assert_eq!(done_tasks.len(), 1, "1 completed task");
    assert_eq!(done_tasks[0].text, "Implement foundation crate");

    // Reset filter
    state.task_filter.checked = None;

    // 5. Test Interactive Task Toggling from Global Task View
    // Toggle "Design database schema" in Project Plan (line 5)
    state
        .toggle_task_and_sync(&plan_path, 5)
        .expect("toggle task on disk");

    // Re-verify that the task is now checked
    state.task_filter.checked = Some(true);
    let done_tasks_after = state.get_vault_tasks();
    assert_eq!(done_tasks_after.len(), 2, "Now 2 tasks are completed");

    // Verify source note on disk contains updated checkbox
    let read_note = state
        .vault_service
        .as_ref()
        .unwrap()
        .read_note(&plan_path)
        .expect("read note");
    assert!(read_note.content.contains("- [x] Design database schema"));

    // 6. Test Command Palette
    state.command_palette_query = "Search".to_string();
    let palette_items = state.get_command_palette_items();
    assert!(palette_items.iter().any(|i| i.title == "Search Guide"));

    // Execute palette action to open Search Guide
    state
        .execute_palette_action(PaletteAction::OpenNote(search_note_path.clone()))
        .expect("execute palette open");
    assert_eq!(state.active_note.as_ref().unwrap().title, "Search Guide");
    assert_eq!(state.active_view, ActiveView::Editor);

    // Switch view via palette action
    state
        .execute_palette_action(PaletteAction::SwitchView(ActiveView::Tasks))
        .expect("switch to tasks view");
    assert_eq!(state.active_view, ActiveView::Tasks);

    // 7. Test Rebuild Index
    let indexed_count = state.rebuild_vault_index().expect("rebuild index");
    assert_eq!(indexed_count, 2, "Should rebuild 2 notes");
}
