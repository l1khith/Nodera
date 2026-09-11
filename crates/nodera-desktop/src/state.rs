use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info};

use nodera_core::{Note, Result, Vault, VaultEntry, VaultService};
use nodera_markdown::{parse_document, LinkGraph};

use crate::theme::Theme;

/// Persistent local state remembered across application restarts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppPreferences {
    pub last_vault: Option<PathBuf>,
    pub recent_vaults: Vec<PathBuf>,
    pub theme: Theme,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            last_vault: None,
            recent_vaults: Vec::new(),
            theme: Theme::Dark,
        }
    }
}

impl AppPreferences {
    /// Loads saved preferences from disk, or returns default if not found or corrupted.
    pub fn load() -> Self {
        let path = Self::preferences_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<Self>(&content) {
                    Ok(prefs) => return prefs,
                    Err(e) => error!("Failed to parse preferences: {e}"),
                },
                Err(e) => error!("Failed to read preferences: {e}"),
            }
        }
        Self::default()
    }

    /// Saves current preferences to disk.
    pub fn save(&self) {
        let path = Self::preferences_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json);
        }
    }

    pub fn record_vault(&mut self, vault_path: PathBuf) {
        self.last_vault = Some(vault_path.clone());
        self.recent_vaults.retain(|p| p != &vault_path);
        self.recent_vaults.insert(0, vault_path);
        if self.recent_vaults.len() > 10 {
            self.recent_vaults.truncate(10);
        }
        self.save();
    }

    fn preferences_path() -> PathBuf {
        let base_dir = std::env::var_os("APPDATA")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        base_dir.join(".nodera").join("desktop_preferences.json")
    }
}

use nodera_index::{IndexedTask, SearchResult, TaskFilter, VaultIndex};
use nodera_pdf::{CancellationToken, ConversionOptions, ConversionProgress, ImportResult};
use std::sync::{Arc, Mutex};

/// Primary workspace view mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveView {
    #[default]
    Editor,
    Tasks,
}

/// Action item displayed inside the Command Palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteItem {
    pub title: String,
    pub description: String,
    pub action: PaletteAction,
}

/// Executable command triggered by Command Palette selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteAction {
    OpenNote(PathBuf),
    CreateNote,
    ImportPdf,
    SwitchView(ActiveView),
    ToggleTheme,
    RebuildIndex,
}

/// Runtime application state driving the UI.
#[derive(Debug, Clone)]
pub struct AppState {
    pub vault_service: Option<VaultService>,
    pub vault_path: Option<PathBuf>,
    pub vault_name: String,
    pub entries: Vec<VaultEntry>,

    pub active_note: Option<Note>,
    pub editor_content: String,
    pub is_dirty: bool,
    pub is_reading_mode: bool,

    pub active_view: ActiveView,
    pub vault_index: Option<Arc<Mutex<VaultIndex>>>,
    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    pub task_filter: TaskFilter,

    pub theme: Theme,
    pub sidebar_open: bool,
    pub context_panel_open: bool,
    pub status_message: String,

    pub preferences: AppPreferences,
    pub link_graph: LinkGraph,

    // Modal dialogs
    pub show_new_note_dialog: bool,
    pub show_delete_confirm_dialog: bool,
    pub note_to_delete: Option<PathBuf>,
    pub show_rename_dialog: bool,
    pub note_to_rename: Option<PathBuf>,
    pub show_command_palette: bool,
    pub command_palette_query: String,

    // PDF Import modal
    pub show_pdf_import_modal: bool,
    pub pdf_selected_path: Option<PathBuf>,
    pub pdf_import_options: ConversionOptions,
    pub pdf_progress: Option<ConversionProgress>,
    pub pdf_cancellation: Option<CancellationToken>,
    pub pdf_last_result: Option<ImportResult>,
    pub pdf_error: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        let prefs = AppPreferences::load();
        let initial_theme = prefs.theme;

