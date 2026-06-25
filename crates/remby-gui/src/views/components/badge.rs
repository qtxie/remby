use gpui::*;
use gpui_component::*;

#[derive(IntoElement)]
pub struct Badge {
    text: SharedString,
    variant: BadgeVariant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Default,
    Success,
    Warning,
    Error,
}

impl Badge {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::Default,
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (bg, fg) = match self.variant {
            BadgeVariant::Default => (cx.theme().primary.opacity(0.1), cx.theme().primary),
            BadgeVariant::Success => (cx.theme().success.opacity(0.1), cx.theme().success),
            BadgeVariant::Warning => (cx.theme().warning.opacity(0.1), cx.theme().warning),
            BadgeVariant::Error => (cx.theme().danger.opacity(0.1), cx.theme().danger),
        };

        div()
            .px_2()
            .py_1()
            .rounded_full()
            .bg(bg)
            .text_xs()
            .font_medium()
            .text_color(fg)
            .child(self.text)
    }
}
