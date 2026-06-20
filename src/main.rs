mod app;
mod emby;
mod mpv;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Parser)]
#[command(name = "remby", about = "Lightweight Emby client with mpv playback")]
struct Cli {
    #[arg(short, long, env = "EMBY_SERVER")]
    server: Option<String>,
    #[arg(short, long, env = "EMBY_TOKEN")]
    token: Option<String>,
    #[arg(short, long, env = "EMBY_USER")]
    user: Option<String>,
    #[arg(short = 'p', long, env = "EMBY_PASS", hide_env_values = true)]
    pass: Option<String>,
    #[arg(long, env = "MPV_PATH", default_value = "mpv")]
    mpv: String,
}

enum BackgroundResult {
    HomeLoaded(Vec<crate::emby::MediaItem>),
    LibrariesLoaded(Vec<crate::emby::Library>, Vec<(String, Vec<crate::emby::MediaItem>)>),
    SeriesInfoLoaded(app::SeriesState),
    EpisodesLoaded(String, Vec<crate::emby::MediaItem>),
    FolderLoaded(Vec<crate::emby::MediaItem>, String),
    SearchLoaded(Vec<crate::emby::MediaItem>),
    ItemDetailLoaded(crate::emby::MediaItem),
    Timeout(String),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Splash screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Animated splash screen
    let spinner = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
    for i in 0..15 {
        terminal.draw(|f| {
            let area = f.area();
            let vertical = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Length(10),
                    Constraint::Min(1),
                ])
                .split(area);

            let logo_lines = vec![
                Line::from("      ________       "),
                Line::from("   __/        \\__    "),
                Line::from("  /   ________   \\   "),
                Line::from(" |   /        \\   |  "),
                Line::from(" |  |   ▶▶▶▶   |  |  "),
                Line::from("  \\  \\________/  /   "),
                Line::from("   \\__        __/    "),
                Line::from("      \\______/       "),
            ];
            f.render_widget(Clear, area);
            f.render_widget(Paragraph::new(logo_lines).alignment(Alignment::Center), vertical[1]);

            let frame = spinner[i % spinner.len()];
            f.render_widget(
                Paragraph::new(Span::styled(
                    format!("{} Connecting to server...", frame),
                    Style::default().fg(Color::Yellow),
                ))
                .alignment(Alignment::Center),
                vertical[2],
            );
        })?;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Connect to server and authenticate
    let mut state = match app::AppState::new(cli.server, cli.user, cli.pass).await {
        Ok(s) => s,
        Err(e) => {
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;
            eprintln!("Error: {e:#}");
            std::process::exit(1);
        }
    };

    if state.server.is_empty() {
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        eprintln!("Usage: remby -s <server-url> -u <username> -p <password>");
        eprintln!("   or: remby -s <server-url> -t <api-token>");
        std::process::exit(1);
    }

    let result = run_app(&mut terminal, &mut state, &cli.mpv).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err:#}");
    }
    Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, state: &mut app::AppState, mpv_path: &str) -> Result<()> {
    let spinner = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
    let mut spin_idx: usize = 0;
    let (bg_tx, mut bg_rx) = mpsc::unbounded_channel::<BackgroundResult>();

    // Load home in background
    state.loading = true;
    {
        let tx = bg_tx.clone();
        let client = state.client.clone();
        tokio::spawn(async move {
            let timeout = std::time::Duration::from_secs(120);
            let result = tokio::time::timeout(timeout, async {
                let mut items = Vec::new();
                if let Ok(resume) = client.get_resume_items(20).await {
                    if !resume.is_empty() {
                        items.push(crate::emby::MediaItem::separator("Continue Watching"));
                        items.extend(resume);
                    }
                }
                if let Ok(latest) = client.get_latest_items(20).await {
                    if !latest.is_empty() {
                        items.push(crate::emby::MediaItem::separator("Latest"));
                        items.extend(latest);
                    }
                }
                items
            }).await;

            match result {
                Ok(items) => { let _ = tx.send(BackgroundResult::HomeLoaded(items)); }
                Err(_) => { let _ = tx.send(BackgroundResult::Timeout("Home page".to_string())); }
            }
        });
    }

    loop {
        // Handle background results first
        while let Ok(result) = bg_rx.try_recv() {
            match result {
                BackgroundResult::HomeLoaded(items) => {
                    state.home_items = items;
                    state.loading = false;
                }
                BackgroundResult::LibrariesLoaded(libs, latest) => {
                    state.libraries = libs;
                    state.library_latest = latest;
                    state.loading = false;
                    state.status_msg = format!("{} libraries", state.libraries.len());
                }
                BackgroundResult::SeriesInfoLoaded(ss) => {
                    state.series_state = ss;
                    state.view = app::View::SeriesInfo;
                    state.selected = 0;
                    state.loading = false;
                }
                BackgroundResult::EpisodesLoaded(name, episodes) => {
                    state.series_name = name;
                    state.episodes = episodes;
                    state.view = app::View::Episodes;
                    state.selected = 0;
                    state.loading = false;
                }
                BackgroundResult::FolderLoaded(items, folder_id) => {
                    state.items = items;
                    state.current_folder_id = folder_id;
                    state.view = app::View::Items;
                    state.selected = 0;
                    state.loading = false;
                }
                BackgroundResult::SearchLoaded(results) => {
                    state.search_results = results;
                    state.view = app::View::SearchResults;
                    state.selected = 0;
                    state.loading = false;
                }
                BackgroundResult::ItemDetailLoaded(detail) => {
                    state.loading = false;
                    if detail.media_sources.len() > 1 {
                        state.open_source_select(&detail, detail.media_sources.clone());
                    } else if let Some(source) = detail.media_sources.first() {
                        state.open_track_select(&detail, source);
                    } else {
                        let url = state.client.stream_url_for_source(&detail, &Default::default());
                        if let Ok(child) = mpv::play(&url, mpv_path, None, None, None, None) {
                            state.mpv_child = Some(child);
                        }
                    }
                }
                BackgroundResult::Timeout(task) => {
                    state.loading = false;
                    state.status_msg = format!("{} timed out", task);
                }
            }
        }

        // Update spinner
        if state.loading {
            state.status_msg = format!("{} Loading", spinner[spin_idx % spinner.len()]);
            spin_idx += 1;
        } else {
            state.status_msg.clear();
        }
        terminal.draw(|f| ui::render(f, state))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Don't handle keys while loading
                    if state.loading {
                        continue;
                    }

                    match state.view {
                        app::View::SourceSelect => {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => state.go_back().await?,
                                KeyCode::Up | KeyCode::Char('k') => state.select_prev(),
                                KeyCode::Down | KeyCode::Char('j') => state.select_next(),
                                KeyCode::Enter => {
                                    if let Some(item) = state.source_state.item.clone() {
                                        if let Some(source) = state.selected_source().cloned() {
                                            state.open_track_select(&item, &source);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        app::View::TrackSelect => {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => state.go_back().await?,
                                KeyCode::Left | KeyCode::Char('h') => state.track_section_prev(),
                                KeyCode::Right | KeyCode::Char('l') => state.track_section_next(),
                                KeyCode::Up | KeyCode::Char('k') => state.track_select_prev(),
                                KeyCode::Down | KeyCode::Char('j') => state.track_select_next(),
                                KeyCode::Enter => {
                                    if let Some(ref item) = state.track_state.item.clone() {
                                        if let Some(ref source) = state.track_state.media_source {
                                            let url = state.client.stream_url_for_source(item, source);
                                            let video_label = state.track_state.video_tracks
                                                .get(state.track_state.selected_video)
                                                .map(|t| ui::track_label(t))
                                                .unwrap_or_else(|| "Default".to_string());
                                            let audio_label = state.track_state.audio_tracks
                                                .get(state.track_state.selected_audio)
                                                .map(|t| ui::track_label(t))
                                                .unwrap_or_else(|| "Default".to_string());
                                            let sub_label = state.track_state.subtitle_tracks
                                                .get(state.track_state.selected_subtitle)
                                                .map(|t| ui::track_label(t))
                                                .unwrap_or_else(|| "Off".to_string());
                                            let resume_ticks = item.resume_position_ticks();
                                            state.open_playing(&item.display_name(), &url, &video_label, &audio_label, &sub_label, resume_ticks);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        app::View::Playing => {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => {
                                    state.kill_mpv();
                                    state.go_back().await?;
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if state.playing_state.resume_position.is_some() {
                                        state.playing_state.option_selected = 0;
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if state.playing_state.resume_position.is_some() {
                                        state.playing_state.option_selected = 1;
                                    }
                                }
                                KeyCode::Enter => {
                                    let ps = &state.playing_state;
                                    let start_secs = if ps.resume_position.is_some() && ps.option_selected == 0 {
                                        ps.resume_position.map(|t| t as f64 / 10_000_000.0)
                                    } else {
                                        None
                                    };
                                    if let Ok(child) = mpv::play(&ps.url, mpv_path, None, None, None, start_secs) {
                                        state.mpv_child = Some(child);
                                        state.playing_state.playing = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                        app::View::Episodes => {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => state.go_back().await?,
                                KeyCode::Up | KeyCode::Char('k') => state.select_prev(),
                                KeyCode::Down | KeyCode::Char('j') => state.select_next(),
                                KeyCode::Enter => {
                                    if let Some(item) = state.selected_item().cloned() {
                                        if item.is_video() {
                                            state.loading = true;
                                            let tx = bg_tx.clone();
                                            let client = state.client.clone();
                                            let item_id = item.id.clone();
                                            tokio::spawn(async move {
                                                if let Ok(detail) = client.get_item_detail(&item_id).await {
                                                    let _ = tx.send(BackgroundResult::ItemDetailLoaded(detail));
                                                }
                                            });
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        app::View::SeriesInfo => {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => state.go_back().await?,
                                KeyCode::Left | KeyCode::Char('h') => state.series_section_prev(),
                                KeyCode::Right | KeyCode::Char('l') => state.series_section_next(),
                                KeyCode::Up | KeyCode::Char('k') => state.series_select_prev(),
                                KeyCode::Down | KeyCode::Char('j') => state.series_select_next(),
                                KeyCode::Enter => {
                                    match state.series_state.section {
                                        app::SeriesSection::Seasons => {
                                            state.select_season().await?;
                                        }
                                        app::SeriesSection::Episodes => {
                                            if let Some(item) = state.series_selected_item().cloned() {
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
                                                }
                                            }
                                        }
                                        app::SeriesSection::Similar => {
                                            if let Some(item) = state.series_selected_item().cloned() {
                                                state.loading = true;
                                                let tx = bg_tx.clone();
                                                let client = state.client.clone();
                                                let item_clone = item.clone();
                                                tokio::spawn(async move {
                                                    let result = build_series_state(&client, &item_clone).await;
                                                    let _ = tx.send(BackgroundResult::SeriesInfoLoaded(result));
                                                });
                                            }
                                        }
                                    }
                                }
                                KeyCode::Char('e') => {
                                    if let Some(item) = state.series_selected_item().cloned() {
                                        if item.is_video() {
                                            let series_id = item.series_id.clone().unwrap_or_else(|| item.id.clone());
                                            let series_name = item.series_name.clone().unwrap_or_default();
                                            state.loading = true;
                                            let tx = bg_tx.clone();
                                            let client = state.client.clone();
                                            tokio::spawn(async move {
                                                let timeout = std::time::Duration::from_secs(60);
                                                match tokio::time::timeout(timeout, client.get_episodes(&series_id)).await {
                                                    Ok(Ok(episodes)) => { let _ = tx.send(BackgroundResult::EpisodesLoaded(series_name, episodes)); }
                                                    _ => { let _ = tx.send(BackgroundResult::Timeout("Episodes".to_string())); }
                                                }
                                            });
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            match key.code {
                                KeyCode::Char('q') if !state.searching => break,
                                KeyCode::Esc => {
                                    if state.searching {
                                        state.cancel_search();
                                    } else {
                                        state.go_back().await?;
                                    }
                                }
                                KeyCode::Char(c) if state.searching => state.search_input(c),
                                KeyCode::Backspace if state.searching => state.search_backspace(),
                                KeyCode::Enter if state.searching => {
                                    let query = state.search_query.clone();
                                    state.loading = true;
                                    let tx = bg_tx.clone();
                                    let client = state.client.clone();
                                    tokio::spawn(async move {
                                        if let Ok(results) = client.search(&query).await {
                                            let _ = tx.send(BackgroundResult::SearchLoaded(results));
                                        }
                                    });
                                }
                                KeyCode::Up | KeyCode::Char('k') => state.select_prev(),
                                KeyCode::Down | KeyCode::Char('j') => state.select_next(),
                                KeyCode::Left | KeyCode::Char('h') => {
                                    state.go_back().await?;
                                }
                                KeyCode::Right | KeyCode::Char('l') => {
                                    state.show_libraries().await;
                                    if state.library_latest.is_empty() {
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        tokio::spawn(async move {
                                            let timeout = std::time::Duration::from_secs(120);
                                            let result = tokio::time::timeout(timeout, async {
                                                let libs = client.get_libraries().await.unwrap_or_default();
                                                let mut latest = Vec::new();
                                                for lib in &libs {
                                                    if let Ok(items) = client.get_latest_for_library(&lib.id, 10).await {
                                                        if !items.is_empty() {
                                                            latest.push((lib.name.clone(), items));
                                                        }
                                                    }
                                                }
                                                (libs, latest)
                                            }).await;
                                            match result {
                                                Ok((libs, latest)) => { let _ = tx.send(BackgroundResult::LibrariesLoaded(libs, latest)); }
                                                Err(_) => { let _ = tx.send(BackgroundResult::Timeout("Libraries".to_string())); }
                                            }
                                        });
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Some(item) = state.selected_item().cloned() {
                                        if item.is_separator() {
                                            continue;
                                        }
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
                                            let folder_id = item.id.clone();
                                            tokio::spawn(async move {
                                                if let Ok(result) = client.get_items(&folder_id, 0, 50).await {
                                                    let _ = tx.send(BackgroundResult::FolderLoaded(result.items, folder_id));
                                                }
                                            });
                                        }
                                    } else if let Some(lib) = state.selected_library().cloned() {
                                        state.loading = true;
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        let folder_id = lib.id.clone();
                                        tokio::spawn(async move {
                                            if let Ok(result) = client.get_items(&folder_id, 0, 50).await {
                                                let _ = tx.send(BackgroundResult::FolderLoaded(result.items, folder_id));
                                            }
                                        });
                                    }
                                }
                                KeyCode::Backspace => {
                                    state.go_back().await?;
                                }
                                KeyCode::Char('/') => state.start_search(),
                                KeyCode::Char('e') => {
                                    if let Some(item) = state.selected_item().cloned() {
                                        state.loading = true;
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        tokio::spawn(async move {
                                            let result = build_series_state(&client, &item).await;
                                            let _ = tx.send(BackgroundResult::SeriesInfoLoaded(result));
                                        });
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

async fn build_series_state(client: &crate::emby::EmbyClient, item: &crate::emby::MediaItem) -> app::SeriesState {
    let series_id = item.series_id.as_deref().unwrap_or(&item.id);

    let (detail, seasons, similar) = tokio::join!(
        client.get_item_detail(series_id),
        client.get_seasons(series_id),
        client.get_similar(series_id),
    );

    let overview = match &detail {
        Ok(i) => i.overview.clone().unwrap_or_default(),
        Err(_) => String::new(),
    };

    let mut episodes = Vec::new();
    if let Ok(ref s) = seasons {
        if let Some(first) = s.first() {
            if let Ok(eps) = client.get_season_episodes(series_id, &first.id).await {
                episodes = eps;
            }
        }
    }

    app::SeriesState {
        item: detail.ok(),
        overview,
        seasons: seasons.unwrap_or_default(),
        episodes,
        similar: similar.unwrap_or_default(),
        selected_season: 0,
        selected_episode: 0,
        section: app::SeriesSection::Seasons,
    }
}
