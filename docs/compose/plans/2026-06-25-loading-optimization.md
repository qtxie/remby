# 内容加载速度优化计划

## 问题诊断

### 当前加载流程（首页冷启动）

```
load_home_data:
  1. get_resume_items(20)      → 200ms  ← 串行
  2. get_latest_items(20)      → 200ms  ← 串行
  3. get_latest_items(20)      → 200ms  ← 串行（重复！）
  4. load_poster() × 60        → 60 × 100ms = 6000ms  ← 串行！
  总计: ~6.6秒

load_libraries_data:
  1. get_libraries()           → 200ms  ← 串行
  2. get_latest_for_library() × N → N × 200ms  ← 串行！
  3. load_poster() × N×10      → N×10 × 100ms  ← 串行！
  总计: ~3-5秒

load_browser_data:
  1. get_items_filtered()      → 200ms  ← 串行
  2. get_genres/tags/studios   → 200ms  ← 并行
  3. load_poster() × 50        → 50 × 100ms = 5000ms  ← 串行！
  总计: ~5.4秒
```

### Emby Theater 的做法

- 所有 API 调用并行执行
- 图片批量加载（8-10 个并发）
- API 响应缓存（内存 + 磁盘）
- 图片磁盘缓存
- HTTP/2 多路复用
- 请求取消（导航离开时取消旧请求）

---

## 优化方案

### Phase 1: 并行化 API 调用（最高优先级）

#### 1.1 load_home_data 并行化

```rust
// 修改前：串行
let cw = client.get_resume_items(20).await.unwrap_or_default();
let latest = client.get_latest_items(20).await.unwrap_or_default();
let following = client.get_latest_items(20).await.unwrap_or_default()
    .into_iter().filter(...).collect();

// 修改后：并行 + 去重
let (cw, latest) = tokio::join!(
    client.get_resume_items(20),
    client.get_latest_items(20),
);
let cw = cw.unwrap_or_default();
let latest = latest.unwrap_or_default();
let following: Vec<_> = latest.iter()
    .filter(|i| i.series_id.is_some())
    .cloned()
    .collect();
```

#### 1.2 load_libraries_data 并行化

```rust
// 修改前：N+1 串行
let libraries = client.get_libraries().await.unwrap_or_default();
for lib in &libraries {
    let items = client.get_latest_for_library(&lib.id, 10).await.unwrap_or_default();
    all_latest.extend(items);
}

// 修改后：并行
let libraries = client.get_libraries().await.unwrap_or_default();
let futures: Vec<_> = libraries.iter()
    .map(|lib| client.get_latest_for_library(&lib.id, 10))
    .collect();
let results = futures::future::join_all(futures).await;
let all_latest: Vec<_> = results.into_iter()
    .flatten()
    .flatten()
    .collect();
```

#### 1.3 load_series_info 并行化

```rust
// 修改前：串行
let item = client.get_item_detail(&sid).await.ok();
let seasons = client.get_seasons(&sid).await.unwrap_or_default();
let similar = client.get_similar(&sid).await.unwrap_or_default();

// 修改后：并行
let (item, seasons, similar) = tokio::join!(
    client.get_item_detail(&sid),
    client.get_seasons(&sid),
    client.get_similar(&sid),
);
```

#### 1.4 load_browser_data 完全并行化

```rust
// 修改前：items 串行 + filters 并行
let page = client.get_items_filtered(...).await.ok();
let (genres, tags, studios) = tokio::join!(...);

// 修改后：全部并行
let (page, genres, tags, studios) = tokio::join!(
    client.get_items_filtered(...),
    client.get_genres(&library_id),
    client.get_tags(&library_id),
    client.get_studios(&library_id),
);
```

### Phase 2: 并行化图片加载

#### 2.1 批量并发加载图片

```rust
// 修改前：串行 for 循环
for item_id in item_ids {
    if let Some(image) = image_loader.load_poster(&server, &token, &item_id).await {
        // ...
    }
}

// 修改后：并发加载，限制并发数
use futures::stream::{self, StreamExt};

let results: Vec<_> = stream::iter(item_ids)
    .map(|item_id| {
        let loader = image_loader.clone();
        let server = server.clone();
        let token = token.clone();
        async move {
            let image = loader.load_poster(&server, &token, &item_id).await;
            (item_id, image)
        }
    })
    .buffer_unordered(8)  // 最多 8 个并发
    .collect()
    .await;
```

#### 2.2 共享 reqwest::Client

```rust
// 修改前：各自创建 Client
// EmbyClient: reqwest::Client::new()
// ImageLoader: reqwest::Client::new()

// 修改后：共享同一个 Client
pub struct ImageLoader {
    cache: RwLock<HashMap<String, Arc<Image>>>,
    client: reqwest::Client,  // 从 EmbyClient 传入
}

impl ImageLoader {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            client,
        }
    }
}
```

### Phase 3: API 响应缓存

#### 3.1 内存缓存层

