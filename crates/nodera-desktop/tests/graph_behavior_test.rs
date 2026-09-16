use nodera_desktop::components::graph_view::{
    init_simulation, init_simulation_with_forces, step_simulation_with_forces,
};
use nodera_desktop::state::{GraphForcesSettings, GraphSettings};
use nodera_desktop::AppState;
use nodera_markdown::{GraphData, GraphEdge, GraphNode};
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_graph_deterministic_seeding_and_reproducibility() {
    let mut nodes = Vec::new();
    for i in 0..6 {
        nodes.push(GraphNode {
            id: format!("notes/note_{}.md", i),
            path: PathBuf::from(format!("notes/note_{}.md", i)),
            label: format!("Note {}", i),
            degree: if i < 3 { 2 } else { 1 },
            is_unresolved: false,
            is_tag: false,
        });
    }

    let edges = vec![
        GraphEdge {
            source: "notes/note_0.md".to_string(),
            target: "notes/note_1.md".to_string(),
        },
        GraphEdge {
            source: "notes/note_1.md".to_string(),
            target: "notes/note_2.md".to_string(),
        },
        GraphEdge {
            source: "notes/note_3.md".to_string(),
            target: "notes/note_4.md".to_string(),
        },
    ];

    let graph = GraphData { nodes, edges };

    let (sim_nodes_1, sim_edges_1) = init_simulation(&graph, 1000.0, 700.0);
    let (sim_nodes_2, sim_edges_2) = init_simulation(&graph, 1000.0, 700.0);

    assert_eq!(sim_nodes_1.len(), sim_nodes_2.len());
    assert_eq!(sim_edges_1.len(), sim_edges_2.len());

    for (n1, n2) in sim_nodes_1.iter().zip(sim_nodes_2.iter()) {
        assert_eq!(n1.id, n2.id);
        assert_eq!(n1.label, n2.label);
        assert!(
            (n1.x - n2.x).abs() < f32::EPSILON,
            "X positions must match deterministically"
        );
        assert!(
            (n1.y - n2.y).abs() < f32::EPSILON,
            "Y positions must match deterministically"
        );
        assert_eq!(n1.radius, n2.radius);
    }
}

