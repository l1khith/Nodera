use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub fn toggle(&self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Theme::Dark => "theme-dark",
            Theme::Light => "theme-light",
        }
    }
}

pub const BASE_CSS: &str = r#"
:root, .theme-dark {
    --bg-app: #0B0F14;
    --bg-sidebar: #0E1218;
    --bg-sidebar-hover: #161D26;
    --bg-sidebar-active: #202A3E;
    --bg-surface: #11161D;
    --bg-surface-elevated: #161D26;
    --bg-hover: #1B2330;
    --bg-active: #202A3E;
    --border: #28313C;
    --border-strong: #35404D;
    --border-subtle: #1E2530;
    --text-primary: #ECF0F4;
    --text-secondary: #A5AFBC;
    --text-muted: #74808E;
    --text-disabled: #4E5664;
    --accent: #6680FF;
    --accent-hover: #7890FF;
    --accent-pressed: #5369D8;
    --accent-secondary: #4EA3C7;
    --accent-focus: rgba(102, 128, 255, 0.25);
    --selection: #29375D;
    --focus: #6680FF;
    --success: #5DBB8A;
    --warning: #D4A653;
    --danger: #D66D79;
    --info: #5EA6D4;
    --status-success: #5DBB8A;
    --status-warning: #D4A653;
    --status-danger: #D66D79;
    --status-info: #5EA6D4;
    --graph-node: #5C6FE6;
    --graph-node-current: #8A72F5;
    --graph-node-selected: #8A72F5;
    --graph-node-hover: #7890FF;
    --graph-node-connected: #7182EF;
    --graph-node-unrelated: #66717D;
    --graph-edge: #39434F;
    --graph-edge-highlight: #7182EF;
    --graph-label: #D7DCE4;
    --graph-grid-dot: rgba(255, 255, 255, 0.05);
    --shadow-none: none;
    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.2);
    --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.3);
    --shadow-lg: 0 8px 24px rgba(0, 0, 0, 0.45);
    --font-ui: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    --font-editor: "Cascadia Code", "Fira Code", Consolas, "Courier New", monospace;
    --border-color: var(--border);
    --bg-primary: var(--bg-app);
    --bg-secondary: var(--bg-surface);
    --bg-tertiary: var(--bg-surface-elevated);
    --accent-color: var(--accent);
}

.theme-light {
    --bg-app: #F4F6F8;
    --bg-sidebar: #ECEFF3;
    --bg-sidebar-hover: #E3E7ED;
    --bg-sidebar-active: #D7DEE6;
    --bg-surface: #FFFFFF;
    --bg-surface-elevated: #FAFBFC;
    --bg-hover: #EEF2F5;
    --bg-active: #E7ECF4;
    --border: #D7DEE6;
    --border-strong: #C4CDD7;
    --border-subtle: #EBEFF5;
    --text-primary: #1B222B;
    --text-secondary: #56616E;
    --text-muted: #7D8792;
    --text-disabled: #A4ACB5;
    --accent: #4F63DF;
    --accent-hover: #4054C9;
    --accent-pressed: #3346AC;
    --accent-secondary: #2F7F9F;
    --accent-focus: rgba(79, 99, 223, 0.2);
    --selection: #DEE4FF;
    --focus: #4F63DF;
    --success: #338E65;
    --warning: #A97824;
    --danger: #B94F5C;
    --info: #397DA4;
    --status-success: #338E65;
    --status-warning: #A97824;
    --status-danger: #B94F5C;
    --status-info: #397DA4;
    --graph-node: #5C6FE6;
    --graph-node-current: #8A72F5;
    --graph-node-selected: #8A72F5;
    --graph-node-hover: #7182EF;
    --graph-node-connected: #7182EF;
    --graph-node-unrelated: #9AA3AC;
    --graph-edge: #D7DEE6;
    --graph-edge-highlight: #4F63DF;
    --graph-label: #1B222B;
    --graph-grid-dot: rgba(23, 26, 35, 0.05);
    --shadow-none: none;
    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.08);
    --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.1);
    --shadow-lg: 0 8px 24px rgba(0, 0, 0, 0.15);
    --font-ui: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    --font-editor: "Cascadia Code", "Fira Code", Consolas, "Courier New", monospace;
    --border-color: var(--border);
    --bg-primary: var(--bg-app);
    --bg-secondary: var(--bg-surface);
    --bg-tertiary: var(--bg-surface-elevated);
    --accent-color: var(--accent);
}

* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body, html {
    width: 100%;
    height: 100%;
    overflow: hidden;
    background-color: var(--bg-app);
    color: var(--text-primary);
    font-family: var(--font-ui);
    font-size: 13px;
    user-select: none;
}

button {
    font-family: inherit;
    font-size: inherit;
    color: inherit;
    background: none;
    border: none;
    cursor: pointer;
}

input, textarea {
    font-family: inherit;
    color: inherit;
    background: transparent;
    border: 1px solid var(--border);
    outline: none;
}

/* Layout classes */
.app-container {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    background-color: var(--bg-app);
}

.top-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 40px;
    background-color: var(--bg-sidebar);
    border-bottom: 1px solid var(--border);
    padding: 0 12px;
    gap: 12px;
}

.top-bar-left, .top-bar-center, .top-bar-right {
    display: flex;
    align-items: center;
    gap: 8px;
}

.brand-title {
    font-weight: 700;
    font-size: 14px;
    letter-spacing: 0.5px;
    color: var(--accent);
}

.vault-badge {
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 2px 8px;
    font-size: 12px;
    color: var(--text-secondary);
    max-width: 200px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.main-workspace {
    display: flex;
    flex: 1;
    overflow: hidden;
    position: relative;
}

.pane-sidebar {
    width: 100%;
    min-width: 180px;
    background-color: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transition: width 0.15s ease;
}

.pane-resizer {
    width: 5px;
    cursor: col-resize;
    background-color: transparent;
    border-left: 1px solid var(--border);
    transition: background-color 0.15s ease;
    z-index: 5;
    flex-shrink: 0;
}

.pane-resizer:hover {
    background-color: var(--accent);
}

.pane-center {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-app);
    overflow: hidden;
}

/* Multi-tab bar */
.tab-bar {
    display: flex;
    align-items: center;
    background-color: var(--bg-surface-elevated);
    border-bottom: 1px solid var(--border);
    height: 38px;
    padding: 0 8px;
    gap: 4px;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: thin;
    user-select: none;
}

.tab-item {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: 6px 6px 0 0;
    font-size: 12px;
    color: var(--text-secondary);
    background: transparent;
    border: 1px solid transparent;
    border-bottom: none;
    cursor: pointer;
    max-width: 180px;
    white-space: nowrap;
    transition: all 0.12s ease;
}

.tab-item:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
}

.tab-item.active {
    background-color: var(--bg-surface);
    color: var(--text-primary);
    font-weight: 500;
    border-color: var(--border);
}

.tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 120px;
}

.tab-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    color: var(--text-muted);
    transition: all 0.1s ease;
}

.tab-close:hover {
    background-color: var(--bg-active);
    color: var(--text-primary);
}

.tab-dirty-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: var(--accent);
    display: inline-block;
}

/* Breadcrumbs */
.breadcrumb-container {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-muted);
}

.breadcrumb-segment {
    color: var(--text-secondary);
}

.breadcrumb-active {
    color: var(--text-primary);
    font-weight: 600;
}

.pane-context {
    min-width: 180px;
    background-color: var(--bg-sidebar);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transition: width 0.15s ease;
}

.pane-hidden {
    display: none !important;
}

.bottom-statusbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 24px;
    background-color: var(--bg-sidebar);
    border-top: 1px solid var(--border);
    padding: 0 12px;
    font-size: 11px;
    color: var(--text-muted);
}

.btn-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 4px;
    color: var(--text-secondary);
    transition: background-color 0.1s;
}

.btn-icon:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
}

/* Vector Icon Utilities */
.icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    vertical-align: middle;
    flex-shrink: 0;
}

.icon-sm {
    width: 14px;
    height: 14px;
}

.icon-md {
    width: 16px;
    height: 16px;
}

