mod app_header;
mod browser_row;
mod change_icon_header;
mod entry_row_ext;
mod form;
mod icon_picker;
mod isolate_row;
mod maximize_row;
mod name_row;
mod optional_settings;
mod url_row;

use crate::application::{
    App,
    pages::{
        NavPage, PrefPage, ToastOverlayAppExt,
        web_apps::web_app_view::optional_settings::OptionalSettings,
    },
};
use anyhow::anyhow;
use app_header::AppHeader;
use browser_row::BrowserRow;
use change_icon_header::ChangeIconHeader;
use common::desktop_file::{DesktopFile, error::DesktopFileError};
pub use entry_row_ext::EntryRowAppExt;
use form::FormField;
use gtk::{
    Align, EventControllerFocus, EventControllerMotion,
    glib::{self},
    prelude::GtkWindowExt,
};
use icon_picker::IconPicker;
use isolate_row::IsolateRow;
use libadwaita::{
    ActionRow, EntryRow, HeaderBar, NavigationPage, NavigationView, PreferencesGroup, Toast,
    ToastOverlay, WrapBox,
    gtk::{
        self, Button, Image, Orientation,
        prelude::{BoxExt, ButtonExt, EditableExt, WidgetExt},
    },
    prelude::{
        ActionRowExt, EntryRowExt, NavigationPageExt, PreferencesGroupExt, PreferencesPageExt,
    },
};
use maximize_row::MaximizeRow;
use name_row::NameRow;
use std::{cell::RefCell, rc::Rc, time::Duration};
use tracing::debug;
use url_row::UrlRow;

pub trait WebAppViewShared {
    fn is_new(&self) -> bool;
    fn message(&self, message: &str);
    fn discard_message(&self, message: &str) -> Toast;
    fn error_message(&self, message: &str, error: Option<&anyhow::Error>);
}

pub struct WebAppView {
    is_new: RefCell<bool>,
    nav_page: NavigationPage,
    nav_view: Rc<NavigationView>,
    app: Rc<App>,
    header: HeaderBar,
    desktop_file: Rc<DesktopFile>,
    desktop_file_original: DesktopFile,
    toasts: ToastOverlay,
    app_header: Rc<AppHeader>,
    change_icon_form_header: Rc<ChangeIconHeader>,
    reset_button: Button,
    save_button: Button,
    delete_button: Button,
    url_row: Rc<UrlRow>,
    optional_row: ActionRow,
    form_fields: Vec<Rc<dyn FormField>>,
}
impl NavPage for WebAppView {
    fn get_navpage(&self) -> &NavigationPage {
        &self.nav_page
    }

    fn get_nav_row(&self) -> Option<&ActionRow> {
        None
    }
}
impl WebAppView {
    pub fn new(
        app: &Rc<App>,
        nav_view: &Rc<NavigationView>,
        desktop_file: &Rc<DesktopFile>,
        is_new: bool,
    ) -> Rc<Self> {
        let desktop_file_original = desktop_file.as_ref().clone(); // Deep clone
        let title = desktop_file
            .get_name()
            .unwrap_or(t!("web_apps.web_app_view.new_app.title").to_string());

        let icon = "preferences-desktop-apps-symbolic";
        let PrefPage {
            nav_page,
            prefs_page,
            toast_overlay,
            header,
            ..
        } = Self::build_nav_page(&title, icon).with_preference_page();
        let icon_picker = IconPicker::new(app, desktop_file);

        let reset_button = Self::build_header_reset_button();
        let save_button = Self::build_save_button();
        let delete_button = Self::build_delete_button();
        let optional_row = Self::build_optional_row();
        let change_icon_form_header = ChangeIconHeader::new(desktop_file, &icon_picker);

        header.pack_end(&reset_button);

        let app_header = AppHeader::new(desktop_file);
        let name_row = NameRow::new(desktop_file);
        let url_row = UrlRow::new(desktop_file, &icon_picker, &change_icon_form_header);
        let isolate_row = IsolateRow::new(desktop_file);
        let maximize_row = MaximizeRow::new(desktop_file);
        let browser_row = BrowserRow::new(desktop_file, app);

        let form_fields: Vec<Rc<dyn FormField>> = vec![
            app_header.clone(),
            name_row.clone(),
            url_row.clone(),
            isolate_row.clone(),
            maximize_row.clone(),
            browser_row.clone(),
        ];

        // Order of the form rows in the UI
        let form_pref_group = PreferencesGroup::builder()
            .header_suffix(&change_icon_form_header.header)
            .build();
        form_pref_group.add(&name_row.entry_row);
        form_pref_group.add(&url_row.entry_row);
        form_pref_group.add(&isolate_row.switch_row);
        form_pref_group.add(&maximize_row.switch_row);
        form_pref_group.add(&browser_row.combo_row);

        let bottom_pref_group = PreferencesGroup::builder().build();
        bottom_pref_group.add(&optional_row);

        let button_footer_pref_group = Self::build_button_footer(&[&save_button, &delete_button]);

        prefs_page.add(&app_header.pref_group);
        prefs_page.add(&form_pref_group);
        prefs_page.add(&bottom_pref_group);
        prefs_page.add(&button_footer_pref_group);

        Rc::new(Self {
            is_new: RefCell::new(is_new),
            nav_page,
            nav_view: nav_view.clone(),
            app: app.clone(),
            header,
            desktop_file: desktop_file.clone(),
            desktop_file_original,
            toasts: toast_overlay,
            app_header,
            change_icon_form_header,
            reset_button,
            save_button,
            delete_button,
            url_row,
            optional_row,
            form_fields,
        })
    }

