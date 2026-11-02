use super::form::FormField;
use common::desktop_file::DesktopFile;
use gtk::{Widget, glib::object::Cast, prelude::WidgetExt};
use libadwaita::SwitchRow;
use std::rc::Rc;
use tracing::debug;

pub struct MaximizeRow {
    pub switch_row: SwitchRow,
    desktop_file: Rc<DesktopFile>,
}

impl MaximizeRow {
    pub fn new(desktop_file: &Rc<DesktopFile>) -> Rc<Self> {
        let switch_row = SwitchRow::builder()
            .title(t!("web_apps.web_app_view.maximize.title"))
            .subtitle(t!("web_apps.web_app_view.maximize.subtitle"))
            .tooltip_text(t!("web_apps.web_app_view.maximize.disabled"))
            .build();

        Rc::new(Self {
            switch_row,
            desktop_file: desktop_file.clone(),
        })
    }

    fn on_init(self: &Rc<Self>, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        self.set_from_desktop_file();

        let self_clone = self.clone();
        let web_app_view = web_app_view.clone();

        self.switch_row.connect_active_notify(move |_| {
            self_clone.on_active(&web_app_view);
        });
    }

    fn on_active(&self, _web_app_view: &Rc<dyn super::WebAppViewShared>) {
        self.desktop_file.set_maximized(self.switch_row.is_active());
    }

    fn on_reset(&self) {
        debug!("Resetting maximize_row");

        self.set_from_desktop_file();
    }

    fn set_from_desktop_file(&self) {
        let has_maximized = self.desktop_file.get_maximized();
        let is_maximized = has_maximized.unwrap_or(false);
        let browser_can_maximize = self
            .desktop_file
            .get_browser()
            .is_some_and(|browser| browser.can_start_maximized);

        self.switch_row.set_active(is_maximized);
        self.switch_row.set_sensitive(browser_can_maximize);
        self.switch_row.set_has_tooltip(!browser_can_maximize);

        if !browser_can_maximize && is_maximized {
            debug!("Found desktop file with maximize on a browser that is incapable");
            self.switch_row.set_active(false);
        }

        // SwitchRow has already a setting on load, so sync this if empty
        if has_maximized.is_none() {
            self.desktop_file.set_maximized(self.switch_row.is_active());
        }
    }
}

impl FormField for MaximizeRow {
    fn init(self: Rc<Self>, web_app_view: Rc<dyn super::WebAppViewShared>) {
        self.on_init(&web_app_view);
    }

    fn reset(&self, _web_app_view: Rc<dyn super::WebAppViewShared>) {
        self.on_reset();
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
