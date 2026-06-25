# GUI 重新设计实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use compose:subagent (recommended) or compose:execute to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 remby-gui 从简陋的原型升级为现代、美观、易用的桌面媒体客户端。

**Architecture:** 三栏式布局（Header + Sidebar + Content），激活已有组件（Sidebar、SearchBar、Toast、ImageLoader），升级 MediaCard 支持海报图片，完善主题系统，添加鼠标交互和动画。

**Tech Stack:** Rust, gpui, gpui-component, remby-core

## Global Constraints

- 所有 HTTP 请求必须在 Tokio runtime 中执行（`crate::tokio_runtime()`）
- 所有 UI 更新必须通过 `cx.update_entity()` 在 GPUI 主线程中执行
- 所有组件使用主题 token，无硬编码颜色
- 保持与 TUI 完全的功能对等
- 不引入新的外部依赖

---

### Task 1: 连接 ImageLoader 到 RembyApp

**Covers:** S3

**Files:**
- Modify: `crates/remby-gui/src/app.rs` — 添加 image_loader 字段
- Modify: `crates/remby-gui/src/state.rs` — 添加 image_cache 字段

**Interfaces:**
- Consumes: `crate::image_loader::ImageLoader`
- Produces: `RembyApp.image_loader: Arc<ImageLoader>`

- [ ] **Step 1: 在 RembyApp 中添加 ImageLoader 字段**

在 `app.rs` 的 `RembyApp` 结构体中添加：
```rust
pub image_loader: Arc<crate::image_loader::ImageLoader>,
```

- [ ] **Step 2: 在构造函数中初始化 ImageLoader**

在 `RembyApp::new()` 中：
```rust
image_loader: Arc::new(crate::image_loader::ImageLoader::new()),
```

- [ ] **Step 3: 验证编译通过**

Run: `cargo build -p remby-gui`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add crates/remby-gui/src/app.rs
git commit -m "feat(gui): add ImageLoader to RembyApp"
```

---

### Task 2: 实现海报加载函数

**Covers:** S3

**Files:**
- Modify: `crates/remby-gui/src/app.rs` — 添加 load_posters 方法

**Interfaces:**
- Consumes: `RembyApp.image_loader`, `RembyApp.state.client`, `RembyApp.state.server`
- Produces: `GuiState.poster_cache: HashMap<String, Arc<Image>>`

- [ ] **Step 1: 在 GuiState 中添加海报缓存**

在 `state.rs` 的 `GuiState` 结构体中添加：
```rust
pub poster_cache: std::collections::HashMap<String, Arc<gpui::Image>>,
```

在 `GuiState::new()` 中初始化：
```rust
poster_cache: std::collections::HashMap::new(),
```

- [ ] **Step 2: 在 app.rs 中添加 load_posters 方法**

```rust
pub fn load_posters(&self, item_ids: Vec<String>, cx: &mut Context<Self>) {
    let image_loader = self.image_loader.clone();
    let server = self.state.server.clone();
    let token = self.state.client.as_ref()
        .map(|c| c.token().to_string())
        .unwrap_or_default();
    let this = cx.entity();

    crate::tokio_runtime().spawn(async move {
        for item_id in item_ids {
            if let Some(image) = image_loader.load_poster(&server, &token, &item_id).await {
                this.update(cx, |app, cx| {
                    app.state.poster_cache.insert(item_id, image);
                    cx.notify();
                });
            }
        }
    });
}
```

- [ ] **Step 3: 验证编译通过**

Run: `cargo build -p remby-gui`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add crates/remby-gui/src/app.rs crates/remby-gui/src/state.rs
git commit -m "feat(gui): implement poster loading with cache"
```

---

### Task 3: 在数据加载中触发海报加载

**Covers:** S3

**Files:**
- Modify: `crates/remby-gui/src/app.rs` — 在 load_home_data, load_libraries_data, load_browser_data, load_favorites 后调用 load_posters

**Interfaces:**
- Consumes: `load_posters()` from Task 2
- Produces: 海报自动加载

