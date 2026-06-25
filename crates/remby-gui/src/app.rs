use gpui::*;
use gpui_component::*;
use gpui_component::input::InputState;

use crate::state::{GuiState, View};
use crate::views::browser::BrowserView;
use crate::views::home::HomeView;
use crate::views::libraries::LibrariesView;
use crate::views::login::LoginView;
use crate::views::player::PlayerView;
use crate::views::settings::SettingsView;

pub struct RembyApp {
    pub state: GuiState,
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
        _cx: &mut Context<Self>,
    ) -> Self {
        let mut state = GuiState::new();
        state.config = remby_core::config::load_config();

        Self {
            state,
            server_input,
            username_input,
            password_input,
            browser_search_input,
            mpv_path_input,
            player_stop_tx: None,
        }
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
        cx.spawn(async move |_window, cx| {
            match remby_core::emby::EmbyClient::authenticate(&server, &username, &password).await
            {
                Ok(client) => {
                    let _ = cx.update_entity(&this, |app, cx| {
                        app.state.client = Some(client);
                        app.state.navigate(View::Home);
                        app.load_home_data(cx);
                    });
                }
                Err(e) => {
                    let msg = e.to_string();
                    let _ = cx.update_entity(&this, |app, _cx| {
                        app.state.login_error = msg;
                    });
                }
            }
        })
        .detach();
    }

    fn load_home_data(&mut self, cx: &mut Context<Self>) {
        if self.state.client.is_none() {
            return;
        }
        self.state.loading = true;
        self.state.loading_msg = "Loading home data...".into();

        let this = cx.entity();
        cx.spawn(async move |_window, cx| {
            let _ = cx.update_entity(&this, |app, _cx| {
                app.state.loading_msg = "Loading continue watching...".into();
            });

            let result = async {
                let client = cx.read_entity(&this, |app, _| app.state.client.clone())?;
                let cw = client.get_resume_items(20).await.unwrap_or_default();

                let _ = cx.update_entity(&this, |app, _cx| {
                    app.state.continue_watching = cw;
                    app.state.loading_msg = "Loading latest items...".into();
                });

                let latest = client.get_latest_items(20).await.unwrap_or_default();

                let _ = cx.update_entity(&this, |app, _cx| {
                    app.state.latest_items = latest;
                    app.state.loading_msg = "Loading following updates...".into();
                });

                let following = client
                    .get_latest_items(20)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|item| item.series_id.is_some())
                    .collect();

                Some(following)
            }
            .await;

            let _ = cx.update_entity(&this, |app, _cx| {
                if let Some(following) = result {
                    app.state.following_updates = following;
                }
                app.state.loading = false;
                app.state.loading_msg.clear();
            });
        })
        .detach();
    }

    pub fn load_libraries_data(&mut self, cx: &mut Context<Self>) {
        if self.state.client.is_none() {
            return;
        }
        self.state.loading = true;
        self.state.loading_msg = "Loading libraries...".into();

        let this = cx.entity();
        cx.spawn(async move |_window, cx| {
            let result = async {
                let client = cx.read_entity(&this, |app, _| app.state.client.clone())?;
                let libraries = client.get_libraries().await.unwrap_or_default();

                let _ = cx.update_entity(&this, |app, _cx| {
                    app.state.libraries = libraries.clone();
                    app.state.loading_msg = "Loading latest items...".into();
                });

                let mut all_latest = Vec::new();
                for lib in &libraries {
                    let items = client
                        .get_latest_for_library(&lib.id, 10)
                        .await
                        .unwrap_or_default();
                    all_latest.extend(items);
                }

                Some(all_latest)
            }
            .await;

            let _ = cx.update_entity(&this, |app, _cx| {
                if let Some(latest) = result {
                    app.state.latest_items = latest;
                }
                app.state.loading = false;
                app.state.loading_msg.clear();
            });
        })
        .detach();
    }

    pub fn load_browser_data(&mut self, cx: &mut Context<Self>) {
        if self.state.client.is_none() {
            return;
        }
        self.state.loading = true;
        self.state.browser_items.clear();
        self.state.browser_total = 0;

        let this = cx.entity();
        cx.spawn(async move |_window, cx| {
            let result = async {
                let (client, library_id, sort_field, sort_order, filters) = cx.read_entity(&this, |app, _| {
                    (
                        app.state.client.clone(),
                        app.state.browser_library_id.clone(),
                        app.state.browser_sort_field.emby_key().to_string(),
                        app.state.browser_sort_order.emby_key().to_string(),
                        app.state.browser_filters.clone(),
                    )
                });

                let client = client?;

                let genres_str = if filters.genres.is_empty() { None } else { Some(filters.genres.join(",")) };
                let tags_str = if filters.tags.is_empty() { None } else { Some(filters.tags.join(",")) };
                let studios_str = if filters.studios.is_empty() { None } else { Some(filters.studios.join(",")) };
                let years_str = match (filters.year_min, filters.year_max) {
                    (Some(min), Some(max)) => Some(format!("{}-{}", min, max)),
                    (Some(min), None) => Some(min.to_string()),
                    (None, Some(max)) => Some(max.to_string()),
                    (None, None) => None,
                };

                let page = client
                    .get_items_filtered(
                        &library_id,
                        0,
                        50,
                        &sort_field,
                        &sort_order,
                        genres_str.as_deref(),
                        tags_str.as_deref(),
                        studios_str.as_deref(),
                        years_str.as_deref(),
                    )
                    .await
                    .ok();

                let (genres, tags, studios) = tokio::join!(
                    client.get_genres(&library_id),
                    client.get_tags(&library_id),
                    client.get_studios(&library_id),
                );

                Some((
                    page.map(|p| (p.total, p.items)).unwrap_or_default(),
                    genres.unwrap_or_default(),
                    tags.unwrap_or_default(),
                    studios.unwrap_or_default(),
                ))
            }
            .await;

            let _ = cx.update_entity(&this, |app, _cx| {
                if let Some(((total, items), genres, tags, studios)) = result {
                    app.state.browser_items = items;
                    app.state.browser_total = total;
                    app.state.browser_available_genres = genres;
                    app.state.browser_available_tags = tags;
                    app.state.browser_available_studios = studios;
                }
                app.state.loading = false;
            });
        })
        .detach();
    }

    pub fn load_browser_page(&mut self, start: usize, cx: &mut Context<Self>) {
        if self.state.client.is_none() || self.state.browser_loading_more {
            return;
        }
        self.state.browser_loading_more = true;

        let this = cx.entity();
        cx.spawn(async move |_window, cx| {
            let result = async {
                let (client, library_id, sort_field, sort_order, filters) = cx.read_entity(&this, |app, _| {
                    (
                        app.state.client.clone(),
                        app.state.browser_library_id.clone(),
                        app.state.browser_sort_field.emby_key().to_string(),
                        app.state.browser_sort_order.emby_key().to_string(),
                        app.state.browser_filters.clone(),
                    )
                });

                let client = client?;

                let genres_str = if filters.genres.is_empty() { None } else { Some(filters.genres.join(",")) };
                let tags_str = if filters.tags.is_empty() { None } else { Some(filters.tags.join(",")) };
                let studios_str = if filters.studios.is_empty() { None } else { Some(filters.studios.join(",")) };
                let years_str = match (filters.year_min, filters.year_max) {
                    (Some(min), Some(max)) => Some(format!("{}-{}", min, max)),
                    (Some(min), None) => Some(min.to_string()),
                    (None, Some(max)) => Some(max.to_string()),
                    (None, None) => None,
                };

                let page = client
                    .get_items_filtered(
                        &library_id,
                        start,
                        50,
                        &sort_field,
                        &sort_order,
                        genres_str.as_deref(),
                        tags_str.as_deref(),
                        studios_str.as_deref(),
                        years_str.as_deref(),
                    )
                    .await
                    .ok();

                page.map(|p| p.items)
            }
            .await;

            let _ = cx.update_entity(&this, |app, _cx| {
                if let Some(items) = result {
                    app.state.browser_items.extend(items);
                }
                app.state.browser_loading_more = false;
            });
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
        cx.spawn(async move |_window, cx| {
            let result = async {
                let (client, library_id) = cx.read_entity(&this, |app, _| {
                    (
                        app.state.client.clone(),
                        app.state.browser_library_id.clone(),
                    )
                });

                let client = client?;

                if query.is_empty() {
                    let page = client.get_items(&library_id, 0, 50).await.ok();
                    return page.map(|p| (p.total, p.items));
                }

                let items = client
                    .search_in_library(&query, &library_id)
                    .await
                    .unwrap_or_default();
                Some((items.len(), items))
            }
            .await;

            let _ = cx.update_entity(&this, |app, _cx| {
                if let Some((total, items)) = result {
                    app.state.browser_items = items;
                    app.state.browser_total = total;
                }
                app.state.loading = false;
            });
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
        cx.spawn(async move |_window, cx| {
            let result = async {
                let client = cx.read_entity(&this, |app, _| app.state.client.clone())?;
                let sources = client.get_playback_info(&item.id).await.ok();
                Some(sources.unwrap_or_default())
            }
            .await;

            let _ = cx.update_entity(&this, |app, _cx| {
                app.state.player_loading = false;
                if let Some(sources) = result {
                    app.state.player_sources = sources;
                    if app.state.player_sources.len() == 1 {
                        app.select_player_source(0, &item.id);
                    }
                }
            });
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
        cx.spawn(async move |_window, cx| {
            let result = remby_core::mpv::play(&url, &mpv_path, video, audio, subtitle, start);

            let (_child, rx) = match result {
                Ok(r) => r,
                Err(e) => {
                    let _ = cx.update_entity(&this, |app, _cx| {
                        app.state.playing = false;
                        app.state.player_logs.push(format!("Failed to start mpv: {}", e));
                    });
                    return;
                }
            };

            let item_id = item.id.clone();
            let ms_id = media_source_id.clone();
            let ps_id = play_session_id.clone();
            tokio::spawn(async move {
                let _ = client.report_playback_start(&item_id, &ms_id, &ps_id).await;
            });

            let (tokio_tx, mut tokio_rx) = tokio::sync::mpsc::channel::<remby_core::mpv::MpvEvent>(64);
            std::thread::spawn(move || {
                while let Ok(event) = rx.recv() {
                    if tokio_tx.blocking_send(event).is_err() {
                        break;
                    }
                }
            });

            let mut stop_rx = stop_rx;
            loop {
                tokio::select! {
                    biased;
                    _ = &mut stop_rx => {
                        break;
                    }
                    event = tokio_rx.recv() => {
                        match event {
                            Some(remby_core::mpv::MpvEvent::Position(pos)) => {
                                let _ = cx.update_entity(&this, |app, _cx| {
                                    app.state.player_position = pos;
                                });
                            }
                            Some(remby_core::mpv::MpvEvent::Duration(dur)) => {
                                let _ = cx.update_entity(&this, |app, _cx| {
                                    app.state.player_duration = dur;
                                });
                            }
                            Some(remby_core::mpv::MpvEvent::PlaybackStarted) => {
                                let _ = cx.update_entity(&this, |app, _cx| {
                                    app.state.player_started = true;
                                });
                            }
                            Some(remby_core::mpv::MpvEvent::LogLine(line, _level)) => {
                                let _ = cx.update_entity(&this, |app, _cx| {
                                    app.state.player_logs.push(line);
                                    if app.state.player_logs.len() > 500 {
                                        app.state.player_logs.drain(..250);
                                    }
                                });
                            }
                            Some(remby_core::mpv::MpvEvent::PlaybackEnded) | None => {
                                break;
                            }
                        }
                    }
                }
            }

            let _ = cx.update_entity(&this, |app, _cx| {
                let pos_ticks = (app.state.player_position * 10_000_000.0) as i64;
                let item_id = app.state.playing_item.as_ref().map(|i| i.id.clone()).unwrap_or_default();
                let ms_id = app.state.player_media_source_id.clone();
                let ps_id = app.state.player_play_session_id.clone();
                app.state.playing = false;
                app.state.player_started = false;
                let client = app.state.client.clone();
                tokio::spawn(async move {
                    if let Some(c) = client {
                        let _ = c.report_playback_stopped(&item_id, &ms_id, &ps_id, pos_ticks).await;
                    }
                });
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
}

impl Render for RembyApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.state.view {
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
                            let _ = cx.update_entity(&this, |app, cx| {
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
                            let _ = cx.update_entity(&this, |app, cx| {
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
                            let _ = cx.update_entity(&this, |app, cx| {
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
            _ => div()
                .size_full()
                .v_flex()
                .items_center()
                .justify_center()
                .child(format!("remby - {}", self.view_label()))
                .into_any_element(),
        }
    }
}
