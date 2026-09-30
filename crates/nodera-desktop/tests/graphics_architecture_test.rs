use nodera_desktop::graphics::{
    create_graph_renderer, detect_graphics, AdapterKind, CpuGraphRenderer, GraphEdgeInstance,
    GraphNodeInstance, GraphRenderer, GraphScene, GraphicsDiagnostics, GraphicsRendererPreference,
    LabelLod, RendererConfig, Viewport,
};

#[test]
fn test_adapter_detection_and_diagnostics() {
    let diag = detect_graphics();
    assert!(
        !diag.adapter_name.is_empty(),
        "Adapter name must be populated"
    );
    assert!(!diag.backend.is_empty(), "Backend string must be populated");
    assert!(
        !diag.detected_adapters.is_empty(),
        "Must detect at least 1 adapter summary"
    );
    assert!(
        diag.max_texture_dimension_2d >= 2048,
        "Texture dimension must be reasonable"
    );
}

#[test]
fn test_adapter_kind_display() {
    assert_eq!(AdapterKind::DiscreteGpu.to_string(), "Discrete GPU");
    assert_eq!(AdapterKind::IntegratedGpu.to_string(), "Integrated GPU");
    assert_eq!(AdapterKind::VirtualGpu.to_string(), "Virtual GPU");
    assert_eq!(AdapterKind::Cpu.to_string(), "CPU Software Fallback");
}

#[test]
fn test_viewport_coordinate_invariants() {
    let vp = Viewport {
        pan_x: 250.0,
        pan_y: -180.0,
        zoom: 1.45,
        canvas_width: 1200.0,
        canvas_height: 800.0,
    };

    let screen_pt = (450.0f32, 320.0f32);
    let (wx, wy) = vp.screen_to_world(screen_pt.0, screen_pt.1);
    let (sx, sy) = vp.world_to_screen(wx, wy);

    assert!(
        (sx - screen_pt.0).abs() < 1e-4,
        "Screen X invariant violated"
    );
    assert!(
        (sy - screen_pt.1).abs() < 1e-4,
        "Screen Y invariant violated"
    );
}

#[test]
fn test_spatial_frustum_culling() {
    let vp = Viewport {
        pan_x: 0.0,
        pan_y: 0.0,
        zoom: 1.0,
        canvas_width: 800.0,
        canvas_height: 600.0,
    };

    let nodes = vec![
        GraphNodeInstance {
            id: "visible_center".to_string(),
            x: 400.0,
            y: 300.0,
            radius: 10.0,
            color_rgba: [1.0, 1.0, 1.0, 1.0],
            label: "Center".to_string(),
            degree: 2,
            centrality: 50,
            is_hovered: false,
            is_selected: false,
            is_dimmed: false,
        },
        GraphNodeInstance {
            id: "visible_edge".to_string(),
            x: 790.0,
            y: 590.0,
            radius: 12.0,
            color_rgba: [1.0, 0.0, 0.0, 1.0],
            label: "Edge".to_string(),
            degree: 1,
            centrality: 10,
            is_hovered: false,
            is_selected: false,
            is_dimmed: false,
        },
        GraphNodeInstance {
            id: "culled_far_away".to_string(),
            x: 5000.0,
            y: 5000.0,
            radius: 8.0,
            color_rgba: [0.0, 1.0, 0.0, 1.0],
            label: "Distant".to_string(),
            degree: 0,
            centrality: 0,
            is_hovered: false,
            is_selected: false,
            is_dimmed: false,
        },
    ];

    let edges = vec![
        GraphEdgeInstance {
            source_idx: 0,
            target_idx: 1,
            x1: 400.0,
            y1: 300.0,
            x2: 790.0,
            y2: 590.0,
            color_rgba: [0.5, 0.5, 0.5, 1.0],
            width: 1.5,
            is_dimmed: false,
        },
        GraphEdgeInstance {
            source_idx: 0,
            target_idx: 2,
            x1: 400.0,
            y1: 300.0,
            x2: 5000.0,
            y2: 5000.0,
            color_rgba: [0.5, 0.5, 0.5, 1.0],
            width: 1.0,
            is_dimmed: false,
        },
    ];

    let scene = GraphScene::new(nodes, edges, vp);
    let (vis_nodes, vis_edges) = scene.spatial_cull(20.0);

    // Node 0 and 1 are within visible bounds; node 2 is far away
    assert!(vis_nodes.contains(&0), "Node 0 must be visible");
    assert!(vis_nodes.contains(&1), "Node 1 must be visible");
    assert!(!vis_nodes.contains(&2), "Node 2 must be culled");

    // Both edges touch at least one visible node or cross the frustum
    assert!(vis_edges.contains(&0), "Edge 0 must be visible");
    assert!(
        vis_edges.contains(&1),
        "Edge 1 touches visible node 0 and must be kept"
    );
}

