---
feature: library-browser
status: delivered
specs: []
plans:
  - docs/compose/plans/2026-06-21-library-browser.md
branch: main
commits: 0ef0298
---

# Library Browser — Final Report

## What Was Built

A dedicated library browser view for browsing Emby libraries with full sort and filter capabilities. When a user opens any library (电影, 电视剧, etc.) from the Libraries page, they now see a new `LibraryBrowser` view that supports:

- **Sorting** by Name, Year, Rating, or Date Added (ascending/descending)
- **Filtering** by Genre (toggle selection) and Year range (start-end input)
- **Popup panels** for intuitive sort/filter selection
- **Lazy loading** for large collections (loads more as user scrolls)

## Architecture

### Components

- **`LibraryBrowserState`** (`src/app.rs`): State struct tracking sort/filter state, items, genres, and panel status
- **`View::LibraryBrowser`**: New view variant for the library browser
- **`EmbyClient::get_genres()`** (`src/emby.rs`): Fetches available genres for a library
- **`EmbyClient::get_items_filtered()`** (`src/emby.rs`): Fetches items with sort/filter parameters
- **`render_library_browser()`** (`src/ui.rs`): Renders the items list with popup panels

### Data Flow

1. User selects library → `open_library_browser()` creates state and navigates
2. Background task fetches items + genres via Emby API
3. `LibraryBrowserLoaded` result populates state
4. User interacts with sort/filter panels → state updates → API refetch triggered
5. Lazy loading triggers when user scrolls past 2/3 threshold

## Usage

### Hotkeys (in LibraryBrowser view)

| Key | Action |
|-----|--------|
| `j/k` or arrows | Navigate list / panel options |
| `Enter` | Open item / confirm selection |
| `s` | Open sort panel |
| `f` | Open filter panel |
| `c` | Clear all filters |
| `Esc` | Close panel / go back |
| `q` | Quit |

### Sort Panel

Shows 4 options: Name, Year, Rating, Date Added. Current sort is marked with `●`. Select to apply.

### Filter Panel

Shows available genres (toggle with Enter) and Year range option. Year range input: enter start year, press Enter, enter end year, press Enter to apply.

## Verification

- `cargo build` succeeds with exit code 0
- Only 2 minor warnings about unused helper methods (`library_browser_cycle_sort`, `library_browser_toggle_order`) that are available for future use
- All new types, methods, and UI rendering code compiles correctly

## Journey Log

- [lesson] Emby API `Genres` endpoint requires `UserId` and `ParentId` parameters
- [pivot] Generalized from movie-only to all library types per user feedback
