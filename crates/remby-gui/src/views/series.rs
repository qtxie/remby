use gpui::*;
use gpui_component::*;
use gpui_component::scroll::ScrollableElement;

use crate::app::RembyApp;
use crate::state::SeriesSection;
use crate::views::components::{LoadingIndicator, MediaCard};

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
            similar,
            section,
            is_favorite,
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
                    )
                })
            })
            .unwrap_or((false, None, vec![], vec![], vec![], SeriesSection::Seasons, false));

        if loading && item.is_none() {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(LoadingIndicator::new("Loading series info..."))
                .into_any_element();
        }

        let Some(item) = item else {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child("No series selected")
                .into_any_element();
        };

        let app_ref = self.app.clone();
        let item_id = item.id.clone();
        let series_id = item.id.clone();

        let header = v_flex()
            .gap_4()
            .child(
                h_flex()
                    .gap_6()
                    .items_start()
                    .child(
                        div()
                            .w(px(200.))
                            .h(px(300.))
                            .rounded_lg()
                            .bg(cx.theme().muted.opacity(0.15))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Icon::new(IconName::Frame)
                                    .large()
                                    .text_color(cx.theme().muted_foreground.opacity(0.3)),
                            ),
                    )
                    .child(
                        v_flex()
                            .gap_3()
                            .flex_1()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_bold()
                                    .child(item.name.clone()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(
                                        item.overview
                                            .clone()
                                            .unwrap_or_default(),
                                    ),
                            )
                            .child({
                                let app_ref = app_ref.clone();
                                let fav_id = item_id.clone();
                                let is_fav = is_favorite;
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded(cx.theme().radius)
                                    .bg(if is_fav {
                                        cx.theme().primary
                                    } else {
                                        cx.theme().muted
                                    })
                                    .text_color(if is_fav {
                                        cx.theme().primary_foreground
                                    } else {
                                        cx.theme().muted_foreground
                                    })
                                    .cursor_pointer()
                                    .hover(|this| this.opacity(0.8))
                                    .child(if is_fav { "Unfollow" } else { "Follow" })
                                    .id("series-fav-btn")
                                    .on_click(move |_, _window, cx| {
                                        if let Some(app) = app_ref.upgrade() {
                                            let item_id = fav_id.clone();
                                            let new_fav = !is_fav;
                                            cx.update_entity(&app, |app, cx| {
                                                app.toggle_favorite(&item_id, new_fav, cx);
                                            });
                                        }
                                    })
                            }),
                    ),
            );

        let tabs = h_flex()
            .gap_1()
            .children(
                [SeriesSection::Seasons, SeriesSection::Episodes, SeriesSection::Similar]
                    .iter()
                    .enumerate()
                    .map(|(idx, s)| {
                        let label = match s {
                            SeriesSection::Seasons => "Seasons",
                            SeriesSection::Episodes => "Episodes",
                            SeriesSection::Similar => "Similar",
                        };
                        let is_active = section == *s;
                        let app_ref = app_ref.clone();
                        let section_clone = s.clone();
                        let sid = series_id.clone();
                        div()
                            .px_4()
                            .py_2()
                            .rounded(cx.theme().radius)
                            .bg(if is_active {
                                cx.theme().primary
                            } else {
                                cx.theme().muted
                            })
                            .text_color(if is_active {
                                cx.theme().primary_foreground
                            } else {
                                cx.theme().muted_foreground
                            })
                            .cursor_pointer()
                            .hover(|this| this.opacity(0.8))
                            .child(label)
                            .id(("series-tab", idx))
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_ref.upgrade() {
                                    let sid = sid.clone();
                                    cx.update_entity(&app, |app, cx| {
                                        app.state.series_section = section_clone.clone();
                                        app.load_series_episodes(&sid, &section_clone, cx);
                                    });
                                }
                            })
                    }),
            );

        let content: AnyElement = match section {
            SeriesSection::Seasons => {
                if seasons.is_empty() {
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("No seasons available")
                        .into_any_element()
                } else {
                    h_flex()
                        .gap_4()
                        .overflow_x_scrollbar()
                        .children(seasons.into_iter().map(|s| {
                            let subtitle = s
                                .child_count
                                .map(|c| format!("{} episodes", c))
                                .unwrap_or_default();
                            MediaCard::new(&s.id)
                                .title(&s.name)
                                .subtitle(subtitle)
                        }))
                        .into_any_element()
                }
            }
            SeriesSection::Episodes => {
                if episodes.is_empty() {
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("No episodes loaded. Click a season.")
                        .into_any_element()
                } else {
                    v_flex()
                        .gap_2()
                        .children(episodes.into_iter().enumerate().map(|(idx, ep)| {
                            let ep_num = ep
                                .index_number
                                .map(|n| format!("S{}E{}", ep.parent_index_number.unwrap_or(0), n))
                                .unwrap_or_default();
                            let title = if ep_num.is_empty() {
                                ep.name.clone()
                            } else {
                                format!("{} - {}", ep_num, ep.name)
                            };
                            let app_ref = app_ref.clone();
                            let ep_id = ep.id.clone();
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .px_3()
                                .py_2()
                                .rounded(cx.theme().radius)
                                .hover(|this| this.bg(cx.theme().muted.opacity(0.3)))
                                .cursor_pointer()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_medium()
                                        .child(title),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(
                                            ep.overview
                                                .clone()
                                                .unwrap_or_default(),
                                        ),
                                )
                                .id(("series-ep", idx))
                                .on_click(move |_, _window, cx| {
                                    if let Some(app) = app_ref.upgrade() {
                                        let ep_id = ep_id.clone();
                                        cx.update_entity(&app, |app, cx| {
                                            app.play_item(&ep_id, cx);
                                        });
                                    }
                                })
                        }))
                        .into_any_element()
                }
            }
            SeriesSection::Similar => {
                if similar.is_empty() {
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("No similar items")
                        .into_any_element()
                } else {
                    h_flex()
                        .gap_4()
                        .overflow_x_scrollbar()
                        .children(similar.into_iter().map(|s| {
                            let subtitle = s
                                .series_name
                                .clone()
                                .or_else(|| s.media_type.clone())
                                .unwrap_or_default();
                            MediaCard::new(&s.id)
                                .title(&s.name)
                                .subtitle(subtitle)
                        }))
                        .into_any_element()
                }
            }
        };

        v_flex()
            .size_full()
            .p_6()
            .gap_4()
            .child(header)
            .child(tabs)
            .child(content)
            .into_any_element()
    }
}
