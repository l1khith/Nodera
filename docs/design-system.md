---
description: Canonical UI design language and token specification for
  the Nodera desktop application.
title: Technical Precision Workspace
---

# Technical Precision Workspace Design System

> Canonical UI design language and token specification for the Nodera
> desktop application.

## 1. Purpose

This design system defines a high-precision, technical, native-feeling
desktop environment tailored for knowledge workers, software engineers,
and researchers.

The visual language draws from brutalist-adjacent minimalism and modern
technical desktop tools such as Obsidian, Linear, and VS Code. It
prioritizes:

-   Structural clarity
-   Operational speed
-   Keyboard-first ergonomics
-   Content density without visual noise
-   Clear surface hierarchy
-   Precise interaction states
-   Consistent typography and spacing

The visual tone is restrained, cerebral, and quiet.

### Non-negotiable principles

-   **Zero Decorative Fluff:** Broad gradients, blurred translucent
    backdrops, skeuomorphic gloss, and vibrant rainbow accent clusters
    are forbidden.
-   **Architectural Separation:** Layout structure is established
    through deep matte surfaces and crisp 1px hairline strokes rather
    than diffuse blur or exaggerated elevation.
-   **Content Primacy:** Screen real estate favors the editor canvas,
    markdown content, and relational graphs. Desktop chrome remains
    compact and unobtrusive.
-   **Deliberate Accents:** Primary interactions use authoritative
    cobalt blue. Relational knowledge vectors and active focus anchors
    use controlled violet.
-   **Technical Precision:** Dimensions, radii, typography, borders, and
    interaction states should be implemented from tokens rather than
    ad-hoc values.

------------------------------------------------------------------------

# 2. Theme Tokens --- `theme.rs`

## 2.1 Calibrated 70/20/8/2 Balance

The workspace follows a calibrated visual distribution:

| Ratio | Role |
|---|---|
| **70%** | Deep matte slate-black background and surface layers |
| **20%** | Structural lines and high-contrast technical typography |
| **8%** | Cobalt interactive focal points |
| **2%** | Purposeful relational violet |

This ratio is a visual discipline, not a literal pixel-area requirement.

## 2.2 Primary Foundations

| Token | Dark Value | Purpose |
|---|---|---|
| `--bg-app` | `#0B0F14` | Root application backdrop / recessed void canvas |
| `--bg-sidebar` | `#11161D` | Sidebars, navigation, top chrome |
| `--bg-surface` | `#141720` | Work surface, editor panes, cards, dialogs |
| `--bg-surface-elevated` | `#161D26` | Toolbars, tabs, code headers, flyouts |
| `--bg-hover` | `#202533` | Hover state |
| `--bg-active` | `#252A3A` | Active rows, tabs, selections |

## 2.3 Borders

| Token | Value | Usage |
|---|---|---|
| `--border` | `#28313C` | Default 1px hairline borders |
| `--border-strong` | `#383F4F` | Focused elements, modal perimeters |
| `--border-subtle` | `#1E232F` | Ultra-subtle dividers and table rules |

## 2.4 Text & Foreground

| Token | Dark Value | Light Value | Usage |
|---|---|---|---|
| `--text-primary` | `#DEE2ED` | `#171C23` | Primary prose, active titles |
| `--text-secondary` | `#C5C5D6` | `#475060` | Secondary labels and descriptions |
| `--text-muted` | `#8E90A0` | `#6B7687` | Metadata, shortcuts, captions |
| `--text-disabled` | `#444654` | `#98A2B3` | Disabled icons and controls |

## 2.5 Primary Cobalt Accent

| Token | Dark Value | Light Value | Usage |
|---|---|---|---|
| `--accent` | `#6680FF` | `#4A63E8` | Primary interactive cobalt |
| `--accent-hover` | `#7182FF` | `#3B53D8` | Hover state |
| `--accent-pressed` | `#4A55E8` | `#2D41B8` | Pressed state |
| `--accent-focus` | `rgba(102, 128, 255, 0.25)` | `rgba(74, 99, 232, 0.2)` | 2px keyboard focus halo |

