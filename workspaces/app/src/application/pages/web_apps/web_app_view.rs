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
    Align, EventControllerMotion, GestureClick, PropagationPhase,
    gdk_pixbuf::Pixbuf,
    prelude::{EventControllerExt, GtkWindowExt},
};
use icon_picker::IconPicker;
use isolate_row::IsolateRow;
use libadwaita::{
    ActionRow, NavigationPage, NavigationView, PreferencesGroup, Toast, ToastOverlay,
    gtk::{
        self, Button, Orientation,
        prelude::{BoxExt, ButtonExt, WidgetExt},
    },
    prelude::{NavigationPageExt, PreferencesGroupExt, PreferencesPageExt},
};
use maximize_row::MaximizeRow;
use name_row::NameRow;
use std::{cell::RefCell, rc::Rc};
use tracing::{debug, error};
use url_row::UrlRow;

pub trait WebAppViewShared {
    fn is_new(&self) -> bool;
    fn message(&self, message: &str);
    fn urgent_message(&self, message: &str);
    fn discard_message(&self, message: &str) -> Toast;
    fn error_message(&self, message: &str, error: Option<&anyhow::Error>);
}

pub struct WebAppView {
    is_new: RefCell<bool>,
    nav_page: NavigationPage,
    nav_view: Rc<NavigationView>,
    app: Rc<App>,
    desktop_file: Rc<DesktopFile>,
    desktop_file_original: DesktopFile,
    app_icon_original: RefCell<Option<Pixbuf>>,
    nav_back_button: Button,
    reset_button: Button,
    toasts: ToastOverlay,
    app_header: Rc<AppHeader>,
    change_icon_form_header: Rc<ChangeIconHeader>,
    save_button: Button,
    delete_button: Button,
    url_row: Rc<UrlRow>,
    form_fields: Vec<Rc<dyn FormField>>,
}

