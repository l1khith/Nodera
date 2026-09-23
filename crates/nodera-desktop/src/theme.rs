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
@import url('https://fonts.googleapis.com/css2?family=Geist:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500;600;700&display=swap');

:root, .theme-dark {
    --bg-app: #0B0F14;
    --bg-sidebar: #11161D;
    --bg-sidebar-hover: #202533;
    --bg-sidebar-active: #252A3A;
    --bg-surface: #141720;
    --bg-surface-elevated: #161D26;
    --bg-hover: #202533;
    --bg-active: #252A3A;
    --border: #28313C;
    --border-strong: #383F4F;
    --border-subtle: #1E232F;
    --text-primary: #DEE2ED;
    --text-secondary: #C5C5D6;
    --text-muted: #8E90A0;
    --text-disabled: #444654;
    --accent: #6680FF;
    --accent-hover: #7182FF;
    --accent-pressed: #4A55E8;
    --accent-secondary: #9A4BFF;
    --accent-secondary-hover: #AC68FF;
    --accent-focus: rgba(102, 128, 255, 0.25);
    --selection: #252A3A;
    --focus: #6680FF;
    --success: #35B875;
    --success-container: #163527;
    --warning: #E3A93B;
    --warning-container: #392C16;
    --danger: #E45B63;
    --danger-hover: #F06A72;
    --danger-container: #391A1D;
    --info: #4FA3E3;
    --info-container: #172E40;
    --status-success: var(--success);
    --status-warning: var(--warning);
    --status-danger: var(--danger);
    --status-info: var(--info);
    --graph-node: #6680FF;
    --graph-node-current: #9A4BFF;
    --graph-node-selected: #9A4BFF;
    --graph-node-hover: #7182FF;
    --graph-node-connected: #7182FF;
    --graph-node-unrelated: #444A5B;
    --graph-edge: #444A5B;
    --graph-edge-highlight: #7182FF;
    --graph-label: #DEE2ED;
    --graph-grid-dot: rgba(255, 255, 255, 0.08);
    --shadow-none: none;
    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.2);
    --shadow-md: 0 4px 16px rgba(0, 0, 0, 0.28);
    --shadow-lg: 0 16px 40px rgba(0, 0, 0, 0.45);
    --font-ui: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-editor: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-mono: "JetBrains Mono", "Cascadia Code", "Fira Code", Consolas, monospace;
    --border-color: var(--border);
    --bg-primary: var(--bg-app);
    --bg-secondary: var(--bg-surface);
    --bg-tertiary: var(--bg-surface-elevated);
    --accent-color: var(--accent);
    --scrollbar-thumb: rgba(255, 255, 255, 0.09);
    --scrollbar-thumb-hover: rgba(255, 255, 255, 0.22);
    --scrollbar-thumb-active: rgba(255, 255, 255, 0.36);
}

