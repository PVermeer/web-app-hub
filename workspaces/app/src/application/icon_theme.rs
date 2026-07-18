use common::browsers::BrowserIconTheme;
use gtk::{IconTheme, gdk, glib::object::IsA};
use std::path::Path;

pub struct AppIconTheme {
    gtk_icon_theme: IconTheme,
}
impl BrowserIconTheme for AppIconTheme {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            gtk_icon_theme: IconTheme::new(),
        }
    }

    fn has_icon(&self, icon_name: &str) -> bool {
        self.gtk_icon_theme.has_icon(icon_name)
    }

    fn add_search_path(&self, path: &Path) {
        self.gtk_icon_theme.add_search_path(path);
    }
}
impl AppIconTheme {
    pub fn for_display(display: &impl IsA<gdk::Display>) -> Self
    where
        Self: Sized,
    {
        Self {
            gtk_icon_theme: IconTheme::for_display(display),
        }
    }

    pub fn theme_name(&self) -> String {
        self.gtk_icon_theme.theme_name().to_string()
    }
}
