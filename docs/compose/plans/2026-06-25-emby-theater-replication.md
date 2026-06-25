# Emby Theater GUI 复刻实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use compose:subagent (recommended) or compose:execute to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 复刻 Emby Theater 的视觉风格和交互模式，包括深蓝渐变背景、顶部导航、媒体库拼贴卡片、横版继续观看。

**Architecture:** 移除固定侧边栏，改用顶部 Header + 标签页导航。重构所有卡片组件以匹配 Emby Theater 的视觉风格。

**Tech Stack:** Rust, gpui, gpui-component, remby-core

## Global Constraints

- 所有 HTTP 请求必须在 Tokio runtime 中执行
- 所有 UI 更新必须通过 GPUI 主线程
- 颜色使用主题 token，无硬编码
- 保持与 TUI 完全的功能对等

---

### Task 1: 背景渐变 + 颜色系统

**Covers:** S2, S3

**Files:**
- Modify: `crates/remby-gui/src/theme_adapter.rs` — 添加 Emby 蓝色主题
- Modify: `crates/remby-gui/src/app.rs` — 应用渐变背景

**Steps:**
1. 在 `theme_adapter.rs` 中添加 `apply_emby_theater_theme` 函数：
```rust
pub fn apply_emby_theater_theme(cx: &mut App) {
    let mut theme = cx.global_mut::<Theme>();
    // 背景渐变色
    theme.background = hsla(0.57, 0.45, 0.22, 1.); // #1a3a5c
    theme.surface = hsla(0.57, 0.45, 0.15, 1.);    // #0d2137
    // 强调色
    theme.primary = hsla(0.30, 0.45, 0.50, 1.);    // #52b54b (Emby green)
    // 文字
    theme.foreground = hsla(0., 0., 1., 1.);        // #ffffff
    theme.muted_foreground = hsla(0., 0., 1., 0.6); // rgba(255,255,255,0.6)
    // 边框
    theme.border = hsla(0., 0., 1., 0.1);           // rgba(255,255,255,0.1)
}
```

2. 在 `app.rs` 的 `Render` 中应用渐变背景：
```rust
// 替换 .bg(cx.theme().background) 为渐变
.background_image(linear_gradient(
    180.,
    vec![
        color_picker::hsl(210., 45., 22.), // #1a3a5c
        color_picker::hsl(210., 45., 15.), // #0d2137
    ],
))
```

3. 验证编译：`cargo build -p remby-gui`
4. Commit

---

### Task 2: Header 重构（移除侧边栏）

**Covers:** S3, S4

**Files:**
- Modify: `crates/remby-gui/src/app.rs` — 重构 Render 布局
- Create: `crates/remby-gui/src/views/components/top_nav.rs` — 顶部导航组件

**Steps:**
1. 创建 `top_nav.rs` 顶部导航组件：
```rust
use gpui::*;
use gpui_component::*;

#[derive(IntoElement)]
pub struct TopNav {
    current_tab: Tab,
    on_tab_change: Option<Box<dyn Fn(Tab, &mut Window, &mut App)>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Home,
    Favorites,
}

impl TopNav {
    pub fn new(current_tab: Tab) -> Self {
        Self { current_tab, on_tab_change: None }
    }

    pub fn on_tab_change(mut self, handler: impl Fn(Tab, &mut Window, &mut App) + 'static) -> Self {
        self.on_tab_change = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TopNav {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .h(px(60.))
            .items_center()
            .px_6()
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .child(Icon::new(IconName::Menu).large())
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(24.))
                                    .h(px(24.))
                                    .rounded(px(4.))
                                    .bg(cx.theme().primary)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(Icon::new(IconName::Play).small())
                            )
                            .child(
                                div()
                                    .text_xl()
                                    .font_bold()
                                    .child("emby")
                            )
                    )
            )
            .child(div().flex_1())
            .child(
                h_flex()
                    .gap_2()
                    .child(self.tab_button("主页", Tab::Home))
                    .child(self.tab_button("喜欢", Tab::Favorites))
            )
            .child(div().flex_1())
            .child(
                h_flex()
                    .gap_4()
                    .child(Icon::new(IconName::Cast).large())
                    .child(Icon::new(IconName::Search).large())
                    .child(Icon::new(IconName::User).large())
                    .child(Icon::new(IconName::Settings).large())
            )
    }
}

impl TopNav {
    fn tab_button(&self, label: &str, tab: Tab) -> impl IntoElement {
        let is_active = self.current_tab == tab;
        div()
            .px_5()
            .py_2()
            .rounded_full()
            .when(is_active, |this| this.bg(cx.theme().border))
            .when(!is_active, |this| this.hover(|this| this.bg(cx.theme().border.opacity(0.5))))
            .text_sm()
            .child(label)
    }
}
```

