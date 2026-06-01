use gtk4::prelude::*;
use gtk4::{Box, Revealer, Orientation, RevealerTransitionType, Popover};
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
    let revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideUp)
        .transition_duration(300)
        .reveal_child(false)
        .child(content)
        .valign(gtk4::Align::End) // Anchor expansion to bottom
        .build();
    let trigger_box = Box::builder()
        .height_request(config.trigger_height)
        .width_request(200) // Minimum width for the trigger area
        .halign(gtk4::Align::Center)
        .build();
    trigger_box.add_css_class(CLASS_TRIGGER_BOX);
    let root_box = Box::builder()
        .orientation(Orientation::Vertical)
        .valign(gtk4::Align::End)
        .halign(gtk4::Align::Center)
        .build();
    root_box.append(&revealer);
    root_box.append(&trigger_box);
    window.set_child(Some(&root_box));

    attach_auto_hide_logic(ui, &revealer, &trigger_box, &root_box);
    setup_popover_hide_watchers(ui, &revealer);
}

fn attach_auto_hide_logic(ui: &Rc<DockUI>, revealer: &Revealer, trigger: &Box, root: &Box) {
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
            
            // IF NOT in overlay mode, we set the exclusive zone to push windows.
            // IF in overlay mode (default), we keep it at 0 to stay "over" windows.
            if !ui_trigger.config.borrow().overlay {
                ui_trigger.window.set_exclusive_zone(ui_trigger.config.borrow().exclusive_zone);
            }
        }
    });
    trigger.add_controller(enter_trigger);
    root.add_controller(create_motion_ctrl(ui.clone()));

    let leave_root = EventControllerMotion::new();
    let ui_leave = ui.clone();
    let rev_hide = revealer.clone();
    leave_root.connect_leave(move |_| {
        trigger_hide_timeout(&ui_leave, &rev_hide, 500);
    });
    root.add_controller(leave_root);
}

fn trigger_hide_timeout(ui: &Rc<DockUI>, revealer: &Revealer, ms: u32) {
    if let Some(id) = ui.hide_timeout.borrow_mut().take() { id.remove(); }

    let uil = ui.clone();
    let rh = revealer.clone();
    
    let id = glib::timeout_add_local(std::time::Duration::from_millis(ms as u64), move || {
        let launcher_visible = uil.launcher_popover.is_visible();
        let context_visible = uil.context_menu.is_visible();

        if rh.reveals_child() && !launcher_visible && !context_visible {
            rh.set_reveal_child(false);
            
            // Always reset zone to 0 when hiding
            uil.window.set_exclusive_zone(0);
        }
        *uil.hide_timeout.borrow_mut() = None;
        glib::ControlFlow::Break
    });
    *ui.hide_timeout.borrow_mut() = Some(id);
}

fn setup_popover_hide_watchers(ui: &Rc<DockUI>, revealer: &Revealer) {
    let watch_popover = |popover: &Popover, ui: &Rc<DockUI>, rev: &Revealer| {
        let ui_c = ui.clone();
        let rev_c = rev.clone();
        popover.connect_closed(move |_| {
            trigger_hide_timeout(&ui_c, &rev_c, 100);
        });
    };

    watch_popover(&ui.launcher_popover, ui, revealer);
    watch_popover(&ui.context_menu, ui, revealer);
}
