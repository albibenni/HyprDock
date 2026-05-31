use gtk4::prelude::*;
use gtk4::{Button, Box, Image};
use crate::config::Config;
use crate::ui::constants::*;
use crate::ui::utils::{create_base_button, resolve_gicon};
use crate::ui::DockUI;
use crate::ui::context_menu::attach_context_menu;
use crate::hypr::WindowInfo;
use crate::launcher;
use std::collections::HashMap;
use std::rc::Rc;

pub fn populate_pinned_apps(container: &Box, config: &Config) -> HashMap<String, Button> {
    let mut map = HashMap::new();
    for class in &config.pinned_apps {
        let pin = create_pinned_button(class, config.icon_size);
        container.append(&pin);
        map.insert(class.to_lowercase(), pin);
    }
    map
}

pub fn create_pinned_button(class: &str, icon_size: i32) -> Button {
    let button = create_base_button(Some(icon_size), None);
    button.add_css_class(CLASS_TASKBAR_ITEM);
    button.add_css_class(CLASS_PINNED_APP);
    button.set_tooltip_text(Some(class));

    if let Some(icon_child) = button.child().and_downcast::<Image>() {
        if let Some(gicon) = resolve_gicon(class) { icon_child.set_from_gicon(&gicon); }
        else { icon_child.set_icon_name(Some(FALLBACK_ICON_NAME)); }
    }

    let class_clone = class.to_string();
    button.connect_clicked(move |_| {
        if let Some(addr) = crate::hypr::get_first_window_by_class(&class_clone) {
            crate::hypr::focus_window(&addr);
        } else {
            launcher::launch_app_by_class(&class_clone);
        }
    });
    button
}

pub fn create_taskbar_item(ui: &Rc<DockUI>, win: &WindowInfo, is_focused: bool) -> Button {
    let button = create_base_button(Some(ui.config.borrow().icon_size), None);
    button.add_css_class(CLASS_TASKBAR_ITEM);
    button.add_css_class(CLASS_OPEN_APP);
    if is_focused { button.add_css_class(CLASS_FOCUSED_APP); }
    button.set_tooltip_text(Some(&win.title));

    if let Some(icon) = button.child().and_downcast::<Image>() {
        if let Some(gicon) = resolve_gicon(&win.class) { icon.set_from_gicon(&gicon); }
        else { icon.set_icon_name(Some(FALLBACK_ICON_NAME)); }
    }

    let addr = win.address.clone();
    button.connect_clicked(move |_| crate::hypr::focus_window(&addr));
    attach_context_menu(ui, &button, &win.class, false);
    button
}
