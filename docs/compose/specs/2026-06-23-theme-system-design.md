# Theme System Design

> [!NOTE]
> This document may not reflect the current implementation.
> See the final report for up-to-date state:
> [Final Report](../reports/theme-system.md)

## [S1] Semantic color roles

7 roles extracted from current hardcoded colors:

| Role | Default | Used for |
|------|---------|----------|
| `accent` | Cyan | borders, highlights, selected items, active fields, headers |
| `text` | White | title text, content, labels |
| `muted` | DarkGray | footer, hints, inactive borders, durations, mpv info |
| `warning` | Yellow | favorites star, "add new", wizard welcome, separators |
| `success` | Green | success messages, active indicators, play button |
| `error` | Red | errors, delete confirm, mpv fatal output |
| `selection_fg` | Black | text when item is highlighted in popups (sort/filter panels) |

## [S2] Architecture

- `src/theme.rs`: `Theme` struct with 7 `Color` fields, named built-in themes, lookup by name
- `Theme` stored on `AppState`, threaded to all render functions via `&Theme`
- `config.rs`: add `theme: String` to `RembyConfig` (default: `"default"`)
- `ui.rs`: replace all hardcoded `Color::*` with `theme.*` field access

Built-in themes:
- `"default"` — current cyan accent (unchanged appearance)
- `"green"` — green accent
- `"dracula"` — purple/magenta accent with warm tones

## [S3] Theme selection UX

Settings view gains a "Theme" section below Language. Left/Right cycles through `["default", "green", "dracula"]`. Theme applies immediately for live preview. Name saved to config on settings save.
