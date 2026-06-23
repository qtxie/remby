mod app;
mod config;
mod crypto;
mod emby;
mod i18n;
mod mpv;
mod theme;
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

fn spawn_home_load(tx: mpsc::UnboundedSender<BackgroundResult>, client: crate::emby::EmbyClient) {
    tokio::spawn(async move {
        let timeout = std::time::Duration::from_secs(120);
        let result = tokio::time::timeout(timeout, async {
            let (resume_result, latest_result) = tokio::join!(
                client.get_resume_items(20),
                client.get_latest_items(20),
            );
            let mut items = Vec::new();
            if let Ok(resume) = resume_result {
                if !resume.is_empty() {
                    items.push(crate::emby::MediaItem::separator(t("title.continue_watching")));
                    items.extend(resume);
                }
            }
            if let Ok(latest) = latest_result {
                if !latest.is_empty() {
                    items.push(crate::emby::MediaItem::separator(t("title.latest")));
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

fn spawn_following_load(tx: mpsc::UnboundedSender<BackgroundResult>, client: crate::emby::EmbyClient, following: Vec<String>) {
    if following.is_empty() { return; }
    tokio::spawn(async move {
        let mut updates = Vec::new();
        for series_id in &following {
            if let Ok(episodes) = client.get_unwatched_episodes(series_id).await {
                if !episodes.is_empty() {
                    if let Ok(name) = client.get_item_name(series_id).await {
                        updates.push((name, episodes));
                    }
                }
            }
        }
        let _ = tx.send(BackgroundResult::FollowingUpdatesLoaded(updates));
    });
}

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
    #[arg(long, env = "MPV_PATH")]
    mpv: Option<String>,
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

use app::BackgroundResult;
use crate::i18n::{t, tf};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    crate::emby::init_device_id();

    // Initialize i18n
    let config = crate::config::load_config();
    crate::i18n::init(&config.language);

    // Splash screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Splash screen with animation during connection
    // Start connection in background
    let account = if cli.server.is_some() && cli.user.is_some() && cli.pass.is_some() {
        Some(crate::config::Account {
            id: String::new(),
            label: "CLI".to_string(),
            server: cli.server.clone().unwrap(),
            username: cli.user.clone().unwrap(),
            password_enc: crate::crypto::encrypt(&cli.pass.clone().unwrap()),
        })
    } else {
        let accounts_cfg = crate::config::load_accounts();
        accounts_cfg.last_account_id.and_then(|id| {
            accounts_cfg.accounts.into_iter().find(|a| a.id == id)
        })
    };
    let connect_task = tokio::spawn(async move {
        app::AppState::new(account).await
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
                    format!("{} {}", frame, t("status.connecting")),
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

    // Auto-save CLI args if login succeeded
    let cli_used = cli.server.is_some() && cli.user.is_some() && cli.pass.is_some();
    if cli_used && !state.server.is_empty() {
        let mut accounts_cfg = crate::config::load_accounts();
        let server = cli.server.clone().unwrap();
        let username = cli.user.clone().unwrap();
        let existing = accounts_cfg.accounts.iter().position(|a| a.server == server && a.username == username);
        let account_id = if let Some(idx) = existing {
            let id = accounts_cfg.accounts[idx].id.clone();
            accounts_cfg.accounts[idx].password_enc = crate::crypto::encrypt(&cli.pass.unwrap());
            id
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            accounts_cfg.accounts.push(crate::config::Account {
                id: id.clone(),
                label: format!("{}@{}", username, server),
                server,
                username,
                password_enc: crate::crypto::encrypt(&cli.pass.unwrap()),
            });
            id
        };
        accounts_cfg.last_account_id = Some(account_id);
        let _ = crate::config::save_accounts(&accounts_cfg);
        if let Some(ref mpv_path) = cli.mpv {
            state.config.mpv_path = mpv_path.clone();
            let _ = crate::config::save_config(&state.config);
        }
    }

    // Auto-detect mpv path if not configured
    if state.config.mpv_path == "mpv" {
        if let Some(detected) = mpv::find_mpv() {
            state.config.mpv_path = detected;
            let _ = crate::config::save_config(&state.config);
        }
    }

    if state.server.is_empty() {
        let accounts_cfg = crate::config::load_accounts();
        let has_config = std::path::Path::new(&dirs::config_dir().unwrap_or_default().join("remby").join("config.json")).exists();
        let has_accounts = !accounts_cfg.accounts.is_empty();
        if !has_config && !has_accounts {
            state.open_wizard();
        } else {
            state.open_account_manager();
        }
    }

    let result = run_app(&mut terminal, &mut state).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err:#}");
    }
    Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, state: &mut app::AppState) -> Result<()> {
    let mut spin_idx: usize = 0;
    let (bg_tx, mut bg_rx) = mpsc::unbounded_channel::<BackgroundResult>();
    state.bg_tx = Some(bg_tx.clone());

    // Load home in background
    state.loading = true;
    state.loading_msg = t("status.loading_home").to_string();
    {
        let tx = bg_tx.clone();
        let client = state.client.clone();
        spawn_home_load(tx, client);
    }

    // Check following series updates
    if !state.config.following_series.is_empty() {
        let tx = bg_tx.clone();
        let client = state.client.clone();
        let following = state.config.following_series.clone();
        spawn_following_load(tx, client, following);
    }

    loop {
        // Handle background results first
        while let Ok(result) = bg_rx.try_recv() {
            match result {
                BackgroundResult::HomeLoaded(items) => {
                    state.home_items = items;
                    state.loading = false;
                    state.status_msg = None;
                }
                BackgroundResult::FollowingUpdatesLoaded(updates) => {
                    state.following_updates = updates;
                }
                BackgroundResult::HomeSectionLoaded(items, total, label) => {
                    let count = items.len();
                    state.home_items = items;
                    state.total_items = total;
                    state.loading = false;
                    state.status_msg = Some(app::Message::info(format!("{}: {} items", label, count)));
                }
                BackgroundResult::MoreHomeItemsLoaded(more_items) => {
                    if more_items.is_empty() {
                        state.loading = false;
                    } else {
                        state.home_items.extend(more_items);
                        state.loading = false;
                        state.status_msg = Some(app::Message::info(format!("{} items", state.home_items.len())));
                    }
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
                    state.status_msg = None;
                }
                BackgroundResult::SeriesInfoLoaded(ss) => {
                    state.navigate_to(app::View::SeriesInfo);
                    state.series_state = ss;
                    state.loading = false;
                    state.status_msg = None;
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
                    if !more_episodes.is_empty() {
                        state.episodes.extend(more_episodes);
                        state.status_msg = Some(app::Message::info(format!("{} / {} episodes", state.episodes.len(), state.total_episodes)));
                    }
                    state.loading = false;
                }
                BackgroundResult::FolderLoaded(items, folder_id, total) => {
                    state.navigate_to(app::View::Items);
                    state.items = items;
                    state.current_folder_id = folder_id;
                    state.total_items = total;
                    state.loading = false;
                    state.status_msg = None;
                }
                BackgroundResult::MoreItemsLoaded(more_items, _folder_id) => {
                    if !more_items.is_empty() {
                        state.items.extend(more_items);
                        state.status_msg = Some(app::Message::info(format!("{} / {} items", state.items.len(), state.total_items)));
                    }
                    state.loading = false;
                }
                BackgroundResult::SearchLoaded(results) => {
                    state.search_results = results;
                    state.navigate_to(app::View::SearchResults);
                    state.loading = false;
                    state.searching = false;
                    state.status_msg = None;
                }
                BackgroundResult::ItemDetailLoaded(detail) => {
                    state.loading = false;
                    state.status_msg = None;
                    if detail.media_sources.len() > 1 {
                        state.open_source_select(&detail, detail.media_sources.clone());
                    } else if let Some(source) = detail.media_sources.first().cloned() {
                        state.open_track_select_or_playing(&detail, &source);
                    } else {
                        let url = state.client.stream_url_for_source(&detail, &Default::default());
                        if state.config.mpv_path == "mpv" {
                            state.open_mpv_prompt(&url, "", "", "", None);
                        } else if let Ok((child, rx)) = mpv::play(&url, &state.config.mpv_path, None, None, None, None) {
                            state.mpv_child = Some(child);
                            state.mpv_rx = Some(rx);
                            state.mpv_output.clear();
                            state.mpv_output_scroll = 0;
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
                BackgroundResult::AccountLoginSuccess(account, client, _account_id) => {
                    state.client = client;
                    state.server = account.server.clone();
                    state.config = crate::config::load_config();
                    let mut accounts_cfg = crate::config::load_accounts();
                    let existing = accounts_cfg.accounts.iter().position(|a| a.server == account.server && a.username == account.username);
                    let account_id = if let Some(idx) = existing {
                        accounts_cfg.accounts[idx].password_enc = account.password_enc.clone();
                        accounts_cfg.accounts[idx].id.clone()
                    } else {
                        let id = if account.id.is_empty() { uuid::Uuid::new_v4().to_string() } else { account.id.clone() };
                        accounts_cfg.accounts.push(crate::config::Account {
                            id: id.clone(),
                            label: account.label.clone(),
                            server: account.server.clone(),
                            username: account.username.clone(),
                            password_enc: account.password_enc.clone(),
                        });
                        id
                    };
                    accounts_cfg.last_account_id = Some(account_id);
                    let _ = crate::config::save_accounts(&accounts_cfg);
                    state.libraries.clear();
                    state.libraries_fetched_at = None;
                    state.library_latest.clear();
                    state.library_latest_fetched_at = None;
                    state.home_items.clear();
                    state.following_updates.clear();
                    state.favorites.clear();
                    state.items.clear();
                    state.stack.clear();
                    state.view = app::View::Home;
                    state.selected = 0;
                    state.loading = true;
                    state.loading_msg = t("status.loading_home").to_string();
                    state.status_msg = Some(app::Message::success(format!("{} {}", t("status.logged_in"), account.username)));
                    let tx = bg_tx.clone();
                    let client = state.client.clone();
                    spawn_home_load(tx, client);
                    let tx2 = bg_tx.clone();
                    let client2 = state.client.clone();
                    let following = state.config.following_series.clone();
                    spawn_following_load(tx2, client2, following);
                }
                BackgroundResult::AccountLoginFailed(msg) => {
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
                    if state.library_browser_state.library_id == lib_id && !more_items.is_empty() {
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
                BackgroundResult::SeriesMarkedWatched(series_id, count) => {
                    state.favorites.retain(|item| item.id != series_id);
                    state.loading = false;
                    state.status_msg = Some(app::Message::success(tf("status.marked_watched", &count.to_string())));
                }
                BackgroundResult::FavoriteToggled(item_id, is_favorite, item_type) => {
                    update_favorite_in_list(&mut state.home_items, &item_id, is_favorite);
                    update_favorite_in_list(&mut state.items, &item_id, is_favorite);
                    update_favorite_in_list(&mut state.library_browser_state.items, &item_id, is_favorite);
                    if !is_favorite {
                        state.favorites.retain(|item| item.id != item_id);
                        state.total_favorites = state.total_favorites.saturating_sub(1);
                    }
                    // Auto follow/unfollow Series when favoriting Series or Episode
                    let series_id = if item_type == "Series" {
                        Some(item_id.clone())
                    } else if item_type == "Episode" {
                        state.favorites.iter().find(|i| i.id == item_id).and_then(|i| i.series_id.clone())
                            .or_else(|| state.items.iter().find(|i| i.id == item_id).and_then(|i| i.series_id.clone()))
                    } else {
                        None
                    };
                    if let Some(sid) = series_id {
                        if is_favorite && !state.config.following_series.contains(&sid) {
                            state.toggle_follow(&sid);
                        } else if !is_favorite && state.config.following_series.contains(&sid) {
                            state.toggle_follow(&sid);
                        }
                    }
                    state.loading = false;
                    state.status_msg = Some(app::Message::success(if is_favorite { t("status.added_favorites").to_string() } else { t("status.removed_favorites").to_string() }));
                }
            }
        }

        // Check if mpv exited
        if let Some(ref mut child) = state.mpv_child {
            if let Ok(Some(_)) = child.try_wait() {
                let position_ticks = state.stop_playback();
                let item_id = state.playing_state.item_id.clone();
                let source_id = state.playing_state.media_source_id.clone();
                let session_id = state.play_session_id.clone();
                let client = state.client.clone();
                tokio::spawn(async move {
                    let _ = client.report_playback_stopped(&item_id, &source_id, &session_id, position_ticks).await;
                });
                state.status_msg = Some(app::Message::info(t("status.mpv_closed").to_string()));
            }
        }

        // Drain mpv events
        if state.mpv_rx.is_some() {
            let rx = state.mpv_rx.as_ref().unwrap();
            while let Ok(event) = rx.try_recv() {
                match event {
                    mpv::MpvEvent::LogLine(line, level) => {
                        state.mpv_output.push((line, level));
                        if state.mpv_output.len() > 200 {
                            state.mpv_output.remove(0);
                        }
                    }
                    mpv::MpvEvent::Position(pos) => {
                        state.mpv_position = pos;
                    }
                    mpv::MpvEvent::Duration(dur) => {
                        state.mpv_duration = dur;
                    }
                    mpv::MpvEvent::PlaybackStarted => {
                        state.status_msg = Some(app::Message::success("Playback started".to_string()));
                    }
                    mpv::MpvEvent::PlaybackEnded => {
                        // Handled by mpv exit detection below
                    }
                }
            }
        }

        // Update spinner
        if state.loading {
            state.status_msg = Some(app::Message::Loading(
                SPINNER[spin_idx % SPINNER.len()].to_string(),
                state.loading_msg.clone(),
            ));
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
                            let in_mpv = state.settings_state.section == app::SettingsSection::MpvPath;
                            let in_lang = state.settings_state.section == app::SettingsSection::Language;
                            let in_theme = state.settings_state.section == app::SettingsSection::Theme;
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => state.settings_cancel(),
                                KeyCode::Tab => state.settings_switch_section(),
                                KeyCode::Char(c) if in_mpv => state.settings_mpv_input(c),
                                KeyCode::Backspace if in_mpv => state.settings_mpv_backspace(),
                                KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') if in_lang => state.settings_toggle_language(),
                                KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') if in_theme => state.settings_cycle_theme(key.code == KeyCode::Right || key.code == KeyCode::Char('l')),
                                KeyCode::Up if !in_mpv && !in_lang && !in_theme && key.modifiers.contains(KeyModifiers::SHIFT) => state.settings_move_up(),
                                KeyCode::Down if !in_mpv && !in_lang && !in_theme && key.modifiers.contains(KeyModifiers::SHIFT) => state.settings_move_down(),
                                KeyCode::Up | KeyCode::Char('k') if !in_mpv && !in_lang && !in_theme => state.settings_select_prev(),
                                KeyCode::Down | KeyCode::Char('j') if !in_mpv && !in_lang && !in_theme => state.settings_select_next(),
                                KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') if !in_mpv && !in_lang && !in_theme => state.settings_switch_column(),
                                KeyCode::Char(' ') if !in_mpv && !in_lang && !in_theme => state.settings_toggle(),
                                KeyCode::Char('K') if !in_mpv && !in_lang && !in_theme => state.settings_move_up(),
                                KeyCode::Char('J') if !in_mpv && !in_lang && !in_theme => state.settings_move_down(),
                                KeyCode::Enter => {
                                    state.settings_save();
                                    state.loading_msg = t("status.loading_libraries").to_string();
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
                                            state.open_track_select_or_playing(&item, &source);
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
                                            let item_id = item.id.clone();
                                            let source_id = source.id.clone();
                                            let runtime = item.runtime_ticks;
                                            state.open_playing(&item.display_name(), &item_id, &source_id, runtime, &url, &video_label, &audio_label, &sub_label, resume_ticks, Some(source.clone()), state.track_state.selected_video, state.track_state.selected_audio, state.track_state.selected_subtitle);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        app::View::Playing => {
                            let mpv_has_output = !state.mpv_output.is_empty();
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => {
                                    if !state.playing_state.playing {
                                        state.go_back();
                                    } else {
                                        let position_ticks = state.stop_playback();
                                        let item_id = state.playing_state.item_id.clone();
                                        let source_id = state.playing_state.media_source_id.clone();
                                        let session_id = state.play_session_id.clone();
                                        let client = state.client.clone();
                                        tokio::spawn(async move {
                                            let _ = client.report_playback_stopped(&item_id, &source_id, &session_id, position_ticks).await;
                                        });
                                    }
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if state.playing_state.playing && mpv_has_output {
                                        state.mpv_output_scroll = state.mpv_output_scroll.saturating_add(1);
                                    } else if state.playing_state.resume_position.is_some() {
                                        state.playing_state.option_selected = 0;
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if state.playing_state.playing && mpv_has_output {
                                        state.mpv_output_scroll = state.mpv_output_scroll.saturating_sub(1);
                                    } else if state.playing_state.resume_position.is_some() {
                                        state.playing_state.option_selected = 1;
                                    }
                                }
                                KeyCode::PageUp => {
                                    if mpv_has_output {
                                        state.mpv_output_scroll = state.mpv_output_scroll.saturating_add(10);
                                    }
                                }
                                KeyCode::PageDown => {
                                    if mpv_has_output {
                                        state.mpv_output_scroll = state.mpv_output_scroll.saturating_sub(10);
                                    }
                                }
                                KeyCode::Enter => {
                                    let is_default_mpv = state.config.mpv_path == "mpv";
                                    if is_default_mpv {
                                        let ps = &state.playing_state;
                                        let url = ps.url.clone();
                                        let start = ps.resume_position;
                                        state.open_mpv_prompt(&url, "", "", "", start);
                                    } else {
                                        state.status_msg = Some(app::Message::Loading("⣾".to_string(), t("status.launching_mpv").to_string()));
                                        let ps = &state.playing_state;
                                        let start_secs = if ps.resume_position.is_some() && ps.option_selected == 0 {
                                            ps.resume_position.map(|t| t as f64 / 10_000_000.0)
                                        } else {
                                            None
                                        };
                                        if let Ok((child, rx)) = mpv::play(&ps.url, &state.config.mpv_path, None, None, None, start_secs) {
                                            state.mpv_child = Some(child);
                                            state.mpv_rx = Some(rx);
                                            state.mpv_output.clear();
                                            state.mpv_output_scroll = 0;
                                            state.playing_state.playing = true;
                                            state.playback_started_at = Some(std::time::Instant::now());
                                            // Report playback start to Emby
                                            let item_id = state.playing_state.item_id.clone();
                                            let source_id = state.playing_state.media_source_id.clone();
                                            let session_id = state.play_session_id.clone();
                                            let client = state.client.clone();
                                            tokio::spawn(async move {
                                                let _ = client.report_playback_start(&item_id, &source_id, &session_id).await;
                                            });
                                        } else {
                                            state.status_msg = Some(app::Message::error("mpv::play failed".to_string()));
                                        }
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
                                        state.loading_msg = tf("status.loading", &state.series_name);
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
                                        state.loading_msg = tf("status.loading", &state.series_name);
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
                                    let mut series_item = crate::emby::MediaItem::separator("");
                                    series_item.id = series_id;
                                    spawn_series_info(bg_tx.clone(), state.client.clone(), series_item);
                                }
                                KeyCode::Enter => {
                                    if let Some(item) = state.selected_item().cloned() {
                                        if item.is_video() {
                                            state.loading = true;
                                            state.loading_msg = tf("status.loading", &item.display_name());
                                            spawn_item_detail(bg_tx.clone(), state.client.clone(), item.id.clone());
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
                                KeyCode::Char('f') => {
                                    if let Some(ref series_item) = state.series_state.item {
                                        let id = series_item.id.clone();
                                        state.toggle_follow(&id);
                                    }
                                }
                                KeyCode::Enter => {
                                    match state.series_state.section {
                                        app::SeriesSection::Seasons => {
                                            state.select_season().await?;
                                        }
                                        app::SeriesSection::Episodes => {
                                            if let Some(item) = state.series_selected_item().cloned() {
                                                if item.is_video() {
                                                    state.loading = true;
                                                    spawn_item_detail(bg_tx.clone(), state.client.clone(), item.id.clone());
                                                }
                                            }
                                        }
                                        app::SeriesSection::Similar => {
                                            if let Some(item) = state.series_selected_item().cloned() {
                                                state.loading = true;
                                                state.loading_msg = tf("status.loading", &item.display_name());
                                                spawn_series_info(bg_tx.clone(), state.client.clone(), item);
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
                                            state.loading_msg = tf("status.loading", &series_name);
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

                            if state.searching {
                                match key.code {
                                    KeyCode::Esc => state.cancel_search(),
                                    KeyCode::Char(c) => state.search_input(c),
                                    KeyCode::Backspace => state.search_backspace(),
                                    KeyCode::Enter => {
                                        let query = state.search_query.clone();
                                        let lib_id = state.library_browser_state.library_id.clone();
                                        state.loading = true;
                                        state.loading_msg = format!("Searching for \"{}\"...", query);
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        tokio::spawn(async move {
                                            match client.search_in_library(&query, &lib_id).await {
                                                Ok(results) => { let _ = tx.send(BackgroundResult::SearchLoaded(results)); }
                                                Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Search failed: {}", e))); }
                                            }
                                        });
                                    }
                                    _ => {}
                                }
                                continue;
                            }

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
                                KeyCode::Char('s') if !has_panel && key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    state.library_browser_open_sort_panel();
                                }
                                KeyCode::Char('Z') if !has_panel => {
                                    state.open_favorites();
                                    state.loading = true;
                                    state.loading_msg = t("status.loading_favorites").to_string();
                                    spawn_load_favorites(bg_tx.clone(), state.client.clone(), state.config.following_series.clone());
                                }
                                KeyCode::Char('z') if !has_panel => {
                                    if let Some(item) = state.selected_item().cloned() {
                                        let is_favorite = item.user_data.as_ref().map(|ud| ud.is_favorite).unwrap_or(false);
                                        let new_favorite = !is_favorite;
                                        let item_id = item.id.clone();
                                        let item_type = item.item_type.clone();
                                        state.loading = true;
                                        state.loading_msg = "Updating favorite...".to_string();
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        tokio::spawn(async move {
                                            let timeout = std::time::Duration::from_secs(30);
                                            match tokio::time::timeout(timeout, client.toggle_favorite(&item_id, new_favorite)).await {
                                                Ok(Ok(_)) => { let _ = tx.send(BackgroundResult::FavoriteToggled(item_id, new_favorite, item_type)); }
                                                Ok(Err(e)) => { let _ = tx.send(BackgroundResult::Error(format!("Favorite failed: {}", e))); }
                                                Err(_) => { let _ = tx.send(BackgroundResult::Timeout("Favorite".to_string())); }
                                            }
                                        });
                                    }
                                }
                                KeyCode::Char('f') if !has_panel && key.modifiers.contains(KeyModifiers::CONTROL) => {
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
                                KeyCode::Char('/') if !has_panel => {
                                    let lib_id = state.library_browser_state.library_id.clone();
                                    state.start_search(app::SearchContext::Library(lib_id));
                                }
                                KeyCode::Char('e') if !has_panel => {
                                    if let Some(item) = state.selected_item().cloned() {
                                        state.loading = true;
                                        state.loading_msg = tf("status.loading", &item.display_name());
                                        spawn_series_info(bg_tx.clone(), state.client.clone(), item);
                                    }
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if has_panel {
                                        state.library_browser_panel_prev();
                                    } else {
                                        state.select_prev();
                                        let bs = &state.library_browser_state;
                                        if !state.loading && bs.total > bs.items.len() && state.selected + 1 >= bs.items.len() {
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
                                        if !state.loading && bs.total > bs.items.len() && state.selected + 1 >= bs.items.len() {
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
                                        if !state.loading && bs.total > bs.items.len() && state.selected + 1 >= bs.items.len() {
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
                                                state.loading_msg = tf("status.loading", &item.display_name());
                                                spawn_item_detail(bg_tx.clone(), state.client.clone(), item.id.clone());
                                            } else if item.is_navigable() {
                                                state.loading = true;
                                                state.loading_msg = tf("status.loading", &item.display_name());
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
                        app::View::Wizard => {
                            match key.code {
                                KeyCode::Esc => break,
                                KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                                    if state.wizard_state.step == app::WizardField::Language {
                                        state.wizard_toggle_language();
                                    }
                                }
                                KeyCode::Tab => {
                                    if state.wizard_state.step == app::WizardField::Language {
                                        // Tab on language = go to next step
                                        state.wizard_next();
                                    } else if state.wizard_state.step == app::WizardField::MpvPath {
                                        state.wizard_state.mpv_path = "mpv".to_string();
                                        let ws = &state.wizard_state;
                                        let server = ws.server.trim().to_string();
                                        let username = ws.username.trim().to_string();
                                        let password = ws.password.clone();
                                        state.loading = true;
                                        state.loading_msg = t("status.connecting").to_string();
                                        let tx = bg_tx.clone();
                                        tokio::spawn(async move {
                                            match crate::emby::EmbyClient::authenticate(&server, &username, &password).await {
                                                Ok(client) => {
                                                    let account = crate::config::Account {
                                                        id: uuid::Uuid::new_v4().to_string(),
                                                        label: format!("{}@{}", username, server),
                                                        server,
                                                        username,
                                                        password_enc: crate::crypto::encrypt(&password),
                                                    };
                                                    let _ = tx.send(BackgroundResult::AccountLoginSuccess(account, client, String::new()));
                                                }
                                                Err(e) => { let _ = tx.send(BackgroundResult::AccountLoginFailed(format!("{}: {}", t("status.login_failed"), e))); }
                                            }
                                        });
                                    }
                                }
                                KeyCode::Enter => {
                                    match state.wizard_next() {
                                        app::WizardAction::None => {}
                                        app::WizardAction::FinishWizard => {
                                            let mpv = state.wizard_state.mpv_path.trim().to_string();
                                            let lang = state.wizard_state.language.clone();
                                            state.config.language = lang;
                                            if !mpv.is_empty() && mpv != "mpv" {
                                                state.config.mpv_path = mpv;
                                            }
                                            let _ = crate::config::save_config(&state.config);
                                            let server = state.wizard_state.server.trim().to_string();
                                            let username = state.wizard_state.username.trim().to_string();
                                            let password = state.wizard_state.password.clone();
                                            state.loading = true;
                                            state.loading_msg = t("status.connecting").to_string();
                                            let tx = bg_tx.clone();
                                            tokio::spawn(async move {
                                                match crate::emby::EmbyClient::authenticate(&server, &username, &password).await {
                                                    Ok(client) => {
                                                        let account = crate::config::Account {
                                                            id: uuid::Uuid::new_v4().to_string(),
                                                            label: format!("{}@{}", username, server),
                                                            server,
                                                            username,
                                                            password_enc: crate::crypto::encrypt(&password),
                                                        };
                                                        let _ = tx.send(BackgroundResult::AccountLoginSuccess(account, client, String::new()));
                                                    }
                                                    Err(e) => { let _ = tx.send(BackgroundResult::AccountLoginFailed(format!("{}: {}", t("status.login_failed"), e))); }
                                                }
                                            });
                                        }
                                    }
                                }
                                KeyCode::Char(c) => state.wizard_input(c),
                                KeyCode::Backspace => state.wizard_backspace(),
                                _ => {}
                            }
                        }
                        app::View::MpvPrompt => {
                            match key.code {
                                KeyCode::Esc => {
                                    state.view = app::View::Home;
                                    state.status_msg = None;
                                }
                                KeyCode::Enter => {
                                    let mpv_path = state.mpv_prompt_state.mpv_path.trim().to_string();
                                    if mpv_path.is_empty() {
                                        state.status_msg = Some(app::Message::error("MPV path is required".to_string()));
                                    } else {
                                        state.config.mpv_path = mpv_path;
                                        let _ = crate::config::save_config(&state.config);
                                        let ps = &state.mpv_prompt_state;
                                        let start_secs = ps.resume_position.map(|t| t as f64 / 10_000_000.0);
                                        if let Ok((child, rx)) = mpv::play(&ps.url, &state.config.mpv_path, None, None, None, start_secs) {
                                            state.mpv_child = Some(child);
                                            state.mpv_rx = Some(rx);
                                            state.mpv_output.clear();
                                            state.mpv_output_scroll = 0;
                                        }
                                        state.view = app::View::Home;
                                        state.status_msg = Some(app::Message::success("MPV path saved".to_string()));
                                    }
                                }
                                KeyCode::Char(c) => state.mpv_prompt_input(c),
                                KeyCode::Backspace => state.mpv_prompt_backspace(),
                                _ => {}
                            }
                        }
                        app::View::AccountManager => {
                            let ams = &state.account_manager_state;
                            let is_input = matches!(ams.action, app::AccountManagerAction::Add | app::AccountManagerAction::Edit(_));
                            let is_delete = matches!(ams.action, app::AccountManagerAction::Delete(_));

                            if is_input {
                                match key.code {
                                    KeyCode::Esc => {
                                        state.account_manager_state.action = app::AccountManagerAction::View;
                                        state.account_manager_state.status_msg = None;
                                    }
                                    KeyCode::Tab => state.account_manager_next_field(),
                                    KeyCode::Char(c) => state.account_manager_input(c),
                                    KeyCode::Backspace => state.account_manager_backspace(),
                                    KeyCode::Enter => {
                                        let ams = &state.account_manager_state;
                                        let server = ams.input_server.trim().to_string();
                                        let username = ams.input_username.trim().to_string();
                                        let password = ams.input_password.clone();
                                        let label = if ams.input_label.trim().is_empty() {
                                            format!("{}@{}", username, server)
                                        } else {
                                            ams.input_label.trim().to_string()
                                        };

                                        if server.is_empty() || username.is_empty() || password.is_empty() {
                                            state.account_manager_state.status_msg = Some("Server, username and password are required".to_string());
                                        } else {
                                            let password_enc = crate::crypto::encrypt(&password);
                                            let edit_idx = if let app::AccountManagerAction::Edit(idx) = ams.action { Some(idx) } else { None };

                                            let account = crate::config::Account {
                                                id: if let Some(idx) = edit_idx {
                                                    ams.accounts.get(idx).map(|a| a.id.clone()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
                                                } else {
                                                    uuid::Uuid::new_v4().to_string()
                                                },
                                                label,
                                                server,
                                                username,
                                                password_enc,
                                            };

                                            let mut accounts = ams.accounts.clone();
                                            if let Some(idx) = edit_idx {
                                                if idx < accounts.len() {
                                                    accounts[idx] = account;
                                                }
                                            } else {
                                                accounts.push(account);
                                            }

                                            let accounts_cfg = crate::config::AccountsConfig {
                                                accounts: accounts.clone(),
                                                last_account_id: ams.last_account_id.clone(),
                                            };
                                            let _ = crate::config::save_accounts(&accounts_cfg);

                                            state.account_manager_state.accounts = accounts;
                                            state.account_manager_state.action = app::AccountManagerAction::View;
                                            state.account_manager_state.status_msg = Some(t("status.account_saved").to_string());
                                        }
                                    }
                                    _ => {}
                                }
                            } else if is_delete {
                                match key.code {
                                    KeyCode::Esc | KeyCode::Char('n') => {
                                        state.account_manager_state.action = app::AccountManagerAction::View;
                                        state.account_manager_state.status_msg = None;
                                    }
                                    KeyCode::Char('y') | KeyCode::Enter => {
                                        let idx = if let app::AccountManagerAction::Delete(i) = state.account_manager_state.action { i } else { 0 };
                                        state.account_manager_state.accounts.remove(idx);
                                        let accounts_cfg = crate::config::AccountsConfig {
                                            accounts: state.account_manager_state.accounts.clone(),
                                            last_account_id: state.account_manager_state.last_account_id.clone(),
                                        };
                                        let _ = crate::config::save_accounts(&accounts_cfg);
                                        state.account_manager_state.action = app::AccountManagerAction::View;
                                        state.account_manager_state.status_msg = Some(t("status.account_deleted").to_string());
                                    }
                                    _ => {}
                                }
                            } else {
                                match key.code {
                                    KeyCode::Esc | KeyCode::Char('u') => state.go_back(),
                                    KeyCode::Char('q') => break,
                                    KeyCode::Up | KeyCode::Char('k') => state.account_manager_select_prev(),
                                    KeyCode::Down | KeyCode::Char('j') => state.account_manager_select_next(),
                                    KeyCode::Char('a') => {
                                        state.account_manager_state.action = app::AccountManagerAction::Add;
                                        state.account_manager_state.input_server.clear();
                                        state.account_manager_state.input_username.clear();
                                        state.account_manager_state.input_password.clear();
                                        state.account_manager_state.input_label.clear();
                                        state.account_manager_state.input_field = app::AccountInputField::Label;
                                        state.account_manager_state.selected = 0;
                                        state.account_manager_state.status_msg = None;
                                    }
                                    KeyCode::Enter => {
                                        let sel = state.account_manager_state.selected;
                                        let acc_count = state.account_manager_state.accounts.len();
                                        if sel < acc_count {
                                            if let Some(acc) = state.account_manager_state.accounts.get(sel).cloned() {
                                                let password = crate::crypto::decrypt(&acc.password_enc).unwrap_or_default();
                                                let server = acc.server.clone();
                                                let username = acc.username.clone();
                                                let account_id = acc.id.clone();
                                                state.loading = true;
                                                state.loading_msg = format!("Logging in as {}...", acc.username);
                                                let tx = bg_tx.clone();
                                                tokio::spawn(async move {
                                                    match crate::emby::EmbyClient::authenticate(&server, &username, &password).await {
                                                        Ok(client) => {
                                                            let _ = tx.send(BackgroundResult::AccountLoginSuccess(acc, client, account_id));
                                                        }
                                                        Err(e) => {
                                                            let _ = tx.send(BackgroundResult::AccountLoginFailed(format!("Login failed: {}", e)));
                                                        }
                                                    }
                                                });
                                            }
                                        } else if sel == acc_count {
                                            state.account_manager_state.action = app::AccountManagerAction::Add;
                                            state.account_manager_state.input_server.clear();
                                            state.account_manager_state.input_username.clear();
                                            state.account_manager_state.input_password.clear();
                                            state.account_manager_state.input_label.clear();
                                            state.account_manager_state.input_field = app::AccountInputField::Label;
                                            state.account_manager_state.selected = 0;
                                            state.account_manager_state.status_msg = None;
                                        }
                                    }
                                    KeyCode::Char('e') => {
                                        let sel = state.account_manager_state.selected;
                                        if sel < state.account_manager_state.accounts.len() {
                                            if let Some(acc) = state.account_manager_state.accounts.get(sel).cloned() {
                                                let password = crate::crypto::decrypt(&acc.password_enc).unwrap_or_default();
                                                state.account_manager_state.input_label = acc.label;
                                                state.account_manager_state.input_server = acc.server;
                                                state.account_manager_state.input_username = acc.username;
                                                state.account_manager_state.input_password = password;
                                                state.account_manager_state.input_field = app::AccountInputField::Label;
                                                state.account_manager_state.selected = 0;
                                                state.account_manager_state.action = app::AccountManagerAction::Edit(sel);
                                                state.account_manager_state.status_msg = None;
                                            }
                                        }
                                    }
                                    KeyCode::Char('d') | KeyCode::Delete => {
                                        let sel = state.account_manager_state.selected;
                                        if sel < state.account_manager_state.accounts.len() {
                                            state.account_manager_state.action = app::AccountManagerAction::Delete(sel);
                                            state.account_manager_state.selected = 0;
                                        }
                                    }
                                    _ => {}
                                }
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
                                    let context = state.search_context.clone();
                                    state.loading = true;
                                    state.loading_msg = format!("Searching for \"{}\"...", query);
                                    match &context {
                                        app::SearchContext::LocalHome => {
                                            let q = query.to_lowercase();
                                            let results: Vec<_> = state.home_items.iter()
                                                .filter(|item| item.name.to_lowercase().contains(&q))
                                                .cloned()
                                                .collect();
                                            state.search_results = results;
                                            state.navigate_to(app::View::SearchResults);
                                            state.loading = false;
                                        }
                                        app::SearchContext::Library(parent_id) => {
                                            let tx = bg_tx.clone();
                                            let client = state.client.clone();
                                            let parent_id = parent_id.clone();
                                            tokio::spawn(async move {
                                                match client.search_in_library(&query, &parent_id).await {
                                                    Ok(results) => { let _ = tx.send(BackgroundResult::SearchLoaded(results)); }
                                                    Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Search failed: {}", e))); }
                                                }
                                            });
                                        }
                                        app::SearchContext::ServerWide => {
                                            let tx = bg_tx.clone();
                                            let client = state.client.clone();
                                            tokio::spawn(async move {
                                                match client.search(&query).await {
                                                    Ok(results) => { let _ = tx.send(BackgroundResult::SearchLoaded(results)); }
                                                    Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Search failed: {}", e))); }
                                                }
                                            });
                                        }
                                    }
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if state.searching { continue; }
                                    state.select_prev();
                                    if state.should_load_more_items() {
                                        state.loading = true;
                                        state.loading_msg = t("status.loading_items").to_string();
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
                                    } else if state.should_load_more_home_items() {
                                        state.loading = true;
                                        state.loading_msg = t("status.loading").to_string();
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        let start = state.home_items.len();
                                        let is_continue = state.view == app::View::ContinueWatching;
                                        tokio::spawn(async move {
                                            let result = if is_continue {
                                                client.get_resume_items_page(start, 50).await
                                            } else {
                                                client.get_latest_items_page(start, 50).await
                                            };
                                            match result {
                                                Ok(r) => { let _ = tx.send(BackgroundResult::MoreHomeItemsLoaded(r.items)); }
                                                Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Failed to load: {}", e))); }
                                            }
                                        });
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if state.searching { continue; }
                                    state.select_next();
                                    if state.should_load_more_items() {
                                        state.loading = true;
                                        state.loading_msg = t("status.loading_items").to_string();
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
                                    } else if state.should_load_more_home_items() {
                                        state.loading = true;
                                        state.loading_msg = t("status.loading").to_string();
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        let start = state.home_items.len();
                                        let is_continue = state.view == app::View::ContinueWatching;
                                        tokio::spawn(async move {
                                            let result = if is_continue {
                                                client.get_resume_items_page(start, 50).await
                                            } else {
                                                client.get_latest_items_page(start, 50).await
                                            };
                                            match result {
                                                Ok(r) => { let _ = tx.send(BackgroundResult::MoreHomeItemsLoaded(r.items)); }
                                                Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Failed to load: {}", e))); }
                                            }
                                        });
                                    }
                                }
                                KeyCode::PageUp => {
                                    if state.searching { continue; }
                                    state.page_up();
                                }
                                KeyCode::PageDown => {
                                    if state.searching { continue; }
                                    state.page_down();
                                    if state.should_load_more_items() {
                                        state.loading = true;
                                        state.loading_msg = t("status.loading_items").to_string();
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
                                    if state.searching { continue; }
                                    if let Some(item) = state.selected_item().cloned() {
                                        let is_favorite = item.user_data.as_ref().map(|ud| ud.is_favorite).unwrap_or(false);
                                        let new_favorite = !is_favorite;
                                        let item_id = item.id.clone();
                                        let item_type = item.item_type.clone();
                                        state.loading = true;
                                        state.loading_msg = "Updating favorite...".to_string();
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        tokio::spawn(async move {
                                            let timeout = std::time::Duration::from_secs(10);
                                            match tokio::time::timeout(timeout, client.toggle_favorite(&item_id, new_favorite)).await {
                                                Ok(Ok(_)) => { let _ = tx.send(BackgroundResult::FavoriteToggled(item_id, new_favorite, item_type)); }
                                                Ok(Err(e)) => { let _ = tx.send(BackgroundResult::Timeout(format!("Favorite: {}", e))); }
                                                Err(_) => { let _ = tx.send(BackgroundResult::Timeout("Favorite: timeout".to_string())); }
                                            }
                                        });
                                    }
                                }
                                KeyCode::Left | KeyCode::Char('h') => {
                                    if state.searching { continue; }
                                    state.go_back();
                                }
                                KeyCode::Right | KeyCode::Char('l') => {
                                    if state.searching { continue; }
                                    state.show_libraries().await;
                                    if !state.is_libraries_cache_valid() || !state.is_latest_cache_valid() {
                                        state.loading_msg = t("status.loading_libraries").to_string();
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
                                            let cw_label = t("title.continue_watching");
                                            let latest_label = t("title.latest");
                                            let section = if item.name.contains(cw_label) {
                                                "continue_watching"
                                            } else if item.name.contains(latest_label) {
                                                "latest"
                                            } else {
                                                continue;
                                            };
                                            let label = if section == "continue_watching" {
                                                cw_label.to_string()
                                            } else {
                                                latest_label.to_string()
                                            };
                                            if section == "continue_watching" {
                                                state.open_continue_watching();
                                            } else {
                                                state.open_latest_items();
                                            }
                                            state.loading = true;
                                            state.loading_msg = tf("status.loading", &label);
                                            let tx = bg_tx.clone();
                                            let client = state.client.clone();
                                            let section = section.to_string();
                                            tokio::spawn(async move {
                                                let timeout = std::time::Duration::from_secs(60);
                                                let result = if section == "continue_watching" {
                                                    tokio::time::timeout(timeout, client.get_resume_items_page(0, 50)).await
                                                } else {
                                                    tokio::time::timeout(timeout, client.get_latest_items_page(0, 50)).await
                                                };
                                                match result {
                                                    Ok(Ok(r)) => { let _ = tx.send(BackgroundResult::HomeSectionLoaded(r.items, r.total, label)); }
                                                    Ok(Err(e)) => { let _ = tx.send(BackgroundResult::Error(format!("Failed to load: {}", e))); }
                                                    Err(_) => { let _ = tx.send(BackgroundResult::Timeout(label)); }
                                                }
                                            });
                                            continue;
                                        }
                                        if item.is_video() {
                                            state.loading = true;
                                            state.loading_msg = tf("status.loading", &item.display_name());
                                            spawn_item_detail(bg_tx.clone(), state.client.clone(), item.id.clone());
                                        } else if item.is_navigable() {
                                            // If this is a library folder (from "媒体库"), open library browser
                                            if state.view == app::View::Items && item.item_type == "Folder" {
                                                state.open_library_browser(item.id.clone(), item.name.clone());
                                                state.loading = true;
                                                state.loading_msg = tf("status.loading", &item.name);
                                                let tx = bg_tx.clone();
                                                let client = state.client.clone();
                                                let library_id = item.id.clone();
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
                                                        Ok(r) => { let _ = tx.send(BackgroundResult::LibraryBrowserLoaded(r.0, r.1, r.2, r.3, r.4, r.5, r.6)); }
                                                        Err(_) => { let _ = tx.send(BackgroundResult::Timeout("Library".to_string())); }
                                                    }
                                                });
                                                continue;
                                            }
                                            state.loading = true;
                                            state.loading_msg = tf("status.loading", &item.display_name());
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
                                        } else if !item.is_separator() && !item.id.is_empty() {
                                            // Fallback: try loading as item detail
                                            state.loading = true;
                                            state.loading_msg = tf("status.loading", &item.display_name());
                                            spawn_item_detail(bg_tx.clone(), state.client.clone(), item.id.clone());
                                        }
                                    } else if let Some(lib) = state.selected_library().cloned() {
                                        state.open_library_browser(lib.id.clone(), lib.name.clone());
                                        state.loading = true;
                                        state.loading_msg = tf("status.loading", &lib.name);
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
                                    } else if state.view == app::View::Libraries && state.selected == 0 {
                                        // "媒体库" header — fetch all libraries and show as folders
                                        state.loading = true;
                                        state.loading_msg = t("status.loading_libraries").to_string();
                                        let tx = bg_tx.clone();
                                        let client = state.client.clone();
                                        tokio::spawn(async move {
                                            let timeout = std::time::Duration::from_secs(30);
                                            let result = tokio::time::timeout(timeout, client.get_libraries()).await;
                                            match result {
                                                Ok(Ok(libs)) => {
                                                    let items: Vec<_> = libs.into_iter().map(|lib| {
                                                        crate::emby::MediaItem {
                                                            id: lib.id.clone(),
                                                            name: lib.name.clone(),
                                                            item_type: "Folder".to_string(),
                                                            media_type: None,
                                                            series_name: None,
                                                            index_number: None,
                                                            parent_index_number: None,
                                                            runtime_ticks: None,
                                                            series_id: None,
                                                            overview: None,
                                                            child_count: None,
                                                            media_sources: Vec::new(),
                                                            user_data: None,
                                                        }
                                                    }).collect();
                                                    let _ = tx.send(BackgroundResult::FolderLoaded(items, String::new(), 0));
                                                }
                                                _ => { let _ = tx.send(BackgroundResult::Timeout("Libraries".to_string())); }
                                            }
                                        });
                                    } else if let Some(lib_name) = state.selected_section_name() {
                                        // Section header — open library browser for that library
                                        if let Some(lib) = state.libraries.iter().find(|l| l.name == lib_name).cloned() {
                                                state.open_library_browser(lib.id.clone(), lib.name.clone());
                                                state.loading = true;
                                                state.loading_msg = tf("status.loading", &lib.name);
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
                                                        Ok(r) => { let _ = tx.send(BackgroundResult::LibraryBrowserLoaded(r.0, r.1, r.2, r.3, r.4, r.5, r.6)); }
                                                        Err(_) => { let _ = tx.send(BackgroundResult::Timeout("Library".to_string())); }
                                                    }
                                                 });
                                            }
                                    }
                                }
                                KeyCode::Backspace => {
                                    if state.searching { continue; }
                                    state.go_back();
                                }
                                KeyCode::Char('/') => {
                                    if state.searching { continue; }
                                    let context = if matches!(state.view, app::View::ContinueWatching | app::View::LatestItems) {
                                        app::SearchContext::LocalHome
                                    } else {
                                        app::SearchContext::ServerWide
                                    };
                                    state.start_search(context);
                                }
                                KeyCode::Char('Z') | KeyCode::Char('F') => {
                                    if state.searching { continue; }
                                    state.open_favorites();
                                    state.loading = true;
                                    state.loading_msg = t("status.loading_favorites").to_string();
                                    spawn_load_favorites(bg_tx.clone(), state.client.clone(), state.config.following_series.clone());
                                }
                                KeyCode::Char('s') => {
                                    if state.searching { continue; }
                                    if state.libraries.is_empty() {
                                        state.loading = true;
                                        state.loading_msg = t("status.loading_libraries").to_string();
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
                                KeyCode::Char('u') => {
                                    if state.searching { continue; }
                                    state.open_account_manager();
                                }
                                KeyCode::Char('e') => {
                                    if state.searching { continue; }
                                    if let Some(item) = state.selected_item().cloned() {
                                        state.loading = true;
                                        state.loading_msg = tf("status.loading", &item.display_name());
                                        spawn_series_info(bg_tx.clone(), state.client.clone(), item);
                                    }
                                }
                                KeyCode::Char('m') => {
                                    if state.searching { continue; }
                                    if state.view == app::View::Favorites {
                                        if let Some(item) = state.selected_item().cloned() {
                                            if item.item_type == "Series" {
                                                state.loading = true;
                                                state.loading_msg = format!("Marking {} as watched...", item.name);
                                                let tx = bg_tx.clone();
                                                let client = state.client.clone();
                                                let series_id = item.id.clone();
                                                tokio::spawn(async move {
                                                    match client.mark_series_watched(&series_id).await {
                                                        Ok(count) => { let _ = tx.send(BackgroundResult::SeriesMarkedWatched(series_id, count)); }
                                                        Err(e) => { let _ = tx.send(BackgroundResult::Error(format!("Failed: {}", e))); }
                                                    }
                                                });
                                            }
                                        }
                                    }
                                }
                                KeyCode::Char('f') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    if state.searching { continue; }
                                    if matches!(state.view, app::View::Home | app::View::Items | app::View::SearchResults | app::View::Favorites | app::View::LibraryBrowser) {
                                        if let Some(item) = state.selected_item().cloned() {
                                            let series_id = if item.item_type == "Series" {
                                                Some(item.id.clone())
                                            } else if item.item_type == "Episode" {
                                                item.series_id.clone()
                                            } else {
                                                None
                                            };
                                            if let Some(series_id) = series_id {
                                                state.toggle_follow(&series_id);
                                            }
                                        }
                                    }
                                }
                                KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    if state.searching { continue; }
                                    state.loading = true;
                                    state.loading_msg = "Refreshing home...".to_string();
                                    let tx = bg_tx.clone();
                                    let client = state.client.clone();
                                    spawn_home_load(tx, client);
                                    let tx2 = bg_tx.clone();
                                    let client2 = state.client.clone();
                                    let following = state.config.following_series.clone();
                                    spawn_following_load(tx2, client2, following);
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

fn spawn_load_favorites(tx: mpsc::UnboundedSender<BackgroundResult>, client: crate::emby::EmbyClient, following: Vec<String>) {
    tokio::spawn(async move {
        let fav_result = client.get_favorites(0, 200).await;
        let mut fav_ids = std::collections::HashSet::new();
        let mut all_items = Vec::new();
        let mut total = 0;

        if let Ok(result) = fav_result {
            total = result.total;
            for item in &result.items {
                fav_ids.insert(item.id.clone());
            }
            all_items.extend(result.items);
        }

        for series_id in &following {
            if !fav_ids.contains(series_id) {
                if let Ok(item) = client.get_item_detail(series_id).await {
                    all_items.push(item);
                }
            }
        }

        let _ = tx.send(BackgroundResult::FavoritesLoaded(all_items, total));
    });
}

fn spawn_item_detail(tx: mpsc::UnboundedSender<BackgroundResult>, client: crate::emby::EmbyClient, item_id: String) {
    tokio::spawn(async move {
        let timeout = std::time::Duration::from_secs(60);
        match tokio::time::timeout(timeout, client.get_item_detail(&item_id)).await {
            Ok(Ok(detail)) => { let _ = tx.send(BackgroundResult::ItemDetailLoaded(detail)); }
            _ => { let _ = tx.send(BackgroundResult::Timeout("Item detail".to_string())); }
        }
    });
}

fn spawn_series_info(tx: mpsc::UnboundedSender<BackgroundResult>, client: crate::emby::EmbyClient, item: crate::emby::MediaItem) {
    tokio::spawn(async move {
        let result = build_series_state(&client, &item).await;
        let _ = tx.send(BackgroundResult::SeriesInfoLoaded(result));
    });
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
