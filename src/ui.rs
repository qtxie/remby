use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{AppState, BrowserPanel, FilterSection, ItemSort, SeriesSection, SettingsColumn, SettingsSection, SortOrder, TrackSection, View, WizardField};
use crate::i18n::{t, tf};
use unicode_width::UnicodeWidthChar;

fn rounded_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
}

pub fn render(f: &mut Frame, state: &AppState) {
    let area = f.area();
    let theme = &state.theme;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(f, state, layout[0], theme);

    if state.view == View::Help {
        render_help(f, state, area, theme);
        return;
    }

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
        View::Help => unreachable!(),
    }

    render_footer(f, state, layout[2], theme);
}

fn render_header(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
    let title = if state.searching {
        format!("/ {}", state.search_query)
    } else {
        match state.view {
            View::Home => t("title.home").to_string(),
            View::ContinueWatching | View::LatestItems => {
                let label = match state.view {
                    View::ContinueWatching => t("title.continue_watching"),
                    _ => t("title.latest"),
                };
                let count = if state.total_items > 0 {
                    format!("{} / {}", state.home_items.len(), state.total_items)
                } else {
                    state.home_items.len().to_string()
                };
                format!("{} [{count}]", label)
            }
            View::Libraries => t("title.libraries").to_string(),
            View::Items => {
                let count = if state.total_items > 0 {
                    format!("{} / {}", state.items.len(), state.total_items)
                } else {
                    state.items.len().to_string()
                };
                format!("Remby [{count}]")
            }
            View::SearchResults => format!("{}: {}", t("title.search"), state.search_query),
            View::Favorites => format!("{} ({})", t("title.favorites"), state.favorites.len()),
            View::AccountManager => t("title.account_manager").to_string(),
            View::Wizard => t("title.wizard").to_string(),
            View::MpvPrompt => t("title.mpv_prompt").to_string(),
            View::Help => "Help".to_string(),
            View::TrackSelect => t("title.track_select").to_string(),
            View::SourceSelect => t("title.source_select").to_string(),
            View::Episodes => format!("{} - {}", state.series_name, t("title.episodes")),
            View::SeriesInfo => {
                state.series_state.item.as_ref()
                    .map(|i| i.name.clone())
                    .unwrap_or_else(|| "Series".to_string())
            }
            View::Playing => t("title.playing").to_string(),
            View::Settings => t("title.settings").to_string(),
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
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            format!(" {title} "),
            Style::default()
                .fg(theme.text)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    if let Some(ref msg) = state.status_msg {
        let status = match msg {
            crate::app::Message::Loading(spinner, text) => Line::from(vec![
                Span::styled(format!("{} ", spinner), Style::default().fg(theme.accent)),
                Span::styled(text.as_str(), Style::default().fg(theme.text)),
            ]),
            crate::app::Message::Info(s) => Line::from(Span::styled(s.as_str(), Style::default().fg(theme.text))),
            crate::app::Message::Success(s) => Line::from(Span::styled(s.as_str(), Style::default().fg(theme.success))),
            crate::app::Message::Error(s) => Line::from(Span::styled(s.as_str(), Style::default().fg(theme.error))),
        };
        f.render_widget(Paragraph::new(status), Rect {
            x: inner.x + 1,
            y: inner.y,
            width: inner.width.saturating_sub(2),
            height: 1,
        });
    }
}

fn render_home(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
    let mut combined: Vec<crate::emby::MediaItem> = Vec::new();

    for (series_name, episodes) in &state.following_updates {
        if !episodes.is_empty() {
            combined.push(crate::emby::MediaItem::separator(&format!("{} - {}", t("item.following_update"), series_name)));
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
                let selected = i == state.selected;
                let prefix = if selected { "▸ " } else { "  " };
                ListItem::new(Line::from(Span::styled(
                    format!("{}{}", prefix, item.name),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )))
            } else {
                let is_favorite = item.user_data.as_ref().map(|ud| ud.is_favorite).unwrap_or(false);
                let star = if is_favorite { "★ " } else { "" };
                let name = item.display_name();
                let duration = item.duration_str().unwrap_or_default();
                let dur = if !duration.is_empty() {
                    format!(" [{duration}]")
                } else {
                    String::new()
                };
                let selected = i == state.selected;
                let style = if selected {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                // Progress for continue watching items
                let pos = item.resume_position_ticks().unwrap_or(0);
                let total = item.runtime_ticks.unwrap_or(0);
                let bar = if pos > 0 && total > 0 {
                    let pct = (pos as f64 / total as f64 * 100.0) as u32;
                    format!(" {}%", pct)
                } else {
                    String::new()
                };

                let indent = if selected { "  ▸ " } else { "    " };
                let mut spans = vec![
                    Span::raw(indent),
                    Span::styled(star, Style::default().fg(theme.warning)),
                    Span::styled(format!("{name}{dur}"), style),
                ];
                if !bar.is_empty() {
                    spans.push(Span::styled(bar, Style::default().fg(theme.muted)));
                }
                ListItem::new(Line::from(spans))
            }
        })
        .collect();

    let list = List::new(items)
        .block(rounded_block().title(format!(" {} ", t("section.home"))))
        .highlight_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .highlight_symbol("");

    let mut state_list = ListState::default();
    state_list.select(Some(state.selected));
    f.render_stateful_widget(list, area, &mut state_list);
}

fn render_libraries(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
    let mut items: Vec<ListItem> = Vec::new();

    // Libraries header (selectable at index 0)
    let selected = state.selected == 0;
    let prefix = if selected { "▸ " } else { "  " };
    let style = if selected {
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.accent)
    };
    items.push(ListItem::new(Line::from(vec![
        Span::raw(prefix),
        Span::styled(t("section.libraries"), style),
    ])));

    // Library items (selectable from index 1)
    for (i, lib) in state.libraries.iter().enumerate() {
        let icon = match lib.collection_type.as_deref() {
            Some("movies") => " ",
            Some("tvshows") => " ",
            Some("music") => " ",
            Some("books") => " ",
            _ => " ",
        };
        let selected = state.selected == i + 1;
        let prefix = if selected { "  ▸" } else { "   " };
        let style = if selected {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        items.push(ListItem::new(Line::from(vec![
            Span::raw(prefix),
            Span::styled(icon, Style::default().fg(theme.accent)),
            Span::styled(&lib.name, style),
        ])));
    }

    // Latest items sections
    let mut idx = 1 + state.libraries.len();
    for (lib_name, latest_items) in &state.library_latest {
        let selected = state.selected == idx;
        let prefix = if selected { "▸ " } else { "  " };
        let style = if selected {
            Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.warning)
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
            let selected = state.selected == idx;
            let prefix = if selected { "  ▸" } else { "   " };
            let style = if selected {
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            items.push(ListItem::new(Line::from(vec![
                Span::raw(prefix),
                Span::styled(icon, Style::default().fg(theme.accent)),
                Span::styled(name, style),
                Span::styled(dur, Style::default().fg(theme.muted)),
            ])));
            idx += 1;
        }
    }

    let list = List::new(items)
        .block(rounded_block().title(format!(" {} ", t("section.libraries"))));

    let mut state_list = ListState::default();
    state_list.select(Some(state.selected));
    f.render_stateful_widget(list, area, &mut state_list);
}

fn render_items(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
    let items_source = match state.view {
        View::Items => &state.items,
        View::SearchResults => &state.search_results,
        View::Favorites => &state.favorites,
        _ => &state.items,
    };
    let title = match state.view {
        View::SearchResults => t("title.search").to_string(),
        View::Favorites => {
            let fav_count = state.favorites.iter()
                .filter(|i| i.user_data.as_ref().map(|ud| ud.is_favorite).unwrap_or(false))
                .count();
            let follow_count = state.favorites.len() - fav_count;
            if follow_count > 0 {
                    format!("{} (★ {} ⊕ {})", t("title.favorites"), fav_count, follow_count)
            } else {
                format!("{} ({})", t("title.favorites"), fav_count)
            }
        }
        _ => "Items".to_string(),
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
                ("", "⊕ ")
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
                Span::styled(star, Style::default().fg(theme.warning)),
                Span::styled(follow_mark.to_string(), Style::default().fg(theme.success)),
                Span::raw(name),
                Span::styled(dur, Style::default().fg(theme.muted)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            rounded_block().title(
                if state.view == View::Favorites {
                    format!(" {} ", title)
                } else {
                    format!(" {} ({}) ", title, items_source.len())
                }
            ),
        )
        .highlight_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .highlight_symbol("▸ ");

    let mut state_list = ListState::default();
    state_list.select(Some(state.selected));
    f.render_stateful_widget(list, area, &mut state_list);
}

fn render_source_select(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
    let ss = &state.source_state;
    let item_name = ss.item.as_ref().map(|i| i.display_name()).unwrap_or_default();

    let block = rounded_block()
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            format!(" {item_name} - Select Source "),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
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
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
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

fn render_episodes(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
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
            Span::styled(dur, Style::default().fg(theme.muted)),
        ]))
    }).collect();

    let title = if state.total_episodes > state.episodes.len() {
        format!("{} - Episodes ({}/{})", state.series_name, state.episodes.len(), state.total_episodes)
    } else {
        format!("{} - Episodes ({})", state.series_name, state.episodes.len())
    };
    let list = List::new(items)
        .block(rounded_block().title(title))
        .highlight_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .highlight_symbol("▸ ");

    let mut state_list = ListState::default();
    state_list.select(Some(state.selected));
    f.render_stateful_widget(list, area, &mut state_list);
}

