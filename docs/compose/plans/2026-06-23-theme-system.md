# Theme System Implementation Plan

> [!NOTE]
> This document may not reflect the current implementation.
> See the final report for up-to-date state:
> [Final Report](../reports/theme-system.md)

> **For agentic workers:** REQUIRED SUB-SKILL: Use compose:subagent (recommended) or compose:execute to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a named color theme system so users can switch between color palettes (default/green/dracula) from Settings.

**Architecture:** Define a `Theme` struct with 7 semantic color fields in a new `theme.rs`. Store the active theme on `AppState`. Thread `&Theme` through all render functions in `ui.rs`, replacing hardcoded `Color::*` values. Add a Theme section to Settings for cycling through built-in themes.

**Tech Stack:** Rust, ratatui 0.29, serde

---

## File Structure

| File | Action | Purpose |
|------|--------|---------|
| `src/theme.rs` | Create | `Theme` struct, built-in themes, lookup function |
| `src/main.rs` | Modify | Add `mod theme` |
| `src/config.rs` | Modify | Add `theme: String` to `RembyConfig` |
| `src/app.rs` | Modify | Add `theme: Theme` to `AppState`, `Theme` to `SettingsSection`, theme cycling logic |
| `src/ui.rs` | Modify | Replace all hardcoded colors with `state.theme.*`, add Theme section to Settings |

---

### Task 1: Create `src/theme.rs`

**Covers:** S1, S2

**Files:**
- Create: `src/theme.rs`

- [ ] **Step 1: Create the theme module**

```rust
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

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

pub const THEME_NAMES: &[&str] = &["default", "green", "dracula"];

impl Theme {
    pub fn by_name(name: &str) -> Self {
        match name {
            "green" => Self::green(),
            "dracula" => Self::dracula(),
            _ => Self::default(),
        }
    }

    fn default() -> Self {
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
        Self::default()
    }
}
```

- [ ] **Step 2: Register the module in `src/main.rs`**

Add after line 7 (`mod ui;`):
```rust
mod theme;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check`
Expected: no errors related to `theme`

---

### Task 2: Add theme to config

**Covers:** S2

**Files:**
- Modify: `src/config.rs:8-19`

- [ ] **Step 1: Add `theme` field to `RembyConfig`**

In `src/config.rs`, add a new field and default function:

After line 18 (`pub language: String,`), add:
```rust
    #[serde(default = "default_theme")]
    pub theme: String,
```

After line 27 (`fn default_language() -> String {` block), add:
```rust
fn default_theme() -> String {
    "default".to_string()
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check`
Expected: no errors

---

### Task 3: Add theme to AppState and Settings

**Covers:** S2, S3

**Files:**
- Modify: `src/app.rs:54-101` (AppState struct)
- Modify: `src/app.rs:161-175` (SettingsState, SettingsSection)
- Modify: `src/app.rs:536-544` (AppState::new initialization)
- Modify: `src/app.rs:970-990` (settings_state initialization)
- Modify: `src/app.rs:1037-1063` (settings_save)
- Modify: `src/app.rs:1069-1076` (settings_switch_section)
- Modify: `src/app.rs:1090-1095` (settings_toggle_language — add theme cycling)

- [ ] **Step 1: Add `Theme` variant to `SettingsSection`**

In `src/app.rs`, find:
```rust
#[derive(PartialEq, Clone, Debug)]
pub enum SettingsSection {
    Libraries,
    MpvPath,
    Language,
}
```

Change to:
```rust
#[derive(PartialEq, Clone, Debug)]
pub enum SettingsSection {
    Libraries,
    MpvPath,
    Language,
    Theme,
}
```

- [ ] **Step 2: Add `theme` field to `SettingsState`**

Find:
```rust
pub struct SettingsState {
    pub libraries: Vec<SettingsLibrary>,
    pub selected: usize,
    pub column: SettingsColumn,
    pub section: SettingsSection,
    pub mpv_path: String,
    pub language: String,
}
```