2. 在 `app.rs` 的 `Render` 中：
- 移除 `SidebarNav` 相关代码
- 添加 `TopNav` 组件
- 布局改为 `v_flex()` + `TopNav` + 内容区域

3. 验证编译：`cargo build -p remby-gui`
4. Commit

---

### Task 3: 媒体库卡片（拼贴风格）

**Covers:** S3

**Files:**
- Create: `crates/remby-gui/src/views/components/library_card.rs`
- Modify: `crates/remby-gui/src/views/libraries.rs` — 使用新卡片

**Steps:**
1. 创建 `library_card.rs` 媒体库拼贴卡片：
```rust
use gpui::*;
use gpui_component::*;

#[derive(IntoElement)]
pub struct LibraryCard {
    name: String,
    posters: Vec<Option<Arc<Image>>>,
    on_click: Option<Box<dyn Fn(&mut Window, &mut App)>>,
}

impl LibraryCard {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            posters: Vec::new(),
            on_click: None,
        }
    }

    pub fn posters(mut self, posters: Vec<Option<Arc<Image>>>) -> Self {
        self.posters = posters;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for LibraryCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let wrapper = div()
            .w(px(140.))
            .cursor_pointer()
            .hover(|this| this.opacity(0.9).shadow_lg());

        let wrapper = if let Some(handler) = self.on_click {
            wrapper.on_click(move |_event, window, cx| handler(window, cx))
        } else {
            wrapper
        };

        wrapper.child(
            v_flex()
                .gap_2()
                .child(
                    // 2x2 poster grid
                    div()
                        .h(px(100.))
                        .rounded(px(8.))
                        .overflow_hidden()
                        .bg(cx.theme().border.opacity(0.3))
                        .grid()
                        .grid_cols(2)
                        .grid_rows(2)
                        .gap(px(2.))
                        .p(px(4.))
                        .children(self.posters.into_iter().take(4).map(|poster| {
                            if let Some(image) = poster {
                                div().rounded(px(2.)).overflow_hidden()
                                    .child(img(image).w_full().h_full().object_fit(gpui::ObjectFit::Cover))
                            } else {
                                div().rounded(px(2.)).bg(cx.theme().muted.opacity(0.3))
                            }
                        }))
                )
                .child(
                    div()
                        .text_sm()
                        .text_center()
                        .child(self.name)
                )
        )
    }
}
```

2. 在 `libraries.rs` 中使用 `LibraryCard` 替代 `MediaCard`

3. 验证编译：`cargo build -p remby-gui`
4. Commit

---

### Task 4: 继续观看（横版缩略图）

**Covers:** S3

**Files:**
- Create: `crates/remby-gui/src/views/components/continue_watching_card.rs`
- Modify: `crates/remby-gui/src/views/home.rs` — 使用新卡片

