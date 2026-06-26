use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use gpui::{Image, ImageFormat};
use tokio::sync::RwLock;

pub struct ImageLoader {
    memory_cache: RwLock<HashMap<String, Arc<Image>>>,
    disk_cache_dir: PathBuf,
    client: reqwest::Client,
}

impl ImageLoader {
    pub fn new(client: reqwest::Client) -> Self {
        let disk_cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("remby")
            .join("images");

        let _ = std::fs::create_dir_all(&disk_cache_dir);

        Self {
            memory_cache: RwLock::new(HashMap::new()),
            disk_cache_dir,
            client,
        }
    }

    async fn load_from_disk(&self, disk_path: &std::path::Path) -> Option<Arc<Image>> {
        if !disk_path.exists() {
            return None;
        }
        let bytes = tokio::fs::read(disk_path).await.ok()?;
        if bytes.is_empty() {
            return None;
        }
        let ext = disk_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("jpg");
        let format = if ext == "png" {
            ImageFormat::Png
        } else {
            ImageFormat::Jpeg
        };
        Some(Arc::new(Image::from_bytes(format, bytes)))
    }

    pub async fn load_poster(
        &self,
        server: &str,
        token: &str,
        item_id: &str,
    ) -> Option<Arc<Image>> {
        if let Some(cached) = self.memory_cache.read().await.get(item_id) {
            return Some(cached.clone());
        }

        let disk_path = self.disk_cache_dir.join(format!("{}.jpg", item_id));
        if let Some(image) = self.load_from_disk(&disk_path).await {
            self.memory_cache
                .write()
                .await
                .insert(item_id.to_string(), image.clone());
            return Some(image);
        }

        let url = format!(
            "{}/Items/{}/Images/Primary?maxWidth=300&quality=90",
            server, item_id
        );

        let response = self
            .client
            .get(&url)
            .header("X-Emby-Token", token)
            .send()
            .await
            .ok()?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/jpeg")
            .to_string();

        let bytes = response.bytes().await.ok()?.to_vec();

        if bytes.is_empty() {
            return None;
        }

        let format = if content_type.contains("png") {
            ImageFormat::Png
        } else {
            ImageFormat::Jpeg
        };

        let disk_path = if content_type.contains("png") {
            self.disk_cache_dir.join(format!("{}.png", item_id))
        } else {
            disk_path
        };
        let _ = tokio::fs::write(&disk_path, &bytes).await;

        let image = Arc::new(Image::from_bytes(format, bytes));
        self.memory_cache
            .write()
            .await
            .insert(item_id.to_string(), image.clone());

        Some(image)
    }

    pub async fn load_backdrop(
        &self,
        server: &str,
        token: &str,
        item_id: &str,
    ) -> Option<Arc<Image>> {
        let key = format!("backdrop:{}", item_id);
        if let Some(cached) = self.memory_cache.read().await.get(&key) {
            return Some(cached.clone());
        }

        let disk_path = self.disk_cache_dir.join(format!("{}_backdrop.jpg", item_id));
        if let Some(image) = self.load_from_disk(&disk_path).await {
            self.memory_cache
                .write()
                .await
                .insert(key, image.clone());
            return Some(image);
        }

        let url = format!(
            "{}/Items/{}/Images/Backdrop?maxWidth=600&quality=80",
            server, item_id
        );

        let response = self
            .client
            .get(&url)
            .header("X-Emby-Token", token)
            .send()
            .await
            .ok()?;

        let status = response.status();
        if !status.is_success() {
            return None;
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/jpeg")
            .to_string();

        let bytes = response.bytes().await.ok()?.to_vec();

        if bytes.is_empty() {
            return None;
        }

        let format = if content_type.contains("png") {
            ImageFormat::Png
        } else {
            ImageFormat::Jpeg
        };

        let disk_path = if content_type.contains("png") {
            self.disk_cache_dir.join(format!("{}_backdrop.png", item_id))
        } else {
            disk_path
        };
        let _ = tokio::fs::write(&disk_path, &bytes).await;

        let image = Arc::new(Image::from_bytes(format, bytes));
        self.memory_cache.write().await.insert(key, image.clone());

        Some(image)
    }

    pub async fn get_cached(&self, item_id: &str) -> Option<Arc<Image>> {
        self.memory_cache.read().await.get(item_id).cloned()
    }
}
