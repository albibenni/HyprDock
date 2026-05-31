use gtk4::prelude::*;
use gtk4::{Button, Box, Orientation, GestureClick};
use crate::ui::DockUI;
use crate::ui::constants::*;
use crate::ui::taskbar::create_pinned_button;
use crate::ui::utils::clear_container;
use std::rc::Rc;
use gtk4::glib;

pub fn attach_context_menu(ui: &Rc<DockUI>, button: &Button, class: &str, is_pinned: bool) {
    let gesture = GestureClick::builder().button(3).build();
    let ui_clone = ui.clone();
    let btn_clone = button.clone();
    let class_clone = class.to_string();
    gesture.connect_pressed(move |_, _, _, _| show_context_menu(&ui_clone, &btn_clone, &class_clone, is_pinned));
    button.add_controller(gesture);
}

fn show_context_menu(ui: &Rc<DockUI>, button: &Button, class: &str, is_pinned: bool) {
    let popover = &ui.context_menu;
    if popover.parent().is_some() { popover.unparent(); }

    let menu_box = Box::builder().orientation(Orientation::Vertical).build();
    
    let ui_fav = ui.clone();
    let class_fav = class.to_string();
    let p_fav = popover.clone();
    let fav_btn = Button::builder().has_frame(false).label(if is_pinned { "Remove from Favorites" } else { "Add to Favorites" }).build();
    fav_btn.add_css_class(CLASS_CONTEXT_MENU_ITEM);
    fav_btn.connect_clicked(move |_| {
        if is_pinned { unpin_app(&ui_fav, &class_fav); }
        else { pin_app(&ui_fav, &class_fav); }
        p_fav.popdown();
    });
    menu_box.append(&fav_btn);

    if !crate::hypr::get_all_windows_by_class(class).is_empty() {
        let class_close = class.to_string();
        let p_close = popover.clone();
        let close_btn = Button::builder().has_frame(false).label("Close Application").build();
        close_btn.add_css_class(CLASS_CONTEXT_MENU_ITEM);
        close_btn.connect_clicked(move |_| {
            crate::hypr::close_all_windows_by_class(&class_close);
            p_close.popdown();
        });
        menu_box.append(&close_btn);
    }

    popover.set_child(Some(&menu_box));
    popover.set_parent(button);
    popover.popup();
}

fn pin_app(ui: &Rc<DockUI>, class: &str) {
    let mut config = ui.config.borrow_mut();
    if !config.pinned_apps.iter().any(|p| p.to_lowercase() == class.to_lowercase()) {
        config.pinned_apps.push(class.to_string());
        crate::config::save_config(&config);
        drop(config);
        let uic = ui.clone();
        glib::idle_add_local(move || { refresh_dock(&uic); glib::ControlFlow::Break });
    }
}

fn unpin_app(ui: &Rc<DockUI>, class: &str) {
    let mut config = ui.config.borrow_mut();
    let original_len = config.pinned_apps.len();
    config.pinned_apps.retain(|p| p.to_lowercase() != class.to_lowercase());
    if config.pinned_apps.len() < original_len {
        crate::config::save_config(&config);
        drop(config);
        let uic = ui.clone();
        glib::idle_add_local(move || { refresh_dock(&uic); glib::ControlFlow::Break });
    }
}

pub fn refresh_dock(ui: &Rc<DockUI>) {
    let config = ui.config.borrow().clone();
    clear_container(&ui.pins_box);
    let mut pins_map = ui.pins_map.borrow_mut();
    pins_map.clear();
    for class in &config.pinned_apps {
        let pin = create_pinned_button(class, config.icon_size);
        ui.pins_box.append(&pin);
        pins_map.insert(class.to_lowercase(), pin.clone());
        attach_context_menu(ui, &pin, class, true);
    }
    let windows = ui.last_windows.borrow().clone();
    drop(pins_map);
    super::update_taskbar(ui, windows);
}
