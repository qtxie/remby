use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::*;
use gpui_component::scroll::ScrollableElement;

use crate::app::RembyApp;
use crate::views::components::badge::BadgeVariant;
use crate::views::components::{ContinueWatchingCard, LoadingIndicator, MediaCard};
use crate::views::components::library_card::LibraryCard;

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
        let app_for_my_media = self.app.clone();
        let (loading, continue_watching, latest_items, following_updates, poster_cache, libraries, app_entity) = self
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
                        state.state.libraries.clone(),
                        app.downgrade(),
                    )
                })
            })
            .unwrap_or((false, vec![], vec![], vec![], Default::default(), vec![], self.app.clone()));

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

        if !libraries.is_empty() {
            let this = app_for_my_media.clone();
            sections.push(
                v_flex()
                    .gap_3()
                    .child(
                        div()
                            .text_lg()
                            .font_bold()
                            .child("我的媒体")
                    )
                    .child(
                        h_flex()
                            .gap_4()
                            .flex_wrap()
                            .children(libraries.iter().map(move |lib| {
                                let this = this.clone();
                                let lib_id = lib.id.clone();
                                let lib_name = lib.name.clone();
                                LibraryCard::new(&lib.id, &lib.name)
                                    .posters([None, None, None, None])
                                    .on_click(move |_, cx| {
                                        if let Some(app) = this.upgrade() {
                                            cx.update_entity(&app, |app, cx| {
                                                app.state.browser_library_id = lib_id.clone();
                                                app.state.browser_library_name = lib_name.clone();
                                                app.state.navigate(crate::state::View::LibraryBrowser);
                                                app.load_browser_data(cx);
                                            });
                                        }
                                    })
                            }))
                    )
                    .into_any_element(),
            );
        }

        if !continue_watching.is_empty() {
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
                                    .id("view-all-latest")
                                    .text_sm()
                                    .text_color(cx.theme().primary)
                                    .cursor_pointer()
                                    .hover(|this| this.opacity(0.8))
                                    .child("查看全部 →")
                                    .on_click({
                                        let this = app_entity.clone();
                                        move |_, _, cx| {
                                            if let Some(app) = this.upgrade() {
                                                cx.update_entity(&app, |app, cx| {
                                                    app.state.navigate(crate::state::View::LibraryBrowser);
                                                    app.load_browser_data(cx);
                                                });
                                            }
                                        }
                                    })
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

        div()
            .id("home-scroll")
            .size_full()
            .overflow_y_scroll()
            .child(
                v_flex()
                    .p_6()
                    .gap_6()
                    .children(sections)
            )
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
                    let subtitle = format!("{}{}{}",
                        item.community_rating.map(|r| format!("★ {:.1}", r)).unwrap_or_default(),
                        if item.community_rating.is_some() && item.production_year.is_some() { " · " } else { "" },
                        item.production_year.map(|y| y.to_string()).unwrap_or_default()
                    );
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
