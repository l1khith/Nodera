use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct IconProps {
    #[props(default = 16)]
    pub size: u32,
    #[props(default = "")]
    pub class: &'static str,
}

/// Notes / File-text icon (replaces 📝)
#[component]
pub fn IconNotes(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" }
            polyline { points: "14 2 14 8 20 8" }
            line { x1: "16", y1: "13", x2: "8", y2: "13" }
            line { x1: "16", y1: "17", x2: "8", y2: "17" }
            polyline { points: "10 9 9 9 8 9" }
        }
    }
}

/// Check-square / Tasks icon (replaces ✅)
#[component]
pub fn IconTasks(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "9 11 12 14 22 4" }
            path { d: "M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" }
        }
    }
}

/// Library / Book-open icon (replaces 📚 and 📖)
#[component]
pub fn IconLibrary(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" }
            path { d: "M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" }
        }
    }
}

/// Book icon for reading / book covers
#[component]
pub fn IconBook(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M4 19.5A2.5 2.5 0 0 1 6.5 17H20" }
            path { d: "M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" }
        }
    }
}

/// Folder icon (replaces 📁)
#[component]
pub fn IconFolder(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" }
        }
    }
}

/// Open folder icon (replaces 📂)
#[component]
pub fn IconFolderOpen(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M6 14l1.45-6.23A2 2 0 0 1 9.4 6H20a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h5l2 3h9" }
        }
    }
}

/// Folder-plus / Create vault icon (replaces ✨)
#[component]
pub fn IconFolderPlus(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" }
            line { x1: "12", y1: "11", x2: "12", y2: "17" }
            line { x1: "9", y1: "14", x2: "15", y2: "14" }
        }
    }
}

/// Document / File icon (replaces 📄)
#[component]
pub fn IconFile(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z" }
            polyline { points: "13 2 13 9 20 9" }
        }
    }
}

/// Plus / New Note icon (replaces ➕)
#[component]
pub fn IconPlus(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "12", y1: "5", x2: "12", y2: "19" }
            line { x1: "5", y1: "12", x2: "19", y2: "12" }
        }
    }
}

/// Search / Magnifier icon (replaces 🔍)
#[component]
pub fn IconSearch(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "11", cy: "11", r: "8" }
            line { x1: "21", y1: "21", x2: "16.65", y2: "16.65" }
        }
    }
}

/// Settings gear icon (replaces ⚙️)
#[component]
pub fn IconSettings(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "3" }
            path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" }
        }
    }
}

/// Sun / Light mode icon (replaces ☀️)
#[component]
pub fn IconSun(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "5" }
            line { x1: "12", y1: "1", x2: "12", y2: "3" }
            line { x1: "12", y1: "21", x2: "12", y2: "23" }
            line { x1: "4.22", y1: "4.22", x2: "5.64", y2: "5.64" }
            line { x1: "18.36", y1: "18.36", x2: "19.78", y2: "19.78" }
            line { x1: "1", y1: "12", x2: "3", y2: "12" }
            line { x1: "21", y1: "12", x2: "23", y2: "12" }
            line { x1: "4.22", y1: "19.78", x2: "5.64", y2: "18.36" }
            line { x1: "18.36", y1: "5.64", x2: "19.78", y2: "4.22" }
        }
    }
}

/// Moon / Dark mode icon (replaces 🌙)
#[component]
pub fn IconMoon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" }
        }
    }
}

/// Refresh / Rebuild index icon (replaces 🔄)
#[component]
pub fn IconRefresh(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "23 4 23 10 17 10" }
            polyline { points: "1 20 1 14 7 14" }
            path { d: "M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" }
        }
    }
}

/// Trash / Delete icon (replaces 🗑️)
#[component]
pub fn IconTrash(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "3 6 5 6 21 6" }
            path { d: "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" }
        }
    }
}

/// Edit / Pen / Rename icon (replaces ✏️)
#[component]
pub fn IconEdit(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" }
            path { d: "M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" }
        }
    }
}

/// Save floppy icon (replaces 💾)
#[component]
pub fn IconSave(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" }
            polyline { points: "17 21 17 13 7 13 7 21" }
            polyline { points: "7 3 7 8 15 8" }
        }
    }
}

/// Close / X icon (replaces ✕)
#[component]
pub fn IconClose(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "18", y1: "6", x2: "6", y2: "18" }
            line { x1: "6", y1: "6", x2: "18", y2: "18" }
        }
    }
}