#[test]
fn test_multi_cluster_separation_and_spring_attraction() {
    // Two distinct clusters:
    // Cluster A: A0, A1, A2 (connected)
    // Cluster B: B0, B1, B2 (connected)
    let nodes = vec![
        GraphNode {
            id: "notes/a0.md".to_string(),
            path: PathBuf::from("notes/a0.md"),
            label: "A0".to_string(),
            degree: 2,
            is_unresolved: false,
            is_tag: false,
        },
        GraphNode {
            id: "notes/a1.md".to_string(),
            path: PathBuf::from("notes/a1.md"),
            label: "A1".to_string(),
            degree: 2,
            is_unresolved: false,
            is_tag: false,
        },
        GraphNode {
            id: "notes/a2.md".to_string(),
            path: PathBuf::from("notes/a2.md"),
            label: "A2".to_string(),
            degree: 2,
            is_unresolved: false,
            is_tag: false,
        },
        GraphNode {
            id: "notes/b0.md".to_string(),
            path: PathBuf::from("notes/b0.md"),
            label: "B0".to_string(),
            degree: 2,
            is_unresolved: false,
            is_tag: false,
        },
        GraphNode {
            id: "notes/b1.md".to_string(),
            path: PathBuf::from("notes/b1.md"),
            label: "B1".to_string(),
            degree: 2,
            is_unresolved: false,
            is_tag: false,
        },
        GraphNode {
            id: "notes/b2.md".to_string(),
            path: PathBuf::from("notes/b2.md"),
            label: "B2".to_string(),
            degree: 2,
            is_unresolved: false,
            is_tag: false,
        },
    ];

    let edges = vec![
        // Cluster A (cohesive cluster)
        GraphEdge {
            source: "notes/a0.md".to_string(),
            target: "notes/a1.md".to_string(),
        },
        GraphEdge {
            source: "notes/a1.md".to_string(),
            target: "notes/a2.md".to_string(),
        },
        GraphEdge {
            source: "notes/a2.md".to_string(),
            target: "notes/a0.md".to_string(),
        },
        // Cluster B (cohesive cluster)
        GraphEdge {
            source: "notes/b0.md".to_string(),
            target: "notes/b1.md".to_string(),
        },
        GraphEdge {
            source: "notes/b1.md".to_string(),
            target: "notes/b2.md".to_string(),
        },
        GraphEdge {
            source: "notes/b2.md".to_string(),
            target: "notes/b0.md".to_string(),
        },
    ];

    let graph = GraphData { nodes, edges };
    let (sim_nodes, _) = init_simulation(&graph, 1200.0, 800.0);

    // Compute center of mass for Cluster A and Cluster B
    let (mut sum_ax, mut sum_ay) = (0.0f32, 0.0f32);
    let (mut sum_bx, mut sum_by) = (0.0f32, 0.0f32);

    for n in &sim_nodes {
        if n.id.starts_with("notes/a") {
            sum_ax += n.x;
            sum_ay += n.y;
        } else if n.id.starts_with("notes/b") {
            sum_bx += n.x;
            sum_by += n.y;
        }
    }

    let a_center = (sum_ax / 3.0, sum_ay / 3.0);
    let b_center = (sum_bx / 3.0, sum_by / 3.0);

    let cluster_distance =
        ((a_center.0 - b_center.0).powi(2) + (a_center.1 - b_center.1).powi(2)).sqrt();

    // Average distance between members of Cluster A
    let a0 = sim_nodes.iter().find(|n| n.id == "notes/a0.md").unwrap();
    let a1 = sim_nodes.iter().find(|n| n.id == "notes/a1.md").unwrap();
    let a2 = sim_nodes.iter().find(|n| n.id == "notes/a2.md").unwrap();

    let d_a0_a1 = ((a0.x - a1.x).powi(2) + (a0.y - a1.y).powi(2)).sqrt();
    let d_a1_a2 = ((a1.x - a2.x).powi(2) + (a1.y - a2.y).powi(2)).sqrt();
    let d_a2_a0 = ((a2.x - a0.x).powi(2) + (a2.y - a0.y).powi(2)).sqrt();
    let avg_internal_a = (d_a0_a1 + d_a1_a2 + d_a2_a0) / 3.0;

    // Average distance between members of Cluster B
    let b0 = sim_nodes.iter().find(|n| n.id == "notes/b0.md").unwrap();
    let b1 = sim_nodes.iter().find(|n| n.id == "notes/b1.md").unwrap();
    let b2 = sim_nodes.iter().find(|n| n.id == "notes/b2.md").unwrap();

    let d_b0_b1 = ((b0.x - b1.x).powi(2) + (b0.y - b1.y).powi(2)).sqrt();
    let d_b1_b2 = ((b1.x - b2.x).powi(2) + (b1.y - b2.y).powi(2)).sqrt();
    let d_b2_b0 = ((b2.x - b0.x).powi(2) + (b2.y - b0.y).powi(2)).sqrt();
    let avg_internal_b = (d_b0_b1 + d_b1_b2 + d_b2_b0) / 3.0;

    // Cluster radius from center
    let radius_a = ((a0.x - a_center.0).powi(2) + (a0.y - a_center.1).powi(2))
        .sqrt()
        .max(((a1.x - a_center.0).powi(2) + (a1.y - a_center.1).powi(2)).sqrt())
        .max(((a2.x - a_center.0).powi(2) + (a2.y - a_center.1).powi(2)).sqrt());

    let radius_b = ((b0.x - b_center.0).powi(2) + (b0.y - b_center.1).powi(2))
        .sqrt()
        .max(((b1.x - b_center.0).powi(2) + (b1.y - b_center.1).powi(2)).sqrt())
        .max(((b2.x - b_center.0).powi(2) + (b2.y - b_center.1).powi(2)).sqrt());

    // Repulsion must separate the two cluster centroids further than their internal radii
    assert!(
        cluster_distance > radius_a,
        "Cluster separation ({:.2}) should exceed cluster A radius ({:.2})",
        cluster_distance,
        radius_a
    );
    assert!(
        cluster_distance > radius_b,
        "Cluster separation ({:.2}) should exceed cluster B radius ({:.2})",
        cluster_distance,
        radius_b
    );

    // Cross-cluster distances between A and B
    let mut cross_distances = Vec::new();
    for a in &[a0, a1, a2] {
        for b in &[b0, b1, b2] {
            let dist = ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt();
            cross_distances.push(dist);
        }
    }
    let avg_cross = cross_distances.iter().sum::<f32>() / cross_distances.len() as f32;

    // Cluster cohesion should keep intra-cluster distance smaller than cross-cluster distance
    assert!(
        avg_internal_a < avg_cross,
        "Cluster A internal distance ({:.2}) should be smaller than cross-cluster distance ({:.2})",
        avg_internal_a,
        avg_cross
    );
    assert!(
        avg_internal_b < avg_cross,
        "Cluster B internal distance ({:.2}) should be smaller than cross-cluster distance ({:.2})",
        avg_internal_b,
        avg_cross
    );
}

