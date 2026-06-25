use gpui::*;
use gpui_component::*;
use gpui_component::scroll::ScrollableElement;

use crate::app::RembyApp;
use crate::state::View;
use crate::views::components::{LoadingIndicator, MediaCard};

#[derive(IntoElement)]
pub struct LibrariesView {
    app: WeakEntity<RembyApp>,
}

impl LibrariesView {
    pub fn new(app: WeakEntity<RembyApp>) -> Self {
        Self { app }
    }
}

impl RenderOnce for LibrariesView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (loading, libraries, latest_items, poster_cache) = self
            .app
            .upgrade()
            .map(|app| {
                cx.read_entity(&app, |state, _| {
                    (
                        state.state.loading,
                        state.state.libraries.clone(),
                        state.state.latest_items.clone(),
                        state.state.poster_cache.clone(),
                    )
                })
            })
            .unwrap_or((false, vec![], vec![], Default::default()));

        if loading && libraries.is_empty() {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(LoadingIndicator::new("Loading libraries..."))
                .into_any_element();
        }

        let mut content: Vec<AnyElement> = Vec::new();

        if !libraries.is_empty() {
            let rows = libraries.chunks(4);
            for (row_idx, row) in rows.enumerate() {
                let cards = h_flex()
                    .gap_4()
                    .children(row.iter().enumerate().map(|(col_idx, lib)| {
                        let lib_id = lib.id.clone();
                        let lib_name = lib.name.clone();
                        let app_weak = self.app.clone();
                        let id = row_idx * 4 + col_idx;
                        div()
                            .id(("library-card", id))
                            .w_48()
                            .h_32()
                            .rounded_lg()
                            .bg(rgb(0x2a2a2a))
                            .hover(|s| s.bg(rgb(0x3a3a3a)))
                            .p_4()
                            .flex_col()
                            .justify_between()
                            .cursor_pointer()
                            .on_click(move |_window, _event, cx| {
                                if let Some(app) = app_weak.upgrade() {
                                    cx.update_entity(&app, |app, _cx| {
                                        app.state.browser_library_id = lib_id.clone();
                                        app.state.browser_library_name = lib_name.clone();
                                        app.state.navigate(View::LibraryBrowser);
                                    });
                                }
                            })
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_lg()
                                            .child("📁"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_bold()
                                            .child(lib.name.clone()),
                                    ),
                            )
                            .into_any_element()
                    }));
                content.push(cards.into_any_element());
            }
        }

        if !latest_items.is_empty() {
            content.push(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_bold()
                            .child("Latest"),
                    )
                    .child(
                        h_flex()
                            .gap_4()
                            .overflow_x_scrollbar()
                            .children(latest_items.into_iter().map(|item| {
                                let subtitle = item
                                    .series_name
                                    .clone()
                                    .or_else(|| item.media_type.clone())
                                    .unwrap_or_default();
                                MediaCard::new(&item.id)
                                    .title(&item.name)
                                    .subtitle(subtitle)
                                    .poster_image(poster_cache.get(&item.id).cloned())
                            })),
                    )
                    .into_any_element(),
            );
        }

        if content.is_empty() {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(LoadingIndicator::new("No libraries found"))
                .into_any_element();
        }

        v_flex()
            .size_full()
            .p_6()
            .gap_6()
            .children(content)
            .into_any_element()
    }
}
