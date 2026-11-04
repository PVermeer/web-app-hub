use gtk::{
    Button, EventControllerKey, GestureClick,
    glib::Propagation,
    prelude::{EventControllerExt, WidgetExt},
};
use std::rc::Rc;

/// Add early connect to ``Button``. This runs before other event controllers like 'widget leave'
pub trait ButtonAppExt {
    fn connect_pressed_early<F: Fn(&Self) + 'static>(&self, callback: F);
}

impl ButtonAppExt for Button {
    fn connect_pressed_early<F: Fn(&Self) + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);

        // Mouse / touch
        let gesture = GestureClick::new();
        gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
        let callback_clone = callback.clone();
        let self_clone = self.clone();

        gesture.connect_pressed(move |_, _, _, _| {
            callback_clone(&self_clone);
        });

        self.add_controller(gesture);

        // Keyboard
        let key_controller = EventControllerKey::new();
        key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        let callback = callback.clone();
        let self_clone = self.clone();

        key_controller.connect_key_pressed(move |_, key, _, _| {
            if matches!(
                key,
                gtk::gdk::Key::Return | gtk::gdk::Key::KP_Enter | gtk::gdk::Key::space
            ) {
                callback(&self_clone);
            }

            Propagation::Proceed
        });

        self.add_controller(key_controller);
    }
}
