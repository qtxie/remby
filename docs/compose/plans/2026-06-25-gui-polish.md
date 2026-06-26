# GUI 完善实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use compose:subagent (recommended) or compose:execute to implement this plan task-by-task.

**Goal:** 修复 remby-gui 的核心交互问题，实现布局自适应，添加视觉效果，完善数据展示，实现高级功能。

**Architecture:** 分 5 个阶段实施，每个阶段包含多个独立任务。优先修复交互问题，然后改进布局和视觉效果。

**Tech Stack:** Rust, gpui, gpui-component, remby-core

## Global Constraints

- 所有 HTTP 请求必须在 Tokio runtime 中执行
- 所有 UI 更新必须通过 GPUI 主线程
- 颜色使用主题 token，无硬编码
- 保持与 TUI 完全的功能对等

---

## Phase 1: 核心交互修复

### Task 1.1: Header 图标功能化

**Covers:** Header 交互

**Files:**
- Modify: `crates/remby-gui/src/app.rs`

**Steps:**
1. 为搜索图标添加点击处理，聚焦搜索栏
2. 为设置图标添加点击处理，导航到设置页
3. 为用户图标显示当前用户名
4. 为汉堡菜单添加点击处理（暂时无功能，预留接口）

```rust
// 搜索图标点击
.child(
    Icon::new(IconName::Search).large()
        .cursor_pointer()
        .on_click(cx.listener(|this, _, _, cx| {
            // 聚焦搜索栏
            cx.notify();
        }))
)

// 设置图标点击
.child(
    Icon::new(IconName::Settings).large()
        .cursor_pointer()
        .on_click(cx.listener(|this, _, _, cx| {
            this.state.navigate(View::Settings);
            cx.notify();
        }))
)
```

3. 验证编译：`cargo build -p remby-gui`
4. Commit

---

### Task 1.2: 标签栏功能化

**Covers:** 标签栏交互

**Files:**
- Modify: `crates/remby-gui/src/views/browser.rs`
- Modify: `crates/remby-gui/src/app.rs`

**Steps:**
1. 在 `GuiState` 中添加 `browser_tab: usize` 字段
2. 为每个标签添加 `on_click` 处理
3. 点击标签时更新 `browser_tab` 并重新加载数据
4. 使用主题色而非硬编码绿色

```rust
// 标签点击处理
.on_click({
    let this = this.clone();
    move |_, cx| {
        this.update(cx, |app, cx| {
            app.state.browser_tab = idx;
            app.load_browser_data(cx);
        });
    }
})
```

5. 验证编译：`cargo build -p remby-gui`
6. Commit

---

### Task 1.3: 卡片点击修复

**Covers:** 卡片交互

**Files:**
- Modify: `crates/remby-gui/src/views/home.rs`
- Modify: `crates/remby-gui/src/views/browser.rs`

**Steps:**
1. 修改卡片点击逻辑：系列内容导航到详情页，电影内容也导航到详情页
2. 详情页有播放按钮

```rust
// 修改前：直接播放
.on_click(move |_, cx| {
    this.update(cx, |app, cx| {
        app.play_item(&item_id, cx);
    });
})

// 修改后：导航到详情页
.on_click(move |_, cx| {
    this.update(cx, |app, cx| {
        if series_id.is_some() || media_type == "Series" {
            app.state.navigate(View::SeriesInfo);
            app.load_series_info(&sid, cx);
        } else {
            app.state.navigate(View::SeriesInfo);
            app.load_series_info(&item_id, cx);
        }
    });
})
```

3. 验证编译：`cargo build -p remby-gui`
4. Commit

---

## Phase 2: 布局自适应

### Task 2.1: 网格自适应

**Covers:** 布局响应式

**Files:**
- Modify: `crates/remby-gui/src/views/browser.rs`

**Steps:**
1. 移除硬编码 `let cols = 5usize;`
2. 使用 `flex_wrap()` 替代手动分块
3. 设置卡片最小宽度 120px，最大宽度 200px

```rust
// 替换手动分块为 flex_wrap
h_flex()
    .flex_wrap()
    .gap_4()
    .children(items.iter().map(|item| {
        div()
            .min_w(px(120.))
            .max_w(px(200.))
            .child(/* card content */)
    }))
```

4. 验证编译：`cargo build -p remby-gui`
5. Commit

---

### Task 2.2: 详情页响应式

**Covers:** 布局响应式

**Files:**
- Modify: `crates/remby-gui/src/views/series.rs`

**Steps:**
1. 使用 `flex_wrap()` 让海报和信息区自动换行
2. 窄窗口时改为垂直布局

```rust
h_flex()
    .flex_wrap()
    .gap_8()
    .child(/* poster */)
    .child(/* info section */)
```

3. 验证编译：`cargo build -p remby-gui`
4. Commit