## 2.6 Relational Violet

| Token | Dark Value | Light Value | Usage |
|---|---|---|---|
| `--accent-secondary` | `#9A4BFF` | `#7E3FE0` | Wikilinks, transclusions, active graph node |
| `--accent-secondary-hover` | `#AC68FF` | `#9253F0` | Relational hover state |

## 2.7 Knowledge Graph Roles

| Role | Value |
|---|---|
| Graph nodes | `#6680FF` |
| Graph edges | `#444A5B` |
| Illuminated graph edges | `#7182FF` |
| Dot grid | `rgba(255, 255, 255, 0.08)` |

## 2.8 Semantic Tones

| Semantic | Foreground | Container | Usage |
|---|---|---|---|
| Success | `#35B875` | `#163527` | Completed indexing, positive feedback |
| Warning | `#E3A93B` | `#392C16` | Unsaved buffers, unlinked mentions |
| Error | `#E45B63` | `#391A1D` | Parse errors, broken links, destructive actions |
| Info | `#4FA3E3` | `#172E40` | Citation updates, sync advisories |

------------------------------------------------------------------------

# 3. Canonical Light Theme

The light theme mirrors the same structural hierarchy while preserving
contrast and semantic relationships.

| Token | Dark | Light |
|---|---|---|
| `--bg-app` | `#0B0F14` | `#F4F6F9` |
| `--bg-sidebar` | `#11161D` | `#ECEFF3` |
| `--bg-surface` | `#141720` | `#FFFFFF` |
| `--bg-surface-elevated` | `#161D26` | `#F8FAFC` |
| `--bg-hover` | `#202533` | `#EEF2F6` |
| `--bg-active` | `#252A3A` | `#E4E9F2` |
| `--border` | `#28313C` | `#D1D7E0` |
| `--border-strong` | `#383F4F` | `#A8B2C0` |
| `--border-subtle` | `#1E232F` | `#E6EAF0` |
| `--text-primary` | `#DEE2ED` | `#171C23` |
| `--text-secondary` | `#C5C5D6` | `#475060` |
| `--text-muted` | `#8E90A0` | `#6B7687` |
| `--text-disabled` | `#444654` | `#98A2B3` |
| `--accent` | `#6680FF` | `#4A63E8` |
| `--accent-hover` | `#7182FF` | `#3B53D8` |
| `--accent-pressed` | `#4A55E8` | `#2D41B8` |
| `--accent-secondary` | `#9A4BFF` | `#7E3FE0` |
| `--accent-secondary-hover` | `#AC68FF` | `#9253F0` |
| `--accent-focus` | `rgba(102, 128, 255, 0.25)` | `rgba(74, 99, 232, 0.2)` |

------------------------------------------------------------------------

# 4. Typography --- `editor.rs` / `state.rs`

## 4.1 Font Families

### UI

Primary UI font:

``` text
Geist
```

Fallback stack:

``` text
-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif
```

### Code and editor metadata

Primary monospace:

``` text
JetBrains Mono
```

Fallback stack:

``` text
"Cascadia Code", "Fira Code", Consolas, monospace
```

## 4.2 Typography Scale

| Style | Font | Size | Weight | Line Height | Tracking |
|---|---|---|---|---|---|
| Display | Geist | 28px | 700 | 34px | -0.01em |
| H1 | Geist | 24px | 700 | 30px | -0.01em |
| H2 | Geist | 20px | 600 | 26px | -0.005em |
| H3 | Geist | 17px | 600 | 23px | 0 |
| Body | Geist | 14px | 400 | 22px | 0 |
| Body Small | Geist | 12px | 400 | 18px | 0 |
| Editor Body | Geist | 15px | 400 | 26px | 0 |
| Caption | Geist | 11px | 500 | 16px | 0.02em |
| Code Label | JetBrains Mono | 13px | 400 | 20px | 0 |
| Property Label | JetBrains Mono | 11px | 500 | 16px | 0.01em |

