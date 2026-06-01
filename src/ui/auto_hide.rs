use gtk4::prelude::*;
use gtk4::{Box, Revealer, Orientation, RevealerTransitionType, Popover};
use gtk4_layer_shell::LayerShell;
use crate::ui::constants::*;
use crate::ui::DockUI;
use std::rc::Rc;
use gtk4::glib;
use gtk4::EventControllerMotion;
use gtk4::cairo::Region;

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
        .width_request(config.trigger_width) // Use configurable width
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

    let ui_r = ui.clone();
    let trigger_r = trigger_box.clone();
    let revealer_r = revealer.clone();
    revealer.connect_reveal_child_notify(move |_| {
        update_input_region(&ui_r, &revealer_r, &trigger_r);
    });

    // Initial update
    let ui_i = ui.clone();
    let trigger_i = trigger_box.clone();
    let revealer_i = revealer.clone();
    glib::idle_add_local(move || {
        update_input_region(&ui_i, &revealer_i, &trigger_i);
        glib::ControlFlow::Break
    });

    attach_auto_hide_logic(ui, &revealer, &trigger_box, &root_box);
    setup_popover_hide_watchers(ui, &revealer, &trigger_box);
}

fn update_input_region(ui: &DockUI, revealer: &Revealer, trigger: &Box) {
    let window = &ui.window;
    if let Some(surface) = window.surface() {
        let region = Region::create();

        // 1. Always include the trigger box
        if let Some(rect) = trigger.compute_bounds(window) {
            let _ = region.union_rectangle(&gtk4::cairo::RectangleInt::new(
                rect.x() as i32,
                rect.y() as i32,
                rect.width() as i32,
                rect.height() as i32,
            ));
        }

        // 2. If revealed or revealing, include the content area
        if revealer.reveals_child() || revealer.is_child_revealed() {
            if let Some(rect) = revealer.compute_bounds(window) {
                let _ = region.union_rectangle(&gtk4::cairo::RectangleInt::new(
                    rect.x() as i32,
                    rect.y() as i32,
                    rect.width() as i32,
                    rect.height() as i32,
                ));
            }
        }

        if region.is_empty() {
            surface.set_input_region(None);
        } else {
            surface.set_input_region(Some(&region));
        }
    }
}

fn attach_auto_hide_logic(ui: &Rc<DockUI>, revealer: &Revealer, trigger: &Box, root: &Box) {
    let cancel_hide = |ui: &Rc<DockUI>| {
        if let Some(id) = ui.hide_timeout.borrow_mut().take() { id.remove(); }
    };

    let create_motion_ctrl = |ui: Rc<DockUI>, t: Box, r: Revealer| {
        let ctrl = EventControllerMotion::new();
        let ui_enter = ui.clone();
        let tc = t.clone();
        let rc = r.clone();
        ctrl.connect_enter(move |_, _, _| {
            cancel_hide(&ui_enter);
            update_input_region(&ui_enter, &rc, &tc);
        });
        let ui_motion = ui.clone();
        let tc2 = t.clone();
        let rc2 = r.clone();
        ctrl.connect_motion(move |_, _, _| {
            cancel_hide(&ui_motion);
            update_input_region(&ui_motion, &rc2, &tc2);
        });
        ctrl
    };

    let enter_trigger = EventControllerMotion::new();
    let ui_trigger = ui.clone();
    let rev_trigger = revealer.clone();
    let trigger_box_clone = trigger.clone();
    enter_trigger.connect_enter(move |_, _, _| {
        cancel_hide(&ui_trigger);
        if !rev_trigger.reveals_child() {
            rev_trigger.set_reveal_child(true);
            
            if !ui_trigger.config.borrow().overlay {
                ui_trigger.window.set_exclusive_zone(ui_trigger.config.borrow().exclusive_zone);
            }
            update_input_region(&ui_trigger, &rev_trigger, &trigger_box_clone);
        }
    });
    trigger.add_controller(enter_trigger);
    root.add_controller(create_motion_ctrl(ui.clone(), trigger.clone(), revealer.clone()));

    let leave_root = EventControllerMotion::new();
    let ui_leave = ui.clone();
    let rev_hide = revealer.clone();
    let trigger_box_leave = trigger.clone();
    leave_root.connect_leave(move |_| {
        trigger_hide_timeout(&ui_leave, &rev_hide, &trigger_box_leave, 500);
    });
    root.add_controller(leave_root);
}

fn trigger_hide_timeout(ui: &Rc<DockUI>, revealer: &Revealer, trigger: &Box, ms: u32) {
    if let Some(id) = ui.hide_timeout.borrow_mut().take() { id.remove(); }

    let uil = ui.clone();
    let rh = revealer.clone();
    let tr = trigger.clone();
    
    let id = glib::timeout_add_local(std::time::Duration::from_millis(ms as u64), move || {
        let launcher_visible = uil.launcher_popover.is_visible();
        let context_visible = uil.context_menu.is_visible();

        if rh.reveals_child() && !launcher_visible && !context_visible {
            rh.set_reveal_child(false);
            
            // Always reset zone to 0 when hiding
            uil.window.set_exclusive_zone(0);
            update_input_region(&uil, &rh, &tr);
        }
        *uil.hide_timeout.borrow_mut() = None;
        glib::ControlFlow::Break
    });
    *ui.hide_timeout.borrow_mut() = Some(id);
}

fn setup_popover_hide_watchers(ui: &Rc<DockUI>, revealer: &Revealer, trigger: &Box) {
    let watch_popover = |popover: &Popover, ui: &Rc<DockUI>, rev: &Revealer, tr: &Box| {
        let ui_c = ui.clone();
        let rev_c = rev.clone();
        let tr_c = tr.clone();
        popover.connect_closed(move |_| {
            trigger_hide_timeout(&ui_c, &rev_c, &tr_c, 100);
        });
    };

    watch_popover(&ui.launcher_popover, ui, revealer, trigger);
    watch_popover(&ui.context_menu, ui, revealer, trigger);
}
