use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box, Popover, Revealer, Orientation, RevealerTransitionType};
use gtk4_layer_shell::LayerShell;
use crate::config::Config;
use crate::ui::constants::*;
use std::rc::Rc;
use std::cell::RefCell;
use gtk4::glib;
use gtk4::EventControllerMotion;

pub fn setup_auto_hide(window: &ApplicationWindow, content: &Box, l_pop: &Popover, c_pop: &Popover, config: &Config) {
    window.set_exclusive_zone(0);
    let revealer = Revealer::builder().transition_type(RevealerTransitionType::SlideUp).reveal_child(false).child(content).build();
    let trigger_box = Box::builder().height_request(config.trigger_height).build();
    trigger_box.add_css_class(CLASS_TRIGGER_BOX);
    let root_box = Box::builder().orientation(Orientation::Vertical).build();
    root_box.append(&revealer);
    root_box.append(&trigger_box);
    window.set_child(Some(&root_box));

    attach_auto_hide_logic(window, &revealer, &trigger_box, &root_box, l_pop, c_pop, config.exclusive_zone);
}

fn attach_auto_hide_logic(window: &ApplicationWindow, revealer: &Revealer, trigger: &Box, root: &Box, l_pop: &Popover, c_pop: &Popover, zone: i32) {
    let hide_timeout = Rc::new(RefCell::new(None::<glib::SourceId>));
    let cancel_hide = move |timeout: &Rc<RefCell<Option<glib::SourceId>>>| {
        if let Some(id) = timeout.borrow_mut().take() { id.remove(); }
    };

    let create_motion_ctrl = |timeout: Rc<RefCell<Option<glib::SourceId>>>| {
        let ctrl = EventControllerMotion::new();
        let t_enter = timeout.clone();
        ctrl.connect_enter(move |_, _, _| cancel_hide(&t_enter));
        let t_motion = timeout.clone();
        ctrl.connect_motion(move |_, _, _| cancel_hide(&t_motion));
        ctrl
    };

    let enter_trigger = EventControllerMotion::new();
    let win_rev = window.clone();
    let rev_rev = revealer.clone();
    let t_trigger = hide_timeout.clone();
    enter_trigger.connect_enter(move |_, _, _| {
        cancel_hide(&t_trigger);
        if !rev_rev.reveals_child() {
            rev_rev.set_reveal_child(true);
            win_rev.set_exclusive_zone(zone);
        }
    });
    trigger.add_controller(enter_trigger);
    root.add_controller(create_motion_ctrl(hide_timeout.clone()));

    let leave_root = EventControllerMotion::new();
    let win_h = window.clone();
    let rev_h = revealer.clone();
    let lp = l_pop.clone();
    let cp = c_pop.clone();
    let t_root = hide_timeout.clone();
    leave_root.connect_leave(move |_| {
        let wh = win_h.clone();
        let rh = rev_h.clone();
        let lpc = lp.clone();
        let cpc = cp.clone();
        let th = t_root.clone();
        let id = glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
            if rh.reveals_child() && !lpc.is_visible() && !cpc.is_visible() {
                rh.set_reveal_child(false);
                wh.set_exclusive_zone(0);
            }
            *th.borrow_mut() = None;
            glib::ControlFlow::Break
        });
        *t_root.borrow_mut() = Some(id);
    });
    root.add_controller(leave_root);
}
