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

- **Multi-account support** — add, edit, delete, and switch accounts; encrypted credential storage
- **Auto-login** — remembers last account, auto-connects on startup
- **First-time wizard** — guides setup on first launch (server, account, mpv path)
- **Home page** with Continue Watching, Latest media, and Following updates
- **Libraries view** — browse all libraries, navigate to library browser or latest items
- **Library browser** with sort (Name/Year/Rating/Date Added) and filter (Genre/Tag/Studio/Year/Folder)
- **Favorites & Following** — toggle favorite with `z`, follow series with `f`, unified management view
- **Settings** — configure libraries, latest items, mpv path, language, and theme
- **Theme system** — 3 built-in themes (default/green/dracula) + custom themes via `theme.json`
- **Multi-language** — English and 中文 with runtime switching
- **Help system** — press `?` anytime for context-sensitive keybinding reference
- **Context-aware search** — search globally or within specific libraries
- **Source selection** with detailed info (resolution, codec, audio, file size)
- **Track selection** for video, audio, and subtitle
- **Resume playback** — choose to resume from saved position or play from start
- **Series info** — view seasons, episodes, and similar shows
- **mpv integration** — launches mpv for playback with full track support
- **mpv output capture** — displays mpv log in a scrollable panel during playback
- **mpv auto-detection** — automatically finds mpv on PATH or common install locations
- **Lazy loading** — items load at end of list with context-aware messages
- **Keyboard driven** — vim-style navigation (j/k/h/l)

### Requirements

