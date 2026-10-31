use super::{WebAppView, form::FormField};
use common::{
    browsers::{Base, Browser},
    desktop_file::DesktopFile,
    utils,
};
use gtk::{
    Button, Image, Label, Orientation,
    glib::JoinHandle,
    prelude::{BoxExt, ButtonExt, WidgetExt},
};
use libadwaita::{PreferencesGroup, WrapBox, prelude::PreferencesGroupExt};
use std::{cell::RefCell, fmt::Write as _, rc::Rc};
use tracing::{debug, error};

pub struct AppHeader {
    pub pref_group: PreferencesGroup,
    pub run_app_button: Button,
    desktop_file: Rc<RefCell<DesktopFile>>,
    app_name: Label,
    app_icon: Image,
    browser_name: Label,
}

impl AppHeader {
    pub fn new(desktop_file: &Rc<RefCell<DesktopFile>>) -> Rc<Self> {
        let desktop_file_borrow = desktop_file.borrow();
        let content_box = gtk::Box::new(Orientation::Vertical, 6);

        // Visual representation of the Web App
        let app_name = desktop_file_borrow
            .get_name()
            .unwrap_or(t!("web_apps.web_app_view.new_app.header.name").to_string());
        let app_label = Label::builder()
            .label(app_name)
            .css_classes(["title-1"])
            .build();

        let app_image = desktop_file_borrow.get_icon();
        Self::style_app_icon(&app_image);

        let browser_label = Label::builder()
            .css_classes(["subtitle"])
            .sensitive(false)
            .build();
        if let Some(browser) = desktop_file_borrow.get_browser() {
            browser_label.set_label(&browser.get_name_with_installation());
        }

        // Run app button
        let run_app_button = Button::builder()
            .label(t!("web_apps.web_app_view.button.open"))
            .css_classes(["suggested-action", "pill"])
            .build();

        if desktop_file_borrow.get_exec().is_some() {
            run_app_button.set_sensitive(true);
        } else {
            run_app_button.set_sensitive(false);
        }

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

    pub fn init(self: &Rc<Self>, web_app_view: &Rc<WebAppView>) {
        self.run_app_button
            .set_visible(*web_app_view.is_new.borrow());

        let web_app_view_clone = web_app_view.clone();

        self.run_app_button
            .connect_clicked(move |_| Self::on_run_app_clicked(&web_app_view_clone));
    }

    pub fn set_app_name(&self, name: &str) {
        self.app_name.set_label(name);
    }

    pub fn set_app_icon(&self, app_image: &Image) {
        if let Some(paintable) = app_image.paintable() {
            self.app_icon.set_paintable(Some(&paintable));
        } else if let Some(icon_name) = app_image.icon_name() {
            self.app_icon.set_icon_name(Some(&icon_name));
        }
    }

    pub fn set_browser(&self, browser: &Rc<Browser>) {
        self.browser_name
            .set_label(&browser.get_name_with_installation());
    }

    pub fn reset(self: &Rc<Self>) {
        debug!("Resetting app header");

        let desktop_file_borrow = self.desktop_file.borrow();

        self.set_app_name(&desktop_file_borrow.get_name().unwrap_or_default());
        self.set_app_icon(&desktop_file_borrow.get_icon());
        if let Some(browser) = desktop_file_borrow.get_browser() {
            self.set_browser(&browser);
        }
    }

    fn style_app_icon(app_image: &Image) {
        app_image.add_css_class("icon-dropshadow");
        app_image.set_pixel_size(96);
        app_image.set_margin_start(25);
        app_image.set_margin_end(25);
    }

    fn on_run_app_clicked(web_app_view: &Rc<WebAppView>) {
        let desktop_file_borrow = web_app_view.desktop_file.borrow();
        let Some(mut executable) = desktop_file_borrow.get_exec() else {
            return;
        };

        if utils::env::is_devcontainer() {
            if desktop_file_borrow
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
    fn apply(self: Rc<Self>, _web_app_view: &Rc<WebAppView>) -> Option<JoinHandle<()>> {
        None
    }

    fn is_valid(&self, _do_dirty_check: bool) -> bool {
        true
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn reset(&self, _do_dirty_check: bool) {
        debug!("Resetting app header");

        let desktop_file_borrow = self.desktop_file.borrow();

        self.set_app_name(&desktop_file_borrow.get_name().unwrap_or_default());
        self.set_app_icon(&desktop_file_borrow.get_icon());
        if let Some(browser) = desktop_file_borrow.get_browser() {
            self.set_browser(&browser);
        }
    }
}