## 4.3 Editor Reading Defaults

The editor default is:

-   **Font size:** `15px`
-   **Line height:** `26px`
-   **Ratio:** approximately `1.73`
-   **Font family:** `Geist`
-   Applies consistently to single-pane and split-view editors.

Long-form editor buffers use this generous line height to support
sustained reading while retaining desktop density.

------------------------------------------------------------------------

# 5. Layout & Spacing

The workspace follows a strict 4px incremental baseline grid.

## 5.1 Spacing Tokens

| Token | Value |
|---|---|
| `gutter` | `1px` |
| `gutter-panel` | `0.5rem` / 8px |
| `margin` | `1rem` / 16px |
| `margin-editor` | `1.75rem` / 28px |
| `space-xs` | `0.25rem` / 4px |
| `space-sm` | `0.5rem` / 8px |
| `space-md` | `0.75rem` / 12px |
| `space-lg` | `1rem` / 16px |
| `space-xl` | `1.5rem` / 24px |

## 5.2 Desktop Chrome

| Region | Height |
|---|---|
| Top Header Utility Bar | 40px |
| Document Tab Bar | 38px |
| Status Bar | 24px |

### Top Header Utility Bar

Fixed at `40px`, spanning the full window width.

Contains:

-   Application controls
-   Vault status indicator
-   Global command triggers

### Document Tab Bar

Fixed at `38px`.

Tabs use approximately `4px` element gaps.

### Status Bar

Fixed to the bottom edge at `24px`.

Uses the `11px` caption typography.

------------------------------------------------------------------------

# 6. Three-Pane Workspace Model

## Left Navigation Pane

Default width:

``` text
240px–300px
```

Minimum:

``` text
180px
```

Contains:

-   Hierarchical file trees
-   Vault bookmarks
-   Navigation utilities

## Center Canvas

Uses:

``` text
flex: 1
```

Markdown prose is constrained to an optimal reading width of
approximately:

``` text
780px–840px
```

Graph canvases may span full bleed.

## Right Context Panel

Default width:

``` text
260px–320px
```

Contains:

-   Backlinks
-   Metadata inspector
-   Outgoing references
-   Contextual knowledge information

## Pane Dividers

Visual divider:

``` text
1px
```

Interactive hit area:

``` text
5px
```

This allows bidirectional split resizing without visually thickening the
divider.

------------------------------------------------------------------------

# 7. Responsive Breakpoints

## Compact Desktop --- `< 1100px`

The right context panel collapses into:

-   An off-canvas drawer, or
-   A toggleable context tab

## Narrow Desktop --- `< 800px`

The left sidebar:

-   Folds into an icon rail, or
-   Collapses completely

Document navigation should then remain accessible through
keyboard-driven quick-open controls.

------------------------------------------------------------------------

# 8. Elevation & Depth

Visual hierarchy uses tiered matte surfaces and crisp hairline borders
rather than blurred translucency or ambient atmospheric glow.

## 8.1 Surface Hierarchy

| Level | Surface | Value | Typical Usage |
|---|---|---|---|
| Level 0 | Recessed Void | `#0B0F14` | Canvas backdrop, graph background, editor empty space |
| Level 1 | Docked Structural | `#11161D` | Tree panels, sidebar chrome, window headers |
| Level 2 | Active Work Surface | `#141720` | Editor panes, inputs, document cards |
| Level 3 | Elevated Technical | `#161D26` | Tab bars, code headers, context dropdowns |
| Level 4 | Floating Overlay | `#141720` / `#161D26` | Command palettes, menus, setting panels |

## 8.2 Shadows

Shadows are restricted to floating overlays.

### Overlay