fn render_series_info(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
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
        .block(rounded_block().title(format!(" {}", t("section.overview"))))
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
        (t("section.seasons"), &ss.seasons, SeriesSection::Seasons),
        (t("section.episodes"), &ss.episodes, SeriesSection::Episodes),
        (t("section.similar"), &ss.similar, SeriesSection::Similar),
    ];

    for (i, (label, items, section)) in tabs.iter().enumerate() {
        let active = ss.section == *section;
        let border_color = if active { theme.accent } else { theme.muted };
        let title_color = if active { theme.accent } else { theme.muted };

        let tab_items: Vec<ListItem> = items.iter().map(|item| {
            let name = item.display_name();
            ListItem::new(Line::from(Span::raw(name)))
        }).collect();

        let list = List::new(tab_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border_color))
                    .title(Span::styled(
                        format!(" {} ({}) ", label, items.len()),
                        Style::default().fg(title_color).add_modifier(Modifier::BOLD),
                    ))
            )
            .highlight_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
            .highlight_symbol("▸ ");

        let mut state_list = ListState::default();
        if active {
            state_list.select(Some(state.selected));
        }
        f.render_widget(Clear, sections_layout[i]);
        f.render_stateful_widget(list, sections_layout[i], &mut state_list);
    }
}

fn display_width(s: &str) -> usize {
    s.chars().map(|c| c.width().unwrap_or(0)).sum()
}

