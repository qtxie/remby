use gpui::*;
use gpui_component::*;
use gpui_component::spinner::Spinner;

pub struct LoadingIndicator {
    message: String,
}

impl LoadingIndicator {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl RenderOnce for LoadingIndicator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .items_center()
            .gap_3()
            .child(Spinner::new().small())
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.message),
            )
    }
}
