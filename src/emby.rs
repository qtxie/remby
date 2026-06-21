use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use url::Url;

const CLIENT_NAME: &str = "Emby";
const CLIENT_VERSION: &str = "4.8.0.80";
const DEVICE_NAME: &str = "remby";
const DEVICE_ID: &str = "remby-tui-cli";

#[derive(Debug, Clone)]
pub struct EmbyClient {
    base_url: String,
    token: String,
    user_id: String,
    http: Client,
}

fn auth_header(token: &str) -> String {
    format!(
        "MediaBrowser Client=\"{}\", Device=\"{}\", DeviceId=\"{}\", Version=\"{}\", Token=\"{}\"",
        CLIENT_NAME, DEVICE_NAME, DEVICE_ID, CLIENT_VERSION, token
    )
}

fn base_headers(token: &str, _user_id: &str) -> Vec<(&'static str, String)> {
    vec![
        ("X-Emby-Authorization", auth_header(token)),
        ("X-Emby-Token", token.to_string()),
        ("X-Emby-Client", CLIENT_NAME.to_string()),
        ("X-Emby-DeviceName", DEVICE_NAME.to_string()),
        ("X-Emby-DeviceId", DEVICE_ID.to_string()),
        ("X-Emby-Version", CLIENT_VERSION.to_string()),
    ]
}