```rust
use std::time::{Duration, Instant};

pub struct ApiCache {
    cache: RwLock<HashMap<String, CacheEntry>>,
}

struct CacheEntry {
    data: Vec<u8>,
    inserted: Instant,
    ttl: Duration,
}

impl ApiCache {
    pub fn new() -> Self {
        Self { cache: RwLock::new(HashMap::new()) }
    }

    pub async fn get_or_fetch<F, Fut, T>(&self, key: &str, ttl: Duration, fetcher: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
        T: Clone + serde::Serialize + serde::de::DeserializeOwned,
    {
        // 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(key) {
                if entry.inserted.elapsed() < entry.ttl {
                    return Ok(serde_json::from_slice(&entry.data)?);
                }
            }
        }
        
        // 缓存未命中，获取数据
        let data = fetcher().await?;
        
        // 写入缓存
        {
            let mut cache = self.cache.write().await;
            cache.insert(key.to_string(), CacheEntry {
                data: serde_json::to_vec(&data)?,
                inserted: Instant::now(),
                ttl,
            });
        }
        
        Ok(data)
    }
}
```

#### 3.2 在数据加载中使用缓存

```rust
// 修改前：直接调用 API
let cw = client.get_resume_items(20).await.unwrap_or_default();

// 修改后：使用缓存
let cw = cache.get_or_fetch("resume_items", Duration::from_secs(60), || {
    client.get_resume_items(20)
}).await.unwrap_or_default();
```

### Phase 4: 图片磁盘缓存

#### 4.1 添加磁盘缓存

```rust
pub struct ImageLoader {
    memory_cache: RwLock<HashMap<String, Arc<Image>>>,
    disk_cache_dir: PathBuf,
    client: reqwest::Client,
}

impl ImageLoader {
    pub async fn load_poster(&self, server: &str, token: &str, item_id: &str) -> Option<Arc<Image>> {
        // 1. 检查内存缓存
        if let Some(cached) = self.memory_cache.read().await.get(item_id) {
            return Some(cached.clone());
        }
        
        // 2. 检查磁盘缓存
        let disk_path = self.disk_cache_dir.join(format!("{}.jpg", item_id));
        if disk_path.exists() {
            let bytes = tokio::fs::read(&disk_path).await.ok()?;
            let image = Arc::new(Image::from_bytes(ImageFormat::Jpeg, bytes));
            self.memory_cache.write().await.insert(item_id.to_string(), image.clone());
            return Some(image);
        }
        
        // 3. 从网络加载
        let url = format!("{}/Items/{}/Images/Primary?maxWidth=300&quality=90", server, item_id);
        let response = self.client.get(&url)
            .header("X-Emby-Token", token)
            .send().await.ok()?;
        let bytes = response.bytes().await.ok()?.to_vec();
        
        // 4. 写入磁盘缓存
        let _ = tokio::fs::write(&disk_path, &bytes).await;
        
        // 5. 写入内存缓存
        let image = Arc::new(Image::from_bytes(ImageFormat::Jpeg, bytes));
        self.memory_cache.write().await.insert(item_id.to_string(), image.clone());
        
        Some(image)
    }
}
```

### Phase 5: 请求取消

#### 5.1 导航时取消旧请求

```rust
pub struct GuiState {
    // ... existing fields ...
    pub load_token: CancellationToken,
}

impl GuiState {
    pub fn navigate(&mut self, view: View) {
        // 取消之前的请求
        self.load_token.cancel();
        self.load_token = CancellationToken::new();
        
        self.view_stack.push(self.view.clone());
        self.view = view;
    }
}

// 在数据加载中使用
pub fn load_home_data(&mut self, cx: &mut Context<Self>) {
    let token = self.state.load_token.clone();
    
    crate::loaders::spawn_async(&cx.entity(), cx, async move {
        // 检查是否被取消
        if token.is_cancelled() { return None; }
        
        let (cw, latest) = tokio::join!(...);
        
        // 再次检查
        if token.is_cancelled() { return None; }
        
        Some((cw, latest))
    }, |app, cx, result| {
        if let Some((cw, latest)) = result {
            // 更新状态
        }
    });
}
```

---

## 预期效果

| 优化项 | 修改前 | 修改后 | 提升 |
|--------|--------|--------|------|
| 首页 API 调用 | 3 × 200ms = 600ms | 1 × 200ms = 200ms | 3x |
| 首页图片加载 | 60 × 100ms = 6s | 60/8 × 100ms = 750ms | 8x |
| 首页总加载 | ~6.6s | ~1s | 6x |
| 媒体库 API 调用 | N × 200ms | 1 × 200ms | Nx |
| 媒体库图片加载 | N×10 × 100ms | N×10/8 × 100ms | 8x |
| 详情页 API 调用 | 3 × 200ms = 600ms | 1 × 200ms = 200ms | 3x |
| 重复访问 | 全部重新加载 | 缓存命中 | 10x+ |

---

## 实施顺序

| 任务 | 描述 | 优先级 | 预计工作量 |
|------|------|--------|-----------|
| 1 | load_home_data 并行化 | P0 | 15min |
| 2 | load_libraries_data 并行化 | P0 | 15min |
| 3 | load_series_info 并行化 | P0 | 15min |
| 4 | load_browser_data 完全并行化 | P0 | 15min |
| 5 | 图片批量并发加载 | P0 | 30min |
| 6 | 共享 reqwest::Client | P1 | 15min |
| 7 | API 响应缓存 | P1 | 1h |
| 8 | 图片磁盘缓存 | P2 | 1h |
| 9 | 请求取消 | P2 | 30min |
