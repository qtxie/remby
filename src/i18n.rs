use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lang {
    En,
    Zh,
}

static LANG: AtomicU8 = AtomicU8::new(0);

pub fn init(lang_str: &str) {
    let lang = match lang_str {
        "zh" | "zh-CN" | "chinese" => Lang::Zh,
        _ => Lang::En,
    };
    LANG.store(lang as u8, Ordering::Relaxed);
}

fn current_lang() -> Lang {
    match LANG.load(Ordering::Relaxed) {
        1 => Lang::Zh,
        _ => Lang::En,
    }
}

pub fn detect_system_lang() -> &'static str {
    // Check environment variables
    for var in &["LANG", "LC_ALL", "LC_MESSAGES", "LANGUAGE"] {
        if let Ok(val) = std::env::var(var) {
            if val.starts_with("zh") {
                return "zh";
            }
        }
    }
    // Windows: check common locale env
    if let Ok(locale) = std::env::var("MICROSOFT_WINDOWS_LOCALE") {
        if locale.starts_with("zh") {
            return "zh";
        }
    }
    // Windows: check USERPROFILE path for CJK hint
    if let Ok(profile) = std::env::var("USERPROFILE") {
        // If username contains CJK characters, likely Chinese user
        let username = profile.rsplit('\\').next().unwrap_or("");
        if username.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)) {
            return "zh";
        }
    }
    "en"
}

pub fn t(key: &str) -> &'static str {
    match current_lang() {
        Lang::En => EN.get(key).copied().unwrap_or(""),
        Lang::Zh => ZH.get(key).copied().unwrap_or(""),
    }
}

pub fn tf(key: &str, arg: &str) -> String {
    let template = t(key);
    template.replace("{}", arg)
}