Change to:
```rust
pub struct SettingsState {
    pub libraries: Vec<SettingsLibrary>,
    pub selected: usize,
    pub column: SettingsColumn,
    pub section: SettingsSection,
    pub mpv_path: String,
    pub language: String,
    pub theme: String,
}
```

- [ ] **Step 3: Add `theme` field to `AppState`**

Find in the `AppState` struct (around line 93):
```rust
    pub config: RembyConfig,
```

Add after it:
```rust
    pub theme: crate::theme::Theme,
```

- [ ] **Step 4: Initialize `theme` in `AppState::new`**

Find in the `new` function (around line 537):
```rust
            config,
```

Change to:
```rust
            theme: crate::theme::Theme::by_name(&config.theme),
            config,
```

- [ ] **Step 5: Add `theme` to `SettingsState::default`**

Find:
```rust
impl Default for SettingsState {
    fn default() -> Self {
        Self {
            libraries: Vec::new(),
            selected: 0,
            column: SettingsColumn::Enabled,
            section: SettingsSection::Libraries,
            mpv_path: String::new(),
            language: String::new(),
        }
    }
}
```

Change to:
```rust
impl Default for SettingsState {
    fn default() -> Self {
        Self {
            libraries: Vec::new(),
            selected: 0,
            column: SettingsColumn::Enabled,
            section: SettingsSection::Libraries,
            mpv_path: String::new(),
            language: String::new(),
            theme: String::new(),
        }
    }
}
```

- [ ] **Step 6: Copy theme into settings_state in the settings load**

Find where `settings_state` is initialized (around line 970-990), add `theme` field:
After `language: lang,` add:
```rust
            theme: self.config.theme.clone(),
```

- [ ] **Step 7: Update `settings_switch_section` to cycle through Theme**

Find:
```rust
    pub fn settings_switch_section(&mut self) {
        self.settings_state.section = match self.settings_state.section {
            SettingsSection::Libraries => SettingsSection::MpvPath,
            SettingsSection::MpvPath => SettingsSection::Language,
            SettingsSection::Language => SettingsSection::Libraries,
        };
        self.settings_state.selected = 0;
    }
```

Change to:
```rust
    pub fn settings_switch_section(&mut self) {
        self.settings_state.section = match self.settings_state.section {
            SettingsSection::Libraries => SettingsSection::MpvPath,
            SettingsSection::MpvPath => SettingsSection::Language,
            SettingsSection::Language => SettingsSection::Theme,
            SettingsSection::Theme => SettingsSection::Libraries,
        };
        self.settings_state.selected = 0;
    }
```

- [ ] **Step 8: Add theme cycling method**

After `settings_toggle_language` (around line 1090), add:
```rust
    pub fn settings_cycle_theme(&mut self, forward: bool) {
        if self.settings_state.section == SettingsSection::Theme {
            let names = crate::theme::THEME_NAMES;
            let idx = names.iter().position(|n| *n == self.settings_state.theme).unwrap_or(0);
            let new_idx = if forward {
                (idx + 1) % names.len()
            } else {
                (idx + names.len() - 1) % names.len()
            };
            self.settings_state.theme = names[new_idx].to_string();
            self.theme = crate::theme::Theme::by_name(&self.settings_state.theme);
        }
    }
```

- [ ] **Step 9: Save theme in `settings_save`**

Find in `settings_save` (around line 1050):
```rust
        self.config.language = self.settings_state.language.clone();
```

Add after it:
```rust
        self.config.theme = self.settings_state.theme.clone();
```

- [ ] **Step 10: Verify it compiles**

Run: `cargo check`
Expected: no errors

---

### Task 4: Replace hardcoded colors in `ui.rs` with theme

**Covers:** S1, S2

**Files:**
- Modify: `src/ui.rs` (all render functions)

- [ ] **Step 1: Change `render` signature to pass theme**

Find:
```rust
pub fn render(f: &mut Frame, state: &AppState) {
```

Change to:
```rust
pub fn render(f: &mut Frame, state: &AppState) {
    let theme = &state.theme;
```

