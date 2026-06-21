# Library Browser Implementation Plan

> [!NOTE]
> This document may not reflect the current implementation.
> See the final report for up-to-date state:
> [Final Report](../reports/library-browser.md)

> **For agentic workers:** REQUIRED SUB-SKILL: Use compose:subagent (recommended) or compose:execute to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a dedicated library browser view with sort and filter capabilities for any library type.

**Architecture:** New `LibraryBrowserState` in app.rs tracks sort/filter state. Emby API supports `SortBy`, `SortOrder`, `Genres`, `Years` parameters on `/Users/{userId}/Items`. UI uses popup panels for sort/filter selection.

**Tech Stack:** Rust, ratatui, tokio, reqwest, serde

---

### Task 1: Add LibraryBrowserState and View variant

**Covers:** [S1]

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Add new types and state struct**

Add after `SettingsState` definition (around line 125):

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum ItemSort {
    Name,
    Year,
    Rating,
    DateAdded,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BrowserPanel {
    None,
    Sort,
    Filter,
}

pub struct LibraryBrowserState {
    pub library_id: String,
    pub library_name: String,
    pub items: Vec<MediaItem>,
    pub total: usize,
    pub sort_by: ItemSort,
    pub sort_order: SortOrder,
    pub filter_genre: Option<String>,
    pub filter_years: Option<(u32, u32)>,
    pub available_genres: Vec<String>,
    pub panel: BrowserPanel,
    pub panel_selected: usize,
    pub filter_year_input: String,
    pub filter_year_field: Option<YearField>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum YearField {
    Start,
    End,
}

impl Default for LibraryBrowserState {
    fn default() -> Self {
        Self {
            library_id: String::new(),
            library_name: String::new(),
            items: Vec::new(),
            total: 0,
            sort_by: ItemSort::DateAdded,
            sort_order: SortOrder::Desc,
            filter_genre: None,
            filter_years: None,
            available_genres: Vec::new(),
            panel: BrowserPanel::None,
            panel_selected: 0,
            filter_year_input: String::new(),
            filter_year_field: None,
        }
    }
}
```

- [ ] **Step 2: Add View variant**

In the `View` enum, add `LibraryBrowser` variant:

```rust
#[derive(PartialEq, Clone, Debug)]
pub enum View {
    Home,
    Libraries,
    Items,
    SearchResults,
    SourceSelect,
    TrackSelect,
    Episodes,
    SeriesInfo,
    Playing,
    Settings,
    LibraryBrowser,  // Add this
}
```

- [ ] **Step 3: Add state field to AppState**

In `AppState` struct, add:

```rust
pub library_browser_state: LibraryBrowserState,
```

In `AppState::new()`, add:

```rust
library_browser_state: LibraryBrowserState::default(),
```

- [ ] **Step 4: Add helper methods to AppState**

Add these methods to `impl AppState`:

```rust
pub fn open_library_browser(&mut self, library_id: String, library_name: String) {
    self.library_browser_state = LibraryBrowserState {
        library_id,
        library_name,
        ..Default::default()
    };
    self.navigate_to(View::LibraryBrowser);
}

pub fn library_browser_sort_label(&self) -> &str {
    match self.library_browser_state.sort_by {
        ItemSort::Name => "Name",
        ItemSort::Year => "Year",
        ItemSort::Rating => "Rating",
        ItemSort::DateAdded => "Date Added",
    }
}

pub fn library_browser_order_label(&self) -> &str {
    match self.library_browser_state.sort_order {
        SortOrder::Asc => "↑",
        SortOrder::Desc => "↓",
    }
}

pub fn library_browser_cycle_sort(&mut self) {
    let bs = &mut self.library_browser_state;
    bs.sort_by = match bs.sort_by {
        ItemSort::Name => ItemSort::Year,
        ItemSort::Year => ItemSort::Rating,
        ItemSort::Rating => ItemSort::DateAdded,
        ItemSort::DateAdded => ItemSort::Name,
    };
}

pub fn library_browser_toggle_order(&mut self) {
    let bs = &mut self.library_browser_state;
    bs.sort_order = match bs.sort_order {
        SortOrder::Asc => SortOrder::Desc,
        SortOrder::Desc => SortOrder::Asc,
    };
}

pub fn library_browser_open_sort_panel(&mut self) {
    self.library_browser_state.panel = BrowserPanel::Sort;
    self.library_browser_state.panel_selected = match self.library_browser_state.sort_by {
        ItemSort::Name => 0,
        ItemSort::Year => 1,
        ItemSort::Rating => 2,
        ItemSort::DateAdded => 3,
    };
}

pub fn library_browser_open_filter_panel(&mut self) {
    self.library_browser_state.panel = BrowserPanel::Filter;
    self.library_browser_state.panel_selected = 0;
    self.library_browser_state.filter_year_field = None;
}

pub fn library_browser_close_panel(&mut self) {
    self.library_browser_state.panel = BrowserPanel::None;
    self.library_browser_state.filter_year_field = None;
    self.library_browser_state.filter_year_input.clear();
}

pub fn library_browser_select_sort(&mut self) {
    let bs = &mut self.library_browser_state;
    bs.sort_by = match bs.panel_selected {
        0 => ItemSort::Name,
        1 => ItemSort::Year,
        2 => ItemSort::Rating,
        3 => ItemSort::DateAdded,
        _ => ItemSort::Name,
    };
    bs.panel = BrowserPanel::None;
}

pub fn library_browser_toggle_genre(&mut self) {
    let bs = &mut self.library_browser_state;
    if let Some(genre) = bs.available_genres.get(bs.panel_selected).cloned() {
        if bs.filter_genre.as_ref() == Some(&genre) {
            bs.filter_genre = None;
        } else {
            bs.filter_genre = Some(genre);
        }
    }
}

pub fn library_browser_panel_next(&mut self) {
    let bs = &mut self.library_browser_state;
    let len = match bs.panel {
        BrowserPanel::Sort => 4,
        BrowserPanel::Filter => {
            if bs.filter_year_field.is_some() {
                2 // Start/End year fields
            } else {
                bs.available_genres.len() + 1 // genres + "Year range" option
            }
        }
        BrowserPanel::None => 0,
    };
    if len > 0 {
        bs.panel_selected = (bs.panel_selected + 1) % len;
    }
}

pub fn library_browser_panel_prev(&mut self) {
    let bs = &mut self.library_browser_state;
    let len = match bs.panel {
        BrowserPanel::Sort => 4,
        BrowserPanel::Filter => {
            if bs.filter_year_field.is_some() {
                2
            } else {
                bs.available_genres.len() + 1
            }
        }
        BrowserPanel::None => 0,
    };
    if len > 0 {
        bs.panel_selected = (bs.panel_selected + len - 1) % len;
    }
}

pub fn library_browser_filter_select(&mut self) {
    let bs = &mut self.library_browser_state;
    let genre_count = bs.available_genres.len();
    
    if bs.panel_selected < genre_count {
        // Toggle genre
        self.library_browser_toggle_genre();
    } else {
        // Enter year range mode
        bs.filter_year_field = Some(YearField::Start);
        bs.filter_year_input = bs.filter_years
            .map(|(s, _)| s.to_string())
            .unwrap_or_default();
    }
}

pub fn library_browser_year_input(&mut self, c: char) {
    let bs = &mut self.library_browser_state;
    if bs.filter_year_field.is_some() {
        bs.filter_year_input.push(c);
    }
}

pub fn library_browser_year_backspace(&mut self) {
    let bs = &mut self.library_browser_state;
    if bs.filter_year_field.is_some() {
        bs.filter_year_input.pop();
    }
}

pub fn library_browser_year_confirm(&mut self) {
    let bs = &mut self.library_browser_state;
    let year: u32 = bs.filter_year_input.parse().unwrap_or(0);
    
    match bs.filter_year_field {
        Some(YearField::Start) => {
            let end = bs.filter_years.map(|(_, e)| e).unwrap_or(year);
            bs.filter_years = Some((year, end));
            bs.filter_year_field = Some(YearField::End);
            bs.filter_year_input = end.to_string();
        }
        Some(YearField::End) => {
            if let Some((start, _)) = bs.filter_years {
                bs.filter_years = Some((start, year));
            }
            bs.filter_year_field = None;
            bs.filter_year_input.clear();
            bs.panel = BrowserPanel::None;
        }
        None => {}
    }
}

pub fn library_browser_clear_filters(&mut self) {
    let bs = &mut self.library_browser_state;
    bs.filter_genre = None;
    bs.filter_years = None;
}
```

- [ ] **Step 5: Update current_list_len**

In `current_list_len()`, add match arm:

```rust
View::LibraryBrowser => self.library_browser_state.items.len(),
```

- [ ] **Step 6: Update selected_item**

In `selected_item()`, add match arm:

```rust
View::LibraryBrowser => self.library_browser_state.items.get(self.selected),
```

- [ ] **Step 7: Build and verify**

Run: `cargo build`
Expected: Compiles successfully

---

### Task 2: Add Emby API methods for genres and filtered items

**Covers:** [S1]

**Files:**
- Modify: `src/emby.rs`

- [ ] **Step 1: Add get_genres method**

Add to `impl EmbyClient`:

```rust
pub async fn get_genres(&self, parent_id: &str) -> Result<Vec<String>> {
    let url = self.api_url(&format!("/Genres"));
    let resp = self.authed_get(&url)
        .query(&[
            ("UserId", self.user_id.as_str()),
            ("ParentId", parent_id),
        ])
        .send()
        .await
        .context("Failed to fetch genres")?;

    let data: serde_json::Value = resp.json().await.context("Invalid genres response")?;
    let items = data.get("Items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    
    let genres: Vec<String> = items.iter()
        .filter_map(|item| item.get("Name").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect();
    Ok(genres)
}
```

- [ ] **Step 2: Add get_items_filtered method**

Add to `impl EmbyClient`:

```rust
pub async fn get_items_filtered(
    &self,
    parent_id: &str,
    start: usize,
    limit: usize,
    sort_by: &str,
    sort_order: &str,
    genres: Option<&str>,
    years: Option<&str>,
) -> Result<PageResult> {
    let url = self.api_url(&format!("/Users/{}/Items", self.user_id));
    let mut query = vec![
        ("ParentId", parent_id.to_string()),
        ("Recursive", "true".to_string()),
        ("Fields", "Overview,MediaSources,ChildCount".to_string()),
        ("StartIndex", start.to_string()),
        ("Limit", limit.to_string()),
        ("SortBy", sort_by.to_string()),
        ("SortOrder", sort_order.to_string()),
    ];
    
    if let Some(g) = genres {
        query.push(("Genres", g.to_string()));
    }
    if let Some(y) = years {
        query.push(("Years", y.to_string()));
    }
    
    let resp = self.authed_get(&url)
        .query(&query)
        .send()
        .await
        .context("Failed to fetch filtered items")?;

    let data: ItemsResponse = resp.json().await.context("Invalid filtered items response")?;
    Ok(PageResult {
        items: data.items,
        total: data.total,
    })
}
```

- [ ] **Step 3: Build and verify**

Run: `cargo build`
Expected: Compiles successfully

---

### Task 3: Update main.rs to open LibraryBrowser

**Covers:** [S1]

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Add BackgroundResult variants**

Add to `BackgroundResult` enum:

```rust
LibraryBrowserLoaded(Vec<MediaItem>, String, usize, Vec<String>), // items, lib_id, total, genres
MoreLibraryBrowserLoaded(Vec<MediaItem>, String), // more items, lib_id
```

- [ ] **Step 2: Update library selection handler**

Find the `selected_library()` block (around line 676) and change to open LibraryBrowser:

```rust
} else if let Some(lib) = state.selected_library().cloned() {
    state.open_library_browser(lib.id.clone(), lib.name.clone());
    state.loading = true;
    let tx = bg_tx.clone();
    let client = state.client.clone();
    let library_id = lib.id.clone();
    let sort_by = "DateCreated".to_string();
    let sort_order = "Descending".to_string();
    tokio::spawn(async move {
        let timeout = std::time::Duration::from_secs(120);
        let result = tokio::time::timeout(timeout, async {
            let items_result = client.get_items_filtered(&library_id, 0, 50, &sort_by, &sort_order, None, None).await;
            let genres_result = client.get_genres(&library_id).await;
            let items = items_result.unwrap_or_else(|_| PageResult { items: vec![], total: 0 });
            let genres = genres_result.unwrap_or_default();
            (items.items, library_id, items.total, genres)
        }).await;
        match result {
            Ok((items, lib_id, total, genres)) => {
                let _ = tx.send(BackgroundResult::LibraryBrowserLoaded(items, lib_id, total, genres));
            }
            Err(_) => { let _ = tx.send(BackgroundResult::Timeout("Library".to_string())); }
        }
    });
}
```

- [ ] **Step 3: Add BackgroundResult handler**

Add handler in the result processing loop:

```rust
BackgroundResult::LibraryBrowserLoaded(items, lib_id, total, genres) => {
    if state.library_browser_state.library_id == lib_id {
        state.library_browser_state.items = items;
        state.library_browser_state.total = total;
        state.library_browser_state.available_genres = genres;
    }
    state.loading = false;
    state.status_msg = format!("{} / {} items", state.library_browser_state.items.len(), total);
}
BackgroundResult::MoreLibraryBrowserLoaded(more_items, lib_id) => {
    if state.library_browser_state.library_id == lib_id {
        state.library_browser_state.items.extend(more_items);
        let total = state.library_browser_state.total;
        state.status_msg = format!("{} / {} items", state.library_browser_state.items.len(), total);
    }
    state.loading = false;
}
```

- [ ] **Step 4: Build and verify**

Run: `cargo build`
Expected: Compiles successfully

---

### Task 4: Add UI rendering for LibraryBrowser

**Covers:** [S1]

**Files:**
- Modify: `src/ui.rs`

- [ ] **Step 1: Add LibraryBrowser to render match**

In `render()`, add to the match:

```rust
View::LibraryBrowser => render_library_browser(f, state, layout[1]),
```

- [ ] **Step 2: Add header title**

In `render_header()`, add match arm:

```rust
View::LibraryBrowser => {
    let bs = &state.library_browser_state;
    let count = if bs.total > 0 {
        format!("{} / {}", bs.items.len(), bs.total)
    } else {
        bs.items.len().to_string()
    };
    let genre = bs.filter_genre.as_deref().unwrap_or("All");
    let years = bs.filter_years
        .map(|(s, e)| format!("{}-{}", s, e))
        .unwrap_or_else(|| "All".to_string());
    format!(
        "{} | Sort: {}{} | Genre: {} | Years: {} [{}]",
        bs.library_name,
        state.library_browser_sort_label(),
        state.library_browser_order_label(),
        genre,
        years,
        count
    )
}
```

- [ ] **Step 3: Add render_library_browser function**

Add to `src/ui.rs`:

```rust
fn render_library_browser(f: &mut Frame, state: &AppState, area: Rect) {
    let bs = &state.library_browser_state;
    
    // Render items list
    let items: Vec<ListItem> = bs.items.iter().map(|item| {
        let name = item.display_name();
        let duration = item.duration_str().map(|d| format!(" ({})", d)).unwrap_or_default();
        ListItem::new(Line::from(Span::raw(format!("{}{}", name, duration))))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", bs.library_name)))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .highlight_symbol("▸ ");

    let mut state_list = ListState::default();
    state_list.select(Some(state.selected));
    f.render_stateful_widget(list, area, &mut state_list);

    // Render popup panel if active
    match bs.panel {
        BrowserPanel::Sort => render_sort_panel(f, state, area),
        BrowserPanel::Filter => render_filter_panel(f, state, area),
        BrowserPanel::None => {}
    }
}

fn render_sort_panel(f: &mut Frame, state: &AppState, area: Rect) {
    let bs = &state.library_browser_state;
    let options = ["Name", "Year", "Rating", "Date Added"];
    
    let items: Vec<ListItem> = options.iter().enumerate().map(|(i, opt)| {
        let selected = i == bs.panel_selected;
        let current = match bs.sort_by {
            ItemSort::Name => 0,
            ItemSort::Year => 1,
            ItemSort::Rating => 2,
            ItemSort::DateAdded => 3,
        } == i;
        
        let style = if selected {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else if current {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        
        let marker = if current { "● " } else { "  " };
        ListItem::new(Line::from(Span::styled(format!("{}{}", marker, opt), style)))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Sort By "));

    let popup = centered_rect(30, 12, area);
    f.render_widget(Clear, popup);
    f.render_widget(list, popup);
}

fn render_filter_panel(f: &mut Frame, state: &AppState, area: Rect) {
    let bs = &state.library_browser_state;
    
    let mut items: Vec<ListItem> = Vec::new();
    
    // Genre options
    for (i, genre) in bs.available_genres.iter().enumerate() {
        let selected = i == bs.panel_selected;
        let active = bs.filter_genre.as_ref() == Some(genre);
        
        let style = if selected {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else if active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        
        let marker = if active { "● " } else { "  " };
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{}{}", marker, genre),
            style,
        ))));
    }
    
    // Year range option
    let year_idx = bs.available_genres.len();
    let year_selected = bs.panel_selected == year_idx;
    let year_active = bs.filter_years.is_some() || bs.filter_year_field.is_some();
    let year_style = if year_selected {
        Style::default().fg(Color::Black).bg(Color::Cyan)
    } else if year_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    
    let year_text = if let Some((s, e)) = bs.filter_years {
        format!("  Years: {}-{}", s, e)
    } else if bs.filter_year_field.is_some() {
        format!("  Years: {}_", bs.filter_year_input)
    } else {
        "  Year range".to_string()
    };
    items.push(ListItem::new(Line::from(Span::styled(year_text, year_style))));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Filter "));

    let height = (bs.available_genres.len() + 3).min(20) as u16;
    let popup = centered_rect(40, height, area);
    f.render_widget(Clear, popup);
    f.render_widget(list, popup);
}

fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(height),
            Constraint::Min(1),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

- [ ] **Step 4: Build and verify**

Run: `cargo build`
Expected: Compiles successfully

---

### Task 5: Add key handlers for LibraryBrowser

**Covers:** [S1]

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Add LibraryBrowser key handler block**

In the `match state.view` block, add before the `_ =>` catch-all:

```rust
app::View::LibraryBrowser => {
    let has_panel = state.library_browser_state.panel != app::BrowserPanel::None;
    
    match key.code {
        KeyCode::Char('q') => break,
        KeyCode::Esc => {
            if has_panel {
                state.library_browser_close_panel();
            } else {
                state.go_back();
            }
        }
        KeyCode::Char('s') if !has_panel => {
            state.library_browser_open_sort_panel();
        }
        KeyCode::Char('f') if !has_panel => {
            state.library_browser_open_filter_panel();
        }
        KeyCode::Char('c') if !has_panel => {
            state.library_browser_clear_filters();
            // Reload with cleared filters
            state.loading = true;
            let tx = bg_tx.clone();
            let client = state.client.clone();
            let lib_id = state.library_browser_state.library_id.clone();
            let sort_by = match state.library_browser_state.sort_by {
                app::ItemSort::Name => "SortName",
                app::ItemSort::Year => "ProductionYear",
                app::ItemSort::Rating => "CommunityRating",
                app::ItemSort::DateAdded => "DateCreated",
            }.to_string();
            let sort_order = match state.library_browser_state.sort_order {
                app::SortOrder::Asc => "Ascending",
                app::SortOrder::Desc => "Descending",
            }.to_string();
            tokio::spawn(async move {
                if let Ok(result) = client.get_items_filtered(&lib_id, 0, 50, &sort_by, &sort_order, None, None).await {
                    let _ = tx.send(BackgroundResult::LibraryBrowserLoaded(result.items, lib_id, result.total, vec![]));
                }
            });
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if has_panel {
                state.library_browser_panel_prev();
            } else {
                state.select_prev();
                // Lazy loading
                let bs = &state.library_browser_state;
                if !state.loading && bs.total > bs.items.len() && state.selected + 5 >= bs.items.len() * 2 / 3 {
                    state.loading = true;
                    let tx = bg_tx.clone();
                    let client = state.client.clone();
                    let lib_id = bs.library_id.clone();
                    let sort_by = match bs.sort_by {
                        app::ItemSort::Name => "SortName",
                        app::ItemSort::Year => "ProductionYear",
                        app::ItemSort::Rating => "CommunityRating",
                        app::ItemSort::DateAdded => "DateCreated",
                    }.to_string();
                    let sort_order = match bs.sort_order {
                        app::SortOrder::Asc => "Ascending",
                        app::SortOrder::Desc => "Descending",
                    }.to_string();
                    let genre = bs.filter_genre.clone();
                    let years = bs.filter_years.map(|(s, e)| format!("{}-{}", s, e));
                    let start = bs.items.len();
                    tokio::spawn(async move {
                        if let Ok(result) = client.get_items_filtered(&lib_id, start, 50, &sort_by, &sort_order, genre.as_deref(), years.as_deref()).await {
                            let _ = tx.send(BackgroundResult::MoreLibraryBrowserLoaded(result.items, lib_id));
                        }
                    });
                }
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if has_panel {
                state.library_browser_panel_next();
            } else {
                state.select_next();
                // Lazy loading
                let bs = &state.library_browser_state;
                if !state.loading && bs.total > bs.items.len() && state.selected + 5 >= bs.items.len() * 2 / 3 {
                    state.loading = true;
                    let tx = bg_tx.clone();
                    let client = state.client.clone();
                    let lib_id = bs.library_id.clone();
                    let sort_by = match bs.sort_by {
                        app::ItemSort::Name => "SortName",
                        app::ItemSort::Year => "ProductionYear",
                        app::ItemSort::Rating => "CommunityRating",
                        app::ItemSort::DateAdded => "DateCreated",
                    }.to_string();
                    let sort_order = match bs.sort_order {
                        app::SortOrder::Asc => "Ascending",
                        app::SortOrder::Desc => "Descending",
                    }.to_string();
                    let genre = bs.filter_genre.clone();
                    let years = bs.filter_years.map(|(s, e)| format!("{}-{}", s, e));
                    let start = bs.items.len();
                    tokio::spawn(async move {
                        if let Ok(result) = client.get_items_filtered(&lib_id, start, 50, &sort_by, &sort_order, genre.as_deref(), years.as_deref()).await {
                            let _ = tx.send(BackgroundResult::MoreLibraryBrowserLoaded(result.items, lib_id));
                        }
                    });
                }
            }
        }
        KeyCode::Enter => {
            if has_panel {
                match state.library_browser_state.panel {
                    app::BrowserPanel::Sort => {
                        state.library_browser_select_sort();
                        // Reload with new sort
                        state.loading = true;
                        let tx = bg_tx.clone();
                        let client = state.client.clone();
                        let lib_id = state.library_browser_state.library_id.clone();
                        let sort_by = match state.library_browser_state.sort_by {
                            app::ItemSort::Name => "SortName",
                            app::ItemSort::Year => "ProductionYear",
                            app::ItemSort::Rating => "CommunityRating",
                            app::ItemSort::DateAdded => "DateCreated",
                        }.to_string();
                        let sort_order = match state.library_browser_state.sort_order {
                            app::SortOrder::Asc => "Ascending",
                            app::SortOrder::Desc => "Descending",
                        }.to_string();
                        let genre = state.library_browser_state.filter_genre.clone();
                        let years = state.library_browser_state.filter_years.map(|(s, e)| format!("{}-{}", s, e));
                        tokio::spawn(async move {
                            if let Ok(result) = client.get_items_filtered(&lib_id, 0, 50, &sort_by, &sort_order, genre.as_deref(), years.as_deref()).await {
                                let _ = tx.send(BackgroundResult::LibraryBrowserLoaded(result.items, lib_id, result.total, vec![]));
                            }
                        });
                    }
                    app::BrowserPanel::Filter => {
                        if state.library_browser_state.filter_year_field.is_some() {
                            state.library_browser_year_confirm();
                        } else {
                            state.library_browser_filter_select();
                        }
                        // If panel closed, reload with filters
                        if state.library_browser_state.panel == app::BrowserPanel::None {
                            state.loading = true;
                            let tx = bg_tx.clone();
                            let client = state.client.clone();
                            let lib_id = state.library_browser_state.library_id.clone();
                            let sort_by = match state.library_browser_state.sort_by {
                                app::ItemSort::Name => "SortName",
                                app::ItemSort::Year => "ProductionYear",
                                app::ItemSort::Rating => "CommunityRating",
                                app::ItemSort::DateAdded => "DateCreated",
                            }.to_string();
                            let sort_order = match state.library_browser_state.sort_order {
                                app::SortOrder::Asc => "Ascending",
                                app::SortOrder::Desc => "Descending",
                            }.to_string();
                            let genre = state.library_browser_state.filter_genre.clone();
                            let years = state.library_browser_state.filter_years.map(|(s, e)| format!("{}-{}", s, e));
                            tokio::spawn(async move {
                                if let Ok(result) = client.get_items_filtered(&lib_id, 0, 50, &sort_by, &sort_order, genre.as_deref(), years.as_deref()).await {
                                    let _ = tx.send(BackgroundResult::LibraryBrowserLoaded(result.items, lib_id, result.total, vec![]));
                                }
                            });
                        }
                    }
                    app::BrowserPanel::None => {}
                }
            } else {
                // Open selected item
                if let Some(item) = state.selected_item().cloned() {
                    if item.is_video() {
                        state.loading = true;
                        let tx = bg_tx.clone();
                        let client = state.client.clone();
                        let item_id = item.id.clone();
                        tokio::spawn(async move {
                            let timeout = std::time::Duration::from_secs(60);
                            match tokio::time::timeout(timeout, client.get_item_detail(&item_id)).await {
                                Ok(Ok(detail)) => { let _ = tx.send(BackgroundResult::ItemDetailLoaded(detail)); }
                                _ => { let _ = tx.send(BackgroundResult::Timeout("Item detail".to_string())); }
                            }
                        });
                    } else if item.is_navigable() {
                        state.loading = true;
                        let tx = bg_tx.clone();
                        let client = state.client.clone();
                        let item_id = item.id.clone();
                        let item_type = item.item_type.clone();
                        let series_id = item.series_id.clone();
                        tokio::spawn(async move {
                            if item_type == "Series" {
                                let series_id = series_id.unwrap_or(item_id);
                                let mut series_item = crate::emby::MediaItem::separator("");
                                series_item.id = series_id;
                                let result = build_series_state(&client, &series_item).await;
                                let _ = tx.send(BackgroundResult::SeriesInfoLoaded(result));
                            } else {
                                if let Ok(result) = client.get_items(&item_id, 0, 200).await {
                                    let _ = tx.send(BackgroundResult::FolderLoaded(result.items, item_id, result.total));
                                }
                            }
                        });
                    }
                }
            }
        }
        KeyCode::Char(c) => {
            if state.library_browser_state.filter_year_field.is_some() {
                state.library_browser_year_input(c);
            }
        }
        KeyCode::Backspace => {
            if state.library_browser_state.filter_year_field.is_some() {
                state.library_browser_year_backspace();
            }
        }
        _ => {}
    }
}
```

- [ ] **Step 2: Build and verify**

Run: `cargo build`
Expected: Compiles successfully

---

### Task 6: Final integration and cleanup

**Files:**
- Modify: `src/main.rs`
- Modify: `src/ui.rs`

- [ ] **Step 1: Add footer hints for LibraryBrowser**

In `render_footer()`, add match arm for LibraryBrowser:

```rust
View::LibraryBrowser => {
    let hints = if state.library_browser_state.panel != BrowserPanel::None {
        "j/k: Navigate | Enter: Select | Esc: Close"
    } else {
        "j/k: Navigate | Enter: Open | s: Sort | f: Filter | c: Clear filters | Esc: Back"
    };
    Line::from(Span::raw(hints))
}
```

- [ ] **Step 2: Build and run final test**

Run: `cargo build`
Expected: Compiles with no warnings

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: add LibraryBrowser view with sort and filter panels"
```