/// Menu / Sidebar toggle icon (replaces ☰)
#[component]
pub fn IconMenu(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "3", y1: "12", x2: "21", y2: "12" }
            line { x1: "3", y1: "6", x2: "21", y2: "6" }
            line { x1: "3", y1: "18", x2: "21", y2: "18" }
        }
    }
}

/// Info icon (replaces ℹ️)
#[component]
pub fn IconInfo(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            line { x1: "12", y1: "16", x2: "12", y2: "12" }
            line { x1: "12", y1: "8", x2: "12.01", y2: "8" }
        }
    }
}

/// Alert triangle / Warning icon (replaces ⚠️)
#[component]
pub fn IconWarning(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" }
            line { x1: "12", y1: "9", x2: "12", y2: "13" }
            line { x1: "12", y1: "17", x2: "12.01", y2: "17" }
        }
    }
}

/// Import / Download icon (replaces 📥)
#[component]
pub fn IconImport(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
            polyline { points: "7 10 12 15 17 10" }
            line { x1: "12", y1: "15", x2: "12", y2: "3" }
        }
    }
}

/// External link / Navigation arrow icon (replaces ↗️)
#[component]
pub fn IconExternalLink(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" }
            polyline { points: "15 3 21 3 21 9" }
            line { x1: "10", y1: "14", x2: "21", y2: "3" }
        }
    }
}

/// Checkmark icon (replaces ✓)
#[component]
pub fn IconCheck(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "20 6 9 17 4 12" }
        }
    }
}

/// Keyboard shortcuts icon (replaces ⌨️)
#[component]
pub fn IconKeyboard(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "2", y: "4", width: "20", height: "16", rx: "2", ry: "2" }
            line { x1: "6", y1: "8", x2: "6", y2: "8" }
            line { x1: "10", y1: "8", x2: "10", y2: "8" }
            line { x1: "14", y1: "8", x2: "14", y2: "8" }
            line { x1: "18", y1: "8", x2: "18", y2: "8" }
            line { x1: "6", y1: "12", x2: "6", y2: "12" }
            line { x1: "10", y1: "12", x2: "10", y2: "12" }
            line { x1: "14", y1: "12", x2: "14", y2: "12" }
            line { x1: "18", y1: "12", x2: "18", y2: "12" }
            line { x1: "7", y1: "16", x2: "17", y2: "16" }
        }
    }
}

/// Table of contents / List icon (replaces 📑)
#[component]
pub fn IconList(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "8", y1: "6", x2: "21", y2: "6" }
            line { x1: "8", y1: "12", x2: "21", y2: "12" }
            line { x1: "8", y1: "18", x2: "21", y2: "18" }
            line { x1: "3", y1: "6", x2: "3.01", y2: "6" }
            line { x1: "3", y1: "12", x2: "3.01", y2: "12" }
            line { x1: "3", y1: "18", x2: "3.01", y2: "18" }
        }
    }
}

/// Link / Wikilink / Chain icon (replaces 🔗)
#[component]
pub fn IconLink(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" }
            path { d: "M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" }
        }
    }
}

/// Vault / Database / Archive cabinet icon (replaces 🗄️)
#[component]
pub fn IconVault(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            ellipse { cx: "12", cy: "5", rx: "9", ry: "3" }
            path { d: "M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" }
            path { d: "M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" }
        }
    }
}

/// Palette / Theme styling icon (replaces 🎨)
#[component]
pub fn IconPalette(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "13.5", cy: "6.5", r: ".5", fill: "currentColor" }
            circle { cx: "17.5", cy: "10.5", r: ".5", fill: "currentColor" }
            circle { cx: "8.5", cy: "7.5", r: ".5", fill: "currentColor" }
            circle { cx: "6.5", cy: "12.5", r: ".5", fill: "currentColor" }
            path { d: "M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.563-2.512 5.563-5.563C22 6.5 17.5 2 12 2z" }
        }
    }
}

/// Chevron right icon (replaces ▶)
#[component]
pub fn IconChevronRight(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "9 18 15 12 9 6" }
        }
    }
}

/// Chevron down icon (replaces ▼)
#[component]
pub fn IconChevronDown(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "6 9 12 15 18 9" }
        }
    }
}

/// Enter / Return key icon (replaces ↵)
#[component]
pub fn IconCornerDownLeft(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "9 10 4 15 9 20" }
            path { d: "M20 4v7a4 4 0 0 1-4 4H4" }
        }
    }
}

