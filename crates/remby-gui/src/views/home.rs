use gpui::*;
use gpui_component::*;
use gpui_component::scroll::ScrollableElement;

use crate::app::RembyApp;
use crate::views::components::{LoadingIndicator, MediaCard};

#[derive(IntoElement)]
pub struct HomeView {
    app: WeakEntity<RembyApp>,
}

impl HomeView {
    pub fn new(app: WeakEntity<RembyApp>) -> Self {
        Self { app }
    }
}

impl RenderOnce for HomeView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (loading, continue_watching, latest_items, following_updates, poster_cache, app_entity) = self
            .app
            .upgrade()
            .map(|app| {
                cx.read_entity(&app, |state, _| {
                    (
                        state.state.loading,
                        state.state.continue_watching.clone(),
                        state.state.latest_items.clone(),
                        state.state.following_updates.clone(),
                        state.state.poster_cache.clone(),
                        app.downgrade(),
                    )
                })
            })
            .unwrap_or((false, vec![], vec![], vec![], Default::default(), self.app.clone()));

        if loading {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(LoadingIndicator::new("Loading home data..."))
                .into_any_element();
        }

        let mut sections: Vec<AnyElement> = Vec::new();

        if !continue_watching.is_empty() {
            sections.push(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_bold()
                            .child("Continue Watching"),
                    )
                    .child(horizontal_row(continue_watching, poster_cache.clone(), app_entity.clone()))
                    .into_any_element(),
            );
        }

        if !latest_items.is_empty() {
            sections.push(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_bold()
                            .child("Latest"),
                    )
                    .child(horizontal_row(latest_items, poster_cache.clone(), app_entity.clone()))
                    .into_any_element(),
            );
        }

        if !following_updates.is_empty() {
            sections.push(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_bold()
                            .child("Following Updates"),
                    )
                    .child(horizontal_row(following_updates, poster_cache, app_entity))
                    .into_any_element(),
            );
        }

        if sections.is_empty() {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(LoadingIndicator::new("No data available"))
                .into_any_element();
        }

        v_flex()
            .size_full()
            .p_6()
            .gap_6()
            .children(sections)
            .into_any_element()
    }
}

fn horizontal_row(items: Vec<remby_core::emby::MediaItem>, poster_cache: std::collections::HashMap<String, std::sync::Arc<gpui::Image>>, app: WeakEntity<RembyApp>) -> impl IntoElement {
    h_flex()
        .gap_4()
        .overflow_x_scrollbar()
        .children(items.into_iter().map(move |item| {
            let subtitle = item
                .series_name
                .clone()
                .or_else(|| item.media_type.clone())
                .unwrap_or_default();
            let item_id = item.id.clone();
            let app = app.clone();
            MediaCard::new(&item.id)
                .title(&item.name)
                .subtitle(subtitle)
                .poster_image(poster_cache.get(&item.id).cloned())
                .on_click(move |_window, cx| {
                    if let Some(app) = app.upgrade() {
                        cx.update_entity(&app, |app, cx| {
                            app.play_item(&item_id, cx);
                        });
                    }
                })
        }))
}
