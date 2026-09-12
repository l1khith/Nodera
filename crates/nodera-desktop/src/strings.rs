//! Centralized user-facing strings, labels, tooltips, dialogs, and messages for Nodera.

pub mod app {
    pub const BRAND_TITLE: &str = "Nodera";
    pub const WINDOW_TITLE: &str = "Nodera — Knowledge Workspace";
    pub const WORKSPACE: &str = "Workspace";
    pub const CONTEXT_PANEL_TITLE: &str = "Context & Links";
    pub const NOTE_PROPERTIES: &str = "Note Properties";
    pub const INCOMING_LINKS: &str = "Incoming Links";
    pub const OUTGOING_LINKS: &str = "Outgoing Links";
    pub const TAGS: &str = "Tags";
    pub const CONTENTS: &str = "Contents";
}

pub mod nav {
    pub const NOTES: &str = "Notes";
    pub const TASKS: &str = "Tasks";
    pub const LIBRARY: &str = "Library";
    pub const GRAPH: &str = "Graph";
    pub const GLOBAL_TASKS: &str = "Global Tasks";
}

pub mod actions {
    pub const NEW_NOTE: &str = "New Note";
    pub const NOTE_BTN: &str = "Note";
    pub const CREATE_NOTE: &str = "Create Note";
    pub const NEW_VAULT: &str = "New Vault";
    pub const OPEN_VAULT: &str = "Open Vault";
    pub const OPEN_EXISTING_VAULT: &str = "Open Existing Vault";
    pub const OPEN_GRAPH: &str = "Open Graph View";
    pub const ZOOM_IN: &str = "Zoom In";
    pub const ZOOM_OUT: &str = "Zoom Out";
    pub const RESET_VIEW: &str = "Reset View";
    pub const REFRESH_GRAPH: &str = "Re-layout Graph";
    pub const IMPORT_PDF: &str = "Import PDF as Markdown";
    pub const IMPORT_PDF_BTN: &str = "Import PDF";
    pub const SAVE: &str = "Save";
    pub const SAVED: &str = "Saved";
    pub const UNSAVED: &str = "Unsaved";
    pub const EDIT_MODE: &str = "Edit Mode";
    pub const READING_MODE: &str = "Reading Mode";
    pub const REBUILD_INDEX: &str = "Rebuild Search Index";
    pub const SETTINGS: &str = "Settings";
    pub const TOGGLE_THEME: &str = "Toggle Theme";
    pub const TOGGLE_SIDEBAR: &str = "Toggle Sidebar";
    pub const TOGGLE_CONTEXT: &str = "Toggle Context Panel";
    pub const PALETTE: &str = "Palette";
    pub const SEARCH: &str = "Search";
    pub const CLEAR: &str = "Clear";
    pub const CLEAR_SEARCH: &str = "Clear Search";
    pub const CANCEL: &str = "Cancel";
    pub const CLOSE: &str = "Close";
    pub const DELETE: &str = "Delete";
    pub const DELETE_PERMANENTLY: &str = "Delete Permanently";
    pub const RENAME: &str = "Rename";
    pub const OPEN_NOTE: &str = "Open Markdown Note";
    pub const READ: &str = "Read";
    pub const EDIT: &str = "Edit";
    pub const CHANGE: &str = "Change";
    pub const CHOOSE_PDF: &str = "Choose PDF File...";
    pub const CONVERT_TO_MD: &str = "Convert to Markdown";
    pub const CANCEL_CONVERSION: &str = "Cancel Conversion";
    pub const RESET_LAYOUT: &str = "Reset Pane Layout";
    pub const FILTER_ALL: &str = "All";
    pub const FILTER_TODO: &str = "To Do";
    pub const FILTER_DONE: &str = "Done";
    pub const DISMISS: &str = "Dismiss";
    pub const DONE: &str = "Done";
    pub const REBUILD_INDEX_NOW: &str = "Rebuild Index Now";
    pub const HIDE_DETAILS: &str = "Hide Details";
    pub const VIEW_DETAILS: &str = "View Technical Details";
}

