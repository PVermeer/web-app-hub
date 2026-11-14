#![allow(
    clippy::expect_used, // TODO This should not be allowed
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::new_without_default
)]

pub mod app_dirs;
pub mod assets;
pub mod browsers;
pub mod cache_settings;
pub mod config;
pub mod desktop_file;
pub mod fetch;
pub mod url;
pub mod utils;
