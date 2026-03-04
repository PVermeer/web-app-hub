mod web_app_view;

use super::NavPage;
use crate::application::{App, pages::PrefNavPage};
use common::{
    desktop_file::{DesktopFile, error::DesktopFileError},
    utils,
};
use gtk::{
    Button, Image,
    prelude::{ButtonExt, WidgetExt},
};
use libadwaita::{
    ActionRow, ButtonContent, NavigationPage, NavigationView, PreferencesGroup, PreferencesPage,
    StatusPage,
    prelude::{ActionRowExt, PreferencesGroupExt, PreferencesPageExt},
};
use std::{cell::RefCell, rc::Rc};
use tracing::{debug, error, info};
use web_app_view::WebAppView;

pub struct WebAppsPage {
    nav_page: NavigationPage,
    nav_row: ActionRow,
    nav_view: Rc<NavigationView>,
    prefs_page: PreferencesPage,
    app_section: RefCell<PreferencesGroup>,
}
impl NavPage for WebAppsPage {
    fn get_navpage(&self) -> &NavigationPage {
        &self.nav_page
    }

    fn get_nav_row(&self) -> Option<&ActionRow> {
        Some(&self.nav_row)
    }
}
impl WebAppsPage {
    pub fn new() -> Rc<Self> {
        let title = t!("web_apps.title");
        let icon = "preferences-desktop-apps-symbolic";
        let app_section = RefCell::new(PreferencesGroup::new());

        let PrefNavPage {
            nav_page,
            nav_row,
            nav_view,
            prefs_page,
            ..
        } = Self::build_nav_page(&title, icon).with_preference_navigation_view();

        Rc::new(Self {
            nav_page,
            nav_row,
            nav_view: Rc::new(nav_view),
            prefs_page,
            app_section,
        })
    }

    pub fn init(self: &Rc<Self>, app: &Rc<App>) {
        let app_section = self.clone().build_apps_section(app);
        self.prefs_page.add(&app_section);
        *self.app_section.borrow_mut() = app_section;

        let self_clone = self.clone();
        let app_clone = app.clone();

        self.nav_view
            .connect_popped(move |_, _| self_clone.reset_app_section(&app_clone));
    }

    fn build_apps_section(self: Rc<Self>, app: &Rc<App>) -> PreferencesGroup {
        let button_content = ButtonContent::builder()
            .label(t!("web_apps.button.new_app"))
            .icon_name("list-add-symbolic")
            .build();
        let new_app_button = Button::builder()
            .css_classes(["flat"])
            .child(&button_content)
            .build();

        let self_clone = self.clone();
        let app_clone = app.clone();

        new_app_button.connect_clicked(move |_| {
            let desktop_file = Rc::new(RefCell::new(DesktopFile::new(
                &app_clone.browser_configs,
                &app_clone.dirs,
            )));
            let app_page = WebAppView::new(&app_clone, &self_clone.nav_view, &desktop_file, true);
            app_page.init();

            let nav_page = app_page.get_navpage();
            let app_page_clone = app_page.clone();
            nav_page.connect_unrealize(move |_| {
                if app_page_clone.get_is_new() {
                    let _ = desktop_file.borrow().delete();
                }
            });

            self_clone.nav_view.push(nav_page);
        });

        let pref_group = PreferencesGroup::builder()
            .header_suffix(&new_app_button)
            .build();

        let (web_app_desktop_files, desktop_files_have_updated) =
            Self::get_owned_desktop_files(app);
        if web_app_desktop_files.is_empty() {
            let status_page = StatusPage::builder()
                .title(t!("web_apps.no_apps.title"))
                .description(t!("web_apps.no_apps.description"))
                .icon_name("system-search-symbolic")
                .build();

            pref_group.add(&status_page);
        } else {
            for (desktop_file, has_issue) in web_app_desktop_files {
                let web_app_row = self.clone().build_app_row(app, desktop_file, has_issue);
                pref_group.add(&web_app_row);
            }
        }

        if desktop_files_have_updated {
            app.on_app_update();
        }

        pref_group
    }

