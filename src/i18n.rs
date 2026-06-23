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
    "title.account_add" => "Add Account",
    "title.account_edit" => "Edit Account",
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
    "settings.library_prefs" => "Library Preferences",
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
    "status.refreshing_home" => "Refreshing home...",
    "status.searching" => "Searching for",
    "status.connecting" => "Connecting...",
    "status.launching_mpv" => "Launching mpv...",
    "status.mpv_closed" => "mpv closed",
    "status.playback_started" => "Playback started",
    "status.settings_saved" => "Settings saved",
    "status.account_saved" => "Account saved",
    "status.account_exists" => "Account already exists (same server + username)",
    "status.account_pw_changed" => "Password changed, confirm update?",
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
    "status.updating_favorite" => "Updating favorite...",
    "status.logging_in" => "Logging in as {}...",
    "status.marking_watched" => "Marking {} as watched...",

    // Playing view
    "playing.in_mpv" => "Playing in mpv...",
    "playing.choose_option" => "Choose playback option:",
    "playing.resume_from" => "Resume from",
    "playing.play_from_start" => "Play from start",
    "playing.play" => "Play",
    "playing.press_enter" => "(Enter)",

    // Footer hints
    "footer.home" => "l: libraries | /: search | u: accounts | s: settings | ?: help | q: quit",
    "footer.libraries" => "Enter: open | ↑/↓: navigate | ?: help",
    "footer.continue_watching" => "/: search | ?: help",
    "footer.items" => "f: follow | /: search | ?: help",
    "footer.search_results" => "f: follow | ?: help",
    "footer.track_select" => "←/→: section | Enter: play | ?: help",
    "footer.source_select" => "Enter: confirm | ?: help",
    "footer.episodes" => "e: episodes | ?: help",
    "footer.series_info" => "←/→: section | Enter: open | f: follow | e: episodes | ?: help",
    "footer.settings" => "Tab: section | Space: toggle | Shift+↑↓: move | Enter: save | ?: help",
    "footer.library_browser" => "Ctrl+s: Sort | Ctrl+f: Filter | /: search | e: info | ?: help",
    "footer.favorites" => "f: follow | z: unfavorite | m: mark watched | ?: help",
    "footer.account_manager" => "a: add | e: edit | d: delete | Enter: switch | Esc: back | ?: help",
    "footer.account_manager_form" => "↑/↓: field | Tab: next | Enter: save | Esc: cancel",
    "footer.account_confirm_update" => "y: confirm update | n: cancel",
    "footer.wizard" => "Tab: next field | Enter: continue | Esc: quit | ?: help",
    "footer.mpv_prompt" => "Enter: save & play | Esc: cancel | ?: help",
    "footer.search" => "Enter: search | Esc: cancel | ?: help",
    "footer.filter_panel" => "←/→: Section | Enter: Apply | ?: help",
    "footer.sort_panel" => "Enter: Select | ?: help",

    // Wizard
    "wizard.welcome" => "Welcome to remby! Please configure your connection.",
    "wizard.server" => "Server URL",
    "wizard.username" => "Username",
    "wizard.password" => "Password",
    "wizard.mpv_path" => "MPV Path",
    "wizard.skip_hint" => "(Tab to skip | Enter to login)",
    "wizard.hint" => "Enter: next | Tab: skip MPV | Esc: quit",

    // Account manager
    "account.add_new" => "+ Add new account",
    "account.delete_confirm" => "Delete account '{}'?",
    "account.confirm_delete" => "y: confirm | n: cancel",
    "account.label" => "Label",
    "account.server" => "Server",
    "account.username" => "Username",
    "account.password" => "Password",
    "account.form_hint" => "Tab: next field | Enter: save | Esc: cancel",

    // MPV prompt
    "mpv_prompt.message" => "MPV path not configured. Please enter the path to mpv:",
    "mpv_prompt.hint" => "Enter: save & play | Esc: cancel",

    // Track info
    "track.video" => "Video",
    "track.audio" => "Audio",
    "track.sub" => "Sub",
    "track.default" => "Default",
    "track.off" => "Off",

    // Error messages
    "error.item_not_found" => "Item not found (may have been deleted from server)",
    "error.server_error" => "Server error: HTTP {}",
    "error.item_detail_failed" => "Failed to fetch item detail",

    // Item display
    "item.following_update" => "Following Update",

    // Favorites
    "favorites.count" => "{} favorites",
    "favorites.following" => "following",

    // Help view labels
    "help.view.home" => "Home",
    "help.view.libraries" => "Libraries",
    "help.view.items" => "Items",
    "help.view.episodes" => "Episodes",
    "help.view.series_info" => "Series Info",
    "help.view.playing" => "Playing",
    "help.view.library_browser" => "Library Browser",
    "help.view.favorites" => "Favorites",
    "help.view.settings" => "Settings",

    // Help descriptions
    "help.navigate" => "Navigate",
    "help.open_item" => "Open item",
    "help.open_library" => "Open library",
    "help.search" => "Search",
    "help.follow" => "Follow series",
    "help.toggle_favorite" => "Toggle favorite",
    "help.unfavorite" => "Unfavorite",
    "help.series_info" => "Series info",
    "help.favorites" => "Favorites",
    "help.accounts" => "Accounts",
    "help.settings" => "Settings",
    "help.refresh" => "Refresh",
    "help.help" => "Help",
    "help.back" => "Back",
    "help.quit" => "Quit",
    "help.play_episode" => "Play episode",
    "help.switch_section" => "Switch section",
    "help.episodes" => "Episodes",
    "help.select_scroll" => "Select / scroll",
    "help.scroll_output" => "Scroll mpv output",
    "help.confirm" => "Confirm / play",
    "help.clear_filters" => "Clear filters",
    "help.sort" => "Sort",
    "help.filter" => "Filter",
    "help.mark_watched" => "Mark watched",
    "help.next_section" => "Next section",
    "help.toggle_switch" => "Toggle / switch",
    "help.toggle_item" => "Toggle item",
    "help.move_item" => "Move item",
    "help.save" => "Save",
    "help.cancel" => "Cancel",
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
    "title.account_add" => "添加账户",
    "title.account_edit" => "编辑账户",
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
    "settings.library_prefs" => "媒体库偏好",
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
    "status.refreshing_home" => "刷新首页...",
    "status.searching" => "搜索中",
    "status.connecting" => "连接中...",
    "status.launching_mpv" => "启动 mpv...",
    "status.mpv_closed" => "mpv 已关闭",
    "status.playback_started" => "播放已开始",
    "status.settings_saved" => "设置已保存",
    "status.account_saved" => "账户已保存",
    "status.account_exists" => "账户已存在（相同服务器 + 用户名）",
    "status.account_pw_changed" => "密码已变更，确认更新？",
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
    "status.updating_favorite" => "正在更新收藏...",
    "status.logging_in" => "正在登录 {}...",
    "status.marking_watched" => "正在标记 {} 为已看...",

    // Playing view
    "playing.in_mpv" => "正在 mpv 中播放...",
    "playing.choose_option" => "选择播放方式:",
    "playing.resume_from" => "继续播放",
    "playing.play_from_start" => "从头播放",
    "playing.play" => "播放",
    "playing.press_enter" => "(回车)",

    // Footer hints
    "footer.home" => "l: 媒体库 | /: 搜索 | u: 账户 | s: 设置 | ?: 帮助 | q: 退出",
    "footer.libraries" => "Enter: 打开 | ↑/↓: 导航 | ?: 帮助",
    "footer.continue_watching" => "/: 搜索 | ?: 帮助",
    "footer.items" => "f: 追剧 | /: 搜索 | ?: 帮助",
    "footer.search_results" => "f: 追剧 | ?: 帮助",
    "footer.track_select" => "←/→: 分区 | 回车: 播放 | ?: 帮助",
    "footer.source_select" => "回车: 确认 | ?: 帮助",
    "footer.episodes" => "e: 剧集 | ?: 帮助",
    "footer.series_info" => "←/→: 分区 | 回车: 打开 | f: 追剧 | e: 剧集 | ?: 帮助",
    "footer.settings" => "Tab: 分区 | 空格: 切换 | Shift+↑↓: 移动 | 回车: 保存 | ?: 帮助",
    "footer.library_browser" => "Ctrl+s: 排序 | Ctrl+f: 筛选 | /: 搜索 | e: 信息 | ?: 帮助",
    "footer.favorites" => "f: 追剧 | z: 取消收藏 | m: 标记已看 | ?: 帮助",
    "footer.account_manager" => "a: 添加 | e: 编辑 | d: 删除 | 回车: 切换 | Esc: 返回 | ?: 帮助",
    "footer.account_manager_form" => "↑/↓: 字段 | Tab: 下一项 | 回车: 保存 | Esc: 取消",
    "footer.account_confirm_update" => "y: 确认更新 | n: 取消",
    "footer.wizard" => "Tab: 下一项 | 回车: 继续 | Esc: 退出 | ?: 帮助",
    "footer.mpv_prompt" => "回车: 保存并播放 | Esc: 取消 | ?: 帮助",
    "footer.search" => "回车: 搜索 | Esc: 取消 | ?: 帮助",
    "footer.filter_panel" => "←/→: 分类 | 回车: 应用 | ?: 帮助",
    "footer.sort_panel" => "回车: 选择 | ?: 帮助",

    // Wizard
    "wizard.welcome" => "欢迎使用 remby！请配置连接信息。",
    "wizard.server" => "服务器地址",
    "wizard.username" => "用户名",
    "wizard.password" => "密码",
    "wizard.mpv_path" => "MPV 路径",
    "wizard.skip_hint" => "(Tab 跳过 | Enter 登录)",
    "wizard.hint" => "回车: 下一步 | Tab: 跳过 MPV | Esc: 退出",

    // Account manager
    "account.add_new" => "+ 添加新账户",
    "account.delete_confirm" => "确定删除账户 '{}'？",
    "account.confirm_delete" => "y: 确认 | n: 取消",
    "account.label" => "标签",
    "account.server" => "服务器",
    "account.username" => "用户名",
    "account.password" => "密码",
    "account.form_hint" => "Tab: 下一项 | 回车: 保存 | Esc: 取消",

    // MPV prompt
    "mpv_prompt.message" => "未配置 MPV 路径，请输入 mpv 的路径:",
    "mpv_prompt.hint" => "回车: 保存并播放 | Esc: 取消",

    // Track info
    "track.video" => "视频",
    "track.audio" => "音频",
    "track.sub" => "字幕",
    "track.default" => "默认",
    "track.off" => "关闭",

    // Error messages
    "error.item_not_found" => "未找到该项目（可能已从服务器删除）",
    "error.server_error" => "服务器错误：HTTP {}",
    "error.item_detail_failed" => "获取项目详情失败",

    // Item display
    "item.following_update" => "追剧更新",

    // Favorites
    "favorites.count" => "{} 个收藏",
    "favorites.following" => "追剧中",

    // Help view labels
    "help.view.home" => "首页",
    "help.view.libraries" => "媒体库",
    "help.view.items" => "项目",
    "help.view.episodes" => "剧集",
    "help.view.series_info" => "剧集信息",
    "help.view.playing" => "播放",
    "help.view.library_browser" => "媒体库浏览",
    "help.view.favorites" => "收藏夹",
    "help.view.settings" => "设置",

    // Help descriptions
    "help.navigate" => "导航",
    "help.open_item" => "打开项目",
    "help.open_library" => "打开媒体库",
    "help.search" => "搜索",
    "help.follow" => "追剧",
    "help.toggle_favorite" => "切换收藏",
    "help.unfavorite" => "取消收藏",
    "help.series_info" => "剧集信息",
    "help.favorites" => "收藏夹",
    "help.accounts" => "账户",
    "help.settings" => "设置",
    "help.refresh" => "刷新",
    "help.help" => "帮助",
    "help.back" => "返回",
    "help.quit" => "退出",
    "help.play_episode" => "播放剧集",
    "help.switch_section" => "切换分区",
    "help.episodes" => "剧集列表",
    "help.select_scroll" => "选择 / 滚动",
    "help.scroll_output" => "滚动 mpv 输出",
    "help.confirm" => "确认 / 播放",
    "help.clear_filters" => "清除筛选",
    "help.sort" => "排序",
    "help.filter" => "筛选",
    "help.mark_watched" => "标记已看",
    "help.next_section" => "下一分区",
    "help.toggle_switch" => "切换",
    "help.toggle_item" => "切换项目",
    "help.move_item" => "移动项目",
    "help.save" => "保存",
    "help.cancel" => "取消",
};
