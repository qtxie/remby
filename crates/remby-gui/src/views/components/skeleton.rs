use gpui::*;
use gpui_component::*;

#[derive(IntoElement)]
pub struct Skeleton {
    width: AbsoluteLength,
    height: AbsoluteLength,
    radius: AbsoluteLength,
}

impl Skeleton {
    pub fn new(width: AbsoluteLength, height: AbsoluteLength) -> Self {
        Self {
            width,
            height,
            radius: AbsoluteLength::Pixels(px(4.)),
        }
    }

    pub fn rounded(mut self, radius: AbsoluteLength) -> Self {
        self.radius = radius;
        self
    }
}

impl RenderOnce for Skeleton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .w(self.width)
            .h(self.height)
            .rounded(self.radius)
            .bg(cx.theme().muted.opacity(0.2))
            .child(
                div()
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        Icon::new(IconName::Frame)
                            .large()
                            .text_color(cx.theme().muted_foreground.opacity(0.15)),
                    ),
            )
    }
}
