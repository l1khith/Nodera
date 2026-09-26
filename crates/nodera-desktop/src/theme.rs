use serde::{Deserialize, Serialize};

/// Lightweight color token wrapper that supports hex and rgba formats,
/// CSS formatting, and WCAG contrast calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub &'static str);

impl Color {
    pub const fn new(s: &'static str) -> Self {
        Self(s)
    }

    pub const fn as_str(&self) -> &'static str {
        self.0
    }

    /// Parses RGB channels from a hex string (e.g. "#1A2B3C" or "#123").
    pub fn parse_rgb(&self) -> Option<(u8, u8, u8)> {
        let s = self.0.trim();
        if !s.starts_with('#') {
            return None;
        }
        let hex = &s[1..];
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        } else if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some((r, g, b))
        } else {
            None
        }
    }

    /// Calculates relative luminance according to WCAG 2.1 specification.
    pub fn relative_luminance(&self) -> Option<f64> {
        let (r, g, b) = self.parse_rgb()?;
        let to_linear = |val: u8| -> f64 {
            let c = val as f64 / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let r_lin = to_linear(r);
        let g_lin = to_linear(g);
        let b_lin = to_linear(b);
        Some(0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin)
    }

    /// Calculates contrast ratio between two colors according to WCAG 2.1.
    pub fn contrast_ratio(&self, other: &Color) -> Option<f64> {
        let l1 = self.relative_luminance()?;
        let l2 = other.relative_luminance()?;
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        Some((lighter + 0.05) / (darker + 0.05))
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for Color {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl AsRef<str> for Color {
    fn as_ref(&self) -> &str {
        self.0
    }
}

/// Authoritative stable identifiers for Nodera's 6 built-in themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ThemeId {
    #[default]
    NoderaDark,
    NoderaLight,
    Midnight,
    Nord,
    Dracula,
    Solarized,
}

impl ThemeId {
    #[allow(non_upper_case_globals)]
    pub const Dark: Self = Self::NoderaDark;
    #[allow(non_upper_case_globals)]
    pub const Light: Self = Self::NoderaLight;

    pub const ALL: [Self; 6] = [
        Self::NoderaDark,
        Self::NoderaLight,
        Self::Midnight,
        Self::Nord,
        Self::Dracula,
        Self::Solarized,
    ];

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NoderaDark => "nodera-dark",
            Self::NoderaLight => "nodera-light",
            Self::Midnight => "midnight",
            Self::Nord => "nord",
            Self::Dracula => "dracula",
            Self::Solarized => "solarized",
        }
    }

    pub fn from_str_lossy(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "nodera-dark" | "dark" => Self::NoderaDark,
            "nodera-light" | "light" => Self::NoderaLight,
            "midnight" => Self::Midnight,
            "nord" => Self::Nord,
            "dracula" => Self::Dracula,
            "solarized" => Self::Solarized,
            _ => Self::NoderaDark,
        }
    }

    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::NoderaDark => "Nodera Dark",
            Self::NoderaLight => "Nodera Light",
            Self::Midnight => "Midnight",
            Self::Nord => "Nord",
            Self::Dracula => "Dracula",
            Self::Solarized => "Solarized",
        }
    }

    pub const fn description(&self) -> &'static str {
        match self {
            Self::NoderaDark => "Reference dark workspace with cobalt & relational violet",
            Self::NoderaLight => "Clean, readable light workspace for long writing sessions",
            Self::Midnight => "Deep blue-black OLED-friendly workspace with high depth",
            Self::Nord => "Cool, muted arctic palette for low visual distraction",
            Self::Dracula => "High-contrast dark palette with vibrant syntax accents",
            Self::Solarized => "Warm, low-contrast reading palette engineered for comfort",
        }
    }

    pub const fn css_class(&self) -> &'static str {
        match self {
            Self::NoderaDark => "theme-nodera-dark",
            Self::NoderaLight => "theme-nodera-light",
            Self::Midnight => "theme-midnight",
            Self::Nord => "theme-nord",
            Self::Dracula => "theme-dracula",
            Self::Solarized => "theme-solarized",
        }
    }

    pub const fn is_dark(&self) -> bool {
        match self {
            Self::NoderaDark | Self::Midnight | Self::Nord | Self::Dracula => true,
            Self::NoderaLight | Self::Solarized => false,
        }
    }

    pub const fn toggle(&self) -> Self {
        if self.is_dark() {
            Self::NoderaLight
        } else {
            Self::NoderaDark
        }
    }

    pub const fn theme(&self) -> Theme {
        Theme::from_id(*self)
    }
}

pub const ALL_THEME_IDS: [ThemeId; 6] = ThemeId::ALL;

impl Serialize for ThemeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ThemeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match serde_json::Value::deserialize(deserializer) {
            Ok(serde_json::Value::String(s)) => Ok(ThemeId::from_str_lossy(&s)),
            _ => Ok(ThemeId::NoderaDark),
        }
    }
}

/// Centralized semantic design token model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub id: ThemeId,
    pub name: &'static str,
    pub description: &'static str,
    pub is_dark: bool,

    // Core surfaces
    pub bg_app: Color,
    pub bg_sidebar: Color,
    pub bg_sidebar_hover: Color,
    pub bg_sidebar_active: Color,
    pub bg_surface: Color,
    pub bg_surface_elevated: Color,
    pub bg_hover: Color,
    pub bg_active: Color,

    // Structure / Borders
    pub border: Color,
    pub border_strong: Color,
    pub border_subtle: Color,

    // Text & Foreground
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_disabled: Color,

    // Interaction / Accents
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    pub accent_secondary: Color,
    pub accent_secondary_hover: Color,
    pub accent_focus: Color,
    pub focus: Color,
    pub selection: Color,

    // Semantic States
    pub success: Color,
    pub success_container: Color,
    pub warning: Color,
    pub warning_container: Color,
    pub danger: Color,
    pub danger_hover: Color,
    pub danger_container: Color,
    pub info: Color,
    pub info_container: Color,

    // Knowledge Graph
    pub graph_node: Color,
    pub graph_node_current: Color,
    pub graph_node_selected: Color,
    pub graph_node_hover: Color,
    pub graph_node_connected: Color,
    pub graph_node_unrelated: Color,
    pub graph_edge: Color,
    pub graph_edge_highlight: Color,
    pub graph_label: Color,
    pub graph_grid_dot: Color,

    // Editor & Code
    pub editor_background: Color,
    pub editor_text: Color,
    pub code_background: Color,
    pub code_text: Color,
    pub scrollbar_thumb: Color,
    pub scrollbar_thumb_hover: Color,
    pub scrollbar_thumb_active: Color,
    pub shadow_sm: Color,
    pub shadow_md: Color,
    pub shadow_lg: Color,
}

impl Theme {
    pub const fn nodera_dark() -> Self {
        Self {
            id: ThemeId::NoderaDark,
            name: "Nodera Dark",
            description: "Reference dark workspace with cobalt & relational violet",
            is_dark: true,
            bg_app: Color("#0B0F14"),
            bg_sidebar: Color("#11161D"),
            bg_sidebar_hover: Color("#202533"),
            bg_sidebar_active: Color("#252A3A"),
            bg_surface: Color("#141720"),
            bg_surface_elevated: Color("#161D26"),
            bg_hover: Color("#202533"),
            bg_active: Color("#252A3A"),
            border: Color("#28313C"),
            border_strong: Color("#383F4F"),
            border_subtle: Color("#1E232F"),
            text_primary: Color("#DEE2ED"),
            text_secondary: Color("#C5C5D6"),
            text_muted: Color("#8E90A0"),
            text_disabled: Color("#444654"),
            accent: Color("#6680FF"),
            accent_hover: Color("#7182FF"),
            accent_pressed: Color("#4A55E8"),
            accent_secondary: Color("#9A4BFF"),
            accent_secondary_hover: Color("#AC68FF"),
            accent_focus: Color("rgba(102, 128, 255, 0.25)"),
            focus: Color("#6680FF"),
            selection: Color("#252A3A"),
            success: Color("#35B875"),
            success_container: Color("#163527"),
            warning: Color("#E3A93B"),
            warning_container: Color("#392C16"),
            danger: Color("#E45B63"),
            danger_hover: Color("#F06A72"),
            danger_container: Color("#391A1D"),
            info: Color("#4FA3E3"),
            info_container: Color("#172E40"),
            graph_node: Color("#6680FF"),
            graph_node_current: Color("#9A4BFF"),
            graph_node_selected: Color("#9A4BFF"),
            graph_node_hover: Color("#7182FF"),
            graph_node_connected: Color("#7182FF"),
            graph_node_unrelated: Color("#444A5B"),
            graph_edge: Color("#444A5B"),
            graph_edge_highlight: Color("#7182FF"),
            graph_label: Color("#DEE2ED"),
            graph_grid_dot: Color("rgba(255, 255, 255, 0.08)"),
            editor_background: Color("#0B0F14"),
            editor_text: Color("#DEE2ED"),
            code_background: Color("#161D26"),
            code_text: Color("#DEE2ED"),
            scrollbar_thumb: Color("rgba(255, 255, 255, 0.09)"),
            scrollbar_thumb_hover: Color("rgba(255, 255, 255, 0.22)"),
            scrollbar_thumb_active: Color("rgba(255, 255, 255, 0.36)"),
            shadow_sm: Color("0 1px 3px rgba(0, 0, 0, 0.2)"),
            shadow_md: Color("0 4px 16px rgba(0, 0, 0, 0.28)"),
            shadow_lg: Color("0 16px 40px rgba(0, 0, 0, 0.45)"),
        }
    }