fn render_media_info(f: &mut Frame, ps: &crate::app::PlayingState, area: Rect, theme: &crate::theme::Theme) {
    let mut lines: Vec<Line> = Vec::new();

    if let Some(ref source) = ps.media_source {
        let container = if source.container.is_empty() { "?" } else { &source.container };
        let size_str = if source.size > 1_073_741_824 {
            format!("{:.1} GB", source.size as f64 / 1_073_741_824.0)
        } else if source.size > 1_048_576 {
            format!("{:.0} MB", source.size as f64 / 1_048_576.0)
        } else {
            String::new()
        };

        let video_streams: Vec<_> = source.media_streams.iter().filter(|s| s.stream_type == "Video").collect();
        let audio_streams: Vec<_> = source.media_streams.iter().filter(|s| s.stream_type == "Audio").collect();
        let sub_streams: Vec<_> = source.media_streams.iter().filter(|s| s.stream_type == "Subtitle").collect();

        let video = video_streams.get(ps.selected_video).or_else(|| video_streams.first());
        let audio = audio_streams.get(ps.selected_audio).or_else(|| audio_streams.first());
        let sub = sub_streams.get(ps.selected_subtitle).or_else(|| sub_streams.first());

        let mut vid_codec = String::new();
        let mut vid_detail = String::new();
        if let Some(v) = video {
            let mut parts = Vec::new();
            parts.push(v.codec.to_uppercase());
            if let Some(ref profile) = v.profile {
                if profile != "Main" { parts.push(profile.clone()); }
            }
            if let Some(ref range) = v.video_range {
                if range != "SDR" { parts.push(range.to_uppercase()); }
            }
            vid_codec = parts.join(" ");

            let mut details = Vec::new();
            if let (Some(w), Some(h)) = (v.width, v.height) {
                details.push(format!("{}x{}", w, h));
            }
            if let Some(fps) = v.avg_frame_rate {
                details.push(format!("{}fps", fps as i32));
            }
            if let Some(depth) = v.bit_depth {
                if depth > 8 { details.push(format!("{}bit", depth)); }
            }
            vid_detail = details.join(" ");
        }

        let mut aud_info = String::new();
        if let Some(a) = audio {
            let codec = a.codec.to_uppercase();
            let layout = a.channel_layout.as_deref().unwrap_or("");
            let lang = if a.language.is_empty() { "" } else { &a.language };
            aud_info = codec;
            if !layout.is_empty() { aud_info = format!("{} {}", aud_info, layout); }
            if !lang.is_empty() { aud_info = format!("{} ({})", aud_info, lang); }
        }

        let mut sub_info = String::new();
        if let Some(s) = sub {
            let codec = s.codec.to_uppercase();
            let lang = if s.language.is_empty() { "" } else { &s.language };
            sub_info = codec;
            if !lang.is_empty() { sub_info = format!("{} ({})", sub_info, lang); }
        }

        let dg = theme.muted;
        let wc = theme.text;
        let lw = 14;
        let vw = 18;

        lines.push(Line::from(vec![
            Span::styled(format!("  {:<w$}", "Container:", w = lw), Style::default().fg(dg)),
            Span::styled(format!("{:<w$}", container.to_uppercase(), w = vw), Style::default().fg(wc)),
            Span::styled(format!("{:<w$}", "Size:", w = lw), Style::default().fg(dg)),
            Span::styled(format!("{:<w$}", size_str, w = vw), Style::default().fg(wc)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<w$}", "Video:", w = lw), Style::default().fg(dg)),
            Span::styled(format!("{:<w$}", vid_codec, w = vw), Style::default().fg(wc)),
            Span::styled(format!("{:<w$}", "Resolution:", w = lw), Style::default().fg(dg)),
            Span::styled(format!("{:<w$}", vid_detail, w = vw), Style::default().fg(wc)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<w$}", "Audio:", w = lw), Style::default().fg(dg)),
            Span::styled(format!("{:<w$}", aud_info, w = vw), Style::default().fg(wc)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<w$}", "Subtitle:", w = lw), Style::default().fg(dg)),
            Span::styled(format!("{:<w$}", sub_info, w = vw), Style::default().fg(wc)),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled(format!("  {}:  ", t("track.video")), Style::default().fg(theme.muted)),
            Span::raw(&ps.video_track),
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("  {}:  ", t("track.audio")), Style::default().fg(theme.muted)),
            Span::raw(&ps.audio_track),
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("  {}:    ", t("track.sub")), Style::default().fg(theme.muted)),
            Span::raw(&ps.subtitle_track),
        ]));
    }

    f.render_widget(Clear, area);
    f.render_widget(Paragraph::new(lines), area);
}

