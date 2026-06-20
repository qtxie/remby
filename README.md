# remby

```
      ________       
   __/        \__    
  /   ________   \   
 |   /        \   |  
 |  |   ▶▶▶▶   |  |  
  \  \________/  /   
   \__        __/    
      \______/       
```

A lightweight Emby client with terminal UI and mpv playback.

> **Note**: This project was entirely written by AI (MiMo Code Agent).

[English](#english) | [中文](#中文)

---

## English

### Features

- **Home page** with Continue Watching and Latest media
- **Browse** libraries, folders, and media items
- **Search** across movies, series, and episodes
- **Source selection** with detailed info (resolution, codec, audio, file size)
- **Track selection** for video, audio, and subtitle
- **Resume playback** — choose to resume from saved position or play from start
- **Series info** — view seasons, episodes, and similar shows
- **mpv integration** — launches mpv for playback with full track support
- **Progressive loading** — items load in the background with animated spinner
- **Keyboard driven** — vim-style navigation (j/k/h/l)

### Requirements

- [Rust](https://www.rust-lang.org/tools/install) (for building)
- [mpv](https://mpv.io/installation/) (for playback)

**Supported platforms**: Windows, Linux, macOS

**mpv paths by platform**:
- Linux: `/usr/bin/mpv` or `/usr/local/bin/mpv`
- macOS: `/opt/homebrew/bin/mpv` (Homebrew) or `/usr/local/bin/mpv`
- Windows: `C:\Tools\mpv\mpv.exe` or add mpv to PATH

### Build

```bash
git clone https://github.com/yourusername/remby.git
cd remby
cargo build --release
```

The binary will be at `target/release/remby`.

### Usage

```bash
remby -s <server-url> -u <username> -p <password>
```

Or with environment variables:

```bash
export EMBY_SERVER=https://your-emby-server:8096
export EMBY_USER=your_username
export EMBY_PASS=your_password
remby
```

With custom mpv path:

```bash
remby -s https://your-server:8096 -u user -p pass --mpv /path/to/mpv
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑`/`↓` or `k`/`j` | Navigate up/down |
| `←`/`→` or `h`/`l` | Navigate left/right (sections) / Go back |
| `Enter` | Select / Play |
| `Esc` | Go back / Cancel |
| `/` | Start search |
| `e` | Show series info |
| `l` | Open libraries |
| `q` | Quit |

---

## 中文

### 功能特性

- **首页** — 继续观看和最近添加
- **浏览** — 媒体库、文件夹和媒体项目
- **搜索** — 搜索电影、剧集和剧集
- **源选择** — 显示详细信息（分辨率、编码、音频、文件大小）
- **轨道选择** — 选择视频、音频和字幕轨道
- **断点续播** — 从上次播放位置继续或从头播放
- **剧集信息** — 查看季、集和相似剧集
- **mpv 集成** — 使用 mpv 播放，支持完整轨道选择
- **渐进加载** — 后台加载数据，带加载动画
- **键盘驱动** — 支持 vim 风格导航（j/k/h/l）

### 环境要求

- [Rust](https://www.rust-lang.org/tools/install)（编译用）
- [mpv](https://mpv.io/installation/)（播放用）

### 编译

```bash
git clone https://github.com/yourusername/remby.git
cd remby
cargo build --release
```

二进制文件位于 `target/release/remby`。

### 使用方法

```bash
remby -s <服务器地址> -u <用户名> -p <密码>
```

或使用环境变量：

```bash
export EMBY_SERVER=https://你的emby服务器:8096
export EMBY_USER=你的用户名
export EMBY_PASS=你的密码
remby
```

指定 mpv 路径：

```bash
remby -s https://你的服务器:8096 -u 用户名 -p 密码 --mpv /path/to/mpv
```

### 快捷键

| 按键 | 功能 |
|------|------|
| `↑`/`↓` 或 `k`/`j` | 上下导航 |
| `←`/`→` 或 `h`/`l` | 左右切换（章节）/ 返回 |
| `Enter` | 选择 / 播放 |
| `Esc` | 返回 / 取消 |
| `/` | 搜索 |
| `e` | 显示剧集信息 |
| `l` | 打开媒体库 |
| `q` | 退出 |

---

## License

[MIT](LICENSE)
