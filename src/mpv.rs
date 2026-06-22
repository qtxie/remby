use anyhow::{Context, Result};
use std::path::Path;
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

pub fn find_mpv() -> Option<String> {
    if let Some(p) = find_in_path() {
        return Some(p);
    }
    find_in_known_locations()
}

fn find_in_path() -> Option<String> {
    let (cmd, arg) = if cfg!(target_os = "windows") {
        ("where", "mpv")
    } else {
        ("which", "mpv")
    };
    Command::new(cmd)
        .arg(arg)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().and_then(|s| {
                    s.lines().next().map(|l| l.trim().to_string())
                })
            } else {
                None
            }
        })
}

fn find_in_known_locations() -> Option<String> {
    let candidates: Vec<String> = if cfg!(target_os = "windows") {
        let mut paths = vec![
            r"C:\Program Files\mpv\mpv.exe".to_string(),
            r"C:\Program Files (x86)\mpv\mpv.exe".to_string(),
        ];
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            paths.push(format!(r"{}\mpv\mpv.exe", local));
        }
        if let Ok(home) = std::env::var("USERPROFILE") {
            paths.push(format!(r"{}\scoop\apps\mpv\current\mpv.exe", home));
            paths.push(format!(r"{}\.local\bin\mpv.exe", home));
        }
        paths.push(r"C:\ProgramData\chocolatey\bin\mpv.exe".to_string());
        paths
    } else if cfg!(target_os = "macos") {
        vec![
            "/opt/homebrew/bin/mpv".to_string(),
            "/usr/local/bin/mpv".to_string(),
            "/Applications/mpv.app/Contents/MacOS/mpv".to_string(),
            "/opt/local/bin/mpv".to_string(),
        ]
    } else {
        let mut paths = vec![
            "/usr/bin/mpv".to_string(),
            "/usr/local/bin/mpv".to_string(),
            "/snap/bin/mpv".to_string(),
        ];
        if let Ok(home) = std::env::var("HOME") {
            paths.push(format!("{}/.local/bin/mpv", home));
        }
        paths
    };

    candidates.into_iter().find(|p| Path::new(p).exists())
}
