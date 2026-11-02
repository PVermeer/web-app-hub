use super::WebAppViewShared;
use gtk::{Widget, glib::JoinHandle, prelude::WidgetExt};
use std::rc::Rc;

pub trait FormField {
    fn init(self: Rc<Self>, web_app_view: Rc<dyn WebAppViewShared>);
    fn apply(self: Rc<Self>, web_app_view: Rc<dyn WebAppViewShared>) -> Option<JoinHandle<()>>;
    fn reset(&self, web_app_view: Rc<dyn WebAppViewShared>);
    fn is_valid(&self, do_dirty_check: bool, apply_to_field: bool) -> bool;
    fn is_dirty(&self) -> bool;

    fn form_widget(&self) -> Widget;

    fn unfocus_widget(&self) {
        if let Some(parent) = self.form_widget().parent() {
            parent.grab_focus();
        }
    }
}