#[test]
fn test_label_lod_thresholds() {
    assert_eq!(LabelLod::from_zoom(1.2), LabelLod::Full);
    assert_eq!(LabelLod::from_zoom(0.75), LabelLod::Full);
    assert_eq!(LabelLod::from_zoom(0.60), LabelLod::Truncated);
    assert_eq!(LabelLod::from_zoom(0.45), LabelLod::Truncated);
    assert_eq!(LabelLod::from_zoom(0.35), LabelLod::MajorHubsOnly);
    assert_eq!(LabelLod::from_zoom(0.25), LabelLod::MajorHubsOnly);
    assert_eq!(LabelLod::from_zoom(0.20), LabelLod::None);
    assert_eq!(LabelLod::from_zoom(0.05), LabelLod::None);
}

#[test]
fn test_cpu_renderer_lifecycle() {
    let mut renderer = CpuGraphRenderer::new();
    let config = RendererConfig {
        width: 1280,
        height: 720,
        dpi_scale: 1.0,
        prefer_low_power: true,
    };

    assert!(renderer.initialize(config).is_ok());
    assert!(!renderer.is_hardware_accelerated());

    let nodes = vec![GraphNodeInstance {
        id: "node_1".to_string(),
        x: 100.0,
        y: 100.0,
        radius: 6.0,
        color_rgba: [0.8, 0.2, 0.2, 1.0],
        label: "Node 1".to_string(),
        degree: 1,
        centrality: 10,
        is_hovered: false,
        is_selected: false,
        is_dimmed: false,
    }];
    let scene = GraphScene::new(nodes, vec![], Viewport::default());

    renderer.update_scene(&scene);
    let stats = renderer.render().expect("CPU render pass must succeed");
    assert_eq!(stats.nodes_rendered, 1);
    assert_eq!(stats.nodes_culled, 0);

    renderer.shutdown();
}

#[test]
fn test_renderer_factory_preferences() {
    let config = RendererConfig::default();

    // ForceCpu preference must always return a CPU fallback renderer
    let (renderer, info) = create_graph_renderer(GraphicsRendererPreference::ForceCpu, config);
    assert!(!info.is_hardware_accelerated);
    assert!(!renderer.is_hardware_accelerated());
    assert_eq!(info.name, "CPU Software Rasterizer");

    // Auto preference attempts GPU first, falling back cleanly to CPU if unavailable
    let (auto_renderer, auto_info) =
        create_graph_renderer(GraphicsRendererPreference::Auto, config);
    assert!(!auto_info.name.is_empty());
    assert_eq!(
        auto_renderer.is_hardware_accelerated(),
        auto_info.is_hardware_accelerated
    );
}

#[test]
fn test_graphics_diagnostics_serialization() {
    let diag = GraphicsDiagnostics::default();
    let serialized = serde_json::to_string(&diag).expect("Must serialize GraphicsDiagnostics");
    let deserialized: GraphicsDiagnostics =
        serde_json::from_str(&serialized).expect("Must deserialize GraphicsDiagnostics");
    assert_eq!(diag, deserialized);
}