- [ ] **Step 1: 在 load_home_data 完成后加载海报**

在 `load_home_data` 的成功回调中，`app.state.continue_watching = cw;` 之后添加：
```rust
let ids: Vec<String> = cw.iter()
    .chain(latest.iter())
    .chain(following.iter())
    .map(|i| i.id.clone())
    .collect();
app.load_posters(ids, cx);
```

- [ ] **Step 2: 在 load_libraries_data 完成后加载海报**

在 `load_libraries_data` 的成功回调中，`app.state.latest_items = latest;` 之后添加：
```rust
let ids: Vec<String> = latest.iter().map(|i| i.id.clone()).collect();
app.load_posters(ids, cx);
```

- [ ] **Step 3: 在 load_browser_data 完成后加载海报**

在 `load_browser_data` 的成功回调中，`app.state.browser_items = items;` 之后添加：
```rust
let ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
app.load_posters(ids, cx);
```

- [ ] **Step 4: 在 load_favorites 完成后加载海报**

在 `load_favorites` 的成功回调中，`app.state.favorites = items;` 之后添加：
```rust
let ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
app.load_posters(ids, cx);
```

- [ ] **Step 5: 验证编译通过**

Run: `cargo build -p remby-gui`
Expected: 编译成功

- [ ] **Step 6: Commit**

```bash
git add crates/remby-gui/src/app.rs
git commit -m "feat(gui): trigger poster loading after data fetch"
```

---

### Task 4: 升级 MediaCard 组件支持海报

**Covers:** S4

**Files:**
- Modify: `crates/remby-gui/src/views/components/media_card.rs` — 重构卡片设计

**Interfaces:**
- Consumes: `poster_image: Option<Arc<Image>>` from Task 1-3
- Produces: 升级后的 MediaCard 组件

- [ ] **Step 1: 重写 MediaCard 渲染逻辑**

替换 `media_card.rs` 的 `RenderOnce` 实现：
```rust
impl RenderOnce for MediaCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let poster_area = if let Some(image) = self.poster_image {
            div()
                .id(self.id.clone())
                .w_full()
                .h(px(270.))
                .rounded(cx.theme().radius)
                .overflow_hidden()
                .child(img(image).w_full().h_full().object_fit(gpui::ObjectFit::Cover))
        } else {
            div()
                .id(self.id.clone())
                .w_full()
                .h(px(270.))
                .bg(cx.theme().muted.opacity(0.15))
                .rounded(cx.theme().radius)
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Icon::new(IconName::Frame)
                        .large()
                        .text_color(cx.theme().muted_foreground.opacity(0.3)),
                )
        };

        let wrapper = div()
            .id(format!("{}-wrapper", self.id))
            .w(px(160.))
            .rounded_lg()
            .overflow_hidden()
            .cursor_pointer()
            .hover(|this| this.opacity(0.9));

        let wrapper = if let Some(handler) = self.on_click {
            wrapper.on_click(move |_event: &ClickEvent, window, cx| handler(window, cx))
        } else {
            wrapper
        };

        wrapper.child(
            v_flex()
                .gap_2()
                .child(poster_area)
                .child(
                    v_flex()
                        .gap_1()
                        .px_2()
                        .pb_2()
                        .child(
                            div()
                                .text_sm()
                                .font_medium()
                                .overflow_x_hidden()
                                .child(self.title),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .overflow_x_hidden()
                                .child(self.subtitle),
                        ),
                ),
        )
    }
}
```

- [ ] **Step 2: 验证编译通过**

