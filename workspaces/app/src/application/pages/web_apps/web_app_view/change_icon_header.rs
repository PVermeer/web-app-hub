use super::icon_picker::IconPicker;
use common::desktop_file::DesktopFile;
use gtk::{
    Button, Label, Orientation,
    prelude::{BoxExt, ButtonExt, WidgetExt},
};
use libadwaita::ButtonContent;
use std::{path::Path, rc::Rc};

pub struct ChangeIconHeader {
    pub header: gtk::Box,
    change_icon_button: Button,
    desktop_file: Rc<DesktopFile>,
    icon_picker: Rc<IconPicker>,
    validation_label: Label,
}

impl ChangeIconHeader {
    pub fn new(desktop_file: &Rc<DesktopFile>, icon_picker: &Rc<IconPicker>) -> Rc<Self> {
        let button_content = ButtonContent::builder()
            .label(t!("web_apps.web_app_view.button.icon"))
            .icon_name("software-update-available-symbolic")
            .build();
        let change_icon_button = Button::builder().child(&button_content).build();

        let validation_label = Label::builder()
            .label(t!(
                "forms.validation.empty",
                field = t!("web_apps.web_app_view.button.icon")
            ))
            .css_classes(["error"])
            .visible(false)
            .build();

        let header_box = gtk::Box::new(Orientation::Horizontal, 12);
        header_box.append(&validation_label);
        header_box.append(&change_icon_button);

        Rc::new(Self {
            header: header_box,
            desktop_file: desktop_file.clone(),
            icon_picker: icon_picker.clone(),
            change_icon_button,
            validation_label,
        })
    }

    pub fn init(self: &Rc<Self>, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        let self_clone = self.clone();
        let web_app_view = web_app_view.clone();

        self.change_icon_button
            .connect_clicked(move |_button| self_clone.on_click(&web_app_view));
    }

    fn on_click(self: &Rc<Self>, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        let undo_icon_path = self
            .desktop_file
            .get_icon_path()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let undo_icon_path_fail = undo_icon_path.clone();

        let desktop_file = self.desktop_file.clone();
        let web_app_view = web_app_view.clone();
        let self_clone = self.clone();
        let self_clone2 = self.clone();

        self.icon_picker.show_dialog(
            Some(move || {
                // Success
                self_clone.validate();
            }),
            Some(move || {
                // Fail
                let undo_icon_path = undo_icon_path_fail.clone();
                desktop_file.set_icon_path(Path::new(&undo_icon_path));
                self_clone2.validate();
                web_app_view.error_message("Failed to save icon", None);
            }),
        );
    }

    pub fn validate(&self) {
        let is_valid = self.is_valid();

        if is_valid {
            self.change_icon_button.remove_css_class("error");
            self.validation_label.set_visible(false);
        } else {
            self.change_icon_button.add_css_class("error");
            self.validation_label.set_visible(true);
        }
    }

    pub fn is_valid(&self) -> bool {
        self.desktop_file.get_icon_path().is_some()
    }

    pub fn set_sensitive(&self, can_click: bool) {
        self.change_icon_button.set_sensitive(can_click);
    }
}
