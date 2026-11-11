use super::{
    EntryRowAppExt, WebAppViewShared, change_icon_header::ChangeIconHeader, form::FormField,
    icon_picker::IconPicker,
};
use common::desktop_file::{self, DesktopFile, Key};
use gtk::{
    InputPurpose,
    glib::{self},
    prelude::{EditableExt, WidgetExt},
};
use libadwaita::{
    EntryRow, Spinner,
    prelude::{EntryRowExt, PreferencesRowExt},
};
use std::{cell::RefCell, rc::Rc};
use tracing::debug;

pub struct UrlRow {
    pub entry_row: EntryRow,
    desktop_file: Rc<DesktopFile>,
    icon_picker: Rc<IconPicker>,
    change_icon_form_header: Rc<ChangeIconHeader>,
    spinner: Spinner,
    is_fetching_icon: RefCell<bool>,
}
impl UrlRow {
    pub fn new(
        desktop_file: &Rc<DesktopFile>,
        icon_picker: &Rc<IconPicker>,
        change_icon_form_header: &Rc<ChangeIconHeader>,
    ) -> Rc<Self> {
        let url = desktop_file.get_url().unwrap_or_default();
        let title = t!("web_apps.web_app_view.url.title").to_string();

        let entry_row = EntryRow::builder()
            .title(&title)
            .text(url)
            .input_purpose(InputPurpose::Url)
            .build();

        entry_row.set_entry_row_ext(
            desktop_file,
            &desktop_file::Key::Url,
            &entry_row.title(),
            false,
        );

        let spinner = Spinner::new();
        spinner.set_visible(false);
        entry_row.add_suffix(&spinner);

        Rc::new(Self {
            entry_row,
            desktop_file: desktop_file.clone(),
            icon_picker: icon_picker.clone(),
            change_icon_form_header: change_icon_form_header.clone(),
            spinner,
            is_fetching_icon: RefCell::new(false),
        })
    }

    fn on_init(self: &Rc<Self>) {
        let self_clone = self.clone();

        self.entry_row
            .connect_apply(move |_entry_row| self_clone.on_apply());

        let self_clone = self.clone();

        self.desktop_file.connect_changed(
            move |_desktop_file, key, _new_value, _previous_value| {
                if matches!(key, Key::Icon) {
                    *self_clone.is_fetching_icon.borrow_mut() = false;
                    self_clone.change_icon_form_header.set_sensitive(true);
                    self_clone.spinner.set_visible(false);
                    self_clone.entry_row.set_sensitive(true);
                }
            },
        );
    }

    pub fn is_fetching_icon(&self) -> bool {
        *self.is_fetching_icon.borrow()
    }

    fn on_apply(self: &Rc<Self>) {
        if !self.is_valid(false, false) || !self.is_dirty() {
            return;
        }

        *self.is_fetching_icon.borrow_mut() = true;
        self.change_icon_form_header.set_sensitive(false);
        self.desktop_file.set_url(&self.entry_row.text());
        self.icon_picker.reset();
        self.entry_row.set_sensitive(false);
        self.spinner.set_visible(true);

        let icon_picker = self.icon_picker.clone();

        glib::spawn_future_local(async move {
            icon_picker.save_first_icon_found().await;
        });
    }

    fn on_reset(&self) {
        debug!("Resetting UrlRow");

        self.entry_row
            .set_text(&self.desktop_file.get_url().unwrap_or_default());

        self.entry_row.reset_touched();
    }
}

impl FormField for UrlRow {
    fn init(self: Rc<Self>, _web_app_view: Rc<dyn WebAppViewShared>) {
        self.on_init();
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
