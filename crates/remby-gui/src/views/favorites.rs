use gpui::*;
use gpui_component::*;

use crate::app::RembyApp;
use crate::views::components::{LoadingIndicator, MediaCard};

#[derive(IntoElement)]
pub struct FavoritesView {
    app: WeakEntity<RembyApp>,
}

impl FavoritesView {
    pub fn new(app: WeakEntity<RembyApp>) -> Self {
        Self { app }
    }
}

impl RenderOnce for FavoritesView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (loading, favorites) = self
            .app
            .upgrade()
            .map(|app| {
                cx.read_entity(&app, |state, _| {
                    (state.state.loading, state.state.favorites.clone())
                })
            })
            .unwrap_or((false, vec![]));

        if loading && favorites.is_empty() {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(LoadingIndicator::new("Loading favorites..."))
                .into_any_element();
        }

        if favorites.is_empty() {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(
                    v_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            Icon::new(IconName::Heart)
                                .large()
                                .text_color(cx.theme().muted_foreground.opacity(0.3)),
                        )
                        .child(
                            div()
                                .text_color(cx.theme().muted_foreground)
                                .child("No favorites yet"),
                        ),
                )
                .into_any_element();
        }

        v_flex()
            .size_full()
            .p_6()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .font_bold()
                    .child(format!("Favorites ({})", favorites.len())),
            )
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_4()
                    .children(favorites.into_iter().map(|item| {
                        let subtitle = item
                            .series_name
                            .clone()
                            .or_else(|| item.media_type.clone())
                            .unwrap_or_default();
                        MediaCard::new(&item.id)
                            .title(&item.name)
                            .subtitle(subtitle)
                    })),
            )
            .into_any_element()
    }
}