.theme-light {
    --bg-app: #F4F6F9;
    --bg-sidebar: #ECEFF3;
    --bg-sidebar-hover: #E2E6EC;
    --bg-sidebar-active: #D5DBE4;
    --bg-surface: #FFFFFF;
    --bg-surface-elevated: #F8FAFC;
    --bg-hover: #EEF2F6;
    --bg-active: #E4E9F2;
    --border: #D1D7E0;
    --border-strong: #A8B2C0;
    --border-subtle: #E6EAF0;
    --text-primary: #171C23;
    --text-secondary: #475060;
    --text-muted: #6B7687;
    --text-disabled: #98A2B3;
    --accent: #4A63E8;
    --accent-hover: #3B53D8;
    --accent-pressed: #2D41B8;
    --accent-secondary: #7E3FE0;
    --accent-secondary-hover: #9253F0;
    --accent-focus: rgba(74, 99, 232, 0.2);
    --selection: #DDE4FF;
    --focus: #4A63E8;
    --success: #20864E;
    --success-container: #E6F6ED;
    --warning: #A87116;
    --warning-container: #FDF3E1;
    --danger: #C83B47;
    --danger-hover: #AF2F3B;
    --danger-container: #FCE8EA;
    --info: #2B78B8;
    --info-container: #E7F3FC;
    --status-success: var(--success);
    --status-warning: var(--warning);
    --status-danger: var(--danger);
    --status-info: var(--info);
    --graph-node: #4A63E8;
    --graph-node-current: #7E3FE0;
    --graph-node-selected: #7E3FE0;
    --graph-node-hover: #3B53D8;
    --graph-node-connected: #3B53D8;
    --graph-node-unrelated: #98A2B3;
    --graph-edge: #D1D7E0;
    --graph-edge-highlight: #4A63E8;
    --graph-label: #171C23;
    --graph-grid-dot: rgba(23, 28, 35, 0.08);
    --shadow-none: none;
    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.08);
    --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.12);
    --shadow-lg: 0 16px 36px rgba(0, 0, 0, 0.16);
    --font-ui: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-editor: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-mono: "JetBrains Mono", "Cascadia Code", "Fira Code", Consolas, monospace;
    --border-color: var(--border);
    --bg-primary: var(--bg-app);
    --bg-secondary: var(--bg-surface);
    --bg-tertiary: var(--bg-surface-elevated);
    --accent-color: var(--accent);
    --scrollbar-thumb: rgba(0, 0, 0, 0.11);
    --scrollbar-thumb-hover: rgba(0, 0, 0, 0.24);
    --scrollbar-thumb-active: rgba(0, 0, 0, 0.38);
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

:focus-visible {
    outline: 1px solid var(--focus);
    box-shadow: 0 0 0 2px var(--accent-focus);
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

/* ==========================================================================
   Editor & Document Scrollbar Styling (Minimal, native-feel document scrollbar)
   ========================================================================== */

textarea.editor-textarea,
.editor-textarea,
.reading-view,
.reading-toc {
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) transparent;
    scrollbar-gutter: stable;
    overflow-x: hidden !important;
}

textarea.editor-textarea,
.editor-textarea {
    overflow-y: auto;
    word-wrap: break-word;
    overflow-wrap: break-word;
}

textarea.editor-textarea:hover,
.editor-textarea:hover,
.reading-view:hover,
.reading-toc:hover {
    scrollbar-color: var(--scrollbar-thumb-hover) transparent;
}

/* WebKit / Chromium (Edge WebView2) Custom Scrollbar */
textarea.editor-textarea::-webkit-scrollbar,
.editor-textarea::-webkit-scrollbar,
.reading-view::-webkit-scrollbar,
.reading-toc::-webkit-scrollbar {
    width: 6px;
    height: 6px;
}

textarea.editor-textarea::-webkit-scrollbar-track,
.editor-textarea::-webkit-scrollbar-track,
.reading-view::-webkit-scrollbar-track,
.reading-toc::-webkit-scrollbar-track {
    background: transparent !important;
    border: none !important;
}

textarea.editor-textarea::-webkit-scrollbar-thumb,
.editor-textarea::-webkit-scrollbar-thumb,
.reading-view::-webkit-scrollbar-thumb,
.reading-toc::-webkit-scrollbar-thumb {
    background-color: var(--scrollbar-thumb);
    border-radius: 4px;
    transition: background-color 0.15s ease;
}

textarea.editor-textarea:hover::-webkit-scrollbar-thumb,
.editor-textarea:hover::-webkit-scrollbar-thumb,
.reading-view:hover::-webkit-scrollbar-thumb,
.reading-toc:hover::-webkit-scrollbar-thumb,
textarea.editor-textarea:focus::-webkit-scrollbar-thumb,
.editor-textarea:focus::-webkit-scrollbar-thumb {
    background-color: var(--scrollbar-thumb-hover);
}