    pub fn init(self: &Rc<Self>) {
        for field in &self.form_fields {
            field.clone().init(self.clone());
        }

        let self_clone = self.clone();

        self.reset_button
            .connect_clicked(move |_| self_clone.reset_desktop_file());

        self.connect_nav_pop();
        self.connect_optional_row();
        self.connect_save_button();
        self.connect_delete_button();

        // Pick up some possible changes after init
        self.reset_form_fields();

        let self_clone = self.clone();

        self.desktop_file
            .on_change(move |_desktop_file, _key, _new_value, _previous_value| {
                self_clone.reset_form_fields();
                self_clone.reset_reset_button();
            });
    }

    pub fn is_new(&self) -> bool {
        *self.is_new.borrow()
    }

    fn connect_nav_pop(self: &Rc<Self>) {
        self.nav_page.set_can_pop(false);
        let back_button = Button::from_icon_name("go-previous-symbolic");
        self.header.pack_start(&back_button);

        let self_clone = self.clone();
        back_button.connect_clicked(move |_button| {
            self_clone
                .app
                .window
                .adw_window
                .set_focus(None::<&gtk::Widget>);

            if self_clone.url_row.is_fetching_icon() {
                let toast = self_clone.discard_message(&t!("forms.fetching_icon"));

                let self_clone = self_clone.clone();
                toast.connect_button_clicked(move |_toast| {
                    self_clone.nav_view.pop();
                });

                return;
            }

            if !self_clone.form_is_valid(true, true) {
                let toast = self_clone.discard_message(&t!("forms.validation_error"));

                let self_clone = self_clone.clone();
                toast.connect_button_clicked(move |_toast| {
                    self_clone.nav_view.pop();
                });

                return;
            }

            self_clone.nav_view.pop();
        });
    }

    fn reset_desktop_file(self: &Rc<Self>) {
        debug!("Resetting desktop file");

        {
            let save_path = self.desktop_file.get_path();
            self.desktop_file
                .set_from_desktop_file(&self.desktop_file_original);
            self.desktop_file.set_path(&save_path);
        }

        self.reset_form_fields();

        self.message(&t!("web_apps.web_app_view.toast.reset"));
    }

    fn build_header_reset_button() -> Button {
        let reset_button = Button::with_label(&t!("web_apps.web_app_view.button.reset"));
        reset_button.set_sensitive(false);

        reset_button
    }

    fn build_optional_row() -> ActionRow {
        let row = ActionRow::builder()
            .title(t!("web_apps.web_app_view.optional.title"))
            .subtitle(t!("web_apps.web_app_view.optional.subtitle"))
            .activatable(true)
            .build();

        let suffix = Image::from_icon_name("go-next-symbolic");
        row.add_suffix(&suffix);

        row
    }

    fn build_button_footer(buttons: &[&Button; 2]) -> PreferencesGroup {
        fn button_wrap_box(button: &Button) -> WrapBox {
            let wrapbox = WrapBox::builder()
                .align(0.5)
                .margin_start(12)
                .margin_end(12)
                .build();
            wrapbox.append(button);
            wrapbox
        }

        let pref_group = PreferencesGroup::new();
        let content_box = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .halign(Align::Center)
            .margin_top(12)
            .margin_bottom(12)
            .build();

        for button in buttons {
            content_box.append(&button_wrap_box(button));
        }

        pref_group.add(&content_box);

        pref_group
    }

    fn build_save_button() -> Button {
        Button::builder()
            .label(t!("web_apps.web_app_view.button.save"))
            .css_classes(["suggested-action", "pill"])
            .build()
    }

    fn build_delete_button() -> Button {
        let button = Button::builder()
            .label(t!("web_apps.web_app_view.button.delete"))
            .css_classes(["destructive-action", "pill", "dimmed"])
            .build();

        let controller = EventControllerMotion::new();
        let button_clone = button.clone();
        controller.connect_enter(move |_, _, _| {
            button_clone.remove_css_class("dimmed");
        });
        let button_clone = button.clone();
        controller.connect_leave(move |_| {
            button_clone.add_css_class("dimmed");
        });

        button.add_controller(controller);

        button
    }

    fn build_validate_icon() -> Image {
        let validate_icon = Image::from_icon_name("dialog-warning-symbolic");
        validate_icon.set_visible(false);
        validate_icon.set_css_classes(&["error"]);

        validate_icon
    }

    fn send_apply_to_a_form_field(_form_field: &EntryRow) {
        // form_field.emit_by_name::<()>("apply", &[]);

        // // Reset button
        // form_field.set_show_apply_button(false);
        // form_field.set_show_apply_button(true);
    }

