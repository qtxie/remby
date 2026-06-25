use gpui::*;
use gpui_component::*;
use gpui_component::input::{Input, InputState};

#[derive(IntoElement)]
pub struct SearchBar {
    input_state: Entity<InputState>,
}

impl SearchBar {
    pub fn new(input_state: Entity<InputState>) -> Self {
        Self { input_state }
    }
}

impl RenderOnce for SearchBar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        Input::new(&self.input_state)
            .small()
            .cleanable(true)
            .prefix(Icon::new(IconName::Search).small())
    }
}