``` css
box-shadow: 0 16px 40px rgba(0, 0, 0, 0.45);
```

### Floating Card

``` css
box-shadow: 0 4px 16px rgba(0, 0, 0, 0.28);
```

### Library Card Hover

``` css
box-shadow: 0 8px 28px rgba(0, 0, 0, 0.45);
```

------------------------------------------------------------------------

# 9. Focus & Interaction States

Keyboard-focused elements use a crisp two-layer focus treatment:

``` text
2px perimeter ring
rgba(102, 128, 255, 0.35)
```

with:

``` text
1px solid #6680FF
```

The focus treatment must not cause layout reflow.

------------------------------------------------------------------------

# 10. Shapes & Radius System

| Token | Value | Usage |
|---|---|---|
| `sm` | `0.125rem` / 2px | Small technical elements |
| `DEFAULT` | `0.25rem` / 4px | Standard compact controls |
| `md` | `0.375rem` / 6px | Buttons and inputs |
| `lg` | `0.5rem` / 8px | Larger controls and shells |
| `xl` | `0.75rem` / 12px | Cards and major dialogs |
| `full` | `9999px` | Pills and counters |

### Shape rules

-   Tree selections, status chips, icon utility buttons, tag pills and
    inline code tags: **4px**
-   Primary buttons, input fields, callouts, tabs and command shells:
    **6–8px**
-   Library cards and major setup dialogs: **12px**
-   Graph zoom indicators and notification counters: **9999px**

------------------------------------------------------------------------

# 11. Component Specifications

## 11.1 Primary Button

``` text
Height:        34px
Padding:       6px 14px
Radius:        6px
Background:    #6680FF
Border:        1px solid #6680FF
Text:          #FFFFFF
Weight:        600
```

States:

| State | Background |
|---|---|
| Default | `#6680FF` |
| Hover | `#7182FF` |
| Pressed | `#4A55E8` |

## 11.2 Secondary / Action Button

``` text
Background: #141720
Text:       #ECF0F4
Border:     1px solid #28313C
```

Hover:

``` text
Background: #202533
Border:     #6680FF
```

## 11.3 Icon Utility Button

``` text
Size:       28px × 28px
Radius:     4px
Background: transparent
Icon:       #A7ADBC
```

Hover:

``` text
Background: #202533
Icon:       #ECF0F4
```

## 11.4 Destructive Button

Default:

``` text
Background: transparent
Text:       #E45B63
```

Hover:

``` text
Background: #391A1D
Text:       #F06A72
```

------------------------------------------------------------------------

# 12. Inputs & Property Rows

## Search / Text Input

``` text
Background: #161D26
Border:     1px solid #28313C
Radius:     6px
Padding:    6px 10px
Typography: body-sm
```

Inside property drawers, the background may use:

``` text
#0B0F14
```

Focused state:

``` text
Border: #6680FF
Halo:   2px rgba(102, 128, 255, 0.25)
```

## Frontmatter Property Row

``` text
Key column: 110px fixed
Font:       JetBrains Mono
Label:      #747B8C
Input:      26px field height
```

An inline delete action should remain available without changing the
row's overall geometry.

------------------------------------------------------------------------

# 13. Knowledge Tags & Metadata

## Knowledge Tag

``` text
Background: #191D27
Border:     1px solid #28313C
Text:       #7182FF
Radius:     4px
Font:       JetBrains Mono
Size:       11px
```

## Status Indicator

Use a compact `4px` vertical pill paired with an `11px` medium label.

Semantic containers:

``` text
Success: #163527
Warning: #392C16
Error:   #391A1D
```

------------------------------------------------------------------------

# 14. Lists & Tree Views

## Tree Node Item

``` text
Height:      28px
Horizontal:  6px
Radius:      4px
```

Inactive:

``` text
Text: #A7ADBC
```

Active:

``` text
Background: #252A3A
Text:       #ECF0F4
Marker:     #6680FF
```

