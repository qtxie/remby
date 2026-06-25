use gpui::Hsla;
use gpui_component::{Colorize, Theme, ThemeTokens, hsl};
use ratatui::style::Color;

struct RembyThemeColors {
    accent: Hsla,
    text: Hsla,
    muted: Hsla,
    warning: Hsla,
    success: Hsla,
    error: Hsla,
    selection_fg: Hsla,
    background: Hsla,
    card: Hsla,
    border: Hsla,
}

pub fn apply_remby_theme(cx: &mut gpui::App, theme_name: &str) {
    let custom_themes = remby_core::config::load_themes();
    let rt = remby_core::theme::Theme::by_name(theme_name, &custom_themes);

    let colors = RembyThemeColors {
        accent: ratatui_to_hsla(rt.accent),
        text: ratatui_to_hsla(rt.text),
        muted: ratatui_to_hsla(rt.muted),
        warning: ratatui_to_hsla(rt.warning),
        success: ratatui_to_hsla(rt.success),
        error: ratatui_to_hsla(rt.error),
        selection_fg: ratatui_to_hsla(rt.selection_fg),
        background: hsl(0., 0., 8.),
        card: hsl(0., 0., 12.),
        border: hsl(0., 0., 20.),
    };

    let theme = Theme::global_mut(cx);
    apply_color_map(theme, &colors);
}

fn apply_color_map(theme: &mut Theme, c: &RembyThemeColors) {
    let transparent = gpui::transparent_black();

    theme.foreground = c.text;
    theme.background = c.background;

    theme.primary = c.accent;
    theme.primary_foreground = c.selection_fg;
    theme.primary_hover = c.accent.opacity(0.85);
    theme.primary_active = c.accent.darken(0.15);
    theme.button_primary = c.accent;
    theme.button_primary_foreground = c.selection_fg;
    theme.button_primary_hover = c.accent.opacity(0.85);
    theme.button_primary_active = c.accent.darken(0.15);

    theme.accent = c.accent;
    theme.accent_foreground = c.selection_fg;

    theme.muted = c.muted;
    theme.muted_foreground = c.text.opacity(0.6);

    theme.warning = c.warning;
    theme.warning_foreground = c.selection_fg;
    theme.warning_hover = c.warning.opacity(0.85);
    theme.warning_active = c.warning.darken(0.15);
    theme.button_warning = c.warning.mix_oklab(transparent, 0.2);
    theme.button_warning_foreground = c.warning;
    theme.button_warning_hover = c.warning.mix_oklab(transparent, 0.3);
    theme.button_warning_active = c.warning.mix_oklab(transparent, 0.4);

    theme.success = c.success;
    theme.success_foreground = c.selection_fg;
    theme.success_hover = c.success.opacity(0.85);
    theme.success_active = c.success.darken(0.15);
    theme.button_success = c.success.mix_oklab(transparent, 0.2);
    theme.button_success_foreground = c.success;
    theme.button_success_hover = c.success.mix_oklab(transparent, 0.3);
    theme.button_success_active = c.success.mix_oklab(transparent, 0.4);

    theme.danger = c.error;
    theme.danger_foreground = c.selection_fg;
    theme.danger_hover = c.error.opacity(0.85);
    theme.danger_active = c.error.darken(0.15);
    theme.button_danger = c.error.mix_oklab(transparent, 0.2);
    theme.button_danger_foreground = c.error;
    theme.button_danger_hover = c.error.mix_oklab(transparent, 0.3);
    theme.button_danger_active = c.error.mix_oklab(transparent, 0.4);

    theme.info = c.accent;
    theme.info_foreground = c.selection_fg;
    theme.button_info = c.accent.mix_oklab(transparent, 0.2);
    theme.button_info_foreground = c.accent;
    theme.button_info_hover = c.accent.mix_oklab(transparent, 0.3);
    theme.button_info_active = c.accent.mix_oklab(transparent, 0.4);

    theme.popover = c.card;
    theme.border = c.border;
    theme.input = c.muted;

    theme.link = c.accent;
    theme.link_active = c.accent;
    theme.link_hover = c.accent;

    theme.selection = c.accent.opacity(0.25);

    theme.tokens = ThemeTokens::from(&theme.colors);
}

fn ratatui_to_hsla(c: Color) -> Hsla {
    match c {
        Color::Black => hsl(0., 0., 0.),
        Color::Red => hsl(0., 100., 50.),
        Color::Green => hsl(120., 100., 50.),
        Color::Yellow => hsl(60., 100., 50.),
        Color::Blue => hsl(220., 100., 50.),
        Color::Magenta => hsl(300., 100., 50.),
        Color::Cyan => hsl(180., 100., 50.),
        Color::White => hsl(0., 0., 100.),
        Color::DarkGray => hsl(0., 0., 25.),
        Color::Gray => hsl(0., 0., 50.),
        Color::LightRed => hsl(0., 100., 75.),
        Color::LightGreen => hsl(120., 100., 75.),
        Color::LightYellow => hsl(60., 100., 75.),
        Color::LightBlue => hsl(220., 100., 75.),
        Color::LightMagenta => hsl(300., 100., 75.),
        Color::LightCyan => hsl(180., 100., 75.),
        Color::Rgb(r, g, b) => rgb_to_hsla(r, g, b),
        Color::Indexed(n) => ansi_256_to_hsla(n),
        Color::Reset => hsl(0., 0., 100.),
    }
}

fn rgb_to_hsla(r: u8, g: u8, b: u8) -> Hsla {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;
    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f32::EPSILON {
        return hsl(0., 0., l * 100.0);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if max == rf {
        ((gf - bf) / d + if gf < bf { 6.0 } else { 0.0 }) / 6.0
    } else if max == gf {
        ((bf - rf) / d + 2.0) / 6.0
    } else {
        ((rf - gf) / d + 4.0) / 6.0
    };

    hsl(h * 360.0, s * 100.0, l * 100.0)
}

fn ansi_256_to_hsla(n: u8) -> Hsla {
    match n {
        0 => hsl(0., 0., 0.),
        1 => hsl(0., 100., 50.),
        2 => hsl(120., 100., 50.),
        3 => hsl(60., 100., 50.),
        4 => hsl(220., 100., 50.),
        5 => hsl(300., 100., 50.),
        6 => hsl(180., 100., 50.),
        7 => hsl(0., 0., 75.),
        8 => hsl(0., 0., 25.),
        9 => hsl(0., 100., 75.),
        10 => hsl(120., 100., 75.),
        11 => hsl(60., 100., 75.),
        12 => hsl(220., 100., 75.),
        13 => hsl(300., 100., 75.),
        14 => hsl(180., 100., 75.),
        15 => hsl(0., 0., 100.),
        n @ 16..=231 => {
            let idx = n - 16;
            let r = idx / 36;
            let g = (idx / 6) % 6;
            let b = idx % 6;
            rgb_to_hsla(
                if r == 0 { 0 } else { 55 + r * 40 },
                if g == 0 { 0 } else { 55 + g * 40 },
                if b == 0 { 0 } else { 55 + b * 40 },
            )
        }
        n @ 232..=255 => {
            let v = 8 + (n - 232) * 10;
            hsl(0., 0., v as f32)
        }
    }
}