- [Rust](https://www.rust-lang.org/tools/install) (for building)
- [mpv](https://mpv.io/installation/) (for playback)

**Supported platforms**: Windows, Linux, macOS

**mpv auto-detection** searches these locations:

| Platform | Locations |
|----------|-----------|
| Windows | PATH, `Program Files\mpv`, `%LOCALAPPDATA%\mpv`, Scoop, Chocolatey |
| macOS | PATH, Homebrew (`/opt/homebrew/bin`, `/usr/local/bin`), App bundle, MacPorts |
| Linux | PATH, `/usr/bin`, `/usr/local/bin`, Snap, `~/.local/bin` |

### Build

```bash
git clone https://github.com/yourusername/remby.git
cd remby
cargo build --release
```

The binary will be at `target/release/remby`.

### Usage

First run (no config) — wizard opens automatically:

```bash
remby
```

With CLI args (auto-saved on successful login):

```bash
remby -s <server-url> -u <username> -p <password>
```

With environment variables:

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

Check version:

```bash
remby --version
```

### Keyboard Shortcuts

Press `?` in any view for a context-sensitive help popup.

#### Global

| Key | Action |
|-----|--------|
| `↑`/`↓` or `k`/`j` | Navigate |
| `Enter` | Select / Play |
| `Esc` | Go back / Cancel |
| `q` | Quit |
| `/` | Search |
| `f` | Follow/unfollow series |
| `z` | Toggle favorite |
| `e` | Show series info |
| `l` | Open libraries |
| `u` | Account manager |
| `s` | Open settings |
| `?` | Show help |

#### Libraries View

| Key | Action |
|-----|--------|
| `↑`/`↓` | Navigate |
| `Enter` | Open library / section items |
| `s` | Settings |
| `q` | Quit |

#### Settings (`s`)

| Key | Action |
|-----|--------|
| `Tab` | Switch section (Libraries / MPV Path / Language / Theme) |
| `↑`/`↓` or `k`/`j` | Navigate libraries |
| `Space` | Toggle checkbox |
| `Shift+↑/↓` | Reorder libraries |
| `←`/`→` | Toggle language / cycle theme |
| `Enter` | Save settings |
| `Esc` | Cancel |

#### Playing View

| Key | Action |
|-----|--------|
| `↑`/`↓` or `k`/`j` | Scroll mpv output / Select resume option |
| `PageUp`/`PageDown` | Scroll mpv output by 10 lines |
| `Enter` | Start playback |
| `Esc` | Stop mpv / Go back |

#### Library Browser

| Key | Action |
|-----|--------|
| `Ctrl+S` | Open sort panel |
| `Ctrl+F` | Open filter panel |
| `/` | Search within library |
| `e` | Show series info |
| `c` | Clear all filters |
| `Enter` | Open item / Apply sort/filter |
| `Esc` | Close panel / Go back |

#### Favorites & Following View

| Key | Action |
|-----|--------|
| `↑`/`↓` or `k`/`j` | Navigate items |
| `f` | Follow/unfollow series |
| `z` | Toggle favorite |
| `m` | Mark all episodes as watched |
| `Enter` | Open/play item |

### Custom Themes

Create `theme.json` in your config directory (`~/.config/remby/` on Linux, `%APPDATA%/remby/` on Windows):

```json
{
  "ocean": {
    "accent": "Blue",
    "text": "White",
    "muted": "DarkGray",
    "warning": "Yellow",
    "success": "Green",
    "error": "Red",
    "selection_fg": "Black"
  }
}
```

Then set `"theme": "ocean"` in `config.json`. All color fields are optional — omitted fields inherit defaults.

Available colors: `Black`, `Red`, `Green`, `Yellow`, `Blue`, `Magenta`, `Cyan`, `White`, `DarkGray`, `LightRed`, `LightGreen`, `LightYellow`, `LightBlue`, `LightMagenta`, `LightCyan`, `Gray`.

---

## 中文

### 功能特性

- **多账户管理** — 添加、编辑、删除和切换账户；加密存储凭据
- **自动登录** — 记住上次登录的账户，启动时自动连接
- **首次向导** — 首次运行时引导配置（服务器、账户、mpv 路径）
- **首页** — 继续观看、最近添加、追剧更新
- **媒体库视图** — 浏览所有媒体库，进入媒体库浏览或最新内容
- **媒体库浏览** — 支持排序（名称/年份/评分/添加日期）和筛选（类型/标签/制片厂/年份/文件夹）
- **收藏与追剧** — 按 `z` 切换收藏，按 `f` 追剧，统一管理页面
- **设置** — 配置媒体库、最新内容、mpv 路径、语言和主题
- **主题系统** — 3 个内置主题（default/green/dracula）+ 自定义主题（通过 `theme.json`）
- **多语言** — 支持 English 和 中文，运行时切换
- **帮助系统** — 随时按 `?` 查看当前视图的快捷键参考
- **上下文搜索** — 全局搜索或在特定媒体库内搜索
- **源选择** — 显示详细信息（分辨率、编码、音频、文件大小）
- **轨道选择** — 选择视频、音频和字幕轨道
- **断点续播** — 从上次播放位置继续或从头播放
- **剧集信息** — 查看季、集和相似剧集
- **mpv 集成** — 使用 mpv 播放，支持完整轨道选择
- **mpv 输出捕获** — 播放时在可滚动面板中显示 mpv 日志
- **mpv 自动检测** — 自动在 PATH 或常见安装位置查找 mpv
- **懒加载** — 滚动到底部自动加载更多
- **键盘驱动** — 支持 vim 风格导航（j/k/h/l）

### 环境要求

- [Rust](https://www.rust-lang.org/tools/install)（编译用）
- [mpv](https://mpv.io/installation/)（播放用）

**支持平台**：Windows、Linux、macOS

**mpv 自动检测**搜索以下位置：

| 平台 | 搜索位置 |
|------|----------|
| Windows | PATH、`Program Files\mpv`、`%LOCALAPPDATA%\mpv`、Scoop、Chocolatey |
| macOS | PATH、Homebrew（`/opt/homebrew/bin`、`/usr/local/bin`）、App bundle、MacPorts |
| Linux | PATH、`/usr/bin`、`/usr/local/bin`、Snap、`~/.local/bin` |

### 编译

```bash
git clone https://github.com/yourusername/remby.git
cd remby
cargo build --release
```

二进制文件位于 `target/release/remby`。

### 使用方法

首次运行（无配置文件）— 自动打开向导：

```bash
remby
```

使用命令行参数（登录成功后自动保存）：

```bash
remby -s <服务器地址> -u <用户名> -p <密码>
```

使用环境变量：

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

查看版本：

```bash
remby --version
```

### 快捷键

在任何视图按 `?` 可查看当前视图的快捷键帮助。

#### 全局

| 按键 | 功能 |
|------|------|
| `↑`/`↓` 或 `k`/`j` | 导航 |
| `Enter` | 选择 / 播放 |
| `Esc` | 返回 / 取消 |
| `q` | 退出 |
| `/` | 搜索 |
| `f` | 追剧/取消追剧 |
| `z` | 切换收藏 |
| `e` | 显示剧集信息 |
| `l` | 打开媒体库 |
| `u` | 账户管理 |
| `s` | 打开设置 |
| `?` | 显示帮助 |

#### 媒体库视图

| 按键 | 功能 |
|------|------|
| `↑`/`↓` | 导航 |
| `Enter` | 打开媒体库 / 分区内容 |
| `s` | 设置 |
| `q` | 退出 |

#### 设置（`s`）

| 按键 | 功能 |
|------|------|
| `Tab` | 切换分区（媒体库 / MPV 路径 / 语言 / 主题） |
| `↑`/`↓` 或 `k`/`j` | 导航媒体库 |
| `Space` | 切换复选框 |
| `Shift+↑/↓` | 调整媒体库顺序 |
| `←`/`→` | 切换语言 / 循环主题 |
| `Enter` | 保存设置 |
| `Esc` | 取消 |

#### 播放界面

| 按键 | 功能 |
|------|------|
| `↑`/`↓` 或 `k`/`j` | 滚动 mpv 输出 / 选择续播选项 |
| `PageUp`/`PageDown` | 滚动 mpv 输出 10 行 |
| `Enter` | 开始播放 |
| `Esc` | 停止 mpv / 返回 |

#### 媒体库浏览

| 按键 | 功能 |
|------|------|
| `Ctrl+S` | 打开排序面板 |
| `Ctrl+F` | 打开筛选面板 |
| `/` | 在媒体库内搜索 |
| `e` | 显示剧集信息 |
| `c` | 清除所有筛选 |
| `Enter` | 打开项目 / 应用排序或筛选 |
| `Esc` | 关闭面板 / 返回 |

#### 收藏与追剧页面

| 按键 | 功能 |
|------|------|
| `↑`/`↓` 或 `k`/`j` | 导航选项 |
| `f` | 追剧/取消追剧 |
| `z` | 切换收藏 |
| `m` | 标记所有剧集为已看 |
| `Enter` | 打开/播放项目 |

### 自定义主题

在配置目录创建 `theme.json`（Linux: `~/.config/remby/`，Windows: `%APPDATA%/remby/`）：

```json
{
  "ocean": {
    "accent": "Blue",
    "text": "White",
    "muted": "DarkGray",
    "warning": "Yellow",
    "success": "Green",
    "error": "Red",
    "selection_fg": "Black"
  }
}
```

然后在 `config.json` 中设置 `"theme": "ocean"`。所有颜色字段可选 — 省略的字段使用默认值。

可用颜色：`Black`、`Red`、`Green`、`Yellow`、`Blue`、`Magenta`、`Cyan`、`White`、`DarkGray`、`LightRed`、`LightGreen`、`LightYellow`、`LightBlue`、`LightMagenta`、`LightCyan`、`Gray`。

---

## License

[MIT](LICENSE)
