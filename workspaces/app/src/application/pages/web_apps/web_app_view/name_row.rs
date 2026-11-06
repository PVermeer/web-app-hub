use super::{EntryRowAppExt, WebAppViewShared, form::FormField};
use common::desktop_file::{self, DesktopFile};
use gtk::{InputPurpose, prelude::EditableExt};
use libadwaita::{
    EntryRow,
    prelude::{EntryRowExt, PreferencesRowExt},
};
use std::rc::Rc;
use tracing::debug;

pub struct NameRow {
    pub entry_row: EntryRow,
    desktop_file: Rc<DesktopFile>,
}
impl NameRow {
    pub fn new(desktop_file: &Rc<DesktopFile>) -> Rc<Self> {
        let name = desktop_file.get_name().unwrap_or_default();
        let title = t!("web_apps.web_app_view.name.title").to_string();

        let entry_row = EntryRow::builder()
            .title(&title)
            .text(name)
            .input_purpose(InputPurpose::FreeForm)
            .build();
        entry_row.set_entry_row_ext(
            desktop_file,
            &desktop_file::Key::Name,
            &entry_row.title(),
            false,
        );

        Rc::new(Self {
            entry_row,
            desktop_file: desktop_file.clone(),
        })
    }

    pub fn on_init(self: &Rc<Self>, _web_app_view: &Rc<dyn WebAppViewShared>) {
        let self_clone = self.clone();

        self.entry_row
            .connect_apply(move |_entry_row| self_clone.on_apply());
    }

    fn on_apply(&self) {
        if !self.is_valid(false, false) || !self.is_dirty() {
            return;
        }
        self.desktop_file.set_name(&self.entry_row.text());
    }

    fn on_reset(&self) {
        debug!("Resetting NameRow");

        self.entry_row
            .set_text(&self.desktop_file.get_name().unwrap_or_default());

        self.entry_row.reset_touched();
    }
}

impl FormField for NameRow {
    fn init(self: Rc<Self>, web_app_view: Rc<dyn WebAppViewShared>) {
        self.on_init(&web_app_view);
    }

    fn apply(self: Rc<Self>, _web_app_view: Rc<dyn WebAppViewShared>) {
        self.on_apply();
    }

    fn reset(&self, _web_app_view: Rc<dyn WebAppViewShared>) {
        self.on_reset();
    }

    fn is_valid(&self, do_dirty_check: bool, apply_to_field: bool) -> bool {
        self.entry_row.is_valid(do_dirty_check, apply_to_field)
    }

    fn is_dirty(&self) -> bool {
        self.entry_row.is_dirty()
    }
}