fn render_playing(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
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
                Constraint::Length(5),
                Constraint::Length(4),
                Constraint::Min(1),
            ])
            .split(halves[0])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(5),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(halves[0])
    };

    // Title
    let title = Paragraph::new(Span::styled(
        &ps.item_name,
        Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
    )).alignment(Alignment::Center);
    f.render_widget(Clear, top[0]);
    f.render_widget(title, top[0]);

    // Media info
    render_media_info(f, ps, top[2], theme);

    // Playing indicator
    if ps.playing {
        let spinner = ["\u{28f8}", "\u{28fd}", "\u{28fb}", "\u{28bf}", "\u{28ff}", "\u{28fe}", "\u{287f}", "\u{287b}"];
        let idx = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() / 100) as usize % spinner.len();
        let playing_text = Paragraph::new(Span::styled(
            format!("{} {}", spinner[idx], t("playing.in_mpv")),
            Style::default().fg(theme.accent),
        )).alignment(Alignment::Center);
        f.render_widget(Clear, top[3]);
        f.render_widget(playing_text, top[3]);
    }

    // Resume choice
    if has_resume {
        let ticks = ps.resume_position.unwrap();
        let secs = ticks / 10_000_000;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        let resume_time = format!("{}:{:02}", m, s);

        let prompt = format!("  {}", t("playing.choose_option"));
        let resume_label = format!("{} {}", t("playing.resume_from"), resume_time);
        let start_label = t("playing.play_from_start").to_string();

        let marker = "\u{25b8} ";
        let dw_marker = display_width(marker);

        let opt1 = if ps.option_selected == 0 {
            format!("{}{}", marker, resume_label)
        } else {
            format!("{}{}", " ".repeat(dw_marker), resume_label)
        };
        let opt2 = if ps.option_selected == 1 {
            format!("{}{}", marker, start_label)
        } else {
            format!("{}{}", " ".repeat(dw_marker), start_label)
        };

        let dw_prompt = display_width(&prompt);
        let dw_opt1 = display_width(&opt1);
        let dw_opt2 = display_width(&opt2);
        let opt_maxdw = dw_opt1.max(dw_opt2);
        let maxdw = dw_prompt.max(opt_maxdw);

        let pad_right = |s: &str, dw: usize, target: usize| -> String {
            let pad = target.saturating_sub(dw);
            format!("{}{}", s, " ".repeat(pad))
        };

        let center = |s: &str, dw: usize| -> String {
            let pad = maxdw.saturating_sub(dw);
            let left = pad / 2;
            format!("{}{}", " ".repeat(left), s)
        };

        let opt1_padded = pad_right(&opt1, dw_opt1, opt_maxdw);
        let opt2_padded = pad_right(&opt2, dw_opt2, opt_maxdw);

        let options = vec![
            Line::from(Span::styled(
                center(&prompt, dw_prompt),
                Style::default().fg(theme.warning),
            )),
            Line::from(""),
            Line::from(Span::styled(
                center(&opt1_padded, opt_maxdw),
                if ps.option_selected == 0 {
                    Style::default().fg(theme.accent)
                } else {
                    Style::default()
                },
            )),
            Line::from(Span::styled(
                center(&opt2_padded, opt_maxdw),
                if ps.option_selected == 1 {
                    Style::default().fg(theme.accent)
                } else {
                    Style::default()
                },
            )),
        ];
        let options_widget = Paragraph::new(options);
        f.render_widget(Clear, top[3]);
        f.render_widget(options_widget, top[3]);
    } else if !ps.playing {
        // Play button when no resume and not playing
        let play_text = format!("\u{25b8} {} {}", t("playing.play"), t("playing.press_enter"));
        let play_btn = Paragraph::new(Line::from(Span::styled(play_text, Style::default().fg(theme.success).add_modifier(Modifier::BOLD))))
            .alignment(Alignment::Center);
        f.render_widget(Clear, top[4]);
        f.render_widget(play_btn, top[4]);
    }

    // Bottom half: mpv output panel
    let output_area = halves[1];
    if !state.mpv_output.is_empty() {
        let output_len = state.mpv_output.len();
        let visible_height = output_area.height as usize;
        let max_scroll = output_len.saturating_sub(visible_height);
        let scroll = state.mpv_output_scroll.min(max_scroll);

        let end = state.mpv_output.len().saturating_sub(scroll);
        let start = end.saturating_sub(visible_height);
        let visible: Vec<Line> = state.mpv_output[start..end].iter().map(|(line, level)| {
            let style = match level.as_str() {
                "error" | "fatal" => Style::default().fg(theme.error),
                "warn" => Style::default().fg(theme.warning),
                "info" => Style::default().fg(theme.muted),
                _ => Style::default(),
            };
            Line::from(Span::styled(line.as_str(), style))
        }).collect();

        let title = format!(" {} ({} lines) ", t("section.mpv_output"), output_len);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.muted))
            .title(Span::styled(title, Style::default().fg(theme.muted)));
        let paragraph = Paragraph::new(visible).block(block);
        f.render_widget(Clear, output_area);
        f.render_widget(paragraph, output_area);
    } else {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.muted))
            .title(format!(" {} ", t("section.mpv_output")));
        f.render_widget(Clear, output_area);
        f.render_widget(block, output_area);
    }
}