.icon-lg {
    width: 20px;
    height: 20px;
}

.icon-hero {
    width: 48px;
    height: 48px;
}

.btn-action {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 4px;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    font-size: 12px;
    color: var(--text-primary);
    transition: background-color 0.1s, border-color 0.1s;
}

.btn-action:hover {
    background-color: var(--bg-hover);
    border-color: var(--accent);
}

.btn-primary {
    background-color: var(--accent);
    color: #ffffff;
    border: 1px solid var(--accent);
}

.btn-primary:hover {
    background-color: var(--accent-hover);
}

.btn-danger {
    color: var(--danger);
}

.btn-danger:hover {
    background-color: var(--bg-hover);
    color: var(--danger-hover);
}

/* Modal styles */
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
}

.modal-dialog {
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 20px;
    width: 420px;
    max-width: 90vw;
    box-shadow: var(--shadow-lg);
}

.modal-title {
    font-size: 16px;
    font-weight: 600;
    margin-bottom: 12px;
}

.modal-body {
    margin-bottom: 20px;
    color: var(--text-secondary);
}

.modal-input {
    width: 100%;
    padding: 8px 10px;
    border-radius: 4px;
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    font-size: 13px;
    color: var(--text-primary);
    margin-top: 8px;
}

.modal-input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-focus);
}

.modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
}

/* Reading View & Markdown Styling */
.reading-view h1 { font-size: 24px; font-weight: 700; margin: 24px 0 12px; border-bottom: 1px solid var(--border); padding-bottom: 8px; color: var(--text-primary); }
.reading-view h2 { font-size: 20px; font-weight: 600; margin: 20px 0 10px; border-bottom: 1px solid var(--border-subtle); padding-bottom: 6px; color: var(--text-primary); }
.reading-view h3 { font-size: 16px; font-weight: 600; margin: 16px 0 8px; color: var(--text-primary); }
.reading-view h4, .reading-view h5, .reading-view h6 { font-size: 14px; font-weight: 600; margin: 12px 0 6px; color: var(--text-primary); }
.reading-view p { margin: 10px 0; line-height: 1.7; }
.reading-view ul, .reading-view ol { margin: 10px 0 10px 24px; }
.reading-view li { margin: 4px 0; }
.reading-view blockquote { border-left: 3px solid var(--accent); padding: 6px 16px; margin: 12px 0; color: var(--text-secondary); background: var(--bg-surface); border-radius: 0 4px 4px 0; }
.reading-view code { background: var(--bg-surface-elevated); padding: 2px 6px; border-radius: 3px; font-family: var(--font-editor); font-size: 13px; }
.reading-view pre { background: var(--bg-surface-elevated); padding: 12px; border-radius: 6px; overflow-x: auto; margin: 14px 0; border: 1px solid var(--border); }
.reading-view pre code { background: none; padding: 0; }
.reading-view table { border-collapse: collapse; width: 100%; margin: 16px 0; }
.reading-view th, .reading-view td { border: 1px solid var(--border); padding: 8px 12px; text-align: left; }
.reading-view th { background: var(--bg-surface); font-weight: 600; }
.reading-view a { color: var(--accent); text-decoration: none; }
.reading-view a:hover { text-decoration: underline; color: var(--accent-hover); }
.reading-view a.wikilink { color: var(--accent-hover); font-weight: 500; border-bottom: 1px dashed var(--accent); padding-bottom: 1px; }
.reading-view a.wikilink:hover { background-color: var(--accent-focus); border-radius: 2px; }

/* Callout blocks */
.callout {
    margin: 16px 0;
    border-radius: 6px;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-left-width: 4px;
    overflow: hidden;
}

.callout-title {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    font-weight: 600;
    font-size: 13px;
    background-color: var(--bg-hover);
    border-bottom: 1px solid var(--border-subtle);
}

.callout-content {
    padding: 10px 14px;
    font-size: 13px;
    line-height: 1.6;
    color: var(--text-primary);
}

.callout-content p:first-child { margin-top: 0; }
.callout-content p:last-child { margin-bottom: 0; }

