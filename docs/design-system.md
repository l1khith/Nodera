# Nodera Design System

> Canonical UI design language for the Nodera desktop application.

This document defines the visual system for Nodera so that every UI surface uses the same colors, typography, spacing, components, states, and interaction patterns.

## 1. Design Principles

Nodera should feel:

- calm
- focused
- technical without being sterile
- dense but readable
- keyboard-first
- local-first
- professional
- consistent

Nodera takes interaction inspiration from knowledge-management applications such as Obsidian, but its visual identity is original.

### Core rules

1. Use semantic design tokens rather than component-specific colors.
2. Prefer neutral surfaces and restrained accents.
3. Use the Nodera blue/indigo family as the dominant brand language.
4. Use violet sparingly as a secondary accent.
5. Do not use gradients throughout the UI. Gradients are primarily a branding/logo treatment.
6. Do not communicate state using color alone.
7. Preserve strong focus and selection states.
8. Components should look related even when used in different application areas.

---

## 2. Brand Identity

### Primary brand

```text
Brand Primary      #5B6CFF
Brand Hover        #7182FF
Brand Pressed      #4A55E8
```

### Secondary brand

```text
Brand Secondary    #9A4BFF
Brand Secondary    #7E3FE0
```

### Brand palette

| Token | Hex | Usage |
|---|---|---|
| `brand-50` | `#F2F5FF` | subtle tint |
| `brand-100` | `#E3E9FF` | selected backgrounds |
| `brand-200` | `#C7D2FF` | soft borders/highlights |
| `brand-300` | `#9AA9FF` | secondary accent |
| `brand-400` | `#7182FF` | interactive accent |
| `brand-500` | `#5B6CFF` | primary brand |
| `brand-600` | `#4A55E8` | hover/active |
| `brand-700` | `#3C43C5` | pressed/strong |
| `violet-500` | `#9A4BFF` | secondary brand |
| `violet-600` | `#7E3FE0` | secondary emphasis |

---

## 3. Dark Theme

Dark mode is the primary visual reference for Nodera.

### Surfaces

```text
Background           #0D0F14
Surface              #141720
Surface Elevated     #1A1E29
Surface Hover        #202533
Surface Active       #282D40
```

### Borders

```text
Border               #292E3A
Border Strong        #383F4F
```

### Text

```text
Text Primary         #F1F3F8
Text Secondary       #A7ADBC
Text Muted           #747B8C
Text Disabled        #525866
```

### Interaction

```text
Primary              #5B6CFF
Primary Hover        #7182FF
Primary Pressed      #4A55E8
Secondary            #9A4BFF
Secondary Hover      #AC68FF
Focus                #7182FF
Selection            #3540A8
```

### Sidebar

```text
Sidebar Background   #11141B
Sidebar Hover        #191D27
Sidebar Active       #252A3A
```

---

## 4. Light Theme

### Surfaces

```text
Background           #F7F8FC
Surface              #FFFFFF
Surface Elevated     #FFFFFF
Surface Hover        #F0F2F8
Surface Active       #E8EBF5
```

### Borders

```text
Border               #DEE2EC
Border Strong        #C8CDD9
```

### Text

```text
Text Primary         #171A23
Text Secondary       #565D6D
Text Muted           #7A8190
Text Disabled        #A7ACB7
```

### Interaction

```text
Primary              #4F5FEF
Primary Hover        #4050D8
Primary Pressed      #3544B8
Secondary            #8744E8
Secondary Hover      #7837D4
Focus                #4F5FEF
Selection            #DDE2FF
```

### Sidebar

```text
Sidebar Background   #F1F3F8
Sidebar Hover        #E8EBF3
Sidebar Active       #DFE3F0
```

---

## 5. Semantic Status Colors

Do not use the brand accent for semantic status.

```text
Success              #35B875
Warning              #E3A93B
Error                #E45B63
Info                 #4FA3E3
```

Dark supporting backgrounds:

