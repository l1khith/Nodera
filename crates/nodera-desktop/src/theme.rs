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
    --bg-app: #121316;
    --bg-sidebar: #18191d;
    --bg-surface: #1e2025;
    --bg-surface-elevated: #262930;
    --bg-hover: #2b2e36;
    --bg-active: #323640;
    --border: #2c2f38;
    --border-subtle: #22252c;
    --text-primary: #f0f2f5;
    --text-secondary: #9da3b4;
    --text-muted: #646a78;
    --accent: #5e81ac;
    --accent-hover: #81a1c1;
    --accent-focus: rgba(94, 129, 172, 0.4);
    --danger: #bf616a;
    --danger-hover: #d08770;
    --success: #a3be8c;
    --font-ui: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    --font-editor: "Cascadia Code", "Fira Code", Consolas, "Courier New", monospace;
}

.theme-light {
    --bg-app: #fcfcfc;
    --bg-sidebar: #f4f5f7;
    --bg-surface: #ffffff;
    --bg-surface-elevated: #ebeef2;
    --bg-hover: #e5e9f0;
    --bg-active: #d8dee9;
    --border: #d8dee9;
    --border-subtle: #e5e9f0;
    --text-primary: #2e3440;
    --text-secondary: #4c566a;
    --text-muted: #7e889b;
    --accent: #4c6a92;
    --accent-hover: #5e81ac;
    --accent-focus: rgba(76, 106, 146, 0.3);
    --danger: #bf616a;
    --danger-hover: #d08770;
    --success: #8fbcbb;
    --font-ui: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    --font-editor: "Cascadia Code", "Fira Code", Consolas, "Courier New", monospace;
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
    width: 260px;
    min-width: 180px;
    background-color: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transition: width 0.15s ease;
}

.pane-center {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-app);
    overflow: hidden;
}

.pane-context {
    width: 260px;
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

/* Context Panel & Knowledge Components */
.tag-badge { display: inline-block; background-color: var(--bg-surface-elevated); color: var(--accent-hover); border: 1px solid var(--border); border-radius: 12px; padding: 2px 8px; font-size: 11px; font-weight: 500; margin: 2px 4px 2px 0; }
.link-list { display: flex; flex-direction: column; gap: 4px; margin-top: 6px; }
.link-item { display: flex; align-items: center; justify-content: space-between; padding: 5px 8px; border-radius: 4px; font-size: 12px; cursor: pointer; color: var(--text-primary); background-color: var(--bg-surface); border: 1px solid var(--border-subtle); transition: background-color 0.1s, border-color 0.1s; text-align: left; width: 100%; }
.link-item:hover { background-color: var(--bg-hover); border-color: var(--accent); }
.link-item-unresolved { color: var(--text-muted); font-style: italic; }
"#;
