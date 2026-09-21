use chrono::{Duration, Local, NaiveDate};
use dioxus::prelude::*;

use crate::calendar::weekday_short_names;
use crate::icons::{IconCalendar, IconChevronLeft, IconChevronRight};
use crate::state::AppState;

fn handle_cell_click(mut state: Signal<AppState>, date: NaiveDate) {
    let mut s = state.write();
    let _ = s.calendar_select_and_open_date(date);
}

fn handle_cell_keydown(mut state: Signal<AppState>, evt: KeyboardEvent, current_date: NaiveDate) {
    let key = evt.key();
    let mut target_date = None;

    match key {
        Key::ArrowLeft => {
            target_date = Some(current_date - Duration::days(1));
        }
        Key::ArrowRight => {
            target_date = Some(current_date + Duration::days(1));
        }
        Key::ArrowUp => {
            target_date = Some(current_date - Duration::days(7));
        }
        Key::ArrowDown => {
            target_date = Some(current_date + Duration::days(7));
        }
        Key::Enter => {
            let mut s = state.write();
            let _ = s.calendar_select_and_open_date(current_date);
        }
        Key::Character(ref c) if c == " " => {
            let mut s = state.write();
            let _ = s.calendar_select_and_open_date(current_date);
        }
        _ => {}
    }

    if let Some(target) = target_date {
        evt.prevent_default();
        let mut s = state.write();
        s.calendar_select_date(target);
    }
}

