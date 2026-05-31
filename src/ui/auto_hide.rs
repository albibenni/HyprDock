use gtk4::prelude::*;
use gtk4::{Box, Revealer, Orientation, RevealerTransitionType};
use gtk4_layer_shell::LayerShell;
use crate::ui::constants::*;
use crate::ui::DockUI;
use std::rc::Rc;
use gtk4::glib;
use gtk4::EventControllerMotion;

pub fn setup_auto_hide(ui: &Rc<DockUI>, content: &Box) {
    let window = &ui.window;
    let config = ui.config.borrow();
    
    window.set_exclusive_zone(0);
    let revealer = Revealer::builder().transition_type(RevealerTransitionType::SlideUp).reveal_child(false).child(content).build();
    let trigger_box = Box::builder().height_request(config.trigger_height).build();
    trigger_box.add_css_class(CLASS_TRIGGER_BOX);
    let root_box = Box::builder().orientation(Orientation::Vertical).build();
    root_box.append(&revealer);
    root_box.append(&trigger_box);
    window.set_child(Some(&root_box));

    attach_auto_hide_logic(ui, &revealer, &trigger_box, &root_box, config.exclusive_zone);
}

fn attach_auto_hide_logic(ui: &Rc<DockUI>, revealer: &Revealer, trigger: &Box, root: &Box, zone: i32) {
    let cancel_hide = |ui: &Rc<DockUI>| {
        if let Some(id) = ui.hide_timeout.borrow_mut().take() { id.remove(); }
    };

    let create_motion_ctrl = |ui: Rc<DockUI>| {
        let ctrl = EventControllerMotion::new();
        let ui_enter = ui.clone();
        ctrl.connect_enter(move |_, _, _| cancel_hide(&ui_enter));
        let ui_motion = ui.clone();
        ctrl.connect_motion(move |_, _, _| cancel_hide(&ui_motion));
        ctrl
    };

    let enter_trigger = EventControllerMotion::new();
    let ui_trigger = ui.clone();
    let rev_trigger = revealer.clone();
    enter_trigger.connect_enter(move |_, _, _| {
        cancel_hide(&ui_trigger);
        if !rev_trigger.reveals_child() {
            rev_trigger.set_reveal_child(true);
            ui_trigger.window.set_exclusive_zone(zone);
        }
    });
    trigger.add_controller(enter_trigger);
    root.add_controller(create_motion_ctrl(ui.clone()));

    let leave_root = EventControllerMotion::new();
    let ui_leave = ui.clone();
    let rev_hide = revealer.clone();
    leave_root.connect_leave(move |_| {
        let uil = ui_leave.clone();
        let rh = rev_hide.clone();
        
        // Use 800ms for a better UX while maintaining tooltip stability
        let id = glib::timeout_add_local(std::time::Duration::from_millis(800), move || {
            // Check visibility of ALL popovers
            let launcher_visible = uil.launcher_popover.is_visible();
            let context_visible = uil.context_menu.is_visible();

            if rh.reveals_child() && !launcher_visible && !context_visible {
                rh.set_reveal_child(false);
                uil.window.set_exclusive_zone(0);
            }
            *uil.hide_timeout.borrow_mut() = None;
            glib::ControlFlow::Break
        });
        *ui_leave.hide_timeout.borrow_mut() = Some(id);
    });
    root.add_controller(leave_root);
}
