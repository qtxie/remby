mod app;
mod config;
mod emby;
mod mpv;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;

const SPINNER: [&str; 8] = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];

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

fn update_favorite_in_list(items: &mut [crate::emby::MediaItem], item_id: &str, is_favorite: bool) {
    for item in items.iter_mut() {
        if item.id == item_id {
            if let Some(ref mut ud) = item.user_data {
                ud.is_favorite = is_favorite;
            } else {
                let mut ud = crate::emby::UserData::default();
                ud.is_favorite = is_favorite;
                item.user_data = Some(ud);
            }
        }
    }
}

enum BackgroundResult {
    HomeLoaded(Vec<crate::emby::MediaItem>),
    LibrariesLoaded(Vec<crate::emby::Library>, Vec<(String, Vec<crate::emby::MediaItem>)>),
    SettingsLoaded(Vec<crate::emby::Library>),
    SeriesInfoLoaded(app::SeriesState),
    EpisodesLoaded(String, Vec<crate::emby::MediaItem>, usize, String),
    MoreEpisodesLoaded(Vec<crate::emby::MediaItem>),
    FolderLoaded(Vec<crate::emby::MediaItem>, String, usize),
    MoreItemsLoaded(Vec<crate::emby::MediaItem>, String),
    SearchLoaded(Vec<crate::emby::MediaItem>),
    ItemDetailLoaded(crate::emby::MediaItem),
    LibraryBrowserLoaded(Vec<crate::emby::MediaItem>, String, usize, Vec<String>, Vec<String>, Vec<String>, Vec<crate::emby::MediaItem>),
    MoreLibraryBrowserLoaded(Vec<crate::emby::MediaItem>, String),
    FavoritesLoaded(Vec<crate::emby::MediaItem>, usize),
    FavoriteToggled(String, bool),
    Error(String),
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

    // Splash screen with animation during connection
    // Start connection in background
    let server = cli.server.clone();
    let user = cli.user.clone();
    let pass = cli.pass.clone();
    let connect_task = tokio::spawn(async move {
        app::AppState::new(server, user, pass).await
    });

