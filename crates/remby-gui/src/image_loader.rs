use std::collections::HashMap;
use std::sync::Arc;

use gpui::{Image, ImageFormat};
use tokio::sync::RwLock;

pub struct ImageLoader {
    cache: RwLock<HashMap<String, Arc<Image>>>,
    client: reqwest::Client,
}

impl ImageLoader {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            client: reqwest::Client::new(),
        }
    }

    pub async fn load_poster(
        &self,
        server: &str,
        token: &str,
        item_id: &str,
    ) -> Option<Arc<Image>> {
        if let Some(cached) = self.cache.read().await.get(item_id) {
            return Some(cached.clone());
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

        let image = Arc::new(Image::from_bytes(format, bytes));

        self.cache
            .write()
            .await
            .insert(item_id.to_string(), image.clone());

        Some(image)
    }

    pub async fn get_cached(&self, item_id: &str) -> Option<Arc<Image>> {
        self.cache.read().await.get(item_id).cloned()
    }
}
