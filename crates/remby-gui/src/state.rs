use remby_core::config::RembyConfig;
use remby_core::emby::{EmbyClient, Library, MediaItem, MediaSource, MediaStream};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    Login,
    Home,
    Libraries,
    LibraryBrowser,
    Favorites,
    SeriesInfo,
    Player,
    Settings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Success,
    Error,
    Loading,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Libraries,
    MpvPath,
    Language,
    Theme,
    TrackPreferences,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeriesSection {
    Seasons,
    Episodes,
    Similar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Name,
    Year,
    Rating,
    DateAdded,
}

impl SortField {
    pub fn label(&self) -> &'static str {
        match self {
            SortField::Name => "Name",
            SortField::Year => "Year",
            SortField::Rating => "Rating",
            SortField::DateAdded => "Date Added",
        }
    }

    pub fn emby_key(&self) -> &'static str {
        match self {
            SortField::Name => "SortName",
            SortField::Year => "ProductionYear",
            SortField::Rating => "CommunityRating",
            SortField::DateAdded => "DateCreated",
        }
    }

    pub fn all() -> &'static [SortField] {
        &[SortField::Name, SortField::Year, SortField::Rating, SortField::DateAdded]
    }

    pub fn cycle(&self) -> SortField {
        let fields = Self::all();
        let idx = fields.iter().position(|f| *f == *self).unwrap_or(0);
        fields[(idx + 1) % fields.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn label(&self) -> &'static str {
        match self {
            SortOrder::Ascending => "Asc",
            SortOrder::Descending => "Desc",
        }
    }

    pub fn emby_key(&self) -> &'static str {
        match self {
            SortOrder::Ascending => "Ascending",
            SortOrder::Descending => "Descending",
        }
    }

    pub fn toggle(&self) -> SortOrder {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserFilters {
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub studios: Vec<String>,
    pub year_min: Option<u32>,
    pub year_max: Option<u32>,
}

impl Default for BrowserFilters {
    fn default() -> Self {
        Self {
            genres: Vec::new(),
            tags: Vec::new(),
            studios: Vec::new(),
            year_min: None,
            year_max: None,
        }
    }
}

impl BrowserFilters {
    pub fn is_empty(&self) -> bool {
        self.genres.is_empty()
            && self.tags.is_empty()
            && self.studios.is_empty()
            && self.year_min.is_none()
            && self.year_max.is_none()
    }
}

pub struct GuiState {
    // Client
    pub client: Option<EmbyClient>,
    pub server: String,

    // Navigation
    pub view: View,
    pub view_stack: Vec<View>,

    // UI state
    pub loading: bool,
    pub loading_msg: String,
    pub status_msg: String,
    pub status_kind: StatusKind,

    // Home data
    pub continue_watching: Vec<MediaItem>,
    pub latest_items: Vec<MediaItem>,
    pub following_updates: Vec<MediaItem>,

    // Libraries
    pub libraries: Vec<Library>,

    // Library browser
    pub browser_library_id: String,
    pub browser_library_name: String,
    pub browser_items: Vec<MediaItem>,
    pub browser_total: usize,
    pub browser_sort_field: SortField,
    pub browser_sort_order: SortOrder,
    pub browser_show_filters: bool,
    pub browser_filters: BrowserFilters,
    pub browser_available_genres: Vec<String>,
    pub browser_available_tags: Vec<String>,
    pub browser_available_studios: Vec<String>,
    pub browser_search_term: String,
    pub browser_loading_more: bool,

    // Favorites
    pub favorites: Vec<MediaItem>,

    // Series
    pub series_item: Option<MediaItem>,
    pub series_seasons: Vec<MediaItem>,
    pub series_episodes: Vec<MediaItem>,
    pub series_similar: Vec<MediaItem>,
    pub series_section: SeriesSection,

    // Player
    pub playing_item: Option<MediaItem>,
    pub playing_url: String,
    pub playing: bool,
    pub player_sources: Vec<MediaSource>,
    pub player_selected_source_idx: Option<usize>,
    pub player_video_tracks: Vec<MediaStream>,
    pub player_audio_tracks: Vec<MediaStream>,
    pub player_subtitle_tracks: Vec<MediaStream>,
    pub player_selected_video: Option<i32>,
    pub player_selected_audio: Option<i32>,
    pub player_selected_subtitle: Option<i32>,
    pub player_position: f64,
    pub player_duration: f64,
    pub player_logs: Vec<String>,
    pub player_loading: bool,
    pub player_started: bool,
    pub player_play_session_id: String,
    pub player_media_source_id: String,

    // Config
    pub config: RembyConfig,
    pub settings_tab: SettingsTab,

    // Login
    pub login_server: String,
    pub login_username: String,
    pub login_password: String,
    pub login_error: String,
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            client: None,
            server: String::new(),

            view: View::Login,
            view_stack: Vec::new(),

            loading: false,
            loading_msg: String::new(),
            status_msg: String::new(),
            status_kind: StatusKind::Info,

            continue_watching: Vec::new(),
            latest_items: Vec::new(),
            following_updates: Vec::new(),

            libraries: Vec::new(),

            browser_library_id: String::new(),
            browser_library_name: String::new(),
            browser_items: Vec::new(),
            browser_total: 0,
            browser_sort_field: SortField::Name,
            browser_sort_order: SortOrder::Ascending,
            browser_show_filters: false,
            browser_filters: BrowserFilters::default(),
            browser_available_genres: Vec::new(),
            browser_available_tags: Vec::new(),
            browser_available_studios: Vec::new(),
            browser_search_term: String::new(),
            browser_loading_more: false,

            favorites: Vec::new(),

            series_item: None,
            series_seasons: Vec::new(),
            series_episodes: Vec::new(),
            series_similar: Vec::new(),
            series_section: SeriesSection::Seasons,

            playing_item: None,
            playing_url: String::new(),
            playing: false,
            player_sources: Vec::new(),
            player_selected_source_idx: None,
            player_video_tracks: Vec::new(),
            player_audio_tracks: Vec::new(),
            player_subtitle_tracks: Vec::new(),
            player_selected_video: None,
            player_selected_audio: None,
            player_selected_subtitle: None,
            player_position: 0.0,
            player_duration: 0.0,
            player_logs: Vec::new(),
            player_loading: false,
            player_started: false,
            player_play_session_id: String::new(),
            player_media_source_id: String::new(),

            config: RembyConfig::default(),
            settings_tab: SettingsTab::Libraries,

            login_server: String::new(),
            login_username: String::new(),
            login_password: String::new(),
            login_error: String::new(),
        }
    }

    pub fn navigate(&mut self, view: View) {
        self.view_stack.push(self.view.clone());
        self.view = view;
    }

    pub fn go_back(&mut self) {
        if let Some(prev) = self.view_stack.pop() {
            self.view = prev;
        }
    }
}
