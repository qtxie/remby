use std::sync::Arc;

use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::*;
use gpui_component::input::InputState;

use crate::state::{GuiState, View};
use crate::views::browser::BrowserView;
use crate::views::favorites::FavoritesView;
use crate::views::home::HomeView;
use crate::views::libraries::LibrariesView;
use crate::views::login::LoginView;
use crate::views::player::PlayerView;
use crate::views::series::SeriesView;
use crate::views::components::sidebar::SidebarNav;
use crate::views::settings::SettingsView;

#[derive(gpui::Action, Clone, PartialEq)]
struct GoBack;

#[derive(gpui::Action, Clone, PartialEq)]
struct NavigateHome;

#[derive(gpui::Action, Clone, PartialEq)]
struct NavigateLibraries;

#[derive(gpui::Action, Clone, PartialEq)]
struct NavigateSettings;

#[derive(gpui::Action, Clone, PartialEq)]
struct SelectNext;

#[derive(gpui::Action, Clone, PartialEq)]
struct SelectPrev;

#[derive(gpui::Action, Clone, PartialEq)]
struct SelectItem;

#[derive(gpui::Action, Clone, PartialEq)]
struct ToggleFavorite;

#[derive(gpui::Action, Clone, PartialEq)]
struct ToggleFollow;

#[derive(gpui::Action, Clone, PartialEq)]
struct QuitApp;

pub fn init_key_bindings(cx: &mut App) {
    cx.bind_keys(vec![
        KeyBinding::new("escape", GoBack, None),
        KeyBinding::new("q", QuitApp, None),
        KeyBinding::new("j", SelectNext, None),
        KeyBinding::new("k", SelectPrev, None),
        KeyBinding::new("enter", SelectItem, None),
        KeyBinding::new("z", ToggleFavorite, None),
        KeyBinding::new("f", ToggleFollow, None),
        KeyBinding::new("s", NavigateSettings, None),
        KeyBinding::new("l", NavigateLibraries, None),
        KeyBinding::new("u", NavigateHome, None),
    ]);
}

