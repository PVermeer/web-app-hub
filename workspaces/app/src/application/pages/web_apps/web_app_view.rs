mod app_header;
mod browser_row;
mod button_ext;
mod change_icon_header;
mod entry_row_ext;
mod form;
mod icon_picker;
mod isolate_row;
mod maximize_row;
mod name_row;
mod optional_row;
mod switch_row_ext;
mod url_row;

use crate::application::{
    App,
    pages::{
        NavPage, PrefPage, ToastOverlayAppExt, web_apps::web_app_view::optional_row::OptionalRow,
    },
};
use anyhow::{Error, anyhow};
use app_header::AppHeader;
use browser_row::BrowserRow;
use button_ext::ButtonAppExt;
use change_icon_header::ChangeIconHeader;
use common::desktop_file::{
    DesktopFile,
    Key::{self},
    error::DesktopFileError,
};
pub use entry_row_ext::EntryRowAppExt;
use form::FormField;
use gtk::{Align, EventControllerMotion, prelude::GtkWindowExt};
use icon_picker::{IconPicker, icon::Icon};
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
pub use switch_row_ext::SwitchRowAppExt;
use tracing::debug;
use url_row::UrlRow;

pub struct WebAppView {
    is_new: RefCell<bool>,
    nav_page: NavigationPage,
    nav_view: Rc<NavigationView>,
    app: Rc<App>,
    desktop_file: Rc<DesktopFile>,
    desktop_file_original: RefCell<Option<DesktopFile>>,
    app_icon: RefCell<Option<Rc<Icon>>>,
    app_icon_original: RefCell<Option<Rc<Icon>>>,
    icon_picker: Rc<IconPicker>,
    nav_back_button: Button,
    reset_button: Button,
    toasts: ToastOverlay,
    app_header: Rc<AppHeader>,
    change_icon_form_header: Rc<ChangeIconHeader>,
    save_button: Button,
    delete_button: Button,
    form_fields: Vec<Rc<dyn FormField>>,
}

