use std::time::Instant;
use tempfile::tempdir;

use nodera_core::{Vault, VaultService};
use nodera_desktop::AppState;
use nodera_index::VaultIndex;

#[test]
fn test_large_vault_1000_notes_scalability_and_search_latency() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().to_path_buf();

    let vault = Vault::create(&vault_path, Some("Large Scale Vault".to_string())).unwrap();
    let service = VaultService::new(vault);

    let total_notes = 1000;
    println!("Creating {total_notes} notes in hierarchical folders...");
    let start_create = Instant::now();

    for i in 0..total_notes {
        let folder = match i % 5 {
            0 => "Projects/Alpha",
            1 => "Projects/Beta",
            2 => "Research/Papers",
            3 => "Daily/2026",
            _ => "Archive",
        };
        let rel_path = format!("{folder}/Note_{i}.md");
        let content = format!(
            "---\ntitle: Note Number {i}\ntags:\n  - scale\n  - batch\n---\n\n# Note Number {i}\n\nThis note is part of the 1000 note large vault stress test.\n\n- [{}] Task for note {i}\n\nReference to [[Projects/Alpha/Note_0]].",
            if i % 3 == 0 { "x" } else { " " }
        );
        service.write_note(&rel_path, &content).unwrap();
    }
    println!(
        "Created {total_notes} notes in {:?}",
        start_create.elapsed()
    );

    // Rebuild index and benchmark performance
    let start_index = Instant::now();
    let mut index = VaultIndex::open(&vault_path).unwrap();
    index.rebuild(&service).unwrap();
    let index_duration = start_index.elapsed();
    println!("Indexed {total_notes} notes in {index_duration:?}");

    // Test Search Latency across 1,000 notes
    let search_start = Instant::now();
    let results = index.search("Number 777", 10).unwrap();
    let search_duration = search_start.elapsed();

    println!(
        "Search returned {} results in {:?}",
        results.len(),
        search_duration
    );
    assert!(
        !results.is_empty(),
        "Should find targeted note in large vault"
    );
    assert_eq!(results[0].title, "Note Number 777");
    assert!(
        search_duration.as_millis() < 50,
        "Search latency in 1,000 note vault should be under 50ms, was {search_duration:?}"
    );

    // Verify task count across 1,000 notes
    let tasks = index.query_tasks(&Default::default()).unwrap();
    assert_eq!(tasks.len(), total_notes);

    // Verify AppState opens large vault seamlessly
    let mut state = AppState::default();
    let state_open_start = Instant::now();
    state.open_vault(&vault_path).unwrap();
    println!(
        "AppState opened large vault in {:?}",
        state_open_start.elapsed()
    );
    let note_entries_count = state
        .entries
        .iter()
        .filter(|e| matches!(e, nodera_core::VaultEntry::Note(_)))
        .count();
    assert_eq!(note_entries_count, total_notes);
}