.callout-note { border-left-color: #3b82f6; }
.callout-note .callout-title { color: #3b82f6; }

.callout-tip { border-left-color: #10b981; }
.callout-tip .callout-title { color: #10b981; }

.callout-warning { border-left-color: #f59e0b; }
.callout-warning .callout-title { color: #f59e0b; }

.callout-important { border-left-color: #8b5cf6; }
.callout-important .callout-title { color: #8b5cf6; }

.callout-caution { border-left-color: #ef4444; }
.callout-caution .callout-title { color: #ef4444; }

.callout-success { border-left-color: #22c55e; }
.callout-success .callout-title { color: #22c55e; }

/* Math rendering */
.math {
    font-family: 'KaTeX_Math', 'Cambria Math', 'Times New Roman', serif;
    font-style: italic;
    color: var(--accent-hover);
}

.math-display {
    display: block;
    text-align: center;
    margin: 14px 0;
    overflow-x: auto;
}

/* Context Panel & Knowledge Components */
.tag-badge { display: inline-block; background-color: var(--bg-surface-elevated); color: var(--accent-hover); border: 1px solid var(--border); border-radius: 12px; padding: 2px 8px; font-size: 11px; font-weight: 500; margin: 2px 4px 2px 0; }
.link-list { display: flex; flex-direction: column; gap: 4px; margin-top: 6px; }
.link-item { display: flex; align-items: center; justify-content: space-between; padding: 5px 8px; border-radius: 4px; font-size: 12px; cursor: pointer; color: var(--text-primary); background-color: var(--bg-surface); border: 1px solid var(--border-subtle); transition: background-color 0.1s, border-color 0.1s; text-align: left; width: 100%; }
.link-item:hover { background-color: var(--bg-hover); border-color: var(--accent); }
.link-item-unresolved { color: var(--text-muted); font-style: italic; }
.command-palette-row:hover { background-color: var(--bg-hover); }

/* 2D Knowledge Graph Styling */
.graph-container {
    position: relative;
    width: 100%;
    height: 100%;
    flex: 1;
    overflow: hidden;
    background-color: var(--bg-app);
    user-select: none;
}

.graph-canvas {
    width: 100%;
    height: 100%;
    display: block;
    cursor: grab;
}

.graph-canvas:active {
    cursor: grabbing;
}

.graph-toolbar {
    position: absolute;
    top: 14px;
    right: 14px;
    display: flex;
    align-items: center;
    gap: 3px;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 20px;
    padding: 3px 6px;
    box-shadow: var(--shadow-sm);
    z-index: 10;
}

.graph-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: 13px;
    color: var(--text-secondary);
    transition: background-color 0.1s, color 0.1s;
}

.graph-btn:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
}

.graph-zoom-label {
    font-size: 11px;
    font-weight: 500;
    min-width: 36px;
    text-align: center;
    color: var(--text-muted);
    user-select: none;
}

.graph-divider {
    width: 1px;
    height: 14px;
    background-color: var(--border);
    margin: 0 2px;
}

.graph-stats {
    position: absolute;
    bottom: 14px;
    left: 14px;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 14px;
    padding: 4px 12px;
    font-size: 11px;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    gap: 10px;
    box-shadow: var(--shadow-sm);
    z-index: 10;
}

.graph-search-bar {
    position: absolute;
    top: 14px;
    left: 14px;
    display: flex;
    align-items: center;
    gap: 8px;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 20px;
    padding: 4px 12px;
    box-shadow: var(--shadow-sm);
    z-index: 10;
    min-width: 200px;
}

.local-graph-box {
    width: 100%;
    height: 220px;
    position: relative;
    border-radius: 6px;
    overflow: hidden;
    background-color: var(--bg-surface);
    border: 1px solid var(--border-subtle);
}

.graph-pill-group {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 2px 4px;
}

.graph-pill {
    font-size: 10px;
    font-weight: 500;
    color: var(--text-muted);
    background: transparent;
    border: none;
    border-radius: 8px;
    padding: 1px 6px;
    cursor: pointer;
    transition: all 0.1s ease;
}

.graph-pill:hover {
    color: var(--text-primary);
    background-color: var(--bg-hover);
}

.graph-pill.active {
    color: #ffffff;
    background-color: var(--accent);
}

/* Graph Settings Drawer */
.graph-canvas-wrapper {
    position: relative;
    flex: 1;
    height: 100%;
    overflow: hidden;
}

.graph-settings-panel {
    width: 280px;
    height: 100%;
    background-color: var(--bg-surface);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    z-index: 20;
    box-shadow: -4px 0 16px rgba(0, 0, 0, 0.2);
    animation: slideInRight 0.15s ease-out;
}

@keyframes slideInRight {
    from { transform: translateX(20px); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
}

.graph-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border-subtle);
}

.graph-panel-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: 0.3px;
}

.graph-panel-content {
    flex: 1;
    overflow-y: auto;
    padding: 10px 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.graph-section {
    display: flex;
    flex-direction: column;
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: 10px;
}

.graph-section:last-child {
    border-bottom: none;
}

.graph-section-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 0;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    cursor: pointer;
    user-select: none;
    transition: color 0.1s;
}

.graph-section-header:hover {
    color: var(--text-primary);
}

.graph-section-body {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 4px;
}

.graph-toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 3px 0;
    font-size: 12px;
    color: var(--text-secondary);
}

.graph-toggle-row:hover {
    color: var(--text-primary);
}

.graph-switch {
    position: relative;
    display: inline-block;
    width: 30px;
    height: 16px;
}

.graph-switch input {
    opacity: 0;
    width: 0;
    height: 0;
}

.graph-switch-slider {
    position: absolute;
    cursor: pointer;
    top: 0; left: 0; right: 0; bottom: 0;
    background-color: var(--border-strong);
    transition: .15s ease;
    border-radius: 16px;
}

.graph-switch-slider:before {
    position: absolute;
    content: "";
    height: 12px;
    width: 12px;
    left: 2px;
    bottom: 2px;
    background-color: var(--text-primary);
    transition: .15s ease;
    border-radius: 50%;
}

input:checked + .graph-switch-slider {
    background-color: var(--accent);
}

input:checked + .graph-switch-slider:before {
    transform: translateX(14px);
    background-color: #ffffff;
}

.graph-slider-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 4px 0;
}

.graph-slider-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 11px;
    color: var(--text-secondary);
}

.graph-slider-val {
    font-size: 11px;
    font-family: var(--font-editor);
    color: var(--text-muted);
}

.graph-slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    border-radius: 2px;
    background: var(--border-strong);
    outline: none;
    margin: 4px 0;
}

.graph-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--accent);
    cursor: pointer;
    transition: background 0.15s;
}

.graph-slider::-webkit-slider-thumb:hover {
    background: var(--accent-hover);
}

/* Library View & Document Cards */
.library-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
    background-color: var(--bg-app);
}

.library-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 28px;
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-surface);
    flex-shrink: 0;
}

