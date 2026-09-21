use chrono::{Datelike, Duration, NaiveDate};

/// Represents an individual day cell in a calendar month grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayCell {
    pub date: NaiveDate,
    pub day_number: u32,
    pub is_current_month: bool,
    pub is_today: bool,
    pub is_selected: bool,
    pub has_daily_note: bool,
    pub is_future: bool,
    pub aria_label: String,
}

/// Represents a single week (row) in a calendar grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarWeek {
    pub days: Vec<DayCell>,
}

/// Typed representation of a full monthly calendar grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarMonth {
    pub year: i32,
    pub month: u32,
    pub month_name: String,
    pub weeks: Vec<CalendarWeek>,
}

/// In-memory view state for the calendar view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarState {
    pub view_year: i32,
    pub view_month: u32,
    pub selected_date: NaiveDate,
}

impl CalendarState {
    /// Creates calendar state initialized to the given date (typically today).
    pub fn new(date: NaiveDate) -> Self {
        Self {
            view_year: date.year(),
            view_month: date.month(),
            selected_date: date,
        }
    }

    /// Advances the calendar view to the next month.
    pub fn next_month(&mut self) {
        if self.view_month == 12 {
            self.view_year += 1;
            self.view_month = 1;
        } else {
            self.view_month += 1;
        }
    }

    /// Moves the calendar view to the previous month.
    pub fn prev_month(&mut self) {
        if self.view_month == 1 {
            self.view_year -= 1;
            self.view_month = 12;
        } else {
            self.view_month -= 1;
        }
    }

    /// Resets the calendar view and selection to the provided today's date.
    pub fn go_to_today(&mut self, today: NaiveDate) {
        self.view_year = today.year();
        self.view_month = today.month();
        self.selected_date = today;
    }

    /// Selects a specific date, updating the view month/year if outside current view.
    pub fn select_date(&mut self, date: NaiveDate) {
        self.selected_date = date;
        self.view_year = date.year();
        self.view_month = date.month();
    }
}

impl Default for CalendarState {
    fn default() -> Self {
        let today = chrono::Local::now().date_naive();
        Self::new(today)
    }
}

/// Returns the English name of the given month number (1..=12).
pub fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

/// Returns the English weekday abbreviation for columns (Mon..Sun).
pub fn weekday_short_names() -> [&'static str; 7] {
    ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
}

