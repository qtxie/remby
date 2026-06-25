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
            _seasons,
            _episodes,
            _similar,
            _section,
            is_favorite,
            poster_cache,
            backdrop_cache,
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
                    )
                })
            })
            .unwrap_or((false, None, vec![], vec![], vec![], SeriesSection::Seasons, false, std::collections::HashMap::new(), std::collections::HashMap::new()));

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

        let rating = item.community_rating.unwrap_or(0.0) as i32;
        let year = item.production_year.unwrap_or(0);
        let runtime_min = item.runtime_ticks.unwrap_or(0) / 10_000_000 / 60;
        let official_rating = item.official_rating.clone().unwrap_or_default();
        let genres = item.genres.clone();
        let overview = item.overview.clone().unwrap_or_default();

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
                            .child(format!("{}", rating))
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
                            .child("He's never been cooler.")
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
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .mt_2()
                    .child("导演: 李·塔玛霍瑞")
            );

        let cast_section = v_flex()
            .mt_8()
            .child(
                div()
                    .text_lg()
                    .font_bold()
                    .mb_4()
                    .child("演职人员")
            )
            .child({
                let people: Vec<(&str, &str)> = vec![
                    ("李·塔玛霍瑞", "导演"),
                    ("丹尼尔·克雷格", "饰演: James Bond"),
                    ("朱迪·丹奇", "饰演: M"),
                    ("哈维尔·巴登", "饰演: Raoul Silva"),
                    ("蕾雅·赛杜", "饰演: Sévérine"),
                    ("拉尔夫·费因斯", "饰演: Gareth Mallory"),
                ];
                h_flex()
                    .gap_4()
                    .overflow_x_scrollbar()
                    .children(people.into_iter().map(|(person_name, role)| {
                        v_flex()
                            .w(px(100.))
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(80.))
                                    .h(px(100.))
                                    .rounded(px(6.))
                                    .overflow_hidden()
                                    .bg(cx.theme().border.opacity(0.3))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        Icon::new(IconName::User)
                                            .large()
                                            .text_color(cx.theme().muted_foreground.opacity(0.3))
                                    )
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_center()
                                    .child(person_name.to_string())
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .text_center()
                                    .child(role.to_string())
                            )
                    }))
            });

        let content_scroll = v_flex()
            .flex_1()
            .overflow_y_scrollbar()
            .p_8()
            .child(header)
            .child(
                h_flex()
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
                    .child(info_section)
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
            .child(cast_section);

        let bg_poster: AnyElement = match poster_cache.get(&item_id).cloned() {
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
