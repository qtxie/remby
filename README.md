# remby

<p align="center">
  <img src="assets/logo.png" alt="remby logo" width="200">
</p>

A lightweight Emby client with terminal UI and mpv playback.

> **Note**: This project was entirely written by AI (MiMo Code Agent).

[English](#english) | [中文](#中文)

---

## English

### Features

- **Home page** with Continue Watching, Latest media, and Following updates
- **Library browser** with sort (Name/Year/Rating/Date Added) and filter (Genre/Tag/Studio/Year/Folder)
- **Favorites & Following** — toggle favorite with `z`, follow series with `f`, unified management view
- **Settings** — configure which libraries appear, toggle latest items, reorder
- **Context-aware search** — search globally or within specific libraries
- **Source selection** with detailed info (resolution, codec, audio, file size)
- **Track selection** for video, audio, and subtitle
- **Resume playback** — choose to resume from saved position or play from start
- **Series info** — view seasons, episodes, and similar shows
- **mpv integration** — launches mpv for playback with full track support
- **Lazy loading** — items load at end of list with context-aware messages
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

#### Global

| Key | Action |
|-----|--------|
| `Enter` | Select / Play |
| `Esc` | Go back / Cancel |
| `q` | Quit |
| `/` | Search |
| `f` | Follow/unfollow series |
| `F` | View favorites & following |
| `e` | Show series info |
| `l` | Open libraries |
| `z` | Toggle favorite |
| `s` | Open settings |
| `Ctrl+F` | Refresh home |

#### Library Browser

| Key | Action |
|-----|--------|
| `Ctrl+S` | Open sort panel |
| `Ctrl+F` | Open filter panel |
| `f` | Follow/unfollow series |
| `z` | Toggle favorite |
| `Z` | View favorites |
| `/` | Search within library |
| `e` | Show series info |
| `c` | Clear all filters |
| `Enter` | Open item / Apply sort/filter |
| `Esc` | Close panel / Go back |

#### Filter Panel

| Key | Action |
|-----|--------|
| `↑`/`↓` or `k`/`j` | Navigate items |
| `Enter` | Select / Toggle filter |
| `←`/`→` | Switch section (Genre/Tag/Studio/Year/Folder) |
| `Esc` | Cancel without applying |

#### Sort Panel

| Key | Action |
|-----|--------|
| `↑`/`↓` or `k`/`j` | Navigate options |
| `Enter` | Select sort order |
| `Esc` | Cancel |

#### Favorites & Following View

| Key | Action |
|-----|--------|
| `↑`/`↓` or `k`/`j` | Navigate items |
| `f` | Follow/unfollow series |
| `z` | Toggle favorite |
| `m` | Mark all episodes as watched |
| `Enter` | Open/play item |

---

## 中文

### 功能特性

- **首页** — 继续观看、最近添加、追剧更新
- **媒体库浏览** — 支持排序（名称/年份/评分/添加日期）和筛选（类型/标签/制片厂/年份/文件夹）
- **收藏与追剧** — 按 `z` 切换收藏，按 `f` 追剧，统一管理页面
- **设置** — 配置显示哪些媒体库、是否显示最新内容、调整顺序
- **上下文搜索** — 全局搜索或在特定媒体库内搜索
- **源选择** — 显示详细信息（分辨率、编码、音频、文件大小）
- **轨道选择** — 选择视频、音频和字幕轨道
- **断点续播** — 从上次播放位置继续或从头播放
- **剧集信息** — 查看季、集和相似剧集
- **mpv 集成** — 使用 mpv 播放，支持完整轨道选择
- **懒加载** — 滚动到底部自动加载更多
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

#### 全局

| 按键 | 功能 |
|------|------|
| `Enter` | 选择 / 播放 |
| `Esc` | 返回 / 取消 |
| `q` | 退出 |
| `/` | 搜索 |
| `f` | 追剧/取消追剧 |
| `F` | 查看收藏与追剧 |
| `e` | 显示剧集信息 |
| `l` | 打开媒体库 |
| `z` | 切换收藏 |
| `s` | 打开设置 |
| `Ctrl+F` | 刷新首页 |

#### 媒体库浏览

| 按键 | 功能 |
|------|------|
| `Ctrl+S` | 打开排序面板 |
| `Ctrl+F` | 打开筛选面板 |
| `f` | 追剧/取消追剧 |
| `z` | 切换收藏 |
| `Z` | 查看收藏 |
| `/` | 在媒体库内搜索 |
| `e` | 显示剧集信息 |
| `c` | 清除所有筛选 |
| `Enter` | 打开项目 / 应用排序或筛选 |
| `Esc` | 关闭面板 / 返回 |

#### 筛选面板

| 按键 | 功能 |
|------|------|
| `↑`/`↓` 或 `k`/`j` | 导航选项 |
| `Enter` | 选择 / 切换筛选 |
| `←`/`→` | 切换分类（类型/标签/制片厂/年份/文件夹） |
| `Esc` | 取消不应用 |

#### 排序面板

| 按键 | 功能 |
|------|------|
| `↑`/`↓` 或 `k`/`j` | 导航选项 |
| `Enter` | 选择排序方式 |
| `Esc` | 取消 |

#### 收藏与追剧页面

| 按键 | 功能 |
|------|------|
| `↑`/`↓` 或 `k`/`j` | 导航选项 |
| `f` | 追剧/取消追剧 |
| `z` | 切换收藏 |
| `m` | 标记所有剧集为已看 |
| `Enter` | 打开/播放项目 |

---

## License

[MIT](LICENSE)
