use std::fs;
use tempfile::tempdir;

use nodera_core::{Vault, VaultService};
use nodera_desktop::AppState;
use nodera_index::VaultIndex;

#[test]
fn test_index_auto_recovery_from_corrupted_sqlite_and_tantivy() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().to_path_buf();

    // 1. Create vault and notes
    let vault = Vault::create(&vault_path, Some("Recovery Vault".to_string())).unwrap();
    let service = VaultService::new(vault);

    service
        .write_note(
            "Note1.md",
            "# First Note\n\n- [ ] Urgent recovery task\n\nSome important text content.",
        )
        .unwrap();
    service
        .write_note(
            "Note2.md",
            "# Second Note\n\n- [x] Finished task\n\nRelated to [[Note1]].",
        )
        .unwrap();

    // 2. Build initial valid index
    let mut initial_idx = VaultIndex::open(&vault_path).unwrap();
    initial_idx.rebuild(&service).unwrap();

    // Verify search works
    let results = initial_idx.search("important", 10).unwrap();
    assert_eq!(results.len(), 1);

    // Drop index so file locks are released
    drop(initial_idx);

    // 3. Deliberately corrupt SQLite database with garbage bytes
    let sqlite_path = vault_path.join(".nodera").join("index.sqlite");
    assert!(sqlite_path.exists());
    fs::write(
        &sqlite_path,
        b"CORRUPTED_GARBAGE_BYTES_NOT_A_VALID_SQLITE_HEADER",
    )
    .unwrap();

    // 4. Deliberately corrupt Tantivy index by writing a non-directory / broken files
    let tantivy_path = vault_path.join(".nodera").join("tantivy");
    assert!(tantivy_path.exists());
    let meta_json = tantivy_path.join("meta.json");
    if meta_json.exists() {
        fs::write(&meta_json, b"{ invalid json garbage").unwrap();
    }

    // 5. Open vault with AppState — must self-heal and rebuild without crashing or failing
    let mut state = AppState::default();
    let open_res = state.open_vault(&vault_path);
    assert!(
        open_res.is_ok(),
        "AppState::open_vault should successfully self-heal from corrupted index"
    );

    // 6. Verify index is healthy and functional
    assert!(state.vault_index.is_some());
    let idx_arc = state.vault_index.as_ref().unwrap();
    let idx = idx_arc.lock().unwrap();

    // Search should return the note after automatic rebuild
    let recovered_results = idx.search("important", 10).unwrap();
    assert_eq!(recovered_results.len(), 1);
    assert_eq!(recovered_results[0].title, "First Note");

    // Tasks should be recovered from canonical Markdown
    let tasks = idx.query_tasks(&Default::default()).unwrap();
    assert_eq!(tasks.len(), 2);
}