static EN: phf::Map<&'static str, &'static str> = phf::phf_map! {
    // View titles
    "title.home" => "Remby",
    "title.libraries" => "Remby - Libraries",
    "title.favorites" => "Favorites",
    "title.playing" => "Playing",
    "title.settings" => "Settings",
    "title.search" => "Search",
    "title.account_manager" => "Account Manager",
    "title.wizard" => "Setup Wizard",
    "title.mpv_prompt" => "Configure MPV Path",
    "title.track_select" => "Select Tracks",
    "title.source_select" => "Select Source",
    "title.episodes" => "Episodes",
    "title.series" => "Series",
    "title.continue_watching" => "Continue Watching",
    "title.latest" => "Latest",

    // Section labels
    "section.home" => "Home",
    "section.libraries" => "Libraries",
    "section.overview" => "Overview",
    "section.seasons" => "Seasons",
    "section.episodes" => "Episodes",
    "section.similar" => "Similar",
    "section.video" => "Video",
    "section.audio" => "Audio",
    "section.subtitle" => "Subtitle",
    "section.sort_by" => "Sort By",
    "section.filter" => "Filter",
    "section.confirm" => "Confirm",
    "section.mpv_output" => "mpv output",
    "section.mpv" => "MPV",

    // Settings
    "settings.library_prefs" => "Settings - Library Preferences",
    "settings.enabled" => "Enabled",
    "settings.latest" => "Latest",
    "settings.library" => "Library",
    "settings.mpv_path" => "MPV Path",

    // Status messages
    "status.loading_home" => "Loading home...",
    "status.loading_libraries" => "Loading libraries...",
    "status.loading_favorites" => "Loading favorites...",
    "status.loading_items" => "Loading more items...",
    "status.loading" => "Loading",
    "status.searching" => "Searching for",
    "status.connecting" => "Connecting...",
    "status.launching_mpv" => "Launching mpv...",
    "status.mpv_closed" => "mpv closed",
    "status.playback_started" => "Playback started",
    "status.settings_saved" => "Settings saved",
    "status.account_saved" => "Account saved",
    "status.account_deleted" => "Account deleted",
    "status.logged_in" => "Logged in as",
    "status.login_failed" => "Login failed",
    "status.save_error" => "Save error",
    "status.marked_watched" => "Marked {} episodes as watched",
    "status.added_following" => "Added to following",
    "status.removed_following" => "Removed from following",
    "status.added_favorites" => "Added to favorites",
    "status.removed_favorites" => "Removed from favorites",
    "status.mpv_path_saved" => "MPV path saved",
    "status.mpv_path_required" => "MPV path is required",
    "status.server_required" => "Server URL is required",
    "status.username_required" => "Username is required",
    "status.password_required" => "Password is required",
    "status.fields_required" => "Server, username and password are required",

    // Playing view
    "playing.in_mpv" => "Playing in mpv...",
    "playing.choose_option" => "Choose playback option:",
    "playing.resume_from" => "Resume from",
    "playing.play_from_start" => "Play from start",
    "playing.play" => "Play",
    "playing.press_enter" => "(Enter)",

    // Footer hints
    "footer.home" => "l: libraries | /: search | f: follow | F: favorites | u: accounts | Ctrl+F: refresh | q: quit",
    "footer.continue_watching" => "/: search",
    "footer.items" => "f: follow | /: search",
    "footer.search_results" => "f: follow",
    "footer.track_select" => "←/→: section | Enter: play",
    "footer.source_select" => "Enter: confirm",
    "footer.episodes" => "e: episodes",
    "footer.series_info" => "←/→: section | Enter: open | f: follow | e: episodes",
    "footer.settings" => "Tab: section | ←/→: col | Space: toggle | Shift+↑↓: move | Enter: save",
    "footer.library_browser" => "Ctrl+s: Sort | Ctrl+f: Filter | /: search | e: info | z: Favorite | Z: Favorites",
    "footer.favorites" => "f: follow | z: unfavorite | m: mark watched",
    "footer.account_manager" => "a: add | e: edit | d: delete | Enter: switch | Esc: back",
    "footer.wizard" => "Tab: next field | Enter: continue | Esc: quit",
    "footer.mpv_prompt" => "Enter: save & play | Esc: cancel",
    "footer.search" => "Enter: search | Esc: cancel",
    "footer.filter_panel" => "←/→: Section | Enter: Apply",
    "footer.sort_panel" => "Enter: Select",

    // Wizard
    "wizard.welcome" => "Welcome to remby! Please configure your connection.",
    "wizard.server" => "Server URL",
    "wizard.username" => "Username",
    "wizard.password" => "Password",
    "wizard.mpv_path" => "MPV Path",
    "wizard.skip_hint" => "(Tab to skip)",
    "wizard.hint" => "Enter: next | Tab: skip MPV | Esc: quit",

    // Account manager
    "account.add_new" => "+ Add new account",
    "account.delete_confirm" => "Delete account '{}'?",
    "account.confirm_delete" => "y: confirm | n: cancel",
    "account.label" => "Label",
    "account.form_hint" => "Tab: next field | Enter: save | Esc: cancel",

    // MPV prompt
    "mpv_prompt.message" => "MPV path not configured. Please enter the path to mpv:",
    "mpv_prompt.hint" => "Enter: save & play | Esc: cancel",

    // Track info
    "track.video" => "Video",
    "track.audio" => "Audio",
    "track.sub" => "Sub",

    // Item display
    "item.following_update" => "Following Update",

    // Favorites
    "favorites.count" => "{} favorites",
    "favorites.following" => "following",
};

