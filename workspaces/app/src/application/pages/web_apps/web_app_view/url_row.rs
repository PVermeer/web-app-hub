use super::{EntryRowAppExt, WebAppView, form::FormField};
use common::desktop_file::{self, DesktopFile};
use gtk::{
    EventControllerFocus, InputPurpose, glib,
    glib::JoinHandle,
    prelude::{EditableExt, WidgetExt},
};
use libadwaita::{
    EntryRow, Spinner,
    prelude::{EntryRowExt as _, PreferencesRowExt},
};
use std::{cell::RefCell, path::Path, rc::Rc};
use tracing::{debug, error};

pub struct UrlRow {
    pub entry_row: EntryRow,
    desktop_file: Rc<RefCell<DesktopFile>>,
    spinner: Spinner,
    is_fetching_icon: RefCell<bool>,
}
impl UrlRow {
    pub fn new(desktop_file: &Rc<RefCell<DesktopFile>>) -> Rc<Self> {
        let url = desktop_file.borrow().get_url().unwrap_or_default();
        let title = t!("web_apps.web_app_view.url.title").to_string();

        let entry_row = EntryRow::builder()
            .title(&title)
            .text(url)
            .show_apply_button(true)
            .input_purpose(InputPurpose::Url)
            .build();

        entry_row.set_entry_row_ext(desktop_file, desktop_file::Key::Url, &entry_row.title());

        let spinner = Spinner::new();
        spinner.set_visible(false);
        entry_row.add_suffix(&spinner);

        Rc::new(Self {
            entry_row,
            desktop_file: desktop_file.clone(),
            spinner,
            is_fetching_icon: RefCell::new(false),
        })
    }

    pub fn init(self: &Rc<Self>, web_app_view: &Rc<WebAppView>) {
        // Text changes
        let self_clone = self.clone();

        self.entry_row.connect_changed(move |_| {
            self_clone.on_changes();
        });

        // Apply
        let self_clone = self.clone();
        let web_app_view_clone = web_app_view.clone();

        self.entry_row.connect_apply(move |_| {
            self_clone.on_apply(&web_app_view_clone);
        });

        // Focus changes
        let focus_controller = EventControllerFocus::new();
        let self_clone = self.clone();

        focus_controller.connect_leave(move |_| {
            self_clone.on_leave();
        });

        self.entry_row.add_controller(focus_controller);
    }

    pub fn is_fetching_icon(&self) -> bool {
        *self.is_fetching_icon.borrow()
    }

    fn on_changes(self: &Rc<Self>) {
        let validation = self.entry_row.validate(false);

        debug!(?validation, "Validate input: {}", self.entry_row.title());
    }

    fn on_apply(self: &Rc<Self>, web_app_view: &Rc<WebAppView>) -> JoinHandle<()> {
        web_app_view
            .change_icon_form_header
            .change_icon_button
            .set_sensitive(true);

        self.desktop_file
            .borrow_mut()
            .set_url(&self.entry_row.text());

        self.desktop_file.borrow_mut().set_icon_path(Path::new(""));

        web_app_view.app_header.reset();
        web_app_view.icon_picker.reset();

        let self_clone = self.clone();
        let change_icon_form_header = web_app_view.change_icon_form_header.clone();
        let icon_picker = web_app_view.icon_picker.clone();

        if self.is_valid(false)
            && let Some(next_field) = self.entry_row.next_sibling()
        {
            next_field.grab_focus();
        }

        // Bug with clippy, make sure its dropped before await future
        #[allow(clippy::await_holding_refcell_ref)]
        glib::spawn_future_local(async move {
            *self_clone.is_fetching_icon.borrow_mut() = true;
            self_clone.spinner.set_visible(true);
            change_icon_form_header
                .change_icon_button
                .set_sensitive(false);

            if let Err(error) = icon_picker.save_first_icon_found().await {
                self_clone
                    .desktop_file
                    .borrow_mut()
                    .set_icon_path(Path::new(""));
                error!(?error);
            }

            change_icon_form_header.reset(false);
            change_icon_form_header
                .change_icon_button
                .set_sensitive(true);
            self_clone.spinner.set_visible(false);

            *self_clone.is_fetching_icon.borrow_mut() = false;
        })
    }

    fn on_leave(self: &Rc<Self>) {
        self.entry_row.validate(true);
    }
}

impl FormField for UrlRow {
    fn apply(self: Rc<Self>, web_app_view: &Rc<WebAppView>) -> Option<JoinHandle<()>> {
        let join_handle = self.on_apply(web_app_view);
        Some(join_handle)
    }

    fn is_valid(&self, do_dirty_check: bool) -> bool {
        self.entry_row.is_valid(do_dirty_check)
    }

    fn is_dirty(&self) -> bool {
        self.entry_row.is_dirty()
    }

    fn reset(&self, do_dirty_check: bool) {
        debug!("Resetting change icon");

        self.entry_row
            .set_text(&self.desktop_file.borrow().get_url().unwrap_or_default());

        self.is_valid(do_dirty_check);
    }
}