**Steps:**
1. 创建 `continue_watching_card.rs` 横版卡片：
```rust
use gpui::*;
use gpui_component::*;

#[derive(IntoElement)]
pub struct ContinueWatchingCard {
    title: String,
    subtitle: String,
    image: Option<Arc<Image>>,
    progress: Option<f32>,
    on_click: Option<Box<dyn Fn(&mut Window, &mut App)>>,
}

impl ContinueWatchingCard {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: String::new(),
            image: None,
            progress: None,
            on_click: None,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn image(mut self, image: Option<Arc<Image>>) -> Self {
        self.image = image;
        self
    }

    pub fn progress(mut self, value: f32) -> Self {
        self.progress = Some(value);
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ContinueWatchingCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let wrapper = div()
            .w(px(240.))
            .cursor_pointer()
            .hover(|this| this.opacity(0.9).shadow_lg());

        let wrapper = if let Some(handler) = self.on_click {
            wrapper.on_click(move |_event, window, cx| handler(window, cx))
        } else {
            wrapper
        };

        wrapper.child(
            v_flex()
                .gap_2()
                .child(
                    // Landscape thumbnail
                    div()
                        .h(px(140.))
                        .rounded(px(8.))
                        .overflow_hidden()
                        .bg(cx.theme().border.opacity(0.3))
                        .child(if let Some(image) = self.image {
                            img(image).w_full().h_full().object_fit(gpui::ObjectFit::Cover).into_any_element()
                        } else {
                            div().w_full().h_full().into_any_element()
                        })
                )
                .child(
                    v_flex()
                        .child(
                            div()
                                .text_sm()
                                .font_medium()
                                .child(self.title)
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(self.subtitle)
                        )
                )
        )
    }
}
```

2. 在 `home.rs` 中使用 `ContinueWatchingCard` 替代 `MediaCard`

3. 验证编译：`cargo build -p remby-gui`
4. Commit

---

### Task 5: 最新电影行 + 查看全部

**Covers:** S3

**Files:**
- Modify: `crates/remby-gui/src/views/home.rs` — 添加最新电影行

**Steps:**
1. 在 `home.rs` 的 `render` 中添加"最新电影"行：
```rust
// 最新电影
div()
    .mt_8()
    .child(
        h_flex()
            .items_center()
            .justify_between()
            .mb_4()
            .child(div().text_lg().font_bold().child("最新 电影"))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().primary)
                    .cursor_pointer()
                    .hover(|this| this.underline())
                    .child("查看全部 →")
            )
    )
    .child(
        h_flex()
            .gap_4()
            .overflow_x_scroll()
            .children(latest_movies.iter().map(|item| {
                MediaCard::new(item.id.clone())
                    .title(item.display_name())
                    .poster_image(poster_cache.get(&item.id).cloned())
                    .on_click({
                        let this = this.clone();
                        let item_id = item.id.clone();
                        move |_, cx| {
                            this.update(cx, |app, cx| {
                                app.play_item(&item_id, cx);
                            });
                        }
                    })
            }))
    )
```

2. 验证编译：`cargo build -p remby-gui`
3. Commit

---

### Task 6: 标签页导航

**Covers:** S4

**Files:**
- Modify: `crates/remby-gui/src/app.rs` — 实现标签切换逻辑

**Steps:**
1. 在 `GuiState` 中添加 `home_tab: HomeTab` 字段：
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeTab {
    Home,
    Favorites,
}
```

2. 在 `TopNav` 的 `on_tab_change` 回调中处理切换：
```rust
.on_tab_change(move |tab, _window, cx| {
    this.update(cx, |app, cx| {
        app.state.home_tab = match tab {
            Tab::Home => HomeTab::Home,
            Tab::Favorites => HomeTab::Favorites,
        };
        cx.notify();
    });
})
```

3. 在 `home.rs` 中根据 `home_tab` 显示不同内容

4. 验证编译：`cargo build -p remby-gui`
5. Commit

---

### Task 7: 交互效果 + 动画

**Covers:** S5

**Files:**
- Modify: 所有卡片组件 — 添加悬停效果

**Steps:**
1. 确保所有卡片都有：
- `.cursor_pointer()`
- `.hover(|this| this.opacity(0.9).shadow_lg())`
- 播放按钮覆盖层（继续观看卡片）

2. 添加页面切换过渡：
```rust
// 在 view_element 上添加动画
div()
    .opacity(1.)
    .transition("opacity", Duration::from_millis(200))
```

3. 验证编译：`cargo build -p remby-gui`
4. Commit

---

### Task 8: 最终验证

**Covers:** 所有

**Steps:**
1. 运行 `cargo build -p remby-gui` — 编译成功
2. 运行 `cargo clippy -p remby-gui` — 无错误
3. 手动测试：
- 启动 GUI
- 验证深蓝渐变背景
- 验证顶部导航（主页/喜欢标签）
- 验证媒体库拼贴卡片
- 验证继续观看横版卡片
- 验证最新电影行
- 验证悬停效果
4. Commit 最终清理