Run: `cargo build -p remby-gui`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add crates/remby-gui/src/views/components/media_card.rs
git commit -m "feat(gui): upgrade MediaCard with poster support"
```

---

### Task 5: 在视图中传递海报到 MediaCard

**Covers:** S3, S4

**Files:**
- Modify: `crates/remby-gui/src/views/home.rs` — 传递海报
- Modify: `crates/remby-gui/src/views/browser.rs` — 传递海报
- Modify: `crates/remby-gui/src/views/favorites.rs` — 传递海报
- Modify: `crates/remby-gui/src/views/libraries.rs` — 传递海报

**Interfaces:**
- Consumes: `GuiState.poster_cache` from Task 2, `MediaCard.poster_image()` from Task 4
- Produces: 海报图片在卡片中显示

- [ ] **Step 1: 更新 home.rs 传递海报**

在 `HomeView` 的 `render` 方法中，创建 `MediaCard` 时添加：
```rust
.poster_image(app.state.poster_cache.get(&item.id).cloned())
```

- [ ] **Step 2: 更新 browser.rs 传递海报**

在 `BrowserView` 的 `render` 方法中，创建 `MediaCard` 时添加：
```rust
.poster_image(app.state.poster_cache.get(&item.id).cloned())
```

- [ ] **Step 3: 更新 favorites.rs 传递海报**

在 `FavoritesView` 的 `render` 方法中，创建 `MediaCard` 时添加：
```rust
.poster_image(app.state.poster_cache.get(&item.id).cloned())
```

- [ ] **Step 4: 更新 libraries.rs 传递海报**

在 `LibrariesView` 的 `render` 方法中，创建 `MediaCard` 时添加：
```rust
.poster_image(app.state.poster_cache.get(&item.id).cloned())
```

- [ ] **Step 5: 验证编译通过**

Run: `cargo build -p remby-gui`
Expected: 编译成功

- [ ] **Step 6: Commit**

```bash
git add crates/remby-gui/src/views/home.rs crates/remby-gui/src/views/browser.rs crates/remby-gui/src/views/favorites.rs crates/remby-gui/src/views/libraries.rs
git commit -m "feat(gui): pass poster images to MediaCard in all views"
```

---

### Task 6: 连接 SidebarNav 到主渲染

**Covers:** S2

**Files:**
- Modify: `crates/remby-gui/src/app.rs` — 在 Render 中使用 SidebarNav

**Interfaces:**
- Consumes: `crate::views::components::sidebar::SidebarNav`
- Produces: 侧边栏在主界面显示

- [ ] **Step 1: 在 Render 中添加 SidebarNav**

在 `app.rs` 的 `Render` 实现中，替换 `view_element` 之前的代码：

```rust
// 在 Render 开头添加
use crate::views::components::sidebar::SidebarNav;

// 在 view_element 之后，替换 v_flex() 构建：
let sidebar = if !matches!(self.state.view, View::Login) {
    Some(SidebarNav::new(self.state.view.clone()))
} else {
    None
};

v_flex()
    .id("remby-app")
    .size_full()
    .on_action(cx.listener(Self::handle_go_back))
    .on_action(cx.listener(Self::handle_quit))
    .on_action(cx.listener(Self::handle_select_next))
    .on_action(cx.listener(Self::handle_select_prev))
    .on_action(cx.listener(Self::handle_select_item))
    .on_action(cx.listener(Self::handle_toggle_favorite))
    .on_action(cx.listener(Self::handle_toggle_follow))
    .on_action(cx.listener(Self::handle_navigate_settings))
    .on_action(cx.listener(Self::handle_navigate_libraries))
    .on_action(cx.listener(Self::handle_navigate_home))
    .when(has_toast, |this| {
        this.child(
            div()
                .px_4()
                .py_2()
                .rounded(cx.theme().radius)
                .mx_2()
                .mt_2()
                .bg(toast_bg)
                .border_1()
                .border_color(toast_border)
                .text_sm()
                .child(status_msg),
        )
    })
    .child(
        h_flex()
            .flex_1()
            .children(sidebar)
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .child(view_element),
            ),
    )
```

- [ ] **Step 2: 验证编译通过**

Run: `cargo build -p remby-gui`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add crates/remby-gui/src/app.rs
git commit -m "feat(gui): add SidebarNav to main layout"
```

---

### Task 7: 添加侧边栏导航点击处理

**Covers:** S2, S6

**Files:**
- Modify: `crates/remby-gui/src/views/components/sidebar.rs` — 添加 on_click 回调
- Modify: `crates/remby-gui/src/app.rs` — 处理侧边栏导航

