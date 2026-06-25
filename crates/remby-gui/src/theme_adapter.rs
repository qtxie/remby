use gpui::Hsla;
use gpui_component::{Colorize, Theme, ThemeTokens, hsl};
use ratatui::style::Color;

pub fn apply_remby_theme(cx: &mut gpui::App, theme_name: &str) {
    let custom_themes = remby_core::config::load_themes();
    let rt = remby_core::theme::Theme::by_name(theme_name, &custom_themes);

    let accent = ratatui_to_hsla(rt.accent);
    let text = ratatui_to_hsla(rt.text);
    let muted = ratatui_to_hsla(rt.muted);
    let warning = ratatui_to_hsla(rt.warning);
    let success = ratatui_to_hsla(rt.success);
    let error = ratatui_to_hsla(rt.error);
    let selection_fg = ratatui_to_hsla(rt.selection_fg);

    let theme = Theme::global_mut(cx);
    apply_color_map(theme, accent, text, muted, warning, success, error, selection_fg);
}

fn apply_color_map(
    theme: &mut Theme,
    accent: Hsla,
    text: Hsla,
    muted: Hsla,
    warning: Hsla,
    success: Hsla,
    error: Hsla,
    selection_fg: Hsla,
) {
    let transparent = gpui::transparent_black();

    theme.foreground = text;
    theme.background = hsl(0., 0., 8.);

    theme.primary = accent;
    theme.primary_foreground = selection_fg;
    theme.primary_hover = accent.opacity(0.85);
    theme.primary_active = accent.darken(0.15);
    theme.button_primary = accent.into();
    theme.button_primary_foreground = selection_fg;
    theme.button_primary_hover = accent.opacity(0.85).into();
    theme.button_primary_active = accent.darken(0.15).into();

    theme.accent = accent;
    theme.accent_foreground = selection_fg;

    theme.muted = muted;
    theme.muted_foreground = text.opacity(0.6);

    theme.warning = warning;
    theme.warning_foreground = selection_fg;
    theme.warning_hover = warning.opacity(0.85).into();
    theme.warning_active = warning.darken(0.15).into();
    theme.button_warning = warning.mix_oklab(transparent, 0.2).into();
    theme.button_warning_foreground = warning;
    theme.button_warning_hover = warning.mix_oklab(transparent, 0.3).into();
    theme.button_warning_active = warning.mix_oklab(transparent, 0.4).into();

    theme.success = success;
    theme.success_foreground = selection_fg;
    theme.success_hover = success.opacity(0.85).into();
    theme.success_active = success.darken(0.15).into();
    theme.button_success = success.mix_oklab(transparent, 0.2).into();
    theme.button_success_foreground = success;
    theme.button_success_hover = success.mix_oklab(transparent, 0.3).into();
    theme.button_success_active = success.mix_oklab(transparent, 0.4).into();

    theme.danger = error;
    theme.danger_foreground = selection_fg;
    theme.danger_hover = error.opacity(0.85).into();
    theme.danger_active = error.darken(0.15).into();
    theme.button_danger = error.mix_oklab(transparent, 0.2).into();
    theme.button_danger_foreground = error;
    theme.button_danger_hover = error.mix_oklab(transparent, 0.3).into();
    theme.button_danger_active = error.mix_oklab(transparent, 0.4).into();

    theme.info = accent;
    theme.info_foreground = selection_fg;
    theme.button_info = accent.mix_oklab(transparent, 0.2).into();
    theme.button_info_foreground = accent;
    theme.button_info_hover = accent.mix_oklab(transparent, 0.3).into();
    theme.button_info_active = accent.mix_oklab(transparent, 0.4).into();

    theme.border = muted;
    theme.input = muted;

    theme.link = accent;
    theme.link_active = accent;
    theme.link_hover = accent;

    theme.selection = accent.opacity(0.25).into();

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