    pub const fn nodera_light() -> Self {
        Self {
            id: ThemeId::NoderaLight,
            name: "Nodera Light",
            description: "Clean, readable light workspace for long writing sessions",
            is_dark: false,
            bg_app: Color("#F4F6F9"),
            bg_sidebar: Color("#ECEFF3"),
            bg_sidebar_hover: Color("#E2E6EC"),
            bg_sidebar_active: Color("#D5DBE4"),
            bg_surface: Color("#FFFFFF"),
            bg_surface_elevated: Color("#F8FAFC"),
            bg_hover: Color("#EEF2F6"),
            bg_active: Color("#E4E9F2"),
            border: Color("#D1D7E0"),
            border_strong: Color("#A8B2C0"),
            border_subtle: Color("#E6EAF0"),
            text_primary: Color("#171C23"),
            text_secondary: Color("#475060"),
            text_muted: Color("#6B7687"),
            text_disabled: Color("#98A2B3"),
            accent: Color("#4A63E8"),
            accent_hover: Color("#3B53D8"),
            accent_pressed: Color("#2D41B8"),
            accent_secondary: Color("#7E3FE0"),
            accent_secondary_hover: Color("#9253F0"),
            accent_focus: Color("rgba(74, 99, 232, 0.2)"),
            focus: Color("#4A63E8"),
            selection: Color("#DDE4FF"),
            success: Color("#20864E"),
            success_container: Color("#E6F6ED"),
            warning: Color("#A87116"),
            warning_container: Color("#FDF3E1"),
            danger: Color("#C83B47"),
            danger_hover: Color("#AF2F3B"),
            danger_container: Color("#FCE8EA"),
            info: Color("#2B78B8"),
            info_container: Color("#E7F3FC"),
            graph_node: Color("#4A63E8"),
            graph_node_current: Color("#7E3FE0"),
            graph_node_selected: Color("#7E3FE0"),
            graph_node_hover: Color("#3B53D8"),
            graph_node_connected: Color("#3B53D8"),
            graph_node_unrelated: Color("#98A2B3"),
            graph_edge: Color("#D1D7E0"),
            graph_edge_highlight: Color("#4A63E8"),
            graph_label: Color("#171C23"),
            graph_grid_dot: Color("rgba(23, 28, 35, 0.08)"),
            editor_background: Color("#F4F6F9"),
            editor_text: Color("#171C23"),
            code_background: Color("#F8FAFC"),
            code_text: Color("#171C23"),
            scrollbar_thumb: Color("rgba(0, 0, 0, 0.11)"),
            scrollbar_thumb_hover: Color("rgba(0, 0, 0, 0.24)"),
            scrollbar_thumb_active: Color("rgba(0, 0, 0, 0.38)"),
            shadow_sm: Color("0 1px 3px rgba(0, 0, 0, 0.08)"),
            shadow_md: Color("0 4px 12px rgba(0, 0, 0, 0.12)"),
            shadow_lg: Color("0 16px 36px rgba(0, 0, 0, 0.16)"),
        }
    }

    pub const fn midnight() -> Self {
        Self {
            id: ThemeId::Midnight,
            name: "Midnight",
            description: "Deep blue-black OLED-friendly workspace with high depth",
            is_dark: true,
            bg_app: Color("#020408"),
            bg_sidebar: Color("#060A10"),
            bg_sidebar_hover: Color("#0D1420"),
            bg_sidebar_active: Color("#131D2E"),
            bg_surface: Color("#080D16"),
            bg_surface_elevated: Color("#0E1624"),
            bg_hover: Color("#141F32"),
            bg_active: Color("#1A2840"),
            border: Color("#162234"),
            border_strong: Color("#253650"),
            border_subtle: Color("#0E1622"),
            text_primary: Color("#E2E8F0"),
            text_secondary: Color("#94A3B8"),
            text_muted: Color("#64748B"),
            text_disabled: Color("#334155"),
            accent: Color("#38BDF8"),
            accent_hover: Color("#60A5FA"),
            accent_pressed: Color("#2563EB"),
            accent_secondary: Color("#818CF8"),
            accent_secondary_hover: Color("#A5B4FC"),
            accent_focus: Color("rgba(56, 189, 248, 0.25)"),
            focus: Color("#38BDF8"),
            selection: Color("#1E293B"),
            success: Color("#34D399"),
            success_container: Color("#064E3B"),
            warning: Color("#FBBF24"),
            warning_container: Color("#78350F"),
            danger: Color("#F87171"),
            danger_hover: Color("#EF4444"),
            danger_container: Color("#7F1D1D"),
            info: Color("#38BDF8"),
            info_container: Color("#0C4A6E"),
            graph_node: Color("#38BDF8"),
            graph_node_current: Color("#818CF8"),
            graph_node_selected: Color("#818CF8"),
            graph_node_hover: Color("#60A5FA"),
            graph_node_connected: Color("#60A5FA"),
            graph_node_unrelated: Color("#1E293B"),
            graph_edge: Color("#1E293B"),
            graph_edge_highlight: Color("#38BDF8"),
            graph_label: Color("#E2E8F0"),
            graph_grid_dot: Color("rgba(56, 189, 248, 0.08)"),
            editor_background: Color("#020408"),
            editor_text: Color("#E2E8F0"),
            code_background: Color("#0E1624"),
            code_text: Color("#E2E8F0"),
            scrollbar_thumb: Color("rgba(255, 255, 255, 0.08)"),
            scrollbar_thumb_hover: Color("rgba(255, 255, 255, 0.20)"),
            scrollbar_thumb_active: Color("rgba(255, 255, 255, 0.32)"),
            shadow_sm: Color("0 1px 3px rgba(0, 0, 0, 0.35)"),
            shadow_md: Color("0 4px 16px rgba(0, 0, 0, 0.45)"),
            shadow_lg: Color("0 16px 40px rgba(0, 0, 0, 0.65)"),
        }
    }

    pub const fn nord() -> Self {
        Self {
            id: ThemeId::Nord,
            name: "Nord",
            description: "Cool, muted arctic palette for low visual distraction",
            is_dark: true,
            bg_app: Color("#2E3440"),
            bg_sidebar: Color("#292E39"),
            bg_sidebar_hover: Color("#3B4252"),
            bg_sidebar_active: Color("#434C5E"),
            bg_surface: Color("#333A47"),
            bg_surface_elevated: Color("#3B4252"),
            bg_hover: Color("#434C5E"),
            bg_active: Color("#4C566A"),
            border: Color("#3B4252"),
            border_strong: Color("#4C566A"),
            border_subtle: Color("#353B49"),
            text_primary: Color("#ECEFF4"),
            text_secondary: Color("#D8DEE9"),
            text_muted: Color("#9FA8B8"),
            text_disabled: Color("#4C566A"),
            accent: Color("#88C0D0"),
            accent_hover: Color("#8FBCBB"),
            accent_pressed: Color("#81A1C1"),
            accent_secondary: Color("#B48EAD"),
            accent_secondary_hover: Color("#C69EC0"),
            accent_focus: Color("rgba(136, 192, 208, 0.25)"),
            focus: Color("#88C0D0"),
            selection: Color("#434C5E"),
            success: Color("#A3BE8C"),
            success_container: Color("#2A3A2A"),
            warning: Color("#EBCB8B"),
            warning_container: Color("#3F3820"),
            danger: Color("#BF616A"),
            danger_hover: Color("#D06F79"),
            danger_container: Color("#3E2327"),
            info: Color("#81A1C1"),
            info_container: Color("#243345"),
            graph_node: Color("#88C0D0"),
            graph_node_current: Color("#B48EAD"),
            graph_node_selected: Color("#B48EAD"),
            graph_node_hover: Color("#8FBCBB"),
            graph_node_connected: Color("#81A1C1"),
            graph_node_unrelated: Color("#4C566A"),
            graph_edge: Color("#434C5E"),
            graph_edge_highlight: Color("#88C0D0"),
            graph_label: Color("#ECEFF4"),
            graph_grid_dot: Color("rgba(216, 222, 233, 0.08)"),
            editor_background: Color("#2E3440"),
            editor_text: Color("#ECEFF4"),
            code_background: Color("#3B4252"),
            code_text: Color("#ECEFF4"),
            scrollbar_thumb: Color("rgba(216, 222, 233, 0.12)"),
            scrollbar_thumb_hover: Color("rgba(216, 222, 233, 0.25)"),
            scrollbar_thumb_active: Color("rgba(216, 222, 233, 0.40)"),
            shadow_sm: Color("0 1px 3px rgba(0, 0, 0, 0.2)"),
            shadow_md: Color("0 4px 16px rgba(0, 0, 0, 0.28)"),
            shadow_lg: Color("0 16px 40px rgba(0, 0, 0, 0.45)"),
        }
    }

    pub const fn dracula() -> Self {
        Self {
            id: ThemeId::Dracula,
            name: "Dracula",
            description: "High-contrast dark palette with vibrant syntax accents",
            is_dark: true,
            bg_app: Color("#21222C"),
            bg_sidebar: Color("#1D1E26"),
            bg_sidebar_hover: Color("#2C2E3E"),
            bg_sidebar_active: Color("#44475A"),
            bg_surface: Color("#282A36"),
            bg_surface_elevated: Color("#2F3242"),
            bg_hover: Color("#383A4C"),
            bg_active: Color("#44475A"),
            border: Color("#3B3E52"),
            border_strong: Color("#6272A4"),
            border_subtle: Color("#2C2E3E"),
            text_primary: Color("#F8F8F2"),
            text_secondary: Color("#D6D6E0"),
            text_muted: Color("#9AA6C4"),
            text_disabled: Color("#545B78"),
            accent: Color("#BD93F9"),
            accent_hover: Color("#CAA7FA"),
            accent_pressed: Color("#A877F0"),
            accent_secondary: Color("#FF79C6"),
            accent_secondary_hover: Color("#FF92D0"),
            accent_focus: Color("rgba(189, 147, 249, 0.25)"),
            focus: Color("#BD93F9"),
            selection: Color("#44475A"),
            success: Color("#50FA7B"),
            success_container: Color("#1C3B24"),
            warning: Color("#FFB86C"),
            warning_container: Color("#3D2C1A"),
            danger: Color("#FF5555"),
            danger_hover: Color("#FF6E6E"),
            danger_container: Color("#3D1B1B"),
            info: Color("#8BE9FD"),
            info_container: Color("#1C353D"),
            graph_node: Color("#BD93F9"),
            graph_node_current: Color("#FF79C6"),
            graph_node_selected: Color("#FF79C6"),
            graph_node_hover: Color("#8BE9FD"),
            graph_node_connected: Color("#8BE9FD"),
            graph_node_unrelated: Color("#44475A"),
            graph_edge: Color("#44475A"),
            graph_edge_highlight: Color("#BD93F9"),
            graph_label: Color("#F8F8F2"),
            graph_grid_dot: Color("rgba(248, 248, 242, 0.08)"),
            editor_background: Color("#282A36"),
            editor_text: Color("#F8F8F2"),
            code_background: Color("#2F3242"),
            code_text: Color("#F8F8F2"),
            scrollbar_thumb: Color("rgba(248, 248, 242, 0.12)"),
            scrollbar_thumb_hover: Color("rgba(248, 248, 242, 0.25)"),
            scrollbar_thumb_active: Color("rgba(248, 248, 242, 0.40)"),
            shadow_sm: Color("0 1px 3px rgba(0, 0, 0, 0.25)"),
            shadow_md: Color("0 4px 16px rgba(0, 0, 0, 0.35)"),
            shadow_lg: Color("0 16px 40px rgba(0, 0, 0, 0.55)"),
        }
    }