## List Dividers

``` text
1px solid #1E232F
```

------------------------------------------------------------------------

# 15. Multi-Tab Bar

## Container

``` text
Height: 38px
```

## Tab

``` text
Maximum width: 180px
Padding:      6px 12px
Top radius:   6px
```

## Active Tab

``` text
Background: #141720
Border:     1px solid #28313C
Bottom:     transparent
Text:       #ECF0F4
```

The transparent bottom border visually merges the active tab with the
content pane.

## Dirty Buffer Indicator

``` text
Size: 6px circular dot
Color: #6680FF
Position: left of close glyph
```

------------------------------------------------------------------------

# 16. Cards & Transclusions

## Library Document Card

``` text
Background: #141720
Border:     1px solid #28313C
Radius:     12px
Shadow:     0 4px 16px rgba(0, 0, 0, 0.28)
```

Hover:

``` text
Transform: translateY(-2px)
Border:    #6680FF
Shadow:    0 8px 28px rgba(0, 0, 0, 0.45)
```

## Transcluded Note Embed

``` text
Background: #141720
Border:     1px solid #28313C
Left border: 4px solid #6680FF
Radius:     8px
```

Header:

``` text
Background: #161D26
Padding:    8px 14px
```

## Markdown Callout

``` text
Background: #141720
Border:     1px solid #28313C
Radius:     6px
Left border: 4px
```

Semantic left-border colors:

| Callout | Color |
|---|---|
| Note | `#4FA3E3` |
| Tip | `#35B875` |
| Warning | `#E3A93B` |
| Danger | `#E45B63` |

------------------------------------------------------------------------

# 17. Command Palette

## Overlay

``` text
Background: rgba(0, 0, 0, 0.60)
Backdrop blur: none
```

The overlay is a matte scrim. Do not use backdrop blur.

## Dialog

``` text
Width:       540px
Background:  #141720
Border:      1px solid #383F4F
Radius:      8px
Shadow:      0 16px 40px rgba(0, 0, 0, 0.45)
```

## Search Header

``` text
Height: 44px
Border: none
Autofocus: yes
```

Contains:

-   Leading search glyph
-   Search input
-   Monospace `Esc` shortcut hint

## Result Row

``` text
Height: 36px
```

Active row:

``` text
Background: #202533
```

Keyboard action badge uses:

``` text
JetBrains Mono
```

------------------------------------------------------------------------

# 18. Knowledge Graph

The graph is treated as a spatial information surface rather than a
decorative visualization.

## Nodes

``` text
Default: #6680FF
```

## Edges

``` text
Default:    #444A5B
Illuminated: #7182FF
```

## Active Relational State

Use:

``` text
#9A4BFF
```

for:

-   Wikilinks
-   Transclusions
-   Active relational nodes
-   Relationship-focused interactions

## Dot Grid

``` text
rgba(255, 255, 255, 0.08)
```

The grid should remain subordinate to nodes, edges and document content.

------------------------------------------------------------------------

# 19. Editor Rules

The editor is the primary content surface and therefore receives the
strongest protection from unnecessary chrome.

### Required defaults

-   `15px` editor body font
-   `26px` line height
-   Geist as primary reading font
-   JetBrains Mono for code/property metadata
-   780–840px optimal prose line length
-   Minimal visual borders inside the document
-   No decorative gradients
-   No excessive shadows
-   No translucent editor backgrounds

### Split View

Single-pane and split-view editors must use the same editor typography
tokens and vertical rhythm.

------------------------------------------------------------------------

# 20. Component Density Rules

Desktop chrome must remain compact.

Use:

-   28px icon utility controls
-   34px primary controls
-   38px tab bar
-   40px header
-   24px status bar
-   28px tree rows
-   36px command result rows

Avoid increasing component height merely to create visual emphasis.
Emphasis should come from typography, contrast, state surfaces and
accent treatment.

------------------------------------------------------------------------