fn render_settings(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
    let ss = &state.settings_state;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);

    // Libraries section
    let header_style = Style::default().fg(theme.accent).add_modifier(Modifier::BOLD);
    let name_col = 24;

    let mut items: Vec<ListItem> = Vec::new();

    let enabled_header = if ss.column == SettingsColumn::Enabled { format!(">{}", t("settings.enabled")) } else { format!(" {}", t("settings.enabled")) };
    let latest_header = if ss.column == SettingsColumn::Latest { format!(">{}", t("settings.latest")) } else { format!(" {}", t("settings.latest")) };
    let header_name = format!("{:<width$}", format!(" {}", t("settings.library")), width = name_col);
    items.push(ListItem::new(Line::from(vec![
        Span::styled(header_name, header_style),
        Span::styled(
            enabled_header,
            if ss.column == SettingsColumn::Enabled { header_style } else { Style::default().fg(theme.muted) },
        ),
        Span::raw("  "),
        Span::styled(
            latest_header,
            if ss.column == SettingsColumn::Latest { header_style } else { Style::default().fg(theme.muted) },
        ),
    ])));

    for (i, lib) in ss.libraries.iter().enumerate() {
        let selected = i == ss.selected && ss.section == SettingsSection::Libraries;
        let name_style = if selected {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let enabled_mark = if lib.enabled { "[x]" } else { "[ ]" };
        let latest_mark = if lib.fetch_latest { "[x]" } else { "[ ]" };

        let enabled_style = if selected && ss.column == SettingsColumn::Enabled {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else if lib.enabled {
            Style::default().fg(theme.success)
        } else {
            Style::default().fg(theme.muted)
        };

        let latest_style = if selected && ss.column == SettingsColumn::Latest {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else if lib.fetch_latest {
            Style::default().fg(theme.success)
        } else {
            Style::default().fg(theme.muted)
        };

        let marker = if selected { ">" } else { " " };
        let name_with_marker = format!("{}{}", marker, lib.name);
        let dw = display_width(&name_with_marker);
        let padded_name = if dw < name_col {
            format!("{}{}", name_with_marker, " ".repeat(name_col - dw))
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

    let lib_border = if ss.section == SettingsSection::Libraries { theme.accent } else { theme.muted };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(lib_border))
                .title(Span::styled(
                    format!(" {} ", t("settings.library_prefs")),
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ))
                .title_alignment(Alignment::Center),
        );

    f.render_widget(Clear, area);
    f.render_widget(list, layout[0]);

    // MPV path section
    let mpv_active = ss.section == SettingsSection::MpvPath;
    let mpv_border = if mpv_active { theme.accent } else { theme.muted };
    let cursor = if mpv_active { "█" } else { "" };
    let mpv_text = format!("  {}: {}{}", t("settings.mpv_path"), ss.mpv_path, cursor);
    let mpv_style = if mpv_active {
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let mpv_block = Paragraph::new(Span::styled(mpv_text, mpv_style))
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(mpv_border)).title(format!(" {} ", t("section.mpv"))));
    f.render_widget(mpv_block, layout[1]);

    // Language section
    let lang_active = ss.section == SettingsSection::Language;
    let lang_border = if lang_active { theme.accent } else { theme.muted };
    let lang_label = if ss.language == "zh" { "中文" } else { "English" };
    let lang_text = format!("  Language: {}", lang_label);
    let lang_style = if lang_active {
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let lang_hint = if lang_active { "  (←/→ to switch)" } else { "" };
    let lang_block = Paragraph::new(Line::from(vec![
        Span::styled(lang_text, lang_style),
        Span::styled(lang_hint.to_string(), Style::default().fg(theme.muted)),
    ]))
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(lang_border)).title(" Language "));
    f.render_widget(lang_block, layout[2]);

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
}