.library-content {
    flex: 1;
    overflow-y: auto;
    padding: 28px;
}

.library-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
    gap: 24px;
}

.library-card {
    display: flex;
    flex-direction: column;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    box-shadow: var(--shadow-sm);
    transition: border-color 0.15s ease;
}

.library-card:hover {
    border-color: var(--border-strong);
}

.library-card-header {
    padding: 16px 18px;
    background-color: var(--bg-surface-elevated);
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    align-items: flex-start;
    gap: 14px;
}

.library-card-cover {
    width: 48px;
    height: 64px;
    border-radius: 4px;
    background-color: var(--accent);
    color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    border: 1px solid var(--border-subtle);
}

.library-card-body {
    padding: 16px 18px;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.library-badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
}

.library-tag {
    font-size: 11px;
    font-weight: 500;
    padding: 3px 8px;
    border-radius: 4px;
    background-color: var(--accent-focus);
    border: 1px solid var(--border-subtle);
    color: var(--accent);
}

.library-progress-track {
    height: 6px;
    border-radius: 3px;
    background-color: var(--bg-hover);
    border: 1px solid var(--border);
    overflow: hidden;
}

.library-progress-bar {
    height: 100%;
    background-color: var(--accent);
    border-radius: 2px;
    transition: width 0.3s ease;
}