Then thread `theme` into every `render_*` call. Each `render_*` function gets a `theme: &crate::theme::Theme` parameter.

Update all calls in `render`:
```rust
    render_header(f, state, layout[0], theme);

    match state.view {
        View::Home => render_home(f, state, layout[1], theme),
        View::Libraries => render_libraries(f, state, layout[1], theme),
        View::Items => render_items(f, state, layout[1], theme),
        View::SearchResults => render_items(f, state, layout[1], theme),
        View::Favorites => render_items(f, state, layout[1], theme),
        View::SourceSelect => render_source_select(f, state, layout[1], theme),
        View::TrackSelect => render_track_select(f, state, layout[1], theme),
        View::Episodes => render_episodes(f, state, layout[1], theme),
        View::SeriesInfo => render_series_info(f, state, layout[1], theme),
        View::Playing => render_playing(f, state, layout[1], theme),
        View::Settings => render_settings(f, state, layout[1], theme),
        View::LibraryBrowser => render_library_browser(f, state, layout[1], theme),
        View::ContinueWatching | View::LatestItems => render_home(f, state, layout[1], theme),
        View::AccountManager => render_account_manager(f, state, layout[1], theme),
        View::Wizard => render_wizard(f, state, layout[1], theme),
        View::MpvPrompt => render_mpv_prompt(f, state, layout[1], theme),
    }

    render_footer(f, state, layout[2], theme);
```

- [ ] **Step 2: Update `render_header`**

Change signature to:
```rust
fn render_header(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Cyan` → `theme.accent` (border_style)
- `Color::White` → `theme.text` (title span, Info status)
- `Color::Cyan` → `theme.accent` (Loading spinner)
- `Color::Green` → `theme.success` (Success status)
- `Color::Red` → `theme.error` (Error status)

- [ ] **Step 3: Update `render_home`**

Change signature to:
```rust
fn render_home(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Cyan` → `theme.accent` (separator style, selected style, highlight_style)
- `Color::Yellow` → `theme.warning` (star)
- `Color::DarkGray` → `theme.muted` (progress bar)

- [ ] **Step 4: Update `render_libraries`**

Change signature to:
```rust
fn render_libraries(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Cyan` → `theme.accent` (header, icons, selected style, selected prefix)
- `Color::Yellow` → `theme.warning` (latest items style)
- `Color::DarkGray` → `theme.muted` (duration)

- [ ] **Step 5: Update `render_items`**

Change signature to:
```rust
fn render_items(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Yellow` → `theme.warning` (star)
- `Color::Green` → `theme.success` (follow mark)
- `Color::DarkGray` → `theme.muted` (duration)
- `Color::Cyan` → `theme.accent` (highlight_style)

- [ ] **Step 6: Update `render_source_select`**

Change signature to:
```rust
fn render_source_select(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Cyan` → `theme.accent` (border, selected style)
- `Color::White` → `theme.text` (title)

- [ ] **Step 7: Update `render_episodes`**

Change signature to:
```rust
fn render_episodes(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::DarkGray` → `theme.muted` (duration)
- `Color::Cyan` → `theme.accent` (highlight_style)

- [ ] **Step 8: Update `render_series_info`**

Change signature to:
```rust
fn render_series_info(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Cyan` / `Color::DarkGray` → `theme.accent` / `theme.muted` (border/title for active/inactive tabs)
- `Color::Cyan` → `theme.accent` (highlight_style)

- [ ] **Step 9: Update `render_media_info`**

Change signature to:
```rust
fn render_media_info(f: &mut Frame, ps: &crate::app::PlayingState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::DarkGray` → `theme.muted` (labels)
- `Color::White` → `theme.text` (values)

Also update the call site in `render_playing`:
```rust
    render_media_info(f, ps, top[2], theme);
```

- [ ] **Step 10: Update `render_playing`**

