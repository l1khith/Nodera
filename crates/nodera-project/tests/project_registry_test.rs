use std::fs;
use tempfile::tempdir;

use nodera_project::{ProjectRegistry, ProjectRegistryEntry};

#[test]
fn test_registry_crud_operations() {
    let tmp = tempdir().unwrap();
    let reg_file = tmp.path().join("test_registry.json");
    std::env::set_var("NODERA_PROJECTS_REGISTRY_PATH", &reg_file);

    let proj_a_dir = tmp.path().join("proj_a");
    let proj_b_dir = tmp.path().join("proj_b");
    fs::create_dir_all(&proj_a_dir).unwrap();
    fs::create_dir_all(&proj_b_dir).unwrap();

    let entry_a = ProjectRegistryEntry {
        id: "proj-a-123".to_string(),
        name: "proj_a".to_string(),
        root: proj_a_dir.clone(),
        project_type: "rust".to_string(),
        last_updated: "2026-09-23T21:00:00Z".to_string(),
    };

    let entry_b = ProjectRegistryEntry {
        id: "proj-b-456".to_string(),
        name: "proj_b".to_string(),
        root: proj_b_dir.clone(),
        project_type: "rust".to_string(),
        last_updated: "2026-09-23T21:05:00Z".to_string(),
    };

    // Register A and B
    ProjectRegistry::register(entry_a.clone()).unwrap();
    ProjectRegistry::register(entry_b.clone()).unwrap();

    let list = ProjectRegistry::list().unwrap();
    assert_eq!(list.len(), 2);

    let found_a = ProjectRegistry::find_by_id("proj-a-123").unwrap();
    assert!(found_a.is_some());
    assert_eq!(found_a.unwrap().name, "proj_a");

    let found_b = ProjectRegistry::find_by_root(&proj_b_dir).unwrap();
    assert!(found_b.is_some());
    assert_eq!(found_b.unwrap().id, "proj-b-456");

    // Unregister A
    ProjectRegistry::unregister("proj-a-123").unwrap();
    let list_after = ProjectRegistry::list().unwrap();
    assert_eq!(list_after.len(), 1);
    assert_eq!(list_after[0].id, "proj-b-456");

    // Prune missing directories
    fs::remove_dir_all(&proj_b_dir).unwrap();
    let pruned = ProjectRegistry::prune_missing().unwrap();
    assert_eq!(pruned, 1);
    assert_eq!(ProjectRegistry::list().unwrap().len(), 0);

    std::env::remove_var("NODERA_PROJECTS_REGISTRY_PATH");
}