fn render_help(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
    let view_name = match state.help_state.previous_view {
        crate::app::View::Home => "Home",
        crate::app::View::Libraries => "Libraries",
        crate::app::View::Items => "Items",
        crate::app::View::SearchResults => "Items",
        crate::app::View::Episodes => "Episodes",
        crate::app::View::SeriesInfo => "SeriesInfo",
        crate::app::View::Playing => "Playing",
        crate::app::View::LibraryBrowser => "LibraryBrowser",
        crate::app::View::Favorites => "Favorites",
        crate::app::View::Settings => "Settings",
        crate::app::View::ContinueWatching | crate::app::View::LatestItems => "Home",
        _ => "Home",
    };

    let bindings = crate::help::bindings_for_view(view_name);
    let label_key = crate::help::view_label_key(view_name);
    let label = t(label_key);

    let items: Vec<ListItem> = bindings.iter().map(|b| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("  {:<16}", b.keys), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw(t(b.description)),
        ]))
    }).collect();

    let height = (items.len() + 4) as u16;
    let popup = centered_rect(50, height, area);

    let block = rounded_block()
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            format!(" Help: {} ", label),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    f.render_widget(Clear, popup);
    f.render_widget(List::new(items).block(block), popup);
}

fn render_footer(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
    let help = match state.view {
        View::Home => t("footer.home"),
        View::ContinueWatching | View::LatestItems => t("footer.continue_watching"),
        View::Libraries => t("footer.libraries"),
        View::Items => t("footer.items"),
        View::SearchResults => t("footer.search_results"),
        View::TrackSelect => t("footer.track_select"),
        View::SourceSelect => t("footer.source_select"),
        View::Episodes => t("footer.episodes"),
        View::SeriesInfo => t("footer.series_info"),
        View::Playing => {
            if state.playing_state.playing {
                ""
            } else if state.playing_state.resume_position.is_some() {
                "↑/↓: select | Enter: confirm"
            } else {
                "Enter: play"
            }
        }
        View::Settings => t("footer.settings"),
        View::LibraryBrowser => {
            if state.library_browser_state.panel == BrowserPanel::Filter {
                t("footer.filter_panel")
            } else if state.library_browser_state.panel != BrowserPanel::None {
                t("footer.sort_panel")
            } else {
                t("footer.library_browser")
            }
        },
        View::Favorites => t("footer.favorites"),
        View::AccountManager => t("footer.account_manager"),
        View::Wizard => t("footer.wizard"),
        View::MpvPrompt => t("footer.mpv_prompt"),
        View::Help => "",
    };
    let help = if state.searching {
        t("footer.search")
    } else {
        help
    };
    let line = Line::from(Span::styled(help, Style::default().fg(theme.muted)))
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

fn render_track_select(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
    let ts = &state.track_state;
    let item_name = ts.item.as_ref().map(|i| i.name.as_str()).unwrap_or("");

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    let title_block = rounded_block()
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            format!(" {item_name} "),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
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

    render_track_section(f, state, sections_layout[0], t("section.video"), &ts.video_tracks, ts.selected_video, &ts.section, TrackSection::Video, theme);
    render_track_section(f, state, sections_layout[1], t("section.audio"), &ts.audio_tracks, ts.selected_audio, &ts.section, TrackSection::Audio, theme);
    render_track_section(f, state, sections_layout[2], t("section.subtitle"), &ts.subtitle_tracks, ts.selected_subtitle, &ts.section, TrackSection::Subtitle, theme);
}

fn render_track_section(
    f: &mut Frame, _state: &AppState, area: Rect,
    title: &str, tracks: &[crate::emby::MediaStream],
    selected: usize, current_section: &TrackSection, section: TrackSection,
    theme: &crate::theme::Theme,
) {
    let active = *current_section == section;
    let border_color = if active { theme.accent } else { theme.muted };

    let items: Vec<ListItem> = tracks.iter().enumerate().map(|(i, track)| {
        let label = track_label(track);
        let marker = if i == selected { "▸ " } else { "  " };
        let style = if i == selected && active {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(Line::from(Span::styled(format!("{marker}{label}"), style)))
    }).collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    format!(" {title} ({}) ", tracks.len()),
                    Style::default().fg(if active { theme.accent } else { theme.muted }).add_modifier(Modifier::BOLD),
                ))
        );

    f.render_widget(Clear, area);
    f.render_widget(list, area);
}

