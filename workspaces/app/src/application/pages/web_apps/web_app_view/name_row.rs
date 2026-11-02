use super::{EntryRowAppExt, form::FormField};
use common::desktop_file::{self, DesktopFile};
use gtk::{
    EventControllerFocus, InputPurpose, Widget,
    glib::object::Cast,
    prelude::{EditableExt, WidgetExt},
};
use libadwaita::{
    EntryRow,
    prelude::{EntryRowExt as _, PreferencesRowExt},
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

        entry_row.set_entry_row_ext(desktop_file, desktop_file::Key::Name, &entry_row.title());

        Rc::new(Self {
            entry_row,
            desktop_file: desktop_file.clone(),
        })
    }

    pub fn on_init(self: &Rc<Self>, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        // Text changes
        let self_clone = self.clone();

        self.entry_row.connect_changed(move |_| {
            self_clone.on_changes();
        });

        // Apply
        let self_clone = self.clone();

        self.entry_row.connect_entry_activated(move |_| {
            if self_clone.is_valid(false, true) {
                self_clone.unfocus_widget(); // Triggers the connect_leave
            }
        });

        // Focus changes
        let focus_controller = EventControllerFocus::new();
        let self_clone = self.clone();
        let web_app_view_clone = web_app_view.clone();

        focus_controller.connect_leave(move |_| {
            self_clone.on_apply(&web_app_view_clone);
        });

        self.entry_row.add_controller(focus_controller);
    }

    fn on_changes(&self) {
        let validation = self.entry_row.validate(false, true);
        debug!(?validation, "Validate input: {}", self.entry_row.title());
    }

    fn on_apply(&self, _web_app_view: &Rc<dyn super::WebAppViewShared>) {
        if !self.is_valid(false, true) || !self.is_dirty() {
            return;
        }

        self.desktop_file.set_name(&self.entry_row.text());
    }

    fn on_reset(&self) {
        debug!("Resetting name_row");

        self.entry_row
            .set_text(&self.desktop_file.get_name().unwrap_or_default());
    }
}

impl FormField for NameRow {
    fn init(self: Rc<Self>, web_app_view: Rc<dyn super::WebAppViewShared>) {
        self.on_init(&web_app_view);
    }

    fn reset(&self, _web_app_view: Rc<dyn super::WebAppViewShared>) {
        self.on_reset();
    }

    fn is_valid(&self, do_dirty_check: bool, apply_to_field: bool) -> bool {
        self.entry_row.is_valid(do_dirty_check, apply_to_field)
    }

    fn is_dirty(&self) -> bool {
        self.entry_row.is_dirty()
    }

    fn form_widget(&self) -> Widget {
        self.entry_row.clone().upcast()
    }
}
