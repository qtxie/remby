use anyhow::Result;

use crate::emby::{EmbyClient, Library, MediaItem, MediaSource, MediaStream};

const PAGE_SIZE: usize = 50;
const HOME_LIMIT: usize = 20;

pub struct AppState {
    pub client: EmbyClient,
    pub server: String,
    pub libraries: Vec<Library>,
    pub library_latest: Vec<(String, Vec<MediaItem>)>,
    pub items: Vec<MediaItem>,
    pub total_items: usize,
    pub current_folder_id: String,
    pub loading: bool,
    pub selected: usize,
    pub stack: Vec<StackEntry>,
    pub status_msg: String,
    pub searching: bool,
    pub search_query: String,
    pub search_results: Vec<MediaItem>,
    pub view: View,
    pub home_items: Vec<MediaItem>,
    pub source_state: SourceState,
    pub track_state: TrackState,
    pub episodes: Vec<MediaItem>,
    pub series_name: String,
    pub series_state: SeriesState,
    pub playing_state: PlayingState,
    pub mpv_child: Option<std::process::Child>,
}

pub(crate) struct StackEntry {
    items: Vec<MediaItem>,
    folder_id: String,
    view: View,
}

pub struct SourceState {
    pub item: Option<MediaItem>,
    pub sources: Vec<MediaSource>,
}

pub struct SeriesState {
    pub item: Option<MediaItem>,
    pub overview: String,
    pub seasons: Vec<MediaItem>,
    pub episodes: Vec<MediaItem>,
    pub similar: Vec<MediaItem>,
    pub selected_season: usize,
    pub selected_episode: usize,
    pub section: SeriesSection,
}

#[derive(PartialEq, Clone, Debug)]
pub enum SeriesSection {
    Seasons,
    Episodes,
    Similar,
}

pub struct PlayingState {
    pub item_name: String,
    pub video_track: String,
    pub audio_track: String,
    pub subtitle_track: String,
    pub url: String,
    pub resume_position: Option<i64>,
    pub chosen_start: Option<f64>,
    pub option_selected: usize,
    pub playing: bool,
}

impl Default for PlayingState {
    fn default() -> Self {
        Self {
            item_name: String::new(),
            video_track: String::new(),
            audio_track: String::new(),
            subtitle_track: String::new(),
            url: String::new(),
            resume_position: None,
            chosen_start: None,
            option_selected: 0,
            playing: false,
        }
    }
}

impl Default for SeriesState {
    fn default() -> Self {
        Self {
            item: None,
            overview: String::new(),
            seasons: Vec::new(),
            episodes: Vec::new(),
            similar: Vec::new(),
            selected_season: 0,
            selected_episode: 0,
            section: SeriesSection::Seasons,
        }
    }
}

pub struct TrackState {
    pub item: Option<MediaItem>,
    pub media_source: Option<MediaSource>,
    pub video_tracks: Vec<MediaStream>,
    pub audio_tracks: Vec<MediaStream>,
    pub subtitle_tracks: Vec<MediaStream>,
    pub selected_video: usize,
    pub selected_audio: usize,
    pub selected_subtitle: usize,
    pub section: TrackSection,
}

#[derive(PartialEq, Clone, Debug)]
pub enum TrackSection {
    Video,
    Audio,
    Subtitle,
}

impl Default for SourceState {
    fn default() -> Self {
        Self {
            item: None,
            sources: Vec::new(),
        }
    }
}