    pub const fn solarized() -> Self {
        Self {
            id: ThemeId::Solarized,
            name: "Solarized",
            description: "Warm, low-contrast reading palette engineered for comfort",
            is_dark: false,
            bg_app: Color("#FDF6E3"),
            bg_sidebar: Color("#F5EED9"),
            bg_sidebar_hover: Color("#E8E1CD"),
            bg_sidebar_active: Color("#EEE8D5"),
            bg_surface: Color("#FFFFFF"),
            bg_surface_elevated: Color("#EEE8D5"),
            bg_hover: Color("#E9E2CE"),
            bg_active: Color("#DFD7C2"),
            border: Color("#D6CFBA"),
            border_strong: Color("#93A1A1"),
            border_subtle: Color("#EBE4D0"),
            text_primary: Color("#586E75"),
            text_secondary: Color("#657B83"),
            text_muted: Color("#839496"),
            text_disabled: Color("#A8B3B5"),
            accent: Color("#268BD2"),
            accent_hover: Color("#1F76B4"),
            accent_pressed: Color("#1A6398"),
            accent_secondary: Color("#6C71C4"),
            accent_secondary_hover: Color("#5A5FA8"),
            accent_focus: Color("rgba(38, 139, 210, 0.25)"),
            focus: Color("#268BD2"),
            selection: Color("#EEE8D5"),
            success: Color("#859900"),
            success_container: Color("#EEF3D8"),
            warning: Color("#B58900"),
            warning_container: Color("#FAF2D7"),
            danger: Color("#DC322F"),
            danger_hover: Color("#C62B28"),
            danger_container: Color("#FAEAE8"),
            info: Color("#2AA198"),
            info_container: Color("#E0F4F2"),
            graph_node: Color("#268BD2"),
            graph_node_current: Color("#6C71C4"),
            graph_node_selected: Color("#6C71C4"),
            graph_node_hover: Color("#2AA198"),
            graph_node_connected: Color("#2AA198"),
            graph_node_unrelated: Color("#D6CFBA"),
            graph_edge: Color("#D6CFBA"),
            graph_edge_highlight: Color("#268BD2"),
            graph_label: Color("#586E75"),
            graph_grid_dot: Color("rgba(88, 110, 117, 0.10)"),
            editor_background: Color("#FDF6E3"),
            editor_text: Color("#586E75"),
            code_background: Color("#EEE8D5"),
            code_text: Color("#586E75"),
            scrollbar_thumb: Color("rgba(88, 110, 117, 0.18)"),
            scrollbar_thumb_hover: Color("rgba(88, 110, 117, 0.32)"),
            scrollbar_thumb_active: Color("rgba(88, 110, 117, 0.48)"),
            shadow_sm: Color("0 1px 3px rgba(0, 0, 0, 0.08)"),
            shadow_md: Color("0 4px 12px rgba(0, 0, 0, 0.12)"),
            shadow_lg: Color("0 16px 36px rgba(0, 0, 0, 0.16)"),
        }
    }

    pub const fn from_id(id: ThemeId) -> Self {
        match id {
            ThemeId::NoderaDark => Self::nodera_dark(),
            ThemeId::NoderaLight => Self::nodera_light(),
            ThemeId::Midnight => Self::midnight(),
            ThemeId::Nord => Self::nord(),
            ThemeId::Dracula => Self::dracula(),
            ThemeId::Solarized => Self::solarized(),
        }
    }

    pub const fn all() -> [Self; 6] {
        [
            Self::nodera_dark(),
            Self::nodera_light(),
            Self::midnight(),
            Self::nord(),
            Self::dracula(),
            Self::solarized(),
        ]
    }

    pub const fn css_class(&self) -> &'static str {
        self.id.css_class()
    }

    // Shorthand semantic accessors matching token guidelines
    pub const fn background(&self) -> Color {
        self.bg_app
    }

    pub const fn surface(&self) -> Color {
        self.bg_surface
    }

    pub const fn surface_alt(&self) -> Color {
        self.bg_surface_elevated
    }

    pub const fn text(&self) -> Color {
        self.text_primary
    }
}

pub const BASE_CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Geist:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500;600;700&display=swap');

:root, .theme-dark, .theme-nodera-dark {
    --bg-app: #0B0F14;
    --bg-sidebar: #11161D;
    --bg-sidebar-hover: #202533;
    --bg-sidebar-active: #252A3A;
    --bg-surface: #141720;
    --bg-surface-elevated: #161D26;
    --bg-hover: #202533;
    --bg-active: #252A3A;
    --border: #28313C;
    --border-strong: #383F4F;
    --border-subtle: #1E232F;
    --text-primary: #DEE2ED;
    --text-secondary: #C5C5D6;
    --text-muted: #8E90A0;
    --text-disabled: #444654;
    --accent: #6680FF;
    --accent-hover: #7182FF;
    --accent-pressed: #4A55E8;
    --accent-secondary: #9A4BFF;
    --accent-secondary-hover: #AC68FF;
    --accent-focus: rgba(102, 128, 255, 0.25);
    --selection: #252A3A;
    --focus: #6680FF;
    --success: #35B875;
    --success-container: #163527;
    --warning: #E3A93B;
    --warning-container: #392C16;
    --danger: #E45B63;
    --danger-hover: #F06A72;
    --danger-container: #391A1D;
    --info: #4FA3E3;
    --info-container: #172E40;
    --status-success: var(--success);
    --status-warning: var(--warning);
    --status-danger: var(--danger);
    --status-info: var(--info);
    --graph-node: #6680FF;
    --graph-node-current: #9A4BFF;
    --graph-node-selected: #9A4BFF;
    --graph-node-hover: #7182FF;
    --graph-node-connected: #7182FF;
    --graph-node-unrelated: #444A5B;
    --graph-edge: #444A5B;
    --graph-edge-highlight: #7182FF;
    --graph-label: #DEE2ED;
    --graph-grid-dot: rgba(255, 255, 255, 0.08);
    --shadow-none: none;
    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.2);
    --shadow-md: 0 4px 16px rgba(0, 0, 0, 0.28);
    --shadow-lg: 0 16px 40px rgba(0, 0, 0, 0.45);
    --font-ui: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-editor: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-mono: "JetBrains Mono", "Cascadia Code", "Fira Code", Consolas, monospace;
    --border-color: var(--border);
    --bg-primary: var(--bg-app);
    --bg-secondary: var(--bg-surface);
    --bg-tertiary: var(--bg-surface-elevated);
    --accent-color: var(--accent);
    --scrollbar-thumb: rgba(255, 255, 255, 0.09);
    --scrollbar-thumb-hover: rgba(255, 255, 255, 0.22);
    --scrollbar-thumb-active: rgba(255, 255, 255, 0.36);
}

.theme-light, .theme-nodera-light {
    --bg-app: #F4F6F9;
    --bg-sidebar: #ECEFF3;
    --bg-sidebar-hover: #E2E6EC;
    --bg-sidebar-active: #D5DBE4;
    --bg-surface: #FFFFFF;
    --bg-surface-elevated: #F8FAFC;
    --bg-hover: #EEF2F6;
    --bg-active: #E4E9F2;
    --border: #D1D7E0;
    --border-strong: #A8B2C0;
    --border-subtle: #E6EAF0;
    --text-primary: #171C23;
    --text-secondary: #475060;
    --text-muted: #6B7687;
    --text-disabled: #98A2B3;
    --accent: #4A63E8;
    --accent-hover: #3B53D8;
    --accent-pressed: #2D41B8;
    --accent-secondary: #7E3FE0;
    --accent-secondary-hover: #9253F0;
    --accent-focus: rgba(74, 99, 232, 0.2);
    --selection: #DDE4FF;
    --focus: #4A63E8;
    --success: #20864E;
    --success-container: #E6F6ED;
    --warning: #A87116;
    --warning-container: #FDF3E1;
    --danger: #C83B47;
    --danger-hover: #AF2F3B;
    --danger-container: #FCE8EA;
    --info: #2B78B8;
    --info-container: #E7F3FC;
    --status-success: var(--success);
    --status-warning: var(--warning);
    --status-danger: var(--danger);
    --status-info: var(--info);
    --graph-node: #4A63E8;
    --graph-node-current: #7E3FE0;
    --graph-node-selected: #7E3FE0;
    --graph-node-hover: #3B53D8;
    --graph-node-connected: #3B53D8;
    --graph-node-unrelated: #98A2B3;
    --graph-edge: #D1D7E0;
    --graph-edge-highlight: #4A63E8;
    --graph-label: #171C23;
    --graph-grid-dot: rgba(23, 28, 35, 0.08);
    --shadow-none: none;
    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.08);
    --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.12);
    --shadow-lg: 0 16px 36px rgba(0, 0, 0, 0.16);
    --font-ui: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-editor: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-mono: "JetBrains Mono", "Cascadia Code", "Fira Code", Consolas, monospace;
    --border-color: var(--border);
    --bg-primary: var(--bg-app);
    --bg-secondary: var(--bg-surface);
    --bg-tertiary: var(--bg-surface-elevated);
    --accent-color: var(--accent);
    --scrollbar-thumb: rgba(0, 0, 0, 0.11);
    --scrollbar-thumb-hover: rgba(0, 0, 0, 0.24);
    --scrollbar-thumb-active: rgba(0, 0, 0, 0.38);
}