```text
Success Background   #163527
Warning Background   #392C16
Error Background     #391A1D
Info Background      #172E40
```

Status must be communicated with at least one non-color cue such as:

- icon
- text
- shape
- progress indicator

---

## 6. Editor Syntax Colors

Keep syntax highlighting restrained.

```text
Heading              #8090FF
Link                 #71B7FF
Wikilink             #A477FF
Tag                  #67C7B0
Code                 #D5A6FF
Quote                #8D95A6
Comment              #62697A
```

The editor must remain primarily text-focused.

Do not turn the editor into a rainbow syntax display.

---

## 7. Graph Colors

```text
Normal Node          #5B6CFF
Current Note         #9A4BFF
Hovered Node         #7182FF
Connected Node       #7E8CFF
Edge                 #444A5B
Highlighted Edge     #7182FF
Label                #D7DBE6
```

Graph meaning must not rely exclusively on color.

---

## 8. PDF Import Colors

Reuse application semantics.

```text
Selected             #5B6CFF
Converting           #7182FF
Completed            #35B875
Warning              #E3A93B
Failed               #E45B63
```

Do not create a separate PDF-specific visual language.

---

## 9. Typography

Use the platform's high-quality UI/system font by default unless a project-wide font is explicitly selected later.

### Type scale

```text
Display      28 px / 34 px
H1           24 px / 30 px
H2           20 px / 26 px
H3           17 px / 23 px
Body         14 px / 22 px
Small        12 px / 18 px
Caption      11 px / 16 px
Code         13 px / 20 px
```

### Weights

```text
Regular      400
Medium       500
Semibold     600
Bold         700
```

Use bold sparingly.

The interface should not look heavy.

### Editor typography

The editor should support a slightly larger reading size than utility UI:

```text
Editor Body       15–16 px
Editor Line Height 1.55–1.7
```

---

## 10. Spacing Scale

Use a 4 px base grid.

```text
4     xs
8     sm
12    md
16    lg
20    xl
24    2xl
32    3xl
40    4xl
48    5xl
64    6xl
```

Do not introduce arbitrary spacing values unless necessary.

Common usage:

```text
icon-to-label       8 px
small control gap   8 px
control groups      12 px
panel padding       16 px
dialog padding      24 px
section spacing     24–32 px
```

---

## 11. Corner Radius

Use modest radii.

```text
Radius XS            4 px
Radius SM            6 px
Radius MD            8 px
Radius LG            12 px
Radius XL            16 px
```

Guideline:

```text
Inputs/buttons       6–8 px
Panels               8–12 px
Dialogs              12–16 px
Cards                8–12 px
```

Avoid excessive pill-shaped UI.

Use full pills only for tags, compact status chips, or similar semantic elements.

---

## 12. Shadows

Nodera should rely more on surface contrast than heavy shadows.

### Dark theme

Use subtle elevation:

```text
small:
0 2px 8px rgba(0,0,0,0.20)

medium:
0 8px 24px rgba(0,0,0,0.28)

large:
0 16px 40px rgba(0,0,0,0.35)
```

### Light theme

Use softer shadows:

```text
small:
0 2px 8px rgba(16,24,40,0.08)

medium:
0 8px 24px rgba(16,24,40,0.12)

large:
0 16px 40px rgba(16,24,40,0.14)
```

Do not apply shadows to every component.

---

## 13. Icons

Use one consistent icon family throughout the application.

Rules:

- 16 px for dense navigation
- 18 px for standard controls
- 20–24 px for prominent actions
- same stroke weight throughout
- avoid mixing filled and outlined icon styles without reason

Icons must support the label rather than replace it when the action is ambiguous.

Examples:

```text
Notes          document icon
Tasks          checkbox/check icon
Library        book icon
Search         magnifier
Settings       gear
Import         download/file icon
Graph          nodes icon
Backlinks      link/arrow icon
```

Avoid emoji as permanent UI icons.

Emoji may appear in user-created content or optional decorative surfaces, but application chrome should use the chosen icon system.

