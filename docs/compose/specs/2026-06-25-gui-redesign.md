# Remby GUI 重新设计规格

## [S1] 问题陈述

当前 GUI 存在以下问题：
- 已构建的组件（SidebarNav、SearchBar、Toast、ImageLoader）从未被使用
- `app.rs` 是 1190 行的巨型对象，混合业务逻辑、数据加载和渲染
- 海报图片从未加载，所有卡片只显示占位图标
- 纯键盘导航，无鼠标交互支持
- 主题系统不完整，背景硬编码
- 异步加载模式重复 15+ 次

## [S2] 布局设计 — 三栏式结构

### 整体布局
```
┌──────────────────────────────────────────────────┐
│  Header (48px)                                   │
│  [Logo] [搜索栏]                    [用户] [设置]  │
├──────────┬───────────────────────────────────────┤
│ 侧边栏    │  内容区域                              │
│ (200px)  │  (自适应)                              │
│          │                                       │
│ 🏠 首页   │  卡片网格 / 详情页 / 播放器 / 设置       │
│ 📚 媒体库 │                                       │
│ ⭐ 收藏   │                                       │
│ ⚙ 设置   │                                       │
│          │                                       │
├──────────┴───────────────────────────────────────┤
│  Toast 通知 (底部浮动)                             │
└──────────────────────────────────────────────────┘
```

### 侧边栏
- 固定宽度 200px，深色背景
- 4 个导航项：首页、媒体库、收藏、设置
- 当前页面高亮（accent 色背景 + 白色文字）
- 鼠标悬停效果
- 点击切换页面

### Header
- 高度 48px
- 左侧：Remby logo + 版本号
- 中间：搜索栏（全局搜索，支持实时搜索）
- 右侧：当前用户名 + 设置按钮

### 内容区域
- 自适应窗口大小
- 根据当前页面渲染不同内容

## [S3] 海报图片系统

### ImageLoader 集成
- `ImageLoader` 注入 `RembyApp` 作为共享状态
- `MediaCard` 接收 `poster_url` 参数
- 异步加载：显示骨架屏 → 加载完成显示海报
- 加载失败：显示占位图标
- 内存缓存：`RwLock<HashMap<String, Arc<Image>>>`

### 海报 URL 构建
```
{server}/Items/{item_id}/Images/Primary?maxWidth=300&quality=90
Header: X-Emby-Token: {token}
```

## [S4] 卡片设计升级

### MediaCard 组件重构
- 海报比例 2:3（标准电影海报）
- 自适应宽度（最小 120px，最大 200px）
- 圆角 8px
- 悬停效果：放大 1.05x + 阴影加深 + 播放按钮覆盖层

### 卡片信息区
```
┌──────────────┐
│  ┌────────┐  │
│  │        │  │
│  │  海报   │  │
│  │        │  │
│  │        │  │
│  └────────┘  │
│  进度条 █████ │  (Continue Watching)
│  标题        │
│  年份 · 评分  │
│  时长        │
└──────────────┘
```

### 徽章系统
- 左上角：类型徽章（电影/剧集/集数）
- 右上角：收藏/关注状态
- 底部：进度条（Continue Watching）

## [S5] 主题系统完善

### 颜色槽位扩展
从 7 个扩展到 12 个：
- `background` — 主背景色
- `surface` — 卡片/面板背景
- `surface_variant` — 次要表面色
- `border` — 边框颜色
- `shadow` — 阴影颜色
- `accent` — 主题色（已有）
- `text` — 主文字（已有）
- `muted` — 次要文字（已有）
- `warning` — 警告色（已有）
- `success` — 成功色（已有）
- `error` — 错误色（已有）
- `selection_fg` — 选中前景色（已有）

### 内置主题
- Default（深蓝主题）
- Dark（纯黑主题）
- Dracula（紫色主题）
- 自定义主题（通过 theme.json）

## [S6] 交互体验

### 鼠标支持
- 侧边栏：点击导航
- 卡片：点击查看详情，悬停高亮
- 按钮：点击触发动作
- 搜索栏：点击聚焦，输入触发搜索
- 滚轮：内容区域滚动

### 键盘快捷键（保留）
- `j/k` — 上下导航
- `Enter` — 确认/进入
- `Escape` — 返回/取消
- `/` — 聚焦搜索栏
- `1-4` — 快速切换侧边栏页面
- `q` — 退出

### 动画
- 页面切换：淡入淡出（200ms）
- 卡片悬停：缩放 + 阴影（150ms）
- 加载状态：骨架屏闪烁
- Toast：从底部滑入，3 秒后滑出

## [S7] 架构清理

### app.rs 拆分
```
src/
├── main.rs          — 应用入口
├── app.rs           — 主渲染逻辑（精简到 ~300 行）
├── state.rs         — 状态管理（已有，扩展）
├── actions.rs       — 所有 Action 定义和处理
├── loaders.rs       — 通用异步加载函数
├── theme_adapter.rs — 主题适配（已有）
├── image_loader.rs  — 图片加载（已有，激活）
└── views/
    ├── mod.rs
    ├── login.rs
    ├── home.rs
    ├── libraries.rs
    ├── browser.rs
    ├── player.rs
    ├── settings.rs
    ├── series.rs
    ├── favorites.rs
    └── components/
        ├── mod.rs
        ├── media_card.rs  — 升级
        ├── sidebar.rs     — 激活
        ├── search_bar.rs  — 激活
        ├── toast.rs       — 激活
        ├── loading.rs     — 已有
        ├── badge.rs       — 新增
        └── progress.rs    — 新增
```

### 异步加载统一模式
```rust
macro_rules! spawn_load {
    ($app:expr, $field:expr, $future:expr) => {
        let weak = $app.downgrade();
        $app.cx.spawn(|_, cx| async move {
            let result = $future.await;
            if let Some(app) = weak.upgrade() {
                app.update(cx, |app, cx| {
                    $field = result;
                    cx.notify();
                });
            }
        })
    };
}
```

## [S8] 实现阶段

### P0: 激活已有组件（1-2 天）
- 连接 SidebarNav 到主渲染
- 连接 SearchBar 到 Header
- 连接 Toast 到通知系统
- 连接 ImageLoader 到 MediaCard

### P1: 海报图片 + 卡片升级（2-3 天）
- MediaCard 组件重构
- 海报异步加载 + 缓存
- 骨架屏加载状态
- 进度条、徽章、评分显示

### P2: 布局重构（2-3 天）
- 三栏式布局实现
- Header 组件
- 侧边栏交互
- 内容区域自适应

### P3: 主题系统完善（1-2 天）
- 扩展颜色槽位
- 所有组件使用主题 token
- 3 个内置主题
- 自定义主题支持

### P4: 鼠标交互 + 动画（1-2 天）
- 鼠标点击导航
- 悬停效果
- 页面过渡动画
- Toast 动画

### P5: 架构清理（1-2 天）
- app.rs 拆分
- 异步模式统一
- 代码清理

## [S9] 验收标准

1. 所有 18 个 TUI 视图在 GUI 中有对应实现
2. 海报图片正常加载和显示
3. 鼠标和键盘均可导航
4. 主题切换正常工作
5. 无硬编码颜色
6. 无死代码
7. 所有组件被使用
8. 异步加载无重复模式
