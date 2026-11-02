use super::form::FormField;
use common::desktop_file::DesktopFile;
use gtk::{Widget, glib::object::Cast, prelude::WidgetExt};
use libadwaita::SwitchRow;
use std::{
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};
use tracing::debug;

pub struct IsolateRow {
    pub switch_row: SwitchRow,
    desktop_file: Rc<DesktopFile>,
}

impl IsolateRow {
    pub fn new(desktop_file: &Rc<DesktopFile>) -> Rc<Self> {
        let switch_row = SwitchRow::builder()
            .title(t!("web_apps.web_app_view.isolate.title"))
            .subtitle(t!("web_apps.web_app_view.isolate.subtitle"))
            .tooltip_text(t!("web_apps.web_app_view.isolate.disabled"))
            .build();

        Rc::new(Self {
            switch_row,
            desktop_file: desktop_file.clone(),
        })
    }

    fn on_init(self: &Rc<Self>, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        self.set_from_desktop_file();

        let self_clone = self.clone();
        let web_app_view_clone = web_app_view.clone();

        self.switch_row.connect_active_notify(move |_| {
            self_clone.on_active(&web_app_view_clone);
        });
    }

    fn on_active(&self, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        self.desktop_file.set_isolated(self.switch_row.is_active());

        self.on_isolation_change(web_app_view);
    }

    fn on_isolation_change(&self, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        let is_isolated = self.switch_row.is_active();
        let old_profile_path = self.desktop_file.get_profile_path().unwrap_or_default();
        let browser_can_isolate = self
            .desktop_file
            .get_browser()
            .is_some_and(|browser| browser.can_isolate);

        let new_profile_path = if is_isolated && browser_can_isolate {
            match self.desktop_file.build_profile_path() {
                Err(error) => {
                    web_app_view.error_message("Could not set isolation", Some(&error));
                    return;
                }
                Ok(profile) => profile,
            }
        } else {
            PathBuf::default()
        };

        if old_profile_path != new_profile_path && Path::new(&old_profile_path).is_dir() {
            debug!(
                path = old_profile_path.display().to_string(),
                "Deleting profile"
            );
            let _ = fs::remove_dir_all(old_profile_path);
        }

        self.desktop_file.set_profile_path(&new_profile_path);
    }

    fn on_reset(&self, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        debug!("Resetting isolate_row");

        self.set_from_desktop_file();
        self.on_isolation_change(web_app_view);
    }

    fn set_from_desktop_file(&self) {
        let browser = self.desktop_file.get_browser();
        let browser_can_isolate = browser.as_ref().is_some_and(|browser| browser.can_isolate);
        let has_isolated = self.desktop_file.get_isolated();
        let is_isolated = has_isolated.unwrap_or(false);

        self.switch_row.set_active(is_isolated);
        self.switch_row.set_sensitive(browser_can_isolate);
        self.switch_row.set_has_tooltip(!browser_can_isolate);

        if let Some(browser) = browser
            && !browser.can_isolate
            && is_isolated
        {
            debug!("Found desktop file with isolate on a browser that is incapable");
            self.switch_row.set_active(false);
        }

        // SwitchRow has already a setting on load, so sync this if empty
        if has_isolated.is_none() {
            self.desktop_file.set_isolated(self.switch_row.is_active());
        }
    }
}

impl FormField for IsolateRow {
    fn init(self: Rc<Self>, web_app_view: Rc<dyn super::WebAppViewShared>) {
        self.on_init(&web_app_view);
    }

    fn reset(&self, web_app_view: Rc<dyn super::WebAppViewShared>) {
        self.on_reset(&web_app_view);
    }

    fn is_valid(&self, _do_dirty_check: bool, _apply_to_field: bool) -> bool {
        true
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn form_widget(&self) -> Widget {
        self.switch_row.clone().upcast()
    }
}