---

## 14. Buttons

### Primary

Used for the main action of a surface.

```text
Background     Primary
Text           #FFFFFF
Radius         8 px
Height         34–38 px
Horizontal     12–16 px
```

### Secondary

```text
Background     Surface
Border         Border
Text           Text Primary
```

### Ghost

```text
Background     transparent
Hover          Surface Hover
Text           Text Secondary
```

### Destructive

Use the error semantic token.

Do not use red for ordinary secondary actions.

---

## 15. Inputs

Inputs should have:

```text
Background     Surface
Border         Border
Text           Text Primary
Placeholder    Text Muted
Radius         8 px
Focus          Focus ring
```

Focus must be visible without relying only on subtle color changes.

---

## 16. Panels

Panels provide spatial structure, not visual decoration.

Preferred:

```text
surface
+
subtle border
+
spacing
```

Avoid heavy borders around every nested element.

---

## 17. Navigation

### Sidebar

The sidebar should remain visually subordinate to the editor.

Hierarchy:

```text
Section label
  folder
    note
    note
```

Active item:

```text
subtle active background
+
primary accent/icon
+
primary text
```

Do not use a huge bright filled pill for the active note.

---

## 18. Tabs

Tabs should be compact.

States:

```text
Inactive
Hover
Active
Modified
Close-hover
```

An unsaved/modified state must use a non-color indicator such as a dot.

---

## 19. Dialogs

Dialog structure:

```text
Title
Description
Content
Actions
```

Primary action should be visually obvious.

Destructive dialogs must explicitly identify the destructive consequence.

Dialogs should trap focus appropriately.

Escape should close cancellable dialogs.

---

## 20. Toasts / Notifications

Use for short-lived feedback:

```text
Saved
Imported successfully
Index rebuilt
Copied
```

Do not use to display errors that require user decisions.

Errors that require action belong in an error surface/dialog.

---

## 21. Empty States

Structure:

```text
icon
title
short explanation
primary action
optional secondary action
```

Keep empty states compact.

Example:

```text
No notes yet

Create your first Markdown note or import an existing document.

[New Note] [Import]
```

---

## 22. Error States

Every user-facing error must answer:

```text
What happened?
What can I do?
```

Example:

```text
PDF could not be imported

The file appears to be encrypted with a password.

Choose another PDF or provide an accessible file.
```

Technical diagnostics should be available separately.

---

## 23. Editor UX

The editor is the primary application surface.

Requirements:

- minimal chrome
- strong text readability
- reliable cursor behavior
- clear selection
- visible active link
- predictable keyboard shortcuts
- no UI jitter while typing
- autosave status should be subtle

Preferred editor states:

```text
Editing
Saving
Saved
Error
Read-only
```

---

## 24. Live Preview / Reading Mode

Reading mode should prioritize content.

Remove unnecessary controls.

Use:

```text
comfortable reading width
large enough line height
clear heading hierarchy
consistent code blocks
visible links
```

Do not force a full-width document when reading.

---

## 25. Context Panel

The right panel is contextual rather than permanently dedicated to one feature.

It may contain:

```text
Outline
Backlinks
Outgoing Links
Properties
Tasks
```

The panel can be hidden.

---

## 26. Graph UI

Graph is a knowledge visualization surface, not decoration.

Support eventually:

```text
Global graph
Local graph
Node selection
Open note
Focus
Search/filter
Depth
```

Use subtle motion.

Disable/reduce animation when requested by the system/user.

---

## 27. Workspace Layout

Default layout:

```text
┌───────────────┬───────────────────────────┬─────────────────┐
│ Navigation    │ Editor / Reader           │ Context         │
│               │                           │                 │
│ 240–300 px    │ Flexible                  │ 260–320 px      │
└───────────────┴───────────────────────────┴─────────────────┘
```

These are defaults, not hard limits.

Users can resize/hide side panels.

The center editor is always the dominant surface.

---

## 28. Responsive Desktop Behavior

