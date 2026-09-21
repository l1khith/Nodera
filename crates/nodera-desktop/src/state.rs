use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info};

use nodera_core::{BibLibrary, IndexingProgress, Note, Result, Vault, VaultEntry, VaultService};
use nodera_markdown::{parse_document, LinkAuditReport, LinkGraph};

use crate::strings::palette;
use crate::theme::Theme;

fn default_sidebar_width() -> u32 {
    260
}
fn default_context_width() -> u32 {
    280
}
fn default_editor_font_size() -> u32 {
    15
}
fn default_reading_font_size() -> u32 {
    15
}
fn default_true() -> bool {
    true
}
fn default_autosave() -> u32 {
    2
}

fn default_text_fade() -> f32 {
    -1.10
}
fn default_node_size() -> f32 {
    1.0
}
fn default_link_thickness() -> f32 {
    1.0
}
fn default_center_force() -> f32 {
    0.40
}
fn default_repel_force() -> f32 {
    8.0
}
fn default_link_force() -> f32 {
    0.80
}
fn default_link_distance() -> f32 {
    120.0
}

/// Filter settings for knowledge graph view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphFilterSettings {
    #[serde(default)]
    pub search_query: String,
    #[serde(default)]
    pub tags: bool,
    #[serde(default)]
    pub attachments: bool,
    #[serde(default)]
    pub existing_files_only: bool,
    #[serde(default = "default_true")]
    pub orphans: bool,
}

impl Default for GraphFilterSettings {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            tags: false,
            attachments: false,
            existing_files_only: false,
            orphans: true,
        }
    }
}

/// Visual display settings for knowledge graph rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphDisplaySettings {
    #[serde(default)]
    pub arrows: bool,
    #[serde(default = "default_text_fade")]
    pub text_fade_threshold: f32,
    #[serde(default = "default_node_size")]
    pub node_size: f32,
    #[serde(default = "default_link_thickness")]
    pub link_thickness: f32,
    #[serde(default = "default_true")]
    pub animate: bool,
    #[serde(default = "default_true")]
    pub color_by_community: bool,
    #[serde(default = "default_true")]
    pub centrality_sizing: bool,
}

impl Default for GraphDisplaySettings {
    fn default() -> Self {
        Self {
            arrows: false,
            text_fade_threshold: default_text_fade(),
            node_size: default_node_size(),
            link_thickness: default_link_thickness(),
            animate: true,
            color_by_community: true,
            centrality_sizing: true,
        }
    }
}

/// Physics force configuration for knowledge graph simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphForcesSettings {
    #[serde(default = "default_center_force")]
    pub center_force: f32,
    #[serde(default = "default_repel_force")]
    pub repel_force: f32,
    #[serde(default = "default_link_force")]
    pub link_force: f32,
    #[serde(default = "default_link_distance")]
    pub link_distance: f32,
}

impl Default for GraphForcesSettings {
    fn default() -> Self {
        Self {
            center_force: default_center_force(),
            repel_force: default_repel_force(),
            link_force: default_link_force(),
            link_distance: default_link_distance(),
        }
    }
}

/// Collapsed/expanded state for graph controls sections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphExpandedSections {
    #[serde(default = "default_true")]
    pub filters: bool,
    #[serde(default)]
    pub groups: bool,
    #[serde(default)]
    pub display: bool,
    #[serde(default)]
    pub forces: bool,
}

impl Default for GraphExpandedSections {
    fn default() -> Self {
        Self {
            filters: true,
            groups: false,
            display: false,
            forces: false,
        }
    }
}

/// Comprehensive configuration for knowledge graph view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GraphSettings {
    #[serde(default)]
    pub filters: GraphFilterSettings,
    #[serde(default)]
    pub display: GraphDisplaySettings,
    #[serde(default)]
    pub forces: GraphForcesSettings,
    #[serde(default)]
    pub is_panel_open: bool,
    #[serde(default)]
    pub expanded_sections: GraphExpandedSections,
}

impl GraphSettings {
    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
    }
}

/// Persistent local state remembered across application restarts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppPreferences {
    pub last_vault: Option<PathBuf>,
    pub recent_vaults: Vec<PathBuf>,
    pub theme: Theme,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u32,
    #[serde(default = "default_context_width")]
    pub context_panel_width: u32,
    #[serde(default)]
    pub reading_progress: HashMap<String, u32>,
    #[serde(default = "default_editor_font_size")]
    pub editor_font_size: u32,
    #[serde(default = "default_reading_font_size")]
    pub reading_font_size: u32,
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,
    #[serde(default = "default_autosave")]
    pub auto_save_seconds: u32,
    #[serde(default)]
    pub graph_settings: GraphSettings,
    #[serde(default)]
    pub bookmarks: Vec<PathBuf>,
    #[serde(default)]
    pub recent_notes: Vec<PathBuf>,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            last_vault: None,
            recent_vaults: Vec::new(),
            theme: Theme::Dark,
            sidebar_width: 260,
            context_panel_width: 280,
            reading_progress: HashMap::new(),
            editor_font_size: 15,
            reading_font_size: 15,
            show_line_numbers: true,
            auto_save_seconds: 2,
            graph_settings: GraphSettings::default(),
            bookmarks: Vec::new(),
            recent_notes: Vec::new(),
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

use nodera_index::{
    IndexedTask, KnowledgeStats, RelatedNote, ReviewCategory, ReviewQueueItem, SearchResult,
    TaskFilter, VaultIndex,
};
use nodera_markdown::{
    index_note_template, inject_or_update_frontmatter, meeting_note_template, parse_frontmatter,
    permanent_note_template, project_note_template, rough_note_template, source_note_template,
    video_source_template, NOTE_TYPE_INDEX, NOTE_TYPE_MEETING, NOTE_TYPE_PERMANENT,
    NOTE_TYPE_PROJECT, NOTE_TYPE_ROUGH, NOTE_TYPE_SOURCE, SOURCE_TYPE_BOOK, SOURCE_TYPE_VIDEO,
};
use nodera_pdf::{CancellationToken, ConversionOptions, ConversionProgress, ImportResult};
use std::sync::{Arc, Mutex};

/// Primary workspace view mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveView {
    #[default]
    Editor,
    Tasks,
    ReviewQueue,
    Library,
    Graph,
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
    OpenDailyNote,
    CreateNote,
    CreateTypedNote {
        note_type: String,
        sub_type: Option<String>,
    },
    OpenQuickCapture,
    PromoteActiveNoteToPermanent,
    ImportPdf,
    SwitchView(ActiveView),
    ToggleTheme,
    ToggleReadingMode,
    OpenSettings,
    ResetLayout,
    InsertTemplate,
    RebuildIndex,
    OpenCitationPicker,
    ExtractPdfAnnotations,
    ExecutePluginCommand(String),
}

/// Book or long-form document displayed in the Library view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryBook {
    pub relative_path: PathBuf,
    pub title: String,
    pub source: Option<String>,
    pub pages: usize,
    pub chapters: usize,
    pub tags: Vec<String>,
    pub word_count: usize,
    pub char_count: usize,
    pub reading_progress_pct: u32,
}

/// An open tab in the editor pane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenTab {
    pub relative_path: PathBuf,
    pub title: String,
    #[serde(default)]
    pub is_pinned: bool,
}

/// Note template item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateItem {
    pub name: String,
    pub description: String,
    pub content: String,
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

    // Tabs & Navigation History
    pub open_tabs: Vec<OpenTab>,
    pub active_tab_index: Option<usize>,
    pub nav_history: Vec<PathBuf>,
    pub nav_history_index: usize,

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

    // Layout configuration
    pub sidebar_width: u32,
    pub context_panel_width: u32,
    pub is_resizing_sidebar: bool,
    pub is_resizing_context: bool,

    // Reading mode Table of Contents
    pub toc_headings: Vec<(usize, String)>,

    // Sidebar sections
    pub show_bookmarks_section: bool,
    pub show_recent_section: bool,

    // Library view
    pub library_search_query: String,

    // Modal dialogs
    pub show_new_note_dialog: bool,
    pub show_delete_confirm_dialog: bool,
    pub note_to_delete: Option<PathBuf>,
    pub show_rename_dialog: bool,
    pub note_to_rename: Option<PathBuf>,
    pub show_command_palette: bool,
    pub command_palette_query: String,

    // Settings modal
    pub show_settings_modal: bool,
    pub settings_active_tab: String,

    // Error alert dialog
    pub show_error_dialog: bool,
    pub error_dialog_title: String,
    pub error_dialog_message: String,
    pub error_dialog_details: Option<String>,

    // Template modal
    pub show_template_modal: bool,
    pub template_search_query: String,

    // PDF Import modal
    pub show_pdf_import_modal: bool,
    pub pdf_selected_path: Option<PathBuf>,
    pub pdf_import_options: ConversionOptions,
    pub pdf_progress: Option<ConversionProgress>,
    pub pdf_cancellation: Option<CancellationToken>,
    pub pdf_last_result: Option<ImportResult>,
    pub pdf_error: Option<String>,

    // Trash modal
    pub show_trash_modal: bool,

    // Split view
    pub split_pane: Option<SplitPane>,
    pub split_direction: SplitDirection,

    // Vault Health Doctor & Graph Intelligence
    pub show_vault_health_modal: bool,
    pub graph_color_by_community: bool,
    pub graph_centrality_sizing: bool,

    // Research Workspace (V0.4)
    pub bib_library: BibLibrary,
    pub show_citation_picker_modal: bool,
    pub show_pdf_annotation_modal: bool,
    pub indexing_progress: Option<IndexingProgress>,

    // Extensible Plugins (V0.5)
    pub plugin_manager: nodera_core::PluginManager,

    // Quick Capture & Review Queue (V0.6)
    pub show_quick_capture: bool,
    pub quick_capture_title: String,
    pub quick_capture_body: String,
    pub quick_capture_tags: String,
    pub review_queue_filter: ReviewCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SplitDirection {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitPane {
    pub relative_path: Option<PathBuf>,
    pub is_reading_mode: bool,
    pub editor_content: String,
}

/// Discovered mention of a note title in another note without an existing wikilink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnlinkedMention {
    pub source_path: PathBuf,
    pub source_title: String,
    pub snippet_before: String,
    pub matched_text: String,
    pub snippet_after: String,
}