textarea.editor-textarea::-webkit-scrollbar-thumb:hover,
.editor-textarea::-webkit-scrollbar-thumb:hover,
.reading-view::-webkit-scrollbar-thumb:hover,
.reading-toc::-webkit-scrollbar-thumb:hover {
    background-color: var(--scrollbar-thumb-active);
}

textarea.editor-textarea::-webkit-scrollbar-button,
.editor-textarea::-webkit-scrollbar-button,
.reading-view::-webkit-scrollbar-button,
.reading-toc::-webkit-scrollbar-button {
    display: none !important;
    width: 0 !important;
    height: 0 !important;
}

textarea.editor-textarea::-webkit-scrollbar-corner,
.editor-textarea::-webkit-scrollbar-corner,
.reading-view::-webkit-scrollbar-corner,
.reading-toc::-webkit-scrollbar-corner {
    background: transparent !important;
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
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: 6px;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    font-size: 12px;
    color: var(--text-primary);
    transition: background-color 0.12s ease, border-color 0.12s ease;
}

.btn-action:hover {
    background-color: var(--bg-hover);
    border-color: var(--accent);
}

.btn-primary {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    background-color: var(--accent);
    color: #ffffff;
    font-weight: 600;
    font-size: 13px;
    border: 1px solid var(--accent);
    border-radius: 6px;
    height: 34px;
    padding: 6px 14px;
    transition: background-color 0.12s ease, border-color 0.12s ease;
}

.btn-primary:hover {
    background-color: var(--accent-hover);
    border-color: var(--accent-hover);
}

.btn-primary:active {
    background-color: var(--accent-pressed);
    border-color: var(--accent-pressed);
}

.btn-danger {
    color: var(--danger);
    background: transparent;
    border-radius: 6px;
    transition: background-color 0.12s ease, color 0.12s ease;
}