.theme-midnight {
    --bg-app: #020408;
    --bg-sidebar: #060A10;
    --bg-sidebar-hover: #0D1420;
    --bg-sidebar-active: #131D2E;
    --bg-surface: #080D16;
    --bg-surface-elevated: #0E1624;
    --bg-hover: #141F32;
    --bg-active: #1A2840;
    --border: #162234;
    --border-strong: #253650;
    --border-subtle: #0E1622;
    --text-primary: #E2E8F0;
    --text-secondary: #94A3B8;
    --text-muted: #64748B;
    --text-disabled: #334155;
    --accent: #38BDF8;
    --accent-hover: #60A5FA;
    --accent-pressed: #2563EB;
    --accent-secondary: #818CF8;
    --accent-secondary-hover: #A5B4FC;
    --accent-focus: rgba(56, 189, 248, 0.25);
    --selection: #1E293B;
    --focus: #38BDF8;
    --success: #34D399;
    --success-container: #064E3B;
    --warning: #FBBF24;
    --warning-container: #78350F;
    --danger: #F87171;
    --danger-hover: #EF4444;
    --danger-container: #7F1D1D;
    --info: #38BDF8;
    --info-container: #0C4A6E;
    --status-success: var(--success);
    --status-warning: var(--warning);
    --status-danger: var(--danger);
    --status-info: var(--info);
    --graph-node: #38BDF8;
    --graph-node-current: #818CF8;
    --graph-node-selected: #818CF8;
    --graph-node-hover: #60A5FA;
    --graph-node-connected: #60A5FA;
    --graph-node-unrelated: #1E293B;
    --graph-edge: #1E293B;
    --graph-edge-highlight: #38BDF8;
    --graph-label: #E2E8F0;
    --graph-grid-dot: rgba(56, 189, 248, 0.08);
    --shadow-none: none;
    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.35);
    --shadow-md: 0 4px 16px rgba(0, 0, 0, 0.45);
    --shadow-lg: 0 16px 40px rgba(0, 0, 0, 0.65);
    --font-ui: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-editor: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-mono: "JetBrains Mono", "Cascadia Code", "Fira Code", Consolas, monospace;
    --border-color: var(--border);
    --bg-primary: var(--bg-app);
    --bg-secondary: var(--bg-surface);
    --bg-tertiary: var(--bg-surface-elevated);
    --accent-color: var(--accent);
    --scrollbar-thumb: rgba(255, 255, 255, 0.08);
    --scrollbar-thumb-hover: rgba(255, 255, 255, 0.20);
    --scrollbar-thumb-active: rgba(255, 255, 255, 0.32);
}

.theme-nord {
    --bg-app: #2E3440;
    --bg-sidebar: #292E39;
    --bg-sidebar-hover: #3B4252;
    --bg-sidebar-active: #434C5E;
    --bg-surface: #333A47;
    --bg-surface-elevated: #3B4252;
    --bg-hover: #434C5E;
    --bg-active: #4C566A;
    --border: #3B4252;
    --border-strong: #4C566A;
    --border-subtle: #353B49;
    --text-primary: #ECEFF4;
    --text-secondary: #D8DEE9;
    --text-muted: #9FA8B8;
    --text-disabled: #4C566A;
    --accent: #88C0D0;
    --accent-hover: #8FBCBB;
    --accent-pressed: #81A1C1;
    --accent-secondary: #B48EAD;
    --accent-secondary-hover: #C69EC0;
    --accent-focus: rgba(136, 192, 208, 0.25);
    --selection: #434C5E;
    --focus: #88C0D0;
    --success: #A3BE8C;
    --success-container: #2A3A2A;
    --warning: #EBCB8B;
    --warning-container: #3F3820;
    --danger: #BF616A;
    --danger-hover: #D06F79;
    --danger-container: #3E2327;
    --info: #81A1C1;
    --info-container: #243345;
    --status-success: var(--success);
    --status-warning: var(--warning);
    --status-danger: var(--danger);
    --status-info: var(--info);
    --graph-node: #88C0D0;
    --graph-node-current: #B48EAD;
    --graph-node-selected: #B48EAD;
    --graph-node-hover: #8FBCBB;
    --graph-node-connected: #81A1C1;
    --graph-node-unrelated: #4C566A;
    --graph-edge: #434C5E;
    --graph-edge-highlight: #88C0D0;
    --graph-label: #ECEFF4;
    --graph-grid-dot: rgba(216, 222, 233, 0.08);
    --shadow-none: none;
    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.2);
    --shadow-md: 0 4px 16px rgba(0, 0, 0, 0.28);
    --shadow-lg: 0 16px 40px rgba(0, 0, 0, 0.45);
    --font-ui: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-editor: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-mono: "JetBrains Mono", "Cascadia Code", "Fira Code", Consolas, monospace;
    --border-color: var(--border);
    --bg-primary: var(--bg-app);
    --bg-secondary: var(--bg-surface);
    --bg-tertiary: var(--bg-surface-elevated);
    --accent-color: var(--accent);
    --scrollbar-thumb: rgba(216, 222, 233, 0.12);
    --scrollbar-thumb-hover: rgba(216, 222, 233, 0.25);
    --scrollbar-thumb-active: rgba(216, 222, 233, 0.40);
}

.theme-dracula {
    --bg-app: #21222C;
    --bg-sidebar: #1D1E26;
    --bg-sidebar-hover: #2C2E3E;
    --bg-sidebar-active: #44475A;
    --bg-surface: #282A36;
    --bg-surface-elevated: #2F3242;
    --bg-hover: #383A4C;
    --bg-active: #44475A;
    --border: #3B3E52;
    --border-strong: #6272A4;
    --border-subtle: #2C2E3E;
    --text-primary: #F8F8F2;
    --text-secondary: #D6D6E0;
    --text-muted: #9AA6C4;
    --text-disabled: #545B78;
    --accent: #BD93F9;
    --accent-hover: #CAA7FA;
    --accent-pressed: #A877F0;
    --accent-secondary: #FF79C6;
    --accent-secondary-hover: #FF92D0;
    --accent-focus: rgba(189, 147, 249, 0.25);
    --selection: #44475A;
    --focus: #BD93F9;
    --success: #50FA7B;
    --success-container: #1C3B24;
    --warning: #FFB86C;
    --warning-container: #3D2C1A;
    --danger: #FF5555;
    --danger-hover: #FF6E6E;
    --danger-container: #3D1B1B;
    --info: #8BE9FD;
    --info-container: #1C353D;
    --status-success: var(--success);
    --status-warning: var(--warning);
    --status-danger: var(--danger);
    --status-info: var(--info);
    --graph-node: #BD93F9;
    --graph-node-current: #FF79C6;
    --graph-node-selected: #FF79C6;
    --graph-node-hover: #8BE9FD;
    --graph-node-connected: #8BE9FD;
    --graph-node-unrelated: #44475A;
    --graph-edge: #44475A;
    --graph-edge-highlight: #BD93F9;
    --graph-label: #F8F8F2;
    --graph-grid-dot: rgba(248, 248, 242, 0.08);
    --shadow-none: none;
    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.25);
    --shadow-md: 0 4px 16px rgba(0, 0, 0, 0.35);
    --shadow-lg: 0 16px 40px rgba(0, 0, 0, 0.55);
    --font-ui: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-editor: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-mono: "JetBrains Mono", "Cascadia Code", "Fira Code", Consolas, monospace;
    --border-color: var(--border);
    --bg-primary: var(--bg-app);
    --bg-secondary: var(--bg-surface);
    --bg-tertiary: var(--bg-surface-elevated);
    --accent-color: var(--accent);
    --scrollbar-thumb: rgba(248, 248, 242, 0.12);
    --scrollbar-thumb-hover: rgba(248, 248, 242, 0.25);
    --scrollbar-thumb-active: rgba(248, 248, 242, 0.40);
}

.theme-solarized {
    --bg-app: #FDF6E3;
    --bg-sidebar: #F5EED9;
    --bg-sidebar-hover: #E8E1CD;
    --bg-sidebar-active: #EEE8D5;
    --bg-surface: #FFFFFF;
    --bg-surface-elevated: #EEE8D5;
    --bg-hover: #E9E2CE;
    --bg-active: #DFD7C2;
    --border: #D6CFBA;
    --border-strong: #93A1A1;
    --border-subtle: #EBE4D0;
    --text-primary: #586E75;
    --text-secondary: #657B83;
    --text-muted: #839496;
    --text-disabled: #A8B3B5;
    --accent: #268BD2;
    --accent-hover: #1F76B4;
    --accent-pressed: #1A6398;
    --accent-secondary: #6C71C4;
    --accent-secondary-hover: #5A5FA8;
    --accent-focus: rgba(38, 139, 210, 0.25);
    --selection: #EEE8D5;
    --focus: #268BD2;
    --success: #859900;
    --success-container: #EEF3D8;
    --warning: #B58900;
    --warning-container: #FAF2D7;
    --danger: #DC322F;
    --danger-hover: #C62B28;
    --danger-container: #FAEAE8;
    --info: #2AA198;
    --info-container: #E0F4F2;
    --status-success: var(--success);
    --status-warning: var(--warning);
    --status-danger: var(--danger);
    --status-info: var(--info);
    --graph-node: #268BD2;
    --graph-node-current: #6C71C4;
    --graph-node-selected: #6C71C4;
    --graph-node-hover: #2AA198;
    --graph-node-connected: #2AA198;
    --graph-node-unrelated: #D6CFBA;
    --graph-edge: #D6CFBA;
    --graph-edge-highlight: #268BD2;
    --graph-label: #586E75;
    --graph-grid-dot: rgba(88, 110, 117, 0.10);
    --shadow-none: none;
    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.08);
    --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.12);
    --shadow-lg: 0 16px 36px rgba(0, 0, 0, 0.16);
    --font-ui: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-editor: Geist, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    --font-mono: "JetBrains Mono", "Cascadia Code", "Fira Code", Consolas, monospace;
    --border-color: var(--border);
    --bg-primary: var(--bg-app);
    --bg-secondary: var(--bg-surface);
    --bg-tertiary: var(--bg-surface-elevated);
    --accent-color: var(--accent);
    --scrollbar-thumb: rgba(88, 110, 117, 0.18);
    --scrollbar-thumb-hover: rgba(88, 110, 117, 0.32);
    --scrollbar-thumb-active: rgba(88, 110, 117, 0.48);
}

* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body, html {
    width: 100%;
    height: 100%;
    overflow: hidden;
    background-color: var(--bg-app);
    color: var(--text-primary);
    font-family: var(--font-ui);
    font-size: 13px;
    user-select: none;
}

button {
    font-family: inherit;
    font-size: inherit;
    color: inherit;
    background: none;
    border: none;
    cursor: pointer;
}

input, textarea {
    font-family: inherit;
    color: inherit;
    background: transparent;
    border: 1px solid var(--border);
    outline: none;
}

:focus-visible {
    outline: 1px solid var(--focus);
    box-shadow: 0 0 0 2px var(--accent-focus);
}

/* Layout classes */
.app-container {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    background-color: var(--bg-app);
}

.top-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 40px;
    background-color: var(--bg-sidebar);
    border-bottom: 1px solid var(--border);
    padding: 0 12px;
    gap: 12px;
}

.top-bar-left, .top-bar-center, .top-bar-right {
    display: flex;
    align-items: center;
    gap: 8px;
}

.brand-title {
    font-weight: 700;
    font-size: 14px;
    letter-spacing: 0.5px;
    color: var(--accent);
}

.vault-badge {
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 2px 8px;
    font-size: 12px;
    color: var(--text-secondary);
    max-width: 200px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.main-workspace {
    display: flex;
    flex: 1;
    overflow: hidden;
    position: relative;
}

.pane-sidebar {
    width: 100%;
    min-width: 180px;
    background-color: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transition: width 0.15s ease;
}

.pane-resizer {
    width: 5px;
    cursor: col-resize;
    background-color: transparent;
    border-left: 1px solid var(--border);
    transition: background-color 0.15s ease;
    z-index: 5;
    flex-shrink: 0;
}

.pane-resizer:hover {
    background-color: var(--accent);
}

.pane-center {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-app);
    overflow: hidden;
}

/* Multi-tab bar */
.tab-bar {
    display: flex;
    align-items: center;
    background-color: var(--bg-surface-elevated);
    border-bottom: 1px solid var(--border);
    height: 38px;
    padding: 0 8px;
    gap: 4px;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: thin;
    user-select: none;
}

.tab-item {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: 6px 6px 0 0;
    font-size: 12px;
    color: var(--text-secondary);
    background: transparent;
    border: 1px solid transparent;
    border-bottom: none;
    cursor: pointer;
    max-width: 180px;
    white-space: nowrap;
    transition: all 0.12s ease;
}

.tab-item:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
}

.tab-item.active {
    background-color: var(--bg-surface);
    color: var(--text-primary);
    font-weight: 500;
    border-color: var(--border);
}

.tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 120px;
}

.tab-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    color: var(--text-muted);
    transition: all 0.1s ease;
}

.tab-close:hover {
    background-color: var(--bg-active);
    color: var(--text-primary);
}

.tab-dirty-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: var(--accent);
    display: inline-block;
}

/* Breadcrumbs */
.breadcrumb-container {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-muted);
}

.breadcrumb-segment {
    color: var(--text-secondary);
}

.breadcrumb-active {
    color: var(--text-primary);
    font-weight: 600;
}

/* ==========================================================================
   Editor & Document Scrollbar Styling (Minimal, native-feel document scrollbar)
   ========================================================================== */

textarea.editor-textarea,
.editor-textarea,
.reading-view,
.reading-toc {
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) transparent;
    scrollbar-gutter: stable;
    overflow-x: hidden !important;
}

textarea.editor-textarea,
.editor-textarea {
    overflow-y: auto;
    word-wrap: break-word;
    overflow-wrap: break-word;
}

textarea.editor-textarea:hover,
.editor-textarea:hover,
.reading-view:hover,
.reading-toc:hover {
    scrollbar-color: var(--scrollbar-thumb-hover) transparent;
}

/* WebKit / Chromium (Edge WebView2) Custom Scrollbar */
textarea.editor-textarea::-webkit-scrollbar,
.editor-textarea::-webkit-scrollbar,
.reading-view::-webkit-scrollbar,
.reading-toc::-webkit-scrollbar {
    width: 6px;
    height: 6px;
}

textarea.editor-textarea::-webkit-scrollbar-track,
.editor-textarea::-webkit-scrollbar-track,
.reading-view::-webkit-scrollbar-track,
.reading-toc::-webkit-scrollbar-track {
    background: transparent !important;
    border: none !important;
}

textarea.editor-textarea::-webkit-scrollbar-thumb,
.editor-textarea::-webkit-scrollbar-thumb,
.reading-view::-webkit-scrollbar-thumb,
.reading-toc::-webkit-scrollbar-thumb {
    background-color: var(--scrollbar-thumb);
    border-radius: 4px;
    transition: background-color 0.15s ease;
}

textarea.editor-textarea:hover::-webkit-scrollbar-thumb,
.editor-textarea:hover::-webkit-scrollbar-thumb,
.reading-view:hover::-webkit-scrollbar-thumb,
.reading-toc:hover::-webkit-scrollbar-thumb,
textarea.editor-textarea:focus::-webkit-scrollbar-thumb,
.editor-textarea:focus::-webkit-scrollbar-thumb {
    background-color: var(--scrollbar-thumb-hover);
}

textarea.editor-textarea::-webkit-scrollbar-thumb:hover,
.editor-textarea::-webkit-scrollbar-thumb:hover,
.reading-view::-webkit-scrollbar-thumb:hover,
.reading-toc::-webkit-scrollbar-thumb:hover {
    background-color: var(--scrollbar-thumb-active);
}

textarea.editor-textarea::-webkit-scrollbar-button,
.editor-textarea::-webkit-scrollbar-button,
.reading-view::-webkit-scrollbar-button,
.reading-toc::-webkit-scrollbar-button {
    display: none !important;
    width: 0 !important;
    height: 0 !important;
}

textarea.editor-textarea::-webkit-scrollbar-corner,
.editor-textarea::-webkit-scrollbar-corner,
.reading-view::-webkit-scrollbar-corner,
.reading-toc::-webkit-scrollbar-corner {
    background: transparent !important;
}

.pane-context {
    min-width: 180px;
    background-color: var(--bg-sidebar);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transition: width 0.15s ease;
}

.pane-hidden {
    display: none !important;
}

.bottom-statusbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 24px;
    background-color: var(--bg-sidebar);
    border-top: 1px solid var(--border);
    padding: 0 12px;
    font-size: 11px;
    color: var(--text-muted);
}

.btn-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 4px;
    color: var(--text-secondary);
    transition: background-color 0.1s;
}

.btn-icon:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
}

/* Vector Icon Utilities */
.icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    vertical-align: middle;
    flex-shrink: 0;
}

.icon-sm {
    width: 14px;
    height: 14px;
}

.icon-md {
    width: 16px;
    height: 16px;
}

.icon-lg {
    width: 20px;
    height: 20px;
}

.icon-hero {
    width: 48px;
    height: 48px;
}

.btn-action {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: 6px;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    font-size: 12px;
    color: var(--text-primary);
    transition: background-color 0.12s ease, border-color 0.12s ease;
}

.btn-action:hover {
    background-color: var(--bg-hover);
    border-color: var(--accent);
}

.btn-primary {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    background-color: var(--accent);
    color: #ffffff;
    font-weight: 600;
    font-size: 13px;
    border: 1px solid var(--accent);
    border-radius: 6px;
    height: 34px;
    padding: 6px 14px;
    transition: background-color 0.12s ease, border-color 0.12s ease;
}

.btn-primary:hover {
    background-color: var(--accent-hover);
    border-color: var(--accent-hover);
}

.btn-primary:active {
    background-color: var(--accent-pressed);
    border-color: var(--accent-pressed);
}

.btn-danger {
    color: var(--danger);
    background: transparent;
    border-radius: 6px;
    transition: background-color 0.12s ease, color 0.12s ease;
}

.btn-danger:hover {
    background-color: var(--danger-container);
    color: var(--danger-hover);
}

/* Modal styles */
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
}

.modal-dialog {
    background-color: var(--bg-surface);
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    padding: 20px;
    width: 420px;
    max-width: 90vw;
    box-shadow: var(--shadow-lg);
}

.modal-title {
    font-size: 16px;
    font-weight: 600;
    margin-bottom: 12px;
}

.modal-body {
    margin-bottom: 20px;
    color: var(--text-secondary);
}

.modal-input {
    width: 100%;
    padding: 6px 10px;
    border-radius: 6px;
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    font-size: 13px;
    color: var(--text-primary);
    margin-top: 8px;
}

.modal-input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-focus);
}

.modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
}

/* Reading View & Markdown Styling */
.reading-view {
    font-family: var(--font-editor);
    font-size: 15px;
    line-height: 26px;
    color: var(--text-primary);
}

.editor-textarea {
    font-family: var(--font-editor);
    font-size: 15px;
    font-weight: 400;
    line-height: 26px;
    letter-spacing: 0;
    color: var(--text-primary);
    background: transparent;
    border: none;
    outline: none;
    resize: none;
}