**Interfaces:**
- Consumes: `SidebarNav.on_click` callback
- Produces: 侧边栏点击导航

- [ ] **Step 1: 更新 SidebarNav 支持点击回调**

修改 `sidebar.rs`：
```rust
pub struct SidebarNav {
    current_view: View,
    on_navigate: Option<Box<dyn Fn(View, &mut Window, &mut App)>>,
}

impl SidebarNav {
    pub fn new(current_view: View) -> Self {
        Self {
            current_view,
            on_navigate: None,
        }
    }

    pub fn on_navigate(mut self, handler: impl Fn(View, &mut Window, &mut App) + 'static) -> Self {
        self.on_navigate = Some(Box::new(handler));
        self
    }
}
```

- [ ] **Step 2: 在侧边栏项上添加点击事件**

在 `sidebar.rs` 的 `RenderOnce` 实现中，为每个导航项添加点击：
```rust
.children(nav_items.into_iter().map(move |(label, icon, view)| {
    let is_active = self.current_view == view;
    let on_navigate = self.on_navigate.clone();
    h_flex()
        .id(label)
        .w_full()
        .items_center()
        .gap_3()
        .px_3()
        .py_2()
        .rounded(cx.theme().radius)
        .text_sm()
        .cursor_pointer()
        .when(is_active, |this| {
            this.bg(cx.theme().primary.opacity(0.1))
                .text_color(cx.theme().primary)
        })
        .when(!is_active, |this| {
            this.text_color(cx.theme().foreground.opacity(0.7))
                .hover(|this| this.bg(cx.theme().muted.opacity(0.5)))
        })
        .on_click(move |_event, window, cx| {
            if let Some(ref handler) = on_navigate {
                handler(view.clone(), window, cx);
            }
        })
        .child(Icon::new(icon).small())
        .child(div().child(label))
}))
```

- [ ] **Step 3: 在 app.rs 中处理侧边栏导航**

在 `Render` 中创建 `SidebarNav` 时添加回调：
```rust
let sidebar = if !matches!(self.state.view, View::Login) {
    let this = cx.entity();
    Some(
        SidebarNav::new(self.state.view.clone())
            .on_navigate(move |view, _window, cx| {
                this.update(cx, |app, cx| {
                    app.state.navigate(view);
                    // 触发数据加载
                    match app.state.view {
                        View::Home => app.load_home_data(cx),
                        View::Libraries => app.load_libraries_data(cx),
                        View::Favorites => app.load_favorites(cx),
                        _ => {}
                    }
                    cx.notify();
                });
            })
    )
} else {
    None
};
```

- [ ] **Step 4: 验证编译通过**

Run: `cargo build -p remby-gui`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add crates/remby-gui/src/views/components/sidebar.rs crates/remby-gui/src/app.rs
git commit -m "feat(gui): add sidebar navigation click handling"
```

---

### Task 8: 连接 SearchBar 到 Header

**Covers:** S2

**Files:**
- Modify: `crates/remby-gui/src/app.rs` — 添加搜索栏到 Header

**Interfaces:**
- Consumes: `crate::views::components::search_bar::SearchBar`
- Produces: 搜索栏在 Header 中显示

- [ ] **Step 1: 在 Render 中添加 Header 布局**

在 `app.rs` 的 `Render` 实现中，添加 Header：
```rust
let header = if !matches!(self.state.view, View::Login) {
    Some(
        h_flex()
            .h(px(48.))
            .items_center()
            .px_4()
            .bg(cx.theme().background)
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .text_lg()
                    .font_bold()
                    .child("Remby"),
            )
            .child(div().flex_1()) // 占位
            .child(
                crate::views::components::search_bar::SearchBar::new(
                    self.browser_search_input.clone(),
                ),
            )
            .child(div().flex_1()) // 占位
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.state.server.clone()),
            )
    )
} else {
    None
};
```

- [ ] **Step 2: 更新主布局包含 Header**

在 `Render` 的返回值中，将 `header` 添加到布局：
```rust
v_flex()
    .id("remby-app")
    .size_full()
    // ... action handlers ...
    .children(header)
    .child(
        h_flex()
            .flex_1()
            .children(sidebar)
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .child(view_element),
            ),
    )
