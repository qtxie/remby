use anyhow::Result;
use std::time::Instant;

use crate::config::RembyConfig;
use crate::emby::{EmbyClient, Library, MediaItem, MediaSource, MediaStream};

const CACHE_TTL_SECS: u64 = 300; // 5 minutes

pub struct AppState {
    pub client: EmbyClient,
    pub server: String,
    pub libraries: Vec<Library>,
    pub libraries_fetched_at: Option<Instant>,
    pub library_latest: Vec<(String, Vec<MediaItem>)>,
    pub library_latest_fetched_at: Option<Instant>,
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
    pub total_episodes: usize,
    pub episodes_series_id: String,
    pub series_name: String,
    pub series_state: SeriesState,
    pub playing_state: PlayingState,
    pub mpv_child: Option<std::process::Child>,
    pub settings_state: SettingsState,
    pub config: RembyConfig,
}

pub(crate) struct StackEntry {
    pub items: Vec<MediaItem>,
    pub folder_id: String,
    pub view: View,
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
    pub option_selected: usize,
    pub playing: bool,
}

#[derive(Clone)]
pub struct SettingsLibrary {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub fetch_latest: bool,
}

pub struct SettingsState {
    pub libraries: Vec<SettingsLibrary>,
    pub selected: usize,
    pub column: SettingsColumn,
}

