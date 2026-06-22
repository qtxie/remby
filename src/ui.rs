use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{AppState, BrowserPanel, FilterSection, ItemSort, SeriesSection, SettingsColumn, SettingsSection, SortOrder, TrackSection, View, WizardField};

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
        View::AccountManager => render_account_manager(f, state, layout[1]),
        View::Wizard => render_wizard(f, state, layout[1]),
        View::MpvPrompt => render_mpv_prompt(f, state, layout[1]),
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
            View::AccountManager => "Account Manager".to_string(),
            View::Wizard => "Setup Wizard".to_string(),
            View::MpvPrompt => "Configure MPV Path".to_string(),
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
    let mut combined: Vec<crate::emby::MediaItem> = Vec::new();

    for (series_name, episodes) in &state.following_updates {
        if !episodes.is_empty() {
            combined.push(crate::emby::MediaItem::separator(&format!("追剧更新 - {}", series_name)));
            for ep in episodes.iter().take(5) {
                combined.push(ep.clone());
            }
        }
    }

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

    let halves = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top = if has_resume {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Min(1),
            ])
            .split(halves[0])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(halves[0])
    };

    // Title
    let title = Paragraph::new(Span::styled(
        &ps.item_name,
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )).alignment(Alignment::Center);
    f.render_widget(Clear, top[0]);
    f.render_widget(title, top[0]);

    // Playing indicator or resume prompt
    if ps.playing {
        let spinner = ["\u{28f8}", "\u{28fd}", "\u{28fb}", "\u{28bf}", "\u{28ff}", "\u{28fe}", "\u{287f}", "\u{287b}"];
        let idx = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() / 100) as usize % spinner.len();
        let playing_text = Paragraph::new(Span::styled(
            format!("{} Playing in mpv...", spinner[idx]),
            Style::default().fg(Color::Cyan),
        )).alignment(Alignment::Center);
        f.render_widget(Clear, top[1]);
        f.render_widget(playing_text, top[1]);
    } else {
        let prompt = Paragraph::new(Span::styled(
            "Choose playback option:",
            Style::default().fg(Color::Yellow),
        )).alignment(Alignment::Center);
        f.render_widget(Clear, top[1]);
        f.render_widget(prompt, top[1]);
    }

    // Track info
    render_track_info(f, ps, top[2]);

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
                    if ps.option_selected == 0 { "\u{25b8} " } else { "  " },
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
                    if ps.option_selected == 1 { "\u{25b8} " } else { "  " },
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
        f.render_widget(Clear, top[5]);
        f.render_widget(options_widget, top[5]);
    }

    // URL at bottom of top half
    let url_idx = if has_resume { 6 } else { 3 };
    let url_text = Paragraph::new(Span::styled(
        &ps.url,
        Style::default().fg(Color::DarkGray),
    )).wrap(Wrap { trim: false });
    f.render_widget(Clear, top[url_idx]);
    f.render_widget(url_text, top[url_idx]);

    // Bottom half: mpv output panel
    let output_area = halves[1];
    if !state.mpv_output.is_empty() {
        let output_len = state.mpv_output.len();
        let visible_height = output_area.height as usize;
        let max_scroll = output_len.saturating_sub(visible_height);
        let scroll = state.mpv_output_scroll.min(max_scroll);

        let end = state.mpv_output.len().saturating_sub(scroll);
        let start = end.saturating_sub(visible_height);
        let visible: Vec<Line> = state.mpv_output[start..end].iter().map(|l| {
            let style = if l.contains("error") || l.contains("Error") || l.contains("ERROR") {
                Style::default().fg(Color::Red)
            } else if l.contains("warn") || l.contains("Warn") || l.contains("WARN") {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            Line::from(Span::styled(l.as_str(), style))
        }).collect();

        let title = format!(" mpv output ({} lines) ", output_len);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(title, Style::default().fg(Color::DarkGray)));
        let paragraph = Paragraph::new(visible).block(block);
        f.render_widget(Clear, output_area);
        f.render_widget(paragraph, output_area);
    } else {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" mpv output ");
        f.render_widget(Clear, output_area);
        f.render_widget(block, output_area);
    }
}

fn render_settings(f: &mut Frame, state: &AppState, area: Rect) {
    let ss = &state.settings_state;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    // Libraries section
    let header_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let name_col = 24;

    let mut items: Vec<ListItem> = Vec::new();

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
        let selected = i == ss.selected && ss.section == SettingsSection::Libraries;
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

    let lib_border = if ss.section == SettingsSection::Libraries { Color::Cyan } else { Color::DarkGray };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(lib_border))
                .title(Span::styled(
                    " Settings - Library Preferences ",
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ))
                .title_alignment(Alignment::Center),
        );

    f.render_widget(Clear, area);
    f.render_widget(list, layout[0]);

    // MPV path section
    let mpv_active = ss.section == SettingsSection::MpvPath;
    let mpv_border = if mpv_active { Color::Cyan } else { Color::DarkGray };
    let cursor = if mpv_active { "█" } else { "" };
    let mpv_text = format!("  MPV Path: {}{}", ss.mpv_path, cursor);
    let mpv_style = if mpv_active {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let mpv_block = Paragraph::new(Span::styled(mpv_text, mpv_style))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(mpv_border)).title(" MPV "));
    f.render_widget(mpv_block, layout[1]);
}

