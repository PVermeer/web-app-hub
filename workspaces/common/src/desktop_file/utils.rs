use std::path::{Path, PathBuf};

pub fn map_to_string_option(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

pub fn map_to_bool_option(value: &str) -> Option<bool> {
    if value.is_empty() {
        None
    } else {
        Some(value.eq("true"))
    }
}

pub fn map_to_path_option(value: &str) -> Option<PathBuf> {
    if value.is_empty() {
        None
    } else {
        Some(Path::new(value).to_path_buf())
    }
}
