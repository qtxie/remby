---
feature: theme-system
status: delivered
specs:
  - docs/compose/specs/2026-06-23-theme-system-design.md
plans:
  - docs/compose/plans/2026-06-23-theme-system.md
---

# Theme System — Final Report

## What Was Built

A named color theme system for the Remby TUI client. Users can switch between three built-in color palettes ("default", "green", "dracula") from the Settings view. The theme applies live as the user cycles through options, and persists to the config file on save.

All hardcoded `Color::*` values across the 1600-line `ui.rs` were replaced with semantic color lookups from a `Theme` struct. The 7 semantic roles (accent, text, muted, warning, success, error, selection_fg) cover every color usage in the app.

## Architecture

**`src/theme.rs`** — Defines `Theme` struct with 7 `Color` fields and three built-in themes. `THEME_NAMES` constant lists available themes. `Theme::by_name()` resolves a theme string to a `Theme` instance, falling back to default.

**`src/config.rs`** — `RembyConfig` gains a `theme: String` field (default: `"default"`). Serde handles backward compatibility via `#[serde(default)]`.

**`src/app.rs`** — `AppState` holds the active `Theme`. `SettingsState` holds the theme name during editing. `SettingsSection` enum gains a `Theme` variant. `settings_cycle_theme()` cycles through themes and applies immediately. `settings_save()` persists the theme name to config.

**`src/ui.rs`** — Every render function accepts `theme: &Theme`. All hardcoded colors replaced with `theme.accent`, `theme.text`, `theme.muted`, `theme.warning`, `theme.success`, `theme.error`, `theme.selection_fg`.

**`src/main.rs`** — Key handling adds `in_theme` guard and Left/Right handler for theme cycling.

### Design Decisions

- **Semantic roles over per-element colors**: 7 roles keep the API small while covering all usage. Adding a new theme is a 10-line function.
- **Theme on AppState, not passed as parameter**: Avoids threading a parameter through every function call — render functions access it via `state.theme`.
- **Live preview**: Theme changes apply immediately in Settings without saving, so users can see the effect before committing.

## Usage

1. Open Settings (from the app)
2. Press `Tab` to cycle to the "Theme" section
3. Press `Left`/`Right` (or `h`/`l`) to cycle through: default → green → dracula
4. Press `Enter` to save

Config file (`config.json`):
```json
{
  "theme": "dracula"
}
```

## Verification

- `cargo check` — zero warnings
- `cargo build` — clean build
- All 1600+ lines of `ui.rs` verified to have no remaining hardcoded `Color::*` references
- `main.rs` has one `Color::Yellow` in the splash screen (pre-AppState) — intentional

## Journey Log

- [lesson] Replacing colors in a 1600-line UI file requires systematic function-by-function editing; batch replacements with `replaceAll` are risky when the same color has different semantic meanings in different contexts.
