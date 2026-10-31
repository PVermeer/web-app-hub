use common::desktop_file::{self, DesktopFile};
use gtk::{
    EventControllerFocus, InputPurpose,
    glib::JoinHandle,
    prelude::{EditableExt, WidgetExt},
};
use libadwaita::{
    EntryRow,
    prelude::{EntryRowExt as _, PreferencesRowExt},
};
use std::{cell::RefCell, rc::Rc};
use tracing::debug;

use super::{EntryRowAppExt, WebAppView, form::FormField};

pub struct NameRow {
    pub entry_row: EntryRow,
    desktop_file: Rc<RefCell<DesktopFile>>,
}
impl NameRow {
    pub fn new(desktop_file: &Rc<RefCell<DesktopFile>>) -> Rc<Self> {
        let name = desktop_file.borrow().get_name().unwrap_or_default();
        let title = t!("web_apps.web_app_view.name.title").to_string();

        let entry_row = EntryRow::builder()
            .title(&title)
            .text(name)
            .show_apply_button(true)
            .input_purpose(InputPurpose::FreeForm)
            .build();

        entry_row.set_entry_row_ext(desktop_file, desktop_file::Key::Name, &entry_row.title());

        Rc::new(Self {
            entry_row,
            desktop_file: desktop_file.clone(),
        })
    }

    pub fn init(self: &Rc<Self>) {
        // Text changes
        let self_clone = self.clone();

        self.entry_row.connect_changed(move |_| {
            self_clone.on_changes();
        });

        // Apply
        let self_clone = self.clone();

        self.entry_row.connect_apply(move |_| {
            self_clone.on_apply();
        });

        // Focus changes
        let focus_controller = EventControllerFocus::new();
        let self_clone = self.clone();

        focus_controller.connect_leave(move |_| {
            self_clone.on_leave();
        });

        self.entry_row.add_controller(focus_controller);
    }

    fn on_changes(&self) {
        let validation = self.entry_row.validate(false);
        debug!(?validation, "Validate input: {}", self.entry_row.title());
    }

    fn on_apply(&self) {
        self.desktop_file
            .borrow_mut()
            .set_name(&self.entry_row.text());

        self.entry_row.set_show_apply_button(false);

        if self.is_valid(false)
            && let Some(next_field) = self.entry_row.next_sibling()
        {
            next_field.grab_focus();
        }
    }

    fn on_leave(&self) {
        self.entry_row.validate(true);
    }
}

impl FormField for NameRow {
    fn apply(self: Rc<Self>, _web_app_view: &Rc<WebAppView>) -> Option<JoinHandle<()>> {
        self.on_apply();
        None
    }

    fn is_valid(&self, do_dirty_check: bool) -> bool {
        self.entry_row.is_valid(do_dirty_check)
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn reset(&self, do_dirty_check: bool) {
        debug!("Resetting change icon");

        self.entry_row
            .set_text(&self.desktop_file.borrow().get_name().unwrap_or_default());

        self.is_valid(do_dirty_check);
    }
}
