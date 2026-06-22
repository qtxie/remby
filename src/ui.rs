use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{AppState, BrowserPanel, FilterSection, ItemSort, SeriesSection, SettingsColumn, SortOrder, TrackSection, View};

pub fn render(f: &mut Frame, state: &AppState) {
    let area = f.area();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(f, state, layout[0]);

    match state.view {
        View::Home => render_home(f, state, layout[1]),
        View::Libraries => render_libraries(f, state, layout[1]),
        View::Items => render_items(f, state, layout[1]),
        View::SearchResults => render_items(f, state, layout[1]),
        View::Favorites => render_items(f, state, layout[1]),
        View::SourceSelect => render_source_select(f, state, layout[1]),
        View::TrackSelect => render_track_select(f, state, layout[1]),
        View::Episodes => render_episodes(f, state, layout[1]),
        View::SeriesInfo => render_series_info(f, state, layout[1]),
        View::Playing => render_playing(f, state, layout[1]),
        View::Settings => render_settings(f, state, layout[1]),
        View::LibraryBrowser => render_library_browser(f, state, layout[1]),
        View::ContinueWatching | View::LatestItems => render_home(f, state, layout[1]),
    }

    render_footer(f, state, layout[2]);
}

fn render_header(f: &mut Frame, state: &AppState, area: Rect) {
    let title = if state.searching {
        format!("/ {}", state.search_query)
    } else {
        match state.view {
            View::Home => "Remby".to_string(),
            View::ContinueWatching | View::LatestItems => {
                let label = match state.view {
                    View::ContinueWatching => "Continue Watching",
                    _ => "Latest",
                };
                let count = if state.total_items > 0 {
                    format!("{} / {}", state.home_items.len(), state.total_items)
                } else {
                    state.home_items.len().to_string()
                };
                format!("{} [{count}]", label)
            }
            View::Libraries => "Remby - Libraries".to_string(),
            View::Items => {
                let count = if state.total_items > 0 {
                    format!("{} / {}", state.items.len(), state.total_items)
                } else {
                    state.items.len().to_string()
                };
                format!("Remby [{count}]")
            }
            View::SearchResults => format!("Search: {}", state.search_query),
            View::Favorites => format!("Favorites ({})", state.favorites.len()),
            View::TrackSelect => "Select Tracks".to_string(),
            View::SourceSelect => "Select Source".to_string(),
            View::Episodes => format!("{} - Episodes", state.series_name),
            View::SeriesInfo => {
                state.series_state.item.as_ref()
                    .map(|i| i.name.clone())
                    .unwrap_or_else(|| "Series".to_string())
            }
            View::Playing => "Playing".to_string(),
            View::Settings => "Settings".to_string(),
            View::LibraryBrowser => {
                let bs = &state.library_browser_state;
                let count = if bs.total > 0 {
                    format!("{} / {}", bs.items.len(), bs.total)
                } else {
                    bs.items.len().to_string()
                };
                let genre = bs.filter_genre.as_deref().unwrap_or("All");
                let tag = bs.filter_tag.as_deref().unwrap_or("All");
                let studio = bs.filter_studio.as_deref().unwrap_or("All");
                let years = bs.filter_years
                    .map(|(s, e)| format!("{}-{}", s, e))
                    .unwrap_or_else(|| "All".to_string());
                let folder = bs.filter_folder.as_deref().unwrap_or("All");
                let mut filters = Vec::new();
                if genre != "All" { filters.push(format!("G:{}", genre)); }
                if tag != "All" { filters.push(format!("T:{}", tag)); }
                if studio != "All" { filters.push(format!("S:{}", studio)); }
                if years != "All" { filters.push(format!("Y:{}", years)); }
                if folder != "All" { filters.push(format!("F:{}", folder)); }
                let filter_str = if filters.is_empty() { "All".to_string() } else { filters.join(",") };
                format!(
                    "{} | Sort: {}{} | Filter: {} [{}]",
                    bs.library_name,
                    state.library_browser_sort_label(),
                    state.library_browser_order_label(),
                    filter_str,
                    count
                )
            }
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            format!(" {title} "),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    if let Some(ref msg) = state.status_msg {
        let status = match msg {
            crate::app::Message::Loading(spinner, text) => Line::from(vec![
                Span::styled(format!("{} ", spinner), Style::default().fg(Color::Cyan)),
                Span::styled(text.as_str(), Style::default().fg(Color::White)),
            ]),
            crate::app::Message::Info(s) => Line::from(Span::styled(s.as_str(), Style::default().fg(Color::White))),
            crate::app::Message::Success(s) => Line::from(Span::styled(s.as_str(), Style::default().fg(Color::Green))),
            crate::app::Message::Error(s) => Line::from(Span::styled(s.as_str(), Style::default().fg(Color::Red))),
        };
        f.render_widget(Paragraph::new(status), Rect {
            x: inner.x + 1,
            y: inner.y,
            width: inner.width.saturating_sub(2),
            height: 1,
        });
    }
}

fn render_home(f: &mut Frame, state: &AppState, area: Rect) {
    // Build combined list: following updates first, then home items
    let mut combined: Vec<crate::emby::MediaItem> = Vec::new();

    // Following updates section
    for (series_name, episodes) in &state.following_updates {
        if !episodes.is_empty() {
            combined.push(crate::emby::MediaItem::separator(&format!("追剧更新 - {}", series_name)));
            for ep in episodes.iter().take(5) {
                combined.push(ep.clone());
            }
        }
    }

    // Home items
    combined.extend(state.home_items.iter().cloned());

    let items: Vec<ListItem> = combined
        .iter()
        .enumerate()
        .map(|(i, item)| {
            if item.is_separator() {
                ListItem::new(Line::from(Span::styled(
                    item.name.clone(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )))
            } else {
                let is_favorite = item.user_data.as_ref().map(|ud| ud.is_favorite).unwrap_or(false);
                let star = if is_favorite { "★ " } else { "  " };
                let name = item.display_name();
                let duration = item.duration_str().unwrap_or_default();
                let dur = if !duration.is_empty() {
                    format!(" [{duration}]")
                } else {
                    String::new()
                };
                let style = if i == state.selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(vec![
                    Span::styled(star, Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{name}{dur}"), style),
                ]))
            }
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Home"))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .highlight_symbol("▸ ");

    let mut state_list = ListState::default();
    state_list.select(Some(state.selected));
    f.render_stateful_widget(list, area, &mut state_list);
}

fn render_libraries(f: &mut Frame, state: &AppState, area: Rect) {
    let mut items: Vec<ListItem> = Vec::new();

    // Libraries header (not selectable)
    items.push(ListItem::new(Line::from(Span::styled(
        "  Libraries",
        Style::default().fg(Color::Cyan),
    ))));

    // Library items (selectable from index 0)
    for (i, lib) in state.libraries.iter().enumerate() {
        let icon = match lib.collection_type.as_deref() {
            Some("movies") => " ",
            Some("tvshows") => " ",
            Some("music") => " ",
            Some("books") => " ",
            _ => " ",
        };
        let selected = state.view == View::Libraries && state.selected == i;
        let prefix = if selected { "▸ " } else { "  " };
        let style = if selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        items.push(ListItem::new(Line::from(vec![
            Span::raw(prefix),
            Span::styled(icon, Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::styled(&lib.name, style),
        ])));
    }

    // Latest items sections
    let mut idx = state.libraries.len();
    for (lib_name, latest_items) in &state.library_latest {
        let selected = state.view == View::Libraries && state.selected == idx;
        let prefix = if selected { "▸ " } else { "  " };
        let style = if selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        };
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{}最新 {}", prefix, lib_name),
            style,
        ))));
        idx += 1;

        for item in latest_items {
            let name = item.display_name();
            let duration = item.duration_str().unwrap_or_default();
            let dur = if !duration.is_empty() {
                format!(" [{duration}]")
            } else {
                String::new()
            };
            let icon = " ";
            let selected = state.view == View::Libraries && state.selected == idx;
            let prefix = if selected { "▸ " } else { "  " };
            let style = if selected {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            items.push(ListItem::new(Line::from(vec![
                Span::raw(prefix),
                Span::styled(icon, Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::styled(name, style),
                Span::styled(dur, Style::default().fg(Color::DarkGray)),
            ])));
            idx += 1;
        }
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Library"),
        );

    let mut state_list = ListState::default();
    state_list.select(Some(state.selected));
    f.render_stateful_widget(list, area, &mut state_list);
}

fn render_items(f: &mut Frame, state: &AppState, area: Rect) {
    let items_source = match state.view {
        View::Items => &state.items,
        View::SearchResults => &state.search_results,
        View::Favorites => &state.favorites,
        _ => &state.items,
    };
    let title = match state.view {
        View::SearchResults => "Search Results",
        View::Favorites => {
            let fav_count = state.favorites.iter().filter(|i| i.user_data.as_ref().map(|ud| ud.is_favorite).unwrap_or(false)).count();
            let follow_count = state.favorites.iter().filter(|i| i.item_type == "Series" && state.config.following_series.contains(&i.id) && !i.user_data.as_ref().map(|ud| ud.is_favorite).unwrap_or(false)).count();
            if follow_count > 0 {
                &format!("★ {} | ▶ {}", fav_count, follow_count)
            } else {
                "Favorites"
            }
        }
        _ => "Items",
    };

    let items: Vec<ListItem> = items_source
        .iter()
        .map(|item| {
            let is_favorite = item.user_data.as_ref().map(|ud| ud.is_favorite).unwrap_or(false);
            let is_following = state.view == View::Favorites
                && item.item_type == "Series"
                && state.config.following_series.contains(&item.id);
            let (star, follow_mark) = if is_favorite {
                ("★ ", "")
            } else if is_following {
                ("", "▶ ")
            } else {
                ("", "")
            };
            let name = item.display_name();
            let duration = item.duration_str().unwrap_or_default();
            let dur = if !duration.is_empty() {
                format!(" [{duration}]")
            } else {
                String::new()
            };
            ListItem::new(Line::from(vec![
                Span::styled(star, Style::default().fg(Color::Yellow)),
                Span::styled(follow_mark.to_string(), Style::default().fg(Color::Green)),
                Span::raw(name),
                Span::styled(dur, Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if state.view == View::Favorites {
                    title.to_string()
                } else {
                    format!("{title} ({})", items_source.len())
                }),
        )
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .highlight_symbol("▸ ");

    let mut state_list = ListState::default();
    state_list.select(Some(state.selected));
    f.render_stateful_widget(list, area, &mut state_list);
}

fn render_source_select(f: &mut Frame, state: &AppState, area: Rect) {
    let ss = &state.source_state;
    let item_name = ss.item.as_ref().map(|i| i.display_name()).unwrap_or_default();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            format!(" {item_name} - Select Source "),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);
    let inner = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    for (i, source) in ss.sources.iter().enumerate() {
        if i as u16 >= inner.height {
            break;
        }
        let y = inner.y + i as u16;
        let selected = i == state.selected;
        let prefix = if selected { "▸ " } else { "  " };
        let label = source.display_label();
        let spans = if selected {
            Line::from(Span::styled(
                format!("{prefix}{label}"),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from(vec![
                Span::raw(prefix),
                Span::raw(&label),
            ])
        };
        f.buffer_mut().set_line(inner.x, y, &spans, inner.width);
    }
}

fn render_episodes(f: &mut Frame, state: &AppState, area: Rect) {
    let items: Vec<ListItem> = state.episodes.iter().map(|item| {
        let name = item.display_name();
        let duration = item.duration_str().unwrap_or_default();
        let dur = if !duration.is_empty() {
            format!(" [{duration}]")
        } else {
            String::new()
        };
        ListItem::new(Line::from(vec![
            Span::raw(name),
            Span::styled(dur, Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();

    let title = if state.total_episodes > state.episodes.len() {
        format!("{} - Episodes ({}/{})", state.series_name, state.episodes.len(), state.total_episodes)
    } else {
        format!("{} - Episodes ({})", state.series_name, state.episodes.len())
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
        )
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .highlight_symbol("▸ ");

    let mut state_list = ListState::default();
    state_list.select(Some(state.selected));
    f.render_stateful_widget(list, area, &mut state_list);
}

fn render_series_info(f: &mut Frame, state: &AppState, area: Rect) {
    let ss = &state.series_state;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(1),
        ])
        .split(area);

    // Overview panel
    let overview_text = if ss.overview.is_empty() {
        "No overview available".to_string()
    } else if ss.overview.len() > 500 {
        let mut end = 500;
        while !ss.overview.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &ss.overview[..end])
    } else {
        ss.overview.clone()
    };
    let overview = Paragraph::new(overview_text)
        .block(Block::default().borders(Borders::ALL).title("Overview"))
        .wrap(Wrap { trim: true });
    f.render_widget(Clear, layout[0]);
    f.render_widget(overview, layout[0]);

    // Section tabs
    let sections_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(layout[1]);

    let tabs = [
        ("Seasons", &ss.seasons, SeriesSection::Seasons),
        ("Episodes", &ss.episodes, SeriesSection::Episodes),
        ("Similar", &ss.similar, SeriesSection::Similar),
    ];

    for (i, (label, items, section)) in tabs.iter().enumerate() {
        let active = ss.section == *section;
        let border_color = if active { Color::Cyan } else { Color::DarkGray };
        let title_color = if active { Color::Cyan } else { Color::DarkGray };

        let tab_items: Vec<ListItem> = items.iter().map(|item| {
            let name = item.display_name();
            ListItem::new(Line::from(Span::raw(name)))
        }).collect();

        let list = List::new(tab_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(Span::styled(
                        format!(" {} ({}) ", label, items.len()),
                        Style::default().fg(title_color).add_modifier(Modifier::BOLD),
                    ))
            )
            .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .highlight_symbol("▸ ");

        let mut state_list = ListState::default();
        if active {
            state_list.select(Some(state.selected));
        }
        f.render_widget(Clear, sections_layout[i]);
        f.render_stateful_widget(list, sections_layout[i], &mut state_list);
    }
}

fn render_track_info(f: &mut Frame, ps: &crate::app::PlayingState, area: Rect) {
    let track_info = vec![
        Line::from(vec![
            Span::styled("  Video:  ", Style::default().fg(Color::DarkGray)),
            Span::raw(&ps.video_track),
        ]),
        Line::from(vec![
            Span::styled("  Audio:  ", Style::default().fg(Color::DarkGray)),
            Span::raw(&ps.audio_track),
        ]),
        Line::from(vec![
            Span::styled("  Sub:    ", Style::default().fg(Color::DarkGray)),
            Span::raw(&ps.subtitle_track),
        ]),
    ];
    f.render_widget(Clear, area);
    f.render_widget(Paragraph::new(track_info), area);
}

fn render_playing(f: &mut Frame, state: &AppState, area: Rect) {
    let ps = &state.playing_state;
    let has_resume = ps.resume_position.is_some() && !ps.playing;

    let layout = if has_resume {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Min(1),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(area)
    };

    // Title
    let title = Paragraph::new(Span::styled(
        &ps.item_name,
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )).alignment(Alignment::Center);
    f.render_widget(Clear, layout[0]);
    f.render_widget(title, layout[0]);

    // Playing indicator or resume prompt
    if ps.playing {
        let spinner = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
        let idx = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() / 100) as usize % spinner.len();
        let playing_text = Paragraph::new(Span::styled(
            format!("{} Playing in mpv...", spinner[idx]),
            Style::default().fg(Color::Cyan),
        )).alignment(Alignment::Center);
        f.render_widget(Clear, layout[1]);
        f.render_widget(playing_text, layout[1]);
    } else {
        let prompt = Paragraph::new(Span::styled(
            "Choose playback option:",
            Style::default().fg(Color::Yellow),
        )).alignment(Alignment::Center);
        f.render_widget(Clear, layout[1]);
        f.render_widget(prompt, layout[1]);
    }

    // Track info
    render_track_info(f, ps, layout[2]);

    // Resume choice
    if has_resume {
        let ticks = ps.resume_position.unwrap();
        let secs = ticks / 10_000_000;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        let resume_time = format!("{}:{:02}", m, s);

        let options = vec![
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    if ps.option_selected == 0 { "▸ " } else { "  " },
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("Resume from {}", resume_time),
                    if ps.option_selected == 0 {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    if ps.option_selected == 1 { "▸ " } else { "  " },
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    "Play from start",
                    if ps.option_selected == 1 {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    },
                ),
            ]),
        ];
        let options_widget = Paragraph::new(options);
        f.render_widget(Clear, layout[5]);
        f.render_widget(options_widget, layout[5]);
    } else if ps.playing {
        render_track_info(f, ps, layout[5]);
    }

    // URL (truncated)
    let url_idx = if has_resume { 6 } else { 5 };
    let url_text = Paragraph::new(Span::styled(
        &ps.url,
        Style::default().fg(Color::DarkGray),
    )).wrap(Wrap { trim: false });
    f.render_widget(Clear, layout[url_idx]);
    f.render_widget(url_text, layout[url_idx]);
}

fn render_settings(f: &mut Frame, state: &AppState, area: Rect) {
    let ss = &state.settings_state;

    let header_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let name_col = 24; // Fixed width for name column

    let mut items: Vec<ListItem> = Vec::new();

    // Column headers - use same layout as rows
    let enabled_header = if ss.column == SettingsColumn::Enabled { ">Enabled" } else { " Enabled" };
    let latest_header = if ss.column == SettingsColumn::Latest { ">Latest" } else { " Latest" };
    let header_name = format!("{:<width$}", " Library", width = name_col);
    items.push(ListItem::new(Line::from(vec![
        Span::styled(header_name, header_style),
        Span::styled(
            enabled_header,
            if ss.column == SettingsColumn::Enabled { header_style } else { Style::default().fg(Color::DarkGray) },
        ),
        Span::raw("  "),
        Span::styled(
            latest_header,
            if ss.column == SettingsColumn::Latest { header_style } else { Style::default().fg(Color::DarkGray) },
        ),
    ])));

    for (i, lib) in ss.libraries.iter().enumerate() {
        let selected = i == ss.selected;
        let name_style = if selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let enabled_mark = if lib.enabled { "[x]" } else { "[ ]" };
        let latest_mark = if lib.fetch_latest { "[x]" } else { "[ ]" };

        let enabled_style = if selected && ss.column == SettingsColumn::Enabled {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else if lib.enabled {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let latest_style = if selected && ss.column == SettingsColumn::Latest {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else if lib.fetch_latest {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Build name with marker, pad to fixed width
        let marker = if selected { ">" } else { " " };
        let name_with_marker = format!("{}{}", marker, lib.name);
        let display_width: usize = name_with_marker.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum();
        let padded_name = if display_width < name_col {
            format!("{}{}", name_with_marker, " ".repeat(name_col - display_width))
        } else {
            name_with_marker
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(padded_name, name_style),
            Span::styled(enabled_mark, enabled_style),
            Span::raw("   "),
            Span::styled(latest_mark, latest_style),
        ])));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(
                    " Settings - Library Preferences ",
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ))
                .title_alignment(Alignment::Center),
        );

    f.render_widget(Clear, area);
    f.render_widget(list, area);
}

fn render_footer(f: &mut Frame, state: &AppState, area: Rect) {
    let help = match state.view {
        View::Home => "↑↓: navigate | Enter: play/open | l: libraries | /: search | f: follow | F: favorites | Ctrl+F: refresh | q: quit",
        View::ContinueWatching | View::LatestItems => "↑↓: navigate | Enter: play | /: search | ←/BS: back",
        View::Libraries => "↑↓: select | Enter: open | ←/BS: back",
        View::Items => "↑↓: navigate | Enter: open/play | f: follow | ←/BS: back | /: search",
        View::SearchResults => "↑↓: navigate | Enter: play | f: follow | ←/BS: back",
        View::TrackSelect => "←/→: section | ↑/↓: select track | Enter: play | Esc: back",
        View::SourceSelect => "↑↓: select source | Enter: confirm | Esc: back",
        View::Episodes => "↑↓: navigate | Enter: play | e: episodes | ←/BS: back",
        View::SeriesInfo => "←/→: section | ↑/↓: select | Enter: open | f: follow | e: episodes | Esc: back",
        View::Playing => {
            if state.playing_state.playing {
                "Esc: back to tracks"
            } else if state.playing_state.resume_position.is_some() {
                "↑/↓: select | Enter: confirm | Esc: back"
            } else {
                "Enter: play | Esc: back to tracks"
            }
        }
        View::Settings => "↑↓: nav | ←/→: col | Space: toggle | Shift+↑↓: move | Enter: save | Esc: cancel",
        View::LibraryBrowser => {
            if state.library_browser_state.panel == BrowserPanel::Filter {
                "j/k: Nav | ←/→: Section | Enter: Select/Apply | Esc: Cancel"
            } else if state.library_browser_state.panel != BrowserPanel::None {
                "j/k: Navigate | Enter: Select | Esc: Close"
            } else {
                "j/k: Navigate | Enter: Open | Ctrl+s: Sort | Ctrl+f: Filter | /: search | z: Favorite | Z: View favorites | Esc: Back"
            }
        },
        View::Favorites => "↑↓: navigate | Enter: open/play | f: follow | z: unfavorite | m: mark watched | ←/BS: back",
    };
    let help = if state.searching {
        "Enter: search | Esc: cancel"
    } else {
        help
    };
    let line = Line::from(Span::styled(help, Style::default().fg(Color::DarkGray)))
        .alignment(Alignment::Center);
    f.render_widget(Clear, area);
    f.render_widget(Paragraph::new(line), area);
}

pub fn track_label(stream: &crate::emby::MediaStream) -> String {
    if let Some(ref title) = stream.display_title {
        if !title.is_empty() {
            return title.clone();
        }
    }
    if let Some(ref title) = stream.title {
        if !title.is_empty() {
            return title.clone();
        }
    }
    let lang = if stream.language.is_empty() { "und" } else { &stream.language };
    match stream.stream_type.as_str() {
        "Video" => {
            if let (Some(_w), Some(h)) = (stream.width, stream.height) {
                format!("{} {}p", lang, h)
            } else {
                format!("{} {}", lang, stream.codec)
            }
        }
        "Audio" => {
            let layout = stream.channel_layout.as_deref().unwrap_or("");
            if !layout.is_empty() {
                format!("{} {} ({})", lang, stream.codec, layout)
            } else {
                format!("{} {}", lang, stream.codec)
            }
        }
        _ => format!("{} {}", lang, stream.codec),
    }
}

fn render_track_select(f: &mut Frame, state: &AppState, area: Rect) {
    let ts = &state.track_state;
    let item_name = ts.item.as_ref().map(|i| i.name.as_str()).unwrap_or("");

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            format!(" {item_name} "),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);
    f.render_widget(Clear, layout[0]);
    f.render_widget(title_block, layout[0]);

    let sections_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(layout[1]);

    render_track_section(f, state, sections_layout[0], "Video", &ts.video_tracks, ts.selected_video, &ts.section, TrackSection::Video);
    render_track_section(f, state, sections_layout[1], "Audio", &ts.audio_tracks, ts.selected_audio, &ts.section, TrackSection::Audio);
    render_track_section(f, state, sections_layout[2], "Subtitle", &ts.subtitle_tracks, ts.selected_subtitle, &ts.section, TrackSection::Subtitle);
}

fn render_track_section(
    f: &mut Frame, _state: &AppState, area: Rect,
    title: &str, tracks: &[crate::emby::MediaStream],
    selected: usize, current_section: &TrackSection, section: TrackSection,
) {
    let active = *current_section == section;
    let border_color = if active { Color::Cyan } else { Color::DarkGray };

    let items: Vec<ListItem> = tracks.iter().enumerate().map(|(i, track)| {
        let label = track_label(track);
        let marker = if i == selected { "▸ " } else { "  " };
        let style = if i == selected && active {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(Line::from(Span::styled(format!("{marker}{label}"), style)))
    }).collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    format!(" {title} ({}) ", tracks.len()),
                    Style::default().fg(if active { Color::Cyan } else { Color::DarkGray }).add_modifier(Modifier::BOLD),
                ))
        );

    f.render_widget(Clear, area);
    f.render_widget(list, area);
}

fn render_library_browser(f: &mut Frame, state: &AppState, area: Rect) {
    let bs = &state.library_browser_state;

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

    match bs.panel {
        BrowserPanel::Sort => render_sort_panel(f, state, area),
        BrowserPanel::Filter => render_filter_panel(f, state, area),
        BrowserPanel::None => {}
    }
}

fn render_sort_panel(f: &mut Frame, state: &AppState, area: Rect) {
    let bs = &state.library_browser_state;
    let order_label = match bs.sort_order {
        SortOrder::Asc => "Order: Ascending",
        SortOrder::Desc => "Order: Descending",
    };
    let options = ["Name", "Year", "Rating", "Date Added", order_label];

    let items: Vec<ListItem> = options.iter().enumerate().map(|(i, opt)| {
        let selected = i == bs.panel_selected;
        let current = if i < 4 {
            (match bs.sort_by {
                ItemSort::Name => 0,
                ItemSort::Year => 1,
                ItemSort::Rating => 2,
                ItemSort::DateAdded => 3,
            }) == i
        } else {
            false
        };

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

    let popup = centered_rect(30, 14, area);
    f.render_widget(Clear, popup);
    f.render_widget(list, popup);
}

fn render_filter_panel(f: &mut Frame, state: &AppState, area: Rect) {
    let bs = &state.library_browser_state;

    let mut items: Vec<ListItem> = Vec::new();

    // Section header
    let section_title = match bs.filter_section {
        FilterSection::Genre => format!("Genre ({})", bs.available_genres.len()),
        FilterSection::Tag => format!("Tag ({})", bs.available_tags.len()),
        FilterSection::Studio => format!("Studio ({})", bs.available_studios.len()),
        FilterSection::Year => "Year".to_string(),
        FilterSection::Folder => format!("Folder ({})", bs.available_folders.len()),
    };
    items.push(ListItem::new(Line::from(Span::styled(
        format!("── {} ──", section_title),
        Style::default().fg(Color::DarkGray),
    ))));

    match bs.filter_section {
        FilterSection::Genre => {
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
            // Next section option
            let next_selected = bs.panel_selected == bs.available_genres.len();
            if next_selected {
                items.push(ListItem::new(Line::from(Span::styled(
                    "  → Tag",
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                ))));
            } else {
                items.push(ListItem::new(Line::from(Span::raw("  → Tag"))));
            }
        }
        FilterSection::Tag => {
            for (i, tag) in bs.available_tags.iter().enumerate() {
                let selected = i == bs.panel_selected;
                let active = bs.filter_tag.as_ref() == Some(tag);

                let style = if selected {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else if active {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };

                let marker = if active { "● " } else { "  " };
                items.push(ListItem::new(Line::from(Span::styled(
                    format!("{}{}", marker, tag),
                    style,
                ))));
            }
            let next_selected = bs.panel_selected == bs.available_tags.len();
            if next_selected {
                items.push(ListItem::new(Line::from(Span::styled(
                    "  → Studio",
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                ))));
            } else {
                items.push(ListItem::new(Line::from(Span::raw("  → Studio"))));
            }
        }
        FilterSection::Studio => {
            for (i, studio) in bs.available_studios.iter().enumerate() {
                let selected = i == bs.panel_selected;
                let active = bs.filter_studio.as_ref() == Some(studio);

                let style = if selected {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else if active {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };

                let marker = if active { "● " } else { "  " };
                items.push(ListItem::new(Line::from(Span::styled(
                    format!("{}{}", marker, studio),
                    style,
                ))));
            }
            let next_selected = bs.panel_selected == bs.available_studios.len();
            if next_selected {
                items.push(ListItem::new(Line::from(Span::styled(
                    "  → Year",
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                ))));
            } else {
                items.push(ListItem::new(Line::from(Span::raw("  → Year"))));
            }
        }
        FilterSection::Year => {
            let year_active = bs.filter_years.is_some() || bs.filter_year_field.is_some();
            let year_style = if bs.panel_selected == 0 {
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
        }
        FilterSection::Folder => {
            for (i, folder) in bs.available_folders.iter().enumerate() {
                let selected = i == bs.panel_selected;
                let active = bs.filter_folder.as_ref() == Some(&folder.id);

                let style = if selected {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else if active {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };

                let marker = if active { "● " } else { "  " };
                items.push(ListItem::new(Line::from(Span::styled(
                    format!("{}{}", marker, folder.name),
                    style,
                ))));
            }
        }
    }

    let list = List::new(items.clone())
        .block(Block::default().borders(Borders::ALL).title(" Filter "))
        .highlight_style(Style::default())
        .highlight_symbol("");

    let total_items = items.len();
    let max_height = 20usize;
    let height = (total_items + 2).min(max_height) as u16;
    let popup = centered_rect(40, height, area);

    let mut state_list = ListState::default();
    state_list.select(Some(bs.panel_selected + 1)); // +1 for section header
    f.render_widget(Clear, popup);
    f.render_stateful_widget(list, popup, &mut state_list);
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
