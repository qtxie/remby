use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

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
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_or_fetch<F, Fut, T>(&self, key: &str, ttl: Duration, fetcher: F) -> Option<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Option<T>>,
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(key) {
                if entry.inserted.elapsed() < entry.ttl {
                    return serde_json::from_slice(&entry.data).ok();
                }
            }
        }

        let data = fetcher().await?;

        {
            let mut cache = self.cache.write().await;
            cache.insert(
                key.to_string(),
                CacheEntry {
                    data: serde_json::to_vec(&data).ok()?,
                    inserted: Instant::now(),
                    ttl,
                },
            );
        }

        Some(data)
    }

    pub async fn invalidate(&self, key: &str) {
        self.cache.write().await.remove(key);
    }

    #[allow(dead_code)]
    pub async fn clear(&self) {
        self.cache.write().await.clear();
    }
}
