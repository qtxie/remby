use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::*;

use crate::app::RembyApp;
use crate::views::components::badge::BadgeVariant;
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
        let (loading, favorites, poster_cache) = self
            .app
            .upgrade()
            .map(|app| {
                cx.read_entity(&app, |state, _| {
                    (state.state.loading, state.state.favorites.clone(), state.state.poster_cache.clone())
                })
            })
            .unwrap_or((false, vec![], Default::default()));

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
                        let badge_text: Option<&str> = match item.item_type.as_str() {
                            "Movie" => Some("Movie"),
                            "Series" => Some("Series"),
                            "Episode" => Some("Ep"),
                            _ => None,
                        };
                        let progress = item.user_data.as_ref().and_then(|u| {
                            let pos = u.playback_position_ticks?;
                            let dur = item.runtime_ticks?;
                            if dur > 0 { Some((pos as f32) / (dur as f32)) } else { None }
                        });
                        MediaCard::new(&item.id)
                            .title(&item.name)
                            .subtitle(subtitle)
                            .poster_image(poster_cache.get(&item.id).cloned())
                            .when_some(badge_text, |card, text| card.badge(text, BadgeVariant::Default))
                            .when_some(progress, |card, p| card.progress(p))
                    })),
            )
            .into_any_element()
    }
}
