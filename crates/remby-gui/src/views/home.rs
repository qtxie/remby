use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::*;
use gpui_component::scroll::ScrollableElement;

use crate::app::RembyApp;
use crate::views::components::badge::BadgeVariant;
use crate::views::components::{ContinueWatchingCard, LoadingIndicator, MediaCard};

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
        let (loading, continue_watching, latest_items, following_updates, poster_cache, backdrop_cache, app_entity) = self
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
                        state.state.backdrop_cache.clone(),
                        app.downgrade(),
                    )
                })
            })
            .unwrap_or((false, vec![], vec![], vec![], Default::default(), Default::default(), self.app.clone()));

        if loading {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(LoadingIndicator::new("Loading home data..."))
                .into_any_element();
        }

        let mut sections: Vec<AnyElement> = Vec::new();
        let bg_color = cx.theme().background;

        if !continue_watching.is_empty() {
            let hero = &continue_watching[0];
            let hero_backdrop = backdrop_cache.get(&hero.id).cloned();
            let hero_poster = poster_cache.get(&hero.id).cloned();
            let hero_title = hero.name.clone();
            let hero_year = hero.production_year.unwrap_or(0);
            let hero_rating = hero.community_rating.unwrap_or(0.0);
            let hero_series = hero.series_name.clone().unwrap_or_default();
            let hero_episode = match (hero.parent_index_number, hero.index_number) {
                (Some(s), Some(e)) => format!("S{:02}E{:02}", s, e),
                (None, Some(e)) => format!("E{:02}", e),
                _ => String::new(),
            };
            let hero_id = hero.id.clone();
            let hero_series_id = hero.series_id.clone();
            let hero_item_type = hero.item_type.clone();
            let hero_app = app_entity.clone();

            let hero_bg: AnyElement = match hero_backdrop {
                Some(bd) => img(bd).w_full().h_full().object_fit(gpui::ObjectFit::Cover).into_any_element(),
                None => match hero_poster {
                    Some(p) => img(p).w_full().h_full().object_fit(gpui::ObjectFit::Cover).into_any_element(),
                    None => div().w_full().h_full().bg(cx.theme().muted.opacity(0.2)).into_any_element(),
                },
            };

            sections.push(
                div()
                    .h(px(360.))
                    .relative()
                    .overflow_hidden()
                    .rounded(px(12.))
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .child(hero_bg)
                    )
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .bg(gpui::linear_gradient(
                                90.,
                                gpui::linear_color_stop(cx.theme().background.opacity(0.95), 0.),
                                gpui::linear_color_stop(cx.theme().background.opacity(0.3), 0.6),
                            ))
                    )
                    .child(
                        div()
                            .absolute()
                            .bottom_0()
                            .left_0()
                            .p_8()
                            .child(
                                v_flex()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_3xl()
                                            .font_bold()
                                            .child(hero_title)
                                    )
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_3()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .when(!hero_series.is_empty(), |this| {
                                                this.child(
                                                    div()
                                                        .text_color(cx.theme().foreground)
                                                        .child(hero_series)
                                                )
                                            })
                                            .when(!hero_episode.is_empty(), |this| {
                                                this.child(div().child(hero_episode))
                                            })
                                            .when(hero_year > 0, |this| {
                                                this.child(div().child(format!("{}", hero_year)))
                                            })
                                            .when(hero_rating > 0.0, |this| {
                                                this.child(
                                                    h_flex()
                                                        .items_center()
                                                        .gap_1()
                                                        .child(Icon::new(IconName::Star).small().text_color(hsl(45., 0.8, 0.5)))
                                                        .child(div().child(format!("{:.1}", hero_rating)))
                                                )
                                            })
                                    )
                                    .child(
                                        {
                                            let app_ref = hero_app.clone();
                                            let play_id = hero_id.clone();
                                            let play_sid = hero_series_id.clone();
                                            let play_type = hero_item_type.clone();
                                            h_flex()
                                                .items_center()
                                                .gap_2()
                                                .px_5()
                                                .py_2()
                                                .mt_2()
                                                .rounded(px(8.))
                                                .bg(cx.theme().primary)
                                                .text_color(cx.theme().background)
                                                .cursor_pointer()
                                                .hover(|this| this.opacity(0.8))
                                                .child(Icon::new(IconName::Play).small())
                                                .child("继续播放")
                                                .id("hero-play")
                                                .on_click(move |_event, _window, cx| {
                                                    if let Some(app) = app_ref.upgrade() {
                                                        cx.update_entity(&app, |app, cx| {
                                                            let sid = if play_sid.is_some() || play_type == "Series" {
                                                                play_sid.clone().unwrap_or_else(|| play_id.clone())
                                                            } else {
                                                                play_id.clone()
                                                            };
                                                            app.state.navigate(crate::state::View::SeriesInfo);
                                                            app.load_series_info(&sid, cx);
                                                        });
                                                    }
                                                })
                                        }
                                    )
                            )
                    )
                    .into_any_element(),
            );

            sections.push(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_bold()
                            .child("继续观看"),
                    )
                    .child(continue_watching_row(continue_watching, poster_cache.clone(), app_entity.clone(), bg_color))
                    .into_any_element(),
            );
        }

        if !latest_items.is_empty() {
            sections.push(
                v_flex()
                    .gap_3()
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_lg()
                                    .font_bold()
                                    .child("最新 电影")
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().primary)
                                    .cursor_pointer()
                                    .hover(|this| this.opacity(0.8))
                                    .child("查看全部 →")
                            )
                    )
                    .child(latest_movies_row(latest_items, poster_cache.clone(), app_entity.clone(), bg_color))
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
                            .child("关注更新"),
                    )
                    .child(horizontal_row(following_updates, poster_cache, app_entity, bg_color))
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

