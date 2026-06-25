use gpui::*;
use gpui_component::*;

use crate::state::StatusKind;

pub struct Toast {
    message: String,
    kind: StatusKind,
}

impl Toast {
    pub fn new(message: impl Into<String>, kind: StatusKind) -> Self {
        Self {
            message: message.into(),
            kind,
        }
    }
}

impl RenderOnce for Toast {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (bg, border, icon) = match self.kind {
            StatusKind::Info => (
                cx.theme().info.opacity(0.15),
                cx.theme().info,
                IconName::Info,
            ),
            StatusKind::Success => (
                cx.theme().success.opacity(0.15),
                cx.theme().success,
                IconName::CircleCheck,
            ),
            StatusKind::Error => (
                cx.theme().danger.opacity(0.15),
                cx.theme().danger,
                IconName::TriangleAlert,
            ),
            StatusKind::Loading => (
                cx.theme().muted.opacity(0.15),
                cx.theme().muted,
                IconName::Loader,
            ),
        };

        h_flex()
            .items_center()
            .gap_3()
            .px_4()
            .py_3()
            .rounded(cx.theme().radius)
            .bg(bg)
            .border_1()
            .border_color(border)
            .child(Icon::new(icon).small().text_color(border))
            .child(
                div()
                    .text_sm()
                    .child(self.message),
            )
    }
}
