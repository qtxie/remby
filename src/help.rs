pub struct Keybinding {
    pub keys: &'static str,
    pub description: &'static str,
}

pub fn bindings_for_view(view: &str) -> Vec<Keybinding> {
    match view {
        "Home" => vec![
            Keybinding { keys: "↑/↓", description: "Navigate" },
            Keybinding { keys: "Enter", description: "Open item" },
            Keybinding { keys: "/", description: "Search" },
            Keybinding { keys: "F", description: "Favorites" },
            Keybinding { keys: "u", description: "Accounts" },
            Keybinding { keys: "s", description: "Settings" },
            Keybinding { keys: "Ctrl+F", description: "Refresh" },
            Keybinding { keys: "Esc", description: "Back" },
            Keybinding { keys: "q", description: "Quit" },
        ],
        "Libraries" => vec![
            Keybinding { keys: "↑/↓", description: "Navigate" },
            Keybinding { keys: "Enter", description: "Open library" },
            Keybinding { keys: "s", description: "Settings" },
            Keybinding { keys: "q", description: "Quit" },
        ],
        "Items" => vec![
            Keybinding { keys: "↑/↓", description: "Navigate" },
            Keybinding { keys: "Enter", description: "Open item" },
            Keybinding { keys: "/", description: "Search" },
            Keybinding { keys: "f", description: "Follow series" },
            Keybinding { keys: "Esc", description: "Back" },
            Keybinding { keys: "q", description: "Quit" },
        ],
        "Episodes" => vec![
            Keybinding { keys: "↑/↓", description: "Navigate" },
            Keybinding { keys: "Enter", description: "Play episode" },
            Keybinding { keys: "←/→", description: "Switch season" },
            Keybinding { keys: "Esc", description: "Back" },
            Keybinding { keys: "q", description: "Quit" },
        ],
        "SeriesInfo" => vec![
            Keybinding { keys: "←/→", description: "Switch section" },
            Keybinding { keys: "↑/↓", description: "Navigate" },
            Keybinding { keys: "Enter", description: "Open item" },
            Keybinding { keys: "f", description: "Follow series" },
            Keybinding { keys: "e", description: "Episodes" },
            Keybinding { keys: "Esc", description: "Back" },
            Keybinding { keys: "q", description: "Quit" },
        ],
        "Playing" => vec![
            Keybinding { keys: "↑/↓", description: "Select option" },
            Keybinding { keys: "Enter", description: "Confirm" },
            Keybinding { keys: "Esc", description: "Back" },
            Keybinding { keys: "q", description: "Quit" },
        ],
        "LibraryBrowser" => vec![
            Keybinding { keys: "↑/↓", description: "Navigate" },
            Keybinding { keys: "Enter", description: "Open item" },
            Keybinding { keys: "/", description: "Search" },
            Keybinding { keys: "e", description: "Series info" },
            Keybinding { keys: "z", description: "Favorite" },
            Keybinding { keys: "Ctrl+S", description: "Sort" },
            Keybinding { keys: "Ctrl+F", description: "Filter" },
            Keybinding { keys: "Esc", description: "Back" },
            Keybinding { keys: "q", description: "Quit" },
        ],
        "Favorites" => vec![
            Keybinding { keys: "↑/↓", description: "Navigate" },
            Keybinding { keys: "Enter", description: "Open item" },
            Keybinding { keys: "/", description: "Search" },
            Keybinding { keys: "f", description: "Follow series" },
            Keybinding { keys: "z", description: "Unfavorite" },
            Keybinding { keys: "m", description: "Mark watched" },
            Keybinding { keys: "Esc", description: "Back" },
            Keybinding { keys: "q", description: "Quit" },
        ],
        "Settings" => vec![
            Keybinding { keys: "Tab", description: "Next section" },
            Keybinding { keys: "↑/↓", description: "Navigate" },
            Keybinding { keys: "←/→", description: "Toggle / switch" },
            Keybinding { keys: "Space", description: "Toggle item" },
            Keybinding { keys: "Shift+↑↓", description: "Move item" },
            Keybinding { keys: "Enter", description: "Save" },
            Keybinding { keys: "Esc", description: "Cancel" },
        ],
        _ => vec![],
    }
}

pub fn view_label(view: &str) -> &'static str {
    match view {
        "Home" => "Home",
        "Libraries" => "Libraries",
        "Items" => "Items",
        "Episodes" => "Episodes",
        "SeriesInfo" => "Series Info",
        "Playing" => "Playing",
        "LibraryBrowser" => "Library Browser",
        "Favorites" => "Favorites",
        "Settings" => "Settings",
        _ => "Help",
    }
}