#[derive(Debug, Deserialize, Clone)]
pub struct MediaItem {
    #[serde(default, rename = "Id")]
    pub id: String,
    #[serde(default, rename = "Name")]
    pub name: String,
    #[serde(default, rename = "Type")]
    pub item_type: String,
    #[serde(default, rename = "MediaType")]
    pub media_type: Option<String>,
    #[serde(default, rename = "SeriesName")]
    pub series_name: Option<String>,
    #[serde(default, rename = "IndexNumber")]
    pub index_number: Option<i32>,
    #[serde(default, rename = "ParentIndexNumber")]
    pub parent_index_number: Option<i32>,
    #[serde(default, rename = "RunTimeTicks")]
    pub runtime_ticks: Option<i64>,
    #[serde(default, rename = "SeriesId")]
    pub series_id: Option<String>,
    #[serde(default, rename = "Overview")]
    pub overview: Option<String>,
    #[serde(default, rename = "ChildCount")]
    pub child_count: Option<i32>,
    #[serde(default, rename = "MediaSources")]
    pub media_sources: Vec<MediaSource>,
    #[serde(default, rename = "UserData")]
    pub user_data: Option<UserData>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct UserData {
    #[serde(default, rename = "PlaybackPositionTicks")]
    pub playback_position_ticks: Option<i64>,
    #[serde(default, rename = "IsFavorite")]
    pub is_favorite: bool,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct MediaSource {
    #[serde(default, rename = "Id")]
    pub id: String,
    #[serde(default, rename = "Container")]
    pub container: String,
    #[serde(default, rename = "Size")]
    pub size: u64,
    #[serde(default, rename = "RunTimeTicks")]
    pub runtime_ticks: Option<i64>,
    #[serde(default, rename = "MediaStreams")]
    pub media_streams: Vec<MediaStream>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct MediaStream {
    #[serde(default, rename = "Type")]
    pub stream_type: String,
    #[serde(default, rename = "Codec")]
    pub codec: String,
    #[serde(default, rename = "Language")]
    pub language: String,
    #[serde(default, rename = "Title")]
    pub title: Option<String>,
    #[serde(default, rename = "DisplayTitle")]
    pub display_title: Option<String>,
    #[serde(default, rename = "Width")]
    pub width: Option<i32>,
    #[serde(default, rename = "Height")]
    pub height: Option<i32>,
    #[serde(default, rename = "ChannelLayout")]
    pub channel_layout: Option<String>,
    #[serde(default, rename = "Profile")]
    pub profile: Option<String>,
    #[serde(default, rename = "VideoRange")]
    pub video_range: Option<String>,
    #[serde(default, rename = "AverageFrameRate")]
    pub avg_frame_rate: Option<f64>,
    #[serde(default, rename = "BitDepth")]
    pub bit_depth: Option<i32>,
}

impl MediaSource {
    pub fn display_label(&self) -> String {
        let container = if self.container.is_empty() { "?" } else { &self.container };
        let video = self.media_streams.iter().find(|s| s.stream_type == "Video");
        let audio = self.media_streams.iter().find(|s| s.stream_type == "Audio");

        let mut parts = Vec::new();

        // Resolution + video codec
        if let Some(v) = video {
            let res = v.height.map(|h| format!("{}p", h)).unwrap_or_default();
            let codec = v.codec.to_uppercase();
            let range = v.video_range.as_deref().unwrap_or("");
            let profile = v.profile.as_deref().unwrap_or("");
            let fps = v.avg_frame_rate.map(|f| format!("{}fps", f as i32)).unwrap_or_default();
            let mut vid_parts = Vec::new();
            if !res.is_empty() { vid_parts.push(res); }
            if !codec.is_empty() { vid_parts.push(codec); }
            if !range.is_empty() && range != "SDR" { vid_parts.push(range.to_uppercase()); }
            if !profile.is_empty() && profile != "Main" { vid_parts.push(profile.to_string()); }
            if !fps.is_empty() { vid_parts.push(fps); }
            if let Some(depth) = v.bit_depth {
                if depth > 8 { vid_parts.push(format!("{}bit", depth)); }
            }
            parts.push(vid_parts.join(" "));
        }

        // Audio info
        if let Some(a) = audio {
            let codec = a.codec.to_uppercase();
            let layout = a.channel_layout.clone().unwrap_or_default();
            let mut aud_parts = Vec::new();
            if !codec.is_empty() { aud_parts.push(codec); }
            if !layout.is_empty() { aud_parts.push(layout); }
            if !aud_parts.is_empty() {
                parts.push(aud_parts.join(" "));
            }
        }

        // Duration
        if let Some(ticks) = self.runtime_ticks {
            let secs = ticks / 10_000_000;
            let h = secs / 3600;
            let m = (secs % 3600) / 60;
            if h > 0 {
                parts.push(format!("{}h{:02}m", h, m));
            } else {
                parts.push(format!("{}m", m));
            }
        }

        // File size
        if self.size > 0 {
            if self.size > 1_073_741_824 {
                parts.push(format!("{:.1}GB", self.size as f64 / 1_073_741_824.0));
            } else if self.size > 1_048_576 {
                parts.push(format!("{:.0}MB", self.size as f64 / 1_048_576.0));
            }
        }

        // Container
        parts.push(format!("[{}]", container));

        parts.join(" | ")
    }
}

#[derive(Deserialize)]
pub struct ItemsResponse {
    #[serde(default, rename = "Items")]
    pub items: Vec<MediaItem>,
    #[serde(default, rename = "TotalRecordCount")]
    pub total: usize,
}

pub struct PageResult {
    pub items: Vec<MediaItem>,
    pub total: usize,
}

#[derive(Deserialize, Clone)]
pub struct Library {
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
    #[serde(rename = "CollectionType")]
    pub collection_type: Option<String>,
}

impl EmbyClient {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
            user_id: String::new(),
            http: Client::new(),
        }
    }

    pub async fn authenticate(base_url: &str, username: &str, password: &str) -> Result<Self> {
        let url = format!("{}/Users/AuthenticateByName", base_url.trim_end_matches('/'));
        let body = json!({
            "Username": username,
            "Pw": password,
        });

        let resp = Client::new()
            .post(&url)
            .header("X-Emby-Authorization", format!(
                "MediaBrowser Client=\"{}\", Device=\"{}\", DeviceId=\"{}\", Version=\"{}\"",
                CLIENT_NAME, DEVICE_NAME, DEVICE_ID, CLIENT_VERSION
            ))
            .header("X-Emby-Client", CLIENT_NAME)
            .header("X-Emby-DeviceName", DEVICE_NAME)
            .header("X-Emby-DeviceId", DEVICE_ID)
            .header("X-Emby-Version", CLIENT_VERSION)
            .json(&body)
            .send()
            .await
            .context("Failed to authenticate with Emby server")?;

        let data: serde_json::Value = resp.json().await.context("Invalid auth response")?;

        let token = data.get("AccessToken")
            .and_then(|v| v.as_str())
            .context("No access token in auth response")?
            .to_string();
        let user_id = data.get("User")
            .and_then(|u| u.get("Id"))
            .and_then(|v| v.as_str())
            .context("No user ID in auth response")?
            .to_string();

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
            user_id,
            http: Client::new(),
        })
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn authed_get(&self, url: &str) -> reqwest::RequestBuilder {
        let mut req = self.http.get(url);
        for (k, v) in base_headers(&self.token, &self.user_id) {
            req = req.header(k, v);
        }
        req
    }

