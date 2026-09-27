use dioxus::prelude::*;
use nodera_project::{ProjectEdgeKind, ProjectNode, ProjectNodeKind};
use std::path::Path;

use crate::components::graph_view::LocalGraphView;
use crate::icons::*;
use crate::state::{
    ActiveView, AppState, GraphPalettePreset, ProjectContext, STANDARD_GRAPH_SWATCHES,
};
use crate::strings::{app as app_strings, empty_states, graph as graph_strings, nav};

/// Unified Contextual Right Inspector surface.
///
/// Dispatches contextually between Note Inspector (Outline, Properties, Links, Related, Local Graph),
/// External Project Inspector (Symbol AST metadata, signatures, file references),
/// and Graph Inspector (Filters, Groups, Display, Forces).
#[component]
pub fn Inspector(mut state: Signal<AppState>) -> Element {
    let app_state = state.read();
    if !app_state.context_panel_open {
        return rsx! {};
    }

    let is_graph_mode = app_state.active_view == ActiveView::Graph;
    let panel_width = app_state.context_panel_width;

    rsx! {
        aside {
            class: "pane-context",
            style: "width: {panel_width}px; flex-shrink: 0;",
            if is_graph_mode {
                if app_state.active_project.is_some() {
                    ProjectInspector { state }
                } else {
                    GraphInspector { state }
                }
            } else {
                NoteInspector { state }
            }
        }
    }
}