    // Animate while waiting for connection
    let mut state = None;
    for i in 0.. {
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

            let frame = SPINNER[i % SPINNER.len()];
            f.render_widget(
                Paragraph::new(Span::styled(
                    format!("{} Connecting to server...", frame),
                    Style::default().fg(Color::Yellow),
                ))
                .alignment(Alignment::Center),
                vertical[2],
            );
        })?;

        if connect_task.is_finished() {
            state = Some(connect_task.await.expect("Task panicked"));
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let mut state = state.unwrap().unwrap_or_else(|e| {
        disable_raw_mode().ok();
        execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
        terminal.show_cursor().ok();
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    });

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
    let mut spin_idx: usize = 0;
    let (bg_tx, mut bg_rx) = mpsc::unbounded_channel::<BackgroundResult>();

    // Load home in background
    state.loading = true;
    state.loading_msg = "Loading home...".to_string();
    {
        let tx = bg_tx.clone();
        let client = state.client.clone();
        tokio::spawn(async move {
            let timeout = std::time::Duration::from_secs(120);
            let result = tokio::time::timeout(timeout, async {
                // Fetch resume and latest in parallel
                let (resume_result, latest_result) = tokio::join!(
                    client.get_resume_items(20),
                    client.get_latest_items(20),
                );

                let mut items = Vec::new();
                if let Ok(resume) = resume_result {
                    if !resume.is_empty() {
                        items.push(crate::emby::MediaItem::separator("Continue Watching"));
                        items.extend(resume);
                    }
                }
                if let Ok(latest) = latest_result {
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
                    state.libraries_fetched_at = Some(std::time::Instant::now());
                    state.library_latest = latest;
                    state.library_latest_fetched_at = Some(std::time::Instant::now());
                    state.loading = false;
                    state.status_msg = Some(app::Message::info(format!("{} libraries", state.libraries.len())));
                }
                BackgroundResult::SettingsLoaded(libs) => {
                    state.libraries = libs;
                    state.open_settings();
                    state.loading = false;
                }
                BackgroundResult::SeriesInfoLoaded(ss) => {
                    state.navigate_to(app::View::SeriesInfo);
                    state.series_state = ss;
                    state.loading = false;
                }
                BackgroundResult::EpisodesLoaded(name, episodes, total, series_id) => {
                    state.navigate_to(app::View::Episodes);
                    state.series_name = name;
                    state.episodes = episodes;
                    state.total_episodes = total;
                    state.episodes_series_id = series_id;
                    state.status_msg = Some(app::Message::info(format!("{} / {} episodes", state.episodes.len(), total)));
                    state.loading = false;
                }
                BackgroundResult::MoreEpisodesLoaded(more_episodes) => {
                    state.episodes.extend(more_episodes);
                    state.status_msg = Some(app::Message::info(format!("{} / {} episodes", state.episodes.len(), state.total_episodes)));
                    state.loading = false;
                }
                BackgroundResult::FolderLoaded(items, folder_id, total) => {
                    state.navigate_to(app::View::Items);
                    state.items = items;
                    state.current_folder_id = folder_id;
                    state.total_items = total;
                    state.loading = false;
                }
                BackgroundResult::MoreItemsLoaded(more_items, _folder_id) => {
                    state.items.extend(more_items);
                    state.status_msg = Some(app::Message::info(format!("{} / {} items", state.items.len(), state.total_items)));
                    state.loading = false;
                }
                BackgroundResult::SearchLoaded(results) => {
                    state.search_results = results;
                    state.navigate_to(app::View::SearchResults);
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
                    state.status_msg = Some(app::Message::error(format!("{} timed out", task)));
                }
                BackgroundResult::Error(msg) => {
                    state.loading = false;
                    state.status_msg = Some(app::Message::error(msg));
                }
                BackgroundResult::LibraryBrowserLoaded(items, lib_id, total, genres, tags, studios, folders) => {
                    if state.library_browser_state.library_id == lib_id {
                        state.library_browser_state.items = items;
                        state.library_browser_state.total = total;
                        // Only update filter options if they are provided (non-empty)
                        if !genres.is_empty() {
                            state.library_browser_state.available_genres = genres;
                        }
                        if !tags.is_empty() {
                            state.library_browser_state.available_tags = tags;
                        }
                        if !studios.is_empty() {
                            state.library_browser_state.available_studios = studios;
                        }
                        if !folders.is_empty() {
                            state.library_browser_state.available_folders = folders;
                        }
                    }
                    state.loading = false;
                    state.status_msg = Some(app::Message::info(format!("{} / {} items", state.library_browser_state.items.len(), total)));
                }
                BackgroundResult::MoreLibraryBrowserLoaded(more_items, lib_id) => {
                    if state.library_browser_state.library_id == lib_id {
                        state.library_browser_state.items.extend(more_items);
                        let total = state.library_browser_state.total;
                        state.status_msg = Some(app::Message::info(format!("{} / {} items", state.library_browser_state.items.len(), total)));
                    }
                    state.loading = false;
                }
                BackgroundResult::FavoritesLoaded(items, total) => {
                    state.favorites = items;
                    state.total_favorites = total;
                    state.loading = false;
                    state.status_msg = Some(app::Message::info(format!("{} / {} favorites", state.favorites.len(), total)));
                }
                BackgroundResult::FavoriteToggled(item_id, is_favorite) => {
                    update_favorite_in_list(&mut state.home_items, &item_id, is_favorite);
                    update_favorite_in_list(&mut state.items, &item_id, is_favorite);
                    update_favorite_in_list(&mut state.library_browser_state.items, &item_id, is_favorite);
                    if !is_favorite {
                        state.favorites.retain(|item| item.id != item_id);
                        state.total_favorites = state.total_favorites.saturating_sub(1);
                    }
                    state.loading = false;
                    state.status_msg = Some(app::Message::success(if is_favorite { "Added to favorites" } else { "Removed from favorites" }));
                }
            }
        }

        // Update spinner
        if state.loading {
            state.status_msg = Some(app::Message::info(format!(
                "{} {}",
                SPINNER[spin_idx % SPINNER.len()],
                state.loading_msg
            )));
            spin_idx += 1;
        }
        terminal.draw(|f| ui::render(f, state))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Allow ESC to interrupt loading
                    if state.loading {
                        if key.code == KeyCode::Esc {
                            state.loading = false;
                            state.status_msg = None;
                        }
                        continue;
                    }

                    match state.view {
                        app::View::Settings => {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => state.settings_cancel(),
                                KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => state.settings_move_up(),
                                KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => state.settings_move_down(),
                                KeyCode::Up | KeyCode::Char('k') => state.settings_select_prev(),
                                KeyCode::Down | KeyCode::Char('j') => state.settings_select_next(),
                                KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => state.settings_switch_column(),
                                KeyCode::Char(' ') => state.settings_toggle(),
                                KeyCode::Char('K') => state.settings_move_up(),
                                KeyCode::Char('J') => state.settings_move_down(),
                                KeyCode::Enter => {
                                    state.settings_save();
                                    state.loading_msg = "Loading libraries...".to_string();
                                    let tx = bg_tx.clone();
                                    let client = state.client.clone();
                                    let config = state.config.clone();
                                    tokio::spawn(async move {
                                        let timeout = std::time::Duration::from_secs(120);
                                        let result = tokio::time::timeout(timeout, async {
                                            let all_libs = client.get_libraries().await.unwrap_or_default();
                                            let libs = config.filter_and_sort_libraries(all_libs);

                                            let futures: Vec<_> = libs.iter()
                                                .filter(|lib| config.latest_libraries.is_empty() || config.latest_libraries.contains(&lib.id))
                                                .map(|lib| {
                                                    let client = client.clone();
                                                    let lib_id = lib.id.clone();
                                                    let lib_name = lib.name.clone();
                                                    async move {
                                                        match client.get_latest_for_library(&lib_id, 10).await {
                                                            Ok(items) if !items.is_empty() => Some((lib_name, items)),
                                                            _ => None,
                                                        }
                                                    }
                                                })
                                                .collect();
                                            let results = futures::future::join_all(futures).await;
                                            let latest: Vec<_> = results.into_iter().flatten().collect();

                                            (libs, latest)
                                        }).await;
                                        match result {
                                            Ok((libs, latest)) => { let _ = tx.send(BackgroundResult::LibrariesLoaded(libs, latest)); }
                                            Err(_) => { let _ = tx.send(BackgroundResult::Timeout("Libraries".to_string())); }
                                        }
                                    });
                                }
                                _ => {}
                            }
                        }
                        app::View::SourceSelect => {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => state.go_back(),
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
                                KeyCode::Esc => state.go_back(),
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
                                    state.go_back();
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
                                KeyCode::Esc => state.go_back(),
                                KeyCode::Up | KeyCode::Char('k') => {
                                    state.select_prev();
                                    if state.should_load_more_episodes() {
                                        state.loading = true;
                                        state.loading_msg = format!("Loading more episodes for {}...", state.series_name);
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        let series_id = state.episodes_series_id.clone();
                                        let start = state.episodes.len();
                                        tokio::spawn(async move {
                                            match client.get_episodes_page(&series_id, start, 50).await {
                                                Ok(episodes) => { let _ = tx.send(BackgroundResult::MoreEpisodesLoaded(episodes)); }
                                                Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Failed to load episodes: {}", e))); }
                                            }
                                        });
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    state.select_next();
                                    if state.should_load_more_episodes() {
                                        state.loading = true;
                                        state.loading_msg = format!("Loading more episodes for {}...", state.series_name);
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        let series_id = state.episodes_series_id.clone();
                                        let start = state.episodes.len();
                                        tokio::spawn(async move {
                                            match client.get_episodes_page(&series_id, start, 50).await {
                                                Ok(episodes) => { let _ = tx.send(BackgroundResult::MoreEpisodesLoaded(episodes)); }
                                                Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Failed to load episodes: {}", e))); }
                                            }
                                        });
                                    }
                                }
                                KeyCode::Char('e') => {
                                    // Show series info for current series
                                    let series_id = state.episodes_series_id.clone();
                                    let tx = bg_tx.clone();
                                    let client = state.client.clone();
                                    tokio::spawn(async move {
                                        let mut item = crate::emby::MediaItem::separator("");
                                        item.id = series_id;
                                        let result = build_series_state(&client, &item).await;
                                        let _ = tx.send(BackgroundResult::SeriesInfoLoaded(result));
                                    });
                                }
                                KeyCode::Enter => {
                                    if let Some(item) = state.selected_item().cloned() {
                                        if item.is_video() {
                                            state.loading = true;
                                            state.loading_msg = format!("Loading {}...", item.display_name());
                                            let tx = bg_tx.clone();
                                            let client = state.client.clone();
                                            let item_id = item.id.clone();
                                            tokio::spawn(async move {
                                                match client.get_item_detail(&item_id).await {
                                                    Ok(detail) => { let _ = tx.send(BackgroundResult::ItemDetailLoaded(detail)); }
                                                    Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Failed to load item: {}", e))); }
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
                                KeyCode::Esc => state.go_back(),
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
                                                state.loading_msg = format!("Loading similar to {}...", item.display_name());
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
                                            state.loading_msg = format!("Loading episodes for {}...", series_name);
                                            let tx = bg_tx.clone();
                                            let client = state.client.clone();
                                            tokio::spawn(async move {
                                                let timeout = std::time::Duration::from_secs(60);
                                                match tokio::time::timeout(timeout, client.get_episodes(&series_id)).await {
                                                    Ok(Ok((episodes, total))) => { let _ = tx.send(BackgroundResult::EpisodesLoaded(series_name, episodes, total, series_id)); }
                                                    _ => { let _ = tx.send(BackgroundResult::Timeout("Episodes".to_string())); }
                                                }
                                            });
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        app::View::LibraryBrowser => {
                            let has_panel = state.library_browser_state.panel != app::BrowserPanel::None;

                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => {
                                    if has_panel {
                                        // Cancel - close without applying
                                        state.library_browser_close_panel();
                                    } else {
                                        state.go_back();
                                    }
                                }
                                KeyCode::Char('s') if !has_panel => {
                                    state.library_browser_open_sort_panel();
                                }
                                KeyCode::Char('Z') if !has_panel => {
                                    state.open_favorites();
                                    state.loading = true;
                                    state.loading_msg = "Loading favorites...".to_string();
                                    let tx = bg_tx.clone();
                                    let client = state.client.clone();
                                    tokio::spawn(async move {
                                        match client.get_favorites(0, 50).await {
                                            Ok(result) => { let _ = tx.send(BackgroundResult::FavoritesLoaded(result.items, result.total)); }
                                            Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Failed to load favorites: {}", e))); }
                                        }
                                    });
                                }
                                KeyCode::Char('z') if !has_panel => {
                                    if let Some(item) = state.selected_item().cloned() {
                                        let is_favorite = item.user_data.as_ref().map(|ud| ud.is_favorite).unwrap_or(false);
                                        let new_favorite = !is_favorite;
                                        let item_id = item.id.clone();
                                        state.loading = true;
                                        state.loading_msg = "Updating favorite...".to_string();
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        tokio::spawn(async move {
                                            let timeout = std::time::Duration::from_secs(30);
                                            match tokio::time::timeout(timeout, client.toggle_favorite(&item_id, new_favorite)).await {
                                                Ok(Ok(_)) => { let _ = tx.send(BackgroundResult::FavoriteToggled(item_id, new_favorite)); }
                                                Ok(Err(e)) => { let _ = tx.send(BackgroundResult::Error(format!("Favorite failed: {}", e))); }
                                                Err(_) => { let _ = tx.send(BackgroundResult::Timeout("Favorite".to_string())); }
                                            }
                                        });
                                    }
                                }
                                KeyCode::Char('f') if !has_panel => {
                                    state.library_browser_open_filter_panel();
                                }
                                KeyCode::Left | KeyCode::Char('h') if has_panel && state.library_browser_state.panel == app::BrowserPanel::Filter => {
                                    state.library_browser_filter_section_prev();
                                }
                                KeyCode::Right | KeyCode::Char('l') if has_panel && state.library_browser_state.panel == app::BrowserPanel::Filter => {
                                    state.library_browser_filter_section_next();
                                }
                                KeyCode::Char('c') if !has_panel => {
                                    state.library_browser_clear_filters();
                                    reload_library_items(state, &bg_tx, 0);
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if has_panel {
                                        state.library_browser_panel_prev();
                                    } else {
                                        state.select_prev();
                                        let bs = &state.library_browser_state;
                                        if !state.loading && bs.total > bs.items.len() && state.selected + 5 >= bs.items.len() * 2 / 3 {
                                            let start = bs.items.len();
                                            reload_library_items(state, &bg_tx, start);
                                        }
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if has_panel {
                                        state.library_browser_panel_next();
                                    } else {
                                        state.select_next();
                                        let bs = &state.library_browser_state;
                                        if !state.loading && bs.total > bs.items.len() && state.selected + 5 >= bs.items.len() * 2 / 3 {
                                            let start = bs.items.len();
                                            reload_library_items(state, &bg_tx, start);
                                        }
                                    }
                                }
                                KeyCode::PageUp => {
                                    if has_panel {
                                        for _ in 0..10 {
                                            state.library_browser_panel_prev();
                                        }
                                    } else {
                                        state.page_up();
                                    }
                                }
                                KeyCode::PageDown => {
                                    if has_panel {
                                        for _ in 0..10 {
                                            state.library_browser_panel_next();
                                        }
                                    } else {
                                        state.page_down();
                                        let bs = &state.library_browser_state;
                                        if !state.loading && bs.total > bs.items.len() && state.selected + 5 >= bs.items.len() * 2 / 3 {
                                            let start = bs.items.len();
                                            reload_library_items(state, &bg_tx, start);
                                        }
                                    }
                                }
                                KeyCode::Enter => {
                                    if has_panel {
                                        match state.library_browser_state.panel {
                                            app::BrowserPanel::Sort => {
                                                state.library_browser_select_sort();
                                                reload_library_items(state, &bg_tx, 0);
                                            }
                                            app::BrowserPanel::Filter => {
                                                if state.library_browser_state.filter_year_field.is_some() {
                                                    state.library_browser_year_confirm();
                                                } else {
                                                    state.library_browser_filter_select();
                                                }
                                                // Apply filters when panel closes
                                                if state.library_browser_state.panel == app::BrowserPanel::None {
                                                    reload_library_items(state, &bg_tx, 0);
                                                }
                                            }
                                            app::BrowserPanel::None => {}
                                        }
                                    } else {
                                        if let Some(item) = state.selected_item().cloned() {
                                            if item.is_video() {
                                                state.loading = true;
                                                state.loading_msg = format!("Loading {}...", item.display_name());
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
                                                state.loading_msg = format!("Loading {}...", item.display_name());
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
                                                        match client.get_items(&item_id, 0, 200).await {
                                                            Ok(result) => { let _ = tx.send(BackgroundResult::FolderLoaded(result.items, item_id, result.total)); }
                                                            Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Failed to load folder: {}", e))); }
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
                        _ => {
                            match key.code {
                                KeyCode::Char('q') if !state.searching => break,
                                KeyCode::Esc => {
                                    if state.searching {
                                        state.cancel_search();
                                    } else {
                                        state.go_back();
                                    }
                                }
                                KeyCode::Char(c) if state.searching => state.search_input(c),
                                KeyCode::Backspace if state.searching => state.search_backspace(),
                                KeyCode::Enter if state.searching => {
                                    let query = state.search_query.clone();
                                    state.loading = true;
                                    state.loading_msg = format!("Searching for \"{}\"...", query);
                                    let tx = bg_tx.clone();
                                    let client = state.client.clone();
                                    tokio::spawn(async move {
                                        match client.search(&query).await {
                                            Ok(results) => { let _ = tx.send(BackgroundResult::SearchLoaded(results)); }
                                            Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Search failed: {}", e))); }
                                        }
                                    });
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    state.select_prev();
                                    if state.should_load_more_items() {
                                        state.loading = true;
                                        state.loading_msg = "Loading more items...".to_string();
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        let folder_id = state.current_folder_id.clone();
                                        let start = state.items.len();
                                        tokio::spawn(async move {
                                            match client.get_items(&folder_id, start, 50).await {
                                                Ok(result) => { let _ = tx.send(BackgroundResult::MoreItemsLoaded(result.items, folder_id)); }
                                                Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Failed to load items: {}", e))); }
                                            }
                                        });
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    state.select_next();
                                    if state.should_load_more_items() {
                                        state.loading = true;
                                        state.loading_msg = "Loading more items...".to_string();
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        let folder_id = state.current_folder_id.clone();
                                        let start = state.items.len();
                                        tokio::spawn(async move {
                                            match client.get_items(&folder_id, start, 50).await {
                                                Ok(result) => { let _ = tx.send(BackgroundResult::MoreItemsLoaded(result.items, folder_id)); }
                                                Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Failed to load items: {}", e))); }
                                            }
                                        });
                                    }
                                }
                                KeyCode::PageUp => {
                                    state.page_up();
                                }
                                KeyCode::PageDown => {
                                    state.page_down();
                                    if state.should_load_more_items() {
                                        state.loading = true;
                                        state.loading_msg = "Loading more items...".to_string();
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        let folder_id = state.current_folder_id.clone();
                                        let start = state.items.len();
                                        tokio::spawn(async move {
                                            match client.get_items(&folder_id, start, 50).await {
                                                Ok(result) => { let _ = tx.send(BackgroundResult::MoreItemsLoaded(result.items, folder_id)); }
                                                Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Failed to load items: {}", e))); }
                                            }
                                        });
                                    }
                                }
                                KeyCode::Char('z') => {
                                    if let Some(item) = state.selected_item().cloned() {
                                        let is_favorite = item.user_data.as_ref().map(|ud| ud.is_favorite).unwrap_or(false);
                                        let new_favorite = !is_favorite;
                                        let item_id = item.id.clone();
                                        state.loading = true;
                                        state.loading_msg = "Updating favorite...".to_string();
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        tokio::spawn(async move {
                                            let timeout = std::time::Duration::from_secs(10);
                                            match tokio::time::timeout(timeout, client.toggle_favorite(&item_id, new_favorite)).await {
                                                Ok(Ok(_)) => { let _ = tx.send(BackgroundResult::FavoriteToggled(item_id, new_favorite)); }
                                                Ok(Err(e)) => { let _ = tx.send(BackgroundResult::Timeout(format!("Favorite: {}", e))); }
                                                Err(_) => { let _ = tx.send(BackgroundResult::Timeout("Favorite: timeout".to_string())); }
                                            }
                                        });
                                    }
                                }
                                KeyCode::Left | KeyCode::Char('h') => {
                                    state.go_back();
                                }
                                KeyCode::Right | KeyCode::Char('l') => {
                                    state.show_libraries().await;
                                    if !state.is_libraries_cache_valid() || !state.is_latest_cache_valid() {
                                        state.loading_msg = "Loading libraries...".to_string();
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        let config = state.config.clone();
                                        tokio::spawn(async move {
                                            let timeout = std::time::Duration::from_secs(120);
                                            let result = tokio::time::timeout(timeout, async {
                                                let all_libs = client.get_libraries().await.unwrap_or_default();
                                                let libs = config.filter_and_sort_libraries(all_libs);

                                                // Fetch latest for all eligible libraries in parallel
                                                let futures: Vec<_> = libs.iter()
                                                    .filter(|lib| config.latest_libraries.is_empty() || config.latest_libraries.contains(&lib.id))
                                                    .map(|lib| {
                                                        let client = client.clone();
                                                        let lib_id = lib.id.clone();
                                                        let lib_name = lib.name.clone();
                                                        async move {
                                                            match client.get_latest_for_library(&lib_id, 10).await {
                                                                Ok(items) if !items.is_empty() => Some((lib_name, items)),
                                                                _ => None,
                                                            }
                                                        }
                                                    })
                                                    .collect();
                                                let results = futures::future::join_all(futures).await;
                                                let latest: Vec<_> = results.into_iter().flatten().collect();

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
                                            state.loading_msg = format!("Loading {}...", item.display_name());
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
                                            state.loading_msg = format!("Loading {}...", item.display_name());
                                            let tx = bg_tx.clone();
                                            let client = state.client.clone();
                                            let item_id = item.id.clone();
                                            let item_type = item.item_type.clone();
                                            let series_id = item.series_id.clone();
                                            tokio::spawn(async move {
                                                if item_type == "Series" {
                                                    // For series, show series info with seasons
                                                    let series_id = series_id.unwrap_or(item_id);
                                                    let mut series_item = crate::emby::MediaItem::separator("");
                                                    series_item.id = series_id;
                                                    let result = build_series_state(&client, &series_item).await;
                                                    let _ = tx.send(BackgroundResult::SeriesInfoLoaded(result));
                                                } else {
                                                    // For folders/seasons, use regular items
                                                    match client.get_items(&item_id, 0, 200).await {
                                                        Ok(result) => { let _ = tx.send(BackgroundResult::FolderLoaded(result.items, item_id, result.total)); }
                                                        Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Failed to load folder: {}", e))); }
                                                    }
                                                }
                                            });
                                        }
                                    } else if let Some(lib) = state.selected_library().cloned() {
                                        state.open_library_browser(lib.id.clone(), lib.name.clone());
                                        state.loading = true;
                                        state.loading_msg = format!("Loading {}...", lib.name);
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        let library_id = lib.id.clone();
                                        let sort_by = "DateCreated".to_string();
                                        let sort_order = "Descending".to_string();
                                        tokio::spawn(async move {
                                            let timeout = std::time::Duration::from_secs(120);
                                            let result = tokio::time::timeout(timeout, async {
                                                let (items_result, genres_result, tags_result, studios_result, folders_result) = tokio::join!(
                                                    client.get_items_filtered(&library_id, 0, 50, &sort_by, &sort_order, None, None, None, None),
                                                    client.get_genres(&library_id),
                                                    client.get_tags(&library_id),
                                                    client.get_studios(&library_id),
                                                    client.get_folders(&library_id),
                                                );
                                                let items = items_result.unwrap_or_else(|_| crate::emby::PageResult { items: vec![], total: 0 });
                                                let genres = genres_result.unwrap_or_default();
                                                let tags = tags_result.unwrap_or_default();
                                                let studios = studios_result.unwrap_or_default();
                                                let folders = folders_result.unwrap_or_default();
                                                (items.items, library_id, items.total, genres, tags, studios, folders)
                                            }).await;
                                            match result {
                                                Ok((items, lib_id, total, genres, tags, studios, folders)) => {
                                                    let _ = tx.send(BackgroundResult::LibraryBrowserLoaded(items, lib_id, total, genres, tags, studios, folders));
                                                }
                                                Err(_) => { let _ = tx.send(BackgroundResult::Timeout("Library".to_string())); }
                                            }
                                        });
                                    }
                                }
                                KeyCode::Backspace => {
                                    state.go_back();
                                }
                                KeyCode::Char('/') => state.start_search(),
                                KeyCode::Char('Z') => {
                                    state.open_favorites();
                                    state.loading = true;
                                    state.loading_msg = "Loading favorites...".to_string();
                                    let tx = bg_tx.clone();
                                    let client = state.client.clone();
                                    tokio::spawn(async move {
                                        match client.get_favorites(0, 50).await {
                                            Ok(result) => { let _ = tx.send(BackgroundResult::FavoritesLoaded(result.items, result.total)); }
                                            Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Failed to load favorites: {}", e))); }
                                        }
                                    });
                                }
                                KeyCode::Char('s') => {
                                    if state.libraries.is_empty() {
                                        state.loading = true;
                                        state.loading_msg = "Loading libraries...".to_string();
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        tokio::spawn(async move {
                                            let libs = client.get_libraries().await.unwrap_or_default();
                                            let _ = tx.send(BackgroundResult::SettingsLoaded(libs));
                                        });
                                    } else {
                                        state.open_settings();
                                    }
                                }
                                KeyCode::Char('e') => {
                                    if let Some(item) = state.selected_item().cloned() {
                                        state.loading = true;
                                        state.loading_msg = format!("Loading {}...", item.display_name());
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

fn reload_library_items(state: &mut app::AppState, bg_tx: &mpsc::UnboundedSender<BackgroundResult>, start: usize) {
    let bs = &state.library_browser_state;
    let lib_id = bs.library_id.clone();
    let parent_id = bs.filter_folder.clone().unwrap_or_else(|| lib_id.clone());
    let sort_by = state.library_browser_sort_by_str().to_string();
    let sort_order = state.library_browser_sort_order_str().to_string();
    let genre = bs.filter_genre.clone();
    let tag = bs.filter_tag.clone();
    let studio = bs.filter_studio.clone();
    let years = bs.filter_years.map(|(s, e)| format!("{}-{}", s, e));
    state.loading = true;
    let tx = bg_tx.clone();
    let client = state.client.clone();
    tokio::spawn(async move {
        match client.get_items_filtered(&parent_id, start, 50, &sort_by, &sort_order, genre.as_deref(), tag.as_deref(), studio.as_deref(), years.as_deref()).await {
            Ok(result) => {
                if start > 0 {
                    let _ = tx.send(BackgroundResult::MoreLibraryBrowserLoaded(result.items, lib_id));
                } else {
                    let _ = tx.send(BackgroundResult::LibraryBrowserLoaded(result.items, lib_id, result.total, vec![], vec![], vec![], vec![]));
                }
            }
            Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Failed to load items: {}", e))); }
        }
    });
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