#[test]
fn test_collision_avoidance_no_overlapping_nodes() {
    let mut nodes = Vec::new();
    for i in 0..10 {
        nodes.push(GraphNode {
            id: format!("notes/node_{}.md", i),
            path: PathBuf::from(format!("notes/node_{}.md", i)),
            label: format!("Note {}", i),
            degree: 0,
            is_unresolved: false,
            is_tag: false,
        });
    }

    let graph = GraphData {
        nodes,
        edges: Vec::new(),
    };
    let (sim_nodes, _) = init_simulation(&graph, 800.0, 600.0);

    // Ensure all pairs of nodes maintain sufficient distance
    for i in 0..sim_nodes.len() {
        for j in (i + 1)..sim_nodes.len() {
            let dx = sim_nodes[i].x - sim_nodes[j].x;
            let dy = sim_nodes[i].y - sim_nodes[j].y;
            let dist = (dx * dx + dy * dy).sqrt();
            let min_clearance = (sim_nodes[i].radius + sim_nodes[j].radius) * 0.9;
            assert!(
                dist >= min_clearance,
                "Nodes {} and {} should not overlap (dist={:.2}, min_clearance={:.2})",
                i,
                j,
                dist,
                min_clearance
            );
        }
    }
}

#[test]
fn test_app_state_graph_titles_and_local_depths() {
    let tmp = tempdir().expect("tempdir");
    let vault_path = tmp.path().join("GraphTestVault");

    let mut state = AppState::default();
    state
        .create_vault(&vault_path, Some("Graph Test Vault".to_string()))
        .expect("create vault");

    // Create a chain of notes: Alpha -> Beta -> Gamma -> Delta
    state.create_note("Alpha", None).expect("create Alpha");
    state.update_editor_content(
        "---\ntitle: Alpha Document\n---\n# Alpha Document\n\nLinks to [[Beta]].".to_string(),
    );
    state.save_active_note().expect("save Alpha");

    state.create_note("Beta", None).expect("create Beta");
    state.update_editor_content(
        "---\ntitle: Beta Document\n---\n# Beta Document\n\nLinks to [[Gamma]].".to_string(),
    );
    state.save_active_note().expect("save Beta");

    state.create_note("Gamma", None).expect("create Gamma");
    state.update_editor_content(
        "---\ntitle: Gamma Document\n---\n# Gamma Document\n\nLinks to [[Delta]].".to_string(),
    );
    state.save_active_note().expect("save Gamma");

    state.create_note("Delta", None).expect("create Delta");
    state.update_editor_content(
        "---\ntitle: Delta Document\n---\n# Delta Document\n\nTerminal node with [[Alpha]] loop test and self [[Delta]] test."
            .to_string(),
    );
    state.save_active_note().expect("save Delta");

    // Full graph data
    let full_graph = state.get_full_graph_data();
    assert_eq!(full_graph.nodes.len(), 4);

    // Verify display titles used (human readable title from NoteSummary instead of filename with .md extension)
    let alpha_node = full_graph.nodes.iter().find(|n| n.label == "Alpha");
    assert!(
        alpha_node.is_some(),
        "Alpha should use title from NoteSummary"
    );

    // Verify self-link was excluded
    for edge in &full_graph.edges {
        assert_ne!(
            edge.source, edge.target,
            "Self links must be excluded from edges"
        );
    }

    // Select Alpha and test local graph depths
    let alpha_path = state
        .entries
        .iter()
        .find(|e| e.name() == "Alpha")
        .expect("find Alpha entry")
        .relative_path()
        .to_path_buf();

    state.select_note(&alpha_path).expect("select Alpha");

    // Depth 1 from Alpha: should contain Alpha and Beta (and Delta because Delta links to Alpha)
    let local_d1 = state.get_local_graph_data(1);
    assert!(local_d1.nodes.iter().any(|n| n.label == "Alpha"));
    assert!(local_d1.nodes.iter().any(|n| n.label == "Beta"));
    assert!(local_d1.nodes.iter().any(|n| n.label == "Delta"));

    // Depth 2: covers all 4 notes since Gamma connects to Delta and Beta
    let local_d2 = state.get_local_graph_data(2);
    assert_eq!(local_d2.nodes.len(), 4);
}

