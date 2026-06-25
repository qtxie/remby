use std::sync::Arc;

use gpui::*;
use gpui_component::*;

#[derive(IntoElement)]
pub struct LibraryCard {
    id: SharedString,
    name: SharedString,
    posters: [Option<Arc<Image>>; 4],
    on_click: Option<Box<dyn Fn(&mut Window, &mut App)>>,
}

impl LibraryCard {
    pub fn new(id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            posters: [None, None, None, None],
            on_click: None,
        }
    }

    pub fn posters(mut self, p: [Option<Arc<Image>>; 4]) -> Self {
        self.posters = p;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for LibraryCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let wrapper = div()
            .id(self.id.clone())
            .w(px(140.))
            .cursor_pointer()
            .hover(|this| this.opacity(0.9).shadow_lg());

        let wrapper = if let Some(handler) = self.on_click {
            wrapper.on_click(move |_event: &ClickEvent, window, cx| handler(window, cx))
        } else {
            wrapper
        };

        wrapper.child(
            v_flex()
                .gap_2()
                .child(
                    div()
                        .h(px(100.))
                        .rounded(px(8.))
                        .overflow_hidden()
                        .bg(cx.theme().border.opacity(0.3))
                        .grid()
                        .grid_cols(2)
                        .grid_rows(2)
                        .gap(px(2.))
                        .p(px(4.))
                        .children(self.posters.into_iter().map(|poster| {
                            if let Some(image) = poster {
                                div()
                                    .rounded(px(2.))
                                    .overflow_hidden()
                                    .child(
                                        img(image)
                                            .w_full()
                                            .h_full()
                                            .object_fit(gpui::ObjectFit::Cover),
                                    )
                            } else {
                                div()
                                    .rounded(px(2.))
                                    .bg(cx.theme().muted.opacity(0.3))
                            }
                        })),
                )
                .child(
                    div()
                        .text_sm()
                        .text_center()
                        .child(self.name),
                ),
        )
    }
}