#[derive(PartialEq, Clone, Debug)]
pub enum SettingsColumn {
    Enabled,
    Latest,
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
            option_selected: 0,
            playing: false,
        }
    }
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            libraries: Vec::new(),
            selected: 0,
            column: SettingsColumn::Enabled,
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
    Settings,
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

        let config = crate::config::load_config();

        Ok(Self {
            client,
            server,
            libraries: Vec::new(),
            libraries_fetched_at: None,
            library_latest: Vec::new(),
            library_latest_fetched_at: None,
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
            total_episodes: 0,
            episodes_series_id: String::new(),
            series_name: String::new(),
            series_state: SeriesState::default(),
            playing_state: PlayingState::default(),
            mpv_child: None,
            settings_state: SettingsState::default(),
            config,
        })
    }

    pub fn open_source_select(&mut self, item: &MediaItem, sources: Vec<MediaSource>) {
        self.stack.push(StackEntry {
            items: self.items.clone(),
            folder_id: self.current_folder_id.clone(),
            view: self.view.clone(),
        });
        self.source_state = SourceState {
            item: Some(item.clone()),
            sources,
        };
        self.selected = 0;
        self.view = View::SourceSelect;
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
            View::SourceSelect | View::TrackSelect | View::Playing => {
                if let Some(prev) = self.stack.pop() {
                    self.items = prev.items;
                    self.current_folder_id = prev.folder_id;
                    self.view = prev.view;
                    self.selected = 0;
                } else {
                    self.view = View::Home;
                    self.selected = 0;
                }
            }
            View::Episodes | View::SeriesInfo => {
                if let Some(prev) = self.stack.pop() {
                    self.items = prev.items;
                    self.current_folder_id = prev.folder_id;
                    self.view = prev.view;
                    self.selected = 0;
                } else {
                    self.view = View::Home;
                    self.selected = 0;
                }
            }
            View::Settings => {
                self.settings_cancel();
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
            View::Settings => None,
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

    pub fn is_libraries_cache_valid(&self) -> bool {
        self.libraries_fetched_at
            .map(|t| t.elapsed().as_secs() < CACHE_TTL_SECS)
            .unwrap_or(false)
    }

    pub fn is_latest_cache_valid(&self) -> bool {
        self.library_latest_fetched_at
            .map(|t| t.elapsed().as_secs() < CACHE_TTL_SECS)
            .unwrap_or(false)
    }

    pub fn should_load_more_episodes(&self) -> bool {
        self.view == View::Episodes
            && !self.loading
            && self.total_episodes > self.episodes.len()
            && self.selected + 5 >= self.episodes.len() * 2 / 3
    }

    pub fn should_load_more_items(&self) -> bool {
        self.view == View::Items
            && !self.loading
            && self.total_items > self.items.len()
            && self.selected + 5 >= self.items.len() * 2 / 3
    }

    pub async fn show_libraries(&mut self) {
        self.stack.push(StackEntry {
            items: self.items.clone(),
            folder_id: self.current_folder_id.clone(),
            view: self.view.clone(),
        });
        self.view = View::Libraries;
        self.selected = 0;
        if !self.is_libraries_cache_valid() || !self.is_latest_cache_valid() {
            self.loading = true;
            self.status_msg = "Loading libraries...".to_string();
        }
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

    pub fn cancel_search(&mut self) {
        self.searching = false;
        self.search_query.clear();
    }

    pub fn open_track_select(&mut self, item: &MediaItem, source: &MediaSource) {
        self.stack.push(StackEntry {
            items: self.items.clone(),
            folder_id: self.current_folder_id.clone(),
            view: self.view.clone(),
        });
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

    pub fn kill_mpv(&mut self) {
        if let Some(mut child) = self.mpv_child.take() {
            let _ = child.kill();
        }
    }

    pub fn open_settings(&mut self) {
        self.settings_state = SettingsState {
            libraries: self.libraries.iter().map(|lib| {
                SettingsLibrary {
                    id: lib.id.clone(),
                    name: lib.name.clone(),
                    enabled: self.config.enabled_libraries.is_empty() || self.config.enabled_libraries.contains(&lib.id),
                    fetch_latest: self.config.latest_libraries.is_empty() || self.config.latest_libraries.contains(&lib.id),
                }
            }).collect(),
            selected: 0,
            column: SettingsColumn::Enabled,
        };
        self.view = View::Settings;
        self.selected = 0;
    }

    pub fn settings_select_next(&mut self) {
        let len = self.settings_state.libraries.len();
        if len > 0 {
            self.settings_state.selected = (self.settings_state.selected + 1) % len;
        }
    }

    pub fn settings_select_prev(&mut self) {
        let len = self.settings_state.libraries.len();
        if len > 0 {
            self.settings_state.selected = (self.settings_state.selected + len - 1) % len;
        }
    }

    pub fn settings_toggle(&mut self) {
        if let Some(lib) = self.settings_state.libraries.get_mut(self.settings_state.selected) {
            match self.settings_state.column {
                SettingsColumn::Enabled => lib.enabled = !lib.enabled,
                SettingsColumn::Latest => lib.fetch_latest = !lib.fetch_latest,
            }
        }
    }

    pub fn settings_move_up(&mut self) {
        let idx = self.settings_state.selected;
        if idx > 0 {
            self.settings_state.libraries.swap(idx, idx - 1);
            self.settings_state.selected = idx - 1;
        }
    }

    pub fn settings_move_down(&mut self) {
        let idx = self.settings_state.selected;
        if idx + 1 < self.settings_state.libraries.len() {
            self.settings_state.libraries.swap(idx, idx + 1);
            self.settings_state.selected = idx + 1;
        }
    }

    pub fn settings_switch_column(&mut self) {
        self.settings_state.column = match self.settings_state.column {
            SettingsColumn::Enabled => SettingsColumn::Latest,
            SettingsColumn::Latest => SettingsColumn::Enabled,
        };
    }

    pub fn settings_save(&mut self) {
        let enabled: Vec<String> = self.settings_state.libraries.iter()
            .filter(|l| l.enabled)
            .map(|l| l.id.clone())
            .collect();
        let latest: Vec<String> = self.settings_state.libraries.iter()
            .filter(|l| l.enabled && l.fetch_latest)
            .map(|l| l.id.clone())
            .collect();
        self.config.enabled_libraries = enabled;
        self.config.latest_libraries = latest;
        if let Err(e) = crate::config::save_config(&self.config) {
            self.status_msg = format!("Save error: {e}");
        } else {
            self.status_msg = "Settings saved".to_string();
        }
        self.libraries.clear();
        self.libraries_fetched_at = None;
        self.library_latest.clear();
        self.library_latest_fetched_at = None;
        self.view = View::Libraries;
        self.selected = 0;
        self.loading = true;
    }

    pub fn settings_cancel(&mut self) {
        self.view = View::Home;
        self.selected = 0;
    }

    pub fn open_playing(&mut self, item_name: &str, url: &str, video: &str, audio: &str, subtitle: &str, resume_ticks: Option<i64>) {
        self.stack.push(StackEntry {
            items: self.items.clone(),
            folder_id: self.current_folder_id.clone(),
            view: self.view.clone(),
        });
        self.playing_state = PlayingState {
            item_name: item_name.to_string(),
            url: url.to_string(),
            video_track: video.to_string(),
            audio_track: audio.to_string(),
            subtitle_track: subtitle.to_string(),
            resume_position: resume_ticks,
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
            View::Settings => self.settings_state.libraries.len(),
        }
    }
}
