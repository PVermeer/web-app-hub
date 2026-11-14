use gtk::{
    GestureClick, PropagationPhase, Switch, Widget,
    glib::object::{Cast, IsA},
    prelude::{EventControllerExt, WidgetExt},
};
use libadwaita::SwitchRow;

fn add_focus_grab_to_switch(switch_row: &SwitchRow) {
    fn find_switch(widget: &impl IsA<Widget>) -> Option<Switch> {
        let widget: &Widget = widget.as_ref();

        if let Ok(switch) = widget.clone().downcast::<Switch>() {
            return Some(switch);
        }

        let mut child = widget.first_child();

        while let Some(current) = child {
            if let Some(switch) = find_switch(&current) {
                return Some(switch);
            }
            child = current.next_sibling();
        }

        None
    }

    if let Some(switch) = find_switch(switch_row) {
        let gesture = GestureClick::new();
        gesture.set_propagation_phase(PropagationPhase::Capture);

        let row = switch_row.clone();

        gesture.connect_pressed(move |_, _, _, _| {
            row.grab_focus();
        });

        switch.add_controller(gesture);
    }
}

pub trait SwitchRowAppExt {
    /// Add focus grab on clicking the actual switch widget so it behaves the same as a click on the Row.
    fn fix_switch_focus(self) -> Self;
}

impl SwitchRowAppExt for SwitchRow {
    fn fix_switch_focus(self) -> Self {
        add_focus_grab_to_switch(&self);
        self
    }
}
