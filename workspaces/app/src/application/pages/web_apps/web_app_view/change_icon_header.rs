use super::{WebAppViewShared, button_ext::ButtonAppExt, form::FormField, icon_picker::IconPicker};
use common::desktop_file::DesktopFile;
use gtk::{
    Button, Label, Orientation,
    prelude::{BoxExt, WidgetExt},
};
use libadwaita::ButtonContent;
use std::rc::Rc;

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

    fn on_init(self: Rc<Self>) {
        let self_clone = self.clone();

        self.change_icon_button
            .connect_pressed_early(move |_button| self_clone.on_click());
    }

    fn on_click(self: &Rc<Self>) {
        self.icon_picker.show_dialog();
    }

    fn validate(&self, apply_to_fields: bool) -> bool {
        let is_valid = self.is_valid();

        if apply_to_fields {
            if is_valid {
                self.change_icon_button.remove_css_class("error");
                self.validation_label.set_visible(false);
            } else {
                self.change_icon_button.add_css_class("error");
                self.validation_label.set_visible(true);
            }
        }

        is_valid
    }

    fn is_valid(&self) -> bool {
        if self.desktop_file.get_url().is_none() {
            return true;
        }
        self.desktop_file.get_icon_path().is_some()
    }

    pub fn set_sensitive(&self, can_click: bool) {
        self.change_icon_button.set_sensitive(can_click);
    }
}

impl FormField for ChangeIconHeader {
    fn init(self: Rc<Self>, _web_app_view: Rc<dyn WebAppViewShared>) {
        self.on_init();
    }

    fn apply(self: Rc<Self>, _web_app_view: Rc<dyn WebAppViewShared>) {}

    fn reset(&self, _web_app_view: Rc<dyn WebAppViewShared>) {}

    fn is_valid(&self, _do_dirty_check: bool, apply_to_field: bool) -> bool {
        self.validate(apply_to_field)
    }

    fn is_dirty(&self) -> bool {
        false
    }
}