.reading-view h1 { font-family: var(--font-ui); font-size: 24px; font-weight: 700; line-height: 30px; letter-spacing: -0.01em; margin: 24px 0 12px; border-bottom: 1px solid var(--border); padding-bottom: 8px; color: var(--text-primary); }
.reading-view h2 { font-family: var(--font-ui); font-size: 20px; font-weight: 600; line-height: 26px; letter-spacing: -0.005em; margin: 20px 0 10px; border-bottom: 1px solid var(--border-subtle); padding-bottom: 6px; color: var(--text-primary); }
.reading-view h3 { font-family: var(--font-ui); font-size: 17px; font-weight: 600; line-height: 23px; letter-spacing: 0; margin: 16px 0 8px; color: var(--text-primary); }
.reading-view h4, .reading-view h5, .reading-view h6 { font-family: var(--font-ui); font-size: 14px; font-weight: 600; line-height: 22px; margin: 12px 0 6px; color: var(--text-primary); }
.reading-view p { font-family: var(--font-editor); font-size: 15px; font-weight: 400; line-height: 26px; letter-spacing: 0; margin: 12px 0; }
.reading-view ul, .reading-view ol { margin: 10px 0 10px 24px; line-height: 24px; }
.reading-view li { margin: 4px 0; }
.reading-view blockquote { border-left: 3px solid var(--accent); padding: 6px 16px; margin: 12px 0; color: var(--text-secondary); background: var(--bg-surface); border-radius: 0 4px 4px 0; }
.reading-view code { background: var(--bg-surface-elevated); padding: 2px 6px; border-radius: 3px; font-family: var(--font-mono); font-size: 13px; line-height: 20px; }
.reading-view pre { background: var(--bg-surface-elevated); padding: 12px; border-radius: 6px; overflow-x: auto; margin: 14px 0; border: 1px solid var(--border); font-family: var(--font-mono); font-size: 13px; line-height: 20px; }
.reading-view pre code { background: none; padding: 0; font-family: var(--font-mono); }
.reading-view table { border-collapse: collapse; width: 100%; margin: 16px 0; }
.reading-view th, .reading-view td { border: 1px solid var(--border); padding: 8px 12px; text-align: left; }
.reading-view th { background: var(--bg-surface); font-weight: 600; }
.reading-view a { color: var(--accent); text-decoration: none; }
.reading-view a:hover { text-decoration: underline; color: var(--accent-hover); }
.reading-view a.wikilink { color: var(--accent-secondary); font-weight: 500; border-bottom: 1px dashed var(--accent-secondary); padding-bottom: 1px; transition: color 0.12s ease, background-color 0.12s ease; }
.reading-view a.wikilink:hover { color: var(--accent-secondary-hover); background-color: rgba(154, 75, 255, 0.15); border-radius: 2px; }

/* Callout blocks */
.callout {
    margin: 16px 0;
    border-radius: 6px;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-left-width: 4px;
    overflow: hidden;
}

.callout-title {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    font-weight: 600;
    font-size: 13px;
    background-color: var(--bg-hover);
    border-bottom: 1px solid var(--border-subtle);
}

.callout-content {
    padding: 10px 14px;
    font-size: 13px;
    line-height: 1.6;
    color: var(--text-primary);
}

.callout-content p:first-child { margin-top: 0; }
.callout-content p:last-child { margin-bottom: 0; }

.callout-note { border-left-color: var(--info); }
.callout-note .callout-title { color: var(--info); }

.callout-tip { border-left-color: var(--success); }
.callout-tip .callout-title { color: var(--success); }

.callout-warning { border-left-color: var(--warning); }
.callout-warning .callout-title { color: var(--warning); }

.callout-important { border-left-color: var(--accent-secondary); }
.callout-important .callout-title { color: var(--accent-secondary); }

.callout-caution { border-left-color: var(--danger); }
.callout-caution .callout-title { color: var(--danger); }

.callout-success { border-left-color: var(--success); }
.callout-success .callout-title { color: var(--success); }

/* Math rendering */
.math {
    font-family: 'KaTeX_Math', 'Cambria Math', 'Times New Roman', serif;
    font-style: italic;
    color: var(--accent-hover);
}

.math-display {
    display: block;
    text-align: center;
    margin: 14px 0;
    overflow-x: auto;
}

/* Context Panel & Knowledge Components */
.tag-badge { display: inline-block; background-color: var(--bg-surface-elevated); color: var(--accent-hover); border: 1px solid var(--border); border-radius: 4px; padding: 2px 8px; font-family: var(--font-mono); font-size: 11px; font-weight: 500; margin: 2px 4px 2px 0; }
.link-list { display: flex; flex-direction: column; gap: 4px; margin-top: 6px; }
.link-item { display: flex; align-items: center; justify-content: space-between; padding: 5px 8px; border-radius: 4px; font-size: 12px; cursor: pointer; color: var(--text-primary); background-color: var(--bg-surface); border: 1px solid var(--border-subtle); transition: background-color 0.1s, border-color 0.1s; text-align: left; width: 100%; }
.link-item:hover { background-color: var(--bg-hover); border-color: var(--accent); }
.link-item-unresolved { color: var(--text-muted); font-style: italic; }
.command-palette-row:hover { background-color: var(--bg-hover); }

/* 2D Knowledge Graph Styling */
.graph-container {
    position: relative;
    width: 100%;
    height: 100%;
    flex: 1;
    overflow: hidden;
    background-color: var(--bg-app);
    user-select: none;
}

.graph-canvas {
    width: 100%;
    height: 100%;
    display: block;
    cursor: grab;
}

.graph-canvas:active {
    cursor: grabbing;
}

.graph-node-circle {
    transition: r 0.18s cubic-bezier(0.16, 1, 0.3, 1),
                opacity 0.18s ease-out,
                stroke 0.18s ease-out,
                stroke-width 0.18s ease-out;
}

.graph-edge-line {
    transition: opacity 0.18s ease-out,
                stroke 0.18s ease-out,
                stroke-width 0.18s ease-out;
}

.graph-node-label {
    user-select: none;
    pointer-events: none;
    font-size: 11px;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    fill: var(--text-primary);
    paint-order: stroke fill;
    stroke: var(--bg-app);
    stroke-width: 3px;
    stroke-linejoin: round;
    transition: opacity 0.18s ease-out, fill 0.18s ease-out;
}

.graph-node-label.active {
    font-size: 12px;
    font-weight: 600;
    fill: var(--text-primary);
    stroke: var(--bg-app);
    stroke-width: 3.5px;
}

.graph-halo-ring {
    pointer-events: none;
    transition: r 0.18s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.18s ease-out;
}

.graph-color-palette-select {
    width: 100%;
    padding: 6px 8px;
    font-size: 11px;
    background: var(--bg-surface-elevated);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    outline: none;
    cursor: pointer;
}

.graph-color-row {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 6px 0;
}

.graph-color-label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 11px;
    color: var(--text-secondary);
}

.graph-swatches {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
}

.graph-swatch {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    cursor: pointer;
    border: 2px solid transparent;
    transition: transform 0.1s ease, border-color 0.1s ease;
    padding: 0;
    outline: none;
}

.graph-swatch:hover {
    transform: scale(1.15);
}

.graph-swatch.active {
    border-color: #ffffff;
    box-shadow: 0 0 0 1px var(--accent);
}

.graph-color-custom-input {
    width: 72px;
    font-size: 10px;
    font-family: var(--font-mono);
    padding: 2px 6px;
    background: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-primary);
    text-transform: uppercase;
}

.graph-toolbar {
    position: absolute;
    top: 14px;
    right: 14px;
    display: flex;
    align-items: center;
    gap: 3px;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 20px;
    padding: 3px 6px;
    box-shadow: var(--shadow-sm);
    z-index: 10;
}

.graph-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: 13px;
    color: var(--text-secondary);
    transition: background-color 0.1s, color 0.1s;
}

.graph-btn:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
}

.graph-zoom-label {
    font-size: 11px;
    font-weight: 500;
    min-width: 36px;
    text-align: center;
    color: var(--text-muted);
    user-select: none;
}

.graph-divider {
    width: 1px;
    height: 14px;
    background-color: var(--border);
    margin: 0 2px;
}

.graph-stats {
    position: absolute;
    bottom: 14px;
    left: 14px;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 14px;
    padding: 4px 12px;
    font-size: 11px;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    gap: 10px;
    box-shadow: var(--shadow-sm);
    z-index: 10;
}

.graph-search-bar {
    position: absolute;
    top: 14px;
    left: 14px;
    display: flex;
    align-items: center;
    gap: 8px;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 20px;
    padding: 4px 12px;
    box-shadow: var(--shadow-sm);
    z-index: 10;
    min-width: 200px;
}

.local-graph-box {
    width: 100%;
    height: 220px;
    position: relative;
    border-radius: 6px;
    overflow: hidden;
    background-color: var(--bg-surface);
    border: 1px solid var(--border-subtle);
}

.graph-pill-group {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 2px 4px;
}

.graph-pill {
    font-size: 10px;
    font-weight: 500;
    color: var(--text-muted);
    background: transparent;
    border: none;
    border-radius: 8px;
    padding: 1px 6px;
    cursor: pointer;
    transition: all 0.1s ease;
}

.graph-pill:hover {
    color: var(--text-primary);
    background-color: var(--bg-hover);
}

.graph-pill.active {
    color: #ffffff;
    background-color: var(--accent);
}

/* Graph Settings Drawer */
.graph-canvas-wrapper {
    position: relative;
    flex: 1;
    height: 100%;
    overflow: hidden;
}

.graph-settings-panel {
    width: 280px;
    height: 100%;
    background-color: var(--bg-surface);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    z-index: 20;
    box-shadow: -4px 0 16px rgba(0, 0, 0, 0.2);
    animation: slideInRight 0.15s ease-out;
}

@keyframes slideInRight {
    from { transform: translateX(20px); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
}

.graph-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border-subtle);
}

.graph-panel-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: 0.3px;
}

.graph-panel-content {
    flex: 1;
    overflow-y: auto;
    padding: 10px 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.graph-section {
    display: flex;
    flex-direction: column;
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: 10px;
}

.graph-section:last-child {
    border-bottom: none;
}

.graph-section-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 0;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    cursor: pointer;
    user-select: none;
    transition: color 0.1s;
}

.graph-section-header:hover {
    color: var(--text-primary);
}

.graph-section-body {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 4px;
}

.graph-toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 3px 0;
    font-size: 12px;
    color: var(--text-secondary);
}

.graph-toggle-row:hover {
    color: var(--text-primary);
}

.graph-switch {
    position: relative;
    display: inline-block;
    width: 30px;
    height: 16px;
}

