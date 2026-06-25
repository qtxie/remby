mod app;
mod image_loader;
mod state;
mod theme_adapter;
mod views;

use gpui::*;
use gpui_component::*;
use gpui_component::input::InputState;
use std::sync::OnceLock;

use app::RembyApp;

pub(crate) fn tokio_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime")
    })
}

fn main() {
    remby_core::emby::init_device_id();
    remby_core::i18n::init(&remby_core::config::load_config().language);

    gpui_platform::application().run(move |cx| {
        gpui_component::init(cx);
        app::init_key_bindings(cx);

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
                let browser_search_input = cx.new(|cx| {
                    InputState::new(window, cx).placeholder("Search library...")
                });
                let mpv_path_input = cx.new(|cx| {
                    InputState::new(window, cx).placeholder("mpv")
                });
                let view =
                    cx.new(|cx| RembyApp::new(server_input, username_input, password_input, browser_search_input, mpv_path_input, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}
