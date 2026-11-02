use super::{
    EntryRowAppExt, WebAppViewShared, change_icon_header::ChangeIconHeader, form::FormField,
    icon_picker::IconPicker,
};
use common::desktop_file::{self, DesktopFile};
use gtk::{
    EventControllerFocus, InputPurpose, Widget,
    glib::{self, JoinHandle, object::Cast},
    prelude::{EditableExt, WidgetExt},
};
use libadwaita::{
    EntryRow, Spinner,
    prelude::{EntryRowExt as _, PreferencesRowExt},
};
use std::{
    cell::{OnceCell, RefCell},
    path::Path,
    rc::Rc,
};
use tracing::{debug, error};

pub struct UrlRow {
    pub entry_row: EntryRow,
    desktop_file: Rc<DesktopFile>,
    icon_picker: Rc<IconPicker>,
    change_icon_form_header: Rc<ChangeIconHeader>,
    spinner: Spinner,
    is_fetching_icon: RefCell<bool>,
    is_applied_once: OnceCell<()>,
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

        entry_row.set_entry_row_ext(desktop_file, desktop_file::Key::Url, &entry_row.title());

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
            is_applied_once: OnceCell::new(),
        })
    }

    pub fn on_init(self: &Rc<Self>, web_app_view: &Rc<dyn WebAppViewShared>) {
        self.change_icon_form_header.init(web_app_view);

        // Text changes
        let self_clone = self.clone();

        self.entry_row
            .connect_changed(move |_| self_clone.on_changes());

        // Apply
        let self_clone = self.clone();

        self.entry_row.connect_entry_activated(move |_| {
            let _ = self_clone.is_applied_once.set(());

            if self_clone.is_valid(false, true) {
                self_clone.unfocus_widget(); // Triggers connect_leave
            }
        });

        // Focus changes
        let focus_controller = EventControllerFocus::new();
        let self_clone = self.clone();
        let web_app_view_clone = web_app_view.clone();

        focus_controller.connect_leave(move |_| {
            let _ = self_clone.is_applied_once.set(());

            self_clone.on_apply(&web_app_view_clone);
        });

        self.entry_row.add_controller(focus_controller);
    }

    pub fn is_fetching_icon(&self) -> bool {
        *self.is_fetching_icon.borrow()
    }

    fn on_changes(self: &Rc<Self>) {
        if self.is_applied_once.get().is_none() {
            return;
        }

        let validation = self.entry_row.validate(false, true);

        debug!(?validation, "Validate input: {}", self.entry_row.title());
    }

    fn on_apply(
        self: &Rc<Self>,
        _web_app_view: &Rc<dyn WebAppViewShared>,
    ) -> Option<JoinHandle<()>> {
        if !self.is_valid(false, true) || !self.is_dirty() {
            return None;
        }

        self.change_icon_form_header.set_sensitive(false);
        self.desktop_file.set_url(&self.entry_row.text());
        self.desktop_file.set_icon_path(Path::new(""));

        self.icon_picker.reset();

        let self_clone = self.clone();
        let change_icon_form_header = self.change_icon_form_header.clone();
        let icon_picker = self.icon_picker.clone();

        // Bug with clippy, make sure its dropped before await future
        #[allow(clippy::await_holding_refcell_ref)]
        let join_handle = glib::spawn_future_local(async move {
            *self_clone.is_fetching_icon.borrow_mut() = true;
            self_clone.entry_row.set_sensitive(false);
            self_clone.spinner.set_visible(true);

            if let Err(error) = icon_picker.save_first_icon_found().await {
                self_clone.desktop_file.set_icon_path(Path::new(""));
                error!(?error);
            }

            change_icon_form_header.validate();
            change_icon_form_header.set_sensitive(true);

            self_clone.spinner.set_visible(false);
            self_clone.entry_row.set_sensitive(true);
            *self_clone.is_fetching_icon.borrow_mut() = false;
        });

        Some(join_handle)
    }

    fn on_reset(&self) {
        debug!("Resetting url_row");

        self.entry_row
            .set_text(&self.desktop_file.get_url().unwrap_or_default());
    }
}

impl FormField for UrlRow {
    fn init(self: Rc<Self>, web_app_view: Rc<dyn WebAppViewShared>) {
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