pub mod tooltips {
    pub const NEW_NOTE: &str = "New Note (Ctrl+N)";
    pub const SAVE_NOTE: &str = "Save Note (Ctrl+S)";
    pub const TOGGLE_READING: &str = "Toggle Reading Mode (Ctrl+E)";
    pub const PALETTE: &str = "Search & Command Palette (Ctrl+P)";
    pub const SETTINGS: &str = "Settings (Ctrl+,)";
    pub const IMPORT_PDF: &str = "Import PDF as Markdown (Ctrl+Shift+I)";
    pub const REBUILD_INDEX: &str = "Rebuild Search Index";
    pub const TOGGLE_THEME: &str = "Toggle Theme";
    pub const TOGGLE_SIDEBAR: &str = "Toggle Sidebar";
    pub const TOGGLE_CONTEXT: &str = "Toggle Context Panel";
    pub const RESET_SIDEBAR: &str = "Double click to reset sidebar width";
    pub const RESET_CONTEXT: &str = "Double click to reset context width";
    pub const CLOSE_ESC: &str = "Close (Esc)";
    pub const RENAME_NOTE: &str = "Rename";
    pub const DELETE_NOTE: &str = "Delete";
}

pub mod placeholders {
    pub const SEARCH_VAULT: &str = "Search vault...";
    pub const SEARCH_PALETTE: &str = "Type a command or search notes...";
    pub const SEARCH_LIBRARY: &str = "Search library...";
    pub const FILTER_TASKS: &str = "Filter tasks...";
    pub const FILTER_GRAPH: &str = "Filter graph notes...";
    pub const TYPE_MARKDOWN: &str = "Start typing Markdown here...";
}

pub mod empty_states {
    pub const NO_VAULT_OPEN: &str = "No vault open";
    pub const OPEN_VAULT_FOLDER: &str = "Open Vault Folder";
    pub const EMPTY_VAULT_TITLE: &str = "Vault is empty";
    pub const EMPTY_VAULT_DESC: &str = "Create your first note or import a PDF to get started.";
    pub const NO_NOTE_SELECTED_TITLE: &str = "No Note Selected";
    pub const NO_NOTE_SELECTED_DESC: &str =
        "Select a note from the explorer on the left, or create a new note to begin writing.";
    pub const NO_SEARCH_RESULTS_TITLE: &str = "No matching notes found";
    pub const NO_SEARCH_RESULTS_DESC: &str = "Try broadening your query or checking note tags.";
    pub const NO_TASKS_TITLE: &str = "No tasks found";
    pub const NO_TASKS_DESC: &str =
        "Tasks written as '- [ ]' in your notes will automatically appear here.";
    pub const NO_BOOKS_TITLE: &str = "No books or documents found";
    pub const NO_BOOKS_DESC: &str =
        "Import a PDF or add Markdown documents to the Books/ folder to build your personal library.";
    pub const NO_BOOKS_SEARCH_TITLE: &str = "No documents matching search";
    pub const NO_BOOKS_SEARCH_DESC: &str =
        "Try broadening your search term or clearing the filter.";
    pub const NO_BACKLINKS: &str = "This note is not referenced by other notes yet.";
    pub const NO_OUTGOING_LINKS: &str = "No outgoing links in note.";
    pub const NO_PALETTE_MATCHES: &str = "No matching commands or notes found.";
    pub const NO_FILE_OPEN: &str = "No file open";
    pub const NO_GRAPH_NODES_TITLE: &str = "No Notes in Vault";
    pub const NO_GRAPH_NODES_DESC: &str =
        "Create notes with [[wikilinks]] to visualize your knowledge graph.";
    pub const NO_LOCAL_GRAPH: &str = "No links connected to this note.";
    pub const NO_LOCAL_GRAPH_DESC: &str =
        "Link notes using [[wikilinks]] to see this note's local graph.";
}

