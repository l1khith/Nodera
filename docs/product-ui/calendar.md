# Track 3 UI/UX Handoff: Calendar & Today Workspace

## Overview

The Calendar and Today workspace provides native, Markdown-backed date navigation, daily note navigation, and safe daily note creation in Nodera. 

Markdown files (`Daily/YYYY-MM-DD.md`) remain the authoritative single source of truth. The calendar derives all indicators directly from in-memory indexed note entries without a proprietary database or full-vault scans.

---

## 1. Intended Placement & Layout

### Sidebar Workspace Navigation
- Placed directly in the primary `WORKSPACE` group in the sidebar:
  ```
  WORKSPACE
      Notes
      Today
      Tasks
      Review Queue
      Library
      Graph
  ```
- Icon: `IconCalendar` (15px).
- Label: `Today`.
- Active view enum: `ActiveView::Today`.

### Center Pane
- Renders `TodayView` (`crates/nodera-desktop/src/components/today_view.rs`) when `ActiveView::Today` is active.
- Centered container constrained to a maximum width of `520px` for optimal readability and focus.

```
--------------------------------------------------------------
Today                                  Monday, Sep 21, 2026   [Today]
--------------------------------------------------------------

                 <    September 2026    >

      MON    TUE    WED    THU    FRI    SAT    SUN

       31      1      2      3      4      5      6
        7      8      9     10     11     12     13
       14     15     16     17     18     19     20
       21•    22     23     24     25     26     27
       28     29     30      1      2      3      4

--------------------------------------------------------------
Selected date:
Monday, September 21, 2026                [Open Daily Note]
--------------------------------------------------------------
```

---

## 2. Component Hierarchy

```
App (src/app.rs)
  ├── Sidebar (src/components/sidebar.rs)
  │     └── Workspace button: "Today"
  ├── Main Pane Router (src/app.rs)
  │     └── TodayView (src/components/today_view.rs)
  │           ├── Header (Title, formatted date, "Today" jump button)
  │           ├── Month Navigator (< Prev, "Month Year", Next >)
  │           ├── Calendar Grid (role="grid")
  │           │     ├── Weekday Header Row (role="row", columns Mon..Sun)
  │           │     └── Weeks (6 rows × 7 days = 42 cells)
  │           │           └── DayCell (role="gridcell", button)
  │           │                 ├── Day Number
  │           │                 ├── Daily Note Indicator Dot (•)
  │           │                 └── Today Accent Marker
  │           └── Selected Date Card (Date readout + Open/Create button)
  └── Dialogs (src/components/dialogs.rs)
        └── GoToDateDialog ("Go to Daily Note for Date...")
```

---

## 3. Visual States & Design System Tokens

All visual elements use semantic CSS custom properties defined in `theme.rs` to support both Dark and Light themes cleanly:

| State | Visual Treatment | Semantic Tokens |
|---|---|---|
| **Normal Day (Current Month)** | Standard text, transparent border | `color: var(--text-primary)` |
| **Normal Day (Adjacent Month)** | Dimmed / muted text | `color: var(--text-muted)` |
| **Hover** | Subtle background highlight | `background-color: var(--bg-hover)` |
| **Keyboard Focus** | Visible 2px outline | `outline: 2px solid var(--focus); outline-offset: 1px` |
| **Today** | Border outline & subtle bottom accent bar | `border-color: var(--accent-hover); background-color: var(--bg-surface-elevated)` |
| **Selected Day** | Active background & primary accent border | `background-color: var(--bg-active); border-color: var(--accent)` |
| **Existing Daily Note** | Compact accent dot next to number (`21•`) | `color: var(--accent)` |
| **Future Date** | Distinguishable without rainbow colors | Text inherits month state, accessible label adds `"future date"` |

*Note: No rainbow colors, gradients, or glowing effects are used. Restrained Nodera tokens maintain visual harmony.*

---

## 4. Interaction States & User Flows

1. **Date Click**:
   - Clicking a date cell selects the date and opens its daily note in the Editor.
   - If `Daily/YYYY-MM-DD.md` exists: reads and displays it without overwrite.
   - If missing: atomically generates it using the Daily Note template, registers it in `entries` and search index, and opens it.
2. **Keyboard Navigation**:
   - `ArrowLeft` / `ArrowRight`: Navigate date by ±1 day.
   - `ArrowUp` / `ArrowDown`: Navigate date by ±7 days (weeks).
   - `Enter` / `Space`: Open or create the daily note for the currently selected date.
   - Month boundaries are automatically crossed when arrowing past month edges.
3. **Month Navigation**:
   - `<` (Previous Month) / `>` (Next Month): Adjusts visible month view without altering editor tabs or unsaved note buffers.
   - `[Today]` button: Jumps the view to the current month and selects today's date.
4. **Selected Date Action Card**:
   - Displays long date representation (e.g. `Monday, September 21, 2026`).
   - Button dynamic label: `[Open Daily Note]` (if exists) or `[Create Daily Note]` (if missing).

---

## 5. Accessibility (A11y)

- **Grid Semantics**: `role="grid"`, `role="row"`, `role="columnheader"`, `role="gridcell"`.
- **Keyboard Tab Navigation**: Selected cell has `tabindex="0"`; other cells have `tabindex="-1"` (roving tabindex pattern).
- **Aria Selected**: `aria-selected="true"` on the currently selected day cell.
- **Descriptive Labels**: Comprehensive `aria-label` generated on each cell:
  - Example: `"September 21, 2026, daily note exists, today, selected"`
  - Future dates append: `", future date"`
- **Focus Rings**: Clearly visible `:focus-visible` outline using `var(--focus)`.

---

## 6. Command Palette & Top Bar Integration

- **Top Bar**:
  - Existing "Daily" button in top bar is preserved. One click opens or creates today's daily note directly into the editor.
- **Command Palette Commands**:
  - `Open Today` → Switches to the Today workspace view (`ActiveView::Today`).
  - `Open Calendar` → Switches to the Today calendar workspace (`ActiveView::Today`).
  - `Open Today's Daily Note` → Directly opens/creates today's daily note in Editor (`PaletteAction::OpenDailyNote`).
  - `Go to Daily Note for Date...` → Opens the modal date picker dialog (`PaletteAction::OpenGoToDateDialog`).
- **Shortcuts**:
  - `Ctrl+Shift+D`: Opens today's daily note.
  - `Ctrl+G`: Cycles views (`Editor -> Today -> Tasks -> ReviewQueue -> Library -> Graph -> Editor`).
  - `Escape`: Closes `Go to Daily Note for Date...` modal without changes.

---

## 7. Responsive Behavior

- Fits pane widths from `360px` upwards.
- Day cells scale proportionally using `grid-template-columns: repeat(7, 1fr)`.
- Minimum tap target height is `44px` on each day cell.
- Vertical scroll enabled on `pane-center` when viewport height is constrained.
