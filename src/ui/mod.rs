pub mod constants;
pub mod utils;
pub mod layout;
pub mod launcher;
pub mod taskbar;
pub mod auto_hide;
pub mod context_menu;

use crate::config::Config;
use crate::hypr::{HyprEvent, WindowInfo};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, Popover, Orientation};
use gtk4_layer_shell::LayerShell;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;

use constants::*;
use utils::{apply_background_color, clear_container};
use layout::{create_section, create_separator};
use launcher::create_launcher;
use taskbar::{populate_pinned_apps, create_taskbar_item};
use auto_hide::setup_auto_hide;
use context_menu::attach_context_menu;

pub struct DockUI {
    pub window: ApplicationWindow,
    pub taskbar_box: Box,
    pub pins_box: Box,
    pub pins_map: RefCell<HashMap<String, Button>>,
    pub launcher_popover: Popover,
    pub context_menu: Popover,
    pub active_address: RefCell<Option<String>>,
    pub last_windows: RefCell<Vec<WindowInfo>>,
    pub config: RefCell<Config>,
}

pub fn initialize_styling() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(include_str!("../style.css"));
    
    if let Some(proj_dirs) = crate::config::get_project_dirs() {
        let css_path = proj_dirs.config_dir().join("style.css");
        if css_path.exists() {
            if let Ok(content) = fs::read_to_string(&css_path) {
                provider.load_from_data(&content);
            }
        }
    }

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

pub fn build_ui(app: &Application, config: &Config) -> Rc<DockUI> {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("hyprdock")
        .build();
    
    setup_layer_shell(&window);
    window.add_css_class(CLASS_DOCK_WINDOW);

    let (content, taskbar_box, pins_box, pins_map, launcher_popover) = create_layout(config);

    apply_background_color(&content, &config.background_color);

    let context_menu = Popover::builder().has_arrow(true).autohide(true).build();
    context_menu.add_css_class(CLASS_CONTEXT_MENU);

    if config.auto_hide {
        setup_auto_hide(&window, &content, &launcher_popover, &context_menu, config);
    } else {
        window.set_exclusive_zone(config.exclusive_zone);
        window.set_child(Some(&content));
    }

    window.present();

    let ui = Rc::new(DockUI {
        window,
        taskbar_box,
        pins_box,
        pins_map: RefCell::new(pins_map),
        launcher_popover,
        context_menu,
        active_address: RefCell::new(None),
        last_windows: RefCell::new(Vec::new()),
        config: RefCell::new(config.clone()),
    });

    for (class, button) in ui.pins_map.borrow().iter() {
        attach_context_menu(&ui, button, class, true);
    }

    ui
}

fn setup_layer_shell(window: &ApplicationWindow) {
    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Top);
    window.set_namespace(Some(DOCK_NAMESPACE));
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
}

fn create_layout(config: &Config) -> (Box, Box, Box, HashMap<String, Button>, Popover) {
    let content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .halign(gtk4::Align::Center)
        .margin_top(4)
        .margin_bottom(4)
        .build();
    content.add_css_class(CLASS_DOCK_CONTENT);

    let (launcher_button, launcher_popover) = create_launcher();
    let menu_section = create_section(CLASS_SECTION_MENU, 0, &[&launcher_button]);
    content.append(&menu_section);
    content.append(&create_separator());

    let pins_box = Box::builder().orientation(Orientation::Horizontal).spacing(8).build();
    let pins_map = populate_pinned_apps(&pins_box, config);
    let favorites_section = create_section(CLASS_SECTION_FAVORITES, 8, &[&pins_box]);
    content.append(&favorites_section);
    content.append(&create_separator());

    let taskbar_box = Box::builder().orientation(Orientation::Horizontal).spacing(8).build();
    let tasks_section = create_section(CLASS_SECTION_TASKS, 8, &[&taskbar_box]);
    content.append(&tasks_section);

    (content, taskbar_box, pins_box, pins_map, launcher_popover)
}

pub fn handle_event(event: HyprEvent, ui: &Rc<DockUI>) {
    match event {
        HyprEvent::WorkspaceChanged(_) => {}
        HyprEvent::ActiveWindowChanged(addr) => {
            *ui.active_address.borrow_mut() = addr;
            update_taskbar(ui, ui.last_windows.borrow().clone());
        }
        HyprEvent::WindowListUpdate(windows) => {
            *ui.last_windows.borrow_mut() = windows.clone();
            update_taskbar(ui, windows);
        }
        HyprEvent::Error(err) => eprintln!("Hyprland Error: {}", err),
    }
}

pub fn update_taskbar(ui: &Rc<DockUI>, windows: Vec<WindowInfo>) {
    clear_container(&ui.taskbar_box);
    let active_addr = ui.active_address.borrow();
    for btn in ui.pins_map.borrow().values() {
        btn.remove_css_class(CLASS_OPEN_APP);
        btn.remove_css_class(CLASS_FOCUSED_APP);
    }
    for win in windows {
        let is_focused = active_addr.as_ref() == Some(&win.address);
        let class_lower = win.class.to_lowercase();
        if let Some(pinned_btn) = ui.pins_map.borrow().get(&class_lower) {
            pinned_btn.add_css_class(CLASS_OPEN_APP);
            if is_focused { pinned_btn.add_css_class(CLASS_FOCUSED_APP); }
        } else {
            ui.taskbar_box.append(&create_taskbar_item(ui, &win, is_focused));
        }
    }
}
