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
    --bg-app: #0D0F14;
    --bg-sidebar: #11141B;
    --bg-sidebar-hover: #191D27;
    --bg-sidebar-active: #252A3A;
    --bg-surface: #141720;
    --bg-surface-elevated: #1A1E29;
    --bg-hover: #202533;
    --bg-active: #282D40;
    --border: #292E3A;
    --border-strong: #383F4F;
    --border-subtle: #1E232F;
    --text-primary: #F1F3F8;
    --text-secondary: #A7ADBC;
    --text-muted: #747B8C;
    --text-disabled: #525866;
    --accent: #5B6CFF;
    --accent-hover: #7182FF;
    --accent-pressed: #4A55E8;
    --accent-secondary: #9A4BFF;
    --accent-focus: rgba(91, 108, 255, 0.35);
    --danger: #E45B63;
    --danger-hover: #F06A72;
    --warning: #E3A93B;
    --success: #35B875;
    --info: #4FA3E3;
    --graph-node: #5B6CFF;
    --graph-node-current: #9A4BFF;
    --graph-node-hover: #7182FF;
    --graph-node-connected: #7E8CFF;
    --graph-edge: #444A5B;
    --graph-edge-highlight: #7182FF;
    --graph-label: #D7DBE6;
    --graph-grid-dot: rgba(255, 255, 255, 0.08);
    --font-ui: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    --font-editor: "Cascadia Code", "Fira Code", Consolas, "Courier New", monospace;
    --border-color: var(--border);
    --bg-primary: var(--bg-app);
    --bg-secondary: var(--bg-surface);
    --bg-tertiary: var(--bg-surface-elevated);
    --accent-color: var(--accent);
}

.theme-light {
    --bg-app: #F7F8FC;
    --bg-sidebar: #F1F3F8;
    --bg-sidebar-hover: #E8EBF3;
    --bg-sidebar-active: #DFE3F0;
    --bg-surface: #FFFFFF;
    --bg-surface-elevated: #FFFFFF;
    --bg-hover: #F0F2F8;
    --bg-active: #E8EBF5;
    --border: #DEE2EC;
    --border-strong: #C8CDD9;
    --border-subtle: #EBEFF7;
    --text-primary: #171A23;
    --text-secondary: #565D6D;
    --text-muted: #7A8190;
    --text-disabled: #A7ACB7;
    --accent: #4F5FEF;
    --accent-hover: #4050D8;
    --accent-pressed: #3544B8;
    --accent-secondary: #8744E8;
    --accent-focus: rgba(79, 95, 239, 0.3);
    --danger: #E45B63;
    --danger-hover: #D04850;
    --warning: #E3A93B;
    --success: #35B875;
    --info: #4FA3E3;
    --graph-node: #4F5FEF;
    --graph-node-current: #8744E8;
    --graph-node-hover: #7182FF;
    --graph-node-connected: #7E8CFF;
    --graph-edge: #C8CDD9;
    --graph-edge-highlight: #4F5FEF;
    --graph-label: #565D6D;
    --graph-grid-dot: rgba(23, 26, 35, 0.07);
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
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
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
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.25);
    z-index: 10;
    backdrop-filter: blur(10px);
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
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
    z-index: 10;
    backdrop-filter: blur(10px);
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
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.25);
    z-index: 10;
    min-width: 200px;
    backdrop-filter: blur(10px);
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
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.28);
    transition: transform 0.18s ease, box-shadow 0.18s ease, border-color 0.18s ease;
}

.library-card:hover {
    transform: translateY(-3px);
    border-color: var(--accent);
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.45);
}

.library-card-header {
    padding: 18px;
    background: linear-gradient(135deg, var(--bg-surface-elevated) 0%, var(--bg-surface) 100%);
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    align-items: flex-start;
    gap: 14px;
}

.library-card-cover {
    width: 48px;
    height: 64px;
    border-radius: 6px;
    background: linear-gradient(135deg, var(--accent) 0%, var(--accent-secondary) 100%);
    color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    box-shadow: inset 3px 0 0 rgba(0, 0, 0, 0.2), 0 4px 12px rgba(91, 108, 255, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.18);
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
    background-color: rgba(91, 108, 255, 0.12);
    border: 1px solid rgba(91, 108, 255, 0.28);
    color: var(--accent);
}

.library-progress-track {
    height: 8px;
    border-radius: 4px;
    background-color: var(--bg-hover);
    border: 1px solid var(--border);
    overflow: hidden;
}

.library-progress-bar {
    height: 100%;
    background: linear-gradient(90deg, var(--accent), var(--accent-secondary));
    border-radius: 3px;
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
"#;