static ZH: phf::Map<&'static str, &'static str> = phf::phf_map! {
    // View titles
    "title.home" => "Remby",
    "title.libraries" => "Remby - 媒体库",
    "title.favorites" => "收藏夹",
    "title.playing" => "播放",
    "title.settings" => "设置",
    "title.search" => "搜索",
    "title.account_manager" => "账户管理",
    "title.wizard" => "设置向导",
    "title.mpv_prompt" => "配置 MPV 路径",
    "title.track_select" => "选择轨道",
    "title.source_select" => "选择源",
    "title.episodes" => "剧集",
    "title.series" => "剧集信息",
    "title.continue_watching" => "继续观看",
    "title.latest" => "最近添加",

    // Section labels
    "section.home" => "首页",
    "section.libraries" => "媒体库",
    "section.overview" => "简介",
    "section.seasons" => "季",
    "section.episodes" => "集",
    "section.similar" => "相似",
    "section.video" => "视频",
    "section.audio" => "音频",
    "section.subtitle" => "字幕",
    "section.sort_by" => "排序",
    "section.filter" => "筛选",
    "section.confirm" => "确认",
    "section.mpv_output" => "mpv 输出",
    "section.mpv" => "MPV",

    // Settings
    "settings.library_prefs" => "设置 - 媒体库偏好",
    "settings.enabled" => "启用",
    "settings.latest" => "最新",
    "settings.library" => "媒体库",
    "settings.mpv_path" => "MPV 路径",

    // Status messages
    "status.loading_home" => "加载首页...",
    "status.loading_libraries" => "加载媒体库...",
    "status.loading_favorites" => "加载收藏...",
    "status.loading_items" => "加载更多...",
    "status.loading" => "加载中",
    "status.searching" => "搜索中",
    "status.connecting" => "连接中...",
    "status.launching_mpv" => "启动 mpv...",
    "status.mpv_closed" => "mpv 已关闭",
    "status.playback_started" => "播放已开始",
    "status.settings_saved" => "设置已保存",
    "status.account_saved" => "账户已保存",
    "status.account_deleted" => "账户已删除",
    "status.logged_in" => "已登录",
    "status.login_failed" => "登录失败",
    "status.save_error" => "保存错误",
    "status.marked_watched" => "已标记 {} 集为已看",
    "status.added_following" => "已添加到追剧",
    "status.removed_following" => "已从追剧中移除",
    "status.added_favorites" => "已添加到收藏",
    "status.removed_favorites" => "已从收藏中移除",
    "status.mpv_path_saved" => "MPV 路径已保存",
    "status.mpv_path_required" => "请输入 MPV 路径",
    "status.server_required" => "请输入服务器地址",
    "status.username_required" => "请输入用户名",
    "status.password_required" => "请输入密码",
    "status.fields_required" => "请填写服务器、用户名和密码",

    // Playing view
    "playing.in_mpv" => "正在 mpv 中播放...",
    "playing.choose_option" => "选择播放方式:",
    "playing.resume_from" => "继续播放",
    "playing.play_from_start" => "从头播放",
    "playing.play" => "播放",
    "playing.press_enter" => "(回车)",

    // Footer hints
    "footer.home" => "l: 媒体库 | /: 搜索 | f: 追剧 | F: 收藏夹 | u: 账户 | Ctrl+F: 刷新 | q: 退出",
    "footer.continue_watching" => "/: 搜索",
    "footer.items" => "f: 追剧 | /: 搜索",
    "footer.search_results" => "f: 追剧",
    "footer.track_select" => "←/→: 分区 | 回车: 播放",
    "footer.source_select" => "回车: 确认",
    "footer.episodes" => "e: 剧集",
    "footer.series_info" => "←/→: 分区 | 回车: 打开 | f: 追剧 | e: 剧集",
    "footer.settings" => "Tab: 分区 | ←/→: 列 | 空格: 切换 | Shift+↑↓: 移动 | 回车: 保存",
    "footer.library_browser" => "Ctrl+s: 排序 | Ctrl+f: 筛选 | /: 搜索 | e: 信息 | z: 收藏 | Z: 收藏夹",
    "footer.favorites" => "f: 追剧 | z: 取消收藏 | m: 标记已看",
    "footer.account_manager" => "a: 添加 | e: 编辑 | d: 删除 | 回车: 切换 | Esc: 返回",
    "footer.wizard" => "Tab: 下一项 | 回车: 继续 | Esc: 退出",
    "footer.mpv_prompt" => "回车: 保存并播放 | Esc: 取消",
    "footer.search" => "回车: 搜索 | Esc: 取消",
    "footer.filter_panel" => "←/→: 分类 | 回车: 应用",
    "footer.sort_panel" => "回车: 选择",

    // Wizard
    "wizard.welcome" => "欢迎使用 remby！请配置连接信息。",
    "wizard.server" => "服务器地址",
    "wizard.username" => "用户名",
    "wizard.password" => "密码",
    "wizard.mpv_path" => "MPV 路径",
    "wizard.skip_hint" => "(Tab 跳过)",
    "wizard.hint" => "回车: 下一步 | Tab: 跳过 MPV | Esc: 退出",

    // Account manager
    "account.add_new" => "+ 添加新账户",
    "account.delete_confirm" => "确定删除账户 '{}'？",
    "account.confirm_delete" => "y: 确认 | n: 取消",
    "account.label" => "标签",
    "account.form_hint" => "Tab: 下一项 | 回车: 保存 | Esc: 取消",

    // MPV prompt
    "mpv_prompt.message" => "未配置 MPV 路径，请输入 mpv 的路径:",
    "mpv_prompt.hint" => "回车: 保存并播放 | Esc: 取消",

    // Track info
    "track.video" => "视频",
    "track.audio" => "音频",
    "track.sub" => "字幕",

    // Item display
    "item.following_update" => "追剧更新",

    // Favorites
    "favorites.count" => "{} 个收藏",
    "favorites.following" => "追剧中",
};
