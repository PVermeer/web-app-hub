use crate::{
    config::{self},
    utils::OnceLockExt,
};
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum Key {
    Gwa,
    Version,
    Url,
    Id,
    BrowserId,
    Isolate,
    Maximize,
    Profile,
    Name,
    Exec,
    Icon,
    StartupWMClass,
    Categories,
    Comment,
}
impl Key {
    pub fn to_ui_string(&self) -> &str {
        match self {
            Self::Gwa => "Owned by Web App Hub",
            Self::Version => "Version",
            Self::Id => "App ID",
            Self::Url => "Website URL",
            Self::BrowserId => "Browser",
            Self::Isolate => "Isolation",
            Self::Maximize => "Maximize",
            Self::Profile => "Profile",
            Self::Name => "Web app name",
            Self::Exec => "Run command",
            Self::Icon => "Icon",
            Self::StartupWMClass => "WMClass",
            Self::Categories => "Categories",
            Self::Comment => "Comment",
        }
    }
}
impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let identifier = config::APP_NAME_SHORT.get_value().to_uppercase();

        match self {
            Self::Gwa => write!(f, "X-{}", &identifier),
            Self::Version => write!(f, "X-{}-VERSION", &identifier),
            Self::Id => write!(f, "X-{}-ID", &identifier),
            Self::Url => write!(f, "X-{}-URL", &identifier),
            Self::BrowserId => write!(f, "X-{}-BROWSER-ID", &identifier),
            Self::Isolate => write!(f, "X-{}-ISOLATE", &identifier),
            Self::Maximize => write!(f, "X-{}-MAXIMIZE", &identifier),
            Self::Profile => write!(f, "X-{}-PROFILE", &identifier),
            Self::Name => write!(f, "Name"),
            Self::Exec => write!(f, "Exec"),
            Self::Icon => write!(f, "Icon"),
            Self::StartupWMClass => write!(f, "StartupWMClass"),
            Self::Categories => write!(f, "Categories"),
            Self::Comment => write!(f, "Comment"),
        }
    }
}