/// Generates a complete 42-day (6-week) calendar grid starting on Monday for the given year and month.
pub fn generate_calendar_month(
    year: i32,
    month: u32,
    selected_date: NaiveDate,
    today: NaiveDate,
    has_note: impl Fn(NaiveDate) -> bool,
) -> CalendarMonth {
    let first_of_month = NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());

    // Monday is index 0
    let leading_days_needed = first_of_month.weekday().num_days_from_monday();

    // Start date for the first cell in the 42-day grid
    let grid_start_date = first_of_month - Duration::days(leading_days_needed as i64);

    let mut weeks = Vec::with_capacity(6);
    let mut current_date = grid_start_date;

    for _week_idx in 0..6 {
        let mut days = Vec::with_capacity(7);
        for _day_idx in 0..7 {
            let is_current_month = current_date.year() == year && current_date.month() == month;
            let is_today = current_date == today;
            let is_selected = current_date == selected_date;
            let has_daily_note = has_note(current_date);
            let is_future = current_date > today;

            // Construct accessible label
            let mut label_parts = Vec::new();
            label_parts.push(format!(
                "{} {}, {}",
                month_name(current_date.month()),
                current_date.day(),
                current_date.year()
            ));

            if has_daily_note {
                label_parts.push("daily note exists".to_string());
            }
            if is_today {
                label_parts.push("today".to_string());
            }
            if is_selected {
                label_parts.push("selected".to_string());
            }
            if is_future {
                label_parts.push("future date".to_string());
            }

            let aria_label = label_parts.join(", ");

            days.push(DayCell {
                date: current_date,
                day_number: current_date.day(),
                is_current_month,
                is_today,
                is_selected,
                has_daily_note,
                is_future,
                aria_label,
            });

            current_date += Duration::days(1);
        }
        weeks.push(CalendarWeek { days });
    }

    CalendarMonth {
        year,
        month,
        month_name: month_name(month).to_string(),
        weeks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_month_structure_and_cell_count() {
        let today = NaiveDate::from_ymd_opt(2026, 9, 21).unwrap();
        let selected = today;
        let month = generate_calendar_month(2026, 9, selected, today, |_| false);

        assert_eq!(month.year, 2026);
        assert_eq!(month.month, 9);
        assert_eq!(month.month_name, "September");
        assert_eq!(month.weeks.len(), 6);

        let total_cells: usize = month.weeks.iter().map(|w| w.days.len()).sum();
        assert_eq!(total_cells, 42);

        // First cell for September 2026 (Sept 1 is Tuesday, so first cell is Monday Aug 31)
        let first_cell = &month.weeks[0].days[0];
        assert_eq!(
            first_cell.date,
            NaiveDate::from_ymd_opt(2026, 8, 31).unwrap()
        );
        assert!(!first_cell.is_current_month);
        assert_eq!(first_cell.day_number, 31);

        // Sept 1 is Tuesday (index 1)
        let sept_1 = &month.weeks[0].days[1];
        assert_eq!(sept_1.date, NaiveDate::from_ymd_opt(2026, 9, 1).unwrap());
        assert!(sept_1.is_current_month);
        assert_eq!(sept_1.day_number, 1);
    }

    #[test]
    fn test_leap_year_february_2024_vs_2026() {
        let today = NaiveDate::from_ymd_opt(2026, 9, 21).unwrap();

        // 2024 is a leap year (29 days in Feb)
        let feb_2024 = generate_calendar_month(2024, 2, today, today, |_| false);
        let feb_2024_days: Vec<_> = feb_2024
            .weeks
            .iter()
            .flat_map(|w| &w.days)
            .filter(|d| d.is_current_month)
            .collect();
        assert_eq!(feb_2024_days.len(), 29);
        assert_eq!(feb_2024_days.last().unwrap().day_number, 29);

        // 2026 is NOT a leap year (28 days in Feb)
        let feb_2026 = generate_calendar_month(2026, 2, today, today, |_| false);
        let feb_2026_days: Vec<_> = feb_2026
            .weeks
            .iter()
            .flat_map(|w| &w.days)
            .filter(|d| d.is_current_month)
            .collect();
        assert_eq!(feb_2026_days.len(), 28);
        assert_eq!(feb_2026_days.last().unwrap().day_number, 28);
    }

    #[test]
    fn test_month_navigation_and_year_wraparound() {
        let today = NaiveDate::from_ymd_opt(2026, 12, 15).unwrap();
        let mut state = CalendarState::new(today);

        assert_eq!(state.view_year, 2026);
        assert_eq!(state.view_month, 12);

        // Next month wraps around to Jan 2027
        state.next_month();
        assert_eq!(state.view_year, 2027);
        assert_eq!(state.view_month, 1);

        // Previous month wraps back to Dec 2026
        state.prev_month();
        assert_eq!(state.view_year, 2026);
        assert_eq!(state.view_month, 12);

        // Go to today
        let oct_date = NaiveDate::from_ymd_opt(2026, 10, 5).unwrap();
        state.go_to_today(oct_date);
        assert_eq!(state.view_year, 2026);
        assert_eq!(state.view_month, 10);
        assert_eq!(state.selected_date, oct_date);
    }

    #[test]
    fn test_daily_note_and_today_indicators_and_aria_labels() {
        let today = NaiveDate::from_ymd_opt(2026, 9, 21).unwrap();
        let selected = today;
        let note_dates = [NaiveDate::from_ymd_opt(2026, 9, 21).unwrap()];

        let month = generate_calendar_month(2026, 9, selected, today, |d| note_dates.contains(&d));

        let cell_21 = month
            .weeks
            .iter()
            .flat_map(|w| &w.days)
            .find(|d| d.date == today)
            .expect("September 21 must be present");

        assert!(cell_21.is_today);
        assert!(cell_21.is_selected);
        assert!(cell_21.has_daily_note);
        assert!(!cell_21.is_future);
        assert!(cell_21.aria_label.contains("September 21, 2026"));
        assert!(cell_21.aria_label.contains("daily note exists"));
        assert!(cell_21.aria_label.contains("today"));
        assert!(cell_21.aria_label.contains("selected"));
    }
}