    #[allow(dead_code)]
    pub async fn get_libraries(&self) -> Result<Vec<Library>> {
        let url = self.api_url("/Library/VirtualFolders");
        let resp = self.authed_get(&url)
            .send()
            .await
            .context("Failed to connect to Emby server")?;

        let body = resp.text().await?;
        let data: serde_json::Value = serde_json::from_str(&body)
            .context("Invalid response from Emby server")?;

        let items = if let Some(arr) = data.as_array() {
            arr.clone()
        } else {
            data.get("Items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default()
        };

        let mut libraries = Vec::new();
        for item in &items {
            let id = item.get("ItemId")
                .or_else(|| item.get("Id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let name = item.get("Name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !id.is_empty() && !name.is_empty() {
                libraries.push(Library {
                    id: id.to_string(),
                    name: name.to_string(),
                    collection_type: item
                        .get("CollectionType")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                });
            }
        }
        Ok(libraries)
    }

    pub async fn get_items(&self, parent_id: &str, start: usize, limit: usize) -> Result<PageResult> {
        let url = self.api_url(&format!("/Users/{}/Items", self.user_id));
        let resp = self.authed_get(&url)
            .query(&[
                ("ParentId", parent_id),
                ("Recursive", "false"),
                ("Fields", "Overview,MediaSources,ChildCount"),
                ("StartIndex", &start.to_string()),
                ("Limit", &limit.to_string()),
            ])
            .send()
            .await
            .context("Failed to fetch items")?;

        let data: ItemsResponse = resp.json().await.context("Invalid items response")?;
        Ok(PageResult {
            items: data.items,
            total: data.total,
        })
    }

    pub async fn get_resume_items(&self, limit: usize) -> Result<Vec<MediaItem>> {
        let url = self.api_url(&format!("/Users/{}/Items/Resume", self.user_id));
        let limit_str = limit.to_string();
        let resp = self.authed_get(&url)
            .query(&[
                ("Limit", limit_str.as_str()),
                ("Recursive", "true"),
                ("Fields", "Overview,MediaSources,ChildCount"),
                ("IncludeItemTypes", "Movie,Episode"),
            ])
            .send()
            .await
            .context("Failed to fetch resume items")?;

        let data: ItemsResponse = resp.json().await.context("Invalid resume response")?;
        Ok(data.items)
    }

    pub async fn get_latest_items(&self, limit: usize) -> Result<Vec<MediaItem>> {
        let url = self.api_url(&format!("/Users/{}/Items", self.user_id));
        let limit_str = limit.to_string();
        let resp = self.authed_get(&url)
            .query(&[
                ("Recursive", "true"),
                ("IncludeItemTypes", "Movie,Episode"),
                ("SortBy", "DateCreated"),
                ("SortOrder", "Descending"),
                ("Limit", limit_str.as_str()),
                ("Fields", "Overview,MediaSources,ChildCount"),
            ])
            .send()
            .await
            .context("Failed to fetch latest items")?;

        let data: ItemsResponse = resp.json().await.context("Invalid latest response")?;
        Ok(data.items)
    }

    pub async fn search(&self, query: &str) -> Result<Vec<MediaItem>> {
        let url = self.api_url(&format!("/Users/{}/Items", self.user_id));
        let resp = self.authed_get(&url)
            .query(&[
                ("SearchTerm", query),
                ("Recursive", "true"),
                ("IncludeItemTypes", "Movie,Series,Episode"),
                ("Limit", "20"),
            ])
            .send()
            .await
            .context("Search request failed")?;

        let data: ItemsResponse = resp.json().await.context("Invalid search response")?;
        Ok(data.items)
    }

    pub async fn get_item_detail(&self, item_id: &str) -> Result<MediaItem> {
        let url = self.api_url(&format!("/Users/{}/Items/{}", self.user_id, item_id));
        let resp = self.authed_get(&url)
            .query(&[("Fields", "MediaSources")])
            .send()
            .await
            .context("Failed to fetch item detail")?;
        let item: MediaItem = resp.json().await.context("Invalid item detail response")?;
        Ok(item)
    }

    pub async fn get_episodes(&self, series_id: &str) -> Result<(Vec<MediaItem>, usize)> {
        let url = self.api_url(&format!("/Shows/{}/Episodes", series_id));
        let resp = self.authed_get(&url)
            .query(&[
                ("UserId", self.user_id.as_str()),
                ("Fields", "Overview,MediaSources,ChildCount"),
                ("Recursive", "true"),
                ("Limit", "100"),
            ])
            .send()
            .await
            .context("Failed to fetch episodes")?;
        let data: ItemsResponse = resp.json().await.context("Invalid episodes response")?;
        Ok((data.items, data.total))
    }

    pub async fn get_episodes_page(&self, series_id: &str, start: usize, limit: usize) -> Result<Vec<MediaItem>> {
        let url = self.api_url(&format!("/Shows/{}/Episodes", series_id));
        let resp = self.authed_get(&url)
            .query(&[
                ("UserId", self.user_id.as_str()),
                ("Fields", "Overview,MediaSources,ChildCount"),
                ("Recursive", "false"),
                ("StartIndex", &start.to_string()),
                ("Limit", &limit.to_string()),
            ])
            .send()
            .await
            .context("Failed to fetch episodes")?;
        let data: ItemsResponse = resp.json().await.context("Invalid episodes response")?;
        Ok(data.items)
    }

    pub async fn get_seasons(&self, series_id: &str) -> Result<Vec<MediaItem>> {
        let url = self.api_url(&format!("/Shows/{}/Seasons", series_id));
        let resp = self.authed_get(&url)
            .query(&[
                ("UserId", self.user_id.as_str()),
                ("Fields", "Overview,ChildCount"),
            ])
            .send()
            .await
            .context("Failed to fetch seasons")?;
        let data: ItemsResponse = resp.json().await.context("Invalid seasons response")?;
        Ok(data.items)
    }

    pub async fn get_season_episodes(&self, series_id: &str, season_id: &str) -> Result<Vec<MediaItem>> {
        let url = self.api_url(&format!("/Shows/{}/Episodes", series_id));
        let resp = self.authed_get(&url)
            .query(&[
                ("UserId", self.user_id.as_str()),
                ("SeasonId", season_id),
                ("Fields", "Overview,MediaSources,ChildCount"),
                ("Limit", "1000"),
            ])
            .send()
            .await
            .context("Failed to fetch season episodes")?;
        let data: ItemsResponse = resp.json().await.context("Invalid episodes response")?;
        Ok(data.items)
    }

    pub async fn get_similar(&self, item_id: &str) -> Result<Vec<MediaItem>> {
        let url = self.api_url(&format!("/Items/{}/Similar", item_id));
        let resp = self.authed_get(&url)
            .query(&[
                ("UserId", self.user_id.as_str()),
                ("Limit", "12"),
                ("Fields", "Overview,ChildCount"),
            ])
            .send()
            .await
            .context("Failed to fetch similar items")?;
        let data: ItemsResponse = resp.json().await.context("Invalid similar response")?;
        Ok(data.items)
    }

    pub async fn get_latest_for_library(&self, library_id: &str, limit: usize) -> Result<Vec<MediaItem>> {
        let url = self.api_url(&format!("/Users/{}/Items/Latest", self.user_id));
        let resp = self.authed_get(&url)
            .query(&[
                ("ParentId", library_id),
                ("Limit", &limit.to_string()),
                ("Fields", "MediaSources"),
            ])
            .send()
            .await
            .context("Failed to fetch latest items")?;

        let body = resp.text().await.unwrap_or_default();
        // API returns bare array, not {"Items": [...]}
        let items: Vec<MediaItem> = serde_json::from_str(&body)
            .unwrap_or_default();
        Ok(items)
    }

    pub fn stream_url_for_source(&self, item: &MediaItem, source: &MediaSource) -> String {
        let media_source_id = if source.id.is_empty() { &item.id } else { &source.id };
        let container = if source.container.is_empty() { "mkv" } else { source.container.as_str() };
        let play_session_id = uuid::Uuid::new_v4().to_string().replace('-', "");
        let base = Url::parse(&self.base_url).unwrap();
        let path = format!("emby/videos/{}/original.{}", item.id, container);
        let mut url = base.join(&path).unwrap();
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("DeviceId", DEVICE_ID);
            q.append_pair("MediaSourceId", media_source_id);
            q.append_pair("PlaySessionId", &play_session_id);
            q.append_pair("api_key", &self.token);
        }
        url.to_string()
    }

    pub async fn get_genres(&self, parent_id: &str) -> Result<Vec<String>> {
        let url = self.api_url("/Genres");
        let resp = self.authed_get(&url)
            .query(&[
                ("UserId", self.user_id.as_str()),
                ("ParentId", parent_id),
            ])
            .send()
            .await
            .context("Failed to fetch genres")?;

        let data: serde_json::Value = resp.json().await.context("Invalid genres response")?;
        let items = data.get("Items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let genres: Vec<String> = items.iter()
            .filter_map(|item| item.get("Name").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();
        Ok(genres)
    }

    pub async fn get_tags(&self, parent_id: &str) -> Result<Vec<String>> {
        let url = self.api_url("/Tags");
        let resp = self.authed_get(&url)
            .query(&[
                ("UserId", self.user_id.as_str()),
                ("ParentId", parent_id),
            ])
            .send()
            .await
            .context("Failed to fetch tags")?;

        let data: serde_json::Value = resp.json().await.context("Invalid tags response")?;
        let items = data.get("Items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let tags: Vec<String> = items.iter()
            .filter_map(|item| item.get("Name").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();
        Ok(tags)
    }

    pub async fn get_studios(&self, parent_id: &str) -> Result<Vec<String>> {
        let url = self.api_url("/Studios");
        let resp = self.authed_get(&url)
            .query(&[
                ("UserId", self.user_id.as_str()),
                ("ParentId", parent_id),
            ])
            .send()
            .await
            .context("Failed to fetch studios")?;

        let data: serde_json::Value = resp.json().await.context("Invalid studios response")?;
        let items = data.get("Items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let studios: Vec<String> = items.iter()
            .filter_map(|item| item.get("Name").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();
        Ok(studios)
    }

    pub async fn get_folders(&self, parent_id: &str) -> Result<Vec<MediaItem>> {
        let url = self.api_url(&format!("/Users/{}/Items", self.user_id));
        let resp = self.authed_get(&url)
            .query(&[
                ("ParentId", parent_id),
                ("Recursive", "false"),
                ("IncludeItemTypes", "Folder,CollectionFolder"),
            ])
            .send()
            .await
            .context("Failed to fetch folders")?;

        let data: ItemsResponse = resp.json().await.context("Invalid folders response")?;
        Ok(data.items)
    }

    pub async fn toggle_favorite(&self, item_id: &str, is_favorite: bool) -> Result<()> {
        let url = self.api_url(&format!("/Users/{}/Items/{}/Favorite", self.user_id, item_id));
        let resp = if is_favorite {
            self.authed_get(&url).send().await
        } else {
            self.http.delete(&url)
                .header("X-Emby-Authorization", auth_header(&self.token))
                .header("X-Emby-Token", &self.token)
                .send()
                .await
        };
        resp.context("Failed to toggle favorite")?;
        Ok(())
    }

    pub async fn get_favorites(&self, start: usize, limit: usize) -> Result<PageResult> {
        let url = self.api_url(&format!("/Users/{}/Items", self.user_id));
        let resp = self.authed_get(&url)
            .query(&[
                ("Recursive", "true"),
                ("Filters", "IsFavorite"),
                ("Fields", "Overview,MediaSources,ChildCount"),
                ("StartIndex", &start.to_string()),
                ("Limit", &limit.to_string()),
                ("SortBy", "SortName"),
                ("SortOrder", "Ascending"),
            ])
            .send()
            .await
            .context("Failed to fetch favorites")?;

        let data: ItemsResponse = resp.json().await.context("Invalid favorites response")?;
        Ok(PageResult {
            items: data.items,
            total: data.total,
        })
    }

    pub async fn get_items_filtered(
        &self,
        parent_id: &str,
        start: usize,
        limit: usize,
        sort_by: &str,
        sort_order: &str,
        genres: Option<&str>,
        tags: Option<&str>,
        studios: Option<&str>,
        years: Option<&str>,
    ) -> Result<PageResult> {
        let url = self.api_url(&format!("/Users/{}/Items", self.user_id));
        let mut query = vec![
            ("ParentId", parent_id.to_string()),
            ("Recursive", "true".to_string()),
            ("Fields", "Overview,MediaSources,ChildCount".to_string()),
            ("StartIndex", start.to_string()),
            ("Limit", limit.to_string()),
            ("SortBy", sort_by.to_string()),
            ("SortOrder", sort_order.to_string()),
        ];

        if let Some(g) = genres {
            query.push(("Genres", g.to_string()));
        }
        if let Some(t) = tags {
            query.push(("Tags", t.to_string()));
        }
        if let Some(s) = studios {
            query.push(("Studios", s.to_string()));
        }
        if let Some(y) = years {
            query.push(("Years", y.to_string()));
        }

        let resp = self.authed_get(&url)
            .query(&query)
            .send()
            .await
            .context("Failed to fetch filtered items")?;

        let data: ItemsResponse = resp.json().await.context("Invalid filtered items response")?;
        Ok(PageResult {
            items: data.items,
            total: data.total,
        })
    }
}

impl MediaItem {
    pub fn separator(label: &str) -> Self {
        Self {
            id: String::new(),
            name: format!("── {label} ──"),
            item_type: "Separator".to_string(),
            media_type: None,
            series_name: None,
            index_number: None,
            parent_index_number: None,
            runtime_ticks: None,
            series_id: None,
            overview: None,
            child_count: None,
            media_sources: Vec::new(),
            user_data: None,
        }
    }

    pub fn is_separator(&self) -> bool {
        self.item_type == "Separator"
    }

    pub fn is_folder(&self) -> bool {
        self.item_type == "Folder"
            || self.item_type == "CollectionFolder"
    }

    pub fn is_navigable(&self) -> bool {
        self.is_folder() || self.item_type == "Series" || self.item_type == "Season"
    }

    pub fn is_video(&self) -> bool {
        self.item_type == "Episode" || self.item_type == "Movie"
            || self.item_type == "Video"
            || self.media_type.as_deref() == Some("Video")
    }

    pub fn resume_position_ticks(&self) -> Option<i64> {
        self.user_data.as_ref()
            .and_then(|ud| ud.playback_position_ticks)
            .filter(|&pos| pos > 0)
    }

    pub fn display_name(&self) -> String {
        // Show episode count for series and seasons
        if self.item_type == "Series" || self.item_type == "Season" {
            if let Some(count) = self.child_count {
                return format!("{} ({})", self.name, count);
            }
        }
        if let Some(ref series) = self.series_name {
            if let Some(ep) = self.index_number {
                let season = self.parent_index_number.unwrap_or(0);
                return format!("{series} S{season:02}E{ep:02} - {}", self.name);
            }
            return format!("{series} - {}", self.name);
        }
        self.name.clone()
    }

    pub fn duration_str(&self) -> Option<String> {
        self.runtime_ticks.map(|ticks| {
            let secs = ticks / 10_000_000;
            let h = secs / 3600;
            let m = (secs % 3600) / 60;
            let s = secs % 60;
            if h > 0 {
                format!("{h}h {m:02}m")
            } else {
                format!("{m}m {s:02}s")
            }
        })
    }
}
