use gpui::*;
use gpui_component::*;

use crate::state::{GuiState, View};

pub struct RembyApp {
    state: GuiState,
}

impl RembyApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let mut state = GuiState::new();
        state.config = remby_core::config::load_config();
        Self { state }
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
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .v_flex()
            .items_center()
            .justify_center()
            .child(format!("remby - {}", self.view_label()))
    }
}
