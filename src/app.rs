use anyhow::Result;
use std::time::Instant;

use crate::config::RembyConfig;
use crate::emby::{EmbyClient, Library, MediaItem, MediaSource, MediaStream};

const CACHE_TTL_SECS: u64 = 300; // 5 minutes

pub enum BackgroundResult {
    HomeSectionLoaded(Vec<MediaItem>, usize, String),
    MoreHomeItemsLoaded(Vec<MediaItem>),
    HomeLoaded(Vec<MediaItem>),
    FollowingUpdatesLoaded(Vec<(String, Vec<MediaItem>)>),
    LibrariesLoaded(Vec<Library>, Vec<(String, Vec<MediaItem>)>),
    SettingsLoaded(Vec<Library>),
    SeriesInfoLoaded(SeriesState),
    EpisodesLoaded(String, Vec<MediaItem>, usize, String),
    MoreEpisodesLoaded(Vec<MediaItem>),
    FolderLoaded(Vec<MediaItem>, String, usize),
    MoreItemsLoaded(Vec<MediaItem>, String),
    SearchLoaded(Vec<MediaItem>),
    ItemDetailLoaded(MediaItem),
    LibraryBrowserLoaded(Vec<MediaItem>, String, usize, Vec<String>, Vec<String>, Vec<String>, Vec<MediaItem>),
    MoreLibraryBrowserLoaded(Vec<MediaItem>, String),
    FavoritesLoaded(Vec<MediaItem>, usize),
    SeriesMarkedWatched(String, usize),
    FavoriteToggled(String, bool, String),
    AccountLoginSuccess(crate::config::Account, EmbyClient, String),
    AccountLoginFailed(String),
    Error(String),
    Timeout(String),
}

pub enum Message {
    Info(String),
    Success(String),
    Error(String),
    Loading(String, String),
}

impl Message {
    pub fn info(s: impl Into<String>) -> Self { Message::Info(s.into()) }
    pub fn success(s: impl Into<String>) -> Self { Message::Success(s.into()) }
    pub fn error(s: impl Into<String>) -> Self { Message::Error(s.into()) }
}

#[derive(PartialEq, Clone, Debug)]
pub enum SearchContext {
    ServerWide,
    LocalHome,
    Library(String),
}

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
    pub loading_msg: String,
    pub selected: usize,
    pub stack: Vec<StackEntry>,
    pub status_msg: Option<Message>,
    pub searching: bool,
    pub search_context: SearchContext,
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
    pub mpv_rx: Option<std::sync::mpsc::Receiver<crate::mpv::MpvEvent>>,
    pub mpv_output: Vec<(String, String)>,
    pub mpv_output_scroll: usize,
    pub mpv_position: f64,
    pub mpv_duration: f64,
    pub play_session_id: String,
    pub playback_started_at: Option<std::time::Instant>,
    pub bg_tx: Option<tokio::sync::mpsc::UnboundedSender<BackgroundResult>>,
    pub settings_state: SettingsState,
    pub config: RembyConfig,
    pub library_browser_state: LibraryBrowserState,
    pub favorites: Vec<MediaItem>,
    pub total_favorites: usize,
    pub following_updates: Vec<(String, Vec<MediaItem>)>,
    pub account_manager_state: AccountManagerState,
    pub wizard_state: WizardState,
    pub mpv_prompt_state: MpvPromptState,
}

pub(crate) struct StackEntry {
    pub items: Vec<MediaItem>,
    pub folder_id: String,
    pub view: View,
    pub selected: usize,
    pub home_items: Vec<MediaItem>,
    pub total_items: usize,
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
    pub item_id: String,
    pub media_source_id: String,
    pub runtime_ticks: Option<i64>,
    pub video_track: String,
    pub audio_track: String,
    pub subtitle_track: String,
    pub selected_video: usize,
    pub selected_audio: usize,
    pub selected_subtitle: usize,
    pub media_source: Option<crate::emby::MediaSource>,
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
    pub section: SettingsSection,
    pub mpv_path: String,
    pub language: String,
}