fn render_library_browser(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
    let bs = &state.library_browser_state;

    let items: Vec<ListItem> = bs.items.iter().map(|item| {
        let name = item.display_name();
        let duration = item.duration_str().map(|d| format!(" ({})", d)).unwrap_or_default();
        ListItem::new(Line::from(Span::raw(format!("{}{}", name, duration))))
    }).collect();

    let list = List::new(items)
        .block(rounded_block().title(format!(" {} ", bs.library_name)))
        .highlight_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .highlight_symbol("▸ ");

    let mut state_list = ListState::default();
    state_list.select(Some(state.selected));
    f.render_stateful_widget(list, area, &mut state_list);

    match bs.panel {
        BrowserPanel::Sort => render_sort_panel(f, state, area, theme),
        BrowserPanel::Filter => render_filter_panel(f, state, area, theme),
        BrowserPanel::None => {}
    }
}

fn render_sort_panel(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
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
            Style::default().fg(theme.selection_fg).bg(theme.accent)
        } else if current {
            Style::default().fg(theme.accent)
        } else {
            Style::default()
        };

        let marker = if current { "● " } else { "  " };
        ListItem::new(Line::from(Span::styled(format!("{}{}", marker, opt), style)))
    }).collect();

    let list = List::new(items)
        .block(rounded_block().title(format!(" {} ", t("section.sort_by"))));

    let popup = centered_rect(30, 14, area);
    f.render_widget(Clear, popup);
    f.render_widget(list, popup);
}

fn render_filter_panel(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
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
        Style::default().fg(theme.muted),
    ))));

    match bs.filter_section {
        FilterSection::Genre => {
            for (i, genre) in bs.available_genres.iter().enumerate() {
                let selected = i == bs.panel_selected;
                let active = bs.filter_genre.as_ref() == Some(genre);

                let style = if selected {
                    Style::default().fg(theme.selection_fg).bg(theme.accent)
                } else if active {
                    Style::default().fg(theme.accent)
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
                    Style::default().fg(theme.selection_fg).bg(theme.accent),
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
                    Style::default().fg(theme.selection_fg).bg(theme.accent)
                } else if active {
                    Style::default().fg(theme.accent)
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
                    Style::default().fg(theme.selection_fg).bg(theme.accent),
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
                    Style::default().fg(theme.selection_fg).bg(theme.accent)
                } else if active {
                    Style::default().fg(theme.accent)
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
                    Style::default().fg(theme.selection_fg).bg(theme.accent),
                ))));
            } else {
                items.push(ListItem::new(Line::from(Span::raw("  → Year"))));
            }
        }
        FilterSection::Year => {
            let year_active = bs.filter_years.is_some() || bs.filter_year_field.is_some();
            let year_style = if bs.panel_selected == 0 {
                Style::default().fg(theme.selection_fg).bg(theme.accent)
            } else if year_active {
                Style::default().fg(theme.accent)
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
                    Style::default().fg(theme.selection_fg).bg(theme.accent)
                } else if active {
                    Style::default().fg(theme.accent)
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
        .block(rounded_block().title(format!(" {} ", t("section.filter"))))
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

fn render_account_manager(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
    let ams = &state.account_manager_state;

    match &ams.action {
        crate::app::AccountManagerAction::View => {
            render_account_list(f, state, area, theme);
        }
        crate::app::AccountManagerAction::Add | crate::app::AccountManagerAction::Edit(_) => {
            render_account_form(f, state, area, theme);
        }
        crate::app::AccountManagerAction::Delete(_) => {
            render_account_list(f, state, area, theme);
            render_delete_confirm(f, state, area, theme);
        }
    }
}

fn render_account_list(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
    let ams = &state.account_manager_state;
    let mut items: Vec<ListItem> = Vec::new();

    for (i, acc) in ams.accounts.iter().enumerate() {
        let is_active = ams.last_account_id.as_ref() == Some(&acc.id);
        let marker = if i == ams.selected { "▸ " } else { "  " };
        let active_mark = if is_active { "● " } else { "  " };
        let style = if i == ams.selected {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default().fg(theme.success)
        } else {
            Style::default()
        };
        let label = format!("{}{}{} @ {}", marker, active_mark, acc.label, acc.server);
        items.push(ListItem::new(Line::from(Span::styled(label, style))));
    }

    let add_idx = ams.accounts.len();
    let add_marker = if add_idx == ams.selected { "▸ " } else { "  " };
    let add_style = if add_idx == ams.selected {
        Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.warning)
    };
    items.push(ListItem::new(Line::from(Span::styled(
        format!("{}{}", add_marker, t("account.add_new")),
        add_style,
    ))));

    let list = List::new(items)
        .block(rounded_block().title(format!(" {} ", t("title.account_manager"))))
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
            Paragraph::new(Span::styled(msg.as_str(), Style::default().fg(theme.success))),
            status_area,
        );
    }
}

