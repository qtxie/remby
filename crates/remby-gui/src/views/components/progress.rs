use gpui::*;
use gpui_component::*;

#[derive(IntoElement)]
pub struct Progress {
    value: f32,
    height: Pixels,
}

impl Progress {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            height: px(4.),
        }
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }
}

impl RenderOnce for Progress {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .w_full()
            .h(self.height)
            .rounded_full()
            .bg(cx.theme().muted.opacity(0.2))
            .child(
                div()
                    .h_full()
                    .w(relative(self.value))
                    .rounded_full()
                    .bg(cx.theme().primary),
            )
    }
}
