use gpui::*;
use gpui_component::*;
use gpui_component::input::InputState;

use crate::state::{GuiState, View};
use crate::views::home::HomeView;
use crate::views::login::LoginView;

pub struct RembyApp {
    pub state: GuiState,
    server_input: Entity<InputState>,
    username_input: Entity<InputState>,
    password_input: Entity<InputState>,
}

impl RembyApp {
    pub fn new(
        server_input: Entity<InputState>,
        username_input: Entity<InputState>,
        password_input: Entity<InputState>,
        _cx: &mut Context<Self>,
    ) -> Self {
        let mut state = GuiState::new();
        state.config = remby_core::config::load_config();

        Self {
            state,
            server_input,
            username_input,
            password_input,
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
