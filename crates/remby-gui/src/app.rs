use gpui::*;
use gpui_component::*;
use gpui_component::input::InputState;

use crate::state::{GuiState, View};
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
                    let _ = cx.update_entity(&this, |app, _cx| {
                        app.state.client = Some(client);
                        app.state.navigate(View::Home);
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