/// Nodera official brand folded-book N logo (full color from docs/icons/nodera-logo.svg)
#[component]
pub fn IconNoderaLogo(props: IconProps) -> Element {
    rsx! {
        img {
            class: "icon-logo {props.class}",
            src: "{crate::logo_data::NODERA_LOGO_DATA_URI}",
            width: "{props.size}",
            height: "{props.size}",
            alt: "Nodera",
            style: "display: inline-block; vertical-align: middle; object-fit: contain; flex-shrink: 0;",
        }
    }
}

/// Monochrome variant of Nodera folded-book N logo
#[component]
pub fn IconNoderaLogoMono(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 512 512",
            path { d: "M92 128Q90 102 117 92L211 132V344Q211 370 187 382L105 425Q86 435 86 410Z", fill: "currentColor" }
            path { d: "M211 132L315 224V379Q315 407 291 420L214 461V342L260 319L214 279Z", fill: "currentColor" }
            path { d: "M315 224L390 126Q406 105 429 115L442 121Q463 132 463 155V403Q463 429 440 441L343 489Q315 501 315 470Z", fill: "currentColor" }
            path { d: "M214 132L326 62Q349 49 370 61L390 73L315 168Z", fill: "currentColor" }
            path { d: "M265 139L355 82Q378 68 397 80L414 91L342 185Z", fill: "currentColor" }
        }
    }
}

/// Graph / Network nodes and links icon (for 2D Knowledge Graph view)
#[component]
pub fn IconGraph(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "18", cy: "5", r: "3" }
            circle { cx: "6", cy: "12", r: "3" }
            circle { cx: "18", cy: "19", r: "3" }
            line { x1: "8.59", y1: "13.51", x2: "15.42", y2: "17.49" }
            line { x1: "15.41", y1: "6.51", x2: "8.59", y2: "10.49" }
        }
    }
}

/// Maximize / Expand icon
#[component]
pub fn IconMaximize(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "15 3 21 3 21 9" }
            polyline { points: "9 21 3 21 3 15" }
            line { x1: "21", y1: "3", x2: "14", y2: "10" }
            line { x1: "3", y1: "21", x2: "10", y2: "14" }
        }
    }
}

/// Minimize / Contract icon
#[component]
pub fn IconMinimize(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "4 14 10 14 10 20" }
            polyline { points: "20 10 14 10 14 4" }
            line { x1: "14", y1: "10", x2: "21", y2: "3" }
            line { x1: "3", y1: "21", x2: "10", y2: "14" }
        }
    }
}

/// Filter icon
#[component]
pub fn IconFilter(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polygon { points: "22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" }
        }
    }
}

/// Minus icon
#[component]
pub fn IconMinus(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "5", y1: "12", x2: "19", y2: "12" }
        }
    }
}

/// Crosshair / Center icon
#[component]
pub fn IconCrosshair(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "8" }
            line { x1: "12", y1: "2", x2: "12", y2: "6" }
            line { x1: "12", y1: "18", x2: "12", y2: "22" }
            line { x1: "2", y1: "12", x2: "6", y2: "12" }
            line { x1: "18", y1: "12", x2: "22", y2: "12" }
        }
    }
}

/// Sliders / Controls icon
#[component]
pub fn IconSliders(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "4", y1: "21", x2: "4", y2: "14" }
            line { x1: "4", y1: "10", x2: "4", y2: "3" }
            line { x1: "12", y1: "21", x2: "12", y2: "12" }
            line { x1: "12", y1: "8", x2: "12", y2: "3" }
            line { x1: "20", y1: "21", x2: "20", y2: "16" }
            line { x1: "20", y1: "12", x2: "20", y2: "3" }
            line { x1: "1", y1: "14", x2: "7", y2: "14" }
            line { x1: "9", y1: "8", x2: "15", y2: "8" }
            line { x1: "17", y1: "16", x2: "23", y2: "16" }
        }
    }
}

/// Chevron left icon (replaces ◀)
#[component]
pub fn IconChevronLeft(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "15 18 9 12 15 6" }
        }
    }
}

/// Pin icon for pinned tabs and notes
#[component]
pub fn IconPin(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M16 2v4l3 3v2h-6v7l-1 1-1-1v-7H5V9l3-3V2z" }
        }
    }
}

/// Calendar icon for daily notes
#[component]
pub fn IconCalendar(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "3", y: "4", width: "18", height: "18", rx: "2", ry: "2" }
            line { x1: "16", y1: "2", x2: "16", y2: "6" }
            line { x1: "8", y1: "2", x2: "8", y2: "6" }
            line { x1: "3", y1: "10", x2: "21", y2: "10" }
        }
    }
}

