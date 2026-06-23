use ratatui::style::Color;

pub const THEME_NAMES: &[&str] = &["default", "green", "dracula"];

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

impl Theme {
    pub fn by_name(name: &str) -> Self {
        match name {
            "green" => Self::green(),
            "dracula" => Self::dracula(),
            _ => Self::default_theme(),
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