---

## Phase 3: 视觉效果

### Task 3.1: 悬停效果

**Covers:** 视觉效果

**Files:**
- Modify: `crates/remby-gui/src/views/components/media_card.rs`
- Modify: `crates/remby-gui/src/views/components/continue_watching_card.rs`

**Steps:**
1. 添加悬停放大效果（1.05x）
2. 添加悬停覆盖层（播放按钮）

```rust
// 悬停效果
.hover(|this| {
    this.opacity(0.95)
        .shadow_xl()
        .border_color(cx.theme().primary)
})

// 悬停覆盖层
.child(
    div()
        .absolute()
        .inset_0()
        .opacity(0.) // 默认隐藏
        .hover(|this| this.opacity(1.)) // 悬停时显示
        .bg(gpui::black().opacity(0.5))
        .flex()
        .items_center()
        .justify_center()
        .child(Icon::new(IconName::Play).large().text_color(gpui::white()))
)
```

3. 验证编译：`cargo build -p remby-gui`
4. Commit

---

### Task 3.2: 骨架屏加载

**Covers:** 视觉效果

**Files:**
- Create: `crates/remby-gui/src/views/components/skeleton.rs`
- Modify: `crates/remby-gui/src/views/components/mod.rs`

**Steps:**
1. 创建骨架屏组件

```rust
#[derive(IntoElement)]
pub struct Skeleton {
    width: Pixels,
    height: Pixels,
    radius: Pixels,
}

impl Skeleton {
    pub fn new(width: Pixels, height: Pixels) -> Self {
        Self { width, height, radius: px(4.) }
    }
}

impl RenderOnce for Skeleton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .w(self.width)
            .h(self.height)
            .rounded(self.radius)
            .bg(cx.theme().border.opacity(0.3))
            .animate("skeleton-pulse", Duration::from_millis(1500), |this| {
                this.opacity(0.3).opacity(1.0)
            })
    }
}
```

2. 在卡片中使用骨架屏替代占位图标

```rust
// 替换占位图标
None => Skeleton::new(px(160.), px(220.)).into_any_element(),
```

3. 验证编译：`cargo build -p remby-gui`
4. Commit

---

### Task 3.3: 横滚行渐变边缘

**Covers:** 视觉效果

**Files:**
- Modify: `crates/remby-gui/src/views/home.rs`

**Steps:**
1. 在横滚行左右添加渐变遮罩

```rust
div()
    .relative()
    .child(
        h_flex()
            .overflow_x_scroll()
            .children(items)
    )
    // 左侧渐变
    .child(
        div()
            .absolute()
            .left_0()
            .top_0()
            .bottom_0()
            .w(px(20.))
            .bg(gpui::linear_gradient(90., vec![
                gpui::linear_color_stop(cx.theme().background, 0.),
                gpui::linear_color_stop(gpui::transparent_black(), 1.),
            ]))
    )
    // 右侧渐变
    .child(
        div()
            .absolute()
            .right_0()
            .top_0()
            .bottom_0()
            .w(px(20.))
            .bg(gpui::linear_gradient(270., vec![
                gpui::linear_color_stop(cx.theme().background, 0.),
                gpui::linear_color_stop(gpui::transparent_black(), 1.),
            ]))
    )
```

2. 验证编译：`cargo build -p remby-gui`
3. Commit

---

## Phase 4: 数据完善

### Task 4.1: 详情页数据修复

**Covers:** 数据完整性

**Files:**
- Modify: `crates/remby-gui/src/views/series.rs`
- Modify: `crates/remby-core/src/emby.rs`

**Steps:**
1. 移除硬编码演职人员数据
2. 从 API 获取导演/演员信息
3. 使用 API 的 tagline 字段
4. 背景使用 backdrop_cache 而非 poster_cache

```rust
// 背景使用 backdrop
.child(
    div()
        .absolute()
        .inset_0()
        .opacity(0.3)
        .child(match backdrop_cache.get(&item.id) {
            Some(img) => img(img.clone()).w_full().h_full().object_fit(gpui::ObjectFit::Cover).into_any_element(),
            None => div().w_full().h_full().into_any_element(),
        })
)

// 使用 API 的 tagline
.child(
    div().text_sm().italic().text_color(cx.theme().muted_foreground)
        .child(item.tagline.clone().unwrap_or_default())
)
```

5. 验证编译：`cargo build -p remby-gui`
6. Commit

---

### Task 4.2: 评分显示修复

**Covers:** 数据显示

**Files:**
- Modify: `crates/remby-gui/src/views/series.rs`
- Modify: `crates/remby-gui/src/views/browser.rs`

**Steps:**
1. 评分保留一位小数