fn continue_watching_row(items: Vec<remby_core::emby::MediaItem>, poster_cache: std::collections::HashMap<String, std::sync::Arc<gpui::Image>>, app: WeakEntity<RembyApp>, bg_color: gpui::Hsla) -> impl IntoElement {
    div()
        .relative()
        .child(
            h_flex()
                .gap_4()
                .overflow_x_scrollbar()
                .children(items.into_iter().map(move |item| {
                    let episode_info = match (item.parent_index_number, item.index_number) {
                        (Some(season), Some(episode)) => format!("S{:02}E{:02}", season, episode),
                        (None, Some(episode)) => format!("E{:02}", episode),
                        _ => String::new(),
                    };
                    let series_name = item.series_name.clone().unwrap_or_default();
                    let subtitle = if !series_name.is_empty() && !episode_info.is_empty() {
                        format!("{} · {}", series_name, episode_info)
                    } else if !series_name.is_empty() {
                        series_name
                    } else {
                        item.media_type.clone().unwrap_or_default()
                    };
                    let progress = item.user_data.as_ref().and_then(|u| {
                        let pos = u.playback_position_ticks?;
                        let dur = item.runtime_ticks?;
                        if dur > 0 { Some((pos as f32) / (dur as f32)) } else { None }
                    });
                    let item_id = item.id.clone();
                    let series_id = item.series_id.clone();
                    let item_type = item.item_type.clone();
                    let app = app.clone();
                    ContinueWatchingCard::new(&item.id, &item.name)
                        .subtitle(subtitle)
                        .image(poster_cache.get(&item.id).cloned())
                        .when_some(progress, |card, p| card.progress(p))
                        .on_click(move |_window, cx| {
                            if let Some(app) = app.upgrade() {
                                cx.update_entity(&app, |app, cx| {
                                    let sid = if series_id.is_some() || item_type == "Series" {
                                        series_id.clone().unwrap_or_else(|| item_id.clone())
                                    } else {
                                        item_id.clone()
                                    };
                                    app.state.navigate(crate::state::View::SeriesInfo);
                                    app.load_series_info(&sid, cx);
                                });
                            }
                        })
                }))
        )
        .child(
            div()
                .absolute()
                .left_0()
                .top_0()
                .bottom_0()
                .w(px(24.))
                .bg(gpui::linear_gradient(
                    90.,
                    gpui::linear_color_stop(bg_color, 0.),
                    gpui::linear_color_stop(gpui::transparent_black(), 1.),
                ))
        )
        .child(
            div()
                .absolute()
                .right_0()
                .top_0()
                .bottom_0()
                .w(px(24.))
                .bg(gpui::linear_gradient(
                    270.,
                    gpui::linear_color_stop(bg_color, 0.),
                    gpui::linear_color_stop(gpui::transparent_black(), 1.),
                ))
        )
}

