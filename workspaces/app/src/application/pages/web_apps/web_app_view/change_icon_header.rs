use std::{cell::RefCell, path::Path, rc::Rc};

use common::desktop_file::DesktopFile;
use gtk::{
    Button, Label, Orientation,
    glib::JoinHandle,
    prelude::{BoxExt, ButtonExt, WidgetExt},
};
use libadwaita::ButtonContent;
use tracing::debug;

use super::{WebAppView, form::FormField};

pub struct ChangeIconHeader {
    pub header: gtk::Box,
    pub change_icon_button: Button,
    desktop_file: Rc<RefCell<DesktopFile>>,
    validation_label: Label,
}

impl ChangeIconHeader {
    pub fn new(desktop_file: &Rc<RefCell<DesktopFile>>) -> Rc<Self> {
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
            change_icon_button,
            validation_label,
        })
    }

    pub fn init(self: &Rc<Self>, web_app_view: &Rc<WebAppView>) {
        let web_app_view_clone = web_app_view.clone();

        self.change_icon_button
            .connect_clicked(move |_button| Self::on_click(&web_app_view_clone));
    }

    pub fn on_click(web_app_view: &Rc<WebAppView>) {
        let undo_icon_path = web_app_view
            .desktop_file
            .borrow()
            .get_icon_path()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let undo_icon_path_fail = undo_icon_path.clone();

        let web_app_view_self_clone_fail = web_app_view.clone();

        web_app_view.icon_picker.show_dialog(
            Some(move || {
                // Success
            }),
            Some(move || {
                // Fail
                let undo_icon_path = undo_icon_path_fail.clone();
                web_app_view_self_clone_fail
                    .desktop_file
                    .borrow_mut()
                    .set_icon_path(Path::new(&undo_icon_path));

                web_app_view_self_clone_fail.on_error("Failed to save icon", None);
            }),
        );
    }
}

impl FormField for ChangeIconHeader {
    fn apply(self: Rc<Self>, _web_app_view: &Rc<WebAppView>) -> Option<JoinHandle<()>> {
        None
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

    fn reset(&self, do_dirty_check: bool) {
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
