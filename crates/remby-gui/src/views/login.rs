use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputState};

use crate::app::RembyApp;

#[derive(IntoElement)]
pub struct LoginView {
    server_input: Entity<InputState>,
    username_input: Entity<InputState>,
    password_input: Entity<InputState>,
    app: WeakEntity<RembyApp>,
}

impl LoginView {
    pub fn new(
        server_input: Entity<InputState>,
        username_input: Entity<InputState>,
        password_input: Entity<InputState>,
        app: WeakEntity<RembyApp>,
    ) -> Self {
        Self {
            server_input,
            username_input,
            password_input,
            app,
        }
    }
}

impl RenderOnce for LoginView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let error = self
            .app
            .upgrade()
            .map(|app| cx.read_entity(&app, |state, _| state.state.login_error.clone()))
            .unwrap_or_default();
        let has_error = !error.is_empty();

        let srv = self.server_input.clone();
        let usr = self.username_input.clone();
        let pwd = self.password_input.clone();
        let app_weak = self.app.clone();

        let error_for_closure = error.clone();

        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .child(
                v_flex()
                    .gap_6()
                    .w(px(380.))
                    .items_center()
                    .child(
                        v_flex()
                            .gap_2()
                            .items_center()
                            .child(div().text_3xl().font_bold().child("remby"))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Lightweight Emby Client"),
                            ),
                    )
                    .child(
                        v_flex()
                            .gap_4()
                            .w_full()
                            .p_6()
                            .rounded(cx.theme().radius)
                            .border_1()
                            .border_color(cx.theme().border)
                            .bg(cx.theme().popover)
                            .child(
                                v_flex()
                                    .gap_4()
                                    .child(
                                        v_flex()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_medium()
                                                    .child("Server URL"),
                                            )
                                            .child(Input::new(&self.server_input)),
                                    )
                                    .child(
                                        v_flex()
                                            .gap_1()
                                            .child(
                                                div().text_xs().font_medium().child("Username"),
                                            )
                                            .child(Input::new(&self.username_input)),
                                    )
                                    .child(
                                        v_flex()
                                            .gap_1()
                                            .child(
                                                div().text_xs().font_medium().child("Password"),
                                            )
                                            .child(Input::new(&self.password_input)),
                                    )
                                    .when(has_error, |this| {
                                        this.child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().danger)
                                                .child(error_for_closure),
                                        )
                                    })
                                    .child(
                                        Button::new("login")
                                            .primary()
                                            .label("Sign In")
                                            .w_full()
                                            .on_click(move |_, _window, cx| {
                                                if let Some(app) = app_weak.upgrade() {
                                                    let server = cx
                                                        .read_entity(&srv, |s, _| s.value().to_string());
                                                    let username = cx
                                                        .read_entity(&usr, |s, _| s.value().to_string());
                                                    let password = cx
                                                        .read_entity(&pwd, |s, _| s.value().to_string());
                                                    cx.update_entity(&app, |app, cx| {
                                                        app.handle_login(server, username, password, cx);
                                                    });
                                                }
                                            }),
                                    ),
                            ),
                    ),
            )
    }
}
