use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::*;
use gpui_component::button::Button;
use gpui_component::input::{Input, InputState};
use gpui_component::scroll::ScrollableElement;

use crate::app::RembyApp;
use crate::state::{SortField, SortOrder};
use crate::views::components::LoadingIndicator;

const POSTER_W: f32 = 160.;
const POSTER_H: f32 = 220.;

const TABS: &[&str] = &["电影", "最近", "合集", "类型风格", "喜欢", "文件夹"];

#[derive(IntoElement)]
pub struct BrowserView {
    app: WeakEntity<RembyApp>,
    search_input: Entity<InputState>,
}

impl BrowserView {
    pub fn new(app: WeakEntity<RembyApp>, search_input: Entity<InputState>) -> Self {
        Self { app, search_input }
    }
}

impl RenderOnce for BrowserView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (loading, items, total, library_name, sort_field, sort_order, show_filters, filters, genres, tags, studios, poster_cache, _search_query, browser_selected) = self
            .app
            .upgrade()
            .map(|app| {
                cx.read_entity(&app, |state, _| {
                    let search_query = state.state.search_query.clone();
                    let (items, total, library_name) = if !search_query.is_empty() {
                        (state.state.search_results.clone(), state.state.search_results.len(), format!("Search: {}", search_query))
                    } else {
                        (state.state.browser_items.clone(), state.state.browser_total, state.state.browser_library_name.clone())
                    };
                    (
                        state.state.loading,
                        items,
                        total,
                        library_name,
                        state.state.browser_sort_field,
                        state.state.browser_sort_order,
                        state.state.browser_show_filters,
                        state.state.browser_filters.clone(),
                        state.state.browser_available_genres.clone(),
                        state.state.browser_available_tags.clone(),
                        state.state.browser_available_studios.clone(),
                        state.state.poster_cache.clone(),
                        search_query,
                        state.state.browser_selected,
                    )
                })
            })
            .unwrap_or((false, vec![], 0, String::new(), SortField::Name, SortOrder::Ascending, false, Default::default(), vec![], vec![], vec![], Default::default(), String::new(), 0));

        let _app_weak = self.app.clone();
        let app_weak2 = self.app.clone();
        let app_weak3 = self.app.clone();
        let app_weak4 = self.app.clone();
        let app_weak5 = self.app.clone();
        let app_weak6 = self.app.clone();
        let app_weak7 = self.app.clone();

        let tab_bar = h_flex()
            .items_center()
            .gap_1()
            .px_4()
            .py_2()
            .children(TABS.iter().enumerate().map(|(i, &tab)| {
                let is_active = i == 0;
                div()
                    .px_4()
                    .py_1()
                    .rounded_full()
                    .when(is_active, |this| this.bg(hsl(110., 45., 50.)).text_color(hsl(0., 0., 100.)))
                    .when(!is_active, |this| this.text_color(cx.theme().muted_foreground).hover(|this| this.bg(cx.theme().muted.opacity(0.15))))
                    .text_sm()
                    .cursor_pointer()
                    .child(tab)
            }));

        let item_count = items.len();
        let info_bar = h_flex()
            .items_center()
            .gap_4()
            .px_4()
            .py_2()
            .child(
                div().text_sm().text_color(cx.theme().muted_foreground)
                    .child(format!("共 {} 个项目", item_count))
            )
            .child(div().h_4().w_px().bg(cx.theme().border))
            .child(
                Button::new("play-all-btn")
                    .small()
                    .label("▶ 全部播放")
                    .on_click(move |_, _window, _cx| {})
            )
            .child(
                Button::new("shuffle-btn")
                    .small()
                    .label("🎲 随机播放")
                    .on_click(move |_, _window, _cx| {})
            )
            .child(div().h_4().w_px().bg(cx.theme().border))
            .child(
                Button::new("sort-btn")
                    .small()
                    .label(format!("≡ 排序方式: {}", sort_field.label()))
                    .on_click(move |_, _window, cx| {
                        if let Some(app) = app_weak2.upgrade() {
                            cx.update_entity(&app, |app, _cx| {
                                app.state.browser_sort_field = app.state.browser_sort_field.cycle();
                            });
                        }
                    })
            )
            .child(
                Button::new("sort-order-btn")
                    .small()
                    .label(sort_order.label())
                    .on_click(move |_, _window, cx| {
                        if let Some(app) = app_weak3.upgrade() {
                            cx.update_entity(&app, |app, _cx| {
                                app.state.browser_sort_order = app.state.browser_sort_order.toggle();
                            });
                        }
                    })
            )
            .child(
                Button::new("filter-btn")
                    .small()
                    .label("≡ 筛选")
                    .selected(show_filters)
                    .on_click(move |_, _window, cx| {
                        if let Some(app) = app_weak4.upgrade() {
                            cx.update_entity(&app, |app, _cx| {
                                app.state.browser_show_filters = !app.state.browser_show_filters;
                            });
                        }
                    })
            );

        let header_bar = h_flex()
            .p_4()
            .gap_4()
            .items_center()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .text_lg()
                    .font_bold()
                    .child(library_name)
            )
            .child(
                Input::new(&self.search_input)
                    .small()
                    .cleanable(true)
                    .prefix(Icon::new(IconName::Search).small())
            );

        v_flex()
            .size_full()
            .child(tab_bar)
            .child(
                header_bar
            )
            .child(info_bar)
            .when(show_filters, |this| {
                let mut filter_content: Vec<AnyElement> = Vec::new();

                if !genres.is_empty() {
                    let genre_labels: Vec<AnyElement> = genres.into_iter().enumerate().map(|(idx, g)| {
                        let g_clone = g.clone();
                        let is_selected = filters.genres.contains(&g);
                        let app_w = app_weak5.clone();
                        Button::new(("genre", idx))
                            .small()
                            .label(g)
                            .selected(is_selected)
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_w.upgrade() {
                                    cx.update_entity(&app, |app, _cx| {
                                        if let Some(pos) = app.state.browser_filters.genres.iter().position(|x| *x == g_clone) {
                                            app.state.browser_filters.genres.remove(pos);
                                        } else {
                                            app.state.browser_filters.genres.push(g_clone.clone());
                                        }
                                    });
                                }
                            })
                            .into_any_element()
                    }).collect();

                    filter_content.push(
                        v_flex()
                            .gap_2()
                            .child(div().text_xs().font_bold().child("Genres"))
                            .child(h_flex().gap_1().flex_wrap().children(genre_labels))
                            .into_any_element()
                    );
                }

                if !tags.is_empty() {
                    let tag_labels: Vec<AnyElement> = tags.into_iter().enumerate().map(|(idx, t)| {
                        let t_clone = t.clone();
                        let is_selected = filters.tags.contains(&t);
                        let app_w = app_weak6.clone();
                        Button::new(("tag", idx))
                            .small()
                            .label(t)
                            .selected(is_selected)
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_w.upgrade() {
                                    cx.update_entity(&app, |app, _cx| {
                                        if let Some(pos) = app.state.browser_filters.tags.iter().position(|x| *x == t_clone) {
                                            app.state.browser_filters.tags.remove(pos);
                                        } else {
                                            app.state.browser_filters.tags.push(t_clone.clone());
                                        }
                                    });
                                }
                            })
                            .into_any_element()
                    }).collect();

                    filter_content.push(
                        v_flex()
                            .gap_2()
                            .child(div().text_xs().font_bold().child("Tags"))
                            .child(h_flex().gap_1().flex_wrap().children(tag_labels))
                            .into_any_element()
                    );
                }

                if !studios.is_empty() {
                    let studio_labels: Vec<AnyElement> = studios.into_iter().enumerate().map(|(idx, s)| {
                        let s_clone = s.clone();
                        let is_selected = filters.studios.contains(&s);
                        let app_w = app_weak7.clone();
                        Button::new(("studio", idx))
                            .small()
                            .label(s)
                            .selected(is_selected)
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_w.upgrade() {
                                    cx.update_entity(&app, |app, _cx| {
                                        if let Some(pos) = app.state.browser_filters.studios.iter().position(|x| *x == s_clone) {
                                            app.state.browser_filters.studios.remove(pos);
                                        } else {
                                            app.state.browser_filters.studios.push(s_clone.clone());
                                        }
                                    });
                                }
                            })
                            .into_any_element()
                    }).collect();

                    filter_content.push(
                        v_flex()
                            .gap_2()
                            .child(div().text_xs().font_bold().child("Studios"))
                            .child(h_flex().gap_1().flex_wrap().children(studio_labels))
                            .into_any_element()
                    );
                }

                this.child(
                    div()
                        .p_4()
                        .gap_4()
                        .border_b_1()
                        .border_color(cx.theme().border)
                        .child(v_flex().gap_4().children(filter_content))
                )
            })
            .child(
                if loading && items.is_empty() {
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(LoadingIndicator::new("Loading..."))
                        .into_any_element()
                } else if items.is_empty() {
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(div().text_sm().text_color(cx.theme().muted_foreground).child("No items found"))
                        .into_any_element()
                } else {
                    let cols = 5usize;
                    let rows = items.chunks(cols);
                    let mut grid_content: Vec<AnyElement> = Vec::new();

                    for row in rows {
                        let cards = h_flex()
                            .gap_4()
                            .justify_center()
                            .children(row.iter().enumerate().map(|(col_idx, item)| {
                                let global_idx = grid_content.len() * cols + col_idx;
                                let is_selected = global_idx == browser_selected;
                                let poster = poster_cache.get(&item.id).cloned();
                                let rating = item.community_rating.unwrap_or(0.0);
                                let year = item.production_year.unwrap_or(0);

                                v_flex()
                                    .w(px(POSTER_W))
                                    .gap_2()
                                    .cursor_pointer()
                                    .hover(|this| this.opacity(0.9))
                                    .child(
                                        div()
                                            .id(format!("poster-{}", item.id))
                                            .h(px(POSTER_H))
                                            .rounded(px(6.))
                                            .overflow_hidden()
                                            .border_2()
                                            .when(is_selected, |this| this.border_color(cx.theme().primary))
                                            .when(!is_selected, |this| this.border_color(gpui::transparent_black()))
                                            .child(match poster {
                                                Some(image) => img(image).w_full().h_full().object_fit(gpui::ObjectFit::Cover).into_any_element(),
                                                None => div().w_full().h_full().flex().items_center().justify_center().bg(cx.theme().muted.opacity(0.15)).child(
                                                    Icon::new(IconName::Frame).large().text_color(cx.theme().muted_foreground.opacity(0.3))
                                                ).into_any_element(),
                                            })
                                    )
                                    .child(
                                        v_flex()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_medium()
                                                    .overflow_x_hidden()
                                                    .whitespace_nowrap()
                                                    .child(item.display_name())
                                            )
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .child(Icon::new(IconName::Star).small().text_color(hsl(45., 0.8, 0.5)))
                                                    .child(
                                                        div().text_xs().text_color(cx.theme().muted_foreground)
                                                            .child(format!("{:.1}", rating))
                                                    )
                                                    .child(div().flex_1())
                                                    .child(
                                                        div().text_xs().text_color(cx.theme().muted_foreground)
                                                            .child(format!("{}", year))
                                                    )
                                            )
                                    )
                                    .into_any_element()
                            }));
                        grid_content.push(cards.into_any_element());
                    }

                    if items.len() < total {
                        grid_content.push(
                            div()
                                .p_4()
                                .flex()
                                .justify_center()
                                .child(LoadingIndicator::new("Loading more..."))
                                .into_any_element()
                        );
                    }

                    div()
                        .flex_1()
                        .overflow_y_scrollbar()
                        .p_4()
                        .child(
                            v_flex()
                                .gap_4()
                                .children(grid_content)
                        )
                        .into_any_element()
                },
            )
    }
}
