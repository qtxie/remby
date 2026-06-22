use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;

#[derive(Debug)]
pub enum MpvEvent {
    LogLine(String, String),  // (line, level)
    Position(f64),
    Duration(f64),
    PlaybackStarted,
    PlaybackEnded,
}

pub fn play(url: &str, mpv_path: &str, video: Option<i32>, audio: Option<i32>, subtitle: Option<i32>, start_secs: Option<f64>) -> Result<(Child, mpsc::Receiver<MpvEvent>)> {
    let ipc_path = make_ipc_path();

    let mut cmd = Command::new(mpv_path);
    cmd.arg(url);
    cmd.arg("--term-osd-bar=no");
    cmd.arg("--term-status-msg=no");
    cmd.arg("--no-terminal");
    cmd.arg(format!("--input-ipc-server={}", ipc_path));
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    if let Some(secs) = start_secs {
        cmd.arg(format!("--start={:.3}", secs));
    }
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

    let (tx, rx) = mpsc::channel();

    // IPC thread
    let ipc = ipc_path.clone();
    std::thread::spawn(move || {
        for _ in 0..50 {
            if connect_ipc(&ipc).is_some() { break; }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let Some(mut stream) = connect_ipc(&ipc) else { return };
        let _ = stream.write_all(b"{\"command\":[\"observe_property\",1,\"time-pos\"]}\n");
        let _ = stream.write_all(b"{\"command\":[\"observe_property\",2,\"duration\"]}\n");
        let _ = stream.write_all(b"{\"command\":[\"observe_property\",3,\"pause\"]}\n");
        let _ = stream.write_all(b"{\"command\":[\"request_log_messages\",\"info\"]}\n");

        let mut buf = Vec::new();
        let mut read_buf = [0u8; 8192];
        loop {
            match stream.read(&mut read_buf) {
                Ok(0) => break,
                Ok(n) => {
                    buf.extend_from_slice(&read_buf[..n]);
                    while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                        let line = buf[..pos].to_vec();
                        buf = buf[pos + 1..].to_vec();
                        if let Ok(msg) = serde_json::from_slice::<serde_json::Value>(&line) {
                            handle_ipc_message(&msg, &tx);
                        }
                    }
                }
                Err(_) => break,
            }
        }
        let _ = tx.send(MpvEvent::PlaybackEnded);
        let _ = std::fs::remove_file(&ipc);
    });

    Ok((child, rx))
}

fn handle_ipc_message(msg: &serde_json::Value, tx: &mpsc::Sender<MpvEvent>) {
    if let Some(event) = msg.get("event").and_then(|v| v.as_str()) {
        match event {
            "property-change" => {
                let name = msg.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let data = msg.get("data");
                match name {
                    "time-pos" => {
                        if let Some(pos) = data.and_then(|v| v.as_f64()) {
                            let _ = tx.send(MpvEvent::Position(pos));
                        }
                    }
                    "duration" => {
                        if let Some(dur) = data.and_then(|v| v.as_f64()) {
                            let _ = tx.send(MpvEvent::Duration(dur));
                        }
                    }
                    _ => {}
                }
            }
            "file-loaded" => {
                let _ = tx.send(MpvEvent::PlaybackStarted);
            }
            "log-message" => {
                let prefix = msg.get("prefix").and_then(|v| v.as_str()).unwrap_or("");
                let level = msg.get("level").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let text = msg.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let line = if prefix.is_empty() {
                    text.trim().to_string()
                } else {
                    format!("[{}] {}", prefix, text.trim())
                };
                if !line.is_empty() {
                    let _ = tx.send(MpvEvent::LogLine(line, level));
                }
            }
            _ => {}
        }
    }
}

fn make_ipc_path() -> String {
    if cfg!(target_os = "windows") {
        format!(r"\\.\pipe\remby-mpv-{}", std::process::id())
    } else {
        let tmp = std::env::temp_dir();
        tmp.join(format!("remby-mpv-{}.sock", std::process::id()))
            .to_string_lossy()
            .to_string()
    }
}

#[cfg(unix)]
fn connect_ipc(path: &str) -> Option<Box<dyn ReadWrite>> {
    std::os::unix::net::UnixStream::connect(path).ok().map(|s| Box::new(s) as Box<dyn ReadWrite>)
}

#[cfg(windows)]
fn connect_ipc(path: &str) -> Option<Box<dyn ReadWrite>> {
    // Windows named pipe - try connecting with retries
    use std::time::Duration;
    for _ in 0..10 {
        match std::fs::OpenOptions::new().read(true).write(true).open(path) {
            Ok(f) => return Some(Box::new(f)),
            Err(_) => std::thread::sleep(Duration::from_millis(100)),
        }
    }
    None
}

trait ReadWrite: Read + Write + Send {}
impl<T: Read + Write + Send> ReadWrite for T {}

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