.graph-switch input {
    opacity: 0;
    width: 0;
    height: 0;
}

.graph-switch-slider {
    position: absolute;
    cursor: pointer;
    top: 0; left: 0; right: 0; bottom: 0;
    background-color: var(--border-strong);
    transition: .15s ease;
    border-radius: 16px;
}

.graph-switch-slider:before {
    position: absolute;
    content: "";
    height: 12px;
    width: 12px;
    left: 2px;
    bottom: 2px;
    background-color: var(--text-primary);
    transition: .15s ease;
    border-radius: 50%;
}

input:checked + .graph-switch-slider {
    background-color: var(--accent);
}

input:checked + .graph-switch-slider:before {
    transform: translateX(14px);
    background-color: #ffffff;
}

.graph-slider-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 4px 0;
}

.graph-slider-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 11px;
    color: var(--text-secondary);
}

.graph-slider-val {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-muted);
}

.graph-slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    border-radius: 2px;
    background: var(--border-strong);
    outline: none;
    margin: 4px 0;
}

.graph-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--accent);
    cursor: pointer;
    transition: background 0.15s;
}

.graph-slider::-webkit-slider-thumb:hover {
    background: var(--accent-hover);
}

/* Library View & Document Cards */
.library-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
    background-color: var(--bg-app);
}

.library-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 28px;
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-surface);
    flex-shrink: 0;
}

.library-content {
    flex: 1;
    overflow-y: auto;
    padding: 28px;
}

.library-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
    gap: 24px;
}

.library-card {
    display: flex;
    flex-direction: column;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    overflow: hidden;
    box-shadow: var(--shadow-md);
    transition: transform 0.15s ease, border-color 0.15s ease, box-shadow 0.15s ease;
}

.library-card:hover {
    transform: translateY(-2px);
    border-color: var(--accent);
    box-shadow: var(--shadow-lg);
}

.library-card-header {
    padding: 16px 18px;
    background-color: var(--bg-surface-elevated);
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    align-items: flex-start;
    gap: 14px;
}

.library-card-cover {
    width: 48px;
    height: 64px;
    border-radius: 4px;
    background-color: var(--accent);
    color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    border: 1px solid var(--border-subtle);
}

.library-card-body {
    padding: 16px 18px;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.library-badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
}

.library-tag {
    font-size: 11px;
    font-weight: 500;
    font-family: var(--font-mono);
    padding: 3px 8px;
    border-radius: 4px;
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    color: var(--accent-hover);
}

.library-progress-track {
    height: 6px;
    border-radius: 3px;
    background-color: var(--bg-hover);
    border: 1px solid var(--border);
    overflow: hidden;
}

.library-progress-bar {
    height: 100%;
    background-color: var(--accent);
    border-radius: 2px;
    transition: width 0.3s ease;
}

.library-card-footer {
    padding: 12px 18px;
    border-top: 1px solid var(--border-subtle);
    background-color: var(--bg-surface-elevated);
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
}

/* Code Blocks & Syntax Highlighting */
.code-block-wrapper {
    margin: 16px 0;
    border-radius: 8px;
    border: 1px solid var(--border);
    background-color: var(--bg-surface);
    overflow: hidden;
    box-shadow: var(--shadow-sm);
}

.code-block-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 14px;
    background-color: var(--bg-surface-elevated);
    border-bottom: 1px solid var(--border-subtle);
    font-size: 11px;
    font-family: var(--font-mono);
}

.code-block-lang {
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
}

.code-block-copy-btn {
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-secondary);
    padding: 2px 8px;
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s ease;
}

.code-block-copy-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--accent);
}

.code-block-wrapper pre {
    margin: 0;
    padding: 14px 16px;
    overflow-x: auto;
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 20px;
}

/* Mermaid Diagrams */
.mermaid-diagram {
    margin: 20px 0;
    border-radius: 8px;
    border: 1px solid var(--border);
    background-color: var(--bg-surface);
    overflow: hidden;
}

.mermaid-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    background-color: var(--bg-surface-elevated);
    border-bottom: 1px solid var(--border-subtle);
    font-size: 11px;
}

.mermaid-badge {
    background: var(--accent-focus);
    color: var(--accent-hover);
    font-weight: 600;
    text-transform: uppercase;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 10px;
    letter-spacing: 0.5px;
}

.mermaid-title {
    color: var(--text-secondary);
    font-weight: 500;
}

.mermaid-svg {
    padding: 24px;
    display: flex;
    justify-content: center;
    align-items: center;
    overflow-x: auto;
    background-color: var(--bg-surface);
}

.mermaid-svg svg {
    max-width: 100%;
    height: auto;
}

/* Transcluded Note Embeds */
.note-embed {
    margin: 16px 0;
    border-radius: 8px;
    border: 1px solid var(--border);
    border-left: 4px solid var(--accent);
    background-color: var(--bg-surface);
    overflow: hidden;
    box-shadow: var(--shadow-sm);
}

.note-embed-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    background-color: var(--bg-surface-elevated);
    border-bottom: 1px solid var(--border-subtle);
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
}

.note-embed-title {
    color: var(--accent);
}

.note-embed-content {
    padding: 16px 20px;
    font-size: 13px;
    line-height: 1.65;
    color: var(--text-secondary);
    max-height: 400px;
    overflow-y: auto;
}

.note-embed-missing {
    margin: 12px 0;
    padding: 10px 14px;
    border-radius: 6px;
    border: 1px dashed var(--warning);
    background-color: rgba(227, 169, 59, 0.08);
    color: var(--warning);
    font-size: 12px;
    display: flex;
    align-items: center;
    gap: 8px;
}

/* Properties Drawer (Frontmatter) */
.properties-drawer {
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-surface);
    padding: 12px 20px;
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.properties-drawer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.property-count-badge {
    background-color: var(--accent-focus);
    color: var(--accent-hover);
    font-size: 10px;
    font-weight: 700;
    padding: 1px 6px;
    border-radius: 10px;
}

.properties-table {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.property-row {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 12px;
}

.property-label {
    width: 110px;
    font-weight: 500;
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 16px;
    letter-spacing: 0.01em;
    flex-shrink: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.property-value-cell {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
}

.property-input {
    flex: 1;
    min-width: 140px;
    max-width: 450px;
    background: var(--bg-app);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 4px 8px;
    color: var(--text-primary);
    font-size: 12px;
    outline: none;
    transition: border-color 0.15s ease;
}

.property-input:focus {
    border-color: var(--accent);
}

.property-mini-input {
    background: var(--bg-app);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 3px 8px;
    color: var(--text-primary);
    font-size: 11px;
    outline: none;
    transition: border-color 0.15s ease;
}

.property-mini-input:focus {
    border-color: var(--accent);
}

.property-type-select {
    background: var(--bg-app);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 3px 6px;
    color: var(--text-secondary);
    font-size: 11px;
    outline: none;
    cursor: pointer;
}

.property-add-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 6px;
    padding-top: 8px;
    border-top: 1px dashed var(--border-subtle);
}

.property-remove-btn {
    width: 20px;
    height: 20px;
    color: var(--text-muted);
}

.property-remove-btn:hover {
    color: var(--danger);
}

.tag-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    color: var(--accent-hover);
    border-radius: 4px;
    padding: 2px 8px;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 500;
}

.tag-chip-remove {
    background: transparent;
    border: none;
    color: currentColor;
    font-size: 12px;
    cursor: pointer;
    line-height: 1;
    padding: 0;
    opacity: 0.7;
}

.tag-chip-remove:hover {
    opacity: 1;
}

/* Split View */
.split-pane-header {
    height: 32px;
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-surface-elevated);
    display: flex;
    align-items: center;
    padding: 0 12px;
    user-select: none;
}

.split-divider {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    background-color: var(--bg-surface);
    border-left: 1px solid var(--border);
    border-right: 1px solid var(--border);
    padding: 6px;
    user-select: none;
    flex-shrink: 0;
}

/* Unlinked Mentions */
.unlinked-mention-card {
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    transition: border-color 0.15s ease;
}

.unlinked-mention-card:hover {
    border-color: var(--accent);
}

.unlinked-snippet {
    font-size: 11px;
    line-height: 1.5;
    color: var(--text-muted);
}

/* Vault Selector & Top Bar Actions */
.vault-selector-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
    cursor: pointer;
    transition: background-color 0.1s, border-color 0.1s;
}

.vault-selector-btn:hover {
    background-color: var(--bg-hover);
    border-color: var(--border-strong);
}

.top-search-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-width: 220px;
    max-width: 360px;
    padding: 4px 12px;
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: 12px;
    color: var(--text-muted);
    cursor: pointer;
    transition: border-color 0.15s ease, background-color 0.15s ease;
}

.top-search-btn:hover {
    border-color: var(--border-strong);
    color: var(--text-secondary);
    background-color: var(--bg-hover);
}

.kbd-badge {
    margin-left: auto;
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 1px 5px;
}

/* Popover Dropdown Menus */
.dropdown-menu {
    position: absolute;
    top: calc(100% + 6px);
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: var(--shadow-md);
    min-width: 200px;
    padding: 4px;
    z-index: 1000;
    display: flex;
    flex-direction: column;
}

.dropdown-header {
    padding: 6px 10px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
}

.dropdown-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    font-size: 12px;
    color: var(--text-primary);
    border-radius: 4px;
    cursor: pointer;
    background: transparent;
    border: none;
    width: 100%;
    text-align: left;
    transition: background-color 0.1s ease;
}

.dropdown-item:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
}

.dropdown-item.danger {
    color: var(--danger);
}

.dropdown-item.danger:hover {
    background-color: rgba(239, 68, 68, 0.1);
    color: var(--danger-hover);
}

.dropdown-divider {
    height: 1px;
    background-color: var(--border-subtle);
    margin: 4px 0;
}

/* Context Menu */
.context-menu {
    position: fixed;
    background-color: var(--bg-surface-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: var(--shadow-lg);
    min-width: 170px;
    padding: 4px;
    z-index: 1100;
    display: flex;
    flex-direction: column;
}

.context-menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    font-size: 12px;
    color: var(--text-primary);
    border-radius: 4px;
    cursor: pointer;
    background: transparent;
    border: none;
    width: 100%;
    text-align: left;
    transition: background-color 0.1s ease;
}

