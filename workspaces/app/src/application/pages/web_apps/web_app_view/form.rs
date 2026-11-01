use super::WebAppViewShared;
use gtk::glib::JoinHandle;
use std::rc::Rc;

pub trait FormField {
    fn init(self: Rc<Self>, web_app_view: Rc<dyn WebAppViewShared>);
    fn apply(self: Rc<Self>, web_app_view: Rc<dyn WebAppViewShared>) -> Option<JoinHandle<()>>;
    fn reset(&self, web_app_view: Rc<dyn WebAppViewShared>, do_dirty_check: bool);
    fn is_valid(&self, do_dirty_check: bool) -> bool;
    fn is_dirty(&self) -> bool;
}
