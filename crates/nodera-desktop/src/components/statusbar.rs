use dioxus::prelude::*;

use crate::state::AppState;
use crate::strings::empty_states;

#[component]
pub fn StatusBar(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let status_message = app_state.status_message.clone();
    let active_path = app_state
        .active_note
        .as_ref()
        .map(|n| n.relative_path.display().to_string())
        .unwrap_or_else(|| empty_states::NO_FILE_OPEN.to_string());
    let notes_count = app_state
        .entries
        .iter()
        .filter(|e| matches!(e, nodera_core::VaultEntry::Note(_)))
        .count();

    rsx! {
        footer { class: "bottom-statusbar",
            div { style: "display: flex; align-items: center; gap: 8px;",
                span { style: "width: 6px; height: 6px; border-radius: 50%; background-color: var(--accent); display: inline-block;" }
                span { "{status_message}" }
            }
            div { style: "font-family: var(--font-editor); opacity: 0.8;",
                "{active_path}"
            }
            div { style: "display: flex; align-items: center; gap: 12px;",
                span { "{notes_count} notes" }
                span { "UTF-8" }
            }
        }
    }
}