```rust
// 修改前
let rating = item.community_rating.unwrap_or(0.0) as i32;
// 修改后
let rating = item.community_rating.unwrap_or(0.0);
format!("{:.1}", rating)
```

2. 验证编译：`cargo build -p remby-gui`
3. Commit

---

### Task 4.3: 继续观看信息

**Covers:** 数据显示

**Files:**
- Modify: `crates/remby-gui/src/views/home.rs`

**Steps:**
1. 显示 S01E05 格式的集数信息

```rust
// 构建集数信息
let episode_info = if let (Some(season), Some(episode)) = (item.parent_index_number, item.index_number) {
    format!("S{:02}E{:02}", season, episode)
} else {
    String::new()
};

// 显示在卡片下方
.child(
    div().text_xs().text_color(cx.theme().muted_foreground)
        .child(format!("{} {}", series_name, episode_info))
)
```

2. 验证编译：`cargo build -p remby-gui`
3. Commit

---

## Phase 5: 高级功能

### Task 5.1: Hero Banner

**Covers:** 首页增强

**Files:**
- Modify: `crates/remby-gui/src/views/home.rs`

**Steps:**
1. 在首页顶部添加 Hero Banner
2. 显示推荐内容的大图 + 信息 + 播放按钮

```rust
// Hero Banner
div()
    .h(px(400.))
    .relative()
    .overflow_hidden()
    .rounded(px(12.))
    .mb_8()
    .child(
        // 背景图
        div()
            .absolute()
            .inset_0()
            .child(backdrop_image)
    )
    .child(
        // 渐变遮罩
        div()
            .absolute()
            .inset_0()
            .bg(gpui::linear_gradient(90., vec![
                gpui::linear_color_stop(cx.theme().background.opacity(0.9), 0.),
                gpui::linear_color_stop(gpui::transparent_black(), 1.),
            ]))
    )
    .child(
        // 内容
        div()
            .absolute()
            .bottom_0()
            .left_0()
            .p_8()
            .child(title)
            .child(meta_info)
            .child(play_button)
    )
```

3. 验证编译：`cargo build -p remby-gui`
4. Commit

---

### Task 5.2: 无限滚动

**Covers:** 分页加载

**Files:**
- Modify: `crates/remby-gui/src/views/browser.rs`
- Modify: `crates/remby-gui/src/app.rs`

**Steps:**
1. 监听滚动事件
2. 接近底部时触发加载更多

```rust
// 在 browser.rs 中监听滚动
.on_scroll(|event, cx| {
    let scroll_position = event.position;
    let content_height = event.content_height;
    let viewport_height = event.viewport_height;
    
    // 接近底部时加载更多
    if scroll_position.y + viewport_height > content_height - 100. {
        // 触发加载更多
    }
})
```

3. 验证编译：`cargo build -p remby-gui`
4. Commit

---

### Task 5.3: 季节/集数浏览

**Covers:** 详情页增强

**Files:**
- Modify: `crates/remby-gui/src/views/series.rs`

**Steps:**
1. 在详情页添加季节选择
2. 点击季节显示集数列表
3. 集数卡片带进度条

```rust
// 季节选择
div()
    .mb_4()
    .child(
        h_flex()
            .gap_2()
            .children(seasons.iter().map(|season| {
                div()
                    .px_4()
                    .py_2()
                    .rounded(px(6.))
                    .when(selected_season == season.id, |this| this.bg(cx.theme().primary))
                    .child(season.name)
            }))
    )

// 集数列表
div()
    .gap_2()
    .children(episodes.iter().map(|episode| {
        h_flex()
            .gap_4()
            .p_4()
            .rounded(px(8.))
            .bg(cx.theme().border.opacity(0.1))
            .child(thumbnail)
            .child(info)
            .child(progress_bar)
    }))
```

4. 验证编译：`cargo build -p remby-gui`
5. Commit

---

## 实施顺序

| 阶段 | 任务 | 描述 | 依赖 |
|------|------|------|------|
| P1 | 1.1 | Header 图标功能化 | 无 |
| P1 | 1.2 | 标签栏功能化 | 无 |
| P1 | 1.3 | 卡片点击修复 | 无 |
| P2 | 2.1 | 网格自适应 | 无 |
| P2 | 2.2 | 详情页响应式 | 无 |
| P3 | 3.1 | 悬停效果 | 无 |
| P3 | 3.2 | 骨架屏加载 | 无 |
| P3 | 3.3 | 横滚行渐变 | 无 |
| P4 | 4.1 | 详情页数据修复 | 无 |
| P4 | 4.2 | 评分显示修复 | 无 |
| P4 | 4.3 | 继续观看信息 | 无 |
| P5 | 5.1 | Hero Banner | 无 |
| P5 | 5.2 | 无限滚动 | 无 |
| P5 | 5.3 | 季节/集数浏览 | 无 |
