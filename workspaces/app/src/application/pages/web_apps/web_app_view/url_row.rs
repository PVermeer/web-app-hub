use super::{
    EntryRowAppExt, WebAppViewShared, change_icon_header::ChangeIconHeader, form::FormField,
    icon_picker::IconPicker,
};
use common::desktop_file::{self, DesktopFile};
use gtk::{
    InputPurpose,
    glib::{self},
    prelude::{EditableExt, WidgetExt},
};
use libadwaita::{
    EntryRow, Spinner,
    prelude::{EntryRowExt, PreferencesRowExt},
};
use std::{cell::RefCell, path::Path, rc::Rc};
use tracing::{debug, error};

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

    pub fn on_init(self: &Rc<Self>, web_app_view: &Rc<dyn WebAppViewShared>) {
        self.change_icon_form_header.init(web_app_view);
        let self_clone = self.clone();
        let web_app_view = web_app_view.clone();

        self.entry_row
            .connect_apply(move |_entry_row| self_clone.on_apply(&web_app_view));
    }

    pub fn is_fetching_icon(&self) -> bool {
        *self.is_fetching_icon.borrow()
    }

    fn on_apply(self: &Rc<Self>, web_app_view: &Rc<dyn WebAppViewShared>) {
        if !self.is_valid(false, false) || !self.is_dirty() {
            return;
        }

        self.change_icon_form_header.set_sensitive(false);
        self.desktop_file.set_url(&self.entry_row.text());
        self.desktop_file.set_icon_path(Path::new(""));

        self.icon_picker.reset();
        *self.is_fetching_icon.borrow_mut() = true;

        let self_clone = self.clone();
        let change_icon_form_header = self.change_icon_form_header.clone();
        let icon_picker = self.icon_picker.clone();
        let web_app_view = web_app_view.clone();

        // Bug with clippy, make sure its dropped before await future
        #[allow(clippy::await_holding_refcell_ref)]
        glib::spawn_future_local(async move {
            self_clone.entry_row.set_sensitive(false);
            self_clone.spinner.set_visible(true);

            if let Err(error) = icon_picker.save_first_icon_found().await {
                self_clone.desktop_file.set_icon_path(Path::new(""));
                error!(?error);
                web_app_view.message(&t!("web_apps.web_app_view.icon.validate"));
            }

            change_icon_form_header.validate();
            change_icon_form_header.set_sensitive(true);

            self_clone.spinner.set_visible(false);
            self_clone.entry_row.set_sensitive(true);
            *self_clone.is_fetching_icon.borrow_mut() = false;
        });
    }

    fn on_reset(&self) {
        debug!("Resetting UrlRow");

        self.entry_row
            .set_text(&self.desktop_file.get_url().unwrap_or_default());
    }
}

impl FormField for UrlRow {
    fn init(self: Rc<Self>, web_app_view: Rc<dyn WebAppViewShared>) {
        self.on_init(&web_app_view);
    }

    fn apply(self: Rc<Self>, web_app_view: Rc<dyn WebAppViewShared>) {
        self.on_apply(&web_app_view);
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