        Self {
            vault_service: None,
            vault_path: None,
            vault_name: "No Vault Opened".to_string(),
            entries: Vec::new(),

            active_note: None,
            editor_content: String::new(),
            is_dirty: false,
            is_reading_mode: false,

            active_view: ActiveView::Editor,
            vault_index: None,
            search_query: String::new(),
            search_results: Vec::new(),
            task_filter: TaskFilter::default(),

            theme: initial_theme,
            sidebar_open: true,
            context_panel_open: false,
            status_message: "Ready".to_string(),

            preferences: prefs,
            link_graph: LinkGraph::new(),

            show_new_note_dialog: false,
            show_delete_confirm_dialog: false,
            note_to_delete: None,
            show_rename_dialog: false,
            note_to_rename: None,
            show_command_palette: false,
            command_palette_query: String::new(),

            show_pdf_import_modal: false,
            pdf_selected_path: None,
            pdf_import_options: ConversionOptions::default(),
            pdf_progress: None,
            pdf_cancellation: None,
            pdf_last_result: None,
            pdf_error: None,
        }
    }
}

impl AppState {
    /// Opens a vault directory and refreshes its file tree.
    pub fn open_vault(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let p = path.as_ref();
        info!(path = %p.display(), "Opening vault in AppState");

        let vault = Vault::open(p)?;
        let name = vault.config().name.clone();
        let service = VaultService::new(vault);

        let entries = service.list_entries()?;

        // Index note links into LinkGraph
        let mut link_graph = LinkGraph::new();
        for entry in &entries {
            if let VaultEntry::Note(summary) = entry {
                if let Ok(note) = service.read_note(&summary.relative_path) {
                    if let Ok(parsed) = parse_document(&note.content) {
                        link_graph
                            .update_note_links(summary.relative_path.clone(), parsed.wikilinks);
                    }
                }
            }
        }

        // Initialize derived VaultIndex (SQLite + Tantivy)
        let vault_index = match VaultIndex::open(p) {
            Ok(mut idx) => {
                let _ = idx.rebuild(&service);
                Some(Arc::new(Mutex::new(idx)))
            }
            Err(e) => {
                error!("Failed to initialize vault index: {e}");
                None
            }
        };

        self.vault_path = Some(p.to_path_buf());
        self.vault_name = name;
        self.entries = entries;
        self.vault_service = Some(service);
        self.vault_index = vault_index;
        self.link_graph = link_graph;
        self.active_note = None;
        self.editor_content.clear();
        self.is_dirty = false;
        self.status_message = format!("Vault '{}' opened", self.vault_name);

        self.preferences.record_vault(p.to_path_buf());
        Ok(())
    }

    /// Creates a new vault directory and sets it as the active vault.
    pub fn create_vault(&mut self, path: impl AsRef<Path>, name: Option<String>) -> Result<()> {
        let p = path.as_ref();
        info!(path = %p.display(), "Creating vault in AppState");

        let vault = Vault::create(p, name)?;
        let vault_name = vault.config().name.clone();
        let service = VaultService::new(vault);
        let entries = service.list_entries()?;

        let vault_index = match VaultIndex::open(p) {
            Ok(idx) => Some(Arc::new(Mutex::new(idx))),
            Err(e) => {
                error!("Failed to initialize vault index on create: {e}");
                None
            }
        };

        self.vault_path = Some(p.to_path_buf());
        self.vault_name = vault_name;
        self.entries = entries;
        self.vault_service = Some(service);
        self.vault_index = vault_index;
        self.link_graph = LinkGraph::new();
        self.active_note = None;
        self.editor_content.clear();
        self.is_dirty = false;
        self.status_message = format!("Vault '{}' created", self.vault_name);

        self.preferences.record_vault(p.to_path_buf());
        Ok(())
    }

    /// Refreshes the file tree listing from the vault.
    pub fn refresh_entries(&mut self) -> Result<()> {
        if let Some(service) = &self.vault_service {
            self.entries = service.list_entries()?;
        }
        Ok(())
    }

    /// Opens a note into the editor.
    pub fn select_note(&mut self, relative_path: impl AsRef<Path>) -> Result<()> {
        let rel = relative_path.as_ref();
        if let Some(service) = &self.vault_service {
            let note = service.read_note(rel)?;
            self.editor_content = note.content.clone();
            self.active_note = Some(note);
            self.is_dirty = false;
            self.status_message = format!("Opened '{}'", rel.display());
        }
        Ok(())
    }