#[test]
fn test_unresolved_links_synthetic_nodes_and_filter_toggle() {
    let tmp = tempdir().expect("tempdir");
    let vault_path = tmp.path().join("UnresolvedGraphVault");

    let mut state = AppState::default();
    state
        .create_vault(&vault_path, Some("Unresolved Vault".to_string()))
        .expect("create vault");

    // Create 2 existing notes that link to each other AND to 3 non-existent notes
    state
        .create_note("Architecture", None)
        .expect("create Architecture");
    state.update_editor_content(
        "# Architecture\n\nLinks to [[Project Plan]], [[Graph View]], and [[Markdown Engine]]."
            .to_string(),
    );
    state.save_active_note().expect("save Architecture");

    state
        .create_note("Project Plan", None)
        .expect("create Project Plan");
    state.update_editor_content(
        "# Project Plan\n\nLinks to [[Architecture]] and [[Milestones]].".to_string(),
    );
    state.save_active_note().expect("save Project Plan");

    // By default: existing_files_only is FALSE
    let default_graph = state.get_full_graph_data();
    // Notes: Architecture, Project Plan + Unresolved: Graph View, Markdown Engine, Milestones = 5 nodes!
    assert_eq!(default_graph.nodes.len(), 5);
    // Connections:
    // Architecture -> Project Plan (and vice versa undirected or directed)
    // Architecture -> Graph View
    // Architecture -> Markdown Engine
    // Project Plan -> Milestones
    assert!(default_graph.edges.len() >= 4);

    let unresolved_nodes: Vec<_> = default_graph
        .nodes
        .iter()
        .filter(|n| n.is_unresolved)
        .collect();
    assert_eq!(unresolved_nodes.len(), 3);
    assert!(unresolved_nodes.iter().any(|n| n.label == "Graph View"));
    assert!(unresolved_nodes
        .iter()
        .any(|n| n.label == "Markdown Engine"));
    assert!(unresolved_nodes.iter().any(|n| n.label == "Milestones"));

    // Now test toggling existing_files_only = TRUE
    let mut settings = GraphSettings::default();
    settings.filters.existing_files_only = true;

    let filtered_graph = state.get_full_graph_data_with_settings(&settings);
    // Should only contain the 2 existing physical notes
    assert_eq!(filtered_graph.nodes.len(), 2);
    assert!(filtered_graph.nodes.iter().all(|n| !n.is_unresolved));
    // Both notes link to each other (bidirectional)
    assert_eq!(filtered_graph.edges.len(), 2);
}