.btn-danger:hover {
    background-color: var(--danger-container);
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
    border: 1px solid var(--border-strong);
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
    padding: 6px 10px;
    border-radius: 6px;
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
.reading-view {
    font-family: var(--font-editor);
    font-size: 15px;
    line-height: 26px;
    color: var(--text-primary);
}

.editor-textarea {
    font-family: var(--font-editor);
    font-size: 15px;
    font-weight: 400;
    line-height: 26px;
    letter-spacing: 0;
    color: var(--text-primary);
    background: transparent;
    border: none;
    outline: none;
    resize: none;
}

.reading-view h1 { font-family: var(--font-ui); font-size: 24px; font-weight: 700; line-height: 30px; letter-spacing: -0.01em; margin: 24px 0 12px; border-bottom: 1px solid var(--border); padding-bottom: 8px; color: var(--text-primary); }
.reading-view h2 { font-family: var(--font-ui); font-size: 20px; font-weight: 600; line-height: 26px; letter-spacing: -0.005em; margin: 20px 0 10px; border-bottom: 1px solid var(--border-subtle); padding-bottom: 6px; color: var(--text-primary); }
.reading-view h3 { font-family: var(--font-ui); font-size: 17px; font-weight: 600; line-height: 23px; letter-spacing: 0; margin: 16px 0 8px; color: var(--text-primary); }
.reading-view h4, .reading-view h5, .reading-view h6 { font-family: var(--font-ui); font-size: 14px; font-weight: 600; line-height: 22px; margin: 12px 0 6px; color: var(--text-primary); }
.reading-view p { font-family: var(--font-editor); font-size: 15px; font-weight: 400; line-height: 26px; letter-spacing: 0; margin: 12px 0; }
.reading-view ul, .reading-view ol { margin: 10px 0 10px 24px; line-height: 24px; }
.reading-view li { margin: 4px 0; }
.reading-view blockquote { border-left: 3px solid var(--accent); padding: 6px 16px; margin: 12px 0; color: var(--text-secondary); background: var(--bg-surface); border-radius: 0 4px 4px 0; }
.reading-view code { background: var(--bg-surface-elevated); padding: 2px 6px; border-radius: 3px; font-family: var(--font-mono); font-size: 13px; line-height: 20px; }
.reading-view pre { background: var(--bg-surface-elevated); padding: 12px; border-radius: 6px; overflow-x: auto; margin: 14px 0; border: 1px solid var(--border); font-family: var(--font-mono); font-size: 13px; line-height: 20px; }
.reading-view pre code { background: none; padding: 0; font-family: var(--font-mono); }
.reading-view table { border-collapse: collapse; width: 100%; margin: 16px 0; }
.reading-view th, .reading-view td { border: 1px solid var(--border); padding: 8px 12px; text-align: left; }
.reading-view th { background: var(--bg-surface); font-weight: 600; }
.reading-view a { color: var(--accent); text-decoration: none; }
.reading-view a:hover { text-decoration: underline; color: var(--accent-hover); }
.reading-view a.wikilink { color: var(--accent-secondary); font-weight: 500; border-bottom: 1px dashed var(--accent-secondary); padding-bottom: 1px; transition: color 0.12s ease, background-color 0.12s ease; }
.reading-view a.wikilink:hover { color: var(--accent-secondary-hover); background-color: rgba(154, 75, 255, 0.15); border-radius: 2px; }

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

.callout-note { border-left-color: #4FA3E3; }
.callout-note .callout-title { color: #4FA3E3; }

.callout-tip { border-left-color: #35B875; }
.callout-tip .callout-title { color: #35B875; }

.callout-warning { border-left-color: #E3A93B; }
.callout-warning .callout-title { color: #E3A93B; }

.callout-important { border-left-color: #9A4BFF; }
.callout-important .callout-title { color: #9A4BFF; }

.callout-caution { border-left-color: #E45B63; }
.callout-caution .callout-title { color: #E45B63; }

.callout-success { border-left-color: #35B875; }
.callout-success .callout-title { color: #35B875; }

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
.tag-badge { display: inline-block; background-color: #191D27; color: var(--accent-hover); border: 1px solid var(--border); border-radius: 4px; padding: 2px 8px; font-family: var(--font-mono); font-size: 11px; font-weight: 500; margin: 2px 4px 2px 0; }
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

.graph-node-circle {
    transition: r 0.18s cubic-bezier(0.16, 1, 0.3, 1),
                opacity 0.18s ease-out,
                stroke 0.18s ease-out,
                stroke-width 0.18s ease-out;
}

.graph-edge-line {
    transition: opacity 0.18s ease-out,
                stroke 0.18s ease-out,
                stroke-width 0.18s ease-out;
}

.graph-node-label {
    user-select: none;
    pointer-events: none;
    transition: opacity 0.18s ease-out, fill 0.18s ease-out;
}

.graph-halo-ring {
    pointer-events: none;
    transition: r 0.18s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.18s ease-out;
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
    font-family: var(--font-mono);
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
    box-shadow: var(--shadow-md);
    transition: transform 0.15s ease, border-color 0.15s ease, box-shadow 0.15s ease;
}

.library-card:hover {
    transform: translateY(-2px);
    border-color: var(--accent);
    box-shadow: var(--shadow-lg);
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
    font-family: var(--font-mono);
    padding: 3px 8px;
    border-radius: 4px;
    background-color: #191D27;
    border: 1px solid var(--border);
    color: var(--accent-hover);
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
    font-family: var(--font-mono);
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
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 20px;
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
    color: #747B8C;
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 16px;
    letter-spacing: 0.01em;
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
    background-color: #191D27;
    border: 1px solid var(--border);
    color: var(--accent-hover);
    border-radius: 4px;
    padding: 2px 8px;
    font-family: var(--font-mono);
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
    font-family: var(--font-mono);
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

/* Calendar / Today Workspace */
.calendar-day-cell {
    user-select: none;
}

.calendar-day-cell:hover {
    background-color: var(--bg-hover) !important;
}

.calendar-day-cell:focus-visible {
    outline: 2px solid var(--focus) !important;
    outline-offset: 1px;
}
"#;
