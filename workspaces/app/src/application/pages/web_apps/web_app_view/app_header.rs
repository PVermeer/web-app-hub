use super::form::FormField;
use common::{browsers::Base, desktop_file::DesktopFile, utils};
use gtk::{
    Button, Image, Label, Orientation,
    prelude::{BoxExt, ButtonExt, WidgetExt},
};
use libadwaita::{PreferencesGroup, WrapBox, prelude::PreferencesGroupExt};
use std::{fmt::Write as _, rc::Rc};
use tracing::{debug, error};

pub struct AppHeader {
    pub pref_group: PreferencesGroup,
    pub run_app_button: Button,
    desktop_file: Rc<DesktopFile>,
    app_name: Label,
    app_icon: Image,
    browser_name: Label,
}

impl AppHeader {
    pub fn new(desktop_file: &Rc<DesktopFile>) -> Rc<Self> {
        let content_box = gtk::Box::new(Orientation::Vertical, 6);

        // Visual representation of the Web App
        let app_label = Label::builder().css_classes(["title-1"]).build();

        let app_image = Image::builder()
            .pixel_size(96)
            .margin_start(25)
            .margin_end(25)
            .css_classes(["icon-dropshadow"])
            .use_fallback(true)
            .build();

        let browser_label = Label::builder()
            .css_classes(["subtitle"])
            .sensitive(false)
            .build();

        // Run app button
        let run_app_button = Button::builder()
            .label(t!("web_apps.web_app_view.button.open"))
            .css_classes(["suggested-action", "pill"])
            .build();

        let button_wrap_box = WrapBox::builder()
            .align(0.5)
            .margin_top(12)
            .margin_bottom(12)
            .build();
        button_wrap_box.append(&run_app_button);

        content_box.append(&app_image);
        content_box.append(&app_label);
        content_box.append(&browser_label);
        content_box.append(&button_wrap_box);

        // Wrap it all in a pref group
        let pref_group = PreferencesGroup::builder().build();
        pref_group.add(&content_box);

        Rc::new(Self {
            pref_group,
            run_app_button,
            desktop_file: desktop_file.clone(),
            app_name: app_label,
            app_icon: app_image,
            browser_name: browser_label,
        })
    }

    fn on_init(self: &Rc<Self>, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        self.set_from_desktop_file(web_app_view);

        let self_clone = self.clone();

        self.run_app_button
            .connect_clicked(move |_| self_clone.on_run_app_clicked());
    }

    fn set_from_desktop_file(&self, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        let is_new = web_app_view.is_new();

        // Get values
        let app_name = self
            .desktop_file
            .get_name()
            .unwrap_or(t!("web_apps.web_app_view.new_app.header.name").to_string());
        let app_image = self.desktop_file.get_icon();
        let browser_name = self
            .desktop_file
            .get_browser()
            .map_or_default(|browser| browser.get_name_with_installation());

        // Update the app header
        if let Some(paintable) = app_image.paintable() {
            self.app_icon.set_paintable(Some(&paintable));
        } else if let Some(icon_name) = app_image.icon_name() {
            self.app_icon.set_icon_name(Some(&icon_name));
        }
        if app_image.uses_fallback() {
            self.app_icon.add_css_class("error");
        } else {
            self.app_icon.remove_css_class("error");
        }
        self.app_name.set_label(&app_name);
        self.browser_name.set_label(&browser_name);
        self.run_app_button.set_visible(!is_new);
    }

    fn on_reset(&self, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        debug!("Resetting app header");

        self.set_from_desktop_file(web_app_view);
    }

    fn on_run_app_clicked(&self) {
        let Some(mut executable) = self.desktop_file.get_exec() else {
            return;
        };

        if utils::env::is_devcontainer() {
            if self
                .desktop_file
                .get_browser()
                .is_some_and(|browser| browser.base == Base::Chromium)
            {
                let _ = write!(executable, " --no-sandbox");
            }
            debug!("Running in dev-container");
        }

        debug!("Running web app: '{executable}'");
        if let Err(error) = utils::command::run_command_background(&executable) {
            error!(
                executable = executable,
                error = error.to_string(),
                "Failed to run app"
            );
        }
    }
}

impl FormField for AppHeader {
    fn init(self: Rc<Self>, web_app_view: Rc<dyn super::WebAppViewShared>) {
        self.on_init(&web_app_view);
    }

    fn apply(self: Rc<Self>, _web_app_view: Rc<dyn super::WebAppViewShared>) {}

    fn touch(&self) {}

    fn reset(&self, web_app_view: Rc<dyn super::WebAppViewShared>) {
        self.on_reset(&web_app_view);
    }

    fn is_valid(&self, _do_dirty_check: bool, _apply_to_field: bool) -> bool {
        true
    }

    fn is_dirty(&self) -> bool {
        false
    }
}