impl WebAppView {
    pub fn new(
        app: &Rc<App>,
        nav_view: &Rc<NavigationView>,
        desktop_file: &Rc<DesktopFile>,
        is_new: bool,
    ) -> Rc<Self> {
        let app_icon = desktop_file
            .get_icon_path()
            .and_then(|path| Icon::from_path(&path).ok());

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
        let optional_row = OptionalRow::new(app, desktop_file);
        let icon_picker = IconPicker::new(app, desktop_file);

        header.pack_end(&reset_button);

        let app_header = AppHeader::new(desktop_file);
        let change_icon_form_header = ChangeIconHeader::new(desktop_file, &icon_picker);
        let name_row = NameRow::new(desktop_file);
        let url_row = UrlRow::new(desktop_file, &icon_picker, &change_icon_form_header);
        let isolate_row = IsolateRow::new(desktop_file);
        let maximize_row = MaximizeRow::new(desktop_file);
        let browser_row = BrowserRow::new(desktop_file, app);

        let form_fields: Vec<Rc<dyn FormField>> = vec![
            app_header.clone(),
            change_icon_form_header.clone(),
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
            desktop_file_original: RefCell::new(None),
            app_icon: RefCell::new(app_icon),
            app_icon_original: RefCell::new(None),
            icon_picker,
            nav_back_button,
            reset_button,
            toasts: toast_overlay,
            app_header,
            change_icon_form_header,
            save_button,
            delete_button,
            form_fields,
        })
        .init()
    }

    fn init(self: Rc<Self>) -> Rc<Self> {
        for field in &self.form_fields {
            field.clone().init(self.clone());
        }

        self.connect_nav_page();
        self.connect_reset_button();
        self.connect_nav_pop();
        self.connect_save_button();
        self.connect_delete_button();
        self.connect_desktop_file();

        // Pick up some possible changes after init
        self.reset_form();
        self.set_original_for_reset();

        self
    }

    fn build_header_reset_button() -> Button {
        Button::builder()
            .label(t!("web_apps.web_app_view.button.reset"))
            .sensitive(false)
            .build()
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
            .css_classes(["suggested-action", "pill", "dimmed"])
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

        self.reset_button.connect_pressed_early(move |_button| {
            if self_clone.reset_desktop_file() {
                self_clone.save(true);
            }
        });
    }

    fn connect_nav_page(self: &Rc<Self>) {
        let self_clone = self.clone();

        self.nav_page.connect_unrealize(move |_| {
            if self_clone.is_new() {
                let _ = self_clone.desktop_file.delete();
            }
        });
    }

    fn connect_nav_pop(self: &Rc<Self>) {
        let self_clone = self.clone();

        self.nav_back_button.connect_pressed_early(move |_button| {
            self_clone.apply_form();

            if self_clone.is_new() {
                if !self_clone.reset_desktop_file() {
                    return;
                }
                self_clone.nav_view.pop();
                return;
            }

            if !self_clone.save(true) {
                // Special case, user has to wait it and try again
                if self_clone.icon_picker.is_fetching_icon() {
                    return;
                }

                let toast = self_clone.discard_message(&t!("forms.validation_error"));
                let self_clone = self_clone.clone();
                toast.connect_button_clicked(move |_toast| {
                    self_clone.reset_desktop_file();
                    self_clone.save(false);
                    self_clone.nav_view.pop();
                });
                return;
            }

            self_clone.nav_view.pop();
        });
    }

    fn connect_save_button(self: &Rc<Self>) {
        let self_clone = self.clone();

        if self.is_new() {
            self.save_button.set_visible(true);
        }

        self.save_button.connect_clicked(move |_| {
            self_clone.save_new();
        });
    }

    fn connect_delete_button(self: &Rc<Self>) {
        let self_clone = self.clone();

        self.delete_button.connect_clicked(move |_| {
            if self_clone.icon_picker.is_fetching_icon() {
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

    fn connect_desktop_file(self: &Rc<Self>) {
        let self_clone = self.clone();

        self.desktop_file
            .connect_changed(move |desktop_file, key, _new_value, _previous_value| {
                if matches!(key, Key::Icon) {
                    let icon = desktop_file
                        .get_icon_path()
                        .and_then(|icon_path| Icon::from_path(&icon_path).ok());

                    self_clone.update_app_icon(icon.as_ref());
                    self_clone.change_icon_form_header.is_valid(false, true);
                }

                if matches!(key, Key::Url) {
                    self_clone.change_icon_form_header.is_valid(false, true);
                }

                self_clone.app_header.reset(self_clone.clone());
                self_clone.validate(false, false);
                self_clone.save(false);
            });
    }

    fn is_new(&self) -> bool {
        *self.is_new.borrow()
    }

    fn set_original_for_reset(self: &Rc<Self>) {
        *self.desktop_file_original.borrow_mut() = Some(self.desktop_file.as_ref().clone()); // Deep clone
        (*self.app_icon_original.borrow_mut()).clone_from(&self.app_icon.borrow()); // Rc clone
    }

    fn reset_desktop_file(self: &Rc<Self>) -> bool {
        debug!("Resetting desktop file");

        if self.icon_picker.is_fetching_icon() {
            self.message(&t!("forms.fetching_icon"));
            return false;
        }

        let Some(desktop_file_original) = &*self.desktop_file_original.borrow() else {
            let message = "Failed to get original desktop file for reset. This cannot be None!!!";
            self.error_message(message, Some(&anyhow!(message)));
            return false;
        };

        // Update Icon
        if self.app_icon_has_changed() {
            let app_icon_original = &*self.app_icon_original.borrow();
            let original_save_path = desktop_file_original.get_icon_path();

            if let Some(app_icon_original) = app_icon_original
                && let Some(original_save_path) = original_save_path
                && let Err(error) = app_icon_original.save(&original_save_path)
            {
                self.error_message("Failed to save original icon", Some(&error));
                return false;
            }
            self.update_app_icon(app_icon_original.as_ref());
        }

        // Desktop file
        let save_path = self.desktop_file.get_path();
        self.desktop_file
            .set_from_desktop_file(desktop_file_original);
        self.desktop_file.set_path(&save_path);

        // Reset state
        self.reset_form();
        self.validate(true, false);
        self.change_icon_form_header.is_valid(false, true);
        self.message(&t!("web_apps.web_app_view.toast.reset"));

        true
    }

    fn app_icon_has_changed(&self) -> bool {
        let has_changed = *self.app_icon.borrow() != *self.app_icon_original.borrow();

        debug!(has_changed, "Icon change check");

        has_changed
    }

    fn app_has_changed(&self) -> bool {
        let mut has_changed = false;

        if self.desktop_file.to_string()
            != self
                .desktop_file_original
                .borrow()
                .as_ref()
                .map_or_default(std::string::ToString::to_string)
        {
            has_changed = true;
        }

        if self.app_icon_has_changed() {
            has_changed = true;
        }

        debug!(has_changed, "App has changed check");

        has_changed
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

    fn update_app_icon(self: &Rc<Self>, icon: Option<&Rc<Icon>>) {
        *self.app_icon.borrow_mut() = icon.cloned();
    }

    fn validate_form(&self, do_dirty_check: bool, apply_to_fields: bool) -> bool {
        let mut form_is_valid = true;

        // Don't short circuit. Run validation on all fields to also set the errors.
        for field in &self.form_fields {
            if !field.is_valid(do_dirty_check, apply_to_fields) {
                form_is_valid = false;
            }
        }

        debug!(form_is_valid, "Form valid validation");

        form_is_valid
    }

    fn validate(self: &Rc<Self>, do_dirty_check: bool, apply_to_fields: bool) -> bool {
        let form_is_valid = self.validate_form(do_dirty_check, apply_to_fields);

        if form_is_valid {
            self.save_button.remove_css_class("dimmed");
        } else {
            self.save_button.add_css_class("dimmed");
        }

        self.reset_button.set_sensitive(self.app_has_changed());

        form_is_valid
    }

    /// * `show_messages` - Show all status messages, error messages are always shown
    fn save(self: &Rc<Self>, show_messages: bool) -> bool {
        if self.is_new() {
            return false;
        }

        debug!(id = self.desktop_file.get_id(), "Saving web app");

        self.app.window.adw_window.set_focus(None::<&gtk::Widget>);

        if self.icon_picker.is_fetching_icon() {
            debug!("Failed to save: fetching icon");
            if show_messages {
                self.urgent_message(&t!("forms.fetching_icon"));
            }
            return false;
        }

        if !self.validate(true, true) {
            debug!("Failed to save: invalid form");
            if show_messages {
                self.urgent_message(&t!("forms.validation_error"));
            }
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

        true
    }

    fn save_new(self: &Rc<Self>) -> bool {
        *self.is_new.borrow_mut() = false;

        if !self.save(true) {
            *self.is_new.borrow_mut() = true;

            return false;
        }

        self.app_header.run_app_button.set_visible(true);
        self.save_button.set_visible(false);

        self.set_original_for_reset();
        self.validate(false, false);

        true
    }
}

pub trait WebAppViewShared {
    fn is_new(&self) -> bool;
    fn message(&self, message: &str);
    fn urgent_message(&self, message: &str);
    fn discard_message(&self, message: &str) -> Toast;
    fn error_message(&self, message: &str, error: Option<&Error>);
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

    fn error_message(&self, message: &str, error: Option<&Error>) {
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
