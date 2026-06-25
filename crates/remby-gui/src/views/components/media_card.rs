use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::*;

use super::badge::{Badge, BadgeVariant};
use super::progress::Progress;

#[derive(IntoElement)]
pub struct MediaCard {
    id: SharedString,
    title: SharedString,
    subtitle: SharedString,
    poster_url: Option<String>,
    poster_image: Option<Arc<Image>>,
    on_click: Option<Box<dyn Fn(&mut Window, &mut App)>>,
    badge: Option<SharedString>,
    badge_variant: BadgeVariant,
    progress: Option<f32>,
}

impl MediaCard {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: SharedString::default(),
            subtitle: SharedString::default(),
            poster_url: None,
            poster_image: None,
            on_click: None,
            badge: None,
            badge_variant: BadgeVariant::Default,
            progress: None,
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

    pub fn poster_image(mut self, image: Option<Arc<Image>>) -> Self {
        self.poster_image = image;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn badge(mut self, text: impl Into<SharedString>, variant: BadgeVariant) -> Self {
        self.badge = Some(text.into());
        self.badge_variant = variant;
        self
    }

    pub fn progress(mut self, value: f32) -> Self {
        self.progress = Some(value);
        self
    }
}

impl RenderOnce for MediaCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let poster_area = if let Some(image) = self.poster_image {
            div()
                .id(self.id.clone())
                .w_full()
                .h(px(270.))
                .rounded(px(8.))
                .overflow_hidden()
                .child(img(image).w_full().h_full().object_fit(gpui::ObjectFit::Cover))
        } else {
            div()
                .id(self.id.clone())
                .w_full()
                .h(px(270.))
                .bg(cx.theme().muted.opacity(0.15))
                .rounded(px(8.))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Icon::new(IconName::Frame)
                        .large()
                        .text_color(cx.theme().muted_foreground.opacity(0.3)),
                )
        };

        let poster_with_badge = div()
            .relative()
            .child(poster_area)
            .when_some(self.badge, |this, text| {
                this.child(
                    div()
                        .absolute()
                        .top(px(8.))
                        .left(px(8.))
                        .child(Badge::new(text).variant(self.badge_variant)),
                )
            });

        let wrapper = div()
            .id(format!("{}-wrapper", self.id))
            .w(px(160.))
            .rounded(px(8.))
            .overflow_hidden()
            .cursor_pointer()
            .hover(|this| {
                this.shadow_lg()
                    .bg(cx.theme().muted.opacity(0.05))
            });

        let wrapper = if let Some(handler) = self.on_click {
            wrapper.on_click(move |_event: &ClickEvent, window, cx| handler(window, cx))
        } else {
            wrapper
        };

        wrapper.child(
                v_flex()
                    .gap_2()
                    .child(poster_with_badge)
                    .when_some(self.progress, |this, value| {
                        this.child(Progress::new(value))
                    })
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