impl WebAppView {
    pub fn new(
        app: &Rc<App>,
        nav_view: &Rc<NavigationView>,
        desktop_file: &Rc<DesktopFile>,
        is_new: bool,
    ) -> Rc<Self> {
        let desktop_file_original = desktop_file.as_ref().clone(); // Deep clone
        let icon_original = desktop_file
            .get_icon_path()
            .and_then(|path| gtk::gdk_pixbuf::Pixbuf::from_file(path).ok());

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

        // Set custom nagination back
        nav_page.set_can_pop(false);
        let nav_back_button = Button::from_icon_name("go-previous-symbolic");
        header.pack_start(&nav_back_button);

        let reset_button = Self::build_header_reset_button();
        let save_button = Self::build_save_button();
        let delete_button = Self::build_delete_button();
        let optional_row = OptionalSettings::new(app, desktop_file);
        let icon_picker = IconPicker::new(app, desktop_file);
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
            optional_row.clone(),
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
        bottom_pref_group.add(&optional_row.action_row);

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
            desktop_file: desktop_file.clone(),
            desktop_file_original,
            app_icon_original: RefCell::new(icon_original),
            nav_back_button,
            reset_button,
            toasts: toast_overlay,
            app_header,
            change_icon_form_header,
            save_button,
            delete_button,
            url_row,
            form_fields,
        })
    }

    pub fn init(self: &Rc<Self>) {
        for field in &self.form_fields {
            field.clone().init(self.clone());
        }

        self.connect_reset_button();
        self.connect_nav_pop();
        self.connect_save_button();
        self.connect_delete_button();

        // Pick up some possible changes after init
        self.reset_form();

        let self_clone = self.clone();

        self.desktop_file
            .on_change(move |_desktop_file, _key, _new_value, _previous_value| {
                self_clone.reset_form();
            });
    }

    pub fn is_new(&self) -> bool {
        *self.is_new.borrow()
    }

    fn reset_desktop_file(self: &Rc<Self>) {
        debug!("Resetting desktop file");

        if self.url_row.is_fetching_icon() {
            self.message(&t!("forms.fetching_icon"));
            return;
        }

        // Desktop file
        let save_path = self.desktop_file.get_path();
        self.desktop_file
            .set_from_desktop_file(&self.desktop_file_original);
        self.desktop_file.set_path(&save_path);

        // Icon
        let original_icon_borrow = self.app_icon_original.borrow();
        if let Some(pixbuf) = &*original_icon_borrow
            && let Some(icon_path) = self.desktop_file.get_icon_path()
        {
            let _ = pixbuf.savev(icon_path, "png", &[]).inspect_err(
                |error| error!(%error, "Failed to save icon on resetting desktop file"),
            );
        }

        self.reset_form();

        self.message(&t!("web_apps.web_app_view.toast.reset"));
    }

    fn build_header_reset_button() -> Button {
        Button::with_label(&t!("web_apps.web_app_view.button.reset"))
    }

    fn build_button_footer(buttons: &[&Button; 2]) -> PreferencesGroup {
        let pref_group = PreferencesGroup::new();
        let content_box = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .halign(Align::Center)
            .spacing(24)
            .margin_top(12)
            .margin_bottom(12)
            .build();

        for button in *buttons {
            content_box.append(button);
        }

        pref_group.add(&content_box);

        pref_group
    }

    fn build_save_button() -> Button {
        Button::builder()
            .label(t!("web_apps.web_app_view.button.save"))
            .css_classes(["suggested-action", "pill"])
            .visible(false)
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

    fn connect_reset_button(self: &Rc<Self>) {
        let self_clone = self.clone();

        // This GestureClick forces the reset button to run before all other callbacks.
        // Thanks GPT, this would have taking me ages.
        let gesture = GestureClick::new();
        gesture.set_propagation_phase(PropagationPhase::Capture);
        gesture.connect_pressed(move |_, _, _, _| self_clone.reset_desktop_file());
        self.reset_button.add_controller(gesture);
    }

    fn connect_nav_pop(self: &Rc<Self>) {
        let self_clone = self.clone();

        // This GestureClick forces the reset button to run before all other callbacks.
        // This used to reset the DesktopFile so no other action runs after pop().
        // Thanks GPT, this would have taking me ages.
        let gesture = GestureClick::new();
        gesture.set_propagation_phase(PropagationPhase::Capture);
        gesture.connect_pressed(move |_, _, _, _| {
            // Before dirty state
            if self_clone.is_new() {
                self_clone.reset_desktop_file();
                self_clone.nav_view.pop();
                return;
            }

            // Now apply dirty state
            self_clone.apply_form();

            if !self_clone.save() {
                // Special case, user has to wait it and try again
                if self_clone.url_row.is_fetching_icon() {
                    return;
                }

                let toast = self_clone.discard_message(&t!("forms.validation_error"));
                let self_clone = self_clone.clone();

                toast.connect_button_clicked(move |_toast| {
                    self_clone.nav_view.pop();
                });
                return;
            }

            self_clone.nav_view.pop();
        });

        self.nav_back_button.add_controller(gesture);
    }

    fn connect_save_button(self: &Rc<Self>) {
        let self_clone = self.clone();

        if self.is_new() {
            self.save_button.set_visible(true);
        }

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

    fn apply_form(self: &Rc<Self>) {
        for field in &self.form_fields {
            field.clone().apply(self.clone());
        }
    }

    fn reset_form(self: &Rc<Self>) {
        for field in &self.form_fields {
            field.reset(self.clone());
        }
    }

    fn save(&self) -> bool {
        self.app.window.adw_window.set_focus(None::<&gtk::Widget>);

        if self.url_row.is_fetching_icon() {
            self.urgent_message(&t!("forms.fetching_icon"));
            return false;
        }

        if !self.form_is_valid(true, true) {
            self.urgent_message(&t!("forms.validation_error"));
            return false;
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

            return false;
        }

        if self.is_new() {
            *self.is_new.borrow_mut() = false;
            self.app_header.run_app_button.set_visible(true);
            self.save_button.set_visible(false);

            // Update data for reset action
            self.desktop_file_original
                .set_from_desktop_file(&self.desktop_file);
            *self.app_icon_original.borrow_mut() = self
                .desktop_file
                .get_icon_path()
                .and_then(|path| gtk::gdk_pixbuf::Pixbuf::from_file(path).ok());
        }

        true
    }
}

impl WebAppViewShared for WebAppView {
    fn is_new(&self) -> bool {
        self.is_new()
    }

    fn message(&self, message: &str) {
        self.toasts.message(message);
    }

    fn urgent_message(&self, message: &str) {
        self.toasts.urgent_message(message);
    }

    fn discard_message(&self, message: &str) -> Toast {
        self.toasts.discard_message(message)
    }

    fn error_message(&self, message: &str, error: Option<&anyhow::Error>) {
        self.toasts.error(message, error);
    }
}

impl NavPage for WebAppView {
    fn get_navpage(&self) -> &NavigationPage {
        &self.nav_page
    }

    fn get_nav_row(&self) -> Option<&ActionRow> {
        None
    }
}
