use std::collections::HashMap;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

pub const BUILTIN_THEME_NAMES: &[&str] = &["default", "green", "dracula"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub accent: Option<String>,
    pub text: Option<String>,
    pub muted: Option<String>,
    pub warning: Option<String>,
    pub success: Option<String>,
    pub error: Option<String>,
    pub selection_fg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub accent: Color,
    pub text: Color,
    pub muted: Color,
    pub warning: Color,
    pub success: Color,
    pub error: Color,
    pub selection_fg: Color,
}

fn parse_color(s: &str) -> Color {
    match s.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "darkgray" | "darkgrey" | "dark_gray" | "dark_grey" => Color::DarkGray,
        "lightred" | "light_red" => Color::LightRed,
        "lightgreen" | "light_green" => Color::LightGreen,
        "lightyellow" | "light_yellow" => Color::LightYellow,
        "lightblue" | "light_blue" => Color::LightBlue,
        "lightmagenta" | "light_magenta" => Color::LightMagenta,
        "lightcyan" | "light_cyan" => Color::LightCyan,
        "gray" | "grey" => Color::Gray,
        _ => Color::White,
    }
}

pub fn all_theme_names(custom: &HashMap<String, ThemeColors>) -> Vec<String> {
    let mut names: Vec<String> = BUILTIN_THEME_NAMES.iter().map(|s| s.to_string()).collect();
    for key in custom.keys() {
        if !names.contains(key) {
            names.push(key.clone());
        }
    }
    names
}

impl Theme {
    pub fn by_name(name: &str, custom: &HashMap<String, ThemeColors>) -> Self {
        if let Some(colors) = custom.get(name) {
            return Self::from_custom(colors);
        }
        match name {
            "green" => Self::green(),
            "dracula" => Self::dracula(),
            _ => Self::default_theme(),
        }
    }

    fn from_custom(colors: &ThemeColors) -> Self {
        let base = Self::default_theme();
        Self {
            accent: colors.accent.as_deref().map(parse_color).unwrap_or(base.accent),
            text: colors.text.as_deref().map(parse_color).unwrap_or(base.text),
            muted: colors.muted.as_deref().map(parse_color).unwrap_or(base.muted),
            warning: colors.warning.as_deref().map(parse_color).unwrap_or(base.warning),
            success: colors.success.as_deref().map(parse_color).unwrap_or(base.success),
            error: colors.error.as_deref().map(parse_color).unwrap_or(base.error),
            selection_fg: colors.selection_fg.as_deref().map(parse_color).unwrap_or(base.selection_fg),
        }
    }

    fn default_theme() -> Self {
        Self {
            accent: Color::Cyan,
            text: Color::White,
            muted: Color::DarkGray,
            warning: Color::Yellow,
            success: Color::Green,
            error: Color::Red,
            selection_fg: Color::Black,
        }
    }

    fn green() -> Self {
        Self {
            accent: Color::Green,
            text: Color::White,
            muted: Color::DarkGray,
            warning: Color::Yellow,
            success: Color::LightGreen,
            error: Color::Red,
            selection_fg: Color::Black,
        }
    }

    fn dracula() -> Self {
        Self {
            accent: Color::Magenta,
            text: Color::White,
            muted: Color::DarkGray,
            warning: Color::Yellow,
            success: Color::Green,
            error: Color::Red,
            selection_fg: Color::Black,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_theme()
    }
}