    fn entry_row_apply_check(
        entry_row: &EntryRow,
        applied_text: &Rc<RefCell<String>>,
        apply_icon: &Image,
    ) {
        let is_applied = entry_row.text().is_empty() || *applied_text.borrow() == entry_row.text();

        if is_applied || !entry_row.shows_apply_button() {
            apply_icon.set_tooltip_text(None);
            apply_icon.set_visible(false);
        } else {
            apply_icon.set_tooltip_text(Some(&t!("forms.apply_changes")));
            apply_icon.set_visible(true);

            let entry_row_clone = entry_row.clone();

            glib::timeout_add_local_once(Duration::ZERO, move || {
                Self::send_apply_to_a_form_field(&entry_row_clone);
            });
        }
    }

    fn connect_entry_row_focus_controller_for_apply(
        entry_row: &EntryRow,
        applied_text: &Rc<RefCell<String>>,
        apply_icon: &Image,
        is_new: Option<bool>,
    ) {
        let focus_controller = EventControllerFocus::new();
        let entry_row_clone = entry_row.clone();
        let applied_text_clone = applied_text.clone();
        let apply_icon_clone = apply_icon.clone();

        entry_row.add_suffix(apply_icon);

        focus_controller.connect_leave(move |_| {
            Self::entry_row_apply_check(&entry_row_clone, &applied_text_clone, &apply_icon_clone);
        });
        entry_row.add_controller(focus_controller);

        if is_new.is_some_and(|is_new| is_new) {
            Self::entry_row_apply_check(entry_row, applied_text, apply_icon);
        }
    }

    fn connect_save_button(self: &Rc<Self>) {
        let self_clone = self.clone();

        self.save_button.connect_clicked(move |_| {
            self_clone.save();
        });
    }

    fn connect_delete_button(self: &Rc<Self>) {
        let self_clone = self.clone();

        self.delete_button.connect_clicked(move |_| {
            if self_clone.url_row.is_fetching_icon() {
                return;
            }

            debug!(
                "Deleting web app: {}",
                self_clone.desktop_file.get_name().unwrap_or_default()
            );

            if !*self_clone.is_new.borrow() && self_clone.desktop_file.delete().is_err() {
                self_clone.error_message("Failed to delete all files", None);
            }

            self_clone.nav_view.pop();
        });
    }

    fn connect_optional_row(self: &Rc<Self>) {
        let app_clone = self.app.clone();
        let desktop_file_clone = self.desktop_file.clone();
        let self_clone = self.clone();

        self.optional_row.connect_activated(move |_| {
            let optional_dialog = OptionalSettings::new(&app_clone, &desktop_file_clone);
            optional_dialog.show_dialog(&self_clone);
        });
    }

    fn reset_reset_button(&self) {
        if self.desktop_file_original.to_string() == self.desktop_file.to_string() {
            self.reset_button.set_sensitive(false);
        } else {
            self.reset_button.set_sensitive(true);
        }
    }

    fn form_is_valid(&self, do_dirty_check: bool, apply_to_fields: bool) -> bool {
        let mut form_is_valid = true;

        // Don't short circuit. Run validation on all fields to also set the errors.
        for field in &self.form_fields {
            if !field.is_valid(do_dirty_check, apply_to_fields) {
                form_is_valid = false;
            }
        }

        if form_is_valid {
            form_is_valid = self.change_icon_form_header.is_valid();
        }

        debug!(form_is_valid, "Form valid validation");

        form_is_valid
    }

    fn reset_form_fields(self: &Rc<Self>) {
        for field in &self.form_fields {
            field.reset(self.clone());
        }
    }

    fn save(&self) {
        self.app.window.adw_window.set_focus(None::<&gtk::Widget>);

        if self.url_row.is_fetching_icon() {
            self.message(&t!("forms.fetching_icon"));
            return;
        }

        if !self.form_is_valid(true, true) {
            self.message(&t!("forms.validation_error"));
            return;
        }

        if let Err(error) = self.desktop_file.save() {
            match error {
                DesktopFileError::ValidationError(error) => {
                    self.error_message(&error.to_string_ui(), Some(&anyhow!(error)));
                }
                DesktopFileError::Other(error) => {
                    self.error_message(&t!("forms.save_error"), Some(&error));
                }
            }

            return;
        }

        *self.is_new.borrow_mut() = false;
        self.app_header.run_app_button.set_visible(true);
        self.desktop_file_original
            .set_from_desktop_file(&self.desktop_file);
        self.reset_reset_button();

        if !self.is_new() {
            self.nav_view.pop();
        }
    }
}

impl WebAppViewShared for WebAppView {
    fn is_new(&self) -> bool {
        self.is_new()
    }

    fn message(&self, message: &str) {
        self.toasts.message(message);
    }

    fn discard_message(&self, message: &str) -> Toast {
        self.toasts.discard_message(message)
    }

    fn error_message(&self, message: &str, error: Option<&anyhow::Error>) {
        self.toasts.error(message, error);
    }
}