.context-menu-item:hover {
    background-color: var(--bg-hover);
}

.context-menu-item.danger {
    color: var(--danger);
}

.context-menu-item.danger:hover {
    background-color: rgba(239, 68, 68, 0.1);
    color: var(--danger-hover);
}

/* Calendar / Today Workspace */
.calendar-day-cell {
    user-select: none;
}

.calendar-day-cell:hover {
    background-color: var(--bg-hover) !important;
}

.calendar-day-cell:focus-visible {
    outline: 2px solid var(--focus) !important;
    outline-offset: 1px;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_six_themes_exist() {
        let all_themes = Theme::all();
        assert_eq!(all_themes.len(), 6);

        let ids = [
            ThemeId::NoderaDark,
            ThemeId::NoderaLight,
            ThemeId::Midnight,
            ThemeId::Nord,
            ThemeId::Dracula,
            ThemeId::Solarized,
        ];

        for (theme, expected_id) in all_themes.iter().zip(ids.iter()) {
            assert_eq!(theme.id, *expected_id);
            assert_eq!(Theme::from_id(*expected_id).id, *expected_id);
            assert_eq!(expected_id.theme().id, *expected_id);
            assert!(!theme.name.is_empty());
            assert!(!theme.description.is_empty());
        }
    }

    #[test]
    fn test_token_integrity() {
        for theme in Theme::all() {
            let tokens = [
                theme.bg_app,
                theme.bg_sidebar,
                theme.bg_sidebar_hover,
                theme.bg_sidebar_active,
                theme.bg_surface,
                theme.bg_surface_elevated,
                theme.bg_hover,
                theme.bg_active,
                theme.border,
                theme.border_strong,
                theme.border_subtle,
                theme.text_primary,
                theme.text_secondary,
                theme.text_muted,
                theme.text_disabled,
                theme.accent,
                theme.accent_hover,
                theme.accent_pressed,
                theme.accent_secondary,
                theme.accent_secondary_hover,
                theme.accent_focus,
                theme.focus,
                theme.selection,
                theme.success,
                theme.success_container,
                theme.warning,
                theme.warning_container,
                theme.danger,
                theme.danger_hover,
                theme.danger_container,
                theme.info,
                theme.info_container,
                theme.graph_node,
                theme.graph_node_current,
                theme.graph_node_selected,
                theme.graph_node_hover,
                theme.graph_node_connected,
                theme.graph_node_unrelated,
                theme.graph_edge,
                theme.graph_edge_highlight,
                theme.graph_label,
                theme.graph_grid_dot,
                theme.editor_background,
                theme.editor_text,
                theme.code_background,
                theme.code_text,
                theme.scrollbar_thumb,
                theme.scrollbar_thumb_hover,
                theme.scrollbar_thumb_active,
                theme.shadow_sm,
                theme.shadow_md,
                theme.shadow_lg,
            ];

            for token in tokens {
                assert!(
                    !token.as_str().is_empty(),
                    "Token in theme {:?} is empty",
                    theme.id
                );
            }
        }
    }

    #[test]
    fn test_theme_id_serialization() {
        let cases = [
            (ThemeId::NoderaDark, "\"nodera-dark\""),
            (ThemeId::NoderaLight, "\"nodera-light\""),
            (ThemeId::Midnight, "\"midnight\""),
            (ThemeId::Nord, "\"nord\""),
            (ThemeId::Dracula, "\"dracula\""),
            (ThemeId::Solarized, "\"solarized\""),
        ];

        for (id, json_str) in cases {
            let serialized = serde_json::to_string(&id).expect("Serialization failed");
            assert_eq!(serialized, json_str);

            let deserialized: ThemeId =
                serde_json::from_str(json_str).expect("Deserialization failed");
            assert_eq!(deserialized, id);
        }
    }

    #[test]
    fn test_theme_id_legacy_and_fallback() {
        // Legacy "Dark" and "Light"
        assert_eq!(
            serde_json::from_str::<ThemeId>("\"Dark\"").unwrap(),
            ThemeId::NoderaDark
        );
        assert_eq!(
            serde_json::from_str::<ThemeId>("\"Light\"").unwrap(),
            ThemeId::NoderaLight
        );
        assert_eq!(
            serde_json::from_str::<ThemeId>("\"dark\"").unwrap(),
            ThemeId::NoderaDark
        );
        assert_eq!(
            serde_json::from_str::<ThemeId>("\"light\"").unwrap(),
            ThemeId::NoderaLight
        );

        // Unknown theme IDs gracefully fallback to NoderaDark
        assert_eq!(
            serde_json::from_str::<ThemeId>("\"cyberpunk-neon\"").unwrap(),
            ThemeId::NoderaDark
        );
        assert_eq!(
            serde_json::from_str::<ThemeId>("\"\"").unwrap(),
            ThemeId::NoderaDark
        );
        assert_eq!(
            serde_json::from_str::<ThemeId>("12345").unwrap(),
            ThemeId::NoderaDark
        );
        assert_eq!(
            serde_json::from_str::<ThemeId>("null").unwrap(),
            ThemeId::NoderaDark
        );
    }

    #[test]
    fn test_theme_toggle() {
        assert_eq!(ThemeId::NoderaDark.toggle(), ThemeId::NoderaLight);
        assert_eq!(ThemeId::NoderaLight.toggle(), ThemeId::NoderaDark);
        assert_eq!(ThemeId::Midnight.toggle(), ThemeId::NoderaLight);
        assert_eq!(ThemeId::Nord.toggle(), ThemeId::NoderaLight);
        assert_eq!(ThemeId::Dracula.toggle(), ThemeId::NoderaLight);
        assert_eq!(ThemeId::Solarized.toggle(), ThemeId::NoderaDark);
    }

    #[test]
    fn test_css_classes() {
        assert_eq!(ThemeId::NoderaDark.css_class(), "theme-nodera-dark");
        assert_eq!(ThemeId::NoderaLight.css_class(), "theme-nodera-light");
        assert_eq!(ThemeId::Midnight.css_class(), "theme-midnight");
        assert_eq!(ThemeId::Nord.css_class(), "theme-nord");
        assert_eq!(ThemeId::Dracula.css_class(), "theme-dracula");
        assert_eq!(ThemeId::Solarized.css_class(), "theme-solarized");
    }

    #[test]
    fn test_contrast_ratios() {
        for theme in Theme::all() {
            // Test primary text vs background and surface
            let ratio_bg = theme
                .text_primary
                .contrast_ratio(&theme.bg_app)
                .expect("Failed to compute contrast ratio with bg_app");
            let ratio_surface = theme
                .text_primary
                .contrast_ratio(&theme.bg_surface)
                .expect("Failed to compute contrast ratio with bg_surface");

            // WCAG AA requirement for normal body text is at least 4.5:1
            assert!(
                ratio_bg >= 4.5,
                "Theme {:?} failed primary text contrast on bg_app: ratio {:.2}",
                theme.id,
                ratio_bg
            );
            assert!(
                ratio_surface >= 4.5,
                "Theme {:?} failed primary text contrast on bg_surface: ratio {:.2}",
                theme.id,
                ratio_surface
            );

            // Muted text should be at least 3.0:1 for secondary UI elements
            let ratio_muted = theme
                .text_muted
                .contrast_ratio(&theme.bg_surface)
                .expect("Failed to compute contrast ratio for text_muted");
            assert!(
                ratio_muted >= 3.0,
                "Theme {:?} failed muted text contrast: ratio {:.2}",
                theme.id,
                ratio_muted
            );
        }
    }

    #[test]
    fn test_preferences_persistence_roundtrip() {
        use crate::state::AppPreferences;

        for id in ThemeId::ALL {
            let prefs = AppPreferences {
                theme: id,
                ..Default::default()
            };
            let json = serde_json::to_string(&prefs).expect("Serialize preferences");
            let loaded: AppPreferences =
                serde_json::from_str(&json).expect("Deserialize preferences");
            assert_eq!(loaded.theme, id);
        }
    }

    #[test]
    fn test_graph_color_settings_palettes_and_overrides() {
        use crate::state::{GraphColorSettings, GraphPalettePreset};

        let mut colors = GraphColorSettings::default();
        assert_eq!(colors.palette, GraphPalettePreset::NoderaTech);
        assert_eq!(colors.effective_center_color(), "#9A4BFF");
        assert_eq!(colors.effective_sub_node_color(), "#5C6FE6");
        assert_eq!(colors.effective_selected_color(), "#7182FF");
        assert_eq!(colors.effective_edge_color(), "#444A5B");
        assert_eq!(colors.effective_text_color(), "#DEE2ED");

        // Custom override takes priority
        colors.center_node_color = Some("#FF0000".to_string());
        assert_eq!(colors.effective_center_color(), "#FF0000");

        // Palette presets
        colors.center_node_color = None;
        colors.palette = GraphPalettePreset::Cyberpunk;
        assert_eq!(colors.effective_center_color(), "#FF007F");
        assert_eq!(colors.effective_sub_node_color(), "#00F0FF");
        assert_eq!(colors.effective_selected_color(), "#FFE600");

        colors.palette = GraphPalettePreset::Emerald;
        assert_eq!(colors.effective_center_color(), "#10B981");

        // Semantic symbol colors
        assert_eq!(GraphColorSettings::symbol_kind_color("fn"), "#3B82F6");
        assert_eq!(GraphColorSettings::symbol_kind_color("struct"), "#A855F7");
        assert_eq!(GraphColorSettings::symbol_kind_color("enum"), "#FB923C");
        assert_eq!(GraphColorSettings::symbol_kind_color("trait"), "#10B981");
        assert_eq!(GraphColorSettings::symbol_kind_color("const"), "#EF4444");

        // Serialization roundtrip
        let json = serde_json::to_string(&colors).expect("Serialize GraphColorSettings");
        let deserialized: GraphColorSettings = serde_json::from_str(&json).expect("Deserialize");
        assert_eq!(deserialized, colors);
    }
}
