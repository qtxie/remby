use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputState};
use gpui_component::scroll::ScrollableElement;

use crate::app::RembyApp;
use crate::state::SettingsTab;

#[derive(IntoElement)]
pub struct SettingsView {
    app: WeakEntity<RembyApp>,
    mpv_path_input: Entity<InputState>,
}

impl SettingsView {
    pub fn new(app: WeakEntity<RembyApp>, mpv_path_input: Entity<InputState>) -> Self {
        Self {
            app,
            mpv_path_input,
        }
    }
}

impl RenderOnce for SettingsView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (active_tab, libraries, config) = self
            .app
            .upgrade()
            .map(|app| {
                cx.read_entity(&app, |state, _| {
                    (
                        state.state.settings_tab,
                        state.state.libraries.clone(),
                        state.state.config.clone(),
                    )
                })
            })
            .unwrap_or((SettingsTab::Libraries, vec![], remby_core::config::RembyConfig::default()));

        let app_clone = self.app.clone();
        let app_clone2 = self.app.clone();

        let radius = cx.theme().radius;
        let border = cx.theme().border;
        let background = cx.theme().background;
        let primary = cx.theme().primary;
        let muted = cx.theme().muted;
        let foreground = cx.theme().foreground;
        let muted_foreground = cx.theme().muted_foreground;

        let tabs: Vec<(SettingsTab, &str)> = vec![
            (SettingsTab::Libraries, "Libraries"),
            (SettingsTab::MpvPath, "MPV Path"),
            (SettingsTab::Language, "Language"),
            (SettingsTab::Theme, "Theme"),
            (SettingsTab::TrackPreferences, "Track Preferences"),
        ];

        let sidebar = v_flex()
            .w(px(200.))
            .h_full()
            .p_3()
            .gap_1()
            .border_r_1()
            .border_color(border)
            .bg(background)
            .child(
                div()
                    .text_lg()
                    .font_bold()
                    .mb_2()
                    .child("Settings"),
            )
            .children(tabs.into_iter().enumerate().map(move |(idx, (tab, label))| {
                let is_active = active_tab == tab;
                let app_w = app_clone.clone();
                h_flex()
                    .id(("settings-tab", idx))
                    .w_full()
                    .items_center()
                    .px_3()
                    .py_2()
                    .rounded(radius)
                    .text_sm()
                    .cursor_pointer()
                    .when(is_active, |this| {
                        this.bg(primary.opacity(0.1))
                            .text_color(primary)
                    })
                    .when(!is_active, |this| {
                        this.text_color(foreground.opacity(0.7))
                            .hover(|this| this.bg(muted.opacity(0.5)))
                    })
                    .child(div().child(label))
                    .on_click(move |_, _window, cx| {
                        if let Some(app) = app_w.upgrade() {
                            cx.update_entity(&app, |app, _cx| {
                                app.state.settings_tab = tab;
                            });
                        }
                    })
            }));

        let content: AnyElement = match active_tab {
            SettingsTab::Libraries => {
                let mut lib_items: Vec<AnyElement> = Vec::new();

                for (lib_idx, lib) in libraries.iter().enumerate() {
                    let lib_id = lib.id.clone();
                    let lib_id2 = lib.id.clone();
                    let is_enabled = config.enabled_libraries.is_empty()
                        || config.enabled_libraries.contains(&lib.id);
                    let is_latest = config.latest_libraries.contains(&lib.id);

                    let app_w = app_clone2.clone();
                    let app_w2 = app_clone2.clone();

                    lib_items.push(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .py_2()
                            .px_3()
                            .rounded(radius)
                            .hover(|s| s.bg(muted.opacity(0.3)))
                            .child(
                                h_flex()
                                    .gap_3()
                                    .items_center()
                                    .child(div().text_sm().font_medium().child(lib.name.clone()))
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        Button::new(("enabled", lib_idx))
                                            .small()
                                            .label("Enabled")
                                            .selected(is_enabled)
                                            .on_click(move |_, _window, cx| {
                                                if let Some(app) = app_w.upgrade() {
                                                    cx.update_entity(&app, |app, _cx| {
                                                        if app.state.config.enabled_libraries.contains(&lib_id) {
                                                            app.state.config.enabled_libraries.retain(|x| x != &lib_id);
                                                        } else {
                                                            app.state.config.enabled_libraries.push(lib_id.clone());
                                                        }
                                                    });
                                                }
                                            }),
                                    )
                                    .child(
                                        Button::new(("latest", lib_idx))
                                            .small()
                                            .label("Latest")
                                            .selected(is_latest)
                                            .on_click(move |_, _window, cx| {
                                                if let Some(app) = app_w2.upgrade() {
                                                    cx.update_entity(&app, |app, _cx| {
                                                        if app.state.config.latest_libraries.contains(&lib_id2) {
                                                            app.state.config.latest_libraries.retain(|x| x != &lib_id2);
                                                        } else {
                                                            app.state.config.latest_libraries.push(lib_id2.clone());
                                                        }
                                                    });
                                                }
                                            }),
                                    ),
                            )
                            .into_any_element(),
                    );
                }

                if lib_items.is_empty() {
                    v_flex()
                        .flex_1()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .text_sm()
                                .text_color(muted_foreground)
                                .child("No libraries loaded. Navigate to Libraries first."),
                        )
                        .into_any_element()
                } else {
                    v_flex()
                        .gap_1()
                        .children(lib_items)
                        .into_any_element()
                }
            }
            SettingsTab::MpvPath => {
                v_flex()
                    .gap_3()
                    .child(
                        div()
                            .text_sm()
                            .text_color(muted_foreground)
                            .child("Path to the mpv executable"),
                    )
                    .child(Input::new(&self.mpv_path_input))
                    .into_any_element()
            }
            SettingsTab::Language => {
                let current_lang = config.language.clone();
                let app_w = app_clone2.clone();
                let app_w2 = app_clone2.clone();

                v_flex()
                    .gap_3()
                    .child(
                        div()
                            .text_sm()
                            .text_color(muted_foreground)
                            .child("Interface language"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("lang-en")
                                    .label("English")
                                    .selected(current_lang == "en")
                                    .on_click(move |_, _window, cx| {
                                        if let Some(app) = app_w.upgrade() {
                                            cx.update_entity(&app, |app, _cx| {
                                                app.state.config.language = "en".to_string();
                                            });
                                        }
                                    }),
                            )
                            .child(
                                Button::new("lang-zh")
                                    .label("中文")
                                    .selected(current_lang == "zh")
                                    .on_click(move |_, _window, cx| {
                                        if let Some(app) = app_w2.upgrade() {
                                            cx.update_entity(&app, |app, _cx| {
                                                app.state.config.language = "zh".to_string();
                                            });
                                        }
                                    }),
                            ),
                    )
                    .into_any_element()
            }
            SettingsTab::Theme => {
                let themes = vec!["default", "green", "dracula"];
                let current_theme = config.theme.clone();

                let theme_buttons: Vec<AnyElement> = themes
                    .into_iter()
                    .enumerate()
                    .map(|(idx, theme_name)| {
                        let t = theme_name.to_string();
                        let is_active = current_theme == theme_name;
                        let app_w = app_clone2.clone();
                        Button::new(("theme", idx))
                            .label(theme_name)
                            .selected(is_active)
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_w.upgrade() {
                                    cx.update_entity(&app, |app, _cx| {
                                        app.state.config.theme = t.clone();
                                    });
                                }
                                crate::theme_adapter::apply_remby_theme(cx, &t);
                            })
                            .into_any_element()
                    })
                    .collect();

                v_flex()
                    .gap_3()
                    .child(
                        div()
                            .text_sm()
                            .text_color(muted_foreground)
                            .child("Application theme"),
                    )
                    .child(h_flex().gap_2().children(theme_buttons))
                    .into_any_element()
            }
            SettingsTab::TrackPreferences => {
                let current_resolution = config.preferred_resolution.clone();
                let current_audio = config.preferred_audio_language.clone();
                let current_sub = config.preferred_subtitle_language.clone();

                let resolutions = vec!["", "4k", "1080p", "720p", "480p"];
                let res_buttons: Vec<AnyElement> = resolutions
                    .into_iter()
                    .enumerate()
                    .map(|(idx, res)| {
                        let r = res.to_string();
                        let is_active = current_resolution == res;
                        let label = if res.is_empty() { "Any" } else { res };
                        let app_w = app_clone2.clone();
                        Button::new(("res", idx))
                            .label(label)
                            .selected(is_active)
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_w.upgrade() {
                                    cx.update_entity(&app, |app, _cx| {
                                        app.state.config.preferred_resolution = r.clone();
                                    });
                                }
                            })
                            .into_any_element()
                    })
                    .collect();

                let common_audio = vec!["", "eng", "chi", "jpn", "kor"];
                let audio_buttons: Vec<AnyElement> = common_audio
                    .into_iter()
                    .enumerate()
                    .map(|(idx, lang)| {
                        let l = lang.to_string();
                        let is_active = current_audio == lang;
                        let label = if lang.is_empty() { "Any" } else { lang };
                        let app_w = app_clone2.clone();
                        Button::new(("audio", idx))
                            .label(label)
                            .selected(is_active)
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_w.upgrade() {
                                    cx.update_entity(&app, |app, _cx| {
                                        app.state.config.preferred_audio_language = l.clone();
                                    });
                                }
                            })
                            .into_any_element()
                    })
                    .collect();

                let common_subs = vec!["", "eng", "chi", "jpn"];
                let sub_buttons: Vec<AnyElement> = common_subs
                    .into_iter()
                    .enumerate()
                    .map(|(idx, lang)| {
                        let l = lang.to_string();
                        let is_active = current_sub == lang;
                        let label = if lang.is_empty() { "Off" } else { lang };
                        let app_w = app_clone2.clone();
                        Button::new(("sub", idx))
                            .label(label)
                            .selected(is_active)
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_w.upgrade() {
                                    cx.update_entity(&app, |app, _cx| {
                                        app.state.config.preferred_subtitle_language = l.clone();
                                    });
                                }
                            })
                            .into_any_element()
                    })
                    .collect();

                v_flex()
                    .gap_4()
                    .child(
                        v_flex()
                            .gap_2()
                            .child(div().text_sm().font_medium().child("Resolution"))
                            .child(h_flex().gap_2().children(res_buttons)),
                    )
                    .child(
                        v_flex()
                            .gap_2()
                            .child(div().text_sm().font_medium().child("Audio Language"))
                            .child(h_flex().gap_2().children(audio_buttons)),
                    )
                    .child(
                        v_flex()
                            .gap_2()
                            .child(div().text_sm().font_medium().child("Subtitle Language"))
                            .child(h_flex().gap_2().children(sub_buttons)),
                    )
                    .into_any_element()
            }
        };

        let app_save = self.app.clone();
        let mpv_save = self.mpv_path_input.clone();

        v_flex()
            .size_full()
            .child(
                h_flex()
                    .flex_1()
                    .size_full()
                    .child(sidebar)
                    .child(
                        v_flex()
                            .flex_1()
                            .h_full()
                            .p_6()
                            .overflow_y_scrollbar()
                            .child(content),
                    ),
            )
            .child(
                h_flex()
                    .p_4()
                    .gap_2()
                    .justify_end()
                    .border_t_1()
                    .border_color(border)
                    .child(
                        Button::new("save-settings")
                            .primary()
                            .label("Save")
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_save.upgrade() {
                                    let mpv_val = cx.read_entity(&mpv_save, |s, _| {
                                        s.value().to_string()
                                    });
                                    let theme_name = cx.read_entity(&app, |a, _| {
                                        a.state.config.theme.clone()
                                    });
                                    cx.update_entity(&app, |app, _cx| {
                                        app.state.config.mpv_path = mpv_val;
                                        let _ = remby_core::config::save_config(&app.state.config);
                                        app.state.status_msg = "Settings saved".into();
                                    });
                                    crate::theme_adapter::apply_remby_theme(cx, &theme_name);
                                }
                            }),
                    ),
            )
    }
}