pub struct RembyApp {
    pub state: GuiState,
    pub image_loader: Arc<crate::image_loader::ImageLoader>,
    server_input: Entity<InputState>,
    username_input: Entity<InputState>,
    password_input: Entity<InputState>,
    browser_search_input: Entity<InputState>,
    mpv_path_input: Entity<InputState>,
    player_stop_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl RembyApp {
    pub fn new(
        server_input: Entity<InputState>,
        username_input: Entity<InputState>,
        password_input: Entity<InputState>,
        browser_search_input: Entity<InputState>,
        mpv_path_input: Entity<InputState>,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut state = GuiState::new();
        state.config = remby_core::config::load_config();

        crate::theme_adapter::apply_remby_theme(cx, &state.config.theme);

        // Auto-login from saved accounts
        let accounts_cfg = remby_core::config::load_accounts();
        let last_account = accounts_cfg.last_account_id.and_then(|id| {
            accounts_cfg.accounts.into_iter().find(|a| a.id == id)
        });

        let app = Self {
            state,
            image_loader: Arc::new(crate::image_loader::ImageLoader::new()),
            server_input,
            username_input,
            password_input,
            browser_search_input,
            mpv_path_input,
            player_stop_tx: None,
        };

        if let Some(account) = last_account {
            let this = cx.entity();
            let server = account.server.clone();
            let username = account.username.clone();
            let password = remby_core::crypto::decrypt(&account.password_enc).unwrap_or_default();
            let server2 = server.clone();
            let (tx, rx) = tokio::sync::oneshot::channel();
            crate::tokio_runtime().spawn(async move {
                let result = remby_core::emby::EmbyClient::authenticate(&server, &username, &password).await;
                let _ = tx.send(result);
            });
            cx.spawn(async move |_window, cx| {
                if let Ok(Ok(client)) = rx.await {
                    cx.update_entity(&this, |app, cx| {
                        app.state.client = Some(client);
                        app.state.server = server2;
                        app.state.navigate(View::Home);
                        app.load_home_data(cx);
                    });
                }
            })
            .detach();
        }

        app
    }

    pub fn handle_login(
        &mut self,
        server: String,
        username: String,
        password: String,
        cx: &mut Context<Self>,
    ) {
        self.state.login_error.clear();
        self.state.server = server.clone();

        let this = cx.entity();
        let server2 = server.clone();
        let username2 = username.clone();
        let password2 = password.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        crate::tokio_runtime().spawn(async move {
            let result = remby_core::emby::EmbyClient::authenticate(&server, &username, &password).await;
            let _ = tx.send(result);
        });
        cx.spawn(async move |_window, cx| {
            match rx.await {
                Ok(Ok(client)) => {
                    // Save account to accounts.json
                    let mut accounts_cfg = remby_core::config::load_accounts();
                    let existing = accounts_cfg.accounts.iter().position(|a| a.server == server2 && a.username == username2);
                    let account_id = if let Some(idx) = existing {
                        accounts_cfg.accounts[idx].password_enc = remby_core::crypto::encrypt(&password2);
                        accounts_cfg.accounts[idx].id.clone()
                    } else {
                        let id = uuid::Uuid::new_v4().to_string();
                        accounts_cfg.accounts.push(remby_core::config::Account {
                            id: id.clone(),
                            label: format!("{}@{}", username2, server2),
                            server: server2.clone(),
                            username: username2.clone(),
                            password_enc: remby_core::crypto::encrypt(&password2),
                        });
                        id
                    };
                    accounts_cfg.last_account_id = Some(account_id);
                    let _ = remby_core::config::save_accounts(&accounts_cfg);

                    cx.update_entity(&this, |app, cx| {
                        app.state.client = Some(client);
                        app.state.server = server2;
                        app.state.navigate(View::Home);
                        app.load_home_data(cx);
                    });
                }
                Ok(Err(e)) => {
                    let msg = e.to_string();
                    cx.update_entity(&this, |app, _cx| {
                        app.state.login_error = msg;
                        app.state.status_msg = "Login failed. Check server URL and credentials.".into();
                        app.state.status_kind = crate::state::StatusKind::Error;
                    });
                }
                Err(_) => {
                    cx.update_entity(&this, |app, _cx| {
                        app.state.login_error = "Internal error".into();
                        app.state.status_kind = crate::state::StatusKind::Error;
                    });
                }
            }
        })
        .detach();
    }

    pub fn load_posters(&self, item_ids: Vec<String>, cx: &mut Context<Self>) {
        if self.state.client.is_none() || item_ids.is_empty() {
            return;
        }
        let image_loader = self.image_loader.clone();
        let server = self.state.server.clone();
        let token = self.state.client.as_ref()
            .map(|c| c.token().to_string())
            .unwrap_or_default();

        let this = cx.entity();
        let (tx, mut rx) = tokio::sync::mpsc::channel(item_ids.len());
        crate::tokio_runtime().spawn(async move {
            for item_id in item_ids {
                if let Some(image) = image_loader.load_poster(&server, &token, &item_id).await {
                    let _ = tx.send((item_id, image)).await;
                }
            }
        });
        cx.spawn(async move |_window, cx| {
            while let Some((item_id, image)) = rx.recv().await {
                cx.update_entity(&this, |app, cx| {
                    app.state.poster_cache.insert(item_id, image);
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn load_home_data(&mut self, cx: &mut Context<Self>) {
        if self.state.client.is_none() {
            self.show_toast("Not connected to server".into(), crate::state::StatusKind::Error);
            return;
        }
        self.state.loading = true;
        self.state.loading_msg = "Loading home data...".into();

        let this = cx.entity();
        let client = self.state.client.clone().unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel();
        crate::tokio_runtime().spawn(async move {
            let cw = client.get_resume_items(20).await.unwrap_or_default();
            let latest = client.get_latest_items(20).await.unwrap_or_default();
            let following = client.get_latest_items(20).await.unwrap_or_default()
                .into_iter()
                .filter(|item| item.series_id.is_some())
                .collect();
            let _ = tx.send((cw, latest, following));
        });
        cx.spawn(async move |_window, cx| {
            if let Ok((cw, latest, following)) = rx.await {
                cx.update_entity(&this, |app, cx| {
                    app.state.continue_watching = cw;
                    app.state.latest_items = latest;
                    app.state.following_updates = following;
                    app.state.loading = false;
                    app.state.loading_msg.clear();
                    let ids: Vec<String> = app.state.continue_watching.iter()
                        .chain(app.state.latest_items.iter())
                        .chain(app.state.following_updates.iter())
                        .map(|i| i.id.clone())
                        .collect();
                    app.load_posters(ids, cx);
                });
            } else {
                cx.update_entity(&this, |app, _cx| {
                    app.state.status_msg = "Failed to load home data".into();
                    app.state.status_kind = crate::state::StatusKind::Error;
                    app.state.loading = false;
                    app.state.loading_msg.clear();
                });
            }
        })
        .detach();
    }

    pub fn load_libraries_data(&mut self, cx: &mut Context<Self>) {
        if self.state.client.is_none() {
            self.show_toast("Not connected to server".into(), crate::state::StatusKind::Error);
            return;
        }
        self.state.loading = true;
        self.state.loading_msg = "Loading libraries...".into();

        let this = cx.entity();
        let client = self.state.client.clone().unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel();
        crate::tokio_runtime().spawn(async move {
            let libraries = client.get_libraries().await.unwrap_or_default();
            let mut all_latest = Vec::new();
            for lib in &libraries {
                let items = client.get_latest_for_library(&lib.id, 10).await.unwrap_or_default();
                all_latest.extend(items);
            }
            let _ = tx.send((libraries, all_latest));
        });
        cx.spawn(async move |_window, cx| {
            if let Ok((libraries, latest)) = rx.await {
                cx.update_entity(&this, |app, cx| {
                    app.state.libraries = libraries;
                    app.state.latest_items = latest;
                    app.state.loading = false;
                    app.state.loading_msg.clear();
                    let ids: Vec<String> = app.state.latest_items.iter().map(|i| i.id.clone()).collect();
                    app.load_posters(ids, cx);
                });
            } else {
                cx.update_entity(&this, |app, _cx| {
                    app.state.status_msg = "Failed to load libraries".into();
                    app.state.status_kind = crate::state::StatusKind::Error;
                    app.state.loading = false;
                    app.state.loading_msg.clear();
                });
            }
        })
        .detach();
    }

    pub fn load_browser_data(&mut self, cx: &mut Context<Self>) {
        if self.state.client.is_none() {
            self.show_toast("Not connected to server".into(), crate::state::StatusKind::Error);
            return;
        }
        self.state.loading = true;
        self.state.browser_items.clear();
        self.state.browser_total = 0;

        let this = cx.entity();
        let client = self.state.client.clone().unwrap();
        let library_id = self.state.browser_library_id.clone();
        let sort_field = self.state.browser_sort_field.emby_key().to_string();
        let sort_order = self.state.browser_sort_order.emby_key().to_string();
        let filters = self.state.browser_filters.clone();

        let (tx, rx) = tokio::sync::oneshot::channel();
        crate::tokio_runtime().spawn(async move {
            let genres_str = if filters.genres.is_empty() { None } else { Some(filters.genres.join(",")) };
            let tags_str = if filters.tags.is_empty() { None } else { Some(filters.tags.join(",")) };
            let studios_str = if filters.studios.is_empty() { None } else { Some(filters.studios.join(",")) };
            let years_str = match (filters.year_min, filters.year_max) {
                (Some(min), Some(max)) => Some(format!("{}-{}", min, max)),
                (Some(min), None) => Some(min.to_string()),
                (None, Some(max)) => Some(max.to_string()),
                (None, None) => None,
            };

            let page = client.get_items_filtered(
                &library_id, 0, 50, &sort_field, &sort_order,
                genres_str.as_deref(), tags_str.as_deref(), studios_str.as_deref(), years_str.as_deref(),
            ).await.ok();

            let (genres, tags, studios) = tokio::join!(
                client.get_genres(&library_id),
                client.get_tags(&library_id),
                client.get_studios(&library_id),
            );

            let _ = tx.send((
                page.map(|p| (p.total, p.items)).unwrap_or_default(),
                genres.unwrap_or_default(),
                tags.unwrap_or_default(),
                studios.unwrap_or_default(),
            ));
        });
        cx.spawn(async move |_window, cx| {
            if let Ok(((total, items), genres, tags, studios)) = rx.await {
                cx.update_entity(&this, |app, cx| {
                    app.state.browser_items = items;
                    app.state.browser_total = total;
                    app.state.browser_available_genres = genres;
                    app.state.browser_available_tags = tags;
                    app.state.browser_available_studios = studios;
                    app.state.loading = false;
                    let ids: Vec<String> = app.state.browser_items.iter().map(|i| i.id.clone()).collect();
                    app.load_posters(ids, cx);
                });
            } else {
                cx.update_entity(&this, |app, _cx| {
                    app.state.status_msg = "Failed to load library items".into();
                    app.state.status_kind = crate::state::StatusKind::Error;
                    app.state.loading = false;
                });
            }
        })
        .detach();
    }

    pub fn load_browser_page(&mut self, start: usize, cx: &mut Context<Self>) {
        if self.state.client.is_none() || self.state.browser_loading_more {
            return;
        }
        self.state.browser_loading_more = true;

        let this = cx.entity();
        let client = self.state.client.clone().unwrap();
        let library_id = self.state.browser_library_id.clone();
        let sort_field = self.state.browser_sort_field.emby_key().to_string();
        let sort_order = self.state.browser_sort_order.emby_key().to_string();
        let filters = self.state.browser_filters.clone();

        let (tx, rx) = tokio::sync::oneshot::channel();
        crate::tokio_runtime().spawn(async move {
            let genres_str = if filters.genres.is_empty() { None } else { Some(filters.genres.join(",")) };
            let tags_str = if filters.tags.is_empty() { None } else { Some(filters.tags.join(",")) };
            let studios_str = if filters.studios.is_empty() { None } else { Some(filters.studios.join(",")) };
            let years_str = match (filters.year_min, filters.year_max) {
                (Some(min), Some(max)) => Some(format!("{}-{}", min, max)),
                (Some(min), None) => Some(min.to_string()),
                (None, Some(max)) => Some(max.to_string()),
                (None, None) => None,
            };
            let page = client.get_items_filtered(
                &library_id, start, 50, &sort_field, &sort_order,
                genres_str.as_deref(), tags_str.as_deref(), studios_str.as_deref(), years_str.as_deref(),
            ).await.ok();
            let _ = tx.send(page.map(|p| p.items));
        });
        cx.spawn(async move |_window, cx| {
            if let Ok(Some(items)) = rx.await {
                cx.update_entity(&this, |app, _cx| {
                    app.state.browser_items.extend(items);
                    app.state.browser_loading_more = false;
                });
            } else {
                cx.update_entity(&this, |app, _cx| {
                    app.state.browser_loading_more = false;
                });
            }
        })
        .detach();
    }

    pub fn search_browser(&mut self, query: &str, cx: &mut Context<Self>) {
        if self.state.client.is_none() {
            return;
        }

        let query = query.to_string();
        self.state.loading = true;
        self.state.browser_search_term = query.clone();

        let this = cx.entity();
        let client = self.state.client.clone().unwrap();
        let library_id = self.state.browser_library_id.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        crate::tokio_runtime().spawn(async move {
            let result = if query.is_empty() {
                client.get_items(&library_id, 0, 50).await.ok().map(|p| (p.total, p.items))
            } else {
                let items = client.search_in_library(&query, &library_id).await.unwrap_or_default();
                Some((items.len(), items))
            };
            let _ = tx.send(result);
        });
        cx.spawn(async move |_window, cx| {
            if let Ok(Some((total, items))) = rx.await {
                cx.update_entity(&this, |app, _cx| {
                    app.state.browser_items = items;
                    app.state.browser_total = total;
                    app.state.loading = false;
                });
            } else {
                cx.update_entity(&this, |app, _cx| {
                    app.state.loading = false;
                });
            }
        })
        .detach();
    }

    pub fn load_player_sources(&mut self, cx: &mut Context<Self>) {
        let item = match self.state.playing_item.as_ref() {
            Some(i) => i.clone(),
            None => return,
        };
        self.state.player_loading = true;

        let this = cx.entity();
        let client = self.state.client.clone().unwrap();
        let item_id = item.id.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        crate::tokio_runtime().spawn(async move {
            let sources = client.get_playback_info(&item_id).await.ok().unwrap_or_default();
            let _ = tx.send(sources);
        });
        cx.spawn(async move |_window, cx| {
            if let Ok(sources) = rx.await {
                cx.update_entity(&this, |app, _cx| {
                    app.state.player_loading = false;
                    app.state.player_sources = sources;
                    if app.state.player_sources.len() == 1 {
                        app.select_player_source(0, &item.id);
                    }
                });
            } else {
                cx.update_entity(&this, |app, _cx| {
                    app.state.player_loading = false;
                });
            }
        })
        .detach();
    }

    pub fn select_player_source(&mut self, idx: usize, _item_id: &str) {
        self.state.player_selected_source_idx = Some(idx);
        if let Some(source) = self.state.player_sources.get(idx) {
            let video: Vec<_> = source.media_streams.iter()
                .filter(|s| s.stream_type == "Video")
                .cloned()
                .collect();
            let audio: Vec<_> = source.media_streams.iter()
                .filter(|s| s.stream_type == "Audio")
                .cloned()
                .collect();
            let subs: Vec<_> = source.media_streams.iter()
                .filter(|s| s.stream_type == "Subtitle")
                .cloned()
                .collect();

            self.state.player_video_tracks = video;
            self.state.player_audio_tracks = audio;
            self.state.player_subtitle_tracks = subs;

            self.state.player_selected_video = if self.state.player_video_tracks.len() > 1 {
                Some(0)
            } else {
                None
            };
            self.state.player_selected_audio = if self.state.player_audio_tracks.len() > 1 {
                Some(0)
            } else {
                None
            };
            self.state.player_selected_subtitle = if self.state.player_subtitle_tracks.is_empty() {
                None
            } else {
                Some(-1)
            };
        }
    }

    pub fn launch_player(&mut self, resume: bool, start_secs: f64, cx: &mut Context<Self>) {
        let item = match self.state.playing_item.as_ref() {
            Some(i) => i.clone(),
            None => return,
        };
        let source_idx = self.state.player_selected_source_idx.unwrap_or(0);
        let source = match self.state.player_sources.get(source_idx) {
            Some(s) => s.clone(),
            None => return,
        };

        let client = match self.state.client.clone() {
            Some(c) => c,
            None => return,
        };

        let url = client.stream_url_for_source(&item, &source);
        let mpv_path = self.state.config.mpv_path.clone();
        let video = self.state.player_selected_video;
        let audio = self.state.player_selected_audio;
        let subtitle = self.state.player_selected_subtitle;
        let start = if resume { Some(start_secs) } else { None };

        let play_session_id = uuid::Uuid::new_v4().to_string().replace('-', "");
        let media_source_id = source.id.clone();

        self.state.playing_url = url.clone();
        self.state.player_play_session_id = play_session_id.clone();
        self.state.player_media_source_id = media_source_id.clone();
        self.state.player_logs.clear();
        self.state.player_position = 0.0;
        self.state.player_duration = 0.0;
        self.state.player_started = false;
        self.state.playing = true;

        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
        self.player_stop_tx = Some(stop_tx);

        let this = cx.entity();
        let report_client = self.state.client.clone();
        let item_id = item.id.clone();
        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<String>(64);
        let (_stop_tx_inner, stop_rx) = tokio::sync::oneshot::channel::<()>();

        crate::tokio_runtime().spawn(async move {
            let result = remby_core::mpv::play(&url, &mpv_path, video, audio, subtitle, start);
            let (_child, rx) = match result {
                Ok(r) => r,
                Err(e) => {
                    let _ = event_tx.send(format!("ERROR:{}", e)).await;
                    return;
                }
            };

            if let Some(c) = report_client.as_ref() {
                let _ = c.report_playback_start(&item_id, &media_source_id, &play_session_id).await;
            }

            let (mpv_tx, mut mpv_rx) = tokio::sync::mpsc::channel::<remby_core::mpv::MpvEvent>(64);
            std::thread::spawn(move || {
                while let Ok(event) = rx.recv() {
                    if mpv_tx.blocking_send(event).is_err() { break; }
                }
            });

            let mut stop_rx = stop_rx;
            loop {
                tokio::select! {
                    biased;
                    _ = &mut stop_rx => { break; }
                    event = mpv_rx.recv() => {
                        match event {
                            Some(remby_core::mpv::MpvEvent::Position(pos)) => {
                                let _ = event_tx.send(format!("POS:{}", pos)).await;
                            }
                            Some(remby_core::mpv::MpvEvent::Duration(dur)) => {
                                let _ = event_tx.send(format!("DUR:{}", dur)).await;
                            }
                            Some(remby_core::mpv::MpvEvent::PlaybackStarted) => {
                                let _ = event_tx.send("STARTED".into()).await;
                            }
                            Some(remby_core::mpv::MpvEvent::LogLine(line, _)) => {
                                let _ = event_tx.send(format!("LOG:{}", line)).await;
                            }
                            Some(remby_core::mpv::MpvEvent::PlaybackEnded) | None => { break; }
                        }
                    }
                }
            }

            let pos_ticks = 0i64;
            if let Some(c) = report_client.as_ref() {
                let _ = c.report_playback_stopped(&item_id, &media_source_id, &play_session_id, pos_ticks).await;
            }
        });

        cx.spawn(async move |_window, cx| {
            while let Some(msg) = event_rx.recv().await {
                if msg == "STOPPED" { break; }
                cx.update_entity(&this, |app, _cx| {
                    if let Some(rest) = msg.strip_prefix("POS:") {
                        app.state.player_position = rest.parse().unwrap_or(0.0);
                    } else if let Some(rest) = msg.strip_prefix("DUR:") {
                        app.state.player_duration = rest.parse().unwrap_or(0.0);
                    } else if msg == "STARTED" {
                        app.state.player_started = true;
                    } else if let Some(rest) = msg.strip_prefix("LOG:") {
                        app.state.player_logs.push(rest.to_string());
                        if app.state.player_logs.len() > 500 {
                            app.state.player_logs.drain(..250);
                        }
                    } else if let Some(rest) = msg.strip_prefix("ERROR:") {
                        app.state.playing = false;
                        app.state.player_logs.push(rest.to_string());
                        app.state.status_msg = format!("mpv error: {}", rest);
                        app.state.status_kind = crate::state::StatusKind::Error;
                    }
                });
            }
            cx.update_entity(&this, |app, _cx| {
                app.state.playing = false;
                app.state.player_started = false;
            });
        })
        .detach();
    }

    pub fn stop_player(&mut self) {
        if let Some(tx) = self.player_stop_tx.take() {
            let _ = tx.send(());
        }
    }

    pub fn reset_player(&mut self) {
        self.stop_player();
        self.state.playing_item = None;
        self.state.playing_url.clear();
        self.state.playing = false;
        self.state.player_sources.clear();
        self.state.player_selected_source_idx = None;
        self.state.player_video_tracks.clear();
        self.state.player_audio_tracks.clear();
        self.state.player_subtitle_tracks.clear();
        self.state.player_selected_video = None;
        self.state.player_selected_audio = None;
        self.state.player_selected_subtitle = None;
        self.state.player_position = 0.0;
        self.state.player_duration = 0.0;
        self.state.player_logs.clear();
        self.state.player_loading = false;
        self.state.player_started = false;
        self.state.player_play_session_id.clear();
        self.state.player_media_source_id.clear();
    }

    pub fn load_favorites(&mut self, cx: &mut Context<Self>) {
        if self.state.client.is_none() {
            self.show_toast("Not connected to server".into(), crate::state::StatusKind::Error);
            return;
        }
        self.state.loading = true;

        let this = cx.entity();
        let client = self.state.client.clone().unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel();
        crate::tokio_runtime().spawn(async move {
            let page = client.get_favorites(0, 100).await.ok();
            let _ = tx.send(page.map(|p| p.items).unwrap_or_default());
        });
        cx.spawn(async move |_window, cx| {
            if let Ok(favs) = rx.await {
                cx.update_entity(&this, |app, cx| {
                    app.state.favorites = favs;
                    app.state.loading = false;
                    let ids: Vec<String> = app.state.favorites.iter().map(|i| i.id.clone()).collect();
                    app.load_posters(ids, cx);
                });
            } else {
                cx.update_entity(&this, |app, _cx| {
                    app.state.status_msg = "Failed to load favorites".into();
                    app.state.status_kind = crate::state::StatusKind::Error;
                    app.state.loading = false;
                });
            }
        })
        .detach();
    }

    pub fn load_series_info(
        &mut self,
        series_id: &str,
        cx: &mut Context<Self>,
    ) {
        if self.state.client.is_none() {
            self.show_toast("Not connected to server".into(), crate::state::StatusKind::Error);
            return;
        }
        self.state.loading = true;
        let series_id = series_id.to_string();

        let this = cx.entity();
        let client = self.state.client.clone().unwrap();
        let sid = series_id.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        crate::tokio_runtime().spawn(async move {
            let item = client.get_item_detail(&sid).await.ok();
            let seasons = client.get_seasons(&sid).await.unwrap_or_default();
            let similar = client.get_similar(&sid).await.unwrap_or_default();
            let _ = tx.send((item, seasons, similar));
        });
        cx.spawn(async move |_window, cx| {
            if let Ok((item, seasons, similar)) = rx.await {
                cx.update_entity(&this, |app, _cx| {
                    app.state.series_item = item;
                    app.state.series_seasons = seasons;
                    app.state.series_similar = similar;
                    app.state.loading = false;
                });
            } else {
                cx.update_entity(&this, |app, _cx| {
                    app.state.status_msg = "Failed to load series info".into();
                    app.state.status_kind = crate::state::StatusKind::Error;
                    app.state.loading = false;
                });
            }
        })
        .detach();
    }

    pub fn load_series_episodes(
        &mut self,
        series_id: &str,
        section: &crate::state::SeriesSection,
        cx: &mut Context<Self>,
    ) {
        use crate::state::SeriesSection;

        if self.state.client.is_none() {
            return;
        }
        self.state.loading = true;
        let series_id = series_id.to_string();
        let section = section.clone();

        let this = cx.entity();
        let client = self.state.client.clone().unwrap();
        let sid = series_id.clone();
        let sec = section.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        crate::tokio_runtime().spawn(async move {
            let result = match sec {
                SeriesSection::Seasons => {
                    let seasons = client.get_seasons(&sid).await.unwrap_or_default();
                    (None, Some(seasons), None)
                }
                SeriesSection::Episodes => {
                    let episodes = client.get_episodes(&sid).await.unwrap_or_default();
                    (None, None, Some(episodes.0))
                }
                SeriesSection::Similar => {
                    let _similar = client.get_similar(&sid).await.unwrap_or_default();
                    (None, None, None)
                }
            };
            let _ = tx.send(result);
        });
        cx.spawn(async move |_window, cx| {
            if let Ok((item_opt, seasons_opt, episodes_opt)) = rx.await {
                cx.update_entity(&this, |app, _cx| {
                    if let Some(item) = item_opt {
                        app.state.series_item = Some(item);
                    }
                    if let Some(seasons) = seasons_opt {
                        app.state.series_seasons = seasons;
                    }
                    if let Some(episodes) = episodes_opt {
                        app.state.series_episodes = episodes;
                    }
                    app.state.loading = false;
                });
            }
        })
        .detach();
    }

    pub fn toggle_favorite(
        &mut self,
        item_id: &str,
        is_favorite: bool,
        cx: &mut Context<Self>,
    ) {
        if self.state.client.is_none() {
            return;
        }
        let item_id = item_id.to_string();
        let this = cx.entity();
        let client = self.state.client.clone().unwrap();
        let iid = item_id.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        crate::tokio_runtime().spawn(async move {
            let _ = client.toggle_favorite(&iid, is_favorite).await.ok();
            let item = client.get_item_detail(&iid).await.ok();
            let _ = tx.send(item);
        });
        cx.spawn(async move |_window, cx| {
            if let Ok(Some(updated_item)) = rx.await {
                cx.update_entity(&this, |app, _cx| {
                    if let Some(ref mut si) = app.state.series_item {
                        if si.id == item_id {
                            *si = updated_item.clone();
                        }
                    }
                    app.state.favorites.retain(|i| i.id != item_id);
                    if is_favorite {
                        app.state.favorites.push(updated_item);
                    }
                });
            }
        })
        .detach();
    }

    pub fn play_item(&mut self, item_id: &str, cx: &mut Context<Self>) {
        if self.state.client.is_none() {
            return;
        }
        let item_id = item_id.to_string();
        let this = cx.entity();
        let client = self.state.client.clone().unwrap();
        let iid = item_id.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        crate::tokio_runtime().spawn(async move {
            let item = client.get_item_detail(&iid).await.ok();
            let _ = tx.send(item);
        });
        cx.spawn(async move |_window, cx| {
            if let Ok(Some(item)) = rx.await {
                cx.update_entity(&this, |app, cx| {
                    app.state.playing_item = Some(item);
                    app.state.navigate(View::Player);
                    app.load_player_sources(cx);
                });
            }
        })
        .detach();
    }

    fn view_label(&self) -> &str {
        match self.state.view {
            View::Login => "Login",
            View::Home => "Home",
            View::Libraries => "Libraries",
            View::LibraryBrowser => "Library Browser",
            View::Favorites => "Favorites",
            View::SeriesInfo => "Series Info",
            View::Player => "Player",
            View::Settings => "Settings",
        }
    }

    fn window_title(&self) -> String {
        let view = self.view_label();
        match self.state.view {
            View::LibraryBrowser => format!("remby - {}", self.state.browser_library_name),
            _ => format!("remby - {}", view),
        }
    }

    fn show_toast(&mut self, msg: String, kind: crate::state::StatusKind) {
        self.state.status_msg = msg;
        self.state.status_kind = kind;
    }

    fn handle_go_back(&mut self, _: &GoBack, _window: &mut Window, cx: &mut Context<Self>) {
        self.state.go_back();
        cx.notify();
    }

    fn handle_quit(&mut self, _: &QuitApp, _window: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    fn handle_select_next(&mut self, _: &SelectNext, _window: &mut Window, cx: &mut Context<Self>) {
        if self.state.client.is_none() { return; }
        let is_input = matches!(self.state.view, View::Login | View::Settings | View::LibraryBrowser);
        if is_input { return; }
        match self.state.view {
            View::Home => { self.state.home_selected = self.state.home_selected.saturating_add(1); }
            View::Libraries => {
                let max = self.state.libraries.len().saturating_sub(1);
                self.state.libraries_selected = self.state.libraries_selected.min(max).saturating_add(1);
            }
            View::Favorites => {
                let max = self.state.favorites.len().saturating_sub(1);
                self.state.favorites_selected = self.state.favorites_selected.min(max).saturating_add(1);
            }
            View::LibraryBrowser => {
                let max = self.state.browser_items.len().saturating_sub(1);
                self.state.browser_selected = self.state.browser_selected.min(max).saturating_add(1);
            }
            View::SeriesInfo => { self.state.series_selected = self.state.series_selected.saturating_add(1); }
            _ => {}
        }
        cx.notify();
    }

    fn handle_select_prev(&mut self, _: &SelectPrev, _window: &mut Window, cx: &mut Context<Self>) {
        if self.state.client.is_none() { return; }
        let is_input = matches!(self.state.view, View::Login | View::Settings | View::LibraryBrowser);
        if is_input { return; }
        match self.state.view {
            View::Home => { self.state.home_selected = self.state.home_selected.saturating_sub(1); }
            View::Libraries => { self.state.libraries_selected = self.state.libraries_selected.saturating_sub(1); }
            View::Favorites => { self.state.favorites_selected = self.state.favorites_selected.saturating_sub(1); }
            View::LibraryBrowser => { self.state.browser_selected = self.state.browser_selected.saturating_sub(1); }
            View::SeriesInfo => { self.state.series_selected = self.state.series_selected.saturating_sub(1); }
            _ => {}
        }
        cx.notify();
    }

    fn handle_select_item(&mut self, _: &SelectItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.state.client.is_none() { return; }
        let is_input = matches!(self.state.view, View::Login | View::Settings | View::LibraryBrowser);
        if is_input { return; }
        match self.state.view {
            View::Home => {
                let idx = self.state.home_selected;
                let item_opt = self.state.continue_watching.get(idx).map(|i| {
                    (i.id.clone(), i.series_id.clone(), i.media_type.clone())
                });
                if let Some((item_id, series_id, media_type)) = item_opt {
                    if series_id.is_some() || media_type.as_deref() == Some("Series") {
                        let sid = series_id.unwrap_or(item_id);
                        self.state.navigate(View::SeriesInfo);
                        self.load_series_info(&sid, cx);
                    } else {
                        self.play_item(&item_id, cx);
                    }
                }
            }
            View::Libraries => {
                let idx = self.state.libraries_selected;
                if let Some(lib) = self.state.libraries.get(idx) {
                    let lib_id = lib.id.clone();
                    let lib_name = lib.name.clone();
                    self.state.browser_library_id = lib_id;
                    self.state.browser_library_name = lib_name;
                    self.state.navigate(View::LibraryBrowser);
                    cx.notify();
                }
            }
            View::LibraryBrowser | View::Favorites => {
                let items = match self.state.view {
                    View::LibraryBrowser => &self.state.browser_items,
                    View::Favorites => &self.state.favorites,
                    _ => unreachable!(),
                };
                let idx = match self.state.view {
                    View::LibraryBrowser => self.state.browser_selected,
                    View::Favorites => self.state.favorites_selected,
                    _ => unreachable!(),
                };
                let item_opt = items.get(idx).map(|i| {
                    (i.id.clone(), i.series_id.clone(), i.media_type.clone())
                });
                if let Some((item_id, series_id, media_type)) = item_opt {
                    if series_id.is_some() || media_type.as_deref() == Some("Series") {
                        let sid = series_id.unwrap_or(item_id);
                        self.state.navigate(View::SeriesInfo);
                        self.load_series_info(&sid, cx);
                    } else {
                        self.play_item(&item_id, cx);
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_toggle_favorite(&mut self, _: &ToggleFavorite, _window: &mut Window, cx: &mut Context<Self>) {
        if self.state.client.is_none() { return; }
        let is_input = matches!(self.state.view, View::Login | View::Settings | View::LibraryBrowser);
        if is_input { return; }
        match self.state.view {
            View::LibraryBrowser => {
                if let Some(item) = self.state.browser_items.get(self.state.browser_selected) {
                    let item_id = item.id.clone();
                    let is_fav = item.user_data.as_ref().map(|u| u.is_favorite).unwrap_or(false);
                    self.toggle_favorite(&item_id, !is_fav, cx);
                }
            }
            View::Home => {
                if let Some(item) = self.state.continue_watching.get(self.state.home_selected) {
                    let item_id = item.id.clone();
                    let is_fav = item.user_data.as_ref().map(|u| u.is_favorite).unwrap_or(false);
                    self.toggle_favorite(&item_id, !is_fav, cx);
                }
            }
            _ => {}
        }
    }

    fn handle_toggle_follow(&mut self, _: &ToggleFollow, _window: &mut Window, cx: &mut Context<Self>) {
        if self.state.client.is_none() { return; }
        if self.state.view == View::SeriesInfo {
            if let Some(ref item) = self.state.series_item {
                let item_id = item.id.clone();
                let is_fav = item.user_data.as_ref().map(|u| u.is_favorite).unwrap_or(false);
                self.toggle_favorite(&item_id, !is_fav, cx);
            }
        }
    }

    fn handle_navigate_settings(&mut self, _: &NavigateSettings, _window: &mut Window, cx: &mut Context<Self>) {
        if self.state.client.is_none() { return; }
        let is_input = matches!(self.state.view, View::Login | View::Settings | View::LibraryBrowser);
        if is_input { return; }
        self.state.navigate(View::Settings);
        cx.notify();
    }

    fn handle_navigate_libraries(&mut self, _: &NavigateLibraries, _window: &mut Window, cx: &mut Context<Self>) {
        if self.state.client.is_none() { return; }
        let is_input = matches!(self.state.view, View::Login | View::Settings | View::LibraryBrowser);
        if is_input { return; }
        self.state.navigate(View::Libraries);
        cx.notify();
    }

    fn handle_navigate_home(&mut self, _: &NavigateHome, _window: &mut Window, cx: &mut Context<Self>) {
        if self.state.client.is_none() { return; }
        let is_input = matches!(self.state.view, View::Login | View::Settings | View::LibraryBrowser);
        if is_input { return; }
        self.state.navigate(View::Home);
        cx.notify();
    }
}

impl Render for RembyApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_window_title(&self.window_title());

        let status_msg = self.state.status_msg.clone();
        let status_kind = self.state.status_kind.clone();

        let view_element: AnyElement = match self.state.view {
            View::Login => {
                let this = cx.entity();
                LoginView::new(
                    self.server_input.clone(),
                    self.username_input.clone(),
                    self.password_input.clone(),
                    this.downgrade(),
                )
                .into_any_element()
            }
            View::Home => {
                let this = cx.entity();
                HomeView::new(this.downgrade()).into_any_element()
            }
            View::Libraries => {
                let this = cx.entity();
                if self.state.libraries.is_empty() && !self.state.loading {
                    cx.spawn({
                        let this = this.clone();
                        async move |_window, cx| {
                            cx.update_entity(&this, |app, cx| {
                                app.load_libraries_data(cx);
                            });
                        }
                    })
                    .detach();
                }
                LibrariesView::new(this.downgrade()).into_any_element()
            }
            View::LibraryBrowser => {
                let this = cx.entity();
                if self.state.browser_items.is_empty() && !self.state.loading {
                    cx.spawn({
                        let this = this.clone();
                        async move |_window, cx| {
                            cx.update_entity(&this, |app, cx| {
                                app.load_browser_data(cx);
                            });
                        }
                    })
                    .detach();
                }
                BrowserView::new(this.downgrade(), self.browser_search_input.clone()).into_any_element()
            }
            View::Player => {
                let this = cx.entity();
                if !self.state.playing
                    && !self.state.player_loading
                    && self.state.player_sources.is_empty()
                    && self.state.playing_item.is_some()
                {
                    cx.spawn({
                        let this = this.clone();
                        async move |_window, cx| {
                            cx.update_entity(&this, |app, cx| {
                                app.load_player_sources(cx);
                            });
                        }
                    })
                    .detach();
                }
                PlayerView::new(this.downgrade()).into_any_element()
            }
            View::Settings => {
                let this = cx.entity();
                SettingsView::new(this.downgrade(), self.mpv_path_input.clone()).into_any_element()
            }
            View::Favorites => {
                let this = cx.entity();
                if self.state.favorites.is_empty() && !self.state.loading {
                    cx.spawn({
                        let this = this.clone();
                        async move |_window, cx| {
                            cx.update_entity(&this, |app, cx| {
                                app.load_favorites(cx);
                            });
                        }
                    })
                    .detach();
                }
                FavoritesView::new(this.downgrade()).into_any_element()
            }
            View::SeriesInfo => {
                let this = cx.entity();
                if self.state.series_seasons.is_empty()
                    && self.state.series_episodes.is_empty()
                    && self.state.series_item.is_some()
                    && !self.state.loading
                {
                    let series_id = self.state.series_item.as_ref().map(|i| i.id.clone()).unwrap_or_default();
                    let section = self.state.series_section.clone();
                    cx.spawn({
                        let this = this.clone();
                        let series_id = series_id.clone();
                        async move |_window, cx| {
                            cx.update_entity(&this, |app, cx| {
                                app.load_series_episodes(&series_id, &section, cx);
                            });
                        }
                    })
                    .detach();
                }
                SeriesView::new(this.downgrade()).into_any_element()
            }
        };

        let has_toast = !status_msg.is_empty();
        let toast_bg = match status_kind {
            crate::state::StatusKind::Info => cx.theme().info.opacity(0.15),
            crate::state::StatusKind::Success => cx.theme().success.opacity(0.15),
            crate::state::StatusKind::Error => cx.theme().danger.opacity(0.15),
            crate::state::StatusKind::Loading => cx.theme().muted.opacity(0.15),
        };
        let toast_border = match status_kind {
            crate::state::StatusKind::Info => cx.theme().info,
            crate::state::StatusKind::Success => cx.theme().success,
            crate::state::StatusKind::Error => cx.theme().danger,
            crate::state::StatusKind::Loading => cx.theme().muted,
        };
        let sidebar = if !matches!(self.state.view, View::Login) {
            let this = cx.entity();
            Some(
                SidebarNav::new(self.state.view.clone())
                    .on_navigate(move |view, _window, cx| {
                        this.update(cx, |app, cx| {
                            app.state.navigate(view);
                            match app.state.view {
                                View::Home => app.load_home_data(cx),
                                View::Libraries => app.load_libraries_data(cx),
                                View::Favorites => app.load_favorites(cx),
                                _ => {}
                            }
                            cx.notify();
                        });
                    }),
            )
        } else {
            None
        };

        v_flex()
            .id("remby-app")
            .size_full()
            .on_action(cx.listener(Self::handle_go_back))
            .on_action(cx.listener(Self::handle_quit))
            .on_action(cx.listener(Self::handle_select_next))
            .on_action(cx.listener(Self::handle_select_prev))
            .on_action(cx.listener(Self::handle_select_item))
            .on_action(cx.listener(Self::handle_toggle_favorite))
            .on_action(cx.listener(Self::handle_toggle_follow))
            .on_action(cx.listener(Self::handle_navigate_settings))
            .on_action(cx.listener(Self::handle_navigate_libraries))
            .on_action(cx.listener(Self::handle_navigate_home))
            .when(has_toast, |this| {
                this.child(
                    div()
                        .px_4()
                        .py_2()
                        .rounded(cx.theme().radius)
                        .mx_2()
                        .mt_2()
                        .bg(toast_bg)
                        .border_1()
                        .border_color(toast_border)
                        .text_sm()
                        .child(status_msg),
                )
            })
            .child(
                h_flex()
                    .flex_1()
                    .when_some(sidebar, |this, sidebar| this.child(sidebar))
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .child(view_element),
                    ),
            )
    }
}