fn render_footer(f: &mut Frame, state: &AppState, area: Rect) {
    let help = match state.view {
        View::Home => "l: libraries | /: search | f: follow | F: favorites | u: accounts | Ctrl+F: refresh | q: quit",
        View::ContinueWatching | View::LatestItems => "/: search",
        View::Libraries => "",
        View::Items => "f: follow | /: search",
        View::SearchResults => "f: follow",
        View::TrackSelect => "←/→: section | Enter: play",
        View::SourceSelect => "Enter: confirm",
        View::Episodes => "e: episodes",
        View::SeriesInfo => "←/→: section | Enter: open | f: follow | e: episodes",
        View::Playing => {
            if state.playing_state.playing {
                ""
            } else if state.playing_state.resume_position.is_some() {
                "↑/↓: select | Enter: confirm"
            } else {
                "Enter: play"
            }
        }
        View::Settings => "Tab: section | ←/→: col | Space: toggle | Shift+↑↓: move | Enter: save",
        View::LibraryBrowser => {
            if state.library_browser_state.panel == BrowserPanel::Filter {
                "←/→: Section | Enter: Apply"
            } else if state.library_browser_state.panel != BrowserPanel::None {
                "Enter: Select"
            } else {
                "Ctrl+s: Sort | Ctrl+f: Filter | /: search | e: info | z: Favorite | Z: Favorites"
            }
        },
        View::Favorites => "f: follow | z: unfavorite | m: mark watched",
        View::AccountManager => "a: add | e: edit | d: delete | Enter: switch | Esc: back",
        View::Wizard => "Tab: next field | Enter: continue | Esc: quit",
        View::MpvPrompt => "Enter: save & play | Esc: cancel",
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

fn render_account_manager(f: &mut Frame, state: &AppState, area: Rect) {
    let ams = &state.account_manager_state;

    match &ams.action {
        crate::app::AccountManagerAction::View => {
            render_account_list(f, state, area);
        }
        crate::app::AccountManagerAction::Add | crate::app::AccountManagerAction::Edit(_) => {
            render_account_form(f, state, area);
        }
        crate::app::AccountManagerAction::Delete(_) => {
            render_account_list(f, state, area);
            render_delete_confirm(f, state, area);
        }
    }
}

fn render_account_list(f: &mut Frame, state: &AppState, area: Rect) {
    let ams = &state.account_manager_state;
    let mut items: Vec<ListItem> = Vec::new();

    for (i, acc) in ams.accounts.iter().enumerate() {
        let is_active = ams.last_account_id.as_ref() == Some(&acc.id);
        let marker = if i == ams.selected { "▸ " } else { "  " };
        let active_mark = if is_active { "● " } else { "  " };
        let style = if i == ams.selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };
        let label = format!("{}{}{} @ {}", marker, active_mark, acc.label, acc.server);
        items.push(ListItem::new(Line::from(Span::styled(label, style))));
    }

    let add_idx = ams.accounts.len();
    let add_marker = if add_idx == ams.selected { "▸ " } else { "  " };
    let add_style = if add_idx == ams.selected {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Yellow)
    };
    items.push(ListItem::new(Line::from(Span::styled(
        format!("{}+ Add new account", add_marker),
        add_style,
    ))));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Account Manager "))
        .highlight_style(Style::default())
        .highlight_symbol("");

    f.render_widget(Clear, area);
    f.render_widget(list, area);

    if let Some(ref msg) = ams.status_msg {
        let status_area = Rect {
            x: area.x + 1,
            y: area.y + area.height.saturating_sub(2),
            width: area.width.saturating_sub(2),
            height: 1,
        };
        f.render_widget(
            Paragraph::new(Span::styled(msg.as_str(), Style::default().fg(Color::Green))),
            status_area,
        );
    }
}

