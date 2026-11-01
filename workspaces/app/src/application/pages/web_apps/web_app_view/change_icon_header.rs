use super::{form::FormField, icon_picker::IconPicker};
use common::desktop_file::DesktopFile;
use gtk::{
    Button, Label, Orientation, Widget,
    glib::{JoinHandle, object::Cast},
    prelude::{BoxExt, ButtonExt, WidgetExt},
};
use libadwaita::ButtonContent;
use std::{cell::RefCell, path::Path, rc::Rc};
use tracing::debug;

pub struct ChangeIconHeader {
    pub header: gtk::Box,
    pub change_icon_button: Button,
    desktop_file: Rc<RefCell<DesktopFile>>,
    icon_picker: Rc<IconPicker>,
    validation_label: Label,
}

impl ChangeIconHeader {
    pub fn new(desktop_file: &Rc<RefCell<DesktopFile>>, icon_picker: &Rc<IconPicker>) -> Rc<Self> {
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

    fn on_init(self: &Rc<Self>, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        let self_clone = self.clone();
        let web_app_view = web_app_view.clone();

        self.change_icon_button
            .connect_clicked(move |_button| self_clone.on_click(&web_app_view));
    }

    fn on_click(&self, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        let undo_icon_path = self
            .desktop_file
            .borrow()
            .get_icon_path()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let undo_icon_path_fail = undo_icon_path.clone();

        let desktop_file = self.desktop_file.clone();
        let web_app_view = web_app_view.clone();

        self.icon_picker.show_dialog(
            Some(move || {
                // Success
            }),
            Some(move || {
                // Fail
                let undo_icon_path = undo_icon_path_fail.clone();
                desktop_file
                    .borrow_mut()
                    .set_icon_path(Path::new(&undo_icon_path));

                web_app_view.error_message("Failed to save icon", None);
            }),
        );
    }

    fn on_reset(&self, do_dirty_check: bool) {
        debug!("Resetting change icon");

        if self.is_valid(do_dirty_check) {
            self.change_icon_button.remove_css_class("error");
            self.validation_label.set_visible(false);
        } else {
            self.change_icon_button.add_css_class("error");
            self.validation_label.set_visible(true);
        }
    }
}

impl FormField for ChangeIconHeader {
    fn init(self: Rc<Self>, web_app_view: Rc<dyn super::WebAppViewShared>) {
        self.on_init(&web_app_view);
    }

    fn apply(
        self: Rc<Self>,
        _web_app_view: Rc<dyn super::WebAppViewShared>,
    ) -> Option<JoinHandle<()>> {
        None
    }

    fn reset(&self, _web_app_view: Rc<dyn super::WebAppViewShared>, do_dirty_check: bool) {
        self.on_reset(do_dirty_check);
    }

    fn is_valid(&self, _do_dirty_check: bool) -> bool {
        self.desktop_file
            .borrow()
            .get_icon_path()
            .is_some_and(|icon_path| !icon_path.to_string_lossy().is_empty())
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn form_widget(&self) -> Widget {
        self.change_icon_button.clone().upcast()
    }
}
