use super::{WebAppView, form::FormField};
use common::desktop_file::DesktopFile;
use gtk::{glib::JoinHandle, prelude::WidgetExt};
use libadwaita::SwitchRow;
use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};
use tracing::debug;

pub struct IsolateRow {
    pub switch_row: SwitchRow,
    desktop_file: Rc<RefCell<DesktopFile>>,
}

impl IsolateRow {
    pub fn new(desktop_file: &Rc<RefCell<DesktopFile>>) -> Rc<Self> {
        let mut desktop_file_borrow = desktop_file.borrow_mut();
        let browser = desktop_file_borrow.get_browser();
        let browser_can_isolate = browser.as_ref().is_some_and(|browser| browser.can_isolate);
        let has_isolated = desktop_file_borrow.get_isolated();
        let is_isolated = has_isolated.unwrap_or(false);

        let switch_row = SwitchRow::builder()
            .title(t!("web_apps.web_app_view.isolate.title"))
            .subtitle(t!("web_apps.web_app_view.isolate.subtitle"))
            .active(is_isolated)
            .sensitive(browser_can_isolate)
            .tooltip_text(t!("web_apps.web_app_view.isolate.disabled"))
            .has_tooltip(!browser_can_isolate)
            .build();

        if let Some(browser) = browser
            && !browser.can_isolate
            && is_isolated
        {
            debug!("Found desktop file with isolate on a browser that is incapable");
            switch_row.set_active(false);
        }

        // SwitchRow has already a setting on load, so sync this if empty
        if has_isolated.is_none() {
            desktop_file_borrow.set_isolated(switch_row.is_active());
        }

        Rc::new(Self {
            switch_row,
            desktop_file: desktop_file.clone(),
        })
    }

    pub fn init(self: &Rc<Self>, web_app_view: &Rc<WebAppView>) {
        let self_clone = self.clone();
        let web_app_view_clone = web_app_view.clone();

        self.switch_row.connect_active_notify(move |_| {
            self_clone.on_active(&web_app_view_clone);
        });
    }

    fn on_active(&self, web_app_view: &Rc<WebAppView>) {
        self.desktop_file
            .borrow_mut()
            .set_isolated(self.switch_row.is_active());

        self.on_isolation_change(web_app_view);

        if self.is_valid(false)
            && let Some(next_field) = self.switch_row.next_sibling()
        {
            next_field.grab_focus();
        }
    }

    pub fn on_isolation_change(&self, web_app_view: &Rc<WebAppView>) {
        let mut desktop_file_borrow = self.desktop_file.borrow_mut();
        let is_isolated = self.switch_row.is_active();
        let old_profile_path = desktop_file_borrow.get_profile_path().unwrap_or_default();
        let browser_can_isolate = desktop_file_borrow
            .get_browser()
            .is_some_and(|browser| browser.can_isolate);

        let new_profile_path = if is_isolated && browser_can_isolate {
            match desktop_file_borrow.build_profile_path() {
                Err(error) => {
                    drop(desktop_file_borrow);
                    web_app_view.on_error("Could not set isolation", Some(&error));
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

        desktop_file_borrow.set_profile_path(&new_profile_path);
    }
}

impl FormField for IsolateRow {
    fn apply(self: Rc<Self>, _web_app_view: &Rc<WebAppView>) -> Option<JoinHandle<()>> {
        None
    }

    fn is_valid(&self, _do_dirty_check: bool) -> bool {
        true
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn reset(&self, _do_dirty_check: bool) {
        let browser = self.desktop_file.borrow().get_browser();
        let browser_can_isolate = browser.as_ref().is_some_and(|browser| browser.can_isolate);

        self.switch_row.set_sensitive(browser_can_isolate);

        if browser_can_isolate {
            self.switch_row.set_has_tooltip(false);
        } else {
            self.switch_row.set_has_tooltip(true);
        }
        if browser.is_some() && !browser_can_isolate {
            self.switch_row.set_active(false);
        }
    }
}