```

- [ ] **Step 3: 验证编译通过**

Run: `cargo build -p remby-gui`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add crates/remby-gui/src/app.rs
git commit -m "feat(gui): add SearchBar to header"
```

---

### Task 9: 连接 Toast 组件

**Covers:** S2

**Files:**
- Modify: `crates/remby-gui/src/app.rs` — 使用 Toast 组件替代内联 toast

**Interfaces:**
- Consumes: `crate::views::components::toast::Toast`
- Produces: Toast 通知使用统一组件

- [ ] **Step 1: 替换内联 toast 为 Toast 组件**

在 `app.rs` 的 `Render` 实现中，替换 toast 渲染：
```rust
.when(has_toast, |this| {
    this.child(
        crate::views::components::toast::Toast::new(status_msg, status_kind)
    )
})
```

- [ ] **Step 2: 验证编译通过**

Run: `cargo build -p remby-gui`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add crates/remby-gui/src/app.rs
git commit -m "feat(gui): use Toast component for notifications"
```

---

### Task 10: 完善主题系统 — 扩展颜色槽位

**Covers:** S5

**Files:**
- Modify: `crates/remby-gui/src/theme_adapter.rs` — 添加新的颜色槽位

**Interfaces:**
- Consumes: `remby_core::theme::Theme`
- Produces: 完整的主题颜色映射

- [ ] **Step 1: 扩展 apply_color_map 函数**

在 `theme_adapter.rs` 中，添加新的颜色槽位：
```rust
fn apply_color_map(
    cx: &mut App,
    accent: Hsla,
    text: Hsla,
    muted: Hsla,
    warning: Hsla,
    success: Hsla,
    error: Hsla,
    selection_fg: Hsla,
) {
    let mut theme = cx.global_mut::<Theme>();
    
    // 现有映射
    theme.primary = accent;
    theme.foreground = text;
    theme.muted_foreground = muted;
    theme.warning = warning;
    theme.success = success;
    theme.danger = error;
    
    // 新增映射
    theme.background = hsla(0., 0., 0.08, 1.);  // 深色背景
    theme.card = hsla(0., 0., 0.12, 1.);         // 卡片背景
    theme.border = hsla(0., 0., 0.2, 1.);        // 边框颜色
}
```

- [ ] **Step 2: 验证编译通过**

Run: `cargo build -p remby-gui`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add crates/remby-gui/src/theme_adapter.rs
git commit -m "feat(gui): extend theme color slots"
```

---

### Task 11: 添加 Badge 和 Progress 组件

**Covers:** S4

**Files:**
- Create: `crates/remby-gui/src/views/components/badge.rs`
- Create: `crates/remby-gui/src/views/components/progress.rs`
- Modify: `crates/remby-gui/src/views/components/mod.rs` — 注册新组件

**Interfaces:**
- Produces: `Badge` 和 `Progress` 组件

- [ ] **Step 1: 创建 Badge 组件**

创建 `badge.rs`：
```rust
use gpui::*;
use gpui_component::*;

#[derive(IntoElement)]
pub struct Badge {
    text: SharedString,
    variant: BadgeVariant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Default,
    Success,
    Warning,
    Error,
}

impl Badge {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::Default,
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (bg, fg) = match self.variant {
            BadgeVariant::Default => (cx.theme().primary.opacity(0.1), cx.theme().primary),
            BadgeVariant::Success => (cx.theme().success.opacity(0.1), cx.theme().success),
            BadgeVariant::Warning => (cx.theme().warning.opacity(0.1), cx.theme().warning),
            BadgeVariant::Error => (cx.theme().danger.opacity(0.1), cx.theme().danger),
        };

        div()
            .px_2()
            .py_1()
            .rounded_full()
            .bg(bg)
            .text_xs()
            .font_medium()
            .text_color(fg)
            .child(self.text)
    }
}
```

- [ ] **Step 2: 创建 Progress 组件**

