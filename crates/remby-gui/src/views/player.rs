use gpui::*;
use gpui_component::*;
use gpui_component::button::Button;
use gpui_component::scroll::ScrollableElement;

use crate::app::RembyApp;

#[derive(IntoElement)]
pub struct PlayerView {
    app: WeakEntity<RembyApp>,
}

impl PlayerView {
    pub fn new(app: WeakEntity<RembyApp>) -> Self {
        Self { app }
    }
}

impl RenderOnce for PlayerView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let data = self.app.upgrade().map(|app| {
            cx.read_entity(&app, |state, _| {
                (
                    state.state.playing_item.clone(),
                    state.state.playing,
                    state.state.player_loading,
                    state.state.player_sources.clone(),
                    state.state.player_selected_source_idx,
                    state.state.player_video_tracks.clone(),
                    state.state.player_audio_tracks.clone(),
                    state.state.player_subtitle_tracks.clone(),
                    state.state.player_selected_video,
                    state.state.player_selected_audio,
                    state.state.player_selected_subtitle,
                    state.state.player_position,
                    state.state.player_duration,
                    state.state.player_logs.clone(),
                )
            })
        });

        let (
            item, playing, loading, sources, selected_source,
            video_tracks, audio_tracks, subtitle_tracks,
            sel_video, sel_audio, sel_subtitle,
            position, duration, logs,
        ) = match data {
            Some(d) => d,
            None => {
                return div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("No item loaded")
                    .into_any_element();
            }
        };

        let item = match item {
            Some(i) => i,
            None => {
                return div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("No item selected")
                    .into_any_element();
            }
        };

        let app_weak = self.app.clone();
        let app_weak2 = self.app.clone();
        let app_weak3 = self.app.clone();
        let app_weak4 = self.app.clone();
        let app_weak5 = self.app.clone();
        let item_id = item.id.clone();

        if playing {
            let pos_str = format_time(position);
            let dur_str = format_time(duration);

            return v_flex()
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
                                .flex_1()
                                .child(item.display_name())
                        )
                        .child(
                            div()
                                .text_sm()
                                .child(format!("{} / {}", pos_str, dur_str))
                        )
                        .child(
                            Button::new("stop-btn")
                                .label("Stop")
                                .on_click(move |_, _window, cx| {
                                    if let Some(app) = app_weak.upgrade() {
                                        cx.update_entity(&app, |app, _cx| {
                                            app.stop_player();
                                        });
                                    }
                                })
                        ),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .p_4()
                        .gap_1()
                        .text_xs()
                        .overflow_y_scrollbar()
                        .child(
                            if logs.is_empty() {
                                div().text_color(cx.theme().muted_foreground).child("Waiting for mpv output...").into_any_element()
                            } else {
                                v_flex()
                                    .gap_1()
                                    .children(logs.iter().rev().take(200).map(|line| {
                                        div().child(line.clone()).into_any_element()
                                    }))
                                    .into_any_element()
                            }
                        )
                )
                .into_any_element();
        }

        if loading {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(div().text_sm().child("Loading playback info..."))
                .into_any_element();
        }

        let mut content: Vec<AnyElement> = Vec::new();

        content.push(
            div()
                .text_lg()
                .font_bold()
                .mb_4()
                .child(item.display_name())
                .into_any_element()
        );

        if sources.len() > 1 && selected_source.is_none() {
            let mut source_buttons: Vec<AnyElement> = Vec::new();
            for (idx, src) in sources.iter().enumerate() {
                let label = src.display_label();
                let app_w = app_weak2.clone();
                let item_id_c = item_id.clone();
                source_buttons.push(
                    Button::new(("source", idx))
                        .label(label)
                        .on_click(move |_, _window, cx| {
                            if let Some(app) = app_w.upgrade() {
                                cx.update_entity(&app, |app, _cx| {
                                    app.select_player_source(idx, &item_id_c);
                                });
                            }
                        })
                        .into_any_element()
                );
            }

            content.push(
                v_flex()
                    .gap_2()
                    .mb_4()
                    .child(div().text_sm().font_bold().child("Select Source:"))
                    .child(v_flex().gap_2().children(source_buttons))
                    .into_any_element()
            );
        }

        if selected_source.is_some() {
            if video_tracks.len() > 1 {
                let mut buttons: Vec<AnyElement> = Vec::new();
                for (idx, track) in video_tracks.iter().enumerate() {
                    let label = track_label(track);
                    let app_w = app_weak3.clone();
                    let is_selected = sel_video == Some(idx as i32);
                    buttons.push(
                        Button::new(("vid", idx))
                            .label(label)
                            .selected(is_selected)
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_w.upgrade() {
                                    cx.update_entity(&app, |app, _cx| {
                                        app.state.player_selected_video = Some(idx as i32);
                                    });
                                }
                            })
                            .into_any_element()
                    );
                }
                content.push(
                    v_flex()
                        .gap_1()
                        .mb_3()
                        .child(div().text_sm().font_bold().child("Video Track:"))
                        .child(h_flex().gap_2().flex_wrap().children(buttons))
                        .into_any_element()
                );
            }

            if audio_tracks.len() > 1 {
                let mut buttons: Vec<AnyElement> = Vec::new();
                for (idx, track) in audio_tracks.iter().enumerate() {
                    let label = track_label(track);
                    let app_w = app_weak4.clone();
                    let is_selected = sel_audio == Some(idx as i32);
                    buttons.push(
                        Button::new(("aud", idx))
                            .label(label)
                            .selected(is_selected)
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_w.upgrade() {
                                    cx.update_entity(&app, |app, _cx| {
                                        app.state.player_selected_audio = Some(idx as i32);
                                    });
                                }
                            })
                            .into_any_element()
                    );
                }
                content.push(
                    v_flex()
                        .gap_1()
                        .mb_3()
                        .child(div().text_sm().font_bold().child("Audio Track:"))
                        .child(h_flex().gap_2().flex_wrap().children(buttons))
                        .into_any_element()
                );
            }

            if !subtitle_tracks.is_empty() {
                let mut buttons: Vec<AnyElement> = Vec::new();
                let none_selected = sel_subtitle == Some(-1);
                let app_w_n = app_weak5.clone();
                buttons.push(
                    Button::new("sub-none")
                        .label("None")
                        .selected(none_selected)
                        .on_click(move |_, _window, cx| {
                            if let Some(app) = app_w_n.upgrade() {
                                cx.update_entity(&app, |app, _cx| {
                                    app.state.player_selected_subtitle = Some(-1);
                                });
                            }
                        })
                        .into_any_element()
                );
                for (idx, track) in subtitle_tracks.iter().enumerate() {
                    let label = track_label(track);
                    let app_w = app_weak.clone();
                    let is_selected = sel_subtitle == Some(idx as i32);
                    buttons.push(
                        Button::new(("sub", idx))
                            .label(label)
                            .selected(is_selected)
                            .on_click(move |_, _window, cx| {
                                if let Some(app) = app_w.upgrade() {
                                    cx.update_entity(&app, |app, _cx| {
                                        app.state.player_selected_subtitle = Some(idx as i32);
                                    });
                                }
                            })
                            .into_any_element()
                    );
                }
                content.push(
                    v_flex()
                        .gap_1()
                        .mb_3()
                        .child(div().text_sm().font_bold().child("Subtitle Track:"))
                        .child(h_flex().gap_2().flex_wrap().children(buttons))
                        .into_any_element()
                );
            }

            let has_resume = item.resume_position_ticks().unwrap_or(0) > 0;
            let resume_secs = item.resume_position_ticks().map(|t| t as f64 / 10_000_000.0);
            let resume_label = resume_secs.map(|s| format!("Resume ({})", format_time(s))).unwrap_or_default();

            let mut play_buttons: Vec<AnyElement> = Vec::new();

            if has_resume {
                let app_wr = self.app.clone();
                let item_id_r = item.id.clone();
                let resume_s = resume_secs.unwrap_or(0.0);
                play_buttons.push(
                    Button::new("resume-btn")
                        .label(resume_label)
                        .on_click(move |_, _window, cx| {
                            if let Some(app) = app_wr.upgrade() {
                                cx.update_entity(&app, |app, cx| {
                                    app.launch_player(true, resume_s, cx);
                                });
                            }
                        })
                        .into_any_element()
                );
            }

            let app_wb = self.app.clone();
            play_buttons.push(
                Button::new("start-btn")
                    .label("Start from Beginning")
                    .on_click(move |_, _window, cx| {
                        if let Some(app) = app_wb.upgrade() {
                            cx.update_entity(&app, |app, cx| {
                                app.launch_player(false, 0.0, cx);
                            });
                        }
                    })
                    .into_any_element()
            );

            content.push(
                h_flex()
                    .gap_3()
                    .mt_4()
                    .children(play_buttons)
                    .into_any_element()
            );
        }

        v_flex()
            .size_full()
            .p_6()
            .children(content)
            .into_any_element()
    }
}

fn track_label(track: &remby_core::emby::MediaStream) -> String {
    if let Some(ref dt) = track.display_title {
        if !dt.is_empty() {
            return dt.clone();
        }
    }
    if let Some(ref title) = track.title {
        if !title.is_empty() {
            return title.clone();
        }
    }
    if !track.language.is_empty() {
        return format!("{} ({})", track.language, track.codec);
    }
    track.codec.clone()
}

fn format_time(secs: f64) -> String {
    let total = secs as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}
