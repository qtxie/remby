# Emby Theater 电影页面复刻实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use compose:subagent (recommended) or compose:execute to implement this plan task-by-task.

**Goal:** 复刻 Emby Theater 的电影库浏览器和电影详情页。

**Architecture:** 重构现有的 browser.rs 和 series.rs 视图，添加新的子组件。

**Tech Stack:** Rust, gpui, gpui-component, remby-core

---

### Task 1: 电影库浏览器 — 标签栏 + 信息栏

**Covers:** 电影库浏览器顶部

**Files:**
- Modify: `crates/remby-gui/src/views/browser.rs` — 添加标签栏和信息栏

**Steps:**
1. 在 `BrowserView` 的 `render` 中添加标签栏：
```rust
// 标签栏
let tabs = vec!["电影", "最近", "合集", "类型风格", "喜欢", "文件夹"];
h_flex()
    .justify_center()
    .gap_2()
    .mb_4()
    .children(tabs.iter().enumerate().map(|(i, tab)| {
        let is_active = i == 0; // 当前选中
        div()
            .px_5()
            .py_2()
            .rounded_full()
            .text_sm()
            .when(is_active, |this| this.bg(cx.theme().primary).text_color(cx.theme().background))
            .when(!is_active, |this| this.text_color(cx.theme().muted_foreground).hover(|this| this.bg(cx.theme().border)))
            .child(*tab)
    }))
```

2. 添加信息栏：
```rust
// 信息栏
let total = app.state.browser_items.len();
h_flex()
    .items_center()
    .gap_4()
    .mb_6()
    .text_sm()
    .text_color(cx.theme().muted_foreground)
    .child(format!("共 {} 个项目", total))
    .child(
        h_flex().items_center().gap_1()
            .child(Icon::new(IconName::Play).small())
            .child("全部播放")
    )
    .child(
        h_flex().items_center().gap_1()
            .child(Icon::new(IconName::Shuffle).small())
            .child("随机播放")
    )
    .child(
        h_flex().items_center().gap_1()
            .child(Icon::new(IconName::Sort).small())
            .child("排序方式: 标题名称")
    )
    .child(
        h_flex().items_center().gap_1()
            .child(Icon::new(IconName::Filter).small())
            .child("筛选")
    )
```

3. 验证编译：`cargo build -p remby-gui`
4. Commit

---

### Task 2: 电影库浏览器 — 竖版海报网格

**Covers:** 电影库浏览器网格

**Files:**
- Modify: `crates/remby-gui/src/views/browser.rs` — 重构网格布局

**Steps:**
1. 将网格从 4 列改为 7 列
2. 每个卡片显示：海报 + 标题 + 评分 + 年份
3. 评分用黄色星标显示

```rust
// 电影海报卡片
fn movie_card(item: &MediaItem, poster: Option<Arc<Image>>, is_selected: bool) -> impl IntoElement {
    let rating = item.user_data.as_ref()
        .and_then(|u| u.rating)
        .unwrap_or(0.0);
    let year = item.production_year.unwrap_or(0);
    
    v_flex()
        .w(px(160.))
        .gap_2()
        .child(
            div()
                .h(px(220.))
                .rounded(px(6.))
                .overflow_hidden()
                .border_2()
                .when(is_selected, |this| this.border_color(cx.theme().primary))
                .when(!is_selected, |this| this.border_color(cx.theme().border))
                .child(match poster {
                    Some(img) => img(img).w_full().h_full().object_fit(gpui::ObjectFit::Cover).into_any_element(),
                    None => div().w_full().h_full().bg(cx.theme().border.opacity(0.3)).into_any_element(),
                })
        )
        .child(
            v_flex()
                .child(
                    div()
                        .text_sm()
                        .font_medium()
                        .overflow_x_hidden()
                        .whitespace_nowrap()
                        .child(item.display_name())
                )
                .child(
                    h_flex()
                        .items_center()
                        .gap_1()
                        .child(Icon::new(IconName::Star).small().text_color(hsl(45., 0.8, 0.5)))
                        .child(format!("{:.1}", rating))
                        .child(div().flex_1())
                        .child(format!("{}", year))
                )
        )
}
```

4. 验证编译：`cargo build -p remby-gui`
5. Commit

---

### Task 3: 电影详情页 — 模糊背景 + 布局

**Covers:** 电影详情页顶部

**Files:**
- Modify: `crates/remby-gui/src/views/series.rs` — 重构为电影详情页

**Steps:**
1. 重构 `SeriesView` 为 `MovieDetailView`
2. 添加模糊背景效果
3. 布局：左侧海报 + 右侧信息 + 右上角背景图