创建 `progress.rs`：
```rust
use gpui::*;
use gpui_component::*;

#[derive(IntoElement)]
pub struct Progress {
    value: f32,  // 0.0 to 1.0
    height: Pixels,
}

impl Progress {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            height: px(4.),
        }
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }
}

impl RenderOnce for Progress {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .w_full()
            .h(self.height)
            .rounded_full()
            .bg(cx.theme().muted.opacity(0.2))
            .child(
                div()
                    .h_full()
                    .w(relative(self.value))
                    .rounded_full()
                    .bg(cx.theme().primary),
            )
    }
}
```

- [ ] **Step 3: 注册新组件**

在 `mod.rs` 中添加：
```rust
pub mod badge;
pub mod progress;
```

- [ ] **Step 4: 验证编译通过**

Run: `cargo build -p remby-gui`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add crates/remby-gui/src/views/components/
git commit -m "feat(gui): add Badge and Progress components"
```

---

### Task 12: 在 MediaCard 中使用 Badge 和 Progress

**Covers:** S4

**Files:**
- Modify: `crates/remby-gui/src/views/components/media_card.rs` — 使用 Badge 和 Progress

**Interfaces:**
- Consumes: `Badge`, `Progress` from Task 11
- Produces: 卡片显示徽章和进度条

- [ ] **Step 1: 更新 MediaCard 添加 badge 和 progress 字段**

在 `media_card.rs` 中添加：
```rust
pub struct MediaCard {
    // ... existing fields ...
    badge: Option<SharedString>,
    badge_variant: BadgeVariant,
    progress: Option<f32>,
}

impl MediaCard {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            // ... existing fields ...
            badge: None,
            badge_variant: BadgeVariant::Default,
            progress: None,
        }
    }

    pub fn badge(mut self, text: impl Into<SharedString>, variant: BadgeVariant) -> Self {
        self.badge = Some(text.into());
        self.badge_variant = variant;
        self
    }

    pub fn progress(mut self, value: f32) -> Self {
        self.progress = Some(value);
        self
    }
}
```

- [ ] **Step 2: 在渲染中使用 Badge 和 Progress**

在 `RenderOnce` 实现中：
```rust
// 在 poster_area 之后添加
.child(
    self.badge.map(|text| {
        Badge::new(text, self.badge_variant)
    })
)
.child(
    self.progress.map(|value| {
        Progress::new(value)
    })
)
```

- [ ] **Step 3: 验证编译通过**

Run: `cargo build -p remby-gui`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add crates/remby-gui/src/views/components/media_card.rs
git commit -m "feat(gui): add badge and progress to MediaCard"
```

---

### Task 13: 添加鼠标交互支持

**Covers:** S6

**Files:**
- Modify: `crates/remby-gui/src/views/components/sidebar.rs` — 鼠标悬停
- Modify: `crates/remby-gui/src/views/components/media_card.rs` — 鼠标悬停

**Interfaces:**
- Produces: 鼠标悬停效果

- [ ] **Step 1: 更新 SidebarNav 悬停效果**

在 `sidebar.rs` 中，确保每个导航项有 `cursor_pointer()` 和 `hover()` 效果（已在 Task 7 中完成）。

- [ ] **Step 2: 更新 MediaCard 悬停效果**

在 `media_card.rs` 中，添加悬停放大效果：
```rust
.hover(|this| {
    this.opacity(0.9)
        .shadow_lg()
})
```

- [ ] **Step 3: 验证编译通过**

Run: `cargo build -p remby-gui`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add crates/remby-gui/src/views/components/media_card.rs
git commit -m "feat(gui): add hover effects to components"
```

---

### Task 14: 架构清理 — 提取通用异步加载宏

**Covers:** S7

**Files:**
- Create: `crates/remby-gui/src/loaders.rs` — 通用异步加载函数
- Modify: `crates/remby-gui/src/app.rs` — 使用通用加载函数

**Interfaces:**
- Produces: `spawn_load!` 宏

- [ ] **Step 1: 创建 loaders.rs**

创建 `loaders.rs`：
```rust
use gpui::*;
use std::sync::Arc;