    /// Saves the current editor content to the active note.
    pub fn save_active_note(&mut self) -> Result<()> {
        if let (Some(service), Some(note)) = (&self.vault_service, &self.active_note) {
            let updated = service.write_note(&note.relative_path, &self.editor_content)?;
            if let Ok(parsed) = parse_document(&self.editor_content) {
                self.link_graph
                    .update_note_links(note.relative_path.clone(), parsed.wikilinks.clone());

                if let Some(index_arc) = &self.vault_index {
                    if let Ok(mut idx) = index_arc.lock() {
                        let _ = idx.index_note(&updated, &parsed);
                    }
                }
            }
            self.active_note = Some(updated);
            self.is_dirty = false;
            self.status_message = "Saved".to_string();
            self.refresh_entries()?;
        }
        Ok(())
    }

    /// Deletes a note from disk and closes it if active.
    pub fn delete_note(&mut self, rel_path: impl AsRef<Path>) -> Result<()> {
        let rel = rel_path.as_ref();
        if let Some(service) = &self.vault_service {
            service.delete_note(rel)?;
            self.link_graph.remove_note(rel);
            if let Some(index_arc) = &self.vault_index {
                if let Ok(mut idx) = index_arc.lock() {
                    let _ = idx.remove_note(rel);
                }
            }
            if let Some(active) = &self.active_note {
                if active.relative_path == rel {
                    self.active_note = None;
                    self.editor_content.clear();
                    self.is_dirty = false;
                }
            }
            self.status_message = format!("Deleted '{}'", rel.display());
            self.refresh_entries()?;
        }
        Ok(())
    }

    /// Executes full-text search across vault using Tantivy index.
    pub fn execute_search(&mut self, query: &str) {
        self.search_query = query.to_string();
        if query.trim().is_empty() {
            self.search_results.clear();
            return;
        }

        if let Some(index_arc) = &self.vault_index {
            if let Ok(idx) = index_arc.lock() {
                self.search_results = idx.search(query, 25).unwrap_or_default();
            }
        }
    }

    /// Queries tasks across the vault according to current filter criteria.
    pub fn get_vault_tasks(&self) -> Vec<IndexedTask> {
        if let Some(index_arc) = &self.vault_index {
            if let Ok(idx) = index_arc.lock() {
                return idx.query_tasks(&self.task_filter).unwrap_or_default();
            }
        }
        Vec::new()
    }

    /// Toggles a task checkbox in any note file and syncs both source Markdown and SQLite/Tantivy index.
    pub fn toggle_task_and_sync(&mut self, note_rel_path: &Path, line_number: usize) -> Result<()> {
        if let Some(service) = &self.vault_service {
            let note = service.read_note(note_rel_path)?;
            let toggled_content = nodera_markdown::toggle_task_at_line(&note.content, line_number)?;
            let updated_note = service.write_note(note_rel_path, &toggled_content)?;

            // If this note is currently open in the active editor, sync editor content too!
            if let Some(active) = &self.active_note {
                if active.relative_path == note_rel_path {
                    self.editor_content = toggled_content.clone();
                    self.active_note = Some(updated_note.clone());
                    self.is_dirty = false;
                }
            }

            // Update derived index
            if let Some(index_arc) = &self.vault_index {
                if let Ok(mut idx) = index_arc.lock() {
                    if let Ok(parsed) = parse_document(&toggled_content) {
                        let _ = idx.index_note(&updated_note, &parsed);
                    }
                }
            }

            self.refresh_entries()?;
        }
        Ok(())
    }

    /// Rebuilds the entire vault derived index from source Markdown files.
    pub fn rebuild_vault_index(&mut self) -> Result<usize> {
        if let (Some(service), Some(index_arc)) = (&self.vault_service, &self.vault_index) {
            if let Ok(mut idx) = index_arc.lock() {
                let count = idx.rebuild(service)?;
                self.status_message = format!("Index rebuilt ({count} notes)");
                return Ok(count);
            }
        }
        Ok(0)
    }