    fn build_app_row(
        self: Rc<Self>,
        app: &Rc<App>,
        desktop_file: DesktopFile,
        has_issue: bool,
    ) -> ActionRow {
        let app_name = desktop_file
            .get_name()
            .unwrap_or(t!("web_apps.no_name").to_string());
        let app_row = ActionRow::builder()
            .title(app_name)
            .activatable(true)
            .build();

        if has_issue {
            let issue_icon = Image::from_icon_name("dialog-warning-symbolic");
            issue_icon.set_css_classes(&["warning"]);
            issue_icon.set_margin_end(10);
            app_row.add_suffix(&issue_icon);
        }

        let app_icon = desktop_file.get_icon();
        let suffix = Image::from_icon_name("go-next-symbolic");

        app_row.add_prefix(&app_icon);
        app_row.add_suffix(&suffix);

        let app_clone = app.clone();
        let nav_view_clone = self.nav_view.clone();
        let desktop_file_rc = Rc::new(RefCell::new(desktop_file));

        app_row.connect_activated(move |_| {
            let app_page = WebAppView::new(&app_clone, &nav_view_clone, &desktop_file_rc, false);
            app_page.init();
            self.nav_view.push(app_page.get_navpage());
        });

        app_row
    }

    fn get_owned_desktop_files(app: &Rc<App>) -> (Vec<(DesktopFile, bool)>, bool) {
        debug!("Reading user desktop files");

        let mut owned_desktop_files = Vec::new();
        let applications_path = &app.dirs.user_applications;
        let mut app_has_updated = false;

        for file in utils::files::get_entries_in_dir(applications_path).unwrap_or_default() {
            let file_path = file.path();
            let file_path_string = file_path.to_string_lossy().to_string();

            if file_path
                .extension()
                .is_some_and(|extension| extension != "desktop")
            {
                debug!(path = file_path_string, "Not a desktop file");
                continue;
            }

            let Ok(is_owned) = DesktopFile::is_owned(&file_path) else {
                error!(path = file_path_string, "Failed to read desktop file");
                continue;
            };
            if !is_owned {
                debug!(path = file_path_string, "Desktop file not owned");
                continue;
            }

            let Ok(mut desktop_file) =
                DesktopFile::from_path(&file.path(), &app.browser_configs, &app.dirs)
            else {
                error!(path = file_path_string, "Failed to read desktop file");
                continue;
            };

            let file_name = desktop_file
                .get_path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            info!(file_name = &file_name, "Found desktop file");

            let mut is_updated = false;
            let mut has_error = false;
            match desktop_file.update() {
                Ok(has_updated) => is_updated = has_updated,
                Err(error) => {
                    match error {
                        DesktopFileError::ValidationError(error) => error!(
                            error = error.to_string(),
                            desktop_file = &file_name,
                            app = desktop_file.get_name(),
                            "Failed to validate after updating DesktopFile"
                        ),
                        DesktopFileError::Other(error) => error!(
                            error = error.to_string(),
                            desktop_file = &file_name,
                            app = desktop_file.get_name(),
                            "Failed to update DesktopFile"
                        ),
                    }
                    has_error = true;
                }
            }
            if is_updated {
                info!(file_name = &file_name, "Updated DesktopFile");
                app_has_updated = true;
            } else {
                info!(file_name = &file_name, "Validating DesktopFile");
                if desktop_file.validate().is_err() {
                    has_error = true;
                }
            }

            debug!(file_name = &file_name, "Checking paths");
            if desktop_file.check_paths().is_err() {
                has_error = true;
            }

            owned_desktop_files.push((desktop_file, has_error));
        }
        owned_desktop_files.sort_by_key(|(desktop_file, _)| {
            desktop_file.get_name().unwrap_or(char::MAX.to_string())
        });

        *app.has_created_apps.borrow_mut() = !owned_desktop_files.is_empty();

        (owned_desktop_files, app_has_updated)
    }

    fn reset_app_section(self: &Rc<Self>, app: &Rc<App>) {
        self.prefs_page.remove(&*self.app_section.borrow());
        *self.app_section.borrow_mut() = self.clone().build_apps_section(app);
        self.prefs_page.add(&*self.app_section.borrow());
    }
}
