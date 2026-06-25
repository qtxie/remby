use gpui::*;
use gpui_component::*;
use gpui::prelude::FluentBuilder;

use crate::state::View;

#[derive(IntoElement)]
pub struct SidebarNav {
    current_view: View,
}

impl SidebarNav {
    pub fn new(current_view: View) -> Self {
        Self { current_view }
    }
}

impl RenderOnce for SidebarNav {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let nav_items: Vec<(&str, IconName, View)> = vec![
            ("Home", IconName::LayoutDashboard, View::Home),
            ("Libraries", IconName::Folder, View::Libraries),
            ("Favorites", IconName::Heart, View::Favorites),
            ("Settings", IconName::Settings, View::Settings),
        ];

        v_flex()
            .w(px(220.))
            .h_full()
            .p_3()
            .gap_1()
            .bg(cx.theme().background)
            .border_r_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_3()
                    .child(
                        div()
                            .text_lg()
                            .font_bold()
                            .child("Remby"),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .gap_1()
                    .mt_2()
                    .children(nav_items.into_iter().map(move |(label, icon, view)| {
                        let is_active = self.current_view == view;
                        h_flex()
                            .id(label)
                            .w_full()
                            .items_center()
                            .gap_3()
                            .px_3()
                            .py_2()
                            .rounded(cx.theme().radius)
                            .text_sm()
                            .when(is_active, |this| {
                                this.bg(cx.theme().primary.opacity(0.1))
                                    .text_color(cx.theme().primary)
                            })
                            .when(!is_active, |this| {
                                this.text_color(cx.theme().foreground.opacity(0.7))
                                    .hover(|this| this.bg(cx.theme().muted.opacity(0.5)))
                            })
                            .child(Icon::new(icon).small())
                            .child(div().child(label))
                    })),
            )
    }
}