```rust
impl RenderOnce for MovieDetailView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let item = self.item.clone();
        let backdrop = self.backdrop.clone();
        
        div()
            .id("movie-detail")
            .size_full()
            .relative()
            .overflow_hidden()
            // 模糊背景
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .opacity(0.3)
                    .child(match backdrop {
                        Some(img) => img(img).w_full().h_full().object_fit(gpui::ObjectFit::Cover).into_any_element(),
                        None => div().w_full().h_full().bg(cx.theme().border).into_any_element(),
                    })
            )
            // 内容
            .child(
                div()
                    .relative()
                    .p_8()
                    .child(
                        h_flex()
                            .gap_8()
                            // 左侧海报
                            .child(
                                div()
                                    .w(px(220.))
                                    .h(px(330.))
                                    .rounded(px(8.))
                                    .overflow_hidden()
                                    .child(match self.poster {
                                        Some(img) => img(img).w_full().h_full().object_fit(gpui::ObjectFit::Cover).into_any_element(),
                                        None => div().w_full().h_full().bg(cx.theme().border).into_any_element(),
                                    })
                            )
                            // 右侧信息
                            .child(
                                v_flex()
                                    .flex_1()
                                    .gap_4()
                                    .child(/* 标题 */)
                                    .child(/* 评分 + 年份 + 时长 */)
                                    .child(/* 类型标签 */)
                                    .child(/* 视频/音频/字幕 */)
                                    .child(/* 操作按钮 */)
                                    .child(/* 简介 */)
                            )
                            // 右上角背景图
                            .child(
                                div()
                                    .w(px(200.))
                                    .h(px(300.))
                                    .rounded(px(8.))
                                    .overflow_hidden()
                                    .opacity(0.8)
                                    .child(/* backdrop image */)
                            )
                    )
            )
    }
}
```

4. 验证编译：`cargo build -p remby-gui`
5. Commit

---

### Task 4: 电影详情页 — 信息区

**Covers:** 电影详情页信息

**Files:**
- Modify: `crates/remby-gui/src/views/series.rs`

**Steps:**
1. 添加标题、评分、年份、时长、分级
2. 添加类型标签（冒险、动作、惊悚）
3. 添加视频/音频/字幕信息
4. 添加操作按钮（播放、已播放、收藏）

```rust
// 标题
div().text_2xl().font_bold().child(item.display_name()),

// 评分 + 年份 + 时长 + 分级
h_flex()
    .items_center()
    .gap_2()
    .text_sm()
    .child(
        h_flex().items_center().gap_1()
            .child(Icon::new(IconName::Star).text_color(hsl(45., 0.8, 0.5)))
            .child(format!("{}", rating))
    )
    .child(format!("{}", year))
    .child(format!("{}分钟", runtime)),
    
// 类型标签
h_flex()
    .gap_2()
    .children(genres.iter().map(|g| {
        div()
            .px_3()
            .py_1()
            .rounded(px(4.))
            .border_1()
            .border_color(cx.theme().border)
            .text_xs()
            .child(g)
    })),

// 操作按钮
h_flex()
    .gap_3()
    .child(
        h_flex().items_center().gap_2().px_5().py_2().rounded(px(6.))
            .bg(cx.theme().primary).text_color(cx.theme().background)
            .child(Icon::new(IconName::Play).small())
            .child("播放")
    )
    .child(
        h_flex().items_center().gap_2().px_5().py_2().rounded(px(6.))
            .bg(cx.theme().border.opacity(0.2))
            .child(Icon::new(IconName::Check).small())
            .child("已播放")
    )
    .child(
        h_flex().items_center().px_3().py_2().rounded(px(6.))
            .bg(cx.theme().border.opacity(0.2))
            .child(Icon::new(IconName::Heart).small())
    )
```

5. 验证编译：`cargo build -p remby-gui`
6. Commit

---

### Task 5: 电影详情页 — 演职人员

**Covers:** 电影详情页演职人员

**Files:**
- Modify: `crates/remby-gui/src/views/series.rs`

**Steps:**
1. 添加"演职人员"标题
2. 添加横向滚动的演职人员卡片

```rust
// 演职人员
div()
    .mt_8()
    .child(
        div().text_lg().font_bold().mb_4().child("演职人员")
    )
    .child(
        h_flex()
            .gap_4()
            .overflow_x_scroll()
            .children(people.iter().map(|person| {
                v_flex()
                    .w(px(100.))
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w(px(80.))
                            .h(px(100.))
                            .rounded(px(6.))
                            .overflow_hidden()
                            .bg(cx.theme().border.opacity(0.3))
                            .child(/* person image */)
                    )
                    .child(
                        div().text_xs().text_center().child(&person.name)
                    )
                    .child(
                        div().text_xs().text_color(cx.theme().muted_foreground).text_center()
                            .child(format!("饰演: {}", person.role))
                    )
            }))
    )
```

3. 验证编译：`cargo build -p remby-gui`
4. Commit

---

### Task 6: 最终验证

**Steps:**
1. 运行 `cargo build -p remby-gui`
2. 运行 `cargo clippy -p remby-gui`
3. 手动测试两个页面
4. Commit