Change signature to:
```rust
fn render_playing(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::White` → `theme.text` (title)
- `Color::Cyan` → `theme.accent` (playing indicator, resume option selected)
- `Color::Yellow` → `theme.warning` (choose option prompt)
- `Color::Green` → `theme.success` (play button)
- `Color::DarkGray` → `theme.muted` (mpv output info, mpv border)
- `Color::Red` → `theme.error` (mpv error)
- `Color::Yellow` → `theme.warning` (mpv warn)

- [ ] **Step 11: Update `render_settings`**

Change signature to:
```rust
fn render_settings(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Cyan` → `theme.accent` (header_style, selected styles, active borders)
- `Color::DarkGray` → `theme.muted` (inactive headers, inactive borders, hints)
- `Color::White` → `theme.text` (title)
- `Color::Green` → `theme.success` (enabled/active marks)

Add Theme section rendering after the Language section (see Task 5).

- [ ] **Step 12: Update `render_footer`**

Change signature to:
```rust
fn render_footer(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::DarkGray` → `theme.muted`

- [ ] **Step 13: Update `render_track_select`**

Change signature to:
```rust
fn render_track_select(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Cyan` → `theme.accent` (border, title)
- `Color::White` → `theme.text` (title)

Pass `theme` to `render_track_section` calls.

- [ ] **Step 14: Update `render_track_section`**

Change signature to:
```rust
fn render_track_section(
    f: &mut Frame, _state: &AppState, area: Rect,
    title: &str, tracks: &[crate::emby::MediaStream],
    selected: usize, current_section: &TrackSection, section: TrackSection,
    theme: &crate::theme::Theme,
) {
```

Replace colors:
- `Color::Cyan` / `Color::DarkGray` → `theme.accent` / `theme.muted` (border, title, selected style)

- [ ] **Step 15: Update `render_library_browser`**

Change signature to:
```rust
fn render_library_browser(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Cyan` → `theme.accent` (highlight_style)

Pass `theme` to `render_sort_panel` and `render_filter_panel`.

- [ ] **Step 16: Update `render_sort_panel`**

Change signature to:
```rust
fn render_sort_panel(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Black` / `Color::Cyan` → `theme.selection_fg` / `theme.accent` (selected bg)
- `Color::Cyan` → `theme.accent` (current item)

- [ ] **Step 17: Update `render_filter_panel`**

Change signature to:
```rust
fn render_filter_panel(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Black` / `Color::Cyan` → `theme.selection_fg` / `theme.accent` (selected bg)
- `Color::Cyan` → `theme.accent` (active item)
- `Color::DarkGray` → `theme.muted` (section header)

- [ ] **Step 18: Update `render_account_manager`**

