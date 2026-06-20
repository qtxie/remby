use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{AppState, SeriesSection, TrackSection, View};

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
        View::SourceSelect => render_source_select(f, state, layout[1]),
        View::TrackSelect => render_track_select(f, state, layout[1]),
        View::Episodes => render_episodes(f, state, layout[1]),
        View::SeriesInfo => render_series_info(f, state, layout[1]),
        View::Playing => render_playing(f, state, layout[1]),
    }

    render_footer(f, state, layout[2]);
}

fn render_header(f: &mut Frame, state: &AppState, area: Rect) {
    let title = if state.searching {
        format!("/ {}", state.search_query)
    } else {
        match state.view {
            View::Home => "Remby".to_string(),
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
            View::TrackSelect => "Select Tracks".to_string(),
            View::SourceSelect => "Select Source".to_string(),
            View::Episodes => format!("{} - Episodes", state.series_name),
            View::SeriesInfo => {
                state.series_state.item.as_ref()
                    .map(|i| i.name.clone())
                    .unwrap_or_else(|| "Series".to_string())
            }
            View::Playing => "Playing".to_string(),
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

    if !state.status_msg.is_empty() {
        let is_loading = state.status_msg.contains("Loading");
        let status = if is_loading {
            Line::from(vec![
                Span::styled(
                    &state.status_msg,
                    Style::default().fg(Color::Yellow),
                ),
            ])
        } else {
            Line::from(Span::styled(
                &state.status_msg,
                Style::default().fg(Color::DarkGray),
            ))
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
    let items: Vec<ListItem> = state
        .home_items
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
                let icon = if item.is_video() { "▶ " } else { "  " };
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
                    Span::styled(icon, Style::default().fg(Color::Cyan)),
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
    // Build combined list: libraries + latest items sections
    let mut items: Vec<ListItem> = Vec::new();

    // Libraries section
    items.push(ListItem::new(Line::from(Span::styled(
        " Libraries",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    ))));

    for lib in &state.libraries {
        let icon = match lib.collection_type.as_deref() {
            Some("movies") => " ",
            Some("tvshows") => " ",
            Some("music") => " ",
            Some("books") => " ",
            _ => " ",
        };
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(icon, Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::raw(&lib.name),
        ])));
    }

    // Latest items sections
    for (lib_name, latest_items) in &state.library_latest {
        items.push(ListItem::new(Line::from(Span::styled(
            format!(" 最新 {}", lib_name),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ))));

        for item in latest_items {
            let name = item.display_name();
            let duration = item.duration_str().unwrap_or_default();
            let dur = if !duration.is_empty() {
                format!(" [{duration}]")
            } else {
                String::new()
            };
            let icon = if item.is_video() { "▶" } else { " " };
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(icon, Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::raw(name),
                Span::styled(dur, Style::default().fg(Color::DarkGray)),
            ])));
        }
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Library"),
        )
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .highlight_symbol("▸ ");

    let mut state_list = ListState::default();
    state_list.select(Some(state.selected));
    f.render_stateful_widget(list, area, &mut state_list);
}

fn render_items(f: &mut Frame, state: &AppState, area: Rect) {
    let items_source = match state.view {
        View::Items => &state.items,
        View::SearchResults => &state.search_results,
        _ => &state.items,
    };
    let title = match state.view {
        View::SearchResults => "Search Results",
        _ => "Items",
    };

    let items: Vec<ListItem> = items_source
        .iter()
        .map(|item| {
            let icon = if item.is_folder() {
                " "
            } else if item.is_video() {
                "▶"
            } else {
                " "
            };
            let name = item.display_name();
            let duration = item.duration_str().unwrap_or_default();
            let dur = if !duration.is_empty() {
                format!(" [{duration}]")
            } else {
                String::new()
            };
            ListItem::new(Line::from(vec![
                Span::styled(icon, Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::raw(name),
                Span::styled(dur, Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{title} ({})", items_source.len())),
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
            Span::raw("  "),
            Span::styled("▶", Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::raw(name),
            Span::styled(dur, Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{} - All Episodes ({})", state.series_name, state.episodes.len()))
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
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    // Overview panel
    let overview_text = if ss.overview.is_empty() {
        "No overview available".to_string()
    } else if ss.overview.len() > 300 {
        format!("{}...", &ss.overview[..300])
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
        .split(layout[2]);

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
    let tracks = Paragraph::new(track_info);
    f.render_widget(Clear, layout[2]);
    f.render_widget(tracks, layout[2]);

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
        // Track info when playing
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
        let tracks = Paragraph::new(track_info);
        let url_idx = 5;
        f.render_widget(Clear, layout[url_idx]);
        f.render_widget(tracks, layout[url_idx]);
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

fn render_footer(f: &mut Frame, state: &AppState, area: Rect) {
    let help = match state.view {
        View::Home => "↑↓: navigate | Enter: play | l: libraries | /: search | q: quit",
        View::Libraries => "↑↓: select | Enter: open | ←/BS: back",
        View::Items => "↑↓: navigate | Enter: open/play | ←/BS: back | /: search",
        View::SearchResults => "↑↓: navigate | Enter: play | ←/BS: back",
        View::TrackSelect => "←/→: section | ↑/↓: select track | Enter: play | Esc: back",
        View::SourceSelect => "↑↓: select source | Enter: confirm | Esc: back",
        View::Episodes => "↑↓: navigate | Enter: play | e: episodes | ←/BS: back",
        View::SeriesInfo => "←/→: section | ↑/↓: select | Enter: open | e: episodes | Esc: back",
        View::Playing => {
            if state.playing_state.playing {
                "Esc: back to tracks"
            } else if state.playing_state.resume_position.is_some() {
                "↑/↓: select | Enter: confirm | Esc: back"
            } else {
                "Enter: play | Esc: back to tracks"
            }
        }
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