/// 通用异步加载函数
pub fn spawn_async<T, F>(
    entity: &Entity<T>,
    cx: &mut Context<T>,
    future: F,
    on_success: impl FnOnce(&mut T, &mut Context<T>, <F as std::future::Future>::Output) + 'static,
) where
    T: 'static,
    F: std::future::Future + 'static + Send,
    F::Output: Send + 'static,
{
    let entity = entity.clone();
    crate::tokio_runtime().spawn(async move {
        let result = future.await;
        entity.update(cx, |app, cx| {
            on_success(app, cx, result);
        });
    });
}
```

- [ ] **Step 2: 在 app.rs 中使用通用加载函数**

替换 `load_home_data` 中的重复模式：
```rust
pub fn load_home_data(&mut self, cx: &mut Context<Self>) {
    if self.state.client.is_none() {
        self.show_toast("Not connected to server".into(), crate::state::StatusKind::Error);
        return;
    }
    self.state.loading = true;
    self.state.loading_msg = "Loading home data...".into();

    let client = self.state.client.clone().unwrap();
    let this = cx.entity();

    crate::loaders::spawn_async(&this, cx, async move {
        let cw = client.get_resume_items(20).await.unwrap_or_default();
        let latest = client.get_latest_items(20).await.unwrap_or_default();
        let following = client.get_latest_items(20).await.unwrap_or_default()
            .into_iter()
            .filter(|item| item.series_id.is_some())
            .collect();
        (cw, latest, following)
    }, |app, cx, (cw, latest, following)| {
        app.state.continue_watching = cw;
        app.state.latest_items = latest;
        app.state.following_updates = following;
        app.state.loading = false;
        app.state.loading_msg.clear();
        let ids: Vec<String> = app.state.continue_watching.iter()
            .chain(app.state.latest_items.iter())
            .chain(app.state.following_updates.iter())
            .map(|i| i.id.clone())
            .collect();
        app.load_posters(ids, cx);
    });
}
```

- [ ] **Step 3: 验证编译通过**

Run: `cargo build -p remby-gui`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add crates/remby-gui/src/loaders.rs crates/remby-gui/src/app.rs crates/remby-gui/src/main.rs
git commit -m "refactor(gui): extract common async loading pattern"
```

---

### Task 15: 最终验证和清理

**Covers:** S8, S9

**Files:**
- All files — 最终检查

**Interfaces:**
- N/A

- [ ] **Step 1: 运行完整编译检查**

Run: `cargo build -p remby-gui`
Expected: 编译成功，无警告

- [ ] **Step 2: 运行 clippy 检查**

Run: `cargo clippy -p remby-gui`
Expected: 无警告

- [ ] **Step 3: 手动测试**

1. 启动 GUI：`cargo run -p remby-gui`
2. 登录测试
3. 验证侧边栏显示和导航
4. 验证海报图片加载
5. 验证搜索栏显示
6. 验证 Toast 通知
7. 验证鼠标悬停效果
8. 验证主题切换

- [ ] **Step 4: Commit 最终清理**

```bash
git add -A
git commit -m "chore(gui): final cleanup and verification"
```

---

## 实施顺序总结

| 任务 | 描述 | 依赖 |
|------|------|------|
| 1 | 连接 ImageLoader | 无 |
| 2 | 实现海报加载函数 | 1 |
| 3 | 触发海报加载 | 2 |
| 4 | 升级 MediaCard | 无 |
| 5 | 传递海报到视图 | 3, 4 |
| 6 | 连接 SidebarNav | 无 |
| 7 | 侧边栏点击处理 | 6 |
| 8 | 连接 SearchBar | 无 |
| 9 | 连接 Toast | 无 |
| 10 | 完善主题系统 | 无 |
| 11 | 添加 Badge/Progress | 无 |
| 12 | 使用 Badge/Progress | 11 |
| 13 | 鼠标交互 | 无 |
| 14 | 架构清理 | 1-13 |
| 15 | 最终验证 | 1-14 |