.library-card-footer {
    padding: 12px 18px;
    border-top: 1px solid var(--border-subtle);
    background-color: var(--bg-surface-elevated);
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
}

/* Code Blocks & Syntax Highlighting */
.code-block-wrapper {
    margin: 16px 0;
    border-radius: 8px;
    border: 1px solid var(--border);
    background-color: var(--bg-surface);
    overflow: hidden;
    box-shadow: var(--shadow-sm);
}

.code-block-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 14px;
    background-color: var(--bg-surface-elevated);
    border-bottom: 1px solid var(--border-subtle);
    font-size: 11px;
    font-family: var(--font-editor);
}

.code-block-lang {
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
}

.code-block-copy-btn {
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-secondary);
    padding: 2px 8px;
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s ease;
}

.code-block-copy-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--accent);
}

.code-block-wrapper pre {
    margin: 0;
    padding: 14px 16px;
    overflow-x: auto;
    font-family: var(--font-editor);
    font-size: 13px;
    line-height: 1.55;
}

/* Mermaid Diagrams */
.mermaid-diagram {
    margin: 20px 0;
    border-radius: 8px;
    border: 1px solid var(--border);
    background-color: var(--bg-surface);
    overflow: hidden;
}

.mermaid-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    background-color: var(--bg-surface-elevated);
    border-bottom: 1px solid var(--border-subtle);
    font-size: 11px;
}

.mermaid-badge {
    background: var(--accent-focus);
    color: var(--accent-hover);
    font-weight: 600;
    text-transform: uppercase;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 10px;
    letter-spacing: 0.5px;
}

.mermaid-title {
    color: var(--text-secondary);
    font-weight: 500;
}

.mermaid-svg {
    padding: 24px;
    display: flex;
    justify-content: center;
    align-items: center;
    overflow-x: auto;
    background-color: var(--bg-surface);
}

.mermaid-svg svg {
    max-width: 100%;
    height: auto;
}

/* Transcluded Note Embeds */
.note-embed {
    margin: 16px 0;
    border-radius: 8px;
    border: 1px solid var(--border);
    border-left: 4px solid var(--accent);
    background-color: var(--bg-surface);
    overflow: hidden;
    box-shadow: var(--shadow-sm);
}

.note-embed-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    background-color: var(--bg-surface-elevated);
    border-bottom: 1px solid var(--border-subtle);
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
}

.note-embed-title {
    color: var(--accent);
}

.note-embed-content {
    padding: 16px 20px;
    font-size: 13px;
    line-height: 1.65;
    color: var(--text-secondary);
    max-height: 400px;
    overflow-y: auto;
}

.note-embed-missing {
    margin: 12px 0;
    padding: 10px 14px;
    border-radius: 6px;
    border: 1px dashed var(--warning);
    background-color: rgba(227, 169, 59, 0.08);
    color: var(--warning);
    font-size: 12px;
    display: flex;
    align-items: center;
    gap: 8px;
}

/* Properties Drawer (Frontmatter) */
.properties-drawer {
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-surface);
    padding: 12px 20px;
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.properties-drawer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.property-count-badge {
    background-color: var(--accent-focus);
    color: var(--accent-hover);
    font-size: 10px;
    font-weight: 700;
    padding: 1px 6px;
    border-radius: 10px;
}

.properties-table {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.property-row {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 12px;
}

.property-label {
    width: 110px;
    font-weight: 500;
    color: var(--text-muted);
    font-family: var(--font-editor);
    font-size: 11px;
    flex-shrink: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.property-value-cell {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
}

.property-input {
    flex: 1;
    min-width: 140px;
    max-width: 450px;
    background: var(--bg-app);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 4px 8px;
    color: var(--text-primary);
    font-size: 12px;
    outline: none;
    transition: border-color 0.15s ease;
}

.property-input:focus {
    border-color: var(--accent);
}

.property-mini-input {
    background: var(--bg-app);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 3px 8px;
    color: var(--text-primary);
    font-size: 11px;
    outline: none;
    transition: border-color 0.15s ease;
}

.property-mini-input:focus {
    border-color: var(--accent);
}

.property-type-select {
    background: var(--bg-app);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 3px 6px;
    color: var(--text-secondary);
    font-size: 11px;
    outline: none;
    cursor: pointer;
}

.property-add-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 6px;
    padding-top: 8px;
    border-top: 1px dashed var(--border-subtle);
}

.property-remove-btn {
    width: 20px;
    height: 20px;
    color: var(--text-muted);
}

.property-remove-btn:hover {
    color: var(--danger);
}

.tag-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background-color: var(--accent-focus);
    color: var(--accent-hover);
    border-radius: 4px;
    padding: 2px 8px;
    font-size: 11px;
    font-weight: 500;
}