/// Template / Layout icon for note templates engine
#[component]
pub fn IconTemplate(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "3", y: "3", width: "18", height: "18", rx: "2" }
            line { x1: "3", y1: "9", x2: "21", y2: "9" }
            line { x1: "9", y1: "21", x2: "9", y2: "9" }
        }
    }
}

/// Clock / History icon for recent notes
#[component]
pub fn IconClock(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            polyline { points: "12 6 12 12 16 14" }
        }
    }
}

/// Bookmark / Saved icon for bookmarked notes
#[component]
pub fn IconBookmark(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" }
        }
    }
}

/// Split view / Columns icon
#[component]
pub fn IconColumns(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "3", y: "3", width: "18", height: "18", rx: "2" }
            line { x1: "12", y1: "3", x2: "12", y2: "21" }
        }
    }
}

/// Tag icon
#[component]
pub fn IconTag(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z" }
            line { x1: "7", y1: "7", x2: "7.01", y2: "7" }
        }
    }
}

/// Activity / Pulse icon for vault health doctor
#[component]
pub fn IconActivity(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "22 12 18 12 15 21 9 3 6 12 2 12" }
        }
    }
}

/// Quote icon for citations and bibliography
#[component]
pub fn IconQuote(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M3 21c3 0 7-1 7-8V5c0-1.25-.75-2-2-2H4c-1.25 0-2 .75-2 2v6c0 1.25.75 2 2 2 1 0 1 0 1 1 0 2.5-2 4.5-2 7" }
            path { d: "M15 21c3 0 7-1 7-8V5c0-1.25-.75-2-2-2h-4c-1.25 0-2 .75-2 2v6c0 1.25.75 2 2 2 1 0 1 0 1 1 0 2.5-2 4.5-2 7" }
        }
    }
}

/// Globe icon for Web Clipper and network status
#[component]
pub fn IconGlobe(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            line { x1: "2", y1: "12", x2: "22", y2: "12" }
            path { d: "M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" }
        }
    }
}

/// More / Horizontal 3 dots icon (⋯)
#[component]
pub fn IconMore(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "currentColor",
            stroke: "none",
            circle { cx: "12", cy: "12", r: "2" }
            circle { cx: "19", cy: "12", r: "2" }
            circle { cx: "5", cy: "12", r: "2" }
        }
    }
}

/// Panel Right / Inspector toggle icon
#[component]
pub fn IconPanelRight(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "3", y: "3", width: "18", height: "18", rx: "2" }
            line { x1: "15", y1: "3", x2: "15", y2: "21" }
        }
    }
}

/// Properties / Frontmatter metadata icon
#[component]
pub fn IconProperties(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "4", y1: "21", x2: "4", y2: "14" }
            line { x1: "4", y1: "10", x2: "4", y2: "3" }
            line { x1: "12", y1: "21", x2: "12", y2: "12" }
            line { x1: "12", y1: "8", x2: "12", y2: "3" }
            line { x1: "20", y1: "21", x2: "20", y2: "16" }
            line { x1: "20", y1: "12", x2: "20", y2: "3" }
            line { x1: "1", y1: "14", x2: "7", y2: "14" }
            line { x1: "9", y1: "8", x2: "15", y2: "8" }
            line { x1: "17", y1: "16", x2: "23", y2: "16" }
        }
    }
}

/// Review Queue / Inbox triage icon
#[component]
pub fn IconReviewQueue(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M22 12h-6l-2 3h-4l-2-3H2v7a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-7z" }
            path { d: "M5.45 5.11L2 12v0" }
            path { d: "M18.55 5.11L22 12v0" }
            path { d: "M6 5h12l3 7H3l3-7z" }
        }
    }
}

/// Rough Note / Quick thought icon (lightning bolt)
#[component]
pub fn IconRough(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polygon { points: "13 2 3 14 12 14 11 22 21 10 12 10 13 2" }
        }
    }
}

/// Permanent Note icon (diamond / atomic core)
#[component]
pub fn IconPermanent(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polygon { points: "6 3 18 3 22 9 12 22 2 9 6 3" }
            line { x1: "12", y1: "22", x2: "12", y2: "9" }
            line { x1: "2", y1: "9", x2: "22", y2: "9" }
        }
    }
}

/// Source Note icon (bookmark / reference)
#[component]
pub fn IconSource(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" }
        }
    }
}

/// Index / Map of Content icon (compass / network hub)
#[component]
pub fn IconIndex(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            polygon { points: "16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76" }
        }
    }
}

/// Copy icon (clipboard / duplicate)
#[component]
pub fn IconCopy(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "9", y: "9", width: "13", height: "13", rx: "2", ry: "2" }
            path { d: "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" }
        }
    }
}
