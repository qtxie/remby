use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::*;

#[derive(IntoElement)]
pub struct ContinueWatchingCard {
    id: SharedString,
    title: SharedString,
    subtitle: SharedString,
    image: Option<Arc<Image>>,
    progress: Option<f32>,
    on_click: Option<Box<dyn Fn(&mut Window, &mut App)>>,
}

impl ContinueWatchingCard {
    pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: SharedString::default(),
            image: None,
            progress: None,
            on_click: None,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn image(mut self, image: Option<Arc<Image>>) -> Self {
        self.image = image;
        self
    }

    pub fn progress(mut self, value: f32) -> Self {
        self.progress = Some(value);
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ContinueWatchingCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let thumb_area = if let Some(image) = self.image {
            div()
                .id(format!("{}-thumb", self.id))
                .w_full()
                .h(px(140.))
                .rounded(px(8.))
                .overflow_hidden()
                .child(img(image).w_full().h_full().object_fit(gpui::ObjectFit::Cover))
        } else {
            div()
                .id(format!("{}-thumb", self.id))
                .w_full()
                .h(px(140.))
                .rounded(px(8.))
                .bg(cx.theme().muted.opacity(0.15))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Icon::new(IconName::Play)
                        .large()
                        .text_color(cx.theme().muted_foreground.opacity(0.3)),
                )
        };

        let thumb_with_overlay = div()
            .relative()
            .child(thumb_area)
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .rounded(px(8.))
                    .bg(gpui::transparent_black().opacity(0.5))
                    .opacity(0.)
                    .hover(|this| this.opacity(1.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(40.))
                            .h(px(40.))
                            .rounded_full()
                            .bg(cx.theme().primary.opacity(0.9))
                            .child(Icon::new(IconName::Play).text_color(gpui::Hsla::default())),
                    )
            );

        let wrapper = div()
            .id(format!("{}-wrapper", self.id))
            .w(px(240.))
            .rounded(px(8.))
            .cursor_pointer()
            .border_2()
            .border_color(gpui::transparent_black())
            .hover(|this| this.border_color(cx.theme().primary).shadow_lg());

        let wrapper = if let Some(handler) = self.on_click {
            wrapper.on_click(move |_event: &ClickEvent, window, cx| handler(window, cx))
        } else {
            wrapper
        };

        wrapper.child(
            v_flex()
                .gap_2()
                .child(thumb_with_overlay)
                .when_some(self.progress, |this, value| {
                    this.child(
                        div()
                            .w_full()
                            .h(px(3.))
                            .rounded(px(1.5))
                            .bg(cx.theme().muted.opacity(0.2))
                            .child(
                                div()
                                    .h_full()
                                    .rounded(px(1.5))
                                    .bg(cx.theme().primary)
                                    .w(relative(value)),
                            ),
                    )
                })
                .child(
                    v_flex()
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
