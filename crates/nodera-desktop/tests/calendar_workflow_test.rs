use chrono::NaiveDate;
use nodera_desktop::calendar::{generate_calendar_month, CalendarState};
use nodera_desktop::state::{ActiveView, AppState, PaletteAction};
use std::path::PathBuf;

#[test]
fn test_calendar_domain_model_unit_tests() {
    let today = NaiveDate::from_ymd_opt(2026, 9, 21).unwrap();
    let selected = today;

    // 1. Month generation for 2026-02 (non-leap year: 28 days)
    let feb_2026 = generate_calendar_month(2026, 2, selected, today, |_| false);
    assert_eq!(feb_2026.year, 2026);
    assert_eq!(feb_2026.month, 2);
    assert_eq!(feb_2026.weeks.len(), 6);
    let feb_2026_days: Vec<_> = feb_2026
        .weeks
        .iter()
        .flat_map(|w| &w.days)
        .filter(|d| d.is_current_month)
        .collect();
    assert_eq!(feb_2026_days.len(), 28);
    assert_eq!(feb_2026_days.first().unwrap().day_number, 1);
    assert_eq!(feb_2026_days.last().unwrap().day_number, 28);

    // 2. Leap year for 2024-02 (leap year: 29 days)
    let feb_2024 = generate_calendar_month(2024, 2, selected, today, |_| false);
    let feb_2024_days: Vec<_> = feb_2024
        .weeks
        .iter()
        .flat_map(|w| &w.days)
        .filter(|d| d.is_current_month)
        .collect();
    assert_eq!(feb_2024_days.len(), 29);
    assert_eq!(feb_2024_days.last().unwrap().day_number, 29);

    // 3. Month boundaries and year rollover (2026-12 -> 2027-01)
    let mut cal_state = CalendarState::new(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
    assert_eq!(cal_state.view_year, 2026);
    assert_eq!(cal_state.view_month, 12);
    cal_state.next_month();
    assert_eq!(cal_state.view_year, 2027);
    assert_eq!(cal_state.view_month, 1);
    cal_state.prev_month();
    assert_eq!(cal_state.view_year, 2026);
    assert_eq!(cal_state.view_month, 12);

    // 4. Existing note detection and accessibility aria-labels
    let note_date = NaiveDate::from_ymd_opt(2026, 9, 21).unwrap();
    let sept_2026 = generate_calendar_month(2026, 9, note_date, note_date, |d| d == note_date);
    let target_cell = sept_2026
        .weeks
        .iter()
        .flat_map(|w| &w.days)
        .find(|d| d.date == note_date)
        .expect("Cell for 2026-09-21 must exist");

    assert!(target_cell.is_today);
    assert!(target_cell.is_selected);
    assert!(target_cell.has_daily_note);
    assert!(target_cell.aria_label.contains("September 21, 2026"));
    assert!(target_cell.aria_label.contains("daily note exists"));
    assert!(target_cell.aria_label.contains("today"));
}

#[test]
fn test_calendar_and_daily_note_integration_workflow() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("CalendarTestVault");

    let mut state = AppState::default();
    state
        .create_vault(&vault_path, Some("Calendar Vault".to_string()))
        .unwrap();

    // Verify initial calendar state
    assert_eq!(state.daily_notes_folder(), "Daily");

    // 1. Create daily note for selected date
    let date_1 = NaiveDate::from_ymd_opt(2026, 9, 21).unwrap();
    state.open_or_create_daily_note_for(date_1).unwrap();

    // Verify view, active note, and path
    assert_eq!(state.active_view, ActiveView::Editor);
    let active = state.active_note.as_ref().expect("Active note must exist");
    assert_eq!(
        active.relative_path,
        PathBuf::from("Daily").join("2026-09-21.md")
    );

    // 3. Verify frontmatter correctness
    assert!(state.editor_content.starts_with("---\n"));
    assert!(state.editor_content.contains("title: 2026-09-21"));
    assert!(state.editor_content.contains("date: 2026-09-21"));
    assert!(state.editor_content.contains("tags:\n  - daily"));

    // 4. Verify template correctness
    assert!(state.editor_content.contains("# Daily Note — 2026-09-21"));
    assert!(state.editor_content.contains("## Tasks"));
    assert!(state.editor_content.contains("## Notes"));

    // Verify presence detection
    assert!(state.daily_note_exists_for_date(date_1));
    let existing_dates = state.get_existing_daily_note_dates();
    assert!(existing_dates.contains(&date_1));

    // 2 & 7. Reopen existing daily note and preserve markdown without duplicate creation
    state
        .editor_content
        .push_str("\n- [x] Attended design review\nCustom user journal entry.");
    state.save_active_note().unwrap();

    // Navigate away to a different note
    state.create_note("Scratchpad", None).unwrap();
    assert_ne!(
        state.active_note.as_ref().unwrap().relative_path,
        PathBuf::from("Daily").join("2026-09-21.md")
    );

    // Reopen the daily note for 2026-09-21
    state.open_or_create_daily_note_for(date_1).unwrap();
    assert_eq!(
        state.active_note.as_ref().unwrap().relative_path,
        PathBuf::from("Daily").join("2026-09-21.md")
    );
    assert!(state.editor_content.contains("Attended design review"));
    assert!(state.editor_content.contains("Custom user journal entry."));

    // 5. Test leap-year daily note creation (2024-02-29)
    let leap_date = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
    state.open_or_create_daily_note_for(leap_date).unwrap();
    assert_eq!(
        state.active_note.as_ref().unwrap().relative_path,
        PathBuf::from("Daily").join("2024-02-29.md")
    );
    assert!(state.editor_content.contains("2024-02-29"));

    // Test month-boundary and year rollover notes (2026-12-31 and 2027-01-01)
    let dec31 = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
    state.open_or_create_daily_note_for(dec31).unwrap();
    assert!(state.daily_note_exists_for_date(dec31));

    let jan1 = NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();
    state.open_or_create_daily_note_for(jan1).unwrap();
    assert!(state.daily_note_exists_for_date(jan1));

    // 8. Test index refresh / note lookup
    state.refresh_entries().unwrap();
    let dates = state.get_existing_daily_note_dates();
    assert!(dates.contains(&date_1));
    assert!(dates.contains(&leap_date));
    assert!(dates.contains(&dec31));
    assert!(dates.contains(&jan1));

    // 9. Test calendar month view generation from AppState
    state.calendar_state.view_year = 2026;
    state.calendar_state.view_month = 9;
    let month_view = state.get_calendar_month();
    assert_eq!(month_view.year, 2026);
    assert_eq!(month_view.month, 9);
    assert_eq!(month_view.month_name, "September");

    // Verify that the cell for 2026-09-21 reports has_daily_note = true
    let sept21_cell = month_view
        .weeks
        .iter()
        .flat_map(|w| &w.days)
        .find(|d| d.date == date_1)
        .expect("Cell for 2026-09-21 must be present");
    assert!(sept21_cell.has_daily_note);

    // Verify cell for 2026-09-22 reports has_daily_note = false
    let sept22_date = NaiveDate::from_ymd_opt(2026, 9, 22).unwrap();
    let sept22_cell = month_view
        .weeks
        .iter()
        .flat_map(|w| &w.days)
        .find(|d| d.date == sept22_date)
        .expect("Cell for 2026-09-22 must be present");
    assert!(!sept22_cell.has_daily_note);

    // Test calendar navigation methods
    state.calendar_next_month();
    assert_eq!(state.calendar_state.view_month, 10);
    state.calendar_prev_month();
    assert_eq!(state.calendar_state.view_month, 9);
    state.calendar_go_to_today();
    assert_eq!(
        state.calendar_state.selected_date,
        chrono::Local::now().date_naive()
    );

    // Test convenience wrapper for today's daily note
    state.open_or_create_daily_note().unwrap();
    let today = chrono::Local::now().date_naive();
    assert_eq!(
        state.active_note.as_ref().unwrap().relative_path,
        state.daily_note_path_for_date(today)
    );

    // Test Command Palette actions
    state
        .execute_palette_action(PaletteAction::SwitchView(ActiveView::Today))
        .unwrap();
    assert_eq!(state.active_view, ActiveView::Today);

    state
        .execute_palette_action(PaletteAction::OpenGoToDateDialog)
        .unwrap();
    assert!(state.show_go_to_date_dialog);

    state
        .execute_palette_action(PaletteAction::OpenDailyNote)
        .unwrap();
    assert_eq!(state.active_view, ActiveView::Editor);

    // Test Command Palette list items
    let palette_items = state.get_command_palette_items();
    assert!(palette_items.iter().any(|i| i.title == "Open Today"));
    assert!(palette_items.iter().any(|i| i.title == "Open Calendar"));
    assert!(palette_items
        .iter()
        .any(|i| i.title == "Open Today's Daily Note"));
    assert!(palette_items
        .iter()
        .any(|i| i.title == "Go to Daily Note for Date..."));

    // Test cycle view includes Today
    state.active_view = ActiveView::Editor;
    state.cycle_view();
    assert_eq!(state.active_view, ActiveView::Today);
    state.cycle_view();
    assert_eq!(state.active_view, ActiveView::Tasks);

    // Test external file detection
    let aug_date = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();
    let ext_path = vault_path.join("Daily").join("2026-08-15.md");
    std::fs::write(&ext_path, "# External note").unwrap();
    state.refresh_entries().unwrap();
    assert!(state.daily_note_exists_for_date(aug_date));
}

#[test]
fn test_custom_daily_template_workflow() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("CustomTemplateVault");

    let mut state = AppState::default();
    state
        .create_vault(&vault_path, Some("Custom Template Vault".to_string()))
        .unwrap();

    // Create custom daily template in Templates/
    let tmpl_dir = vault_path.join("Templates");
    std::fs::create_dir_all(&tmpl_dir).unwrap();
    let tmpl_content = "---\ntitle: \"{{title}}\"\ntags:\n  - custom-daily\n---\n# Focus: {{date}}\n\n## Custom Section\n- [ ] Task from template\n";
    std::fs::write(tmpl_dir.join("Daily Note.md"), tmpl_content).unwrap();

    state.refresh_entries().unwrap();

    // Create note for a specific date
    let test_date = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
    state.open_or_create_daily_note_for(test_date).unwrap();

    let content = &state.editor_content;
    assert!(content.contains("tags:\n  - custom-daily"));
    assert!(content.contains("# Focus: 2026-05-10"));
    assert!(content.contains("## Custom Section"));
    assert!(content.contains("- [ ] Task from template"));
}