Below a narrower desktop window:

```text
1. hide context panel
2. collapse navigation
3. preserve editor
4. keep primary commands accessible through keyboard/command palette
```

Never let narrow windows make the application unusable.

---

## 29. Motion

Motion should communicate state, not entertain.

Preferred duration:

```text
Micro interaction     100–150 ms
Panel transition      150–220 ms
Modal transition      180–240 ms
```

Avoid bouncing and elastic effects.

Respect reduced-motion settings.

---

## 30. Color Usage Ratio

Approximate visual balance:

```text
70% neutral surfaces
20% text / secondary UI
8% primary blue/indigo
2% violet
```

This is a guideline, not a pixel-level constraint.

The goal is for the application to feel calm and blue-led, with violet used as an accent.

---

## 31. Logo Usage

Primary asset:

```text
assets/branding/nodera-logo.svg
```

Monochrome asset:

```text
assets/branding/nodera-logo-mono.svg
```

The SVG is the master asset.

Generate raster derivatives only for platform-specific requirements.

Do not repeatedly redraw or approximate the logo inside UI components.

### Logo rules

- keep clear space around the mark
- never distort proportions
- never rotate
- do not add random shadows
- do not place on backgrounds where contrast is insufficient
- use monochrome variant where gradients are inappropriate

---

## 32. Semantic Design Tokens

Implement the theme through semantic tokens.

```text
background
surface
surface-elevated
surface-hover
surface-active

border
border-strong

text-primary
text-secondary
text-muted
text-disabled

brand-primary
brand-primary-hover
brand-primary-pressed

brand-secondary
brand-secondary-hover

success
warning
error
info

focus
selection
```

Components must consume semantic tokens.

Do not scatter raw hexadecimal colors throughout the codebase.

---

## 33. Component State Model

Every reusable component should explicitly define:

```text
Default
Hover
Pressed
Focused
Disabled
Selected
Loading
Error
Success
```

Only implement states that actually apply to that component.

Do not create fake visual states.

---

## 34. UX Consistency Rules

The same action must look and behave the same everywhere.

Examples:

```text
Save
Cancel
Delete
Search
Open
Import
Create
```

If a button opens a dialog in one place, a semantically identical action should not silently perform a different interaction elsewhere.

---

## 35. Accessibility Rules

Minimum:

```text
visible focus
keyboard navigation
semantic labels
sufficient contrast
no color-only meaning
text scaling
reduced motion
accessible dialogs
```

Do not remove focus indicators merely to make the UI look cleaner.

---

## 36. Engineering Rules For UI

Dioxus components must not:

```text
query SQLite directly
write arbitrary files directly
contain business rules
duplicate application commands
```

Preferred:

```text
UI event
   ↓
application command
   ↓
service/domain
   ↓
adapter
```

The UI should primarily manage presentation state.

---

## 37. Definition of Visual Consistency

A new feature is visually complete when:

```text
same typography system
same spacing scale
same radii
same icon family
same semantic colors
same interaction states
same accessibility rules
```

are applied.

A feature must not invent a private design system.

---

## 38. Design Review Checklist

Before merging UI changes:

```text
[ ] Uses semantic tokens
[ ] Uses existing spacing scale
[ ] Uses existing type scale
[ ] Uses existing icon system
[ ] Has focus state
[ ] Has disabled state where applicable
[ ] Has error state where applicable
[ ] Has empty state where applicable
[ ] Works in light theme
[ ] Works in dark theme
[ ] Keyboard interaction verified
[ ] No raw color duplication
[ ] No unnecessary animation
[ ] No unrelated visual refactor
```

---

## 39. Product Identity Summary

Nodera should visually communicate:

```text
Markdown-native
Local-first
Technical
Calm
Connected
Fast
Focused
Native
```

The visual identity is:

```text
Blue / Indigo
        +
Violet accent
        +
Cool neutral surfaces
        +
High readability
        +
Minimal chrome
```