    /// Returns matching items for the Command Palette based on current query.
    pub fn get_command_palette_items(&self) -> Vec<CommandPaletteItem> {
        let q = self.command_palette_query.trim().to_lowercase();
        let mut items = Vec::new();

        // 1. Note jumping items
        for entry in &self.entries {
            if let VaultEntry::Note(summary) = entry {
                if q.is_empty()
                    || summary.title.to_lowercase().contains(&q)
                    || summary
                        .relative_path
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&q)
                {
                    items.push(CommandPaletteItem {
                        title: summary.title.clone(),
                        description: format!("Note · {}", summary.relative_path.display()),
                        action: PaletteAction::OpenNote(summary.relative_path.clone()),
                    });
                }
            }
        }

        // 2. System Commands
        let system_commands = [
            (
                "Switch to Notes Editor",
                "View and edit markdown notes",
                PaletteAction::SwitchView(ActiveView::Editor),
            ),
            (
                "Switch to Tasks View",
                "Global task list across all vault notes",
                PaletteAction::SwitchView(ActiveView::Tasks),
            ),
            (
                "Create New Note",
                "Create a new note in default folder",
                PaletteAction::CreateNote,
            ),
            (
                "Import PDF as Markdown",
                "Convert PDF document to Markdown (Ctrl+Shift+I)",
                PaletteAction::ImportPdf,
            ),
            (
                "Toggle Theme",
                "Switch between dark and light themes",
                PaletteAction::ToggleTheme,
            ),
            (
                "Rebuild Search Index",
                "Clean and recreate SQLite metadata and Tantivy FTS",
                PaletteAction::RebuildIndex,
            ),
        ];

        for (title, desc, action) in system_commands {
            if q.is_empty() || title.to_lowercase().contains(&q) || desc.to_lowercase().contains(&q)
            {
                items.push(CommandPaletteItem {
                    title: title.to_string(),
                    description: desc.to_string(),
                    action,
                });
            }
        }

