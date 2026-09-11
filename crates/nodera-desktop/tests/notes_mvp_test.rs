use std::fs;
use tempfile::tempdir;

use nodera_core::{Vault, VaultService};

#[test]
fn test_vault_create_note_edit_save_reopen_workflow() {
    let tmp = tempdir().unwrap();
    let vault_root = tmp.path().join("UserVault");

    // 1. User creates a vault
    let vault = Vault::create(&vault_root, Some("Personal Vault".to_string())).unwrap();
    let service = VaultService::new(vault);

    // 2. User creates a new note in "Notes" folder
    let note = service
        .create_note(
            Some("Notes"),
            "Getting Started",
            Some("# Welcome to Nodera\nThis is a local-first note."),
        )
        .unwrap();

    assert_eq!(note.title, "Getting Started");
    assert_eq!(
        note.relative_path,
        std::path::Path::new("Notes").join("Getting Started.md")
    );

    // 3. User edits note and saves
    let updated_content =
        "# Welcome to Nodera\nThis note has been updated with more details.\n- [ ] Try task";
    let saved_note = service
        .write_note(&note.relative_path, updated_content)
        .unwrap();
    assert_eq!(saved_note.content, updated_content);

    // 4. Simulate application restart: drop service, reopen vault from disk
    drop(service);

    let reopened_vault = Vault::open(&vault_root).unwrap();
    assert_eq!(reopened_vault.config().name, "Personal Vault");

    let reopened_service = VaultService::new(reopened_vault);

    // 5. User reopens note and verifies persisted content
    let loaded_note = reopened_service
        .read_note("Notes/Getting Started.md")
        .unwrap();
    assert_eq!(loaded_note.title, "Getting Started");
    assert_eq!(loaded_note.content, updated_content);

    // 6. User renames note
    let renamed = reopened_service
        .rename_note(&loaded_note.relative_path, "Nodera Guide")
        .unwrap();
    assert_eq!(renamed.title, "Nodera Guide");
    assert_eq!(
        renamed.relative_path,
        std::path::Path::new("Notes").join("Nodera Guide.md")
    );

    // Old file no longer exists
    assert!(reopened_service
        .read_note("Notes/Getting Started.md")
        .is_err());
    // New file exists
    assert!(reopened_service.read_note("Notes/Nodera Guide.md").is_ok());

    // 7. User moves note to "Projects" folder
    let moved = reopened_service
        .move_note(&renamed.relative_path, "Projects")
        .unwrap();
    assert_eq!(
        moved.relative_path,
        std::path::Path::new("Projects").join("Nodera Guide.md")
    );
    assert!(reopened_service
        .read_note("Projects/Nodera Guide.md")
        .is_ok());

    // 8. User lists entries
    let entries = reopened_service.list_entries().unwrap();
    assert!(entries.iter().any(|e| e.name() == "Nodera Guide"));

    // 9. User deletes note
    reopened_service.delete_note(&moved.relative_path).unwrap();
    assert!(reopened_service
        .read_note("Projects/Nodera Guide.md")
        .is_err());
}

#[test]
fn test_external_file_creation_detected() {
    let tmp = tempdir().unwrap();
    let vault_root = tmp.path().join("ExternalVault");

    let vault = Vault::create(&vault_root, None).unwrap();
    let service = VaultService::new(vault);

    // Simulate an external editor creating a file in the vault
    let external_note_path = vault_root.join("Notes").join("External Note.md");
    fs::write(&external_note_path, "# Created Externally in VS Code").unwrap();

    // VaultService listing immediately reflects the new file
    let entries = service.list_entries().unwrap();
    let note_entry = entries.iter().find(|e| e.name() == "External Note");
    assert!(
        note_entry.is_some(),
        "External note must be discoverable in file tree"
    );

    // Read the externally created note through service
    let note = service.read_note("Notes/External Note.md").unwrap();
    assert_eq!(note.content, "# Created Externally in VS Code");
}