.tag-chip-remove {
    background: transparent;
    border: none;
    color: currentColor;
    font-size: 12px;
    cursor: pointer;
    line-height: 1;
    padding: 0;
    opacity: 0.7;
}

.tag-chip-remove:hover {
    opacity: 1;
}

/* Split View */
.split-pane-header {
    height: 32px;
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-surface-elevated);
    display: flex;
    align-items: center;
    padding: 0 12px;
    user-select: none;
}

.split-divider {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    background-color: var(--bg-surface);
    border-left: 1px solid var(--border);
    border-right: 1px solid var(--border);
    padding: 6px;
    user-select: none;
    flex-shrink: 0;
}

/* Unlinked Mentions */
.unlinked-mention-card {
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    transition: border-color 0.15s ease;
}

.unlinked-mention-card:hover {
    border-color: var(--accent);
}

.unlinked-snippet {
    font-size: 11px;
    line-height: 1.5;
    color: var(--text-muted);
}

/* Vault Selector & Top Bar Actions */
.vault-selector-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
    cursor: pointer;
    transition: background-color 0.1s, border-color 0.1s;
}

.vault-selector-btn:hover {
    background-color: var(--bg-hover);
    border-color: var(--border-strong);
}

.top-search-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-width: 220px;
    max-width: 360px;
    padding: 4px 12px;
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: 12px;
    color: var(--text-muted);
    cursor: pointer;
    transition: border-color 0.15s ease, background-color 0.15s ease;
}

.top-search-btn:hover {
    border-color: var(--border-strong);
    color: var(--text-secondary);
    background-color: var(--bg-hover);
}

.kbd-badge {
    margin-left: auto;
    font-size: 10px;
    font-family: var(--font-editor);
    color: var(--text-muted);
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 1px 5px;
}

/* Popover Dropdown Menus */
.dropdown-menu {
    position: absolute;
    top: calc(100% + 6px);
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: var(--shadow-md);
    min-width: 200px;
    padding: 4px;
    z-index: 1000;
    display: flex;
    flex-direction: column;
}

.dropdown-header {
    padding: 6px 10px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
}

.dropdown-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    font-size: 12px;
    color: var(--text-primary);
    border-radius: 4px;
    cursor: pointer;
    background: transparent;
    border: none;
    width: 100%;
    text-align: left;
    transition: background-color 0.1s ease;
}

.dropdown-item:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
}

.dropdown-item.danger {
    color: var(--danger);
}

.dropdown-item.danger:hover {
    background-color: rgba(239, 68, 68, 0.1);
    color: var(--danger-hover);
}

.dropdown-divider {
    height: 1px;
    background-color: var(--border-subtle);
    margin: 4px 0;
}

/* Context Menu */
.context-menu {
    position: fixed;
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: var(--shadow-lg);
    min-width: 170px;
    padding: 4px;
    z-index: 1100;
    display: flex;
    flex-direction: column;
}

.context-menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    font-size: 12px;
    color: var(--text-primary);
    border-radius: 4px;
    cursor: pointer;
    background: transparent;
    border: none;
    width: 100%;
    text-align: left;
    transition: background-color 0.1s ease;
}

.context-menu-item:hover {
    background-color: var(--bg-hover);
}

.context-menu-item.danger {
    color: var(--danger);
}

.context-menu-item.danger:hover {
    background-color: rgba(239, 68, 68, 0.1);
    color: var(--danger-hover);
}
"#;