pub mod dialogs {
    pub const CREATE_NOTE_TITLE: &str = "Create New Note";
    pub const CREATE_NOTE_PROMPT: &str = "Enter a title for the new Markdown note:";
    pub const DEFAULT_NOTE_TITLE: &str = "Untitled Note";
    pub const RENAME_NOTE_TITLE: &str = "Rename Note";
    pub const DELETE_NOTE_TITLE: &str = "Delete Note";
    pub const DELETE_NOTE_CONFIRM: &str = "Are you sure you want to delete ";
    pub const DELETE_NOTE_WARNING: &str =
        "? This will permanently remove the Markdown file from disk.";
    pub const PDF_IMPORT_TITLE: &str = "Import PDF as Markdown";
    pub const PDF_CONVERTING_TITLE: &str = "Converting PDF...";
    pub const PDF_COMPLETE_TITLE: &str = "Conversion Complete";
    pub const PDF_SELECT_PROMPT: &str = "Select a text-based PDF to import as Markdown";
    pub const PDF_OPTIONS_TITLE: &str = "Conversion Options";
    pub const PDF_OPT_HEADINGS: &str = "Detect chapter & section headings";
    pub const PDF_OPT_HEADERS: &str = "Remove repeated running headers & footers";
    pub const PDF_OPT_PAGE_NUMBERS: &str = "Remove standalone page numbers";
    pub const PDF_OPT_PAGE_MARKERS: &str = "Insert page comment markers (<!-- nodera:page=N -->)";
    pub const PDF_NOTE_CREATED: &str = "Note Created";
}

pub mod settings {
    pub const TITLE: &str = "Settings";
    pub const TAB_GENERAL: &str = "General";
    pub const TAB_APPEARANCE: &str = "Appearance";
    pub const TAB_EDITOR: &str = "Editor";
    pub const TAB_PDF: &str = "PDF Import";
    pub const TAB_INDEX: &str = "Index & Search";
    pub const TAB_SHORTCUTS: &str = "Shortcuts";
}

pub mod palette {
    pub const SWITCH_TO_EDITOR: (&str, &str) =
        ("Switch to Notes Editor", "View and edit markdown notes");
    pub const SWITCH_TO_TASKS: (&str, &str) = (
        "Switch to Tasks View",
        "Global task list across all vault notes",
    );
    pub const SWITCH_TO_LIBRARY: (&str, &str) = (
        "Switch to Library View",
        "Browse books and imported PDF documents",
    );
    pub const SWITCH_TO_GRAPH: (&str, &str) = (
        "Switch to Graph View",
        "Interactive 2D visual knowledge graph of interconnected notes",
    );
    pub const TOGGLE_READING: (&str, &str) = (
        "Toggle Edit / Reading Mode",
        "Switch between markdown editing and distraction-free reading (Ctrl+E)",
    );
    pub const CREATE_NOTE: (&str, &str) = (
        "Create New Note",
        "Create a new note in default folder (Ctrl+N)",
    );
    pub const IMPORT_PDF: (&str, &str) = (
        "Import PDF as Markdown",
        "Convert PDF document to Markdown (Ctrl+Shift+I)",
    );
    pub const OPEN_SETTINGS: (&str, &str) = (
        "Open Settings",
        "Configure appearance, editor, and system preferences (Ctrl+,)",
    );
    pub const RESET_LAYOUT: (&str, &str) = (
        "Reset Pane Layout",
        "Restore default sidebar and context panel widths",
    );
    pub const TOGGLE_THEME: (&str, &str) = ("Toggle Theme", "Switch between dark and light themes");
    pub const REBUILD_INDEX: (&str, &str) = (
        "Rebuild Search Index",
        "Clean and recreate SQLite metadata and Tantivy FTS",
    );
}