impl Default for AppState {
    fn default() -> Self {
        let prefs = AppPreferences::load();
        let initial_theme = prefs.theme;
        let sidebar_w = prefs.sidebar_width;
        let context_w = prefs.context_panel_width;

        Self {
            vault_service: None,
            vault_path: None,
            vault_name: "No Vault Opened".to_string(),
            entries: Vec::new(),

            active_note: None,
            editor_content: String::new(),
            is_dirty: false,
            is_reading_mode: false,

            open_tabs: Vec::new(),
            active_tab_index: None,
            nav_history: Vec::new(),
            nav_history_index: 0,

            active_view: ActiveView::Editor,
            vault_index: None,
            search_query: String::new(),
            search_results: Vec::new(),
            task_filter: TaskFilter::default(),

            theme: initial_theme,
            sidebar_open: true,
            context_panel_open: false,
            status_message: "Ready".to_string(),

            sidebar_width: sidebar_w,
            context_panel_width: context_w,
            is_resizing_sidebar: false,
            is_resizing_context: false,
            toc_headings: Vec::new(),

            show_bookmarks_section: true,
            show_recent_section: true,

            library_search_query: String::new(),

            preferences: prefs,
            link_graph: LinkGraph::new(),

            show_new_note_dialog: false,
            show_delete_confirm_dialog: false,
            note_to_delete: None,
            show_rename_dialog: false,
            note_to_rename: None,
            show_command_palette: false,
            command_palette_query: String::new(),

            show_settings_modal: false,
            settings_active_tab: "General".to_string(),

            show_error_dialog: false,
            error_dialog_title: String::new(),
            error_dialog_message: String::new(),
            error_dialog_details: None,

            show_template_modal: false,
            template_search_query: String::new(),

            show_pdf_import_modal: false,
            pdf_selected_path: None,
            pdf_import_options: ConversionOptions::default(),
            pdf_progress: None,
            pdf_cancellation: None,
            pdf_last_result: None,
            pdf_error: None,

            show_trash_modal: false,
            split_pane: None,
            split_direction: SplitDirection::default(),

            show_vault_health_modal: false,
            graph_color_by_community: true,
            graph_centrality_sizing: true,

            bib_library: BibLibrary::default(),
            show_citation_picker_modal: false,
            show_pdf_annotation_modal: false,
            indexing_progress: None,
            plugin_manager: nodera_core::PluginManager::default(),

            show_quick_capture: false,
            quick_capture_title: String::new(),
            quick_capture_body: String::new(),
            quick_capture_tags: String::new(),
            review_queue_filter: ReviewCategory::RoughNote,
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

        // Parallel note link extraction across all CPU cores via Rayon
        let note_links: Vec<(PathBuf, Vec<nodera_markdown::Wikilink>)> = entries
            .par_iter()
            .filter_map(|entry| {
                if let VaultEntry::Note(summary) = entry {
                    if let Ok(note) = service.read_note(&summary.relative_path) {
                        if let Ok(parsed) = parse_document(&note.content) {
                            return Some((summary.relative_path.clone(), parsed.wikilinks));
                        }
                    }
                }
                None
            })
            .collect();
        let note_paths: Vec<PathBuf> = entries
            .iter()
            .filter_map(|e| match e {
                VaultEntry::Note(s) => Some(s.relative_path.clone()),
                _ => None,
            })
            .collect();

        let link_graph = LinkGraph::build(&note_paths, note_links);

        // Initialize derived VaultIndex (SQLite + Tantivy) with parallel pipeline
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

        // Flush pending unsaved changes from previous vault if any
        if self.is_dirty && self.active_note.is_some() {
            let _ = self.save_active_note();
        }

        self.vault_path = Some(p.to_path_buf());
        self.vault_name = name;
        self.entries = entries;
        self.vault_service = Some(service);
        self.vault_index = vault_index;
        self.link_graph = link_graph;
        self.active_note = None;
        self.editor_content.clear();
        self.is_dirty = false;
        self.open_tabs.clear();
        self.active_tab_index = None;
        self.nav_history.clear();
        self.nav_history_index = 0;
        self.split_pane = None;
        self.search_query.clear();
        self.search_results.clear();
        self.toc_headings.clear();
        self.status_message = format!("Vault '{}' opened", self.vault_name);
        self.indexing_progress = None;

        self.refresh_bib_library();

        let mut plugin_mgr = nodera_core::PluginManager::new();
        let _ = plugin_mgr.discover_from_vault(p);
        plugin_mgr.dispatch_hook(&nodera_core::PluginHook::VaultOpened {
            vault_name: &self.vault_name,
        });
        self.plugin_manager = plugin_mgr;

        self.preferences.record_vault(p.to_path_buf());
        Ok(())
    }

    /// Creates a new vault directory and sets it as the active vault.
    pub fn create_vault(&mut self, path: impl AsRef<Path>, name: Option<String>) -> Result<()> {
        let p = path.as_ref();
        info!(path = %p.display(), "Creating vault in AppState");

        // Flush pending unsaved changes from previous vault if any
        if self.is_dirty && self.active_note.is_some() {
            let _ = self.save_active_note();
        }

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
        self.open_tabs.clear();
        self.active_tab_index = None;
        self.nav_history.clear();
        self.nav_history_index = 0;
        self.split_pane = None;
        self.search_query.clear();
        self.search_results.clear();
        self.toc_headings.clear();
        self.status_message = format!("Vault '{}' created", self.vault_name);

        self.refresh_bib_library();

        let mut plugin_mgr = nodera_core::PluginManager::new();
        let _ = plugin_mgr.discover_from_vault(p);
        self.plugin_manager = plugin_mgr;

        self.preferences.record_vault(p.to_path_buf());
        Ok(())
    }

    /// Closes the currently active vault, flushes pending edits, and resets all vault-scoped state.
    pub fn close_vault(&mut self) -> Result<()> {
        if self.is_dirty && self.active_note.is_some() {
            let _ = self.save_active_note();
        }

        self.vault_service = None;
        self.vault_path = None;
        self.vault_name = "No Vault Opened".to_string();
        self.entries.clear();

        self.active_note = None;
        self.editor_content.clear();
        self.is_dirty = false;
        self.is_reading_mode = false;

        self.open_tabs.clear();
        self.active_tab_index = None;
        self.nav_history.clear();
        self.nav_history_index = 0;

        self.vault_index = None;
        self.search_query.clear();
        self.search_results.clear();
        self.toc_headings.clear();

        self.link_graph = LinkGraph::new();
        self.split_pane = None;
        self.bib_library = BibLibrary::default();
        self.indexing_progress = None;
        self.plugin_manager = nodera_core::PluginManager::default();

        self.show_quick_capture = false;
        self.quick_capture_title.clear();
        self.quick_capture_body.clear();
        self.quick_capture_tags.clear();
        self.review_queue_filter = ReviewCategory::RoughNote;

        self.status_message = "Vault closed".to_string();
        Ok(())
    }

    /// Refreshes the file tree listing from the vault.
    pub fn refresh_entries(&mut self) -> Result<()> {
        if let Some(service) = &self.vault_service {
            self.entries = service.list_entries()?;
        }
        Ok(())
    }

    /// Opens a note into the editor and manages open tabs and navigation history.
    pub fn select_note(&mut self, relative_path: impl AsRef<Path>) -> Result<()> {
        let rel = relative_path.as_ref();
        if let Some(service) = &self.vault_service {
            let note = service.read_note(rel)?;
            let rel_buf = rel.to_path_buf();
            let title = note.title.clone();

            self.editor_content = note.content.clone();
            self.active_note = Some(note);
            self.active_view = ActiveView::Editor;
            self.is_dirty = false;
            self.status_message = format!("Opened '{}'", rel.display());

            // Tab management
            if let Some(idx) = self
                .open_tabs
                .iter()
                .position(|t| t.relative_path == rel_buf)
            {
                self.active_tab_index = Some(idx);
            } else {
                self.open_tabs.push(OpenTab {
                    relative_path: rel_buf.clone(),
                    title,
                    is_pinned: false,
                });
                self.active_tab_index = Some(self.open_tabs.len() - 1);
            }

            self.record_recent_note(&rel_buf);

            // Navigation history
            if self.nav_history.get(self.nav_history_index) != Some(&rel_buf) {
                if !self.nav_history.is_empty()
                    && self.nav_history_index + 1 < self.nav_history.len()
                {
                    self.nav_history.truncate(self.nav_history_index + 1);
                }
                self.nav_history.push(rel_buf);
                self.nav_history_index = self.nav_history.len() - 1;
            }
        }
        Ok(())
    }

    /// Selects an open tab by index.
    pub fn select_tab(&mut self, index: usize) -> Result<()> {
        if let Some(tab) = self.open_tabs.get(index) {
            let path = tab.relative_path.clone();
            self.select_note(&path)?;
        }
        Ok(())
    }

    /// Closes a tab by index.
    pub fn close_tab(&mut self, index: usize) -> Result<()> {
        if index >= self.open_tabs.len() {
            return Ok(());
        }

        let is_closing_active = self.active_tab_index == Some(index);
        let _ = self.open_tabs.remove(index);

        if self.open_tabs.is_empty() {
            self.active_tab_index = None;
            self.active_note = None;
            self.editor_content.clear();
            self.is_dirty = false;
        } else if is_closing_active {
            let next_idx = index.min(self.open_tabs.len() - 1);
            let next_path = self.open_tabs[next_idx].relative_path.clone();
            self.select_note(&next_path)?;
        } else if let Some(cur_idx) = self.active_tab_index {
            if index < cur_idx {
                self.active_tab_index = Some(cur_idx - 1);
            }
        }

        Ok(())
    }

    /// Closes all tabs except the one specified (preserves pinned tabs).
    pub fn close_other_tabs(&mut self, keep_index: usize) -> Result<()> {
        if keep_index >= self.open_tabs.len() {
            return Ok(());
        }
        let keep_tab = self.open_tabs[keep_index].clone();
        self.open_tabs
            .retain(|t| t.is_pinned || t.relative_path == keep_tab.relative_path);
        if let Some(pos) = self
            .open_tabs
            .iter()
            .position(|t| t.relative_path == keep_tab.relative_path)
        {
            self.active_tab_index = Some(pos);
            self.select_note(&keep_tab.relative_path)?;
        }
        Ok(())
    }

    /// Closes all non-pinned tabs.
    pub fn close_all_tabs(&mut self) -> Result<()> {
        self.open_tabs.retain(|t| t.is_pinned);
        if let Some(first_pinned) = self.open_tabs.first().cloned() {
            self.active_tab_index = Some(0);
            self.select_note(&first_pinned.relative_path)?;
        } else {
            self.active_tab_index = None;
            self.active_note = None;
            self.editor_content.clear();
            self.is_dirty = false;
        }
        Ok(())
    }

    /// Toggles the pinned state of a tab.
    pub fn toggle_pin_tab(&mut self, index: usize) {
        if let Some(tab) = self.open_tabs.get_mut(index) {
            tab.is_pinned = !tab.is_pinned;
        }
    }

    /// Toggles a note's bookmark status.
    pub fn toggle_bookmark(&mut self, path: impl AsRef<Path>) {
        let p = path.as_ref().to_path_buf();
        if let Some(idx) = self.preferences.bookmarks.iter().position(|b| b == &p) {
            self.preferences.bookmarks.remove(idx);
            self.status_message = format!("Removed bookmark: {}", p.display());
        } else {
            self.preferences.bookmarks.push(p.clone());
            self.status_message = format!("Bookmarked: {}", p.display());
        }
        self.preferences.save();
    }

    /// Checks if a note is bookmarked.
    pub fn is_bookmarked(&self, path: impl AsRef<Path>) -> bool {
        self.preferences
            .bookmarks
            .iter()
            .any(|b| b == path.as_ref())
    }

    /// Records a note as recently opened, keeping up to 10 entries.
    pub fn record_recent_note(&mut self, path: impl AsRef<Path>) {
        let p = path.as_ref().to_path_buf();
        self.preferences.recent_notes.retain(|x| x != &p);
        self.preferences.recent_notes.insert(0, p);
        if self.preferences.recent_notes.len() > 10 {
            self.preferences.recent_notes.truncate(10);
        }
        self.preferences.save();
    }

    /// Removes a note from recent list.
    pub fn remove_recent_note(&mut self, path: impl AsRef<Path>) {
        let p = path.as_ref();
        self.preferences.recent_notes.retain(|x| x != p);
        self.preferences.save();
    }

    /// Clears all recent notes.
    pub fn clear_recent_notes(&mut self) {
        self.preferences.recent_notes.clear();
        self.preferences.save();
    }

    /// Checks if back navigation is possible.
    pub fn can_navigate_back(&self) -> bool {
        self.nav_history_index > 0
    }

    /// Checks if forward navigation is possible.
    pub fn can_navigate_forward(&self) -> bool {
        !self.nav_history.is_empty() && self.nav_history_index + 1 < self.nav_history.len()
    }

    /// Moves back one step in note history.
    pub fn navigate_back(&mut self) -> Result<()> {
        if self.can_navigate_back() {
            self.nav_history_index -= 1;
            let path = self.nav_history[self.nav_history_index].clone();
            if let Some(service) = &self.vault_service {
                let note = service.read_note(&path)?;
                let title = note.title.clone();
                self.editor_content = note.content.clone();
                self.active_note = Some(note);
                self.active_view = ActiveView::Editor;
                self.is_dirty = false;
                if let Some(idx) = self.open_tabs.iter().position(|t| t.relative_path == path) {
                    self.active_tab_index = Some(idx);
                } else {
                    self.open_tabs.push(OpenTab {
                        relative_path: path.clone(),
                        title,
                        is_pinned: false,
                    });
                    self.active_tab_index = Some(self.open_tabs.len() - 1);
                }
            }
        }
        Ok(())
    }

    /// Moves forward one step in note history.
    pub fn navigate_forward(&mut self) -> Result<()> {
        if self.can_navigate_forward() {
            self.nav_history_index += 1;
            let path = self.nav_history[self.nav_history_index].clone();
            if let Some(service) = &self.vault_service {
                let note = service.read_note(&path)?;
                let title = note.title.clone();
                self.editor_content = note.content.clone();
                self.active_note = Some(note);
                self.active_view = ActiveView::Editor;
                self.is_dirty = false;
                if let Some(idx) = self.open_tabs.iter().position(|t| t.relative_path == path) {
                    self.active_tab_index = Some(idx);
                } else {
                    self.open_tabs.push(OpenTab {
                        relative_path: path.clone(),
                        title,
                        is_pinned: false,
                    });
                    self.active_tab_index = Some(self.open_tabs.len() - 1);
                }
            }
        }
        Ok(())
    }

    /// Opens or creates today's Daily Note (e.g. `Daily/YYYY-MM-DD.md`).
    pub fn open_or_create_daily_note(&mut self) -> Result<()> {
        let now = chrono::Local::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let rel_path = PathBuf::from("Daily").join(format!("{date_str}.md"));

        if let Some(service) = &self.vault_service {
            if service.read_note(&rel_path).is_err() {
                let initial_body = format!(
                    "---\ntitle: {}\ndate: {}\ntags:\n  - daily\n---\n\n# Daily Note — {}\n\n## Tasks\n- [ ] \n\n## Notes\n\n",
                    date_str, date_str, date_str
                );
                let _ = service.create_note(Some("Daily"), &date_str, Some(&initial_body))?;
                self.refresh_entries()?;
            }
            self.select_note(&rel_path)?;
        }
        Ok(())
    }

    /// Attempts to switch to another vault given its name or path.
    pub fn switch_vault_by_name_or_path(&mut self, target: &str) -> Result<()> {
        let trimmed = target.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        // If it's already the active vault, nothing to do
        if self.vault_name.eq_ignore_ascii_case(trimmed) {
            return Ok(());
        }
        if let Some(ref cur_path) = self.vault_path {
            if cur_path.to_string_lossy().eq_ignore_ascii_case(trimmed) {
                return Ok(());
            }
        }

        // Check if target is a valid directory path on disk
        let path = Path::new(trimmed);
        if path.is_dir() {
            return self.open_vault(path);
        }

        // Search recent vaults in preferences
        let matching_recent = self
            .preferences
            .recent_vaults
            .iter()
            .find(|recent| {
                recent
                    .file_name()
                    .and_then(|s| s.to_str())
                    .is_some_and(|name| name.eq_ignore_ascii_case(trimmed))
            })
            .cloned();

        if let Some(recent_path) = matching_recent {
            return self.open_vault(recent_path);
        }

        Ok(())
    }

    /// Handles a `nodera://` deep-linking URI.
    pub fn handle_nodera_uri(&mut self, uri: &nodera_core::NoderaUri) -> Result<()> {
        match uri {
            nodera_core::NoderaUri::Open {
                vault,
                note,
                line: _,
            } => {
                if let Some(v) = vault {
                    let _ = self.switch_vault_by_name_or_path(v);
                }

                let note_trimmed = note.trim();
                let mut note_path = PathBuf::from(note_trimmed.replace('\\', "/"));
                if note_path.extension().is_none() {
                    note_path.set_extension("md");
                }

                // Try direct select
                if let Some(service) = &self.vault_service {
                    if service.read_note(&note_path).is_ok() {
                        self.select_note(&note_path)?;
                        self.status_message = format!("Opened '{}' via URI", note_path.display());
                        return Ok(());
                    }
                }

                // Try target resolver across all notes
                let note_paths = self.note_paths();
                let resolver = nodera_markdown::TargetResolver::from_paths(&note_paths);
                if let Some(resolved) = resolver.resolve(note_trimmed) {
                    self.select_note(&resolved)?;
                    self.status_message = format!("Opened '{}' via URI", resolved.display());
                    return Ok(());
                }

                // If not found, create new note
                if let Some(service) = &self.vault_service {
                    let note_stem = Path::new(note_trimmed)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(note_trimmed);
                    let folder = Path::new(note_trimmed)
                        .parent()
                        .and_then(|p| p.to_str())
                        .filter(|f| !f.is_empty());
                    let new_note = service.create_note(folder, note_stem, Some(""))?;
                    self.refresh_entries()?;
                    self.select_note(&new_note.relative_path)?;
                    self.status_message = format!("Created & opened '{}' via URI", new_note.title);
                }
                Ok(())
            }
            nodera_core::NoderaUri::New {
                vault,
                title,
                content,
                tags,
            } => {
                if let Some(v) = vault {
                    let _ = self.switch_vault_by_name_or_path(v);
                }

                let mut body = String::new();
                if !tags.is_empty() {
                    body.push_str("---\ntags:\n");
                    for t in tags {
                        body.push_str(&format!("  - {}\n", t));
                    }
                    body.push_str("---\n\n");
                }
                if let Some(c) = content {
                    body.push_str(c);
                }

                if let Some(service) = &self.vault_service {
                    let title_stem = Path::new(title)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(title);
                    let folder = Path::new(title)
                        .parent()
                        .and_then(|p| p.to_str())
                        .filter(|f| !f.is_empty());
                    let new_note = service.create_note(folder, title_stem, Some(&body))?;
                    self.refresh_entries()?;
                    self.select_note(&new_note.relative_path)?;
                    self.status_message = format!("Created note '{}' via URI", new_note.title);
                }
                Ok(())
            }
            nodera_core::NoderaUri::Search { vault, query } => {
                if let Some(v) = vault {
                    let _ = self.switch_vault_by_name_or_path(v);
                }
                self.show_command_palette = true;
                self.command_palette_query = query.clone();
                self.execute_search(query);
                self.status_message = format!("Searching for '{}'", query);
                Ok(())
            }
            nodera_core::NoderaUri::Daily { vault } => {
                if let Some(v) = vault {
                    let _ = self.switch_vault_by_name_or_path(v);
                }
                self.open_or_create_daily_note()?;
                Ok(())
            }
        }
    }

    /// Returns available templates from `<vault>/Templates` folder as well as built-in default templates.
    pub fn get_available_templates(&self) -> Vec<TemplateItem> {
        let mut templates = Vec::new();

        // 1. Vault templates from `Templates/` or `templates/` folder
        if let Some(vault_path) = &self.vault_path {
            for sub in &["Templates", "templates"] {
                let tmpl_dir = vault_path.join(sub);
                if tmpl_dir.is_dir() {
                    if let Ok(dir_entries) = std::fs::read_dir(&tmpl_dir) {
                        for entry in dir_entries.flatten() {
                            let path = entry.path();
                            if path.is_file()
                                && path.extension().and_then(|s| s.to_str()) == Some("md")
                            {
                                if let Ok(content) = std::fs::read_to_string(&path) {
                                    let name = path
                                        .file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("Untitled")
                                        .to_string();
                                    let description = content
                                        .lines()
                                        .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
                                        .map(|l| l.trim().chars().take(80).collect::<String>())
                                        .unwrap_or_else(|| "Custom vault template".to_string());
                                    templates.push(TemplateItem {
                                        name,
                                        description,
                                        content,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Built-in starter templates (if not already present with same name)
        let builtins = vec![
            TemplateItem {
                name: "Daily Journal".to_string(),
                description: "Focus of the day, notes, meetings, and reflections".to_string(),
                content: "# Daily Log: {{date}}\n\n## Focus of the Day\n- [ ] \n\n## Notes & Discoveries\n\n## Meetings & Communications\n\n## End of Day Reflection\n".to_string(),
            },
            TemplateItem {
                name: "Meeting Notes".to_string(),
                description: "Attendees, agenda, discussion notes, and action items".to_string(),
                content: "# Meeting: {{title}}\n**Date**: {{datetime}}\n**Attendees**: \n\n## Agenda\n1. \n\n## Discussion & Decisions\n\n## Action Items\n- [ ] \n".to_string(),
            },
            TemplateItem {
                name: "Project Plan".to_string(),
                description: "Overview, milestones, deliverables, and references".to_string(),
                content: "# Project: {{title}}\n**Created**: {{date}}\n**Status**: #project/planning\n\n## Overview & Objectives\n\n## Key Milestones\n- [ ] Milestone 1 (Target: )\n- [ ] Milestone 2 (Target: )\n\n## Tasks & Deliverables\n- [ ] Scoping & Requirements\n- [ ] Initial Architecture\n\n## References & Links\n".to_string(),
            },
            TemplateItem {
                name: "Book / Literature Note".to_string(),
                description: "Author, summary, key insights, and quotes".to_string(),
                content: "# Book: {{title}}\n**Author**: \n**Read Date**: {{date}}\n**Tags**: #book-review\n\n## Summary\n\n## Key Insights\n1. \n\n## Actionable Takeaways\n- [ ] \n\n## Memorable Quotes\n> \n".to_string(),
            },
            TemplateItem {
                name: "Weekly Review".to_string(),
                description: "Highlights, completed goals, challenges, and next week's focus".to_string(),
                content: "# Weekly Review: {{date}}\n\n## Big Wins & Highlights\n- \n\n## Goal Progress\n- [ ] Goal 1: \n- [ ] Goal 2: \n\n## Challenges & Blockers\n\n## Priorities for Next Week\n1. \n2. \n".to_string(),
            },
        ];

        for builtin in builtins {
            if !templates
                .iter()
                .any(|t| t.name.eq_ignore_ascii_case(&builtin.name))
            {
                templates.push(builtin);
            }
        }

        templates
    }

    /// Inserts a template into the active note (or creates a new note if none active), expanding dynamic placeholders.
    pub fn insert_template(
        &mut self,
        template_name: Option<&str>,
        template_content: &str,
    ) -> Result<()> {
        if self.active_note.is_none() {
            let note_name = template_name.unwrap_or("New Note");
            self.create_note(note_name, None)?;
        }

        let now = chrono::Local::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let time_str = now.format("%H:%M").to_string();
        let datetime_str = now.format("%Y-%m-%d %H:%M").to_string();
        let title_str = self
            .active_note
            .as_ref()
            .map(|n| n.title.clone())
            .unwrap_or_else(|| "Untitled".to_string());

        let expanded = template_content
            .replace("{{date}}", &date_str)
            .replace("{{time}}", &time_str)
            .replace("{{datetime}}", &datetime_str)
            .replace("{{title}}", &title_str);

        if self.editor_content.trim().is_empty() {
            self.editor_content = expanded;
        } else {
            self.editor_content.push_str("\n\n");
            self.editor_content.push_str(&expanded);
        }

        self.is_dirty = true;
        self.show_template_modal = false;
        self.template_search_query.clear();
        self.save_active_note()?;
        self.status_message = "Template inserted".to_string();
        Ok(())
    }

    /// Returns a list of relative paths for all notes in the vault entries.
    pub fn note_paths(&self) -> Vec<PathBuf> {
        self.entries
            .iter()
            .filter_map(|e| match e {
                VaultEntry::Note(s) => Some(s.relative_path.clone()),
                _ => None,
            })
            .collect()
    }

    /// Saves the current editor content to the active note.
    pub fn save_active_note(&mut self) -> Result<()> {
        if let (Some(service), Some(note)) = (&self.vault_service, &self.active_note) {
            let rel_path = note.relative_path.clone();
            let updated = service.write_note(&rel_path, &self.editor_content)?;
            if let Ok(parsed) = parse_document(&self.editor_content) {
                let paths = self.note_paths();
                self.link_graph.update_note_links_with_paths(
                    rel_path.clone(),
                    parsed.wikilinks.clone(),
                    &paths,
                );

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

            self.plugin_manager
                .dispatch_hook(&nodera_core::PluginHook::NoteSaved {
                    relative_path: &rel_path,
                    content: &self.editor_content,
                });
        }
        Ok(())
    }

    fn cleanup_closed_or_deleted_note(&mut self, rel: &Path) {
        self.link_graph.remove_note(rel);
        if let Some(index_arc) = &self.vault_index {
            if let Ok(mut idx) = index_arc.lock() {
                let _ = idx.remove_note(rel);
            }
        }
        let rel_buf = rel.to_path_buf();
        self.open_tabs.retain(|t| t.relative_path != rel_buf);
        self.nav_history.retain(|p| p != &rel_buf);
        self.preferences.bookmarks.retain(|b| b != rel);
        self.preferences.recent_notes.retain(|r| r != rel);
        self.preferences.save();
        if let Some(active_idx) = self.active_tab_index {
            if active_idx >= self.open_tabs.len() {
                self.active_tab_index = if self.open_tabs.is_empty() {
                    None
                } else {
                    Some(self.open_tabs.len() - 1)
                };
            }
        }
        if let Some(active) = &self.active_note {
            if active.relative_path == rel {
                self.active_note = None;
                self.editor_content.clear();
                self.is_dirty = false;
                if let Some(active_idx) = self.active_tab_index {
                    if let Some(next_tab) = self.open_tabs.get(active_idx) {
                        let next_path = next_tab.relative_path.clone();
                        let _ = self.select_note(&next_path);
                    }
                }
            }
        }
    }

    /// Moves a note into the vault's trash directory (safe soft delete).
    pub fn trash_note(&mut self, rel_path: impl AsRef<Path>) -> Result<()> {
        let rel = rel_path.as_ref();
        if let Some(service) = &self.vault_service {
            service.trash_note(rel)?;
            self.cleanup_closed_or_deleted_note(rel);
            self.status_message = format!("Moved '{}' to Trash", rel.display());
            self.refresh_entries()?;
        }
        Ok(())
    }

    /// Deletes a note (defaults to moving to trash).
    pub fn delete_note(&mut self, rel_path: impl AsRef<Path>) -> Result<()> {
        self.trash_note(rel_path)
    }

    /// Permanently deletes a note without moving it to trash.
    pub fn delete_note_permanently(&mut self, rel_path: impl AsRef<Path>) -> Result<()> {
        let rel = rel_path.as_ref();
        if let Some(service) = &self.vault_service {
            service.delete_note(rel)?;
            self.cleanup_closed_or_deleted_note(rel);
            self.status_message = format!("Permanently deleted '{}'", rel.display());
            self.refresh_entries()?;
        }
        Ok(())
    }

    /// Restores a note from the vault's trash directory.
    pub fn restore_trashed_note(&mut self, trash_filename: &str) -> Result<()> {
        if let Some(service) = &self.vault_service {
            let restored = service.restore_note(trash_filename)?;
            self.refresh_entries()?;
            let restored_path = restored.relative_path.clone();
            self.select_note(&restored_path)?;
            self.status_message = format!("Restored '{}'", restored.title);
        }
        Ok(())
    }

    /// Lists all notes in the vault's trash.
    pub fn list_trash(&self) -> Vec<nodera_core::TrashedNoteSummary> {
        self.vault_service
            .as_ref()
            .and_then(|s| s.list_trash().ok())
            .unwrap_or_default()
    }

    /// Empties the vault's trash directory permanently.
    pub fn empty_trash(&mut self) -> Result<usize> {
        if let Some(service) = &self.vault_service {
            let count = service.empty_trash()?;
            self.status_message = format!("Emptied trash ({count} items deleted)");
            return Ok(count);
        }
        Ok(0)
    }

    /// Permanently deletes a specific item from the trash directory.
    pub fn delete_trashed_permanently(&mut self, trash_filename: &str) -> Result<()> {
        if let Some(service) = &self.vault_service {
            service.delete_permanently(trash_filename)?;
            self.status_message = "Item permanently deleted from trash".to_string();
        }
        Ok(())
    }

    /// Discovers occurrences of the active note's title in other vault notes where no wikilink exists.
    pub fn get_unlinked_mentions(&self) -> Vec<UnlinkedMention> {
        let active = match &self.active_note {
            Some(n) => n,
            None => return Vec::new(),
        };

        let target_title = active.title.trim();
        if target_title.is_empty() || target_title.eq_ignore_ascii_case("untitled") {
            return Vec::new();
        }

        let target_lower = target_title.to_lowercase();
        let mut results = Vec::new();

        let service = match &self.vault_service {
            Some(s) => s,
            None => return Vec::new(),
        };

        for entry in &self.entries {
            if let VaultEntry::Note(summary) = entry {
                if summary.relative_path == active.relative_path {
                    continue;
                }

                let content = match service.read_note(&summary.relative_path) {
                    Ok(n) => n.content,
                    Err(_) => continue,
                };

                let content_lower = content.to_lowercase();
                let mut search_from = 0;

                while let Some(found_idx) = content_lower[search_from..].find(&target_lower) {
                    let match_start = search_from + found_idx;
                    let match_end = match_start + target_title.len();
                    search_from = match_end;

                    // Word boundary check
                    let char_before = content[..match_start].chars().last();
                    let char_after = content[match_end..].chars().next();
                    let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

                    if char_before.is_some_and(is_word_char) || char_after.is_some_and(is_word_char)
                    {
                        continue;
                    }

                    // Check if inside [[...]] on the same line
                    let line_start = content[..match_start]
                        .rfind('\n')
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    let line_end = content[match_end..]
                        .find('\n')
                        .map(|p| match_end + p)
                        .unwrap_or(content.len());
                    let line = &content[line_start..line_end];
                    let offset_in_line = match_start - line_start;

                    let before_in_line = &line[..offset_in_line];
                    let after_in_line = &line[offset_in_line + target_title.len()..];

                    let open_brackets = before_in_line.rfind("[[");
                    let close_brackets = before_in_line.rfind("]]");
                    let is_inside_wikilink = match (open_brackets, close_brackets) {
                        (Some(_), None) => after_in_line.contains("]]"),
                        (Some(o), Some(c)) if o > c => after_in_line.contains("]]"),
                        _ => false,
                    };

                    if is_inside_wikilink {
                        continue;
                    }

                    // Extract snippet with context
                    let snip_start = match_start.saturating_sub(30);
                    let snip_end = (match_end + 30).min(content.len());

                    results.push(UnlinkedMention {
                        source_path: summary.relative_path.clone(),
                        source_title: summary.title.clone(),
                        snippet_before: content[snip_start..match_start].to_string(),
                        matched_text: content[match_start..match_end].to_string(),
                        snippet_after: content[match_end..snip_end].to_string(),
                    });

                    if results.len() >= 25 {
                        return results;
                    }
                }
            }
        }

        results
    }

    /// Links an unlinked mention of target_title in source_path by converting it to `[[target_title]]`.
    pub fn link_unlinked_mention(&mut self, source_path: &Path, target_title: &str) -> Result<()> {
        let service = match &self.vault_service {
            Some(s) => s,
            None => return Ok(()),
        };

        let note = service.read_note(source_path)?;
        let content = note.content;
        let target_lower = target_title.to_lowercase();
        let content_lower = content.to_lowercase();

        let mut search_from = 0;
        let mut replacement_done = false;
        let mut updated_content = String::with_capacity(content.len() + 16);

        while let Some(found_idx) = content_lower[search_from..].find(&target_lower) {
            let match_start = search_from + found_idx;
            let match_end = match_start + target_title.len();

            let char_before = content[..match_start].chars().last();
            let char_after = content[match_end..].chars().next();
            let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

            if char_before.is_some_and(is_word_char) || char_after.is_some_and(is_word_char) {
                search_from = match_end;
                continue;
            }

            let line_start = content[..match_start]
                .rfind('\n')
                .map(|p| p + 1)
                .unwrap_or(0);
            let line_end = content[match_end..]
                .find('\n')
                .map(|p| match_end + p)
                .unwrap_or(content.len());
            let line = &content[line_start..line_end];
            let offset_in_line = match_start - line_start;
            let before_in_line = &line[..offset_in_line];
            let after_in_line = &line[offset_in_line + target_title.len()..];

            let open_brackets = before_in_line.rfind("[[");
            let close_brackets = before_in_line.rfind("]]");
            let is_inside_wikilink = match (open_brackets, close_brackets) {
                (Some(_), None) => after_in_line.contains("]]"),
                (Some(o), Some(c)) if o > c => after_in_line.contains("]]"),
                _ => false,
            };

            if is_inside_wikilink {
                search_from = match_end;
                continue;
            }

            updated_content.push_str(&content[..match_start]);
            updated_content.push_str(&format!("[[{target_title}]]"));
            updated_content.push_str(&content[match_end..]);
            replacement_done = true;
            break;
        }

        if replacement_done {
            let updated_note = service.write_note(source_path, &updated_content)?;
            if let Ok(parsed) = parse_document(&updated_content) {
                let paths = self.note_paths();
                self.link_graph.update_note_links_with_paths(
                    source_path.to_path_buf(),
                    parsed.wikilinks.clone(),
                    &paths,
                );
                if let Some(index_arc) = &self.vault_index {
                    if let Ok(mut idx) = index_arc.lock() {
                        let _ = idx.index_note(&updated_note, &parsed);
                    }
                }
            }

            if let Some(active) = &mut self.active_note {
                if active.relative_path == source_path {
                    active.content = updated_content.clone();
                    self.editor_content = updated_content;
                }
            }

            self.status_message = format!("Linked mention in '{}'", source_path.display());
            self.refresh_entries()?;
        }

        Ok(())
    }

    /// Performs a comprehensive link audit across the vault, detecting broken links and orphan notes.
    pub fn audit_vault(&self) -> Option<LinkAuditReport> {
        let service = self.vault_service.as_ref()?;
        let all_paths: Vec<PathBuf> = self
            .entries
            .iter()
            .filter_map(|e| {
                if let VaultEntry::Note(s) = e {
                    Some(s.relative_path.clone())
                } else {
                    None
                }
            })
            .collect();

        let mut note_contents = HashMap::new();
        for path in &all_paths {
            if let Ok(note) = service.read_note(path) {
                note_contents.insert(path.clone(), note.content);
            }
        }

        Some(
            self.link_graph
                .audit_vault_links(&all_paths, &note_contents),
        )
    }

    /// Resolves a broken link by creating a new note with the target title.
    pub fn fix_broken_link_create_note(&mut self, target_title: &str) -> Result<()> {
        let service = match &self.vault_service {
            Some(s) => s,
            None => return Ok(()),
        };

        let initial_content = format!("# {target_title}\n\n");
        let note = service.create_note(None, target_title, Some(&initial_content))?;
        if let Ok(parsed) = parse_document(&note.content) {
            let paths = self.note_paths();
            self.link_graph.update_note_links_with_paths(
                note.relative_path.clone(),
                parsed.wikilinks.clone(),
                &paths,
            );
            if let Some(index_arc) = &self.vault_index {
                if let Ok(mut idx) = index_arc.lock() {
                    let _ = idx.index_note(&note, &parsed);
                }
            }
        }

        self.refresh_entries()?;
        self.status_message = format!("Created note for broken link: '{target_title}'");
        Ok(())
    }

    /// Safely unlinks references to a broken link target by replacing `[[target]]` or `[[target|display]]` with plain text.
    pub fn fix_broken_link_unlink(&mut self, target_title: &str) -> Result<()> {
        let service = match &self.vault_service {
            Some(s) => s,
            None => return Ok(()),
        };

        let target_lower = target_title.to_lowercase();
        let mut modified_paths = Vec::new();

        for entry in &self.entries {
            if let VaultEntry::Note(summary) = entry {
                let note = match service.read_note(&summary.relative_path) {
                    Ok(n) => n,
                    Err(_) => continue,
                };

                let content = &note.content;
                let mut updated = String::with_capacity(content.len());
                let mut last_idx = 0;
                let mut changed = false;

                while let Some(open_idx) = content[last_idx..].find("[[") {
                    let abs_open = last_idx + open_idx;
                    if let Some(close_idx) = content[abs_open + 2..].find("]]") {
                        let abs_close = abs_open + 2 + close_idx;
                        let inner = &content[abs_open + 2..abs_close];
                        let parts: Vec<&str> = inner.splitn(2, '|').collect();
                        let link_target = parts[0].trim();
                        let display_text = if parts.len() > 1 {
                            parts[1].trim()
                        } else {
                            link_target
                        };

                        if link_target.to_lowercase() == target_lower {
                            updated.push_str(&content[last_idx..abs_open]);
                            updated.push_str(display_text);
                            last_idx = abs_close + 2;
                            changed = true;
                            continue;
                        }

                        updated.push_str(&content[last_idx..abs_close + 2]);
                        last_idx = abs_close + 2;
                    } else {
                        break;
                    }
                }

                if changed {
                    updated.push_str(&content[last_idx..]);
                    modified_paths.push((summary.relative_path.clone(), updated));
                }
            }
        }

        let note_paths = self.note_paths();
        for (path, new_content) in modified_paths {
            let updated_note = service.write_note(&path, &new_content)?;
            if let Ok(parsed) = parse_document(&new_content) {
                self.link_graph.update_note_links_with_paths(
                    path.clone(),
                    parsed.wikilinks.clone(),
                    &note_paths,
                );
                if let Some(index_arc) = &self.vault_index {
                    if let Ok(mut idx) = index_arc.lock() {
                        let _ = idx.index_note(&updated_note, &parsed);
                    }
                }
            }

            if let Some(active) = &mut self.active_note {
                if active.relative_path == path {
                    active.content = new_content.clone();
                    self.editor_content = new_content;
                }
            }
        }

        self.refresh_entries()?;
        self.status_message = format!("Unlinked references to '{target_title}'");
        Ok(())
    }

    /// Soft-deletes multiple orphan notes to the vault trash.
    pub fn batch_trash_orphans(&mut self, paths: &[PathBuf]) -> Result<()> {
        let count = paths.len();
        for path in paths {
            self.trash_note(path)?;
        }
        self.status_message = format!("Moved {count} orphan notes to Trash");
        Ok(())
    }

    /// Queries related notes for the active note combining lexical BM25 and tag Jaccard similarity.
    pub fn get_related_notes_for_active(&self) -> Vec<RelatedNote> {
        let (active_note, index_arc) = match (&self.active_note, &self.vault_index) {
            (Some(n), Some(idx)) => (n, idx),
            _ => return Vec::new(),
        };

        let active_path = active_note
            .relative_path
            .to_string_lossy()
            .replace('\\', "/");
        let parsed = match parse_document(&self.editor_content) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        let index = match index_arc.lock() {
            Ok(idx) => idx,
            Err(_) => return Vec::new(),
        };

        index
            .get_related_notes(
                &active_path,
                &active_note.title,
                &parsed.body,
                &parsed.tags,
                6,
            )
            .unwrap_or_default()
    }

    /// Appends a wikilink to the active note pointing to related_title.
    pub fn append_link_to_active_note(&mut self, related_title: &str) -> Result<()> {
        let link_text = format!("\n\nSee also: [[{related_title}]]\n");
        self.editor_content.push_str(&link_text);
        self.is_dirty = true;
        self.save_active_note()?;
        self.status_message = format!("Linked to [[{related_title}]]");
        Ok(())
    }

    /// Refreshes the BibTeX bibliography from all `.bib` files in the vault.
    pub fn refresh_bib_library(&mut self) {
        if let Some(ref service) = self.vault_service {
            let vault_root = service.vault().root();
            self.bib_library = nodera_core::BibLibrary::from_vault(vault_root);
        }
    }

    /// Inserts a citation key or formatted reference into the active note.
    pub fn insert_citation(&mut self, citekey: &str, full_reference: bool) {
        let clean_key = citekey.trim_start_matches('@');
        let text_to_insert = if full_reference {
            if let Some(entry) = self.bib_library.find_by_key(clean_key) {
                format!("\n\n> {}\n", entry.formatted_reference())
            } else {
                format!(" [@{}]", clean_key)
            }
        } else {
            format!(" [@{}]", clean_key)
        };

        self.editor_content.push_str(&text_to_insert);
        self.is_dirty = true;
        let _ = self.save_active_note();
    }

    /// Extracts annotations from a PDF file into a new Markdown note in the vault.
    pub fn extract_pdf_annotations_to_note(&mut self, pdf_path: &Path) -> Result<PathBuf> {
        let report = nodera_pdf::extract_annotations(pdf_path)?;
        let md = nodera_pdf::annotations_to_markdown(&report);

        let pdf_stem = pdf_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "document".to_string());

        let note_title = format!("{} Annotations", pdf_stem);

        self.create_note(&note_title, None)?;
        self.update_editor_content(md);
        self.save_active_note()?;
        self.refresh_entries()?;

        let rel_path = self
            .active_note
            .as_ref()
            .map(|n| n.relative_path.clone())
            .unwrap_or_default();
        Ok(rel_path)
    }

    /// Parses the active note's YAML frontmatter.
    pub fn get_active_frontmatter(&self) -> Option<nodera_markdown::Frontmatter> {
        nodera_markdown::parse_frontmatter(&self.editor_content)
            .ok()
            .and_then(|(fm, _)| fm)
    }

    /// Updates the active note's frontmatter and saves it to disk.
    pub fn update_active_frontmatter(&mut self, fm: &nodera_markdown::Frontmatter) -> Result<()> {
        let updated = nodera_markdown::inject_or_update_frontmatter(&self.editor_content, fm);
        self.editor_content = updated;
        self.is_dirty = true;
        if let Some(active) = &mut self.active_note {
            active.content = self.editor_content.clone();
        }
        self.save_active_note()?;
        Ok(())
    }

    /// Sets or updates a property in the active note's frontmatter.
    pub fn set_frontmatter_property(&mut self, key: &str, value: serde_yaml::Value) -> Result<()> {
        let mut fm = self.get_active_frontmatter().unwrap_or_default();
        if key == "title" {
            fm.title = value.as_str().map(|s| s.to_string());
        } else if key == "tags" {
            if let Some(seq) = value.as_sequence() {
                fm.tags = seq
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            } else if let Some(s) = value.as_str() {
                fm.tags = s
                    .split(',')
                    .map(str::trim)
                    .filter(|x| !x.is_empty())
                    .map(str::to_string)
                    .collect();
            }
        } else {
            fm.extra.insert(key.to_string(), value);
        }
        self.update_active_frontmatter(&fm)
    }

    /// Removes a property from the active note's frontmatter.
    pub fn remove_frontmatter_property(&mut self, key: &str) -> Result<()> {
        let mut fm = match self.get_active_frontmatter() {
            Some(f) => f,
            None => return Ok(()),
        };
        if key == "title" {
            fm.title = None;
        } else if key == "tags" {
            fm.tags.clear();
        } else {
            fm.extra.remove(key);
        }
        self.update_active_frontmatter(&fm)
    }

    /// Toggles split view mode (Edit & Reading Preview side-by-side).
    pub fn toggle_split(&mut self) {
        if self.split_pane.is_none() {
            self.split_pane = Some(SplitPane {
                relative_path: self.active_note.as_ref().map(|n| n.relative_path.clone()),
                is_reading_mode: true,
                editor_content: self.editor_content.clone(),
            });
            self.status_message = "Split view enabled (Edit & Preview)".to_string();
        } else {
            self.split_pane = None;
            self.status_message = "Split view closed".to_string();
        }
    }

    /// Toggles split direction between Horizontal and Vertical.
    pub fn toggle_split_direction(&mut self) {
        self.split_direction = match self.split_direction {
            SplitDirection::Horizontal => SplitDirection::Vertical,
            SplitDirection::Vertical => SplitDirection::Horizontal,
        };
    }

    /// Closes active split view.
    pub fn close_split(&mut self) {
        self.split_pane = None;
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
                palette::SWITCH_TO_EDITOR.0,
                palette::SWITCH_TO_EDITOR.1,
                PaletteAction::SwitchView(ActiveView::Editor),
            ),
            (
                palette::SWITCH_TO_TASKS.0,
                palette::SWITCH_TO_TASKS.1,
                PaletteAction::SwitchView(ActiveView::Tasks),
            ),
            (
                palette::SWITCH_TO_LIBRARY.0,
                palette::SWITCH_TO_LIBRARY.1,
                PaletteAction::SwitchView(ActiveView::Library),
            ),
            (
                palette::SWITCH_TO_GRAPH.0,
                palette::SWITCH_TO_GRAPH.1,
                PaletteAction::SwitchView(ActiveView::Graph),
            ),
            (
                palette::TOGGLE_READING.0,
                palette::TOGGLE_READING.1,
                PaletteAction::ToggleReadingMode,
            ),
            (
                palette::CREATE_NOTE.0,
                palette::CREATE_NOTE.1,
                PaletteAction::CreateNote,
            ),
            (
                palette::IMPORT_PDF.0,
                palette::IMPORT_PDF.1,
                PaletteAction::ImportPdf,
            ),
            (
                palette::OPEN_SETTINGS.0,
                palette::OPEN_SETTINGS.1,
                PaletteAction::OpenSettings,
            ),
            (
                palette::RESET_LAYOUT.0,
                palette::RESET_LAYOUT.1,
                PaletteAction::ResetLayout,
            ),
            (
                palette::TOGGLE_THEME.0,
                palette::TOGGLE_THEME.1,
                PaletteAction::ToggleTheme,
            ),
            (
                palette::INSERT_TEMPLATE.0,
                palette::INSERT_TEMPLATE.1,
                PaletteAction::InsertTemplate,
            ),
            (
                "Open Today's Daily Note",
                "Create or jump to today's daily journal note",
                PaletteAction::OpenDailyNote,
            ),
            (
                "Insert Citation",
                "Search and insert BibTeX / Zotero citations (Ctrl+Shift+C)",
                PaletteAction::OpenCitationPicker,
            ),
            (
                "Extract PDF Annotations",
                "Extract highlights and comments from PDF to Markdown note (Ctrl+Shift+E)",
                PaletteAction::ExtractPdfAnnotations,
            ),
            (
                palette::REBUILD_INDEX.0,
                palette::REBUILD_INDEX.1,
                PaletteAction::RebuildIndex,
            ),
            (
                "Quick Capture (Rough Note)",
                "Capture fleeting thoughts directly to Inbox (Ctrl+Shift+Q)",
                PaletteAction::OpenQuickCapture,
            ),
            (
                "New Permanent Note",
                "Create an atomic permanent note with tags and links",
                PaletteAction::CreateTypedNote {
                    note_type: NOTE_TYPE_PERMANENT.to_string(),
                    sub_type: None,
                },
            ),
            (
                "New Rough Note",
                "Create a rough capture note for later review",
                PaletteAction::CreateTypedNote {
                    note_type: NOTE_TYPE_ROUGH.to_string(),
                    sub_type: None,
                },
            ),
            (
                "New Source Note (Book)",
                "Create a literature source note for a book",
                PaletteAction::CreateTypedNote {
                    note_type: NOTE_TYPE_SOURCE.to_string(),
                    sub_type: Some(SOURCE_TYPE_BOOK.to_string()),
                },
            ),
            (
                "New Source Note (Video)",
                "Create a source note with key takeaways and timestamps",
                PaletteAction::CreateTypedNote {
                    note_type: NOTE_TYPE_SOURCE.to_string(),
                    sub_type: Some(SOURCE_TYPE_VIDEO.to_string()),
                },
            ),
            (
                "New Index / MOC Note",
                "Create a Map of Content note to curate and connect ideas",
                PaletteAction::CreateTypedNote {
                    note_type: NOTE_TYPE_INDEX.to_string(),
                    sub_type: None,
                },
            ),
            (
                "Promote Active Note to Permanent",
                "Promote current rough note to permanent without moving files or breaking links",
                PaletteAction::PromoteActiveNoteToPermanent,
            ),
            (
                "Open Review Queue",
                "Triage rough notes, unlinked thoughts, and stale sources",
                PaletteAction::SwitchView(ActiveView::ReviewQueue),
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

        for cmd in self.plugin_manager.get_commands() {
            if q.is_empty()
                || cmd.name.to_lowercase().contains(&q)
                || cmd.description.to_lowercase().contains(&q)
            {
                items.push(CommandPaletteItem {
                    title: cmd.name,
                    description: cmd.description,
                    action: PaletteAction::ExecutePluginCommand(cmd.command_id),
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
            PaletteAction::OpenDailyNote => {
                self.open_or_create_daily_note()?;
            }
            PaletteAction::InsertTemplate => {
                self.show_template_modal = true;
                self.template_search_query.clear();
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
            PaletteAction::ToggleReadingMode => {
                self.is_reading_mode = !self.is_reading_mode;
            }
            PaletteAction::OpenSettings => {
                self.open_settings();
            }
            PaletteAction::ResetLayout => {
                self.reset_layout();
            }
            PaletteAction::RebuildIndex => {
                self.rebuild_vault_index()?;
            }
            PaletteAction::OpenCitationPicker => {
                self.show_citation_picker_modal = true;
            }
            PaletteAction::ExtractPdfAnnotations => {
                self.show_pdf_annotation_modal = true;
            }
            PaletteAction::ExecutePluginCommand(cmd_id) => {
                let res =
                    self.plugin_manager
                        .dispatch_hook(&nodera_core::PluginHook::ExecuteCommand {
                            command_id: &cmd_id,
                            args: &[],
                        });
                if let Some(output) = res.command_output {
                    self.status_message = format!("Plugin: {}", output);
                } else {
                    self.status_message = format!("Plugin: executed '{}'", cmd_id);
                }
            }
            PaletteAction::CreateTypedNote {
                note_type,
                sub_type,
            } => {
                self.create_typed_note(&note_type, "", sub_type.as_deref())?;
            }
            PaletteAction::OpenQuickCapture => {
                self.open_quick_capture();
            }
            PaletteAction::PromoteActiveNoteToPermanent => {
                self.promote_active_note_to_permanent()?;
            }
        }
        Ok(())
    }

    /// Returns list of notes that have Wikilinks pointing to the currently active note.
    pub fn get_current_backlinks(&self) -> Vec<PathBuf> {
        if let Some(active) = &self.active_note {
            let note_paths = self.note_paths();
            self.link_graph
                .get_backlinks(&active.relative_path, &note_paths)
        } else {
            Vec::new()
        }
    }

    /// Returns list of outgoing Wikilinks in current editor content and whether they resolve to an existing note.
    pub fn get_current_outgoing_links(&self) -> Vec<(nodera_markdown::Wikilink, Option<PathBuf>)> {
        let note_paths = self.note_paths();
        let resolver = nodera_markdown::TargetResolver::from_paths(&note_paths);
        let links = nodera_markdown::extract_wikilinks(&self.editor_content);
        links
            .into_iter()
            .map(|l| {
                let resolved = resolver.resolve(&l.target);
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

    /// Returns knowledge graph data for the entire vault configured by the provided GraphSettings.
    pub fn get_full_graph_data_with_settings(
        &self,
        settings: &GraphSettings,
    ) -> nodera_markdown::GraphData {
        let mut note_paths = Vec::new();
        let mut titles = std::collections::HashMap::new();
        let mut note_tags = std::collections::HashMap::new();

        for e in &self.entries {
            if let VaultEntry::Note(s) = e {
                note_paths.push(s.relative_path.clone());
                titles.insert(s.relative_path.clone(), s.title.clone());
            }
        }

        // If tag nodes are requested, extract tags from index
        if settings.filters.tags {
            if let Some(index_arc) = &self.vault_index {
                if let Ok(idx) = index_arc.lock() {
                    if let Ok(tags_map) = idx.query_all_note_tags() {
                        note_tags = tags_map;
                    }
                }
            }
        }

        let filter_options = nodera_markdown::GraphFilterOptions {
            existing_files_only: settings.filters.existing_files_only,
            orphans: settings.filters.orphans,
            tags: settings.filters.tags,
            attachments: settings.filters.attachments,
        };

        let mut data = self.link_graph.to_graph_data_with_options(
            &note_paths,
            &titles,
            &note_tags,
            &filter_options,
        );

        // If search query is non-empty, filter matching nodes
        let query = settings.filters.search_query.trim().to_lowercase();
        if !query.is_empty() {
            let matching_ids: std::collections::HashSet<String> = data
                .nodes
                .iter()
                .filter(|n| {
                    n.label.to_lowercase().contains(&query) || n.id.to_lowercase().contains(&query)
                })
                .map(|n| n.id.clone())
                .collect();

            data.nodes.retain(|n| matching_ids.contains(&n.id));
            data.edges
                .retain(|e| matching_ids.contains(&e.source) && matching_ids.contains(&e.target));
        }

        data
    }

    /// Returns knowledge graph data for the entire vault using saved preferences.
    pub fn get_full_graph_data(&self) -> nodera_markdown::GraphData {
        self.get_full_graph_data_with_settings(&self.preferences.graph_settings)
    }

    /// Returns local knowledge graph data centered on the currently active note configured by GraphSettings.
    pub fn get_local_graph_data_with_settings(
        &self,
        depth: usize,
        settings: &GraphSettings,
    ) -> nodera_markdown::GraphData {
        if let Some(active) = &self.active_note {
            let mut note_paths = Vec::new();
            let mut titles = std::collections::HashMap::new();
            let note_tags = std::collections::HashMap::new();

            for e in &self.entries {
                if let VaultEntry::Note(s) = e {
                    note_paths.push(s.relative_path.clone());
                    titles.insert(s.relative_path.clone(), s.title.clone());
                }
            }

            let filter_options = nodera_markdown::GraphFilterOptions {
                existing_files_only: settings.filters.existing_files_only,
                orphans: settings.filters.orphans,
                tags: settings.filters.tags,
                attachments: settings.filters.attachments,
            };

            self.link_graph.to_local_graph_data_with_options(
                &active.relative_path,
                &note_paths,
                &titles,
                &note_tags,
                &filter_options,
                depth,
            )
        } else {
            nodera_markdown::GraphData::default()
        }
    }

    /// Returns local knowledge graph data centered on the currently active note with custom depth.
    pub fn get_local_graph_data(&self, depth: usize) -> nodera_markdown::GraphData {
        self.get_local_graph_data_with_settings(depth, &self.preferences.graph_settings)
    }

    /// Opens an existing note or creates a new one for a Wikilink target.
    pub fn open_or_create_target(&mut self, target: &str) -> Result<()> {
        let note_paths = self.note_paths();

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

    /// Checks whether a note with the given title exists in the active vault.
    pub fn note_title_exists(&self, title: &str) -> bool {
        let trimmed = title.trim().to_lowercase();
        for entry in &self.entries {
            if let VaultEntry::Note(summary) = entry {
                if summary.title.to_lowercase() == trimmed {
                    return true;
                }
                if let Some(stem) = summary.relative_path.file_stem().and_then(|s| s.to_str()) {
                    if stem.to_lowercase() == trimmed {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Computes the next non-conflicting title for a new note ("Untitled", "Untitled 1", "Untitled 2", etc.).
    pub fn next_available_note_title(&self) -> String {
        let base = "Untitled";
        if !self.note_title_exists(base) {
            return base.to_string();
        }
        let mut i = 1;
        loop {
            let candidate = format!("{base} {i}");
            if !self.note_title_exists(&candidate) {
                return candidate;
            }
            i += 1;
        }
    }

    /// Creates a new note with specified initial content, saves it to disk, and opens it in the editor.
    pub fn create_note_with_content(
        &mut self,
        title: &str,
        folder: Option<&str>,
        content: &str,
    ) -> Result<()> {
        let resolved_title = if title.trim().is_empty() {
            self.next_available_note_title()
        } else {
            title.trim().to_string()
        };

        if let Some(service) = &self.vault_service {
            let note = service.create_note(folder, &resolved_title, Some(content))?;
            self.editor_content = content.to_string();
            let rel = note.relative_path.clone();
            let title_str = note.title.clone();
            self.active_note = Some(note.clone());
            self.active_view = ActiveView::Editor;
            self.is_reading_mode = false;
            self.is_dirty = false;
            self.status_message = format!("Created '{resolved_title}'");

            // Tab management
            if let Some(idx) = self.open_tabs.iter().position(|t| t.relative_path == rel) {
                self.active_tab_index = Some(idx);
            } else {
                self.open_tabs.push(OpenTab {
                    relative_path: rel.clone(),
                    title: title_str,
                    is_pinned: false,
                });
                self.active_tab_index = Some(self.open_tabs.len() - 1);
            }

            // Navigation history
            if self.nav_history.get(self.nav_history_index) != Some(&rel) {
                if !self.nav_history.is_empty()
                    && self.nav_history_index + 1 < self.nav_history.len()
                {
                    self.nav_history.truncate(self.nav_history_index + 1);
                }
                self.nav_history.push(rel.clone());
                self.nav_history_index = self.nav_history.len() - 1;
            }

            self.record_recent_note(&rel);
            self.refresh_entries()?;

            // Index new note
            if let Some(index_arc) = &self.vault_index {
                if let Ok(mut idx) = index_arc.lock() {
                    if let Ok(parsed) = parse_document(content) {
                        let _ = idx.index_note(&note, &parsed);
                    }
                }
            }

            debug!(path = %rel.display(), "Created and opened note in state");
        }
        Ok(())
    }

    /// Creates a new note, saves it to disk, and opens it in the editor.
    pub fn create_note(&mut self, title: &str, folder: Option<&str>) -> Result<()> {
        self.create_note_with_content(title, folder, "")
    }

    /// Creates a note of a specific workflow type with standard frontmatter and structure.
    pub fn create_typed_note(
        &mut self,
        note_type: &str,
        title: &str,
        sub_type: Option<&str>,
    ) -> Result<()> {
        let now = chrono::Local::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let resolved_title = if title.trim().is_empty() {
            match note_type {
                NOTE_TYPE_ROUGH => format!("Rough Note {}", now.format("%Y-%m-%d %H%M")),
                NOTE_TYPE_PERMANENT => format!("Permanent {}", now.format("%Y-%m-%d")),
                NOTE_TYPE_SOURCE => format!("Source {}", now.format("%Y-%m-%d")),
                NOTE_TYPE_INDEX => "Master Index".to_string(),
                NOTE_TYPE_PROJECT => format!("Project {}", now.format("%Y-%m-%d")),
                NOTE_TYPE_MEETING => format!("Meeting {}", now.format("%Y-%m-%d")),
                _ => self.next_available_note_title(),
            }
        } else {
            title.trim().to_string()
        };

        // Determine target folder if present in vault
        let mut target_folder: Option<&str> = None;
        let folder_candidates: &[&str] = match note_type {
            NOTE_TYPE_ROUGH => &["00 Inbox", "Inbox"],
            NOTE_TYPE_SOURCE => &["02 Literature", "01 Sources", "Literature", "Sources", "Books"],
            NOTE_TYPE_PERMANENT => &["03 Permanent", "02 Notes", "Notes", "Permanent"],
            NOTE_TYPE_INDEX => &["04 Index", "Index", "Notes"],
            NOTE_TYPE_PROJECT => &["01 Projects", "Projects"],
            _ => &[],
        };

        for candidate in folder_candidates {
            if self.entries.iter().any(|e| match e {
                VaultEntry::Folder { name, .. } => name.eq_ignore_ascii_case(candidate),
                _ => false,
            }) {
                target_folder = Some(*candidate);
                break;
            }
        }

        let content = match note_type {
            NOTE_TYPE_ROUGH => rough_note_template(&resolved_title, "", &date_str),
            NOTE_TYPE_PERMANENT => permanent_note_template(&resolved_title, "", &date_str),
            NOTE_TYPE_SOURCE => {
                let st = sub_type.unwrap_or(SOURCE_TYPE_BOOK);
                if st == SOURCE_TYPE_VIDEO {
                    video_source_template(
                        &resolved_title,
                        "YouTube Channel",
                        "https://...",
                        &date_str,
                    )
                } else {
                    source_note_template(
                        &resolved_title,
                        st,
                        "Author Name",
                        "https://...",
                        &date_str,
                    )
                }
            }
            NOTE_TYPE_INDEX => index_note_template(&resolved_title, &resolved_title),
            NOTE_TYPE_PROJECT => project_note_template(&resolved_title, &date_str),
            NOTE_TYPE_MEETING => meeting_note_template(&resolved_title, &date_str),
            _ => format!(
                "---\ntitle: \"{resolved_title}\"\ntype: {note_type}\ncreated: \"{date_str}\"\n---\n# {resolved_title}\n\n"
            ),
        };

        self.create_note_with_content(&resolved_title, target_folder, &content)
    }

    /// Opens the Quick Capture modal dialog.
    pub fn open_quick_capture(&mut self) {
        self.show_quick_capture = true;
        self.quick_capture_title.clear();
        self.quick_capture_body.clear();
        self.quick_capture_tags.clear();
    }

    /// Closes the Quick Capture modal dialog.
    pub fn close_quick_capture(&mut self) {
        self.show_quick_capture = false;
        self.quick_capture_title.clear();
        self.quick_capture_body.clear();
        self.quick_capture_tags.clear();
    }

    /// Executes saving the captured rough note with optional open in editor.
    pub fn execute_quick_capture(&mut self, open_after: bool) -> Result<()> {
        let now = chrono::Local::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let title = if self.quick_capture_title.trim().is_empty() {
            format!("Thought {}", now.format("%Y-%m-%d %H%M%S"))
        } else {
            self.quick_capture_title.trim().to_string()
        };

        let body = self.quick_capture_body.trim().to_string();

        let mut target_folder: Option<&str> = None;
        for candidate in &["00 Inbox", "Inbox"] {
            if self.entries.iter().any(|e| match e {
                VaultEntry::Folder { name, .. } => name.eq_ignore_ascii_case(candidate),
                _ => false,
            }) {
                target_folder = Some(*candidate);
                break;
            }
        }

        let mut content = rough_note_template(&title, &body, &date_str);

        // Inject custom tags if provided
        if !self.quick_capture_tags.trim().is_empty() {
            if let Ok((Some(mut fm), body_part)) = parse_frontmatter(&content) {
                for t in self.quick_capture_tags.split(',').map(str::trim) {
                    let clean = t.trim_start_matches('#');
                    if !clean.is_empty() && !fm.tags.contains(&clean.to_string()) {
                        fm.tags.push(clean.to_string());
                    }
                }
                content = inject_or_update_frontmatter(body_part, &fm);
            }
        }

        self.close_quick_capture();

        if open_after {
            self.create_note_with_content(&title, target_folder, &content)?;
        } else if let Some(service) = &self.vault_service {
            let note = service.create_note(target_folder, &title, Some(&content))?;
            self.entries = service.list_entries()?;
            if let Some(index_arc) = &self.vault_index {
                if let Ok(mut idx) = index_arc.lock() {
                    if let Ok(parsed) = parse_document(&content) {
                        let _ = idx.index_note(&note, &parsed);
                    }
                }
            }
            self.status_message = format!("Captured rough note '{title}' to Inbox");
        }
        Ok(())
    }

    /// Safely promotes the active note from a rough/untyped note to a permanent note.
    /// Invariant: NEVER renames or moves the file, strictly preserving 100% of vault wikilinks.
    pub fn promote_active_note_to_permanent(&mut self) -> Result<()> {
        let (existing_fm, body) =
            parse_frontmatter(&self.editor_content).unwrap_or((None, &self.editor_content));
        let mut fm = existing_fm.unwrap_or_default();

        if fm.title.is_none() {
            if let Some(note) = &self.active_note {
                fm.title = Some(note.title.clone());
            }
        }

        fm.set_note_type(NOTE_TYPE_PERMANENT);
        if !fm.tags.contains(&"permanent".to_string()) {
            fm.tags.push("permanent".to_string());
        }
        // Remove rough/inbox tags if present
        fm.tags.retain(|t| t != "rough" && t != "inbox");

        let updated = inject_or_update_frontmatter(body, &fm);
        self.editor_content = updated;
        self.is_dirty = true;
        self.save_active_note()?;
        self.status_message = "Promoted note to Permanent (links & path preserved)".to_string();
        Ok(())
    }

    /// Marks a note as reviewed by adding `reviewed: YYYY-MM-DD` to its frontmatter.
    pub fn mark_note_reviewed(&mut self, rel_path: &Path) -> Result<()> {
        let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
        if let Some(service) = &self.vault_service {
            let note = service.read_note(rel_path)?;
            let (existing_fm, body) =
                parse_frontmatter(&note.content).unwrap_or((None, &note.content));
            let mut fm = existing_fm.unwrap_or_default();
            fm.mark_reviewed(&date_str);
            let updated = inject_or_update_frontmatter(body, &fm);
            let updated_note = service.write_note(rel_path, &updated)?;

            // If active note is the one being marked, update editor content too
            if let Some(active) = &self.active_note {
                if active.relative_path == rel_path {
                    self.editor_content = updated.clone();
                    self.is_dirty = false;
                }
            }

            if let Some(index_arc) = &self.vault_index {
                if let Ok(mut idx) = index_arc.lock() {
                    if let Ok(parsed) = parse_document(&updated) {
                        let _ = idx.index_note(&updated_note, &parsed);
                    }
                }
            }
            self.status_message = format!("Marked '{}' as reviewed", note.title);
        }
        Ok(())
    }

    /// Queries the review queue items for the current filter category.
    pub fn get_review_queue_items(&self) -> Vec<ReviewQueueItem> {
        if let Some(index_arc) = &self.vault_index {
            if let Ok(idx) = index_arc.lock() {
                match self.review_queue_filter {
                    ReviewCategory::RoughNote => idx.query_rough_notes().unwrap_or_default(),
                    ReviewCategory::Unlinked => idx.query_unlinked_notes().unwrap_or_default(),
                    ReviewCategory::StaleSource => {
                        let now_ns = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_nanos() as u64)
                            .unwrap_or(0);
                        let fourteen_days_ns = 14 * 24 * 3600 * 1_000_000_000u64;
                        let cutoff_ns = now_ns.saturating_sub(fourteen_days_ns);
                        idx.query_stale_sources(cutoff_ns).unwrap_or_default()
                    }
                    ReviewCategory::Orphan => idx.query_orphan_notes().unwrap_or_default(),
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// Queries vault-wide knowledge health metrics.
    pub fn get_knowledge_stats(&self) -> KnowledgeStats {
        if let Some(index_arc) = &self.vault_index {
            if let Ok(idx) = index_arc.lock() {
                idx.query_knowledge_stats().unwrap_or_default()
            } else {
                KnowledgeStats::default()
            }
        } else {
            KnowledgeStats::default()
        }
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
            let old_buf = rel.to_path_buf();
            for tab in &mut self.open_tabs {
                if tab.relative_path == old_buf {
                    tab.relative_path = renamed.relative_path.clone();
                    tab.title = renamed.title.clone();
                }
            }
            for p in &mut self.nav_history {
                if *p == old_buf {
                    *p = renamed.relative_path.clone();
                }
            }
            for b in &mut self.preferences.bookmarks {
                if *b == old_buf {
                    *b = renamed.relative_path.clone();
                }
            }
            for r in &mut self.preferences.recent_notes {
                if *r == old_buf {
                    *r = renamed.relative_path.clone();
                }
            }
            self.preferences.save();
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

    /// Appends or inserts a Wikilink snippet into the active editor note.
    pub fn insert_wikilink_snippet(&mut self, target: &str) {
        let snippet = format!("[[{target}]]");
        if self.editor_content.ends_with('\n') || self.editor_content.is_empty() {
            self.editor_content.push_str(&snippet);
        } else {
            self.editor_content.push(' ');
            self.editor_content.push_str(&snippet);
        }
        self.is_dirty = true;
    }

    /// Appends or inserts an embed snippet into the active editor note.
    pub fn insert_embed_snippet(&mut self, target: &str) {
        let snippet = format!("![[{target}]]");
        if self.editor_content.ends_with('\n') || self.editor_content.is_empty() {
            self.editor_content.push_str(&snippet);
        } else {
            self.editor_content.push(' ');
            self.editor_content.push_str(&snippet);
        }
        self.is_dirty = true;
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
            self.is_reading_mode = false;
            let _ = self.select_note(&rel_path);
        }
    }

    /// Records an error during PDF import.
    pub fn set_pdf_error(&mut self, error: String) {
        self.pdf_error = Some(error);
        self.pdf_cancellation = None;
        self.pdf_progress = None;
    }

    /// Sets the sidebar width, clamping between 180 and 500 px, and persists it.
    pub fn set_sidebar_width(&mut self, width: u32) {
        let clamped = width.clamp(180, 500);
        self.sidebar_width = clamped;
        self.preferences.sidebar_width = clamped;
        self.preferences.save();
    }

    /// Sets the context panel width, clamping between 200 and 500 px, and persists it.
    pub fn set_context_panel_width(&mut self, width: u32) {
        let clamped = width.clamp(200, 500);
        self.context_panel_width = clamped;
        self.preferences.context_panel_width = clamped;
        self.preferences.save();
    }

    /// Resets sidebar and context panel widths to defaults.
    pub fn reset_layout(&mut self) {
        self.sidebar_width = 260;
        self.context_panel_width = 280;
        self.sidebar_open = true;
        self.context_panel_open = true;
        self.preferences.sidebar_width = 260;
        self.preferences.context_panel_width = 280;
        self.preferences.save();
        self.status_message = "Layout reset to default".to_string();
    }

    /// Cycles through active view modes: Editor -> Tasks -> ReviewQueue -> Library -> Graph -> Editor.
    pub fn cycle_view(&mut self) {
        self.active_view = match self.active_view {
            ActiveView::Editor => ActiveView::Tasks,
            ActiveView::Tasks => ActiveView::ReviewQueue,
            ActiveView::ReviewQueue => ActiveView::Library,
            ActiveView::Library => ActiveView::Graph,
            ActiveView::Graph => ActiveView::Editor,
        };
    }

    /// Opens the Settings modal.
    pub fn open_settings(&mut self) {
        self.show_settings_modal = true;
    }

    /// Closes the Settings modal and persists preferences.
    pub fn close_settings(&mut self) {
        self.show_settings_modal = false;
        self.preferences.save();
    }

    /// Displays an actionable error dialog.
    pub fn show_error(&mut self, title: &str, message: &str, details: Option<String>) {
        self.show_error_dialog = true;
        self.error_dialog_title = title.to_string();
        self.error_dialog_message = message.to_string();
        self.error_dialog_details = details;
    }

    /// Dismisses the error dialog.
    pub fn clear_error(&mut self) {
        self.show_error_dialog = false;
        self.error_dialog_title.clear();
        self.error_dialog_message.clear();
        self.error_dialog_details = None;
    }

    /// Updates the Table of Contents headings extracted from current note content.
    pub fn update_toc_headings(&mut self) {
        let mut headings = Vec::new();
        for line in self.editor_content.lines() {
            let trimmed = line.trim();
            if let Some(h) = trimmed.strip_prefix("# ") {
                headings.push((1, h.trim().to_string()));
            } else if let Some(h) = trimmed.strip_prefix("## ") {
                headings.push((2, h.trim().to_string()));
            } else if let Some(h) = trimmed.strip_prefix("### ") {
                headings.push((3, h.trim().to_string()));
            }
        }
        self.toc_headings = headings;
    }

    /// Sets the reading progress percentage for a note and persists it.
    pub fn set_reading_progress(&mut self, rel_path: &Path, pct: u32) {
        let key = rel_path.to_string_lossy().to_string();
        self.preferences
            .reading_progress
            .insert(key, pct.clamp(0, 100));
        self.preferences.save();
    }

    /// Discovers all books and long-form documents in the vault (inside `Books/` or tagged `#book`/`#pdf-import`).
    pub fn get_library_books(&self) -> Vec<LibraryBook> {
        let mut books = Vec::new();
        let query = self.library_search_query.trim().to_lowercase();

        for entry in &self.entries {
            if let VaultEntry::Note(summary) = entry {
                let rel_str = summary.relative_path.to_string_lossy();
                let is_in_books = rel_str.starts_with("Books/") || rel_str.starts_with("Books\\");

                // Read note details if service is available
                let (is_book, pages, chapters, source, tags, word_cnt, char_cnt) =
                    if let Some(service) = &self.vault_service {
                        if let Ok(note) = service.read_note(&summary.relative_path) {
                            let word_count = note.content.split_whitespace().count();
                            let char_count = note.content.len();

                            if let Ok(doc) = parse_document(&note.content) {
                                let mut note_tags = doc.tags;
                                let is_tagged_book =
                                    note_tags.iter().any(|t| t == "book" || t == "pdf-import");

                                let (pages, chapters, source) = if let Some(fm) = doc.frontmatter {
                                    let pg =
                                        fm.extra.get("pages").and_then(|v| v.as_u64()).unwrap_or(0)
                                            as usize;
                                    let ch = fm
                                        .extra
                                        .get("chapters")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0)
                                        as usize;
                                    let src = fm
                                        .extra
                                        .get("source")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string());
                                    for t in &fm.tags {
                                        if !note_tags.contains(t) {
                                            note_tags.push(t.clone());
                                        }
                                    }
                                    (pg, ch, src)
                                } else {
                                    (0, 0, None)
                                };

                                (
                                    is_in_books || is_tagged_book || source.is_some(),
                                    pages,
                                    chapters,
                                    source,
                                    note_tags,
                                    word_count,
                                    char_count,
                                )
                            } else {
                                (is_in_books, 0, 0, None, Vec::new(), word_count, char_count)
                            }
                        } else {
                            (is_in_books, 0, 0, None, Vec::new(), 0, 0)
                        }
                    } else {
                        (is_in_books, 0, 0, None, Vec::new(), 0, 0)
                    };

                if is_book {
                    if !query.is_empty()
                        && !summary.title.to_lowercase().contains(&query)
                        && !tags.iter().any(|t| t.to_lowercase().contains(&query))
                    {
                        continue;
                    }

                    let progress_pct = self
                        .preferences
                        .reading_progress
                        .get(&summary.relative_path.to_string_lossy().to_string())
                        .copied()
                        .unwrap_or(0);
                    books.push(LibraryBook {
                        relative_path: summary.relative_path.clone(),
                        title: summary.title.clone(),
                        source,
                        pages,
                        chapters,
                        tags,
                        word_count: word_cnt,
                        char_count: char_cnt,
                        reading_progress_pct: progress_pct,
                    });
                }
            }
        }
        books
    }

    /// Opens a book in reading mode with table of contents extracted.
    pub fn open_book_in_reader(&mut self, rel_path: &Path) -> Result<()> {
        self.select_note(rel_path)?;
        self.active_view = ActiveView::Editor;
        self.is_reading_mode = true;
        self.update_toc_headings();
        Ok(())
    }

    /// Opens a book in standard editor mode.
    pub fn open_book_in_editor(&mut self, rel_path: &Path) -> Result<()> {
        self.select_note(rel_path)?;
        self.active_view = ActiveView::Editor;
        self.is_reading_mode = false;
        Ok(())
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

    #[test]
    fn test_multi_tab_workflow() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("TabsVault");

        let mut state = AppState::default();
        state
            .create_vault(&vault_path, Some("Tabs Vault".to_string()))
            .unwrap();

        // 1. Create three notes and verify tabs are opened
        state.create_note("Alpha", None).unwrap();
        state.create_note("Beta", None).unwrap();
        state.create_note("Gamma", None).unwrap();

        assert_eq!(state.open_tabs.len(), 3);
        assert_eq!(state.active_tab_index, Some(2));
        assert_eq!(state.open_tabs[0].title, "Alpha");
        assert_eq!(state.open_tabs[1].title, "Beta");
        assert_eq!(state.open_tabs[2].title, "Gamma");

        // 2. Pin Tab 0 (Alpha)
        state.toggle_pin_tab(0);
        assert!(state.open_tabs[0].is_pinned);

        // 3. Select Tab 0
        state.select_tab(0).unwrap();
        assert_eq!(state.active_tab_index, Some(0));
        assert_eq!(state.active_note.as_ref().unwrap().title, "Alpha");

        // 4. Close Tab 1 (Beta)
        state.close_tab(1).unwrap();
        assert_eq!(state.open_tabs.len(), 2);
        assert_eq!(state.open_tabs[0].title, "Alpha");
        assert_eq!(state.open_tabs[1].title, "Gamma");
        assert_eq!(state.active_tab_index, Some(0));

        // 5. Select Gamma (now at index 1)
        state.select_tab(1).unwrap();
        assert_eq!(state.active_note.as_ref().unwrap().title, "Gamma");

        // 6. Close all tabs (pinned tab Alpha should remain)
        state.close_all_tabs().unwrap();
        assert_eq!(state.open_tabs.len(), 1);
        assert_eq!(state.open_tabs[0].title, "Alpha");
        assert_eq!(state.active_tab_index, Some(0));
        assert_eq!(state.active_note.as_ref().unwrap().title, "Alpha");
    }

    #[test]
    fn test_navigation_history_and_daily_notes() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("HistoryVault");

        let mut state = AppState::default();
        state
            .create_vault(&vault_path, Some("History Vault".to_string()))
            .unwrap();

        state.create_note("Page A", None).unwrap();
        state.create_note("Page B", None).unwrap();
        state.create_note("Page C", None).unwrap();

        // Check history
        assert!(state.can_navigate_back());
        assert!(!state.can_navigate_forward());

        // Navigate back to Page B
        state.navigate_back().unwrap();
        assert_eq!(state.active_note.as_ref().unwrap().title, "Page B");
        assert!(state.can_navigate_forward());

        // Navigate back to Page A
        state.navigate_back().unwrap();
        assert_eq!(state.active_note.as_ref().unwrap().title, "Page A");

        // Navigate forward to Page B
        state.navigate_forward().unwrap();
        assert_eq!(state.active_note.as_ref().unwrap().title, "Page B");

        // Test Daily Note creation
        state.open_or_create_daily_note().unwrap();
        let active = state.active_note.as_ref().unwrap();
        assert!(active.relative_path.to_string_lossy().starts_with("Daily"));
        assert!(state.editor_content.contains("Daily Note"));
        assert!(state.editor_content.contains("## Tasks"));
    }

    #[test]
    fn test_note_templates_workflow() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("TemplateVault");

        let mut state = AppState::default();
        state
            .create_vault(&vault_path, Some("Template Vault".to_string()))
            .unwrap();

        // 1. Built-in templates exist
        let templates = state.get_available_templates();
        assert!(templates.len() >= 4);
        assert!(templates.iter().any(|t| t.name == "Daily Journal"));
        assert!(templates.iter().any(|t| t.name == "Meeting Notes"));

        // 2. Custom vault template in Templates/ folder
        let templates_dir = vault_path.join("Templates");
        std::fs::create_dir_all(&templates_dir).unwrap();
        let custom_tmpl_content =
            "# Bug Report: {{title}}\n**Reported**: {{date}}\n\n## Reproduction Steps\n1. ";
        std::fs::write(templates_dir.join("Bug Report.md"), custom_tmpl_content).unwrap();

        let updated_templates = state.get_available_templates();
        assert!(updated_templates.iter().any(|t| t.name == "Bug Report"));

        // 3. Insert template into active note
        state.create_note("Issue 42", None).unwrap();
        state
            .insert_template(Some("Bug Report"), custom_tmpl_content)
            .unwrap();

        let current_date = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert!(state.editor_content.contains("# Bug Report: Issue 42"));
        assert!(state
            .editor_content
            .contains(&format!("**Reported**: {current_date}")));
        assert!(state.editor_content.contains("## Reproduction Steps"));

        // Verify disk content
        let active_rel = state.active_note.as_ref().unwrap().relative_path.clone();
        let reloaded_note = state
            .vault_service
            .as_ref()
            .unwrap()
            .read_note(&active_rel)
            .unwrap();
        assert_eq!(reloaded_note.content, state.editor_content);
    }

    #[test]
    fn test_bookmarks_and_recent_notes() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("BookmarksVault");

        let mut state = AppState::default();
        state
            .create_vault(&vault_path, Some("Bookmarks Vault".to_string()))
            .unwrap();

        // 1. Create two notes
        state.create_note("Alpha", None).unwrap();
        state.create_note("Beta", None).unwrap();

        let alpha_path = PathBuf::from("Notes").join("Alpha.md");
        let beta_path = PathBuf::from("Notes").join("Beta.md");

        // 2. Verify recent notes order (most recent first)
        assert_eq!(state.preferences.recent_notes.first(), Some(&beta_path));

        // Select Alpha and verify it moves to front of recent notes
        state.select_note(&alpha_path).unwrap();
        assert_eq!(state.preferences.recent_notes.first(), Some(&alpha_path));

        // 3. Test Bookmarks toggle
        assert!(!state.is_bookmarked(&alpha_path));
        state.toggle_bookmark(&alpha_path);
        assert!(state.is_bookmarked(&alpha_path));
        assert_eq!(state.preferences.bookmarks, vec![alpha_path.clone()]);

        // Toggle again to remove
        state.toggle_bookmark(&alpha_path);
        assert!(!state.is_bookmarked(&alpha_path));
        assert!(state.preferences.bookmarks.is_empty());

        // Re-add bookmark
        state.toggle_bookmark(&alpha_path);
        assert!(state.is_bookmarked(&alpha_path));

        // 4. Test Rename Note updating bookmarks & recent
        state.rename_note(&alpha_path, "AlphaRenamed").unwrap();
        let renamed_path = PathBuf::from("Notes").join("AlphaRenamed.md");
        assert!(state.is_bookmarked(&renamed_path));
        assert!(!state.is_bookmarked(&alpha_path));
        assert_eq!(state.preferences.recent_notes.first(), Some(&renamed_path));

        // 5. Test Delete Note removing from bookmarks & recent
        state.delete_note(&renamed_path).unwrap();
        assert!(!state.is_bookmarked(&renamed_path));
        assert!(!state.preferences.recent_notes.contains(&renamed_path));
    }

    #[test]
    fn test_soft_delete_and_restore_workflow() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("TrashVault");

        let mut state = AppState::default();
        state
            .create_vault(&vault_path, Some("Trash Vault".to_string()))
            .unwrap();

        state.create_note("DiscardMe", None).unwrap();
        let note_path = PathBuf::from("Notes").join("DiscardMe.md");

        // Verify active
        assert!(state.note_title_exists("DiscardMe"));

        // Soft delete to trash
        state.trash_note(&note_path).unwrap();
        assert!(!state.note_title_exists("DiscardMe"));

        // List trash
        let trashed = state.list_trash();
        assert_eq!(trashed.len(), 1);
        assert_eq!(trashed[0].title, "DiscardMe");

        // Restore from trash
        state
            .restore_trashed_note(&trashed[0].trash_filename)
            .unwrap();
        assert!(state.note_title_exists("DiscardMe"));

        // Trash is now empty
        assert!(state.list_trash().is_empty());
    }

    #[test]
    fn test_unlinked_mentions_workflow() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("MentionsVault");

        let mut state = AppState::default();
        state
            .create_vault(&vault_path, Some("Mentions Vault".to_string()))
            .unwrap();

        state.create_note("Rust Guide", None).unwrap();
        state.create_note("Intro", None).unwrap();

        let intro_path = PathBuf::from("Notes").join("Intro.md");
        let guide_path = PathBuf::from("Notes").join("Rust Guide.md");

        // Write plain text mention in Intro note
        state.select_note(&intro_path).unwrap();
        state.update_editor_content("Welcome! Check out the Rust Guide for more info.".to_string());
        state.save_active_note().unwrap();

        // Switch to Rust Guide note
        state.select_note(&guide_path).unwrap();

        // Detect unlinked mentions
        let mentions = state.get_unlinked_mentions();
        assert_eq!(mentions.len(), 1);
        assert_eq!(mentions[0].source_title, "Intro");
        assert_eq!(mentions[0].matched_text, "Rust Guide");

        // Link the mention
        state
            .link_unlinked_mention(&intro_path, "Rust Guide")
            .unwrap();

        // Verify Intro note now contains wikilink
        let service = state.vault_service.as_ref().unwrap();
        let intro_note = service.read_note(&intro_path).unwrap();
        assert!(intro_note.content.contains("[[Rust Guide]]"));

        // Mentions should now be empty
        let mentions_after = state.get_unlinked_mentions();
        assert!(mentions_after.is_empty());

        // Backlinks should now include Intro
        let backlinks = state.get_current_backlinks();
        assert_eq!(backlinks.len(), 1);
    }

    #[test]
    fn test_properties_frontmatter_workflow() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("PropsVault");

        let mut state = AppState::default();
        state
            .create_vault(&vault_path, Some("Props Vault".to_string()))
            .unwrap();

        state.create_note("MyDoc", None).unwrap();

        // Set properties
        state
            .set_frontmatter_property("status", serde_yaml::Value::String("published".to_string()))
            .unwrap();
        state
            .set_frontmatter_property("score", serde_yaml::Value::Number(99.into()))
            .unwrap();
        state
            .set_frontmatter_property(
                "tags",
                serde_yaml::Value::Sequence(vec![
                    serde_yaml::Value::String("tech".to_string()),
                    serde_yaml::Value::String("rust".to_string()),
                ]),
            )
            .unwrap();

        let fm = state.get_active_frontmatter().unwrap();
        assert_eq!(fm.tags, vec!["tech".to_string(), "rust".to_string()]);
        assert_eq!(
            fm.extra.get("status"),
            Some(&serde_yaml::Value::String("published".to_string()))
        );
        assert_eq!(
            fm.extra.get("score"),
            Some(&serde_yaml::Value::Number(99.into()))
        );

        // Remove property
        state.remove_frontmatter_property("score").unwrap();
        let fm2 = state.get_active_frontmatter().unwrap();
        assert_eq!(fm2.extra.get("score"), None);
        assert_eq!(
            fm2.extra.get("status"),
            Some(&serde_yaml::Value::String("published".to_string()))
        );
    }

    #[test]
    fn test_split_view_and_title_increment() {
        let mut state = AppState::default();

        // Split view toggling
        assert!(state.split_pane.is_none());
        state.toggle_split();
        assert!(state.split_pane.is_some());
        assert_eq!(state.split_direction, SplitDirection::Horizontal);

        state.toggle_split_direction();
        assert_eq!(state.split_direction, SplitDirection::Vertical);

        state.close_split();
        assert!(state.split_pane.is_none());

        // Note auto-increment title
        assert_eq!(state.next_available_note_title(), "Untitled");
    }

    #[test]
    fn test_vault_health_audit_and_broken_link_fixes() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("HealthVault");
        let mut state = AppState::default();
        state.create_vault(&vault_path, None).unwrap();

        // Note 1 references a non-existent note [[MissingTarget]]
        state.create_note("Alpha", None).unwrap();
        state.update_editor_content("# Alpha\nThis links to [[MissingTarget]].\n".to_string());
        state.save_active_note().unwrap();

        // Note 2 is an orphan with no links
        state.create_note("OrphanNote", None).unwrap();
        state.update_editor_content("# Orphan\nCompletely disconnected note.".to_string());
        state.save_active_note().unwrap();
        let note2_rel = state.active_note.as_ref().unwrap().relative_path.clone();

        // Note 3 references Note 4
        state.create_note("Beta", None).unwrap();
        state.update_editor_content("# Beta\nThis links to [[Gamma]].\n".to_string());
        state.save_active_note().unwrap();

        state.create_note("Gamma", None).unwrap();
        state.update_editor_content("# Gamma\nTarget of beta link.\n".to_string());
        state.save_active_note().unwrap();

        // 1. Audit vault
        let report = state.audit_vault().expect("Audit report should exist");
        assert_eq!(report.total_notes, 4);
        assert_eq!(report.broken_links.len(), 1);
        assert_eq!(report.broken_links[0].target, "MissingTarget");
        assert_eq!(report.orphan_notes.len(), 1);
        assert_eq!(report.orphan_notes[0], note2_rel);

        // 2. Fix broken link by creating note
        state.fix_broken_link_create_note("MissingTarget").unwrap();
        let report2 = state.audit_vault().unwrap();
        assert_eq!(report2.broken_links.len(), 0);
        assert_eq!(report2.total_notes, 5);

        // 3. Test unlink on a note with broken link
        state.create_note("Delta", None).unwrap();
        state.update_editor_content(
            "# Delta\nReferences [[GhostDoc|Custom Display]].\n".to_string(),
        );
        state.save_active_note().unwrap();

        let report3 = state.audit_vault().unwrap();
        assert_eq!(report3.broken_links.len(), 1);
        assert_eq!(report3.broken_links[0].target, "GhostDoc");

        state.fix_broken_link_unlink("GhostDoc").unwrap();
        let report4 = state.audit_vault().unwrap();
        assert_eq!(report4.broken_links.len(), 0);

        // Verify that [[GhostDoc|Custom Display]] was replaced with Custom Display
        let note5_rel = state
            .entries
            .iter()
            .find_map(|e| {
                if let VaultEntry::Note(s) = e {
                    if s.title == "Delta" {
                        Some(s.relative_path.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap();
        let note5_content = state
            .vault_service
            .as_ref()
            .unwrap()
            .read_note(&note5_rel)
            .unwrap()
            .content;
        assert!(note5_content.contains("Custom Display"));
        assert!(!note5_content.contains("[[GhostDoc"));

        // 4. Batch trash orphans
        state.batch_trash_orphans(&[note2_rel]).unwrap();
        let report5 = state.audit_vault().unwrap();
        // Note that Delta became an orphan when its only link to GhostDoc was unlinked!
        assert_eq!(report5.orphan_notes.len(), 1);
        assert_eq!(report5.orphan_notes[0], note5_rel);

        // Trashing Delta leaves 0 orphans
        state.batch_trash_orphans(&[note5_rel]).unwrap();
        let report6 = state.audit_vault().unwrap();
        assert_eq!(report6.orphan_notes.len(), 0);
    }

    #[test]
    fn test_related_notes_scoring_in_state() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("RelatedVault");
        let mut state = AppState::default();
        state.create_vault(&vault_path, None).unwrap();

        state.create_note("Rust Concurrency", None).unwrap();
        state.update_editor_content("# Rust Concurrency\nFearless concurrency with threads and message passing.\nTags: #rust #systems\n".to_string());
        state.save_active_note().unwrap();
        let note1_rel = state.active_note.as_ref().unwrap().relative_path.clone();

        state.create_note("Cargo Systems", None).unwrap();
        state.update_editor_content("# Cargo Systems\nCargo builds and manages dependencies for rust systems.\nTags: #rust #tools\n".to_string());
        state.save_active_note().unwrap();

        state.create_note("Cooking Recipe", None).unwrap();
        state.update_editor_content("# Cooking Recipe\nDelicious pasta with tomato sauce and basil leaves.\nTags: #food #cooking\n".to_string());
        state.save_active_note().unwrap();

        // Select Note 1
        state.select_note(&note1_rel).unwrap();

        let related = state.get_related_notes_for_active();
        assert!(!related.is_empty());
        assert_eq!(related[0].title, "Cargo Systems");
        assert!(related[0]
            .shared_tags
            .iter()
            .any(|t| t.to_lowercase() == "rust"));
        assert!(related[0].match_percentage > 0);

        // Test append_link_to_active_note
        state.append_link_to_active_note("Cargo Systems").unwrap();
        assert!(state.editor_content.contains("[[Cargo Systems]]"));
    }

    #[test]
    fn test_citation_library_and_insertion() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("CitationVault");
        let mut state = AppState::default();
        state.create_vault(&vault_path, None).unwrap();

        // Write a .bib file inside the vault
        let bib_content = r#"
@article{shannon1948mathematical,
  author = {Shannon, Claude E.},
  title = {A Mathematical Theory of Communication},
  journal = {Bell System Technical Journal},
  year = {1948},
  volume = {27},
  pages = {379--423}
}
"#;
        let bib_file = vault_path.join("references.bib");
        std::fs::write(&bib_file, bib_content).unwrap();

        // Refresh bibliography
        state.refresh_bib_library();
        assert_eq!(state.bib_library.entries.len(), 1);
        assert_eq!(
            state.bib_library.entries[0].citation_key,
            "shannon1948mathematical"
        );
        assert_eq!(
            state.bib_library.entries[0].title.as_deref(),
            Some("A Mathematical Theory of Communication")
        );

        // Create a note and insert citations
        state.create_note("Info Theory", None).unwrap();
        state
            .update_editor_content("# Info Theory\nFoundations of information theory.".to_string());
        state.save_active_note().unwrap();

        // 1. Insert citekey [@key]
        state.insert_citation("shannon1948mathematical", false);
        assert!(state.editor_content.contains("[@shannon1948mathematical]"));

        // 2. Insert full reference
        state.insert_citation("shannon1948mathematical", true);
        assert!(state
            .editor_content
            .contains("Claude E. Shannon (1948). A Mathematical Theory of Communication."));
    }

    #[test]
    fn test_handle_nodera_uri_workflow() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("UriVault");
        let mut state = AppState::default();
        state.create_vault(&vault_path, None).unwrap();

        // 1. Handle New note URI
        let new_uri = nodera_core::NoderaUri::New {
            vault: None,
            title: "QuickIdea".to_string(),
            content: Some("Remember to research distributed consensus.".to_string()),
            tags: vec!["idea".to_string(), "distributed".to_string()],
        };
        state.handle_nodera_uri(&new_uri).unwrap();
        assert!(state.active_note.is_some());
        let active = state.active_note.as_ref().unwrap();
        assert_eq!(active.title, "QuickIdea");
        assert!(state.editor_content.contains("distributed consensus"));
        assert!(state.editor_content.contains("idea"));

        // 2. Handle Search URI
        let search_uri = nodera_core::NoderaUri::Search {
            vault: None,
            query: "consensus".to_string(),
        };
        state.handle_nodera_uri(&search_uri).unwrap();
        assert!(state.show_command_palette);
        assert_eq!(state.command_palette_query, "consensus");
        assert_eq!(state.search_query, "consensus");

        // 3. Handle Open note URI (existing note)
        let open_uri = nodera_core::NoderaUri::Open {
            vault: None,
            note: "QuickIdea.md".to_string(),
            line: None,
        };
        state.handle_nodera_uri(&open_uri).unwrap();
        assert_eq!(state.active_view, ActiveView::Editor);
        assert_eq!(state.active_note.as_ref().unwrap().title, "QuickIdea");

        // 4. Handle Daily note URI
        let daily_uri = nodera_core::NoderaUri::Daily { vault: None };
        state.handle_nodera_uri(&daily_uri).unwrap();
        assert!(state
            .active_note
            .as_ref()
            .unwrap()
            .relative_path
            .to_string_lossy()
            .starts_with("Daily"));
    }

    #[test]
    fn test_cross_vault_state_isolation() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_a = tmp.path().join("VaultA");
        let vault_b = tmp.path().join("VaultB");

        let mut state = AppState::default();

        // 1. Setup Vault A with an active note, open tab, navigation history, and split pane
        state.create_vault(&vault_a, Some("VaultA".to_string())).unwrap();
        state.create_note("NoteInA", None).unwrap();
        state.update_editor_content("Content in Vault A".to_string());
        assert!(!state.open_tabs.is_empty());
        assert!(!state.nav_history.is_empty());
        state.split_pane = Some(SplitPane {
            relative_path: Some(PathBuf::from("NoteInA.md")),
            is_reading_mode: false,
            editor_content: "Content in Vault A".to_string(),
        });

        // 2. Open Vault B - check that state from Vault A does not leak
        state.create_vault(&vault_b, Some("VaultB".to_string())).unwrap();
        assert_eq!(state.vault_name, "VaultB");
        assert!(state.open_tabs.is_empty(), "Tabs must be cleared on vault switch");
        assert!(state.active_tab_index.is_none());
        assert!(state.nav_history.is_empty(), "Nav history must be cleared on vault switch");
        assert_eq!(state.nav_history_index, 0);
        assert!(state.split_pane.is_none(), "Split pane must be cleared on vault switch");
        assert!(state.active_note.is_none(), "Active note must be cleared on vault switch");
        assert!(state.editor_content.is_empty(), "Editor content must be cleared on vault switch");

        // 3. Test close_vault
        state.create_note("NoteInB", None).unwrap();
        assert!(!state.open_tabs.is_empty());
        state.close_vault().unwrap();

        assert_eq!(state.vault_name, "No Vault Opened");
        assert!(state.vault_path.is_none());
        assert!(state.vault_service.is_none());
        assert!(state.entries.is_empty());
        assert!(state.open_tabs.is_empty());
        assert!(state.active_note.is_none());
        assert!(state.editor_content.is_empty());
        assert!(state.split_pane.is_none());
        assert!(state.nav_history.is_empty());
    }

    #[test]
    fn test_safe_note_promotion_preserves_path_and_links() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("PromotionVault");
        let mut state = AppState::default();
        state.create_vault(&vault_path, Some("PromotionVault".to_string())).unwrap();

        // 1. Create a rough note
        state.create_typed_note(NOTE_TYPE_ROUGH, "Raw Insight", None).unwrap();
        let initial_path = state.active_note.as_ref().unwrap().relative_path.clone();

        // 2. Create another note linking to this rough note
        state.create_note("Synthesis Hub", None).unwrap();
        state.update_editor_content("# Hub\nSee [[Raw Insight]] for details.".to_string());
        state.save_active_note().unwrap();

        // 3. Switch back to rough note
        state.select_note(&initial_path).unwrap();
        assert!(state.editor_content.contains("type: rough"));

        // 4. Promote note to permanent
        state.promote_active_note_to_permanent().unwrap();

        // 5. Invariant checks:
        // - Path MUST be identical (no silent renames or moves!)
        let promoted_path = state.active_note.as_ref().unwrap().relative_path.clone();
        assert_eq!(initial_path, promoted_path, "Path must not change during promotion");

        // - Frontmatter type must now be permanent
        let (fm, _) = parse_frontmatter(&state.editor_content).unwrap();
        let fm = fm.unwrap();
        assert_eq!(fm.note_type(), Some("permanent"));
        assert!(fm.is_permanent());
        assert!(!fm.is_rough());

        // - Links to the note remain 100% valid in the index
        state.select_note(&promoted_path).unwrap();
        let backlinks_after = state.get_current_backlinks();
        assert!(backlinks_after.iter().any(|p| p.file_stem().and_then(|s| s.to_str()) == Some("Synthesis Hub")));
    }

    #[test]
    fn test_quick_capture_and_review_queue() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("CaptureVault");
        let mut state = AppState::default();
        state.create_vault(&vault_path, Some("CaptureVault".to_string())).unwrap();

        // 1. Trigger Quick Capture
        state.open_quick_capture();
        assert!(state.show_quick_capture);
        state.quick_capture_title = "Eureka Moment".to_string();
        state.quick_capture_body = "The key to architecture is simplicity.".to_string();
        state.quick_capture_tags = "spark, insight".to_string();

        state.execute_quick_capture(false).unwrap();
        assert!(!state.show_quick_capture);

        // 2. Query Review Queue
        state.review_queue_filter = ReviewCategory::RoughNote;
        let items = state.get_review_queue_items();
        assert!(items.iter().any(|item| item.title == "Eureka Moment"));

        // 3. Mark as reviewed
        let note_path = items.iter().find(|i| i.title == "Eureka Moment").unwrap().path.clone();
        state.mark_note_reviewed(Path::new(&note_path)).unwrap();

        // 4. Verify it no longer appears in unprocessed rough notes
        let items_after = state.get_review_queue_items();
        assert!(!items_after.iter().any(|item| item.title == "Eureka Moment"));
    }
}


