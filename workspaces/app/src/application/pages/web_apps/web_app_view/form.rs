use super::WebAppView;
use gtk::glib::JoinHandle;
use std::rc::Rc;

pub trait FormField {
    fn apply(self: Rc<Self>, web_app_view: &Rc<WebAppView>) -> Option<JoinHandle<()>>;
    fn is_valid(&self, do_dirty_check: bool) -> bool;
    fn is_dirty(&self) -> bool;
    fn reset(&self, do_dirty_check: bool);
}