# 21. Accessibility & Interaction Requirements

-   Keyboard focus must always remain visually identifiable.
-   Focus rings must not cause layout reflow.
-   Disabled elements must use `--text-disabled`.
-   Semantic states must not rely solely on color; pair them with text,
    icons or structural indicators.
-   Text hierarchy must remain legible against its assigned surface.
-   Hover should never be the only way to discover an available action.
-   Destructive actions must use the error semantic system consistently.
-   Interactive hit areas may be larger than their visible borders when
    precision resizing is required.

------------------------------------------------------------------------

# 22. Implementation Checklist

## `theme.rs`

Implement the canonical tokens for:

-   Background surfaces
-   Hover and active states
-   Borders
-   Text hierarchy
-   Cobalt accent
-   Violet relational accent
-   Semantic states
-   Focus ring
-   Light-theme equivalents

## `editor.rs`

Implement:

-   Geist editor typography
-   15px editor default size
-   26px line height
-   Single-pane consistency
-   Split-view consistency
-   Markdown content density
-   Code / metadata typography using JetBrains Mono

## `state.rs`

Ensure editor and UI state defaults reference the design-system values
rather than introducing independent visual constants.

## Component layer

Implement the component dimensions and states in this document before
introducing custom one-off values.

------------------------------------------------------------------------

# 23. Source Token Reference

The original canonical token set includes the following foundational
Material-style semantic values and should remain available where
framework-level semantic mapping is required:

``` yaml
Technical Precision Workspace:
  colors:
    surface: '#0f141b'
    surface-dim: '#0f141b'
    surface-bright: '#353941'
    surface-container-lowest: '#090e15'
    surface-container-low: '#171c23'
    surface-container: '#1b2027'
    surface-container-high: '#252a32'
    surface-container-highest: '#30353d'
    on-surface: '#dee2ed'
    on-surface-variant: '#c5c5d6'
    inverse-surface: '#dee2ed'
    inverse-on-surface: '#2c3138'
    outline: '#8e90a0'
    outline-variant: '#444654'
    surface-tint: '#b9c3ff'
    primary: '#b9c3ff'
    on-primary: '#00218c'
    primary-container: '#7088ff'
    on-primary-container: '#001c7b'
    inverse-primary: '#3551cf'
    secondary: '#d8baff'
    on-secondary: '#440087'
    secondary-container: '#7107d6'
    on-secondary-container: '#d7b9ff'
    tertiary: '#9ccaff'
    on-tertiary: '#003257'
    tertiary-container: '#4c95db'
    on-tertiary-container: '#002b4c'
    error: '#ffb4ab'
    on-error: '#690005'
    error-container: '#93000a'
    on-error-container: '#ffdad6'
    background: '#0f141b'
    on-background: '#dee2ed'
    surface-variant: '#30353d'
```

------------------------------------------------------------------------

# 24. Design-System Rules of Thumb

1.  **Prefer structure over decoration.**
2.  **Prefer hairlines over heavy borders.**
3.  **Prefer matte surfaces over blur.**
4.  **Prefer cobalt for direct interaction.**
5.  **Prefer violet for relational knowledge semantics.**
6.  **Keep editor chrome subordinate to content.**
7.  **Use shadows only where an element actually floats.**
8.  **Use monospace typography when alignment communicates structure.**
9.  **Keep desktop controls compact.**
10. **Use tokens before inventing a new value.**
11. **Maintain the 70/20/8/2 visual balance.**
12. **Do not introduce gradients, glassmorphism, decorative glow, or
    unnecessary animation.**

------------------------------------------------------------------------

# 25. Canonical Status

**Status:** Canonical

**Applies to:** Nodera desktop application

**Primary implementation files:** `theme.rs`, `editor.rs`, `state.rs`

**Documentation:** `docs/design-system.md`

Any component that conflicts with this specification should be treated
as a design-system deviation and reviewed before being introduced into
the application.