Change signature to:
```rust
fn render_account_manager(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Pass `theme` to sub-render functions.

- [ ] **Step 19: Update `render_account_list`**

Change signature to:
```rust
fn render_account_list(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Cyan` → `theme.accent` (selected)
- `Color::Green` → `theme.success` (active account)
- `Color::Yellow` → `theme.warning` (add new)
- `Color::Green` → `theme.success` (status message)

- [ ] **Step 20: Update `render_account_form`**

Change signature to:
```rust
fn render_account_form(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Cyan` → `theme.accent` (active field)
- `Color::DarkGray` → `theme.muted` (hint)

- [ ] **Step 21: Update `render_delete_confirm`**

Change signature to:
```rust
fn render_delete_confirm(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Red` → `theme.error` (delete confirm text)
- `Color::DarkGray` → `theme.muted` (hint)

- [ ] **Step 22: Update `render_wizard`**

Change signature to:
```rust
fn render_wizard(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Yellow` → `theme.warning` (welcome)
- `Color::Cyan` → `theme.accent` (active field)
- `Color::Green` → `theme.success` (language label)
- `Color::DarkGray` → `theme.muted` (hints)
- `Color::Red` → `theme.error` (status error)

- [ ] **Step 23: Update `render_mpv_prompt`**

Change signature to:
```rust
fn render_mpv_prompt(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
```

Replace colors:
- `Color::Yellow` → `theme.warning` (message)
- `Color::Cyan` → `theme.accent` (mpv path label)
- `Color::DarkGray` → `theme.muted` (hint)

- [ ] **Step 24: Verify it compiles**

Run: `cargo check`
Expected: no errors

---

### Task 5: Add Theme section to Settings UI and key handling

**Covers:** S3

**Files:**
- Modify: `src/ui.rs` (render_settings — add Theme section)
- Modify: `src/main.rs` (key handling for Theme section)

- [ ] **Step 1: Add Theme section rendering in `render_settings`**

In `render_settings`, after the Language block rendering (after `f.render_widget(lang_block, layout[2]);`), add a Theme section.

First, update the layout constraints to include a 4th section. Change:
```rust
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);
```

To:
```rust
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);
```

Then add after the lang_block rendering:
```rust
    // Theme section
    let theme_active = ss.section == SettingsSection::Theme;
    let theme_border = if theme_active { theme.accent } else { theme.muted };
    let theme_style = if theme_active {
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let theme_hint = if theme_active { "  (←/→ to switch)" } else { "" };
    let theme_text = format!("  Theme: {}", ss.theme);
    let theme_block = Paragraph::new(Line::from(vec![
        Span::styled(theme_text, theme_style),
        Span::styled(theme_hint.to_string(), Style::default().fg(theme.muted)),
    ]))
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(theme_border)).title(" Theme "));
    f.render_widget(theme_block, layout[3]);
```

- [ ] **Step 2: Add key handling for Theme section in `main.rs`**

Find the settings key handling block (around line 573-587). Add a `in_theme` variable similar to `in_lang`:

Find where `in_lang` is defined (around line 570):
```rust
                                let in_lang = state.settings_state.section == SettingsSection::Language;
```

Add after it:
```rust
                                let in_theme = state.settings_state.section == SettingsSection::Theme;
```

Add theme cycling to the Left/Right handler. Find:
```rust
                                KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') if in_lang => state.settings_toggle_language(),
```

Add after it:
```rust
                                KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') if in_theme => state.settings_cycle_theme(key.code == KeyCode::Right || key.code == KeyCode::Char('l')),
```

Also ensure theme doesn't interfere with other settings sections. Find:
```rust
                                KeyCode::Up | KeyCode::Char('k') if !in_mpv && !in_lang => state.settings_select_prev(),
                                KeyCode::Down | KeyCode::Char('j') if !in_mpv && !in_lang => state.settings_select_next(),
                                KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') if !in_mpv && !in_lang => state.settings_switch_column(),
                                KeyCode::Char(' ') if !in_mpv && !in_lang => state.settings_toggle(),
```

Change to:
```rust
                                KeyCode::Up | KeyCode::Char('k') if !in_mpv && !in_lang && !in_theme => state.settings_select_prev(),
                                KeyCode::Down | KeyCode::Char('j') if !in_mpv && !in_lang && !in_theme => state.settings_select_next(),
                                KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') if !in_mpv && !in_lang && !in_theme => state.settings_switch_column(),
                                KeyCode::Char(' ') if !in_mpv && !in_lang && !in_theme => state.settings_toggle(),
```

- [ ] **Step 3: Verify it compiles and test**

Run: `cargo check`
Expected: no errors

Run: `cargo build`
Expected: successful build

---

### Task 6: Update `main.rs` to use theme for startup wizard colors

**Covers:** S1

**Files:**
- Check if `main.rs` has any hardcoded colors that aren't in `ui.rs`

- [ ] **Step 1: Search for hardcoded colors in main.rs**

Run: `grep -n "Color::" src/main.rs`

If any are found, they should be replaced with `state.theme.*` equivalents.

- [ ] **Step 2: Final compilation check**

Run: `cargo build`
Expected: successful build with no warnings related to unused imports or dead code

- [ ] **Step 3: Manual test**

Run the app, navigate to Settings, Tab to the Theme section, cycle with Left/Right through default → green → dracula. Verify colors change live. Save settings, restart, verify theme persists.
