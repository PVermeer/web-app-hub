mod error_dialog;
mod pages;
mod window;

use anyhow::{Error, Result};
use common::{
    app_dirs::AppDirs,
    assets::{self},
    browsers::BrowserConfigs,
    cache_settings::CacheSettings,
    config::{self},
    fetch::Fetch,
    utils::{self, OnceLockExt},
};
use error_dialog::ErrorDialog;
use gtk::{IconTheme, Image, Settings, gdk};
use pages::{Page, Pages};
use std::{cell::RefCell, path::Path, rc::Rc};
use tracing::{debug, error};
use window::AppWindow;

pub struct App {
    pub cache_settings: RefCell<CacheSettings>,
    pub dirs: Rc<AppDirs>,
    pub browser_configs: Rc<BrowserConfigs>,
    pub error_dialog: ErrorDialog,
    adw_application: libadwaita::Application,
    icon_theme: Rc<IconTheme>,
    window: AppWindow,
    fetch: Fetch,
    pages: Pages,
    has_created_apps: RefCell<bool>,
}
impl App {
    pub fn new(adw_application: &libadwaita::Application) -> Rc<Self> {
        Rc::new({
            let icon_theme = Rc::new(IconTheme::for_display(
                &gdk::Display::default().expect("Failed to connect to display"),
            ));
            let app_dirs = AppDirs::new().expect("Failed to get all needed directories");
            let settings = Settings::default().expect("Failed to load gtk settings");
            let cache_settings = RefCell::new(CacheSettings::new(&app_dirs));
            let window = AppWindow::new(adw_application);
            let fetch = Fetch::new();
            let pages = Pages::new();
            let browsers = BrowserConfigs::new(&icon_theme, &app_dirs);
            let error_dialog = ErrorDialog::new();

            Self::set_theme_settings(&settings);

            Self {
                cache_settings,
                dirs: app_dirs,
                browser_configs: browsers,
                error_dialog,
                adw_application: adw_application.clone(),
                icon_theme,
                window,
                fetch,
                pages,
                has_created_apps: RefCell::new(false),
            }
        })
    }

    pub fn init(self: &Rc<Self>) {
        if let Err(error) = (|| -> Result<()> {
            debug!("Using icon theme: {}", self.icon_theme.theme_name());

            // Order matters!
            self.window.init(self);
            self.error_dialog.init(self);

            assets::init(&self.dirs)?;
            self.add_system_icon_paths();
            self.browser_configs.init();

            // Last
            self.pages.init(self);

            if *self.has_created_apps.borrow() {
                self.navigate(&Page::WebApps);
            } else {
                self.navigate(&Page::Home);
            }

            Ok(())
        })() {
            self.show_error(&error);
        }
    }

    pub fn add_icon_search_path(self: &Rc<Self>, path: &Path) {
        if !path.is_dir() {
            debug!("Not a valid icon path: {}", path.display());
            return;
        }

        debug!("Adding icon path to icon theme: {}", path.display());
        self.icon_theme.add_search_path(path);
    }

    #[allow(clippy::unused_self)]
    pub fn get_icon(self: &Rc<Self>) -> Image {
        Image::from_icon_name(config::APP_ID.get_value())
    }

    pub fn navigate(self: &Rc<Self>, page: &Page) {
        self.window.view.navigate(self, page);
    }

    pub fn show_error(self: &Rc<Self>, error: &Error) {
        error!("{error:?}");
        self.error_dialog.show(self, error);
    }

    pub fn close(self: &Rc<Self>) {
        self.window.close();
    }

    pub fn restart(mut self: Rc<Self>) {
        self.close();
        self.cache_settings.borrow_mut().reset();
        let new_self = Self::new(&self.adw_application);
        self = new_self;
        self.init();
    }

    pub fn on_app_update(self: &Rc<Self>) {
        self.window.view.on_app_update();
    }

    fn add_system_icon_paths(self: &Rc<Self>) {
        if utils::env::is_flatpak_container() {
            for path in &self.dirs.system_icons {
                debug!(path = %path.display(), "Adding system icon path");
                self.add_icon_search_path(path);
            }
        }
    }

    fn set_theme_settings(settings: &Settings) {
        settings.set_gtk_icon_theme_name(Some("Adwaita"));
    }
}
