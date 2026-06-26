# 首页完善计划 — 匹配 Emby Theater

## 当前问题

| 问题 | 描述 |
|------|------|
| 缺少"我的媒体" | LibraryCard 组件存在但未在首页使用 |
| 页面不可滚动 | 内容溢出被裁剪 |
| "查看全部"无功能 | 纯装饰，无点击处理 |
| 卡片缺评分+年份 | 最新电影卡片只显示标题 |
| Hero Banner 多余 | Emby Theater 没有 Hero Banner |

## 修复方案

### Task 1: 添加"我的媒体"部分

**修改文件:** `crates/remby-gui/src/views/home.rs`

在首页最顶部添加"我的媒体"部分，使用 LibraryCard 组件显示所有媒体库。

```rust
// 我的媒体
let my_media_section = if !app.state.libraries.is_empty() {
    Some(
        div()
            .mb_8()
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .mb_4()
                    .child(div().text_lg().font_bold().child("我的媒体"))
            )
            .child(
                h_flex()
                    .gap_4()
                    .flex_wrap()
                    .children(app.state.libraries.iter().map(|lib| {
                        let posters = get_library_posters(lib, &app.state.poster_cache);
                        LibraryCard::new(&lib.name)
                            .posters(posters)
                            .on_click({
                                let this = this.clone();
                                let lib_id = lib.id.clone();
                                let lib_name = lib.name.clone();
                                move |_, cx| {
                                    this.update(cx, |app, cx| {
                                        app.state.browser_library_id = lib_id.clone();
                                        app.state.browser_library_name = lib_name.clone();
                                        app.state.navigate(View::LibraryBrowser);
                                        app.load_browser_data(cx);
                                    });
                                }
                            })
                    }))
            )
    )
} else {
    None
};
```

### Task 2: 使页面可垂直滚动

**修改文件:** `crates/remby-gui/src/views/home.rs`

将最外层容器改为可滚动：

```rust
// 修改前
v_flex()
    .size_full()
    .p_6()
    .gap_6()
    .children(sections)

// 修改后
div()
    .id("home-scroll")
    .size_full()
    .overflow_y_scroll()
    .child(
        v_flex()
            .p_6()
            .gap_6()
            .children(sections)
    )
```

### Task 3: 最新电影卡片添加评分+年份

**修改文件:** `crates/remby-gui/src/views/home.rs`

在 latest_movies_row 中，MediaCard 的 subtitle 改为显示评分和年份：

```rust
// 修改前
.subtitle(item.display_name())

// 修改后
.subtitle(format!("{} · {}", 
    item.community_rating.map(|r| format!("★ {:.1}", r)).unwrap_or_default(),
    item.production_year.map(|y| y.to_string()).unwrap_or_default()
))
```

### Task 4: "查看全部" 添加点击处理

**修改文件:** `crates/remby-gui/src/views/home.rs`

为"查看全部 →"添加点击处理，导航到媒体库浏览器：

```rust
// 修改前
.child("查看全部 →")

// 修改后
.child(
    div()
        .id("view-all-latest")
        .text_sm()
        .text_color(cx.theme().primary)
        .cursor_pointer()
        .hover(|this| this.opacity(0.8))
        .child("查看全部 →")
        .on_click({
            let this = this.clone();
            move |_, cx| {
                this.update(cx, |app, cx| {
                    // 导航到媒体库浏览器
                    app.state.navigate(View::LibraryBrowser);
                    app.load_browser_data(cx);
                });
            }
        })
)
```

### Task 5: 移除 Hero Banner（可选）

**修改文件:** `crates/remby-gui/src/views/home.rs`

Emby Theater 首页没有 Hero Banner。可以移除它，或者保留但不显示在最顶部。

```rust
// 移除 Hero Banner，直接从"我的媒体"开始
// 或者将 Hero Banner 移到最后（作为推荐内容）
```

---

## 实施顺序

| 任务 | 描述 | 优先级 |
|------|------|--------|
| 1 | 添加"我的媒体"部分 | P0 |
| 2 | 页面可垂直滚动 | P0 |
| 3 | 最新电影卡片评分+年份 | P1 |
| 4 | "查看全部"点击处理 | P1 |
| 5 | 移除/调整 Hero Banner | P2 |
