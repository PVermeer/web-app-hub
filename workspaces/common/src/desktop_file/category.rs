use std::fmt::Display;

use gtk::Image;

/// <https://specifications.freedesktop.org/menu/latest/category-registry.html>
#[derive(Copy, Clone)]
pub enum Category {
    AudioVideo,
    Audio,
    Video,
    Development,
    Education,
    Game,
    Graphics,
    Network,
    Office,
    Science,
    Settings,
    System,
    Utility,
}
impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::AudioVideo => write!(f, "AudioVideo"),
            Self::Audio => write!(f, "AudioVideo;Audio"),
            Self::Video => write!(f, "AudioVideo;Video"),
            Self::Development => write!(f, "Development"),
            Self::Education => write!(f, "Education"),
            Self::Game => write!(f, "Game"),
            Self::Graphics => write!(f, "Graphics"),
            Self::Network => write!(f, "Network"),
            Self::Office => write!(f, "Office"),
            Self::Science => write!(f, "Science"),
            Self::Settings => write!(f, "Settings"),
            Self::System => write!(f, "System"),
            Self::Utility => write!(f, "Utility"),
        }
    }
}
impl Category {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_string_ui(&self) -> &str {
        match self {
            Self::AudioVideo => "Multimedia",
            Self::Audio => "Audio",
            Self::Video => "Video",
            Self::Development => "Development",
            Self::Education => "Education",
            Self::Game => "Game",
            Self::Graphics => "Graphics",
            Self::Network => "Network / Internet",
            Self::Office => "Office",
            Self::Science => "Science",
            Self::Settings => "Settings",
            Self::System => "System",
            Self::Utility => "Utility",
        }
    }

    pub fn get_all() -> [Category; 13] {
        let list: [Category; 13] = [
            Self::AudioVideo,
            Self::Audio,
            Self::Video,
            Self::Development,
            Self::Education,
            Self::Game,
            Self::Graphics,
            Self::Network,
            Self::Office,
            Self::Science,
            Self::Settings,
            Self::System,
            Self::Utility,
        ];

        list
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn get_icon(&self) -> Image {
        let icon_name = match self {
            Self::AudioVideo => "applications-multimedia-symbolic",
            Self::Audio => "audio-x-generic-symbolic",
            Self::Video => "video-x-generic-symbolic",
            Self::Development => "applications-engineering-symbolic",
            Self::Education => "emoji-symbols-symbolic",
            Self::Game => "applications-games-symbolic",
            Self::Graphics => "applications-graphics-symbolic",
            Self::Network => "web-browser-symbolic",
            Self::Office => "x-office-document-symbolic",
            Self::Science => "applications-science-symbolic",
            Self::Settings => "preferences-other-symbolic",
            Self::System => "preferences-system-symbolic",
            Self::Utility => "applications-utilities-symbolic",
        };

        Image::from_icon_name(icon_name)
    }
}
