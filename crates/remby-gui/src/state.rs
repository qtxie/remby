use remby_core::config::RembyConfig;
use remby_core::emby::{EmbyClient, Library, MediaItem};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeriesSection {
    Seasons,
    Episodes,
    Similar,
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

    // Config
    pub config: RembyConfig,

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

            favorites: Vec::new(),

            series_item: None,
            series_seasons: Vec::new(),
            series_episodes: Vec::new(),
            series_similar: Vec::new(),
            series_section: SeriesSection::Seasons,

            playing_item: None,
            playing_url: String::new(),
            playing: false,

            config: RembyConfig::default(),

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