fn render_account_form(f: &mut Frame, state: &AppState, area: Rect) {
    let ams = &state.account_manager_state;
    let is_edit = matches!(ams.action, crate::app::AccountManagerAction::Edit(_));
    let title = if is_edit { "Edit Account" } else { "Add Account" };

    let fields = [
        ("Label", &ams.input_label, crate::app::AccountInputField::Label),
        ("Server", &ams.input_server, crate::app::AccountInputField::Server),
        ("Username", &ams.input_username, crate::app::AccountInputField::Username),
        ("Password", &ams.input_password, crate::app::AccountInputField::Password),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (_i, (label, value, field)) in fields.iter().enumerate() {
        let active = ams.input_field == *field;
        let marker = if active { "▸ " } else { "  " };
        let field_style = if active {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let display_value = if *field == crate::app::AccountInputField::Password {
            "*".repeat(value.len())
        } else {
            value.to_string()
        };

        let cursor = if active { "█" } else { "" };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{}{:<12}", marker, label), field_style),
            Span::raw(": "),
            Span::raw(format!("{}{}", display_value, cursor)),
        ])));
    }

    items.push(ListItem::new(Line::from(Span::raw(""))));
    items.push(ListItem::new(Line::from(Span::styled(
        "Tab: next field | Enter: save | Esc: cancel",
        Style::default().fg(Color::DarkGray),
    ))));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", title)))
        .highlight_style(Style::default())
        .highlight_symbol("");

    let popup = centered_rect(60, 10, area);
    f.render_widget(Clear, popup);
    f.render_widget(list, popup);
}

fn render_delete_confirm(f: &mut Frame, state: &AppState, area: Rect) {
    let ams = &state.account_manager_state;
    if let crate::app::AccountManagerAction::Delete(idx) = &ams.action {
        let name = ams.accounts.get(*idx).map(|a| a.label.as_str()).unwrap_or("?");
        let text = vec![
            Line::from(Span::styled(
                format!("Delete account '{}'?", name),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "y: confirm | n: cancel",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        let popup = centered_rect(40, 6, area);
        f.render_widget(Clear, popup);
        f.render_widget(
            Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title(" Confirm "))
                .alignment(Alignment::Center),
            popup,
        );
    }
}

fn render_wizard(f: &mut Frame, state: &AppState, area: Rect) {
    let ws = &state.wizard_state;
    let fields = [
        ("Server URL", &ws.server, WizardField::Server),
        ("Username", &ws.username, WizardField::Username),
        ("Password", &ws.password, WizardField::Password),
        ("MPV Path", &ws.mpv_path, WizardField::MpvPath),
    ];
    let mut items: Vec<ListItem> = Vec::new();
    items.push(ListItem::new(Line::from(Span::styled(
        "  Welcome to remby! Please configure your connection.",
        Style::default().fg(Color::Yellow),
    ))));
    items.push(ListItem::new(Line::from(Span::raw(""))));
    for (label, value, field) in fields.iter() {
        let active = ws.step == *field;
        let marker = if active { "▸ " } else { "  " };
        let field_style = if active {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let display_value = if *field == WizardField::Password {
            "*".repeat(value.len())
        } else {
            value.to_string()
        };
        let cursor = if active { "█" } else { "" };
        let hint = if *field == WizardField::MpvPath {
            "  (Tab to skip)"
        } else {
            ""
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{}{:<14}", marker, label), field_style),
            Span::raw(": "),
            Span::raw(format!("{}{}", display_value, cursor)),
            Span::styled(hint.to_string(), Style::default().fg(Color::DarkGray)),
        ])));
    }
    if let Some(ref msg) = ws.status_msg {
        items.push(ListItem::new(Line::from(Span::styled(
            format!("  {}", msg),
            Style::default().fg(Color::Red),
        ))));
    }
    items.push(ListItem::new(Line::from(Span::raw(""))));
    items.push(ListItem::new(Line::from(Span::styled(
        "  Enter: next | Tab: skip MPV | Esc: quit",
        Style::default().fg(Color::DarkGray),
    ))));
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Setup Wizard "))
        .highlight_style(Style::default())
        .highlight_symbol("");
    let popup = centered_rect(60, 14, area);
    f.render_widget(Clear, popup);
    f.render_widget(list, popup);
}

fn render_mpv_prompt(f: &mut Frame, state: &AppState, area: Rect) {
    let ms = &state.mpv_prompt_state;
    let items: Vec<ListItem> = vec![
        ListItem::new(Line::from(Span::styled(
            "  MPV path not configured. Please enter the path to mpv:",
            Style::default().fg(Color::Yellow),
        ))),
        ListItem::new(Line::from(Span::raw(""))),
        ListItem::new(Line::from(vec![
            Span::styled("  MPV Path: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(&ms.mpv_path),
            Span::raw("\u{2588}"),
        ])),
        ListItem::new(Line::from(Span::raw(""))),
        ListItem::new(Line::from(Span::styled(
            "  Enter: save & play | Esc: cancel",
            Style::default().fg(Color::DarkGray),
        ))),
    ];
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" MPV Path "))
        .highlight_style(Style::default())
        .highlight_symbol("");
    let popup = centered_rect(50, 8, area);
    f.render_widget(Clear, popup);
    f.render_widget(list, popup);
}
