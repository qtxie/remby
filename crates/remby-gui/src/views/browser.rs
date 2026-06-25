use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::*;
use gpui_component::button::Button;
use gpui_component::input::{Input, InputState};
use gpui_component::scroll::ScrollableElement;

use crate::app::RembyApp;
use crate::state::{SortField, SortOrder};
use crate::views::components::badge::BadgeVariant;
use crate::views::components::{LoadingIndicator, MediaCard};

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
        let (loading, items, total, library_name, sort_field, sort_order, show_filters, filters, genres, tags, studios, poster_cache, _search_query) = self
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
                    )
                })
            })
            .unwrap_or((false, vec![], 0, String::new(), SortField::Name, SortOrder::Ascending, false, Default::default(), vec![], vec![], vec![], Default::default(), String::new()));

        let _app_weak = self.app.clone();
        let app_weak2 = self.app.clone();
        let app_weak3 = self.app.clone();
        let app_weak4 = self.app.clone();
        let app_weak5 = self.app.clone();
        let app_weak6 = self.app.clone();
        let app_weak7 = self.app.clone();

        v_flex()
            .size_full()
            .child(
                h_flex()
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
                    )
                    .child(
                        Button::new("sort-btn")
                            .small()
                            .label(format!("Sort: {}", sort_field.label()))
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
                            .label("Filters")
                            .selected(show_filters)
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_weak4.upgrade() {
                                    cx.update_entity(&app, |app, _cx| {
                                        app.state.browser_show_filters = !app.state.browser_show_filters;
                                    });
                                }
                            })
                    ),
            )
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
                    let rows = items.chunks(5);
                    let mut grid_content: Vec<AnyElement> = Vec::new();

                    for row in rows {
                        let cards = h_flex()
                            .gap_4()
                            .justify_center()
                            .children(row.iter().map(|item| {
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