impl Default for TrackState {
    fn default() -> Self {
        Self {
            item: None,
            media_source: None,
            video_tracks: Vec::new(),
            audio_tracks: Vec::new(),
            subtitle_tracks: Vec::new(),
            selected_video: 0,
            selected_audio: 0,
            selected_subtitle: 0,
            section: TrackSection::Video,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum View {
    Home,
    Libraries,
    Items,
    SearchResults,
    SourceSelect,
    TrackSelect,
    Episodes,
    SeriesInfo,
    Playing,
}

impl AppState {
    pub async fn new(
        server: Option<String>,
        user: Option<String>,
        pass: Option<String>,
    ) -> Result<Self> {
        let server = server.unwrap_or_default();
        let user = user.unwrap_or_default();
        let pass = pass.unwrap_or_default();

        let client = if !user.is_empty() && !pass.is_empty() && !server.is_empty() {
            EmbyClient::authenticate(&server, &user, &pass).await?
        } else {
            EmbyClient::new(server.clone(), String::new())
        };

        Ok(Self {
            client,
            server,
            libraries: Vec::new(),
            library_latest: Vec::new(),
            items: Vec::new(),
            total_items: 0,
            current_folder_id: String::new(),
            loading: false,
            selected: 0,
            stack: Vec::new(),
            status_msg: String::new(),
            searching: false,
            search_query: String::new(),
            search_results: Vec::new(),
            view: View::Home,
            home_items: Vec::new(),
            source_state: SourceState::default(),
            track_state: TrackState::default(),
            episodes: Vec::new(),
            series_name: String::new(),
            series_state: SeriesState::default(),
            playing_state: PlayingState::default(),
            mpv_child: None,
        })
    }

    pub async fn load_home(&mut self) -> Result<()> {
        self.loading = true;
        self.status_msg = "Loading...".to_string();

        let resume_fut = self.client.get_resume_items(HOME_LIMIT);
        let latest_fut = self.client.get_latest_items(HOME_LIMIT);

        let (resume_result, latest_result) = tokio::join!(resume_fut, latest_fut);

        let mut items = Vec::new();
        if let Ok(resume) = resume_result {
            if !resume.is_empty() {
                items.push(MediaItem::separator("Continue Watching"));
                items.extend(resume);
            }
        }
        if let Ok(latest) = latest_result {
            if !latest.is_empty() {
                items.push(MediaItem::separator("Latest"));
                items.extend(latest);
            }
        }

        self.home_items = items;
        self.loading = false;
        self.status_msg = format!("{} items", self.home_items.len());
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn load_libraries(&mut self) -> Result<()> {
        match self.client.get_libraries().await {
            Ok(libs) => {
                self.libraries = libs;
                self.library_latest.clear();
                for lib in &self.libraries {
                    match self.client.get_latest_for_library(&lib.id, 10).await {
                        Ok(items) => {
                            if !items.is_empty() {
                                self.library_latest.push((lib.name.clone(), items));
                            }
                        }
                        Err(_) => {}
                    }
                }
                self.status_msg = format!("{} libraries", self.libraries.len());
            }
            Err(e) => {
                self.status_msg = format!("Error: {e}");
            }
        }
        Ok(())
    }

    pub async fn browse_folder(&mut self, folder_id: &str) -> Result<()> {
        self.stack.push(StackEntry {
            items: self.items.clone(),
            folder_id: self.current_folder_id.clone(),
            view: self.view.clone(),
        });
        self.items.clear();
        self.total_items = 0;
        self.current_folder_id = folder_id.to_string();
        self.selected = 0;
        self.view = View::Items;
        self.loading = true;
        self.status_msg = "Loading...".to_string();

        match self.client.get_items(folder_id, 0, PAGE_SIZE).await {
            Ok(page) => {
                self.total_items = page.total;
                self.items = page.items;
                self.status_msg = format!("{} items", self.total_items);
            }
            Err(e) => {
                self.status_msg = format!("Error: {e}");
            }
        }
        self.loading = false;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn load_more_items(&mut self) -> Result<()> {
        if self.loading || self.view != View::Items {
            return Ok(());
        }
        if self.items.len() >= self.total_items {
            return Ok(());
        }
        self.loading = true;

        match self.client.get_items(&self.current_folder_id, self.items.len(), PAGE_SIZE).await {
            Ok(page) => {
                self.total_items = page.total;
                self.items.extend(page.items);
                self.status_msg = format!("{} items", self.total_items);
            }
            Err(e) => {
                self.status_msg = format!("Error loading more: {e}");
            }
        }
        self.loading = false;
        Ok(())
    }

    pub fn open_source_select(&mut self, item: &MediaItem, sources: Vec<MediaSource>) {
        self.source_state = SourceState {
            item: Some(item.clone()),
            sources,
        };
        self.selected = 0;
        self.view = View::SourceSelect;
    }

    pub async fn load_episodes(&mut self, series_id: &str, series_name: &str) -> Result<()> {
        self.loading = true;
        self.series_name = series_name.to_string();
        self.status_msg = format!("Loading episodes of {}...", series_name);
        match self.client.get_episodes(series_id).await {
            Ok(episodes) => {
                self.episodes = episodes;
                self.selected = 0;
                self.view = View::Episodes;
                self.status_msg = format!("{} episodes", self.episodes.len());
            }
            Err(e) => {
                self.status_msg = format!("Error: {e}");
            }
        }
        self.loading = false;
        Ok(())
    }

    pub async fn load_series_info(&mut self, item: &MediaItem) -> Result<()> {
        let series_id = item.series_id.as_deref().unwrap_or(&item.id);
        self.loading = true;
        self.status_msg = "Loading series info...".to_string();

        let (detail, seasons, similar) = tokio::join!(
            self.client.get_item_detail(series_id),
            self.client.get_seasons(series_id),
            self.client.get_similar(series_id),
        );

        let mut episodes = Vec::new();
        if let Ok(ref s) = seasons {
            if let Some(first_season) = s.first() {
                if let Ok(eps) = self.client.get_season_episodes(series_id, &first_season.id).await {
                    episodes = eps;
                }
            }
        }

        let overview_text = match &detail {
            Ok(item) => item.overview.clone().unwrap_or_default(),
            Err(_) => String::new(),
        };

        self.series_state = SeriesState {
            item: detail.ok(),
            overview: overview_text,
            seasons: seasons.unwrap_or_default(),
            episodes,
            similar: similar.unwrap_or_default(),
            selected_season: 0,
            selected_episode: 0,
            section: SeriesSection::Seasons,
        };
        self.selected = 0;
        self.view = View::SeriesInfo;
        self.loading = false;
        self.status_msg = String::new();
        Ok(())
    }

    pub fn series_section_next(&mut self) {
        self.series_state.section = match self.series_state.section {
            SeriesSection::Seasons => SeriesSection::Episodes,
            SeriesSection::Episodes => SeriesSection::Similar,
            SeriesSection::Similar => SeriesSection::Seasons,
        };
        self.selected = match self.series_state.section {
            SeriesSection::Seasons => self.series_state.selected_season,
            SeriesSection::Episodes => self.series_state.selected_episode,
            SeriesSection::Similar => 0,
        };
    }

    pub fn series_section_prev(&mut self) {
        self.series_state.section = match self.series_state.section {
            SeriesSection::Seasons => SeriesSection::Similar,
            SeriesSection::Episodes => SeriesSection::Seasons,
            SeriesSection::Similar => SeriesSection::Episodes,
        };
        self.selected = match self.series_state.section {
            SeriesSection::Seasons => self.series_state.selected_season,
            SeriesSection::Episodes => self.series_state.selected_episode,
            SeriesSection::Similar => 0,
        };
    }

    pub fn series_select_next(&mut self) {
        let len = self.series_current_len();
        if len > 0 {
            self.selected = (self.selected + 1) % len;
            self.sync_series_selection();
        }
    }

    pub fn series_select_prev(&mut self) {
        let len = self.series_current_len();
        if len > 0 {
            self.selected = (self.selected + len - 1) % len;
            self.sync_series_selection();
        }
    }

    fn sync_series_selection(&mut self) {
        match self.series_state.section {
            SeriesSection::Seasons => self.series_state.selected_season = self.selected,
            SeriesSection::Episodes => self.series_state.selected_episode = self.selected,
            _ => {}
        }
    }

    pub async fn select_season(&mut self) -> Result<()> {
        if let Some(season) = self.series_state.seasons.get(self.selected) {
            let series_id = self.series_state.item.as_ref()
                .and_then(|i| i.series_id.as_deref())
                .or(self.series_state.item.as_ref().map(|i| i.id.as_str()))
                .unwrap_or("")
                .to_string();
            self.loading = true;
            self.status_msg = format!("Loading {}...", season.name);
            if let Ok(eps) = self.client.get_season_episodes(&series_id, &season.id).await {
                self.series_state.episodes = eps;
                self.series_state.selected_episode = 0;
                self.series_state.section = SeriesSection::Episodes;
                self.selected = 0;
            }
            self.loading = false;
            self.status_msg = String::new();
        }
        Ok(())
    }

    pub fn series_current_len(&self) -> usize {
        match self.series_state.section {
            SeriesSection::Seasons => self.series_state.seasons.len(),
            SeriesSection::Episodes => self.series_state.episodes.len(),
            SeriesSection::Similar => self.series_state.similar.len(),
        }
    }

    pub fn series_selected_item(&self) -> Option<&MediaItem> {
        match self.series_state.section {
            SeriesSection::Seasons => self.series_state.seasons.get(self.selected),
            SeriesSection::Episodes => self.series_state.episodes.get(self.selected),
            SeriesSection::Similar => self.series_state.similar.get(self.selected),
        }
    }

    pub fn selected_source(&self) -> Option<&MediaSource> {
        self.source_state.sources.get(self.selected)
    }

    pub async fn go_back(&mut self) -> Result<()> {
        self.status_msg.clear();
        if self.searching {
            self.cancel_search();
            return Ok(());
        }
        match self.view {
            View::Home => {}
            View::Libraries => {
                self.view = View::Home;
                self.selected = 0;
            }
            View::Items | View::SearchResults => {
                if let Some(prev) = self.stack.pop() {
                    self.items = prev.items;
                    self.current_folder_id = prev.folder_id;
                    self.view = prev.view;
                    self.selected = 0;
                }
            }
            View::SourceSelect => {
                self.view = View::Home;
                self.selected = 0;
            }
            View::TrackSelect => {
                if self.source_state.sources.len() > 1 {
                    self.view = View::SourceSelect;
                    self.selected = 0;
                } else {
                    self.view = View::Home;
                    self.selected = 0;
                }
            }
            View::Episodes => {
                self.view = View::Home;
                self.selected = 0;
            }
            View::SeriesInfo => {
                self.view = View::Home;
                self.selected = 0;
            }
            View::Playing => {
                self.kill_mpv();
                self.view = View::TrackSelect;
                self.selected = 0;
            }
        }
        Ok(())
    }

    pub fn select_next(&mut self) {
        let len = self.current_list_len();
        if len > 0 {
            self.selected = (self.selected + 1) % len;
        }
    }

    pub fn select_prev(&mut self) {
        let len = self.current_list_len();
        if len > 0 {
            self.selected = (self.selected + len - 1) % len;
        }
    }

    pub fn selected_item(&self) -> Option<&MediaItem> {
        match self.view {
            View::Home => self.home_items.get(self.selected),
            View::Libraries => {
                // Combined list: libraries + section headers + latest items
                let mut idx = self.selected;

                // Skip "Libraries" header
                if idx == 0 {
                    return None;
                }
                idx -= 1;

                // Check libraries
                if idx < self.libraries.len() {
                    return None; // Libraries are not MediaItems
                }
                idx -= self.libraries.len();

                // Check latest items sections
                for (_, items) in &self.library_latest {
                    // Skip section header
                    if idx == 0 {
                        return None;
                    }
                    idx -= 1;
                    if idx < items.len() {
                        return items.get(idx);
                    }
                    idx -= items.len();
                }
                None
            }
            View::Items => self.items.get(self.selected),
            View::SearchResults => self.search_results.get(self.selected),
            View::SourceSelect => self.source_state.item.as_ref(),
            View::TrackSelect => self.track_state.item.as_ref(),
            View::Episodes => self.episodes.get(self.selected),
            View::SeriesInfo => self.series_selected_item(),
            View::Playing => self.track_state.item.as_ref(),
        }
    }

    pub fn selected_library(&self) -> Option<&Library> {
        if self.view == View::Libraries {
            let idx = self.selected;
            // Skip "Libraries" header (index 0)
            if idx > 0 && idx <= self.libraries.len() {
                return self.libraries.get(idx - 1);
            }
        }
        None
    }

    pub async fn browse_library(&mut self, lib: &Library) -> Result<()> {
        self.browse_folder(&lib.id).await
    }

    pub async fn show_libraries(&mut self) {
        self.stack.push(StackEntry {
            items: self.items.clone(),
            folder_id: self.current_folder_id.clone(),
            view: self.view.clone(),
        });
        self.view = View::Libraries;
        self.selected = 0;
        self.loading = true;
        self.status_msg = "Loading libraries...".to_string();
    }

    pub async fn load_libraries_bg(&mut self) {
        self.load_libraries().await;
        self.loading = false;
    }

    pub fn start_search(&mut self) {
        self.searching = true;
        self.search_query.clear();
        self.search_results.clear();
    }

    pub fn search_input(&mut self, c: char) {
        self.search_query.push(c);
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
    }

    pub async fn submit_search(&mut self) -> Result<()> {
        if self.search_query.is_empty() {
            return Ok(());
        }
        match self.client.search(&self.search_query).await {
            Ok(results) => {
                self.search_results = results;
                self.selected = 0;
                self.view = View::SearchResults;
                self.status_msg = format!("{} results", self.search_results.len());
            }
            Err(e) => {
                self.status_msg = format!("Search error: {e}");
            }
        }
        Ok(())
    }

    pub fn cancel_search(&mut self) {
        self.searching = false;
        self.search_query.clear();
    }

    #[allow(dead_code)]
    pub fn should_load_more(&self) -> bool {
        self.view == View::Items
            && !self.loading
            && self.items.len() < self.total_items
            && self.selected + 5 >= self.items.len()
    }

    pub fn open_track_select(&mut self, item: &MediaItem, source: &MediaSource) {
        self.track_state = TrackState {
            item: Some(item.clone()),
            media_source: Some(source.clone()),
            video_tracks: source.media_streams.iter()
                .filter(|s| s.stream_type == "Video")
                .cloned().collect(),
            audio_tracks: source.media_streams.iter()
                .filter(|s| s.stream_type == "Audio")
                .cloned().collect(),
            subtitle_tracks: source.media_streams.iter()
                .filter(|s| s.stream_type == "Subtitle")
                .cloned().collect(),
            selected_video: 0,
            selected_audio: 0,
            selected_subtitle: 0,
            section: TrackSection::Video,
        };
        self.view = View::TrackSelect;
    }

    pub fn track_section_next(&mut self) {
        self.track_state.section = match self.track_state.section {
            TrackSection::Video => TrackSection::Audio,
            TrackSection::Audio => TrackSection::Subtitle,
            TrackSection::Subtitle => TrackSection::Video,
        };
    }

    pub fn track_section_prev(&mut self) {
        self.track_state.section = match self.track_state.section {
            TrackSection::Video => TrackSection::Subtitle,
            TrackSection::Audio => TrackSection::Video,
            TrackSection::Subtitle => TrackSection::Audio,
        };
    }

    pub fn track_select_next(&mut self) {
        match self.track_state.section {
            TrackSection::Video => {
                let len = self.track_state.video_tracks.len();
                if len > 0 { self.track_state.selected_video = (self.track_state.selected_video + 1) % len; }
            }
            TrackSection::Audio => {
                let len = self.track_state.audio_tracks.len();
                if len > 0 { self.track_state.selected_audio = (self.track_state.selected_audio + 1) % len; }
            }
            TrackSection::Subtitle => {
                let len = self.track_state.subtitle_tracks.len();
                if len > 0 { self.track_state.selected_subtitle = (self.track_state.selected_subtitle + 1) % len; }
            }
        }
    }

    pub fn track_select_prev(&mut self) {
        match self.track_state.section {
            TrackSection::Video => {
                let len = self.track_state.video_tracks.len();
                if len > 0 { self.track_state.selected_video = (self.track_state.selected_video + len - 1) % len; }
            }
            TrackSection::Audio => {
                let len = self.track_state.audio_tracks.len();
                if len > 0 { self.track_state.selected_audio = (self.track_state.selected_audio + len - 1) % len; }
            }
            TrackSection::Subtitle => {
                let len = self.track_state.subtitle_tracks.len();
                if len > 0 { self.track_state.selected_subtitle = (self.track_state.selected_subtitle + len - 1) % len; }
            }
        }
    }

    pub fn get_selected_tracks(&self) -> (Option<i32>, Option<i32>, Option<i32>) {
        // Return sequential position (0-based) per track type, not the API Index
        let video = if self.track_state.video_tracks.is_empty() {
            None
        } else {
            Some(self.track_state.selected_video as i32)
        };
        let audio = if self.track_state.audio_tracks.is_empty() {
            None
        } else {
            Some(self.track_state.selected_audio as i32)
        };
        let sub = if self.track_state.subtitle_tracks.is_empty() {
            None
        } else {
            Some(self.track_state.selected_subtitle as i32)
        };
        (video, audio, sub)
    }

    pub fn kill_mpv(&mut self) {
        if let Some(mut child) = self.mpv_child.take() {
            let _ = child.kill();
        }
    }

    pub fn open_playing(&mut self, item_name: &str, url: &str, video: &str, audio: &str, subtitle: &str, resume_ticks: Option<i64>) {
        self.playing_state = PlayingState {
            item_name: item_name.to_string(),
            url: url.to_string(),
            video_track: video.to_string(),
            audio_track: audio.to_string(),
            subtitle_track: subtitle.to_string(),
            resume_position: resume_ticks,
            chosen_start: None,
            option_selected: 0,
            playing: false,
        };
        self.view = View::Playing;
    }

    fn current_list_len(&self) -> usize {
        match self.view {
            View::Home => self.home_items.len(),
            View::Libraries => {
                // Libraries + section headers + latest items
                let mut count = self.libraries.len() + 1; // +1 for "Libraries" header
                for (_, items) in &self.library_latest {
                    count += 1 + items.len(); // +1 for section header
                }
                count
            }
            View::Items => self.items.len(),
            View::SearchResults => self.search_results.len(),
            View::SourceSelect => self.source_state.sources.len(),
            View::TrackSelect => {
                match self.track_state.section {
                    TrackSection::Video => self.track_state.video_tracks.len(),
                    TrackSection::Audio => self.track_state.audio_tracks.len(),
                    TrackSection::Subtitle => self.track_state.subtitle_tracks.len(),
                }
            }
            View::Episodes => self.episodes.len(),
            View::SeriesInfo => self.series_current_len(),
            View::Playing => 0,
        }
    }
}
