use super::{SwitchRowAppExt, form::FormField};
use common::desktop_file::DesktopFile;
use gtk::prelude::WidgetExt;
use libadwaita::SwitchRow;
use std::rc::Rc;
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
            .build()
            .fix_switch_focus();

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
        let is_isolated = self.switch_row.is_active();

        let new_profile_path = if is_isolated {
            let Some(browser) = self.desktop_file.get_browser() else {
                web_app_view.error_message("Failed to get browser...", None);
                return;
            };

            match self.desktop_file.build_profile_path(&browser) {
                Err(error) => {
                    web_app_view.error_message("Could not set isolation", Some(&error));
                    return;
                }
                Ok(profile_path) => Some(profile_path),
            }
        } else {
            None
        };

        self.desktop_file
            .set_profile_path(new_profile_path.as_deref());
    }

    fn on_reset(&self, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        debug!("Resetting isolate_row");

        self.set_from_desktop_file();
        self.on_active(web_app_view);
    }

    fn validate(&self) {
        self.set_from_desktop_file();
    }

    fn set_from_desktop_file(&self) {
        let browser = self.desktop_file.get_browser();
        let browser_can_isolate = browser.as_ref().is_some_and(|browser| browser.can_isolate);
        let is_isolated = self.desktop_file.get_profile_path().is_some();

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
    }
}

impl FormField for IsolateRow {
    fn init(self: Rc<Self>, web_app_view: Rc<dyn super::WebAppViewShared>) {
        self.on_init(&web_app_view);
    }

    fn apply(self: Rc<Self>, web_app_view: Rc<dyn super::WebAppViewShared>) {
        self.on_active(&web_app_view);
    }

    fn reset(&self, web_app_view: Rc<dyn super::WebAppViewShared>) {
        self.on_reset(&web_app_view);
    }

    fn is_valid(&self, _do_dirty_check: bool, _apply_to_field: bool) -> bool {
        self.validate();
        true
    }

    fn is_dirty(&self) -> bool {
        false
    }
}
