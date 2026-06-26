use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::*;
use gpui_component::scroll::ScrollableElement;

use crate::app::RembyApp;
use crate::state::SeriesSection;
use crate::views::components::LoadingIndicator;

#[derive(IntoElement)]
pub struct SeriesView {
    app: WeakEntity<RembyApp>,
}

impl SeriesView {
    pub fn new(app: WeakEntity<RembyApp>) -> Self {
        Self { app }
    }
}

impl RenderOnce for SeriesView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (
            loading,
            item,
            seasons,
            episodes,
            _similar,
            _section,
            is_favorite,
            poster_cache,
            backdrop_cache,
            selected_season,
        ) = self
            .app
            .upgrade()
            .map(|app| {
                cx.read_entity(&app, |state, _| {
                    (
                        state.state.loading,
                        state.state.series_item.clone(),
                        state.state.series_seasons.clone(),
                        state.state.series_episodes.clone(),
                        state.state.series_similar.clone(),
                        state.state.series_section.clone(),
                        state.state.series_item.as_ref()
                            .and_then(|i| i.user_data.as_ref())
                            .map(|u| u.is_favorite)
                            .unwrap_or(false),
                        state.state.poster_cache.clone(),
                        state.state.backdrop_cache.clone(),
                        state.state.series_selected_season.clone(),
                    )
                })
            })
            .unwrap_or((false, None, vec![], vec![], vec![], SeriesSection::Seasons, false, std::collections::HashMap::new(), std::collections::HashMap::new(), None));

        if loading && item.is_none() {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(LoadingIndicator::new("Loading..."))
                .into_any_element();
        }

        let Some(item) = item else {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child("No item selected")
                .into_any_element();
        };

        let app_ref = self.app.clone();
        let item_id = item.id.clone();

        let poster = poster_cache.get(&item_id).cloned();
        let backdrop = backdrop_cache.get(&item_id).cloned();

        let rating = item.community_rating.unwrap_or(0.0);
        let year = item.production_year.unwrap_or(0);
        let runtime_min = item.runtime_ticks.unwrap_or(0) / 10_000_000 / 60;
        let official_rating = item.official_rating.clone().unwrap_or_default();
        let genres = item.genres.clone();
        let overview = item.overview.clone().unwrap_or_default();
        let tagline = item.tagline.clone().unwrap_or_default();

        let video_info = item.media_sources.first().map(|s| s.display_label()).unwrap_or_default();

        let audio_label = item.media_sources.first()
            .and_then(|s| s.media_streams.iter().find(|st| st.stream_type == "Audio"))
            .and_then(|st| st.display_title.clone())
            .unwrap_or_else(|| "未知".into());

        let subtitle_label = item.media_sources.first()
            .and_then(|s| s.media_streams.iter().find(|st| st.stream_type == "Subtitle"))
            .and_then(|st| st.display_title.clone())
            .unwrap_or_else(|| "无".into());

        let header = h_flex()
            .items_center()
            .gap_4()
            .child({
                let app_ref = app_ref.clone();
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(36.))
                    .h(px(36.))
                    .rounded(px(6.))
                    .hover(|this| this.bg(cx.theme().muted.opacity(0.3)))
                    .cursor_pointer()
                    .child(
                        div().text_lg().child("←")
                    )
                    .id("movie-back")
                    .on_click(move |_, _window, cx| {
                        if let Some(app) = app_ref.upgrade() {
                            cx.update_entity(&app, |app, cx| {
                                app.state.go_back();
                                cx.notify();
                            });
                        }
                    })
            })
            .child({
                let app_ref = app_ref.clone();
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(36.))
                    .h(px(36.))
                    .rounded(px(6.))
                    .hover(|this| this.bg(cx.theme().muted.opacity(0.3)))
                    .cursor_pointer()
                    .child(Icon::new(IconName::Globe).large())
                    .id("movie-home")
                    .on_click(move |_, _window, cx| {
                        if let Some(app) = app_ref.upgrade() {
                            cx.update_entity(&app, |app, cx| {
                                app.state.navigate(crate::state::View::Home);
                                cx.notify();
                            });
                        }
                    })
            })
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(36.))
                    .h(px(36.))
                    .rounded(px(6.))
                    .hover(|this| this.bg(cx.theme().muted.opacity(0.3)))
                    .cursor_pointer()
                    .child(Icon::new(IconName::Menu).large())
            );

        let poster_element: AnyElement = match poster {
            Some(poster_img) => img(poster_img)
                .w_full()
                .h_full()
                .object_fit(gpui::ObjectFit::Cover)
                .into_any_element(),
            None => div()
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(cx.theme().muted.opacity(0.15))
                .child(
                    Icon::new(IconName::Frame)
                        .large()
                        .text_color(cx.theme().muted_foreground.opacity(0.3)),
                )
                .into_any_element(),
        };

        let backdrop_element: AnyElement = match backdrop {
            Some(backdrop_img) => img(backdrop_img)
                .w_full()
                .h_full()
                .object_fit(gpui::ObjectFit::Cover)
                .into_any_element(),
            None => div()
                .w_full()
                .h_full()
                .bg(cx.theme().muted.opacity(0.1))
                .into_any_element(),
        };

        let info_section = v_flex()
            .flex_1()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .font_bold()
                    .child(item.name.clone()),
            )
            .child({
                let meta = h_flex()
                    .items_center()
                    .gap_2()
                    .text_sm()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_1()
                            .child(Icon::new(IconName::Star).small().text_color(hsl(45., 0.8, 0.5)))
                            .child(format!("{:.1}", rating))
                    )
                    .child(div().w(px(1.)).h(px(12.)).rounded_full().bg(cx.theme().muted_foreground.opacity(0.4)))
                    .child(format!("{}", year))
                    .child(div().w(px(1.)).h(px(12.)).rounded_full().bg(cx.theme().muted_foreground.opacity(0.4)))
                    .child(format!("{}分钟", runtime_min));
                if !official_rating.is_empty() {
                    meta.child(div().w(px(1.)).h(px(12.)).rounded_full().bg(cx.theme().muted_foreground.opacity(0.4)))
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded(px(2.))
                                .border_1()
                                .border_color(cx.theme().muted_foreground)
                                .text_xs()
                                .child(official_rating.clone())
                        )
                } else {
                    meta
                }
            })
            .when(!genres.is_empty(), |this| {
                this.child(
                    h_flex()
                        .gap_2()
                        .children(genres.iter().map(|g| {
                            div()
                                .px_3()
                                .py_1()
                                .rounded(px(4.))
                                .border_1()
                                .border_color(cx.theme().border)
                                .text_xs()
                                .child(g.clone())
                        }))
                )
            })
            .when(!video_info.is_empty(), |this| {
                this.child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .text_sm()
                        .child("视频:")
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded(px(4.))
                                .bg(cx.theme().border.opacity(0.2))
                                .text_xs()
                                .child(video_info)
                        )
                )
            })
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .text_sm()
                    .child("音频:")
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded(px(4.))
                            .bg(cx.theme().border.opacity(0.2))
                            .text_xs()
                            .child(format!("{} ∨", audio_label))
                    )
            )
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .text_sm()
                    .child("字幕:")
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded(px(4.))
                            .bg(cx.theme().border.opacity(0.2))
                            .text_xs()
                            .child(format!("{} ∨", subtitle_label))
                    )
            )
            .child(
                h_flex()
                    .gap_3()
                    .mt_4()
                    .child({
                        let app_ref = app_ref.clone();
                        let play_id = item_id.clone();
                        h_flex()
                            .items_center()
                            .gap_2()
                            .px_5()
                            .py_2()
                            .rounded(px(6.))
                            .bg(cx.theme().primary)
                            .text_color(cx.theme().background)
                            .cursor_pointer()
                            .hover(|this| this.opacity(0.8))
                            .child(Icon::new(IconName::Play).small())
                            .child("播放")
                            .id("movie-play")
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_ref.upgrade() {
                                    let pid = play_id.clone();
                                    cx.update_entity(&app, |app, cx| {
                                        app.play_item(&pid, cx);
                                    });
                                }
                            })
                    })
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .px_5()
                            .py_2()
                            .rounded(px(6.))
                            .bg(cx.theme().border.opacity(0.2))
                            .cursor_pointer()
                            .hover(|this| this.opacity(0.8))
                            .child(Icon::new(IconName::Check).small())
                            .child("已播放")
                    )
                    .child({
                        let app_ref = app_ref.clone();
                        let fav_id = item_id.clone();
                        let is_fav = is_favorite;
                        h_flex()
                            .items_center()
                            .px_3()
                            .py_2()
                            .rounded(px(6.))
                            .bg(if is_fav { cx.theme().primary } else { cx.theme().border.opacity(0.2) })
                            .text_color(if is_fav { cx.theme().primary_foreground } else { cx.theme().foreground })
                            .cursor_pointer()
                            .hover(|this| this.opacity(0.8))
                            .child(Icon::new(IconName::Heart).small())
                            .id("movie-fav")
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_ref.upgrade() {
                                    let fid = fav_id.clone();
                                    let new_fav = !is_fav;
                                    cx.update_entity(&app, |app, cx| {
                                        app.toggle_favorite(&fid, new_fav, cx);
                                    });
                                }
                            })
                    })
                    .child(
                        h_flex()
                            .items_center()
                            .px_3()
                            .py_2()
                            .rounded(px(6.))
                            .bg(cx.theme().border.opacity(0.2))
                            .cursor_pointer()
                            .hover(|this| this.opacity(0.8))
                            .child("···")
                    )
            )
            .child(
                v_flex()
                    .mt_4()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .italic()
                            .text_color(cx.theme().muted_foreground)
                            .when(!tagline.is_empty(), |this| this.child(tagline))
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().foreground.opacity(0.8))
                            .child(overview)
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .cursor_pointer()
                            .hover(|this| this.text_color(cx.theme().foreground))
                            .child("阅读更多 ∨")
                    )
            );

        let effective_season = selected_season.clone()
            .or_else(|| seasons.first().map(|s| s.id.clone()));

        let season_tabs = if !seasons.is_empty() {
            let tabs = h_flex()
                .gap_2()
                .mb_4()
                .children(seasons.iter().map(|season| {
                    let is_active = effective_season.as_ref() == Some(&season.id);
                    let season_id = season.id.clone();
                    let app_ref = app_ref.clone();
                    let series_id_for_load = item_id.clone();
                    div()
                        .px_4()
                        .py_2()
                        .rounded(px(6.))
                        .cursor_pointer()
                        .when(is_active, |this| this.bg(cx.theme().primary).text_color(gpui::Hsla::default()))
                        .when(!is_active, |this| {
                            this.bg(cx.theme().muted.opacity(0.15))
                                .hover(|this| this.bg(cx.theme().muted.opacity(0.3)))
                        })
                        .text_sm()
                        .child(season.name.clone())
                        .id(format!("season-tab-{}", season.id))
                        .on_click(move |_, _window, cx| {
                            if let Some(app) = app_ref.upgrade() {
                                cx.update_entity(&app, |app, cx| {
                                    app.state.series_selected_season = Some(season_id.clone());
                                    app.load_series_episodes(&series_id_for_load, &crate::state::SeriesSection::Episodes, cx);
                                });
                            }
                        })
                }));
            Some(tabs)
        } else {
            None
        };

        let episode_list = if !episodes.is_empty() {
            let list = v_flex()
                .gap_2()
                .children(episodes.iter().filter_map(|ep| {
                    let ep_season = ep.parent_index_number?;
                    let season_match = effective_season.as_ref().and_then(|sid| {
                        seasons.iter().find(|s| &s.id == sid).and_then(|s| s.index_number)
                    });
                    if let Some(s_num) = season_match {
                        if ep_season != s_num {
                            return None;
                        }
                    }
                    let ep_num = ep.index_number.unwrap_or(0);
                    let ep_title = ep.name.clone();
                    let ep_runtime = ep.runtime_ticks.unwrap_or(0) / 10_000_000 / 60;
                    let ep_overview = ep.overview.clone().unwrap_or_default();
                    let progress = ep.user_data.as_ref().and_then(|u| {
                        let pos = u.playback_position_ticks?;
                        let dur = ep.runtime_ticks?;
                        if dur > 0 { Some((pos as f32) / (dur as f32)) } else { None }
                    });
                    let ep_id = ep.id.clone();
                    let ep_series_id = ep.series_id.clone();
                    let ep_item_type = ep.item_type.clone();
                    let app_ref = app_ref.clone();
                    let thumb = poster_cache.get(&ep.id).cloned();

                    Some(
                        h_flex()
                            .id(format!("ep-{}", ep_id))
                            .gap_4()
                            .p_3()
                            .rounded(px(8.))
                            .bg(cx.theme().muted.opacity(0.08))
                            .hover(|this| this.bg(cx.theme().muted.opacity(0.15)))
                            .cursor_pointer()
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_ref.upgrade() {
                                    cx.update_entity(&app, |app, cx| {
                                        let sid = if ep_series_id.is_some() || ep_item_type == "Series" {
                                            ep_series_id.clone().unwrap_or_else(|| ep_id.clone())
                                        } else {
                                            ep_id.clone()
                                        };
                                        app.state.navigate(crate::state::View::SeriesInfo);
                                        app.load_series_info(&sid, cx);
                                    });
                                }
                            })
                            .child(
                                div()
                                    .w(px(120.))
                                    .h(px(68.))
                                    .rounded(px(6.))
                                    .overflow_hidden()
                                    .flex_shrink_0()
                                    .bg(cx.theme().muted.opacity(0.15))
                                    .child(match thumb {
                                        Some(t) => img(t).w_full().h_full().object_fit(gpui::ObjectFit::Cover).into_any_element(),
                                        None => div().w_full().h_full().flex().items_center().justify_center()
                                            .child(Icon::new(IconName::Play).text_color(cx.theme().muted_foreground.opacity(0.3)))
                                            .into_any_element(),
                                    })
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .gap_1()
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_medium()
                                                    .child(format!("{}. {}", ep_num, ep_title))
                                            )
                                            .child(div().flex_1())
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(format!("{}分钟", ep_runtime))
                                            )
                                    )
                                    .when(!ep_overview.is_empty(), |this| {
                                        this.child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .line_clamp(2)
                                                .child(ep_overview)
                                        )
                                    })
                                    .when_some(progress, |this, p| {
                                        this.child(
                                            div()
                                                .w_full()
                                                .h(px(3.))
                                                .rounded(px(1.5))
                                                .bg(cx.theme().muted.opacity(0.2))
                                                .mt_1()
                                                .child(
                                                    div()
                                                        .h_full()
                                                        .rounded(px(1.5))
                                                        .bg(cx.theme().primary)
                                                        .w(relative(p)),
                                                ),
                                        )
                                    })
                            )
                    )
                }));
            Some(list)
        } else {
            None
        };

        let seasons_section = if season_tabs.is_some() || episode_list.is_some() {
            Some(
                v_flex()
                    .mt_8()
                    .gap_4()
                    .child(
                        div()
                            .text_lg()
                            .font_bold()
                            .child("剧集")
                    )
                    .children(season_tabs)
                    .children(episode_list)
            )
        } else {
            None
        };

        let content_scroll = v_flex()
            .flex_1()
            .overflow_y_scrollbar()
            .p_8()
            .child(header)
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_8()
                    .mt_4()
                    .items_start()
                    .child(
                        div()
                            .w(px(220.))
                            .h(px(330.))
                            .rounded(px(8.))
                            .overflow_hidden()
                            .flex_shrink_0()
                            .child(poster_element)
                    )
                    .child(info_section.flex_1().min_w(px(250.)))
                    .child(
                        div()
                            .w(px(280.))
                            .h(px(180.))
                            .rounded(px(8.))
                            .overflow_hidden()
                            .flex_shrink_0()
                            .opacity(0.8)
                            .child(backdrop_element)
                    )
            )
            .children(seasons_section);

        let bg_poster: AnyElement = match backdrop_cache.get(&item_id).cloned() {
            Some(bg_img) => img(bg_img)
                .w_full()
                .h_full()
                .object_fit(gpui::ObjectFit::Cover)
                .into_any_element(),
            None => div()
                .w_full()
                .h_full()
                .bg(cx.theme().border)
                .into_any_element(),
        };

        div()
            .size_full()
            .relative()
            .overflow_hidden()
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .child(
                        div()
                            .w_full()
                            .h_full()
                            .opacity(0.3)
                            .child(bg_poster)
                    )
            )
            .child(
                div()
                    .relative()
                    .size_full()
                    .child(content_scroll)
            )
            .into_any_element()
    }
}