#[test]
fn test_open_or_create_unresolved_target() {
    let tmp = tempdir().expect("tempdir");
    let vault_path = tmp.path().join("CreateTargetVault");

    let mut state = AppState::default();
    state
        .create_vault(&vault_path, Some("Target Vault".to_string()))
        .expect("create vault");

    // Create Note linking to non-existent "Concept Model"
    state
        .create_note("Overview", None)
        .expect("create Overview");
    state.update_editor_content("# Overview\n\nReferences [[Concept Model]].".to_string());
    state.save_active_note().expect("save Overview");

    let notes_before = state
        .entries
        .iter()
        .filter(|e| matches!(e, nodera_core::VaultEntry::Note(_)))
        .count();
    assert_eq!(notes_before, 1);

    // Call open_or_create_target for "Concept Model" (as occurs when clicking an unresolved node)
    state
        .open_or_create_target("Concept Model")
        .expect("open or create");

    // The note should now exist in entries and be the active note
    let notes_after = state
        .entries
        .iter()
        .filter(|e| matches!(e, nodera_core::VaultEntry::Note(_)))
        .count();
    assert_eq!(notes_after, 2);
    assert!(state.active_note.is_some());
    let active = state.active_note.as_ref().unwrap();
    assert_eq!(active.title, "Concept Model");

    // Re-query graph with default settings: now Concept Model is no longer unresolved!
    let updated_graph = state.get_full_graph_data();
    assert_eq!(updated_graph.nodes.len(), 2);
    let concept_node = updated_graph
        .nodes
        .iter()
        .find(|n| n.label == "Concept Model")
        .unwrap();
    assert!(
        !concept_node.is_unresolved,
        "Now that Concept Model was created on disk, it is a resolved note"
    );
}

#[test]
fn test_graph_settings_defaults_and_forces() {
    let mut settings = GraphSettings::default();
    assert!(!settings.filters.existing_files_only);
    assert!(settings.filters.orphans);
    assert!(!settings.filters.tags);
    assert!(!settings.filters.attachments);
    assert!(!settings.is_panel_open);
    assert!(settings.expanded_sections.filters);
    assert!(!settings.expanded_sections.groups);
    assert!(!settings.expanded_sections.display);
    assert!(!settings.expanded_sections.forces);
    assert_eq!(settings.forces.repel_force, 8.0);
    assert_eq!(settings.forces.center_force, 0.40);
    assert_eq!(settings.forces.link_force, 0.80);
    assert_eq!(settings.forces.link_distance, 120.0);

    // Modify settings
    settings.filters.existing_files_only = true;
    settings.forces.repel_force = 12.0;
    settings.forces.center_force = 0.5;
    settings.forces.link_distance = 150.0;
    settings.display.arrows = true;
    settings.display.node_size = 1.5;

    assert!(settings.filters.existing_files_only);
    assert_eq!(settings.forces.repel_force, 12.0);

    // Test reset_to_defaults
    settings.reset_to_defaults();
    assert!(!settings.filters.existing_files_only);
    assert_eq!(settings.forces.repel_force, 8.0);
    assert_eq!(settings.forces.link_distance, 120.0);
    assert_eq!(settings.display.node_size, 1.0);
    assert!(!settings.display.arrows);

    // Test simulation with customized forces
    let mut nodes = Vec::new();
    for i in 0..4 {
        nodes.push(GraphNode {
            id: format!("n_{}.md", i),
            path: PathBuf::from(format!("n_{}.md", i)),
            label: format!("N{}", i),
            degree: 1,
            is_unresolved: false,
            is_tag: false,
        });
    }
    let edges = vec![
        GraphEdge {
            source: "n_0.md".to_string(),
            target: "n_1.md".to_string(),
        },
        GraphEdge {
            source: "n_2.md".to_string(),
            target: "n_3.md".to_string(),
        },
    ];
    let graph = GraphData { nodes, edges };

    let forces = GraphForcesSettings {
        repel_force: 3.0,
        link_distance: 2.0,
        ..Default::default()
    };

    let (mut sim_nodes, sim_edges) = init_simulation_with_forces(&graph, 800.0, 600.0, &forces);
    assert_eq!(sim_nodes.len(), 4);
    assert_eq!(sim_edges.len(), 2);

    // Run step with forces
    step_simulation_with_forces(
        &mut sim_nodes,
        &sim_edges,
        (400.0, 300.0),
        None,
        0.5,
        &forces,
    );
    for n in &sim_nodes {
        assert!(!n.x.is_nan());
        assert!(!n.y.is_nan());
    }
}
