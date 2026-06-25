use gpui::*;
use gpui_component::*;

pub struct MediaCard {
    id: SharedString,
    title: SharedString,
    subtitle: SharedString,
    poster_url: Option<String>,
}

impl MediaCard {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: SharedString::default(),
            subtitle: SharedString::default(),
            poster_url: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn poster(mut self, url: impl Into<String>) -> Self {
        self.poster_url = Some(url.into());
        self
    }
}

impl RenderOnce for MediaCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .id(self.id)
            .w(px(150.))
            .rounded_lg()
            .overflow_hidden()
            .cursor_pointer()
            .hover(|this| this.opacity(0.9))
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .w_full()
                            .h(px(220.))
                            .bg(cx.theme().muted.opacity(0.15))
                            .rounded(cx.theme().radius)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Icon::new(IconName::Frame)
                                    .large()
                                    .text_color(cx.theme().muted_foreground.opacity(0.3)),
                            ),
                    )
                    .child(
                        v_flex()
                            .gap_1()
                            .px_2()
                            .pb_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .overflow_x_hidden()
                                    .child(self.title),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .overflow_x_hidden()
                                    .child(self.subtitle),
                            ),
                    ),
            )
    }
}
