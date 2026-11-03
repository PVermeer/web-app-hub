use super::WebAppViewShared;
use std::rc::Rc;

pub trait FormField {
    fn init(self: Rc<Self>, web_app_view: Rc<dyn WebAppViewShared>);
    fn reset(&self, web_app_view: Rc<dyn WebAppViewShared>);
    fn is_valid(&self, do_dirty_check: bool, apply_to_field: bool) -> bool;
    fn is_dirty(&self) -> bool;
}