        items
    }

    /// Executes a selected command palette action.
    pub fn execute_palette_action(&mut self, action: PaletteAction) -> Result<()> {
        self.show_command_palette = false;
        self.command_palette_query.clear();

        match action {
            PaletteAction::OpenNote(path) => {
                self.active_view = ActiveView::Editor;
                self.select_note(&path)?;
            }
            PaletteAction::CreateNote => {
                self.show_new_note_dialog = true;
            }
            PaletteAction::ImportPdf => {
                self.open_pdf_import_modal();
            }
            PaletteAction::SwitchView(view) => {
                self.active_view = view;
            }
            PaletteAction::ToggleTheme => {
                self.toggle_theme();
            }
            PaletteAction::RebuildIndex => {
                self.rebuild_vault_index()?;
            }
        }
        Ok(())
    }

    /// Returns list of notes that have Wikilinks pointing to the currently active note.
    pub fn get_current_backlinks(&self) -> Vec<PathBuf> {
        if let Some(active) = &self.active_note {
            let note_paths: Vec<PathBuf> = self
                .entries
                .iter()
                .filter_map(|e| match e {
                    VaultEntry::Note(s) => Some(s.relative_path.clone()),
                    _ => None,
                })
                .collect();

            self.link_graph
                .get_backlinks(&active.relative_path, &note_paths)
        } else {
            Vec::new()
        }
    }

    /// Returns list of outgoing Wikilinks in current editor content and whether they resolve to an existing note.
    pub fn get_current_outgoing_links(&self) -> Vec<(nodera_markdown::Wikilink, Option<PathBuf>)> {
        let note_paths: Vec<PathBuf> = self
            .entries
            .iter()
            .filter_map(|e| match e {
                VaultEntry::Note(s) => Some(s.relative_path.clone()),
                _ => None,
            })
            .collect();

        let links = nodera_markdown::extract_wikilinks(&self.editor_content);
        links
            .into_iter()
            .map(|l| {
                let resolved = LinkGraph::resolve_target(&l.target, &note_paths);
                (l, resolved)
            })
            .collect()
    }

    /// Returns extracted tags from current editor content.
    pub fn get_current_note_tags(&self) -> Vec<String> {
        if let Ok(parsed) = parse_document(&self.editor_content) {
            parsed.tags
        } else {
            Vec::new()
        }
    }

    /// Opens an existing note or creates a new one for a Wikilink target.
    pub fn open_or_create_target(&mut self, target: &str) -> Result<()> {
        let note_paths: Vec<PathBuf> = self
            .entries
            .iter()
            .filter_map(|e| match e {
                VaultEntry::Note(s) => Some(s.relative_path.clone()),
                _ => None,
            })
            .collect();

        if let Some(existing) = LinkGraph::resolve_target(target, &note_paths) {
            self.select_note(&existing)?;
        } else {
            self.create_note(target, None)?;
        }
        Ok(())
    }

    /// Toggles a task checkbox at a specific 1-based line number and saves.
    #[allow(dead_code)]
    pub fn toggle_task_at_line(&mut self, line_number: usize) -> Result<()> {
        let updated = nodera_markdown::toggle_task_at_line(&self.editor_content, line_number)?;
        self.editor_content = updated;
        self.is_dirty = true;
        self.save_active_note()?;
        Ok(())
    }

    /// Creates a new note, saves it to disk, and opens it in the editor.
    pub fn create_note(&mut self, title: &str, folder: Option<&str>) -> Result<()> {
        if let Some(service) = &self.vault_service {
            let note = service.create_note(folder, title, Some(""))?;
            self.editor_content.clear();
            let rel = note.relative_path.clone();
            self.active_note = Some(note);
            self.is_dirty = false;
            self.status_message = format!("Created '{title}'");
            self.refresh_entries()?;
            debug!(path = %rel.display(), "Created and opened note in state");
        }
        Ok(())
    }

    /// Renames a note on disk and updates active note if open.
    pub fn rename_note(&mut self, rel_path: impl AsRef<Path>, new_title: &str) -> Result<()> {
        let rel = rel_path.as_ref();
        if let Some(service) = &self.vault_service {
            let renamed = service.rename_note(rel, new_title)?;
            if let Some(index_arc) = &self.vault_index {
                if let Ok(mut idx) = index_arc.lock() {
                    let _ = idx.remove_note(rel);
                    if let Ok(parsed) = parse_document(&renamed.content) {
                        let _ = idx.index_note(&renamed, &parsed);
                    }
                }
            }
            if let Some(active) = &self.active_note {
                if active.relative_path == rel {
                    self.active_note = Some(renamed);
                }
            }
            self.status_message = format!("Renamed to '{new_title}'");
            self.refresh_entries()?;
        }
        Ok(())
    }

    /// Updates editor content while typing and marks note as dirty.
    pub fn update_editor_content(&mut self, content: String) {
        if self.editor_content != content {
            self.editor_content = content;
            self.is_dirty = true;
        }
    }

    /// Toggles light/dark theme and persists the choice.
    pub fn toggle_theme(&mut self) {
        self.theme = self.theme.toggle();
        self.preferences.theme = self.theme;
        self.preferences.save();
    }

    /// Opens the PDF import dialog.
    pub fn open_pdf_import_modal(&mut self) {
        self.show_pdf_import_modal = true;
        self.pdf_selected_path = None;
        self.pdf_progress = None;
        self.pdf_cancellation = None;
        self.pdf_last_result = None;
        self.pdf_error = None;
    }

    /// Closes the PDF import dialog, cancelling any ongoing conversion.
    pub fn close_pdf_import_modal(&mut self) {
        if let Some(token) = &self.pdf_cancellation {
            token.cancel();
        }
        self.show_pdf_import_modal = false;
        self.pdf_selected_path = None;
        self.pdf_progress = None;
        self.pdf_cancellation = None;
        self.pdf_last_result = None;
        self.pdf_error = None;
    }

    /// Sets the selected PDF file path for import.
    pub fn set_pdf_selected_path(&mut self, path: PathBuf) {
        self.pdf_selected_path = Some(path);
        self.pdf_error = None;
    }

    /// Requests cancellation of active PDF conversion.
    pub fn cancel_pdf_import(&mut self) {
        if let Some(token) = &self.pdf_cancellation {
            token.cancel();
        }
        self.pdf_cancellation = None;
        self.pdf_progress = None;
        self.status_message = "PDF conversion cancelled".to_string();
    }

    /// Completes the PDF import by refreshing vault entries, indexing the new note,
    /// and updating UI completion state.
    pub fn complete_pdf_import(&mut self, result: ImportResult) {
        let _ = self.refresh_entries();

        // Automatically index the newly imported note in SQLite and Tantivy
        if let (Some(vault), Some(index_arc)) = (&self.vault_service, &self.vault_index) {
            if let Ok(note) = vault.read_note(&result.relative_vault_path) {
                if let Ok(parsed) = parse_document(&note.content) {
                    if let Ok(mut idx) = index_arc.lock() {
                        let _ = idx.index_note(&note, &parsed);
                    }
                }
            }
        }

        self.pdf_last_result = Some(result);
        self.pdf_cancellation = None;
        self.pdf_progress = None;
        self.pdf_error = None;
        self.status_message = "PDF import completed successfully".to_string();
    }

    /// Opens the newly imported note in the Editor and closes the modal.
    pub fn open_imported_note(&mut self) {
        if let Some(res) = &self.pdf_last_result {
            let rel_path = res.relative_vault_path.clone();
            self.show_pdf_import_modal = false;
            self.active_view = ActiveView::Editor;
            let _ = self.select_note(&rel_path);
        }
    }

    /// Records an error during PDF import.
    pub fn set_pdf_error(&mut self, error: String) {
        self.pdf_error = Some(error);
        self.pdf_cancellation = None;
        self.pdf_progress = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_app_state_open_and_note_workflow() {
        let tmp = tempdir().unwrap();
        let vault_path = tmp.path().join("StateVault");

        let mut state = AppState::default();
        state
            .create_vault(&vault_path, Some("State Vault".to_string()))
            .unwrap();
        assert_eq!(state.vault_name, "State Vault");

        // Create note
        state.create_note("First Note", Some("Notes")).unwrap();
        assert!(state.active_note.is_some());
        assert_eq!(state.active_note.as_ref().unwrap().title, "First Note");

        // Update content
        state.update_editor_content("# Hello World\nTesting state".to_string());
        assert!(state.is_dirty);

        // Save
        state.save_active_note().unwrap();
        assert!(!state.is_dirty);

        // Re-read directly from service to verify persistence
        let read = state
            .vault_service
            .as_ref()
            .unwrap()
            .read_note("Notes/First Note.md")
            .unwrap();
        assert_eq!(read.content, "# Hello World\nTesting state");
    }

    #[test]
    fn test_theme_toggle() {
        let mut state = AppState::default();
        let initial = state.theme;
        state.toggle_theme();
        assert_ne!(state.theme, initial);
        state.toggle_theme();
        assert_eq!(state.theme, initial);
    }

    #[test]
    fn test_app_state_links_and_tasks() {
        let tmp = tempdir().unwrap();
        let vault_path = tmp.path().join("LinksVault");

        let mut state = AppState::default();
        state
            .create_vault(&vault_path, Some("Links Vault".to_string()))
            .unwrap();

        // 1. Create Source Note with Wikilinks, tags, and tasks
        state.create_note("Source Note", None).unwrap();
        let source_path = state.active_note.as_ref().unwrap().relative_path.clone();
        state.update_editor_content(
            "# Source Note\nLinks to [[Target Note]] and [[New Note]]. Tags: #rust #project\n\n- [ ] Task 1\n- [x] Task 2"
                .to_string(),
        );
        state.save_active_note().unwrap();

        // Check tags
        let tags = state.get_current_note_tags();
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"project".to_string()));

        // Check outgoing links (Target Note is unresolved initially)
        let outgoing = state.get_current_outgoing_links();
        assert_eq!(outgoing.len(), 2);
        assert!(outgoing[0].1.is_none());

        // 2. Open or create target note
        state.open_or_create_target("Target Note").unwrap();
        assert_eq!(state.active_note.as_ref().unwrap().title, "Target Note");

        // Check backlinks for Target Note
        let backlinks = state.get_current_backlinks();
        assert_eq!(backlinks.len(), 1);
        assert_eq!(
            backlinks[0].file_stem().unwrap().to_string_lossy(),
            "Source Note"
        );

        // 3. Switch back to Source Note and test task toggling
        state.select_note(&source_path).unwrap();
        // Line 4 is "- [ ] Task 1"
        state.toggle_task_at_line(4).unwrap();
        assert!(state.editor_content.contains("- [x] Task 1"));
    }
}
