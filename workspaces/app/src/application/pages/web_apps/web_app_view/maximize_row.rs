use super::{WebAppView, form::FormField};
use common::desktop_file::DesktopFile;
use gtk::{glib::JoinHandle, prelude::WidgetExt};
use libadwaita::SwitchRow;
use std::{cell::RefCell, rc::Rc};
use tracing::debug;

pub struct MaximizeRow {
    pub switch_row: SwitchRow,
    desktop_file: Rc<RefCell<DesktopFile>>,
}

impl MaximizeRow {
    pub fn new(desktop_file: &Rc<RefCell<DesktopFile>>) -> Rc<Self> {
        let mut desktop_file_borrow = desktop_file.borrow_mut();
        let has_maximized = desktop_file_borrow.get_maximized();
        let is_maximized = has_maximized.unwrap_or(false);
        let browser_can_maximize = desktop_file_borrow
            .get_browser()
            .is_some_and(|browser| browser.can_start_maximized);

        let switch_row = SwitchRow::builder()
            .title(t!("web_apps.web_app_view.maximize.title"))
            .subtitle(t!("web_apps.web_app_view.maximize.subtitle"))
            .active(is_maximized)
            .sensitive(browser_can_maximize)
            .tooltip_text(t!("web_apps.web_app_view.maximize.disabled"))
            .has_tooltip(!browser_can_maximize)
            .build();

        if !browser_can_maximize && is_maximized {
            debug!("Found desktop file with maximize on a browser that is incapable");
            switch_row.set_active(false);
        }

        // SwitchRow has already a setting on load, so sync this if empty
        if has_maximized.is_none() {
            desktop_file_borrow.set_maximized(switch_row.is_active());
        }

        Rc::new(Self {
            switch_row,
            desktop_file: desktop_file.clone(),
        })
    }

    pub fn init(self: &Rc<Self>) {
        let self_clone = self.clone();

        self.switch_row.connect_active_notify(move |_| {
            self_clone.on_active();
        });
    }

    fn on_active(&self) {
        self.desktop_file
            .borrow_mut()
            .set_maximized(self.switch_row.is_active());

        if self.is_valid(false)
            && let Some(next_field) = self.switch_row.next_sibling()
        {
            next_field.grab_focus();
        }
    }
}

impl FormField for MaximizeRow {
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
        let browser_can_maximize = self
            .desktop_file
            .borrow()
            .get_browser()
            .is_some_and(|browser| browser.can_start_maximized);

        self.switch_row.set_sensitive(browser_can_maximize);

        if browser_can_maximize {
            self.switch_row.set_has_tooltip(false);
        } else {
            self.switch_row.set_active(false);
            self.switch_row.set_has_tooltip(true);
        }
    }
}