#[derive(PartialEq, Clone, Debug)]
pub enum SettingsSection {
    Libraries,
    MpvPath,
    Language,
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
            item_id: String::new(),
            media_source_id: String::new(),
            runtime_ticks: None,
            video_track: String::new(),
            audio_track: String::new(),
            subtitle_track: String::new(),
            selected_video: 0,
            selected_audio: 0,
            selected_subtitle: 0,
            media_source: None,
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
            section: SettingsSection::Libraries,
            mpv_path: String::new(),
            language: String::new(),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum AccountManagerAction {
    View,
    Add,
    Edit(usize),
    Delete(usize),
}

#[derive(PartialEq, Clone, Debug)]
pub enum AccountInputField {
    Label,
    Server,
    Username,
    Password,
}

pub enum WizardAction {
    None,
    FinishWizard,
}

pub struct AccountManagerState {
    pub accounts: Vec<crate::config::Account>,
    pub last_account_id: Option<String>,
    pub selected: usize,
    pub action: AccountManagerAction,
    pub input_server: String,
    pub input_username: String,
    pub input_password: String,
    pub input_label: String,
    pub input_field: AccountInputField,
    pub status_msg: Option<String>,
}

impl Default for AccountManagerState {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            last_account_id: None,
            selected: 0,
            action: AccountManagerAction::View,
            input_server: String::new(),
            input_username: String::new(),
            input_password: String::new(),
            input_label: String::new(),
            input_field: AccountInputField::Label,
            status_msg: None,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum WizardField {
    Language,
    Server,
    Username,
    Password,
    MpvPath,
}

pub struct WizardState {
    pub step: WizardField,
    pub language: String,
    pub server: String,
    pub username: String,
    pub password: String,
    pub mpv_path: String,
    pub status_msg: Option<String>,
}

impl Default for WizardState {
    fn default() -> Self {
        Self {
            step: WizardField::Language,
            language: crate::i18n::detect_system_lang().to_string(),
            server: String::new(),
            username: String::new(),
            password: String::new(),
            mpv_path: "mpv".to_string(),
            status_msg: None,
        }
    }
}

pub struct MpvPromptState {
    pub mpv_path: String,
    pub url: String,
    pub resume_position: Option<i64>,
}

impl Default for MpvPromptState {
    fn default() -> Self {
        Self {
            mpv_path: "mpv".to_string(),
            url: String::new(),
            resume_position: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ItemSort {
    Name,
    Year,
    Rating,
    DateAdded,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BrowserPanel {
    None,
    Sort,
    Filter,
}

#[derive(Clone, Debug, PartialEq)]
pub enum YearField {
    Start,
    End,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FilterSection {
    Genre,
    Tag,
    Studio,
    Year,
    Folder,
}

pub struct LibraryBrowserState {
    pub library_id: String,
    pub library_name: String,
    pub items: Vec<MediaItem>,
    pub total: usize,
    pub sort_by: ItemSort,
    pub sort_order: SortOrder,
    pub filter_section: FilterSection,
    pub filter_genre: Option<String>,
    pub filter_tag: Option<String>,
    pub filter_studio: Option<String>,
    pub filter_years: Option<(u32, u32)>,
    pub filter_folder: Option<String>,
    pub available_genres: Vec<String>,
    pub available_tags: Vec<String>,
    pub available_studios: Vec<String>,
    pub available_folders: Vec<MediaItem>,
    pub panel: BrowserPanel,
    pub panel_selected: usize,
    pub filter_year_input: String,
    pub filter_year_field: Option<YearField>,
}

impl Default for LibraryBrowserState {
    fn default() -> Self {
        Self {
            library_id: String::new(),
            library_name: String::new(),
            items: Vec::new(),
            total: 0,
            sort_by: ItemSort::DateAdded,
            sort_order: SortOrder::Desc,
            filter_section: FilterSection::Genre,
            filter_genre: None,
            filter_tag: None,
            filter_studio: None,
            filter_years: None,
            filter_folder: None,
            available_genres: Vec::new(),
            available_tags: Vec::new(),
            available_studios: Vec::new(),
            available_folders: Vec::new(),
            panel: BrowserPanel::None,
            panel_selected: 0,
            filter_year_input: String::new(),
            filter_year_field: None,
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
    LibraryBrowser,
    Favorites,
    ContinueWatching,
    LatestItems,
    AccountManager,
    Wizard,
    MpvPrompt,
}

impl AppState {
    pub async fn new(
        account: Option<crate::config::Account>,
    ) -> Result<Self> {
        let config = crate::config::load_config();

        let (client, server) = if let Some(acc) = account {
            let password = crate::crypto::decrypt(&acc.password_enc).unwrap_or_default();
            let client = EmbyClient::authenticate(&acc.server, &acc.username, &password).await?;
            (client, acc.server)
        } else {
            (EmbyClient::new(String::new(), String::new()), String::new())
        };

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
            loading_msg: String::new(),
            selected: 0,
            stack: Vec::new(),
            status_msg: None,
            searching: false,
            search_context: SearchContext::ServerWide,
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
            mpv_rx: None,
            mpv_output: Vec::new(),
            mpv_output_scroll: 0,
            mpv_position: 0.0,
            mpv_duration: 0.0,
            play_session_id: String::new(),
            playback_started_at: None,
            bg_tx: None,
            settings_state: SettingsState::default(),
            config,
            library_browser_state: LibraryBrowserState::default(),
            favorites: Vec::new(),
            total_favorites: 0,
            following_updates: Vec::new(),
            account_manager_state: AccountManagerState::default(),
            wizard_state: WizardState::default(),
            mpv_prompt_state: MpvPromptState::default(),
        })
    }

    // --- Series Navigation ---
    pub fn open_source_select(&mut self, item: &MediaItem, sources: Vec<MediaSource>) {
        self.source_state = SourceState {
            item: Some(item.clone()),
            sources,
        };
        self.navigate_to(View::SourceSelect);
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
            self.status_msg = Some(Message::info(format!("Loading {}...", season.name)));
            if let Ok(eps) = self.client.get_season_episodes(&series_id, &season.id).await {
                self.series_state.episodes = eps;
                self.series_state.selected_episode = 0;
                self.series_state.section = SeriesSection::Episodes;
                self.selected = 0;
            }
            self.loading = false;
            self.status_msg = None;
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

    // --- Navigation ---
    pub fn navigate_to(&mut self, view: View) {
        self.stack.push(StackEntry {
            items: self.items.clone(),
            folder_id: self.current_folder_id.clone(),
            view: self.view.clone(),
            selected: self.selected,
            home_items: self.home_items.clone(),
            total_items: self.total_items,
        });
        if self.stack.len() > 50 {
            self.stack.remove(0);
        }
        self.view = view;
        self.selected = 0;
    }

    pub fn go_back(&mut self) {
        self.status_msg = None;
        if self.searching {
            self.cancel_search();
            return;
        }
        if let Some(prev) = self.stack.pop() {
            self.items = prev.items;
            self.current_folder_id = prev.folder_id;
            self.view = prev.view;
            self.selected = prev.selected;
            self.home_items = prev.home_items;
            self.total_items = prev.total_items;
        } else {
            self.view = View::Home;
            self.selected = 0;
        }
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

    pub fn page_down(&mut self) {
        let len = self.current_list_len();
        if len > 0 {
            self.selected = (self.selected + 20).min(len - 1);
        }
    }

    pub fn page_up(&mut self) {
        let len = self.current_list_len();
        if len > 0 {
            self.selected = self.selected.saturating_sub(20);
        }
    }

    fn following_items_count(&self) -> usize {
        let mut count = 0;
        for (_series_name, episodes) in &self.following_updates {
            if !episodes.is_empty() {
                count += 1;
                count += episodes.len().min(5);
            }
        }
        count
    }

    // --- Selection ---
    pub fn selected_item(&self) -> Option<&MediaItem> {
        match self.view {
            View::Home => {
                let offset = self.following_items_count();
                if self.selected < offset {
                    let mut idx = self.selected;
                    for (_name, episodes) in &self.following_updates {
                        if !episodes.is_empty() {
                            if idx == 0 { return None; }
                            idx -= 1;
                            let take = episodes.len().min(5);
                            if idx < take { return episodes.get(idx); }
                            idx -= take;
                        }
                    }
                    None
                } else {
                    self.home_items.get(self.selected - offset)
                }
            }
            View::ContinueWatching | View::LatestItems => self.home_items.get(self.selected),
            View::Libraries => {
                // Combined list: libraries + section headers + latest items
                let mut idx = self.selected;

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
            View::AccountManager => None,
            View::Wizard => None,
            View::MpvPrompt => None,
            View::LibraryBrowser => self.library_browser_state.items.get(self.selected),
            View::Favorites => self.favorites.get(self.selected),
        }
    }

    pub fn selected_library(&self) -> Option<&Library> {
        if self.view == View::Libraries {
            let idx = self.selected;
            // Libraries are at index 0..n (header is not selectable)
            return self.libraries.get(idx);
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
            && self.selected + 1 >= self.episodes.len()
    }

    pub fn should_load_more_items(&self) -> bool {
        self.view == View::Items
            && !self.loading
            && self.total_items > self.items.len()
            && self.selected + 1 >= self.items.len()
    }

    pub fn should_load_more_home_items(&self) -> bool {
        matches!(self.view, View::ContinueWatching | View::LatestItems)
            && !self.loading
            && self.total_items > self.home_items.len()
            && self.selected + 1 >= self.home_items.len()
    }

    pub async fn show_libraries(&mut self) {
        self.navigate_to(View::Libraries);
        if !self.is_libraries_cache_valid() || !self.is_latest_cache_valid() {
            self.loading = true;
            self.status_msg = Some(Message::info("Loading libraries..."));
        }
    }

    // --- Search ---
    pub fn start_search(&mut self, context: SearchContext) {
        self.searching = true;
        self.search_context = context;
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
        self.navigate_to(View::TrackSelect);
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

    // --- MPV Playback ---
    pub fn kill_mpv(&mut self) {
        if let Some(mut child) = self.mpv_child.take() {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("taskkill")
                    .args(["/T", "/F", "/PID", &child.id().to_string()])
                    .output();
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
        self.mpv_rx = None;
    }

    pub fn current_position_ticks(&self) -> i64 {
        if self.mpv_duration > 0.0 {
            (self.mpv_position * 10_000_000.0) as i64
        } else if let (Some(started), Some(runtime)) = (self.playback_started_at, self.playing_state.runtime_ticks) {
            let elapsed = started.elapsed().as_secs() as i64 * 10_000_000;
            let offset = self.playing_state.resume_position.unwrap_or(0);
            (offset + elapsed).min(runtime)
        } else {
            0
        }
    }

    pub fn stop_playback(&mut self) -> i64 {
        let position_ticks = self.current_position_ticks();
        self.kill_mpv();
        self.playing_state.playing = false;
        self.playing_state.resume_position = Some(position_ticks);
        self.playing_state.option_selected = 0;
        self.playback_started_at = None;
        self.mpv_position = 0.0;
        self.mpv_duration = 0.0;
        position_ticks
    }

    // --- Settings ---
    pub fn open_settings(&mut self) {
        let mpv = self.config.mpv_path.clone();
        let lang = self.config.language.clone();
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
            section: SettingsSection::Libraries,
            mpv_path: mpv,
            language: lang,
        };
        self.navigate_to(View::Settings);
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
        let mpv = self.settings_state.mpv_path.trim().to_string();
        self.config.mpv_path = if mpv.is_empty() { "mpv".to_string() } else { mpv };
        self.config.language = self.settings_state.language.clone();
        crate::i18n::init(&self.config.language);
        if let Err(e) = crate::config::save_config(&self.config) {
            self.status_msg = Some(Message::error(format!("Save error: {e}")));
        } else {
            self.status_msg = Some(Message::success("Settings saved"));
        }
        self.libraries.clear();
        self.libraries_fetched_at = None;
        self.library_latest.clear();
        self.library_latest_fetched_at = None;
        self.navigate_to(View::Libraries);
        self.loading = true;
    }

    pub fn settings_cancel(&mut self) {
        self.go_back();
    }

    pub fn settings_switch_section(&mut self) {
        self.settings_state.section = match self.settings_state.section {
            SettingsSection::Libraries => SettingsSection::MpvPath,
            SettingsSection::MpvPath => SettingsSection::Language,
            SettingsSection::Language => SettingsSection::Libraries,
        };
        self.settings_state.selected = 0;
    }

    pub fn settings_mpv_input(&mut self, c: char) {
        if self.settings_state.section == SettingsSection::MpvPath {
            self.settings_state.mpv_path.push(c);
        }
    }

    pub fn settings_mpv_backspace(&mut self) {
        if self.settings_state.section == SettingsSection::MpvPath {
            self.settings_state.mpv_path.pop();
        }
    }

    pub fn settings_toggle_language(&mut self) {
        if self.settings_state.section == SettingsSection::Language {
            self.settings_state.language = if self.settings_state.language == "zh" {
                "en".to_string()
            } else {
                "zh".to_string()
            };
        }
    }

    // --- Playing ---
    pub fn open_track_select_or_playing(&mut self, item: &MediaItem, source: &MediaSource) {
        let has_tracks = source.media_streams.iter().any(|s| s.stream_type == "Video" || s.stream_type == "Audio");
        if has_tracks {
            self.open_track_select(item, source);
        } else {
            let url = self.client.stream_url_for_source(item, source);
            let resume = item.resume_position_ticks();
            self.open_playing(&item.display_name(), &item.id, &source.id, item.runtime_ticks, &url, "", "", "", resume, Some(source.clone()), 0, 0, 0);
        }
    }

    pub fn open_playing(&mut self, item_name: &str, item_id: &str, media_source_id: &str, runtime_ticks: Option<i64>, url: &str, video: &str, audio: &str, subtitle: &str, resume_ticks: Option<i64>, media_source: Option<crate::emby::MediaSource>, selected_video: usize, selected_audio: usize, selected_subtitle: usize) {
        self.playing_state = PlayingState {
            item_name: item_name.to_string(),
            item_id: item_id.to_string(),
            media_source_id: media_source_id.to_string(),
            runtime_ticks,
            url: url.to_string(),
            video_track: video.to_string(),
            audio_track: audio.to_string(),
            subtitle_track: subtitle.to_string(),
            selected_video,
            selected_audio,
            selected_subtitle,
            media_source,
            resume_position: resume_ticks,
            option_selected: 0,
            playing: false,
        };
        self.play_session_id = uuid::Uuid::new_v4().to_string().replace('-', "");
        self.navigate_to(View::Playing);
    }

    // --- Library Browser ---
    pub fn open_library_browser(&mut self, library_id: String, library_name: String) {
        self.library_browser_state = LibraryBrowserState {
            library_id,
            library_name,
            ..Default::default()
        };
        self.navigate_to(View::LibraryBrowser);
    }

    pub fn library_browser_sort_label(&self) -> &str {
        match self.library_browser_state.sort_by {
            ItemSort::Name => "Name",
            ItemSort::Year => "Year",
            ItemSort::Rating => "Rating",
            ItemSort::DateAdded => "Date Added",
        }
    }

    pub fn library_browser_order_label(&self) -> &str {
        match self.library_browser_state.sort_order {
            SortOrder::Asc => "↑",
            SortOrder::Desc => "↓",
        }
    }

    pub fn library_browser_open_sort_panel(&mut self) {
        self.library_browser_state.panel = BrowserPanel::Sort;
        self.library_browser_state.panel_selected = match self.library_browser_state.sort_by {
            ItemSort::Name => 0,
            ItemSort::Year => 1,
            ItemSort::Rating => 2,
            ItemSort::DateAdded => 3,
        };
    }

    pub fn library_browser_open_filter_panel(&mut self) {
        self.library_browser_state.panel = BrowserPanel::Filter;
        self.library_browser_state.panel_selected = 0;
        self.library_browser_state.filter_year_field = None;
    }

    pub fn library_browser_filter_section_next(&mut self) {
        let bs = &mut self.library_browser_state;
        bs.filter_section = match bs.filter_section {
            FilterSection::Genre => FilterSection::Tag,
            FilterSection::Tag => FilterSection::Studio,
            FilterSection::Studio => FilterSection::Year,
            FilterSection::Year => FilterSection::Folder,
            FilterSection::Folder => FilterSection::Genre,
        };
        bs.panel_selected = 0;
        bs.filter_year_field = None;
        bs.filter_year_input.clear();
    }

    pub fn library_browser_filter_section_prev(&mut self) {
        let bs = &mut self.library_browser_state;
        bs.filter_section = match bs.filter_section {
            FilterSection::Genre => FilterSection::Folder,
            FilterSection::Tag => FilterSection::Genre,
            FilterSection::Studio => FilterSection::Tag,
            FilterSection::Year => FilterSection::Studio,
            FilterSection::Folder => FilterSection::Year,
        };
        bs.panel_selected = 0;
        bs.filter_year_field = None;
        bs.filter_year_input.clear();
    }

    pub fn library_browser_close_panel(&mut self) {
        self.library_browser_state.panel = BrowserPanel::None;
        self.library_browser_state.filter_year_field = None;
        self.library_browser_state.filter_year_input.clear();
    }

    pub fn library_browser_select_sort(&mut self) {
        let bs = &mut self.library_browser_state;
        match bs.panel_selected {
            0 => bs.sort_by = ItemSort::Name,
            1 => bs.sort_by = ItemSort::Year,
            2 => bs.sort_by = ItemSort::Rating,
            3 => bs.sort_by = ItemSort::DateAdded,
            4 => {
                bs.sort_order = match bs.sort_order {
                    SortOrder::Asc => SortOrder::Desc,
                    SortOrder::Desc => SortOrder::Asc,
                };
            }
            _ => {}
        }
        bs.panel = BrowserPanel::None;
    }

    pub fn library_browser_panel_next(&mut self) {
        let bs = &mut self.library_browser_state;
        let len = match bs.panel {
            BrowserPanel::Sort => 5,
            BrowserPanel::Filter => {
                if bs.filter_year_field.is_some() {
                    2
                } else {
                    match bs.filter_section {
                        FilterSection::Genre => bs.available_genres.len() + 1,
                        FilterSection::Tag => bs.available_tags.len() + 1,
                        FilterSection::Studio => bs.available_studios.len() + 1,
                        FilterSection::Year => 1,
                        FilterSection::Folder => bs.available_folders.len(),
                    }
                }
            }
            BrowserPanel::None => 0,
        };
        if len > 0 {
            bs.panel_selected = (bs.panel_selected + 1) % len;
        }
    }

    pub fn library_browser_panel_prev(&mut self) {
        let bs = &mut self.library_browser_state;
        let len = match bs.panel {
            BrowserPanel::Sort => 5,
            BrowserPanel::Filter => {
                if bs.filter_year_field.is_some() {
                    2
                } else {
                    match bs.filter_section {
                        FilterSection::Genre => bs.available_genres.len() + 1,
                        FilterSection::Tag => bs.available_tags.len() + 1,
                        FilterSection::Studio => bs.available_studios.len() + 1,
                        FilterSection::Year => 1,
                        FilterSection::Folder => bs.available_folders.len(),
                    }
                }
            }
            BrowserPanel::None => 0,
        };
        if len > 0 {
            bs.panel_selected = (bs.panel_selected + len - 1) % len;
        }
    }

    pub fn library_browser_filter_select(&mut self) {
        let bs = &mut self.library_browser_state;

        match bs.filter_section {
            FilterSection::Genre => {
                if bs.panel_selected < bs.available_genres.len() {
                    if let Some(genre) = bs.available_genres.get(bs.panel_selected).cloned() {
                        // Toggle selection
                        if bs.filter_genre.as_ref() == Some(&genre) {
                            bs.filter_genre = None;
                        } else {
                            bs.filter_genre = Some(genre);
                        }
                    }
                } else {
                    // Move to next section
                    bs.filter_section = FilterSection::Tag;
                    bs.panel_selected = 0;
                    return;
                }
            }
            FilterSection::Tag => {
                if bs.panel_selected < bs.available_tags.len() {
                    if let Some(tag) = bs.available_tags.get(bs.panel_selected).cloned() {
                        if bs.filter_tag.as_ref() == Some(&tag) {
                            bs.filter_tag = None;
                        } else {
                            bs.filter_tag = Some(tag);
                        }
                    }
                } else {
                    bs.filter_section = FilterSection::Studio;
                    bs.panel_selected = 0;
                    return;
                }
            }
            FilterSection::Studio => {
                if bs.panel_selected < bs.available_studios.len() {
                    if let Some(studio) = bs.available_studios.get(bs.panel_selected).cloned() {
                        if bs.filter_studio.as_ref() == Some(&studio) {
                            bs.filter_studio = None;
                        } else {
                            bs.filter_studio = Some(studio);
                        }
                    }
                } else {
                    bs.filter_section = FilterSection::Year;
                    bs.panel_selected = 0;
                    return;
                }
            }
            FilterSection::Year => {
                bs.filter_year_field = Some(YearField::Start);
                bs.filter_year_input = bs.filter_years
                    .map(|(s, _)| s.to_string())
                    .unwrap_or_default();
                bs.panel_selected = 0;
                return;
            }
            FilterSection::Folder => {
                if bs.panel_selected < bs.available_folders.len() {
                    if let Some(folder_id) = bs.available_folders.get(bs.panel_selected).map(|f| f.id.clone()) {
                        if bs.filter_folder.as_ref() == Some(&folder_id) {
                            bs.filter_folder = None;
                        } else {
                            bs.filter_folder = Some(folder_id);
                        }
                    }
                }
            }
        }
        // Apply and close panel after selection
        bs.panel = BrowserPanel::None;
    }

    pub fn library_browser_year_input(&mut self, c: char) {
        let bs = &mut self.library_browser_state;
        if bs.filter_year_field.is_some() {
            bs.filter_year_input.push(c);
        }
    }

    pub fn library_browser_year_backspace(&mut self) {
        let bs = &mut self.library_browser_state;
        if bs.filter_year_field.is_some() {
            bs.filter_year_input.pop();
        }
    }

    pub fn library_browser_year_confirm(&mut self) {
        let bs = &mut self.library_browser_state;
        let year: u32 = bs.filter_year_input.parse().unwrap_or(0);

        match bs.filter_year_field {
            Some(YearField::Start) => {
                let end = bs.filter_years.map(|(_, e)| e).unwrap_or(year);
                bs.filter_years = Some((year, end));
                bs.filter_year_field = Some(YearField::End);
                bs.filter_year_input = end.to_string();
            }
            Some(YearField::End) => {
                if let Some((start, _)) = bs.filter_years {
                    bs.filter_years = Some((start, year));
                }
                bs.filter_year_field = None;
                bs.filter_year_input.clear();
                bs.panel = BrowserPanel::None;
            }
            None => {}
        }
    }

    pub fn library_browser_clear_filters(&mut self) {
        let bs = &mut self.library_browser_state;
        bs.filter_genre = None;
        bs.filter_tag = None;
        bs.filter_studio = None;
        bs.filter_years = None;
        bs.filter_folder = None;
    }

    pub fn library_browser_sort_by_str(&self) -> &str {
        match self.library_browser_state.sort_by {
            ItemSort::Name => "SortName",
            ItemSort::Year => "ProductionYear",
            ItemSort::Rating => "CommunityRating",
            ItemSort::DateAdded => "DateCreated",
        }
    }

    pub fn library_browser_sort_order_str(&self) -> &str {
        match self.library_browser_state.sort_order {
            SortOrder::Asc => "Ascending",
            SortOrder::Desc => "Descending",
        }
    }

    pub fn toggle_follow(&mut self, series_id: &str) {
        if self.config.following_series.iter().any(|id| id == series_id) {
            self.config.following_series.retain(|id| id != series_id);
            self.status_msg = Some(Message::info("Removed from following"));
        } else {
            self.config.following_series.push(series_id.to_string());
            self.status_msg = Some(Message::info("Added to following"));
        }
        let _ = crate::config::save_config(&self.config);
    }

    pub fn open_favorites(&mut self) {
        self.navigate_to(View::Favorites);
    }

    pub fn open_continue_watching(&mut self) {
        self.navigate_to(View::ContinueWatching);
    }

    pub fn open_latest_items(&mut self) {
        self.navigate_to(View::LatestItems);
    }

    // --- Account Manager ---
    pub fn open_account_manager(&mut self) {
        let accounts_cfg = crate::config::load_accounts();
        self.account_manager_state = AccountManagerState {
            accounts: accounts_cfg.accounts,
            last_account_id: accounts_cfg.last_account_id,
            selected: 0,
            action: AccountManagerAction::View,
            ..Default::default()
        };
        self.navigate_to(View::AccountManager);
    }

    pub fn account_manager_select_next(&mut self) {
        let len = match self.account_manager_state.action {
            AccountManagerAction::View => self.account_manager_state.accounts.len() + 1,
            AccountManagerAction::Add | AccountManagerAction::Edit(_) => 4,
            AccountManagerAction::Delete(_) => 2,
        };
        if len > 0 {
            self.account_manager_state.selected = (self.account_manager_state.selected + 1) % len;
        }
    }

    pub fn account_manager_select_prev(&mut self) {
        let len = match self.account_manager_state.action {
            AccountManagerAction::View => self.account_manager_state.accounts.len() + 1,
            AccountManagerAction::Add | AccountManagerAction::Edit(_) => 4,
            AccountManagerAction::Delete(_) => 2,
        };
        if len > 0 {
            self.account_manager_state.selected = (self.account_manager_state.selected + len - 1) % len;
        }
    }

    pub fn account_manager_input(&mut self, c: char) {
        let field = &self.account_manager_state.input_field;
        match field {
            AccountInputField::Label => self.account_manager_state.input_label.push(c),
            AccountInputField::Server => self.account_manager_state.input_server.push(c),
            AccountInputField::Username => self.account_manager_state.input_username.push(c),
            AccountInputField::Password => self.account_manager_state.input_password.push(c),
        }
    }

    pub fn account_manager_backspace(&mut self) {
        let field = &self.account_manager_state.input_field;
        match field {
            AccountInputField::Label => { self.account_manager_state.input_label.pop(); }
            AccountInputField::Server => { self.account_manager_state.input_server.pop(); }
            AccountInputField::Username => { self.account_manager_state.input_username.pop(); }
            AccountInputField::Password => { self.account_manager_state.input_password.pop(); }
        }
    }

    pub fn account_manager_next_field(&mut self) {
        self.account_manager_state.input_field = match self.account_manager_state.input_field {
            AccountInputField::Label => AccountInputField::Server,
            AccountInputField::Server => AccountInputField::Username,
            AccountInputField::Username => AccountInputField::Password,
            AccountInputField::Password => AccountInputField::Label,
        };
        self.account_manager_state.selected = match self.account_manager_state.input_field {
            AccountInputField::Label => 0,
            AccountInputField::Server => 1,
            AccountInputField::Username => 2,
            AccountInputField::Password => 3,
        };
    }

    // --- Wizard ---
    pub fn open_wizard(&mut self) {
        let mpv = crate::mpv::find_mpv().unwrap_or_else(|| "mpv".to_string());
        self.wizard_state = WizardState {
            mpv_path: mpv,
            ..Default::default()
        };
        self.view = View::Wizard;
    }

    pub fn wizard_input(&mut self, c: char) {
        match self.wizard_state.step {
            WizardField::Language => {} // Language uses toggle, not text input
            WizardField::Server => self.wizard_state.server.push(c),
            WizardField::Username => self.wizard_state.username.push(c),
            WizardField::Password => self.wizard_state.password.push(c),
            WizardField::MpvPath => self.wizard_state.mpv_path.push(c),
        }
    }

    pub fn wizard_backspace(&mut self) {
        match self.wizard_state.step {
            WizardField::Language => {}
            WizardField::Server => { self.wizard_state.server.pop(); }
            WizardField::Username => { self.wizard_state.username.pop(); }
            WizardField::Password => { self.wizard_state.password.pop(); }
            WizardField::MpvPath => { self.wizard_state.mpv_path.pop(); }
        }
    }

    pub fn wizard_toggle_language(&mut self) {
        self.wizard_state.language = if self.wizard_state.language == "zh" {
            "en".to_string()
        } else {
            "zh".to_string()
        };
    }

    pub fn wizard_next(&mut self) -> WizardAction {
        match self.wizard_state.step {
            WizardField::Language => {
                // Apply language immediately for UI updates
                crate::i18n::init(&self.wizard_state.language);
                self.wizard_state.step = WizardField::Server;
                self.wizard_state.status_msg = None;
                WizardAction::None
            }
            WizardField::Server => {
                if self.wizard_state.server.trim().is_empty() {
                    self.wizard_state.status_msg = Some(crate::i18n::t("status.server_required").to_string());
                    return WizardAction::None;
                }
                self.wizard_state.step = WizardField::Username;
                self.wizard_state.status_msg = None;
                WizardAction::None
            }
            WizardField::Username => {
                if self.wizard_state.username.trim().is_empty() {
                    self.wizard_state.status_msg = Some(crate::i18n::t("status.username_required").to_string());
                    return WizardAction::None;
                }
                self.wizard_state.step = WizardField::Password;
                self.wizard_state.status_msg = None;
                WizardAction::None
            }
            WizardField::Password => {
                if self.wizard_state.password.is_empty() {
                    self.wizard_state.status_msg = Some(crate::i18n::t("status.password_required").to_string());
                    return WizardAction::None;
                }
                self.wizard_state.step = WizardField::MpvPath;
                self.wizard_state.status_msg = None;
                WizardAction::None
            }
            WizardField::MpvPath => {
                WizardAction::FinishWizard
            }
        }
    }

    // --- MPV Prompt ---
    pub fn open_mpv_prompt(&mut self, url: &str, _video: &str, _audio: &str, _subtitle: &str, resume: Option<i64>) {
        self.mpv_prompt_state = MpvPromptState {
            mpv_path: self.config.mpv_path.clone(),
            url: url.to_string(),
            resume_position: resume,
        };
        self.view = View::MpvPrompt;
    }

    pub fn mpv_prompt_input(&mut self, c: char) {
        self.mpv_prompt_state.mpv_path.push(c);
    }

    pub fn mpv_prompt_backspace(&mut self) {
        self.mpv_prompt_state.mpv_path.pop();
    }

    fn current_list_len(&self) -> usize {
        match self.view {
            View::Home => self.following_items_count() + self.home_items.len(),
            View::ContinueWatching | View::LatestItems => self.home_items.len(),
            View::Libraries => {
                // Libraries + latest items (header is not selectable)
                let mut count = self.libraries.len();
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
            View::LibraryBrowser => self.library_browser_state.items.len(),
            View::Favorites => self.favorites.len(),
            View::AccountManager => {
                let ams = &self.account_manager_state;
                match ams.action {
                    AccountManagerAction::View => ams.accounts.len() + 1,
                    AccountManagerAction::Add | AccountManagerAction::Edit(_) => 4,
                    AccountManagerAction::Delete(_) => 2,
                }
            }
            View::Wizard => 0,
            View::MpvPrompt => 0,
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        self.kill_mpv();
    }
}