fn latest_movies_row(items: Vec<remby_core::emby::MediaItem>, poster_cache: std::collections::HashMap<String, std::sync::Arc<gpui::Image>>, app: WeakEntity<RembyApp>, bg_color: gpui::Hsla) -> impl IntoElement {
    div()
        .relative()
        .child(
            h_flex()
                .gap_4()
                .overflow_x_scrollbar()
                .children(items.into_iter().map(move |item| {
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
                    let item_id = item.id.clone();
                    let series_id = item.series_id.clone();
                    let item_type = item.item_type.clone();
                    let app = app.clone();
                    MediaCard::new(&item.id)
                        .title(&item.name)
                        .subtitle(subtitle)
                        .poster_image(poster_cache.get(&item.id).cloned())
                        .when_some(badge_text, |card, text| card.badge(text, BadgeVariant::Default))
                        .on_click(move |_window, cx| {
                            if let Some(app) = app.upgrade() {
                                cx.update_entity(&app, |app, cx| {
                                    let sid = if series_id.is_some() || item_type == "Series" {
                                        series_id.clone().unwrap_or_else(|| item_id.clone())
                                    } else {
                                        item_id.clone()
                                    };
                                    app.state.navigate(crate::state::View::SeriesInfo);
                                    app.load_series_info(&sid, cx);
                                });
                            }
                        })
                }))
        )
        .child(
            div()
                .absolute()
                .left_0()
                .top_0()
                .bottom_0()
                .w(px(24.))
                .bg(gpui::linear_gradient(
                    90.,
                    gpui::linear_color_stop(bg_color, 0.),
                    gpui::linear_color_stop(gpui::transparent_black(), 1.),
                ))
        )
        .child(
            div()
                .absolute()
                .right_0()
                .top_0()
                .bottom_0()
                .w(px(24.))
                .bg(gpui::linear_gradient(
                    270.,
                    gpui::linear_color_stop(bg_color, 0.),
                    gpui::linear_color_stop(gpui::transparent_black(), 1.),
                ))
        )
}

fn horizontal_row(items: Vec<remby_core::emby::MediaItem>, poster_cache: std::collections::HashMap<String, std::sync::Arc<gpui::Image>>, app: WeakEntity<RembyApp>, bg_color: gpui::Hsla) -> impl IntoElement {
    div()
        .relative()
        .child(
            h_flex()
                .gap_4()
                .overflow_x_scrollbar()
                .children(items.into_iter().map(move |item| {
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
                    let item_id = item.id.clone();
                    let series_id = item.series_id.clone();
                    let item_type = item.item_type.clone();
                    let app = app.clone();
                    MediaCard::new(&item.id)
                        .title(&item.name)
                        .subtitle(subtitle)
                        .poster_image(poster_cache.get(&item.id).cloned())
                        .when_some(badge_text, |card, text| card.badge(text, BadgeVariant::Default))
                        .when_some(progress, |card, p| card.progress(p))
                        .on_click(move |_window, cx| {
                            if let Some(app) = app.upgrade() {
                                cx.update_entity(&app, |app, cx| {
                                    let sid = if series_id.is_some() || item_type == "Series" {
                                        series_id.clone().unwrap_or_else(|| item_id.clone())
                                    } else {
                                        item_id.clone()
                                    };
                                    app.state.navigate(crate::state::View::SeriesInfo);
                                    app.load_series_info(&sid, cx);
                                });
                            }
                        })
                }))
        )
        .child(
            div()
                .absolute()
                .left_0()
                .top_0()
                .bottom_0()
                .w(px(24.))
                .bg(gpui::linear_gradient(
                    90.,
                    gpui::linear_color_stop(bg_color, 0.),
                    gpui::linear_color_stop(gpui::transparent_black(), 1.),
                ))
        )
        .child(
            div()
                .absolute()
                .right_0()
                .top_0()
                .bottom_0()
                .w(px(24.))
                .bg(gpui::linear_gradient(
                    270.,
                    gpui::linear_color_stop(bg_color, 0.),
                    gpui::linear_color_stop(gpui::transparent_black(), 1.),
                ))
        )
}