fn render_account_form(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
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
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
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
        format!("  {}", t("account.form_hint")),
        Style::default().fg(theme.muted),
    ))));

    let list = List::new(items)
        .block(rounded_block().title(format!(" {} ", title)))
        .highlight_style(Style::default())
        .highlight_symbol("");

    let popup = centered_rect(60, 10, area);
    f.render_widget(Clear, popup);
    f.render_widget(list, popup);
}

fn render_delete_confirm(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
    let ams = &state.account_manager_state;
    if let crate::app::AccountManagerAction::Delete(idx) = &ams.action {
        let name = ams.accounts.get(*idx).map(|a| a.label.as_str()).unwrap_or("?");
        let text = vec![
            Line::from(Span::styled(
                tf("account.delete_confirm", name),
                Style::default().fg(theme.error).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                t("account.confirm_delete"),
                Style::default().fg(theme.muted),
            )),
        ];
        let popup = centered_rect(40, 6, area);
        f.render_widget(Clear, popup);
        f.render_widget(
            Paragraph::new(text)
                .block(rounded_block().title(format!(" {} ", t("section.confirm"))))
                .alignment(Alignment::Center),
            popup,
        );
    }
}

fn render_wizard(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
    let ws = &state.wizard_state;
    let mut items: Vec<ListItem> = Vec::new();
    items.push(ListItem::new(Line::from(Span::styled(
        format!("  {}", t("wizard.welcome")),
        Style::default().fg(theme.warning),
    ))));
    items.push(ListItem::new(Line::from(Span::raw(""))));

    // Language selection (special: toggle, not text input)
    {
        let active = ws.step == WizardField::Language;
        let marker = if active { "▸ " } else { "  " };
        let field_style = if active {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let lang_label = if ws.language == "zh" { "中文" } else { "English" };
        let toggle_hint = if active { "  (←/→ to switch)" } else { "" };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{}{:<14}", marker, "Language"), field_style),
            Span::raw(": "),
            Span::styled(lang_label, Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
            Span::styled(toggle_hint.to_string(), Style::default().fg(theme.muted)),
        ])));
    }

    // Text input fields
    let fields = [
        (t("wizard.server"), &ws.server, WizardField::Server),
        (t("wizard.username"), &ws.username, WizardField::Username),
        (t("wizard.password"), &ws.password, WizardField::Password),
        (t("wizard.mpv_path"), &ws.mpv_path, WizardField::MpvPath),
    ];
    for (label, value, field) in fields.iter() {
        let active = ws.step == *field;
        let marker = if active { "▸ " } else { "  " };
        let field_style = if active {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
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
            format!("  {}", t("wizard.skip_hint"))
        } else {
            String::new()
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{}{:<14}", marker, label), field_style),
            Span::raw(": "),
            Span::raw(format!("{}{}", display_value, cursor)),
            Span::styled(hint.to_string(), Style::default().fg(theme.muted)),
        ])));
    }
    if let Some(ref msg) = ws.status_msg {
        items.push(ListItem::new(Line::from(Span::styled(
            format!("  {}", msg),
            Style::default().fg(theme.error),
        ))));
    }
    items.push(ListItem::new(Line::from(Span::raw(""))));
    items.push(ListItem::new(Line::from(Span::styled(
        format!("  {}", t("wizard.hint")),
        Style::default().fg(theme.muted),
    ))));
    let list = List::new(items)
        .block(rounded_block().title(format!(" {} ", t("title.wizard"))))
        .highlight_style(Style::default())
        .highlight_symbol("");
    let popup = centered_rect(60, 14, area);
    f.render_widget(Clear, popup);
    f.render_widget(list, popup);
}

fn render_mpv_prompt(f: &mut Frame, state: &AppState, area: Rect, theme: &crate::theme::Theme) {
    let ms = &state.mpv_prompt_state;
    let items: Vec<ListItem> = vec![
        ListItem::new(Line::from(Span::styled(
            format!("  {}", t("mpv_prompt.message")),
            Style::default().fg(theme.warning),
        ))),
        ListItem::new(Line::from(Span::raw(""))),
        ListItem::new(Line::from(vec![
            Span::styled(format!("  {}: ", t("settings.mpv_path")), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw(&ms.mpv_path),
            Span::raw("\u{2588}"),
        ])),
        ListItem::new(Line::from(Span::raw(""))),
        ListItem::new(Line::from(Span::styled(
            format!("  {}", t("mpv_prompt.hint")),
            Style::default().fg(theme.muted),
        ))),
    ];
    let list = List::new(items)
        .block(rounded_block().title(format!(" {} ", t("title.mpv_prompt"))))
        .highlight_style(Style::default())
        .highlight_symbol("");
    let popup = centered_rect(50, 8, area);
    f.render_widget(Clear, popup);
    f.render_widget(list, popup);
}