#[component]
pub fn TodayView(state: Signal<AppState>) -> Element {
    let app_state = state.read();
    let month_data = app_state.get_calendar_month();
    let selected_date = app_state.calendar_state.selected_date;
    let selected_has_note = app_state.daily_note_exists_for_date(selected_date);
    let today = Local::now().date_naive();
    let today_long_str = today.format("%A, %B %d, %Y").to_string();
    let selected_long_str = selected_date.format("%A, %B %d, %Y").to_string();

    rsx! {
        main {
            class: "pane-center",
            style: "display: flex; flex-direction: column; overflow-y: auto; align-items: center; padding: 24px 20px;",
            role: "region",
            aria_label: "Today Calendar Workspace",

            // Container card centered with bounded width
            div {
                style: "width: 100%; max-width: 520px; display: flex; flex-direction: column; gap: 20px;",

                // Header: Today title and current date
                header {
                    style: "display: flex; align-items: center; justify-content: space-between; padding-bottom: 12px; border-bottom: 1px solid var(--border);",
                    div { style: "display: flex; flex-direction: column; gap: 2px;",
                        div { style: "display: flex; align-items: center; gap: 8px;",
                            IconCalendar { size: 18 }
                            h2 { style: "margin: 0; font-size: 18px; font-weight: 700; color: var(--text-primary);", "Today" }
                        }
                        span { style: "font-size: 12px; color: var(--text-muted);", "{today_long_str}" }
                    }

                    button {
                        class: "btn-action",
                        title: "Jump calendar to today",
                        onclick: move |_| {
                            let mut s = state.write();
                            s.calendar_go_to_today();
                        },
                        "Today"
                    }
                }

                // Month Navigation Toolbar
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 0 4px;",
                    button {
                        class: "btn-icon",
                        title: "Previous month",
                        aria_label: "Previous month",
                        onclick: move |_| {
                            let mut s = state.write();
                            s.calendar_prev_month();
                        },
                        IconChevronLeft { size: 16 }
                    }

                    span {
                        style: "font-size: 15px; font-weight: 600; color: var(--text-primary); letter-spacing: 0.2px;",
                        "{month_data.month_name} {month_data.year}"
                    }

                    button {
                        class: "btn-icon",
                        title: "Next month",
                        aria_label: "Next month",
                        onclick: move |_| {
                            let mut s = state.write();
                            s.calendar_next_month();
                        },
                        IconChevronRight { size: 16 }
                    }
                }

                // Calendar Grid Table / Container
                div {
                    role: "grid",
                    aria_label: "{month_data.month_name} {month_data.year} Calendar",
                    style: "display: flex; flex-direction: column; gap: 6px; background-color: var(--bg-surface); border: 1px solid var(--border-subtle); border-radius: 8px; padding: 12px;",

                    // Weekday header row (Mon .. Sun)
                    div {
                        role: "row",
                        style: "display: grid; grid-template-columns: repeat(7, 1fr); gap: 4px; text-align: center; padding-bottom: 4px; border-bottom: 1px solid var(--border-subtle);",
                        for name in weekday_short_names() {
                            div {
                                key: "{name}",
                                role: "columnheader",
                                style: "font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; padding: 4px 0;",
                                "{name}"
                            }
                        }
                    }

                    // Weeks grid
                    for (w_idx, week) in month_data.weeks.iter().enumerate() {
                        div {
                            key: "week-{w_idx}",
                            role: "row",
                            style: "display: grid; grid-template-columns: repeat(7, 1fr); gap: 4px;",

                            for cell in &week.days {
                                {
                                    let cell_date = cell.date;
                                    let day_num = cell.day_number;
                                    let is_cur = cell.is_current_month;
                                    let is_tod = cell.is_today;
                                    let is_sel = cell.is_selected;
                                    let has_note = cell.has_daily_note;
                                    let aria_lbl = cell.aria_label.clone();

                                    // Construct styling using restrained semantic tokens
                                    let mut cell_style = String::from("height: 44px; display: flex; flex-direction: column; align-items: center; justify-content: center; border-radius: 6px; cursor: pointer; transition: background-color 0.12s, border-color 0.12s; position: relative; border: 1px solid transparent; outline: none;");

                                    if is_sel {
                                        cell_style.push_str(" background-color: var(--bg-active); border-color: var(--accent);");
                                    } else if is_tod {
                                        cell_style.push_str(" border-color: var(--accent-hover); background-color: var(--bg-surface-elevated);");
                                    }

                                    let text_color = if is_cur {
                                        "var(--text-primary)"
                                    } else {
                                        "var(--text-muted)"
                                    };

                                    let font_weight = if is_tod || is_sel {
                                        "600"
                                    } else {
                                        "400"
                                    };

                                    rsx! {
                                        button {
                                            key: "{cell_date}",
                                            role: "gridcell",
                                            aria_selected: "{is_sel}",
                                            aria_label: "{aria_lbl}",
                                            tabindex: if is_sel { "0" } else { "-1" },
                                            style: "{cell_style}",
                                            class: "calendar-day-cell",
                                            onclick: move |_| handle_cell_click(state, cell_date),
                                            onkeydown: move |evt| handle_cell_keydown(state, evt, cell_date),

                                            div {
                                                style: "display: inline-flex; align-items: center; justify-content: center; font-size: 13px; font-weight: {font_weight}; color: {text_color};",
                                                span { "{day_num}" }
                                                if has_note {
                                                    span {
                                                        style: "font-size: 14px; line-height: 1; color: var(--accent); margin-left: 2px;",
                                                        title: "Daily note exists",
                                                        "•"
                                                    }
                                                }
                                            }

                                            if is_tod && !is_sel {
                                                div {
                                                    style: "width: 14px; height: 2px; background-color: var(--accent); border-radius: 1px; margin-top: 2px;",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Selected Date Summary & Action Panel
                div {
                    style: "background-color: var(--bg-surface); border: 1px solid var(--border-subtle); border-radius: 8px; padding: 14px 16px; display: flex; align-items: center; justify-content: space-between;",
                    div { style: "display: flex; flex-direction: column; gap: 2px;",
                        span { style: "font-size: 11px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px;", "Selected date" }
                        span { style: "font-size: 14px; font-weight: 600; color: var(--text-primary);", "{selected_long_str}" }
                    }

                    button {
                        class: if selected_has_note { "btn-action" } else { "btn-action btn-primary" },
                        onclick: move |_| {
                            let mut s = state.write();
                            let _ = s.open_or_create_daily_note_for(selected_date);
                        },
                        if selected_has_note {
                            "Open Daily Note"
                        } else {
                            "Create Daily Note"
                        }
                    }
                }
            }
        }
    }
}
