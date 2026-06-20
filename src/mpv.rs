use anyhow::{Context, Result};
use std::process::{Child, Command};

pub fn play(url: &str, mpv_path: &str, video: Option<i32>, audio: Option<i32>, subtitle: Option<i32>, start_secs: Option<f64>) -> Result<Child> {
    let mut cmd = Command::new(mpv_path);
    cmd.arg(url);

    if let Some(secs) = start_secs {
        cmd.arg(format!("--start={:.3}", secs));
    }

    // Sequential position per type (0-based) → mpv track number (1-based)
    if let Some(vid) = video {
        cmd.arg(format!("--vid={}", vid + 1));
    }
    if let Some(aid) = audio {
        cmd.arg(format!("--aid={}", aid + 1));
    }
    if let Some(sid) = subtitle {
        cmd.arg(format!("--sid={}", sid + 1));
    }

    let child = cmd.spawn()
        .context(format!("Failed to launch mpv at '{mpv_path}'. Is mpv installed?"))?;
    Ok(child)
}