/// Note Inspector (Outline, Properties, Links, Related Notes, Local Graph)
#[component]
fn NoteInspector(mut state: Signal<AppState>) -> Element {
    let app_state = state.read();

    // Section collapse states (default open for primary discovery)
    let mut show_outline = use_signal(|| true);
    let mut show_properties = use_signal(|| true);
    let mut show_links = use_signal(|| true);
    let mut show_related = use_signal(|| true);
    let mut show_local_graph = use_signal(|| true);

    // Frontmatter signals
    let mut new_tag_input = use_signal(String::new);
    let mut new_prop_key = use_signal(String::new);
    let mut new_prop_type = use_signal(|| "text".to_string());
    let mut new_prop_val = use_signal(String::new);

    let active_note_path = app_state
        .active_note
        .as_ref()
        .map(|n| n.relative_path.clone());
    let has_active_note = active_note_path.is_some();

    // Backlinks & Outgoing links
    let backlinks = app_state.get_current_backlinks();
    let outgoing_links = app_state.get_current_outgoing_links();

    // Discovered Unlinked Mentions
    let unlinked_mentions = app_state.get_unlinked_mentions();

    // Related notes
    let related_notes = app_state.get_related_notes_for_active();

    // Frontmatter extraction
    let frontmatter = app_state.get_active_frontmatter();
    let frontmatter_title = frontmatter
        .as_ref()
        .and_then(|f| f.title.clone())
        .unwrap_or_default();
    let frontmatter_tags = frontmatter
        .as_ref()
        .map(|f| f.tags.clone())
        .unwrap_or_default();
    let frontmatter_extra = frontmatter
        .as_ref()
        .map(|f| f.extra.clone())
        .unwrap_or_default();
    let note_type = frontmatter
        .as_ref()
        .and_then(|f| f.note_type().map(|s| s.to_string()));
    let is_permanent = frontmatter
        .as_ref()
        .map(|f| f.is_permanent())
        .unwrap_or(false);

    let properties_count = (if !frontmatter_title.is_empty() { 1 } else { 0 })
        + frontmatter_tags.len()
        + frontmatter_extra.len();

    let toc_headings = app_state.toc_headings.clone();

    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; border-bottom: 1px solid var(--border); background-color: var(--bg-surface); flex-shrink: 0;",
            div { style: "display: flex; align-items: center; gap: 6px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                IconPanelRight { size: 13 }
                span { "{app_strings::CONTEXT_PANEL_TITLE}" }
            }
            button {
                class: "btn-icon",
                title: "Close Inspector (Ctrl+I)",
                onclick: move |_| {
                    state.write().context_panel_open = false;
                },
                IconClose { size: 13 }
            }
        }

        div {
            style: "flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 0;",

            // Section 1: Outline / Table of Contents
            div { style: "border-bottom: 1px solid var(--border-subtle);",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; cursor: pointer; user-select: none; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                    onclick: move |_| show_outline.toggle(),
                    div { style: "display: flex; align-items: center; gap: 6px;",
                        if *show_outline.read() {
                            IconChevronDown { size: 12 }
                        } else {
                            IconChevronRight { size: 12 }
                        }
                        span { "Outline" }
                    }
                    span { style: "font-size: 10px; color: var(--text-muted); font-weight: 500;", "{toc_headings.len()}" }
                }
                if *show_outline.read() {
                    div { style: "padding: 0 14px 12px 14px; display: flex; flex-direction: column; gap: 4px;",
                        if toc_headings.is_empty() {
                            p { style: "font-size: 11px; color: var(--text-muted); font-style: italic; margin: 0;", "No headings in current note" }
                        } else {
                            for (idx, (lvl, h_text)) in toc_headings.iter().enumerate() {
                                {
                                    let indent = (*lvl).saturating_sub(1) * 12;
                                    let text_str = h_text.clone();
                                    rsx! {
                                        div {
                                            key: "toc_{idx}_{text_str}",
                                            style: "padding-left: {indent}px; font-size: 12px; color: var(--text-primary); line-height: 1.4; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                                            span { style: "color: var(--text-muted); margin-right: 4px; font-size: 10px;", "H{lvl}" }
                                            span { "{text_str}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Section 2: Properties (Visual YAML Frontmatter)
            if has_active_note {
                div { style: "border-bottom: 1px solid var(--border-subtle);",
                    div {
                        style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; cursor: pointer; user-select: none; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                        onclick: move |_| show_properties.toggle(),
                        div { style: "display: flex; align-items: center; gap: 6px;",
                            if *show_properties.read() {
                                IconChevronDown { size: 12 }
                            } else {
                                IconChevronRight { size: 12 }
                            }
                            IconProperties { size: 12 }
                            span { "Properties" }
                        }
                        if properties_count > 0 {
                            span { style: "font-size: 10px; font-weight: 700; background: var(--accent-focus); color: var(--accent-hover); padding: 1px 6px; border-radius: 8px;", "{properties_count}" }
                        }
                    }
                    if *show_properties.read() {
                        div { style: "padding: 0 14px 12px 14px; display: flex; flex-direction: column; gap: 8px;",
                            // Workflow Note Type & Safe Promotion Banner
                            if is_permanent {
                                div {
                                    style: "display: flex; align-items: center; gap: 6px; padding: 6px 10px; background: var(--success-container); border: 1px solid var(--success); border-radius: 6px;",
                                    IconPermanent { size: 14 }
                                    span { style: "font-size: 11px; font-weight: 600; color: var(--success);", "Permanent Note" }
                                }
                            } else {
                                {
                                    let type_label = note_type.clone().unwrap_or_else(|| "Rough / Draft".to_string());
                                    rsx! {
                                        div {
                                            style: "display: flex; align-items: center; justify-content: space-between; padding: 6px 10px; background: var(--bg-surface-elevated); border: 1px solid var(--border); border-radius: 6px;",
                                            div { style: "display: flex; flex-direction: column; gap: 1px;",
                                                span { style: "font-size: 11px; font-weight: 600; color: var(--text-primary); text-transform: capitalize;", "{type_label}" }
                                                span { style: "font-size: 10px; color: var(--text-muted);", "Draft knowledge" }
                                            }
                                            button {
                                                class: "btn-action btn-primary",
                                                style: "font-size: 11px; padding: 3px 8px; display: inline-flex; align-items: center; gap: 4px;",
                                                title: "Promote note to permanent atomic knowledge (updates frontmatter in-place, preserves links and path)",
                                                onclick: move |_| {
                                                    let _ = state.write().promote_active_note_to_permanent();
                                                },
                                                IconPermanent { size: 12 }
                                                span { "Promote" }
                                            }
                                        }
                                    }
                                }
                            }

                            // Title property
                            div { style: "display: flex; flex-direction: column; gap: 3px;",
                                span { style: "font-size: 10px; font-weight: 500; color: var(--text-muted); text-transform: uppercase;", "title" }
                                input {
                                    class: "property-mini-input",
                                    r#type: "text",
                                    value: "{frontmatter_title}",
                                    placeholder: "Note title...",
                                    onchange: move |evt| {
                                        let val = evt.value();
                                        let _ = state.write().set_frontmatter_property("title", serde_yaml::Value::String(val));
                                    }
                                }
                            }

                            // Tags property
                            div { style: "display: flex; flex-direction: column; gap: 4px;",
                                span { style: "font-size: 10px; font-weight: 500; color: var(--text-muted); text-transform: uppercase;", "tags" }
                                div { style: "display: flex; flex-wrap: wrap; gap: 4px; align-items: center;",
                                    for (t_idx, tag) in frontmatter_tags.iter().enumerate() {
                                        {
                                            let tag_str = tag.clone();
                                            rsx! {
                                                span { key: "prop_tag_{tag_str}_{t_idx}", class: "tag-chip",
                                                    span { "#{tag_str}" }
                                                    button {
                                                        class: "tag-chip-remove",
                                                        title: "Remove tag",
                                                        onclick: move |_| {
                                                            let mut s = state.write();
                                                            if let Some(mut fm) = s.get_active_frontmatter() {
                                                                if t_idx < fm.tags.len() {
                                                                    fm.tags.remove(t_idx);
                                                                    let _ = s.update_active_frontmatter(&fm);
                                                                }
                                                            }
                                                        },
                                                        "×"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                div { style: "display: flex; gap: 4px; margin-top: 2px;",
                                    input {
                                        class: "property-mini-input",
                                        style: "flex: 1;",
                                        r#type: "text",
                                        placeholder: "New tag...",
                                        value: "{new_tag_input.read()}",
                                        oninput: move |evt| new_tag_input.set(evt.value()),
                                        onkeydown: move |evt| {
                                            if evt.key() == Key::Enter {
                                                let tag_val = new_tag_input.read().trim().to_string();
                                                if !tag_val.is_empty() {
                                                    let mut s = state.write();
                                                    let mut fm = s.get_active_frontmatter().unwrap_or_default();
                                                    if !fm.tags.contains(&tag_val) {
                                                        fm.tags.push(tag_val);
                                                        let _ = s.update_active_frontmatter(&fm);
                                                    }
                                                    new_tag_input.set(String::new());
                                                }
                                            }
                                        }
                                    }
                                    button {
                                        class: "btn-action",
                                        style: "padding: 2px 6px; font-size: 11px;",
                                        onclick: move |_| {
                                            let tag_val = new_tag_input.read().trim().to_string();
                                            if !tag_val.is_empty() {
                                                let mut s = state.write();
                                                let mut fm = s.get_active_frontmatter().unwrap_or_default();
                                                if !fm.tags.contains(&tag_val) {
                                                    fm.tags.push(tag_val);
                                                    let _ = s.update_active_frontmatter(&fm);
                                                }
                                                new_tag_input.set(String::new());
                                            }
                                        },
                                        "+ Add"
                                    }
                                }
                            }

                            // Extra frontmatter properties
                            for (prop_key, prop_val) in frontmatter_extra.iter() {
                                {
                                    let k = prop_key.clone();
                                    let k_remove = prop_key.clone();
                                    let k_change = prop_key.clone();
                                    let val_display = match prop_val {
                                        serde_yaml::Value::String(s) => s.clone(),
                                        serde_yaml::Value::Number(n) => n.to_string(),
                                        serde_yaml::Value::Bool(b) => b.to_string(),
                                        serde_yaml::Value::Sequence(seq) => seq.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "),
                                        other => format!("{other:?}"),
                                    };
                                    let is_bool = matches!(prop_val, serde_yaml::Value::Bool(_));
                                    let bool_val = match prop_val {
                                        serde_yaml::Value::Bool(b) => *b,
                                        _ => false,
                                    };

                                    rsx! {
                                        div { key: "prop_row_{k}", style: "display: flex; flex-direction: column; gap: 3px; padding-top: 4px; border-top: 1px solid var(--border-subtle);",
                                            div { style: "display: flex; align-items: center; justify-content: space-between;",
                                                span { style: "font-size: 10px; font-weight: 500; color: var(--text-muted); font-family: var(--font-editor);", "{k}" }
                                                button {
                                                    class: "btn-icon",
                                                    style: "width: 16px; height: 16px; color: var(--text-muted);",
                                                    title: "Remove property",
                                                    onclick: move |_| {
                                                        let _ = state.write().remove_frontmatter_property(&k_remove);
                                                    },
                                                    IconClose { size: 10 }
                                                }
                                            }
                                            if is_bool {
                                                label { style: "display: flex; align-items: center; gap: 6px; font-size: 11px;",
                                                    input {
                                                        r#type: "checkbox",
                                                        checked: bool_val,
                                                        onchange: move |evt| {
                                                            let checked = evt.value().parse::<bool>().unwrap_or(false);
                                                            let _ = state.write().set_frontmatter_property(&k_change, serde_yaml::Value::Bool(checked));
                                                        }
                                                    }
                                                    span { if bool_val { "true" } else { "false" } }
                                                }
                                            } else {
                                                input {
                                                    class: "property-mini-input",
                                                    r#type: "text",
                                                    value: "{val_display}",
                                                    onchange: move |evt| {
                                                        let text = evt.value();
                                                        let yml_val = if let Ok(num) = text.parse::<i64>() {
                                                            serde_yaml::Value::Number(num.into())
                                                        } else if text.eq_ignore_ascii_case("true") {
                                                            serde_yaml::Value::Bool(true)
                                                        } else if text.eq_ignore_ascii_case("false") {
                                                            serde_yaml::Value::Bool(false)
                                                        } else {
                                                            serde_yaml::Value::String(text)
                                                        };
                                                        let _ = state.write().set_frontmatter_property(&k_change, yml_val);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Add new property row
                            div { style: "display: flex; flex-direction: column; gap: 4px; margin-top: 4px; padding-top: 6px; border-top: 1px dashed var(--border-subtle);",
                                div { style: "display: flex; gap: 4px;",
                                    input {
                                        class: "property-mini-input",
                                        style: "flex: 1;",
                                        placeholder: "Property key...",
                                        value: "{new_prop_key.read()}",
                                        oninput: move |evt| new_prop_key.set(evt.value()),
                                    }
                                    select {
                                        class: "property-type-select",
                                        value: "{new_prop_type.read()}",
                                        onchange: move |evt| new_prop_type.set(evt.value()),
                                        option { value: "text", "Text" }
                                        option { value: "number", "Number" }
                                        option { value: "checkbox", "Checkbox" }
                                        option { value: "list", "List" }
                                    }
                                }
                                div { style: "display: flex; gap: 4px;",
                                    input {
                                        class: "property-mini-input",
                                        style: "flex: 1;",
                                        placeholder: "Value...",
                                        value: "{new_prop_val.read()}",
                                        oninput: move |evt| new_prop_val.set(evt.value()),
                                        onkeydown: move |evt| {
                                            if evt.key() == Key::Enter {
                                                let k = new_prop_key.read().trim().to_string();
                                                let v_raw = new_prop_val.read().trim().to_string();
                                                let t = new_prop_type.read().clone();
                                                if !k.is_empty() {
                                                    let val = match t.as_str() {
                                                        "number" => v_raw.parse::<i64>().map(Into::into).map(serde_yaml::Value::Number).unwrap_or_else(|_| serde_yaml::Value::String(v_raw)),
                                                        "checkbox" => serde_yaml::Value::Bool(v_raw.eq_ignore_ascii_case("true") || v_raw == "1"),
                                                        "list" => serde_yaml::Value::Sequence(v_raw.split(',').map(str::trim).filter(|s| !s.is_empty()).map(|s| serde_yaml::Value::String(s.to_string())).collect()),
                                                        _ => serde_yaml::Value::String(v_raw),
                                                    };
                                                    let _ = state.write().set_frontmatter_property(&k, val);
                                                    new_prop_key.set(String::new());
                                                    new_prop_val.set(String::new());
                                                }
                                            }
                                        }
                                    }
                                    button {
                                        class: "btn-action btn-primary",
                                        style: "padding: 2px 8px; font-size: 11px;",
                                        onclick: move |_| {
                                            let k = new_prop_key.read().trim().to_string();
                                            let v_raw = new_prop_val.read().trim().to_string();
                                            let t = new_prop_type.read().clone();
                                            if !k.is_empty() {
                                                let val = match t.as_str() {
                                                    "number" => v_raw.parse::<i64>().map(Into::into).map(serde_yaml::Value::Number).unwrap_or_else(|_| serde_yaml::Value::String(v_raw)),
                                                    "checkbox" => serde_yaml::Value::Bool(v_raw.eq_ignore_ascii_case("true") || v_raw == "1"),
                                                    "list" => serde_yaml::Value::Sequence(v_raw.split(',').map(str::trim).filter(|s| !s.is_empty()).map(|s| serde_yaml::Value::String(s.to_string())).collect()),
                                                    _ => serde_yaml::Value::String(v_raw),
                                                };
                                                let _ = state.write().set_frontmatter_property(&k, val);
                                                new_prop_key.set(String::new());
                                                new_prop_val.set(String::new());
                                            }
                                        },
                                        "+ Add"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Section 3: Links (Backlinks, Outgoing, Unlinked Mentions)
            div { style: "border-bottom: 1px solid var(--border-subtle);",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; cursor: pointer; user-select: none; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                    onclick: move |_| show_links.toggle(),
                    div { style: "display: flex; align-items: center; gap: 6px;",
                        if *show_links.read() {
                            IconChevronDown { size: 12 }
                        } else {
                            IconChevronRight { size: 12 }
                        }
                        IconLink { size: 12 }
                        span { "Links" }
                    }
                    span { style: "font-size: 10px; color: var(--text-muted); font-weight: 500;", "{backlinks.len() + outgoing_links.len()}" }
                }
                if *show_links.read() {
                    div { style: "padding: 0 14px 12px 14px; display: flex; flex-direction: column; gap: 12px;",
                        // Backlinks
                        div {
                            span { style: "font-size: 10px; font-weight: 600; color: var(--text-muted); text-transform: uppercase;", "{app_strings::INCOMING_LINKS} ({backlinks.len()})" }
                            if backlinks.is_empty() {
                                p { style: "font-size: 11px; color: var(--text-muted); font-style: italic; margin: 4px 0 0 0;", "{empty_states::NO_BACKLINKS}" }
                            } else {
                                div { class: "link-list",
                                    for backlink in backlinks.iter() {
                                        {
                                            let p = backlink.clone();
                                            let p_click = p.clone();
                                            let display_title = p.file_stem().and_then(|s| s.to_str()).unwrap_or("Note").to_string();
                                            rsx! {
                                                button {
                                                    key: "bl_{p.display()}",
                                                    class: "link-item",
                                                    title: "{p.display()}",
                                                    onclick: move |_| {
                                                        let mut s = state.write();
                                                        let _ = s.select_note(&p_click);
                                                    },
                                                    span { style: "display: inline-flex; align-items: center; gap: 6px;",
                                                        IconLink { size: 12 }
                                                        span { "{display_title}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Outgoing links
                        div {
                            span { style: "font-size: 10px; font-weight: 600; color: var(--text-muted); text-transform: uppercase;", "{app_strings::OUTGOING_LINKS} ({outgoing_links.len()})" }
                            if outgoing_links.is_empty() {
                                p { style: "font-size: 11px; color: var(--text-muted); font-style: italic; margin: 4px 0 0 0;", "{empty_states::NO_OUTGOING_LINKS}" }
                            } else {
                                div { class: "link-list",
                                    for (link, resolved) in outgoing_links.iter() {
                                        {
                                            let target = link.target.clone();
                                            let target_click = target.clone();
                                            let label = link.label().to_string();
                                            let is_resolved = resolved.is_some();
                                            rsx! {
                                                button {
                                                    key: "ol_{target}",
                                                    class: if is_resolved { "link-item" } else { "link-item link-item-unresolved" },
                                                    title: if is_resolved { format!("Open '{target}'") } else { format!("Create '{target}'") },
                                                    onclick: move |_| {
                                                        let mut s = state.write();
                                                        let _ = s.open_or_create_target(&target_click);
                                                    },
                                                    span { style: "display: inline-flex; align-items: center; gap: 6px;",
                                                        if is_resolved {
                                                            IconExternalLink { size: 12 }
                                                        } else {
                                                            IconPlus { size: 12 }
                                                        }
                                                        span { "{label}" }
                                                    }
                                                    if !is_resolved {
                                                        span { style: "font-size: 10px; opacity: 0.7;", "new" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Unlinked Mentions
                        if !unlinked_mentions.is_empty() {
                            div {
                                span { style: "font-size: 10px; font-weight: 600; color: var(--text-muted); text-transform: uppercase;", "Unlinked Mentions ({unlinked_mentions.len()})" }
                                div { style: "display: flex; flex-direction: column; gap: 6px; margin-top: 4px;",
                                    for mention in unlinked_mentions.iter() {
                                        {
                                            let src_path = mention.source_path.clone();
                                            let src_click = src_path.clone();
                                            let matched = mention.matched_text.clone();
                                            let title = mention.source_title.clone();
                                            let before = mention.snippet_before.clone();
                                            let after = mention.snippet_after.clone();
                                            let active_t = app_state.active_note.as_ref().map(|n| n.title.clone()).unwrap_or_default();

                                            rsx! {
                                                div {
                                                    key: "um_{mention.source_path.display()}_{mention.matched_text}_{before}",
                                                    class: "unlinked-mention-card",
                                                    div { style: "display: flex; align-items: center; justify-content: space-between;",
                                                        button {
                                                            class: "link-item",
                                                            style: "padding: 0; font-weight: 600; border: none; background: transparent;",
                                                            onclick: move |_| {
                                                                let mut s = state.write();
                                                                let _ = s.select_note(&src_click);
                                                            },
                                                            "{title}"
                                                        }
                                                        button {
                                                            class: "btn-action btn-primary",
                                                            style: "font-size: 10px; padding: 1px 6px;",
                                                            title: "Convert mention into [[wikilink]]",
                                                            onclick: move |_| {
                                                                let mut s = state.write();
                                                                let _ = s.link_unlinked_mention(&src_path, &active_t);
                                                            },
                                                            "Link"
                                                        }
                                                    }
                                                    div { class: "unlinked-snippet",
                                                        span { "...{before}" }
                                                        span { style: "background: var(--accent-focus); color: var(--accent-hover); font-weight: 600; padding: 0 2px; border-radius: 2px;", "{matched}" }
                                                        span { "{after}..." }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Section 4: Related Notes (Lexical BM25 + Tag Overlap)
            div { style: "border-bottom: 1px solid var(--border-subtle);",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; cursor: pointer; user-select: none; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                    onclick: move |_| show_related.toggle(),
                    div { style: "display: flex; align-items: center; gap: 6px;",
                        if *show_related.read() {
                            IconChevronDown { size: 12 }
                        } else {
                            IconChevronRight { size: 12 }
                        }
                        span { "Related Notes" }
                    }
                    span { style: "font-size: 10px; color: var(--text-muted); font-weight: 500;", "{related_notes.len()}" }
                }
                if *show_related.read() {
                    div { style: "padding: 0 14px 12px 14px; display: flex; flex-direction: column; gap: 6px;",
                        span { style: "font-size: 10px; color: var(--text-muted);", "Ranked by Lexical BM25 + Tag Overlap" }
                        if related_notes.is_empty() {
                            p { style: "font-size: 11px; color: var(--text-muted); font-style: italic; margin: 4px 0 0 0;", "No related notes found" }
                        } else {
                            for rel in related_notes.iter() {
                                {
                                    let rel_path = std::path::PathBuf::from(&rel.path);
                                    let rel_path_click = rel_path.clone();
                                    let rel_title = rel.title.clone();
                                    let rel_title_link = rel.title.clone();
                                    let shared_tags = rel.shared_tags.clone();
                                    let tag_count = shared_tags.len();
                                    let tag_chip_label = if tag_count > 1 {
                                        format!("{tag_count} tags")
                                    } else {
                                        format!("{tag_count} tag")
                                    };

                                    rsx! {
                                        div {
                                            key: "rel_{rel.path}",
                                            class: "unlinked-mention-card",
                                            style: "padding: 8px 10px; gap: 4px;",
                                            div { style: "display: flex; align-items: center; justify-content: space-between;",
                                                button {
                                                    class: "link-item",
                                                    style: "padding: 0; font-weight: 600; border: none; background: transparent; text-align: left; font-size: 12px;",
                                                    onclick: move |_| {
                                                        let mut s = state.write();
                                                        let _ = s.select_note(&rel_path_click);
                                                    },
                                                    "{rel_title}"
                                                }
                                                div { style: "display: flex; align-items: center; gap: 6px;",
                                                    if !shared_tags.is_empty() {
                                                        span { style: "font-size: 10px; font-weight: 600; background: var(--accent-focus); color: var(--accent-hover); padding: 1px 5px; border-radius: 4px;",
                                                            "{tag_chip_label}"
                                                        }
                                                    } else {
                                                        span { style: "font-size: 10px; font-weight: 500; background: var(--bg-surface-elevated); color: var(--text-muted); padding: 1px 5px; border-radius: 4px; border: 1px solid var(--border-subtle);",
                                                            "lexical"
                                                        }
                                                    }
                                                    button {
                                                        class: "btn-action btn-primary",
                                                        style: "font-size: 10px; padding: 1px 6px;",
                                                        title: "Add wikilink to this note",
                                                        onclick: move |_| {
                                                            let mut s = state.write();
                                                            let _ = s.append_link_to_active_note(&rel_title_link);
                                                        },
                                                        "+ Link"
                                                    }
                                                }
                                            }
                                            if !shared_tags.is_empty() {
                                                div { style: "display: flex; flex-wrap: wrap; gap: 3px; margin-top: 2px;",
                                                    for t in shared_tags.iter() {
                                                        span {
                                                            key: "rel_tag_{t}",
                                                            style: "font-size: 10px; color: var(--accent); background: var(--accent-focus); padding: 0 4px; border-radius: 3px;",
                                                            "#{t}"
                                                        }
                                                    }
                                                }
                                            } else {
                                                div { style: "margin-top: 1px;",
                                                    span { style: "font-size: 10px; color: var(--text-muted); font-style: italic;", "Lexical BM25 match" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Section 5: Local Graph
            div {
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; cursor: pointer; user-select: none; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                    onclick: move |_| show_local_graph.toggle(),
                    div { style: "display: flex; align-items: center; gap: 6px;",
                        if *show_local_graph.read() {
                            IconChevronDown { size: 12 }
                        } else {
                            IconChevronRight { size: 12 }
                        }
                        IconGraph { size: 12 }
                        span { "{nav::GRAPH}" }
                    }
                }
                if *show_local_graph.read() {
                    div { style: "padding: 0 14px 16px 14px;",
                        LocalGraphView { state }
                    }
                }
            }
        }
    }
}

/// Graph Inspector (Filters, Groups, Display, Forces)
#[component]
fn GraphInspector(mut state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let settings = app_state.preferences.graph_settings.clone();

    let mut expanded_filters = use_signal(|| settings.expanded_sections.filters);
    let mut expanded_groups = use_signal(|| settings.expanded_sections.groups);
    let mut expanded_display = use_signal(|| settings.expanded_sections.display);
    let mut expanded_forces = use_signal(|| settings.expanded_sections.forces);
    let mut expanded_colors = use_signal(|| settings.expanded_sections.colors);

    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; border-bottom: 1px solid var(--border); background-color: var(--bg-surface); flex-shrink: 0;",
            div { style: "display: flex; align-items: center; gap: 6px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                IconSliders { size: 13 }
                span { "Graph Settings" }
            }
            div { style: "display: flex; align-items: center; gap: 4px;",
                button {
                    class: "btn-icon",
                    title: graph_strings::RESTORE_DEFAULTS,
                    onclick: move |_| {
                        let mut s = state.write();
                        s.preferences.graph_settings.reset_to_defaults();
                        s.preferences.save();
                    },
                    IconRefresh { size: 13 }
                }
                button {
                    class: "btn-icon",
                    title: "Close Inspector",
                    onclick: move |_| {
                        state.write().context_panel_open = false;
                    },
                    IconClose { size: 13 }
                }
            }
        }

        div {
            style: "flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 0;",

            // Section 1: Filters
            div { style: "border-bottom: 1px solid var(--border-subtle);",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; cursor: pointer; user-select: none; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                    onclick: move |_| expanded_filters.toggle(),
                    div { style: "display: flex; align-items: center; gap: 6px;",
                        if *expanded_filters.read() {
                            IconChevronDown { size: 12 }
                        } else {
                            IconChevronRight { size: 12 }
                        }
                        span { "{graph_strings::FILTERS}" }
                    }
                }
                if *expanded_filters.read() {
                    div { style: "padding: 0 14px 12px 14px; display: flex; flex-direction: column; gap: 8px;",
                        // Tags toggle
                        div { class: "graph-toggle-row",
                            span { "{graph_strings::TAGS}" }
                            label { class: "graph-switch",
                                input {
                                    r#type: "checkbox",
                                    checked: settings.filters.tags,
                                    onchange: move |evt: FormEvent| {
                                        let checked = evt.value() == "true";
                                        let mut s = state.write();
                                        s.preferences.graph_settings.filters.tags = checked;
                                        s.preferences.save();
                                    }
                                }
                                span { class: "graph-switch-slider" }
                            }
                        }

                        // Attachments toggle
                        div { class: "graph-toggle-row",
                            span { "{graph_strings::ATTACHMENTS}" }
                            label { class: "graph-switch",
                                input {
                                    r#type: "checkbox",
                                    checked: settings.filters.attachments,
                                    onchange: move |evt: FormEvent| {
                                        let checked = evt.value() == "true";
                                        let mut s = state.write();
                                        s.preferences.graph_settings.filters.attachments = checked;
                                        s.preferences.save();
                                    }
                                }
                                span { class: "graph-switch-slider" }
                            }
                        }

                        // Existing files only toggle
                        div { class: "graph-toggle-row",
                            span { "{graph_strings::EXISTING_FILES_ONLY}" }
                            label { class: "graph-switch",
                                input {
                                    r#type: "checkbox",
                                    checked: settings.filters.existing_files_only,
                                    onchange: move |evt: FormEvent| {
                                        let checked = evt.value() == "true";
                                        let mut s = state.write();
                                        s.preferences.graph_settings.filters.existing_files_only = checked;
                                        s.preferences.save();
                                    }
                                }
                                span { class: "graph-switch-slider" }
                            }
                        }

                        // Orphans toggle
                        div { class: "graph-toggle-row",
                            span { "{graph_strings::ORPHANS}" }
                            label { class: "graph-switch",
                                input {
                                    r#type: "checkbox",
                                    checked: settings.filters.orphans,
                                    onchange: move |evt: FormEvent| {
                                        let checked = evt.value() == "true";
                                        let mut s = state.write();
                                        s.preferences.graph_settings.filters.orphans = checked;
                                        s.preferences.save();
                                    }
                                }
                                span { class: "graph-switch-slider" }
                            }
                        }
                    }
                }
            }

            // Section 2: Groups
            div { style: "border-bottom: 1px solid var(--border-subtle);",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; cursor: pointer; user-select: none; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                    onclick: move |_| expanded_groups.toggle(),
                    div { style: "display: flex; align-items: center; gap: 6px;",
                        if *expanded_groups.read() {
                            IconChevronDown { size: 12 }
                        } else {
                            IconChevronRight { size: 12 }
                        }
                        span { "{graph_strings::GROUPS}" }
                    }
                }
                if *expanded_groups.read() {
                    div { style: "padding: 0 14px 12px 14px; display: flex; flex-direction: column; gap: 8px;",
                        div { class: "graph-toggle-row",
                            span { "Color by Community" }
                            label { class: "graph-switch",
                                input {
                                    r#type: "checkbox",
                                    checked: settings.display.color_by_community,
                                    onchange: move |evt: FormEvent| {
                                        let checked = evt.value() == "true";
                                        let mut s = state.write();
                                        s.preferences.graph_settings.display.color_by_community = checked;
                                        s.graph_color_by_community = checked;
                                        s.preferences.save();
                                    }
                                }
                                span { class: "graph-switch-slider" }
                            }
                        }
                    }
                }
            }

            // Section 3: Display
            div { style: "border-bottom: 1px solid var(--border-subtle);",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; cursor: pointer; user-select: none; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                    onclick: move |_| expanded_display.toggle(),
                    div { style: "display: flex; align-items: center; gap: 6px;",
                        if *expanded_display.read() {
                            IconChevronDown { size: 12 }
                        } else {
                            IconChevronRight { size: 12 }
                        }
                        span { "{graph_strings::DISPLAY}" }
                    }
                }
                if *expanded_display.read() {
                    div { style: "padding: 0 14px 12px 14px; display: flex; flex-direction: column; gap: 8px;",
                        // Arrows toggle
                        div { class: "graph-toggle-row",
                            span { "{graph_strings::ARROWS}" }
                            label { class: "graph-switch",
                                input {
                                    r#type: "checkbox",
                                    checked: settings.display.arrows,
                                    onchange: move |evt: FormEvent| {
                                        let checked = evt.value() == "true";
                                        let mut s = state.write();
                                        s.preferences.graph_settings.display.arrows = checked;
                                        s.preferences.save();
                                    }
                                }
                                span { class: "graph-switch-slider" }
                            }
                        }

                        // Size by Centrality toggle
                        div { class: "graph-toggle-row",
                            span { "Size by Centrality" }
                            label { class: "graph-switch",
                                input {
                                    r#type: "checkbox",
                                    checked: settings.display.centrality_sizing,
                                    onchange: move |evt: FormEvent| {
                                        let checked = evt.value() == "true";
                                        let mut s = state.write();
                                        s.preferences.graph_settings.display.centrality_sizing = checked;
                                        s.graph_centrality_sizing = checked;
                                        s.preferences.save();
                                    }
                                }
                                span { class: "graph-switch-slider" }
                            }
                        }

                        // Text fade slider
                        div { class: "graph-slider-row",
                            div { class: "graph-slider-header",
                                span { "{graph_strings::TEXT_FADE}" }
                                span { class: "graph-slider-val", "{settings.display.text_fade_threshold:.1}" }
                            }
                            input {
                                class: "graph-slider",
                                r#type: "range",
                                min: "0.2",
                                max: "2.0",
                                step: "0.1",
                                value: "{settings.display.text_fade_threshold}",
                                oninput: move |evt: FormEvent| {
                                    if let Ok(v) = evt.value().parse::<f32>() {
                                        let mut s = state.write();
                                        s.preferences.graph_settings.display.text_fade_threshold = v;
                                        s.preferences.save();
                                    }
                                }
                            }
                        }

                        // Node size slider
                        div { class: "graph-slider-row",
                            div { class: "graph-slider-header",
                                span { "{graph_strings::NODE_SIZE}" }
                                span { class: "graph-slider-val", "{settings.display.node_size:.1}x" }
                            }
                            input {
                                class: "graph-slider",
                                r#type: "range",
                                min: "0.4",
                                max: "2.5",
                                step: "0.1",
                                value: "{settings.display.node_size}",
                                oninput: move |evt: FormEvent| {
                                    if let Ok(v) = evt.value().parse::<f32>() {
                                        let mut s = state.write();
                                        s.preferences.graph_settings.display.node_size = v;
                                        s.preferences.save();
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Section 4: Forces
            div {
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; cursor: pointer; user-select: none; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                    onclick: move |_| expanded_forces.toggle(),
                    div { style: "display: flex; align-items: center; gap: 6px;",
                        if *expanded_forces.read() {
                            IconChevronDown { size: 12 }
                        } else {
                            IconChevronRight { size: 12 }
                        }
                        span { "{graph_strings::FORCES}" }
                    }
                }
                if *expanded_forces.read() {
                    div { style: "padding: 0 14px 16px 14px; display: flex; flex-direction: column; gap: 8px;",
                        // Center force
                        div { class: "graph-slider-row",
                            div { class: "graph-slider-header",
                                span { "{graph_strings::CENTER_FORCE}" }
                                span { class: "graph-slider-val", "{settings.forces.center_force:.2}" }
                            }
                            input {
                                class: "graph-slider",
                                r#type: "range",
                                min: "0.05",
                                max: "2.0",
                                step: "0.05",
                                value: "{settings.forces.center_force}",
                                oninput: move |evt: FormEvent| {
                                    if let Ok(v) = evt.value().parse::<f32>() {
                                        let mut s = state.write();
                                        s.preferences.graph_settings.forces.center_force = v;
                                        s.preferences.save();
                                    }
                                }
                            }
                        }

                        // Repel force
                        div { class: "graph-slider-row",
                            div { class: "graph-slider-header",
                                span { "{graph_strings::REPEL_FORCE}" }
                                span { class: "graph-slider-val", "{settings.forces.repel_force:.1}" }
                            }
                            input {
                                class: "graph-slider",
                                r#type: "range",
                                min: "10.0",
                                max: "200.0",
                                step: "5.0",
                                value: "{settings.forces.repel_force}",
                                oninput: move |evt: FormEvent| {
                                    if let Ok(v) = evt.value().parse::<f32>() {
                                        let mut s = state.write();
                                        s.preferences.graph_settings.forces.repel_force = v;
                                        s.preferences.save();
                                    }
                                }
                            }
                        }

                        // Link distance
                        div { class: "graph-slider-row",
                            div { class: "graph-slider-header",
                                span { "{graph_strings::LINK_DISTANCE}" }
                                span { class: "graph-slider-val", "{settings.forces.link_distance:.0}px" }
                            }
                            input {
                                class: "graph-slider",
                                r#type: "range",
                                min: "20.0",
                                max: "300.0",
                                step: "5.0",
                                value: "{settings.forces.link_distance}",
                                oninput: move |evt: FormEvent| {
                                    if let Ok(v) = evt.value().parse::<f32>() {
                                        let mut s = state.write();
                                        s.preferences.graph_settings.forces.link_distance = v;
                                        s.preferences.save();
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Section 5: Colors & Palettes
            div { style: "border-bottom: 1px solid var(--border-subtle);",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; cursor: pointer; user-select: none; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                    onclick: move |_| expanded_colors.toggle(),
                    div { style: "display: flex; align-items: center; gap: 6px;",
                        if *expanded_colors.read() {
                            IconChevronDown { size: 12 }
                        } else {
                            IconChevronRight { size: 12 }
                        }
                        span { "Colors & Palettes" }
                    }
                }
                if *expanded_colors.read() {
                    div { style: "padding: 0 14px 16px 14px;",
                        GraphColorControls { state }
                    }
                }
            }
        }
    }
}

/// Project Context Inspector for external software repositories.
#[component]
fn ProjectInspector(mut state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let project = match &app_state.active_project {
        Some(p) => p.clone(),
        None => return rsx! {},
    };

    let selected_id = app_state.graph_view_state.selected_node_id.clone();
    let selected_node = selected_id
        .as_ref()
        .and_then(|id| project.graph.nodes.iter().find(|n| &n.id == id).cloned());

    let mut show_graph_controls = use_signal(|| false);

    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; border-bottom: 1px solid var(--border); background-color: var(--bg-surface); flex-shrink: 0;",
            div { style: "display: flex; align-items: center; gap: 6px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                IconFile { size: 13 }
                span { "Project: {project.name}" }
            }
            div { style: "display: flex; align-items: center; gap: 4px;",
                button {
                    class: "btn-icon",
                    title: "Close Inspector",
                    onclick: move |_| {
                        state.write().context_panel_open = false;
                    },
                    IconClose { size: 13 }
                }
            }
        }

        div {
            style: "flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 0;",

            if let Some(node) = selected_node {
                SymbolInspector {
                    state,
                    project: project.clone(),
                    node,
                }
            } else {
                ProjectOverviewInspector {
                    project: project.clone(),
                }
            }

            // Collapsible Graph Controls Drawer
            div { style: "border-top: 1px solid var(--border-subtle); margin-top: auto;",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; cursor: pointer; user-select: none; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                    onclick: move |_| show_graph_controls.toggle(),
                    div { style: "display: flex; align-items: center; gap: 6px;",
                        if *show_graph_controls.read() {
                            IconChevronDown { size: 12 }
                        } else {
                            IconChevronRight { size: 12 }
                        }
                        span { "Graph Settings" }
                    }
                }
                if *show_graph_controls.read() {
                    div { style: "padding: 0 14px 14px 14px; display: flex; flex-direction: column; gap: 12px;",
                        div { class: "graph-slider-row",
                            div { class: "graph-slider-header",
                                span { "Zoom Level" }
                                span { class: "graph-slider-val", "{app_state.graph_view_state.zoom:.2}x" }
                            }
                        }
                        GraphColorControls { state }
                    }
                }
            }
        }
    }
}

/// Detailed inspector view for an individual AST symbol in an external project.
#[component]
fn SymbolInspector(
    mut state: Signal<AppState>,
    project: ProjectContext,
    node: ProjectNode,
) -> Element {
    let mut show_signature = use_signal(|| true);
    let mut show_doc = use_signal(|| true);
    let mut show_edges = use_signal(|| true);

    // Kind badge metadata
    let (kind_label, kind_bg, kind_fg) = match node.kind {
        ProjectNodeKind::Function => ("FN", "rgba(59, 130, 246, 0.15)", "#60a5fa"),
        ProjectNodeKind::Method => ("METHOD", "rgba(59, 130, 246, 0.15)", "#60a5fa"),
        ProjectNodeKind::Struct => ("STRUCT", "rgba(168, 85, 247, 0.15)", "#c084fc"),
        ProjectNodeKind::Enum => ("ENUM", "rgba(249, 115, 22, 0.15)", "#fb923c"),
        ProjectNodeKind::EnumVariant => ("VARIANT", "rgba(249, 115, 22, 0.15)", "#fb923c"),
        ProjectNodeKind::Trait => ("TRAIT", "rgba(16, 185, 129, 0.15)", "#34d399"),
        ProjectNodeKind::Implementation => ("IMPL", "rgba(20, 184, 166, 0.15)", "#2dd4bf"),
        ProjectNodeKind::Module => ("MOD", "rgba(245, 158, 11, 0.15)", "#fbbf24"),
        ProjectNodeKind::Constant => ("CONST", "rgba(239, 68, 68, 0.15)", "#f87171"),
        ProjectNodeKind::Static => ("STATIC", "rgba(239, 68, 68, 0.15)", "#f87171"),
        ProjectNodeKind::Macro => ("MACRO", "rgba(236, 72, 153, 0.15)", "#f472b6"),
        ProjectNodeKind::TypeAlias => ("TYPE", "rgba(139, 92, 246, 0.15)", "#a78bfa"),
        ProjectNodeKind::Field => ("FIELD", "rgba(100, 116, 139, 0.15)", "#94a3b8"),
        ProjectNodeKind::File => ("FILE", "rgba(107, 114, 128, 0.15)", "#9ca3af"),
        ProjectNodeKind::Other => ("OTHER", "rgba(107, 114, 128, 0.15)", "#9ca3af"),
    };

    let start_line = node.span.map(|s| s.start_line).unwrap_or(1);
    let line_str = node
        .span
        .map(|s| format!("L{}-L{}", s.start_line, s.end_line))
        .unwrap_or_else(|| "L1".to_string());

    // Edges
    let node_id = node.id.clone();
    let outgoing_calls: Vec<String> = project
        .graph
        .edges
        .iter()
        .filter(|e| e.source == node_id && e.kind == ProjectEdgeKind::Calls)
        .map(|e| e.target.clone())
        .collect();

    let incoming_calls: Vec<String> = project
        .graph
        .edges
        .iter()
        .filter(|e| e.target == node_id && e.kind == ProjectEdgeKind::Calls)
        .map(|e| e.source.clone())
        .collect();

    let implements: Vec<String> = project
        .graph
        .edges
        .iter()
        .filter(|e| {
            (e.source == node_id || e.target == node_id) && e.kind == ProjectEdgeKind::Implements
        })
        .map(|e| {
            if e.source == node_id {
                e.target.clone()
            } else {
                e.source.clone()
            }
        })
        .collect();

    rsx! {
        div { style: "padding: 14px; display: flex; flex-direction: column; gap: 12px; border-bottom: 1px solid var(--border-subtle);",
            // Header row with Badges & Deselect
            div { style: "display: flex; align-items: center; justify-content: space-between;",
                div { style: "display: flex; align-items: center; gap: 6px;",
                    span {
                        style: "font-size: 10px; font-weight: 700; padding: 2px 6px; border-radius: 4px; background: {kind_bg}; color: {kind_fg}; letter-spacing: 0.5px;",
                        "{kind_label}"
                    }
                    if !node.visibility.is_empty() {
                        span {
                            style: "font-size: 10px; font-weight: 600; padding: 2px 6px; border-radius: 4px; background: rgba(255,255,255,0.06); color: var(--text-muted); font-family: monospace;",
                            "{node.visibility}"
                        }
                    }
                }
                button {
                    class: "btn-icon",
                    title: "Deselect symbol",
                    onclick: move |_| {
                        state.write().graph_view_state.selected_node_id = None;
                    },
                    IconClose { size: 12 }
                }
            }

            // Symbol Name
            div {
                style: "font-size: 15px; font-weight: 700; font-family: monospace; color: var(--text-primary); word-break: break-all; line-height: 1.3;",
                "{node.name}"
            }

            // File & Line Location Card
            div {
                style: "display: flex; flex-direction: column; gap: 6px; padding: 8px 10px; background: var(--bg-surface-elevated); border: 1px solid var(--border-subtle); border-radius: 6px; font-size: 11px;",
                div { style: "display: flex; align-items: center; justify-content: space-between;",
                    span { style: "color: var(--text-muted); font-family: monospace; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                        "{node.file}:{line_str}"
                    }
                }
                div { style: "display: flex; align-items: center; gap: 6px; margin-top: 2px;",
                    button {
                        class: "btn btn-xs btn-primary",
                        style: "display: inline-flex; align-items: center; gap: 4px; font-size: 11px; padding: 4px 8px;",
                        title: "Open in local editor at line {start_line}",
                        onclick: {
                            let root = project.root.clone();
                            let file = node.file.clone();
                            move |_| {
                                let full_path = root.join(&file);
                                open_file_in_editor(&full_path, start_line);
                            }
                        },
                        IconExternalLink { size: 11 }
                        span { "Open in Editor" }
                    }
                    button {
                        class: "btn btn-xs btn-secondary",
                        style: "display: inline-flex; align-items: center; gap: 4px; font-size: 11px; padding: 4px 8px;",
                        title: "Reveal file in file explorer",
                        onclick: {
                            let root = project.root.clone();
                            let file = node.file.clone();
                            move |_| {
                                let full_path = root.join(&file);
                                reveal_file_in_os(&full_path);
                            }
                        },
                        IconFolder { size: 11 }
                        span { "Reveal" }
                    }
                }
            }
        }

        // Section: Signature
        if let Some(sig) = &node.signature {
            div { style: "border-bottom: 1px solid var(--border-subtle);",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 8px 14px; cursor: pointer; user-select: none; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                    onclick: move |_| show_signature.toggle(),
                    div { style: "display: flex; align-items: center; gap: 6px;",
                        if *show_signature.read() { IconChevronDown { size: 12 } } else { IconChevronRight { size: 12 } }
                        span { "Signature" }
                    }
                }
                if *show_signature.read() {
                    div { style: "padding: 0 14px 10px 14px;",
                        pre {
                            style: "margin: 0; padding: 8px 10px; background: var(--bg-surface-elevated); border: 1px solid var(--border-subtle); border-radius: 6px; font-family: monospace; font-size: 11px; color: var(--text-primary); overflow-x: auto; white-space: pre-wrap; line-height: 1.4;",
                            "{sig}"
                        }
                    }
                }
            }
        }

        // Section: Documentation
        if let Some(doc) = &node.doc {
            div { style: "border-bottom: 1px solid var(--border-subtle);",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 8px 14px; cursor: pointer; user-select: none; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                    onclick: move |_| show_doc.toggle(),
                    div { style: "display: flex; align-items: center; gap: 6px;",
                        if *show_doc.read() { IconChevronDown { size: 12 } } else { IconChevronRight { size: 12 } }
                        span { "Documentation" }
                    }
                }
                if *show_doc.read() {
                    div { style: "padding: 0 14px 10px 14px;",
                        div {
                            style: "font-size: 12px; color: var(--text-secondary); line-height: 1.5; white-space: pre-wrap;",
                            "{doc}"
                        }
                    }
                }
            }
        }

        // Section: Connected Architecture / Edges
        div { style: "border-bottom: 1px solid var(--border-subtle);",
            div {
                style: "display: flex; align-items: center; justify-content: space-between; padding: 8px 14px; cursor: pointer; user-select: none; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary);",
                onclick: move |_| show_edges.toggle(),
                div { style: "display: flex; align-items: center; gap: 6px;",
                    if *show_edges.read() { IconChevronDown { size: 12 } } else { IconChevronRight { size: 12 } }
                    span { "Connected Symbols ({outgoing_calls.len() + incoming_calls.len() + implements.len()})" }
                }
            }
            if *show_edges.read() {
                div { style: "padding: 0 14px 12px 14px; display: flex; flex-direction: column; gap: 10px;",
                    if !outgoing_calls.is_empty() {
                        div {
                            div { style: "font-size: 10px; font-weight: 600; text-transform: uppercase; color: var(--text-muted); margin-bottom: 4px;", "Calls" }
                            div { style: "display: flex; flex-wrap: wrap; gap: 4px;",
                                for target_id in &outgoing_calls {
                                    {
                                        let target_name = project.graph.nodes.iter().find(|n| n.id == *target_id).map(|n| n.name.clone()).unwrap_or_else(|| target_id.clone());
                                        let tid = target_id.clone();
                                        rsx! {
                                            button {
                                                class: "btn btn-xs",
                                                style: "font-family: monospace; font-size: 10px; padding: 2px 6px; background: var(--bg-surface-elevated); border: 1px solid var(--border-subtle); border-radius: 4px; cursor: pointer; color: var(--accent);",
                                                title: "Inspect {tid}",
                                                onclick: move |_| {
                                                    state.write().graph_view_state.selected_node_id = Some(tid.clone());
                                                },
                                                "{target_name}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !incoming_calls.is_empty() {
                        div {
                            div { style: "font-size: 10px; font-weight: 600; text-transform: uppercase; color: var(--text-muted); margin-bottom: 4px;", "Called By" }
                            div { style: "display: flex; flex-wrap: wrap; gap: 4px;",
                                for caller_id in &incoming_calls {
                                    {
                                        let caller_name = project.graph.nodes.iter().find(|n| n.id == *caller_id).map(|n| n.name.clone()).unwrap_or_else(|| caller_id.clone());
                                        let cid = caller_id.clone();
                                        rsx! {
                                            button {
                                                class: "btn btn-xs",
                                                style: "font-family: monospace; font-size: 10px; padding: 2px 6px; background: var(--bg-surface-elevated); border: 1px solid var(--border-subtle); border-radius: 4px; cursor: pointer; color: var(--accent);",
                                                title: "Inspect {cid}",
                                                onclick: move |_| {
                                                    state.write().graph_view_state.selected_node_id = Some(cid.clone());
                                                },
                                                "{caller_name}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !implements.is_empty() {
                        div {
                            div { style: "font-size: 10px; font-weight: 600; text-transform: uppercase; color: var(--text-muted); margin-bottom: 4px;", "Implements / Traits" }
                            div { style: "display: flex; flex-wrap: wrap; gap: 4px;",
                                for impl_id in &implements {
                                    {
                                        let impl_name = project.graph.nodes.iter().find(|n| n.id == *impl_id).map(|n| n.name.clone()).unwrap_or_else(|| impl_id.clone());
                                        let iid = impl_id.clone();
                                        rsx! {
                                            button {
                                                class: "btn btn-xs",
                                                style: "font-family: monospace; font-size: 10px; padding: 2px 6px; background: var(--bg-surface-elevated); border: 1px solid var(--border-subtle); border-radius: 4px; cursor: pointer; color: var(--accent);",
                                                title: "Inspect {iid}",
                                                onclick: move |_| {
                                                    state.write().graph_view_state.selected_node_id = Some(iid.clone());
                                                },
                                                "{impl_name}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if outgoing_calls.is_empty() && incoming_calls.is_empty() && implements.is_empty() {
                        div { style: "font-size: 11px; color: var(--text-muted); font-style: italic;",
                            "No cross-symbol calls or trait references detected."
                        }
                    }
                }
            }
        }
    }
}

/// Overview panel displayed when viewing an external project before any symbol is selected.
#[component]
fn ProjectOverviewInspector(project: ProjectContext) -> Element {
    let mut fn_count = 0;
    let mut struct_count = 0;
    let mut enum_count = 0;
    let mut trait_count = 0;
    let mut mod_count = 0;

    for n in &project.graph.nodes {
        match n.kind {
            ProjectNodeKind::Function | ProjectNodeKind::Method => fn_count += 1,
            ProjectNodeKind::Struct => struct_count += 1,
            ProjectNodeKind::Enum | ProjectNodeKind::EnumVariant => enum_count += 1,
            ProjectNodeKind::Trait => trait_count += 1,
            ProjectNodeKind::Module => mod_count += 1,
            _ => {}
        }
    }

    let files_count = project.graph.files.len();
    let symbols_count = project.graph.nodes.len();
    let edges_count = project.graph.edges.len();

    rsx! {
        div { style: "padding: 14px; display: flex; flex-direction: column; gap: 14px;",
            // Project Header
            div {
                div { style: "font-size: 16px; font-weight: 700; color: var(--text-primary); margin-bottom: 2px;",
                    "📦 {project.name}"
                }
                div { style: "font-size: 11px; color: var(--text-muted); font-family: monospace; word-break: break-all;",
                    "{project.root.display()}"
                }
            }

            // Metrics row
            div { style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px;",
                div { style: "background: var(--bg-surface-elevated); border: 1px solid var(--border-subtle); border-radius: 6px; padding: 8px; text-align: center;",
                    div { style: "font-size: 16px; font-weight: 700; color: var(--accent);", "{files_count}" }
                    div { style: "font-size: 10px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;", "Files" }
                }
                div { style: "background: var(--bg-surface-elevated); border: 1px solid var(--border-subtle); border-radius: 6px; padding: 8px; text-align: center;",
                    div { style: "font-size: 16px; font-weight: 700; color: #a855f7;", "{symbols_count}" }
                    div { style: "font-size: 10px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;", "Symbols" }
                }
                div { style: "background: var(--bg-surface-elevated); border: 1px solid var(--border-subtle); border-radius: 6px; padding: 8px; text-align: center;",
                    div { style: "font-size: 16px; font-weight: 700; color: #10b981;", "{edges_count}" }
                    div { style: "font-size: 10px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;", "Edges" }
                }
            }

            // Breakdown
            div { style: "background: var(--bg-surface-elevated); border: 1px solid var(--border-subtle); border-radius: 6px; padding: 10px; display: flex; flex-direction: column; gap: 6px; font-size: 11px;",
                div { style: "font-weight: 600; text-transform: uppercase; font-size: 10px; color: var(--text-muted); letter-spacing: 0.5px; margin-bottom: 2px;", "Symbol Inventory" }
                div { style: "display: flex; justify-content: space-between; align-items: center;",
                    span { style: "color: var(--text-secondary);", "Functions / Methods" }
                    span { style: "font-weight: 600; color: #60a5fa;", "{fn_count}" }
                }
                div { style: "display: flex; justify-content: space-between; align-items: center;",
                    span { style: "color: var(--text-secondary);", "Structs" }
                    span { style: "font-weight: 600; color: #c084fc;", "{struct_count}" }
                }
                div { style: "display: flex; justify-content: space-between; align-items: center;",
                    span { style: "color: var(--text-secondary);", "Enums" }
                    span { style: "font-weight: 600; color: #fb923c;", "{enum_count}" }
                }
                div { style: "display: flex; justify-content: space-between; align-items: center;",
                    span { style: "color: var(--text-secondary);", "Traits" }
                    span { style: "font-weight: 600; color: #34d399;", "{trait_count}" }
                }
                div { style: "display: flex; justify-content: space-between; align-items: center;",
                    span { style: "color: var(--text-secondary);", "Modules" }
                    span { style: "font-weight: 600; color: #fbbf24;", "{mod_count}" }
                }
            }

            // Help callout
            div { style: "padding: 10px; background: rgba(99, 102, 241, 0.08); border: 1px solid rgba(99, 102, 241, 0.2); border-radius: 6px; font-size: 11px; line-height: 1.5; color: var(--text-secondary);",
                "💡 Click any symbol node in the 2D code graph to inspect its Rust signature, docstrings, enclosing file, and jump directly into your local IDE."
            }
        }
    }
}

/// Opens the specified file in the local code editor or system default.
fn open_file_in_editor(path: &Path, line: usize) {
    let path_str = path.to_string_lossy().to_string();
    let goto_arg = format!("{}:{}", path_str, line);

    // 1. Try VS Code with line jump
    if let Ok(child) = std::process::Command::new("code")
        .args(["-g", &goto_arg])
        .spawn()
    {
        drop(child);
        return;
    }

    // 2. Try $VISUAL or $EDITOR
    if let Ok(editor) = std::env::var("VISUAL").or_else(|_| std::env::var("EDITOR")) {
        if let Ok(child) = std::process::Command::new(editor).arg(&path_str).spawn() {
            drop(child);
            return;
        }
    }

    // 3. Fallback on OS defaults
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &path_str])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&path_str).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&path_str)
            .spawn();
    }
}

/// Reveals the target file in the OS file manager (Explorer, Finder, Nautilus).
fn reveal_file_in_os(path: &Path) {
    let path_str = path.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .args(["/select,", &path_str])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .args(["-R", &path_str])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = path.parent() {
            let _ = std::process::Command::new("xdg-open")
                .arg(parent.to_string_lossy().to_string())
                .spawn();
        }
    }
}

/// Helper component rendering title, custom hex input, and standard swatches
#[component]
fn ColorRow(title: &'static str, current_val: String, on_change: EventHandler<String>) -> Element {
    let mut custom_text = use_signal(|| current_val.clone());
    rsx! {
        div { class: "graph-color-row",
            div { class: "graph-color-label-row",
                span { "{title}" }
                input {
                    class: "graph-color-custom-input",
                    placeholder: "#HEX",
                    maxlength: "7",
                    value: "{custom_text}",
                    oninput: move |evt: FormEvent| {
                        let val = evt.value();
                        custom_text.set(val.clone());
                        if (val.len() == 7 && val.starts_with('#')) || val.len() == 4 {
                            on_change.call(val);
                        }
                    }
                }
            }
            div { class: "graph-swatches",
                for &swatch in STANDARD_GRAPH_SWATCHES.iter() {
                    {
                        let is_active = current_val.eq_ignore_ascii_case(swatch);
                        rsx! {
                            button {
                                key: "{title}_{swatch}",
                                class: if is_active { "graph-swatch active" } else { "graph-swatch" },
                                style: "background-color: {swatch};",
                                title: "{swatch}",
                                onclick: {
                                    let s_col = swatch.to_string();
                                    move |_| {
                                        custom_text.set(s_col.clone());
                                        on_change.call(s_col.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Unified Graph Color and Palette Customization Panel
#[component]
fn GraphColorControls(mut state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let colors = app_state.preferences.graph_settings.colors.clone();

    let cur_center = colors.effective_center_color().to_string();
    let cur_sub = colors.effective_sub_node_color().to_string();
    let cur_selected = colors.effective_selected_color().to_string();
    let cur_edge = colors.effective_edge_color().to_string();
    let cur_text = colors.effective_text_color().to_string();
    let cur_palette = colors.palette;
    let opacity_pct = (colors.edge_opacity * 100.0).round() as u32;

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 10px;",
            // Palette Preset Dropdown
            div { style: "display: flex; flex-direction: column; gap: 4px;",
                label { style: "font-size: 11px; font-weight: 500; color: var(--text-secondary);", "Color Palette" }
                select {
                    class: "graph-color-palette-select",
                    value: match cur_palette {
                        GraphPalettePreset::NoderaTech => "nodera_tech",
                        GraphPalettePreset::Cyberpunk => "cyberpunk",
                        GraphPalettePreset::Emerald => "emerald",
                        GraphPalettePreset::SolarAmber => "solar_amber",
                        GraphPalettePreset::Dracula => "dracula",
                        GraphPalettePreset::Monochrome => "monochrome",
                        GraphPalettePreset::Custom => "custom",
                    },
                    onchange: move |evt: FormEvent| {
                        let val = evt.value();
                        let p = match val.as_str() {
                            "cyberpunk" => GraphPalettePreset::Cyberpunk,
                            "emerald" => GraphPalettePreset::Emerald,
                            "solar_amber" => GraphPalettePreset::SolarAmber,
                            "dracula" => GraphPalettePreset::Dracula,
                            "monochrome" => GraphPalettePreset::Monochrome,
                            "custom" => GraphPalettePreset::Custom,
                            _ => GraphPalettePreset::NoderaTech,
                        };
                        let mut s = state.write();
                        s.preferences.graph_settings.colors.palette = p;
                        s.preferences.graph_settings.colors.center_node_color = None;
                        s.preferences.graph_settings.colors.sub_node_color = None;
                        s.preferences.graph_settings.colors.selected_node_color = None;
                        s.preferences.graph_settings.colors.edge_color = None;
                        s.preferences.graph_settings.colors.text_color = None;
                        s.preferences.save();
                    },
                    option { value: "nodera_tech", "Nodera Tech (Default)" }
                    option { value: "cyberpunk", "Neon Cyberpunk" }
                    option { value: "emerald", "Nordic Emerald" }
                    option { value: "solar_amber", "Solar Amber" }
                    option { value: "dracula", "Dracula Synth" }
                    option { value: "monochrome", "Slate Monochrome" }
                    option { value: "custom", "Custom Palette" }
                }
            }

            // Semantic Symbol Kind Toggle
            div { class: "graph-toggle-row",
                span { "Color by Symbol Kind" }
                label { class: "graph-switch",
                    input {
                        r#type: "checkbox",
                        checked: colors.color_by_kind,
                        onchange: move |evt: FormEvent| {
                            let checked = evt.value() == "true";
                            let mut s = state.write();
                            s.preferences.graph_settings.colors.color_by_kind = checked;
                            s.preferences.save();
                        }
                    }
                    span { class: "graph-switch-slider" }
                }
            }

            // Center Node Color
            ColorRow {
                title: "Center Hub Node",
                current_val: cur_center,
                on_change: move |col: String| {
                    let mut s = state.write();
                    s.preferences.graph_settings.colors.center_node_color = Some(col);
                    s.preferences.graph_settings.colors.palette = GraphPalettePreset::Custom;
                    s.preferences.save();
                }
            }

            // Sub-nodes Color
            ColorRow {
                title: "Sub-Nodes / Symbols",
                current_val: cur_sub,
                on_change: move |col: String| {
                    let mut s = state.write();
                    s.preferences.graph_settings.colors.sub_node_color = Some(col);
                    s.preferences.graph_settings.colors.palette = GraphPalettePreset::Custom;
                    s.preferences.save();
                }
            }

            // Selected Node Color
            ColorRow {
                title: "Selected & Halo",
                current_val: cur_selected,
                on_change: move |col: String| {
                    let mut s = state.write();
                    s.preferences.graph_settings.colors.selected_node_color = Some(col);
                    s.preferences.graph_settings.colors.palette = GraphPalettePreset::Custom;
                    s.preferences.save();
                }
            }

            // Connection Lines (Edges) Color
            ColorRow {
                title: "Connection Lines (Edges)",
                current_val: cur_edge,
                on_change: move |col: String| {
                    let mut s = state.write();
                    s.preferences.graph_settings.colors.edge_color = Some(col);
                    s.preferences.graph_settings.colors.palette = GraphPalettePreset::Custom;
                    s.preferences.save();
                }
            }

            // Edge Opacity Slider
            div { class: "graph-slider-row",
                div { class: "graph-slider-header",
                    span { "Connection Opacity" }
                    span { class: "graph-slider-val", "{opacity_pct}%" }
                }
                input {
                    class: "graph-slider",
                    r#type: "range",
                    min: "0.10",
                    max: "1.00",
                    step: "0.05",
                    value: "{colors.edge_opacity}",
                    oninput: move |evt: FormEvent| {
                        if let Ok(v) = evt.value().parse::<f32>() {
                            let mut s = state.write();
                            s.preferences.graph_settings.colors.edge_opacity = v;
                            s.preferences.save();
                        }
                    }
                }
            }

            // Text Label Color
            ColorRow {
                title: "Label Typography",
                current_val: cur_text,
                on_change: move |col: String| {
                    let mut s = state.write();
                    s.preferences.graph_settings.colors.text_color = Some(col);
                    s.preferences.graph_settings.colors.palette = GraphPalettePreset::Custom;
                    s.preferences.save();
                }
            }

            // Reset Colors button
            button {
                class: "btn btn-xs btn-secondary",
                style: "margin-top: 4px; align-self: flex-start; font-size: 11px;",
                onclick: move |_| {
                    let mut s = state.write();
                    s.preferences.graph_settings.colors = Default::default();
                    s.preferences.save();
                },
                "Reset Colors to Default"
            }
        }
    }
}