That combination should remain consistent across Notes, Tasks, Library, Graph, Search, Settings, and PDF Import.

---

## 40. First-Principles UI Consolidation & Visual Rules

### 40.1 The 12 First-Principles Rules

1. **ONE CANONICAL VISIBLE ENTRY POINT**: Every action has one primary visible location. The Command Palette (`Ctrl+P`) is a universal secondary invocation mechanism and does not count as a duplicate visual control. Context-specific actions appear only where directly relevant.
2. **NAVIGATION ≠ ACTION**: "Notes / Tasks / Library / Graph" are destinations. "Delete / Rename / Import / Rebuild" are actions.
3. **GLOBAL ACTIONS LIVE IN GLOBAL CHROME; CONTEXTUAL ACTIONS LIVE IN SURFACES**: Specialist tools (PDF annotations, citation insertion, note properties, graph controls) live in their contextual panels or Command Palette.
4. **ONE PRIMARY ACTION PER SURFACE**: Avoid multiple competing primary-weight buttons on a single card or toolbar.
5. **ICON-ONLY IS RESERVED FOR**: universally understood primitives, compact window controls, and actions with strong tooltip labels.
6. **GENERIC ICONS MUST NOT REPRESENT UNRELATED CONCEPTS**: Separate note properties (`IconProperties`) from tags (`IconTag`).
7. **COLOR COMMUNICATES MEANING**: Neutral by default. Cobalt accent for focus/selection. Green = success. Amber = warning. Red = danger. Blue = information/action.
8. **NO DECORATION WITHOUT FUNCTION**: No gradients, glass, glow, or decorative radial orbs just to fill space.
9. **NO DUPLICATE TOOLBARS**: A contextual operation must not simultaneously appear in top bar + sidebar + panel + card unless there is a deliberate workflow reason.
10. **THE CONTENT SURFACE GETS MOST OF THE VISUAL WEIGHT**: Minimize chrome thickness and borders.
11. **SECONDARY ACTIONS COLLAPSE INTO OVERFLOW, CONTEXT MENU, OR PALETTE**: Avoid multi-button row clutter.
12. **EVERY ICON MUST ANSWER**: icon → meaning → action → scope.

### 40.2 Visual No-Go List

```text
❌ UI gradients (linear or radial)
❌ Gradient text
❌ Neon borders
❌ Glowing buttons
❌ Liquid / frosted glass (backdrop-filter: blur)
❌ Decorative radial orbs
❌ Sparkle icons / decorative arrows
❌ Rainbow UI or graph color systems
❌ Pastel-accent overload
❌ Giant card drop-shadows
❌ Hover card lift (translateY)
❌ Emoji as application chrome
❌ Duplicate action clusters in table/list rows
❌ Oversized empty-state illustrations
```

### 40.3 Semantic Elevation Tokens

```css
--shadow-none: none;
--shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.2);
--shadow-md: 0 4px 12px rgba(0, 0, 0, 0.3);
--shadow-lg: 0 8px 24px rgba(0, 0, 0, 0.45);
```

Shadows are permitted only on floating surfaces:
- Editor/Sidebar/Library surface: `--shadow-none`
- Dropdown menus & popovers: `--shadow-sm`
- Modal dialogs & Command Palette: `--shadow-lg`

### 40.4 Contextual Inspector Architecture

The right-side rail is a single, contextual Inspector surface:
- **Editor / Note Mode**:
  - Outline (headings with jump-to-section)
  - Properties (frontmatter title, tags, custom key-values)
  - Links (Backlinks, Outgoing, Unlinked mentions)
  - Related Notes (Lexical BM25 + Tag Overlap)
  - Local Graph (Depth 1, 2, 3)
- **Graph Mode**:
  - Filters (Tags, Attachments, Existing, Orphans)
  - Groups (Color by Community)
  - Display (Size by Centrality, Arrows, Text fade, Node size)
  - Forces (Center, Repel, Link distance)

Never render note-inspector sections and graph-control sections together. The active view determines the inspector mode.
