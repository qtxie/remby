mod app;
mod state;
mod views;

use gpui::*;
use gpui_component::*;
use gpui_component::input::InputState;

use app::RembyApp;

fn main() {
    gpui_platform::application().run(move |cx| {
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let server_input = cx.new(|cx| {
                    InputState::new(window, cx)
                        .placeholder("https://your-server:8096")
                });
                let username_input = cx.new(|cx| {
                    InputState::new(window, cx).placeholder("Username")
                });
                let password_input = cx.new(|cx| {
                    InputState::new(window, cx).placeholder("Password")
                });
                let view =
                    cx.new(|cx| RembyApp::new(server_input, username_input, password_input, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}
