use crate::config::Config;
use crate::hypr::{HyprEvent, WindowInfo};
use crate::launcher::{self, AppItem};
use gtk4::gdk::Display;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, Button, CssProvider, EventControllerMotion, GestureClick,
    IconTheme, Image, Label, ListBox, ListBoxRow, Orientation, Popover, Revealer,
    RevealerTransitionType, ScrolledWindow, SearchEntry, Separator, Widget,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

// --- Constants ---
const DOCK_NAMESPACE: &str = "hyprdock";
const LAUNCHER_ICON_SIZE: i32 = 38;
const FALLBACK_ICON_NAME: &str = "application-x-executable";

// --- CSS Classes ---
const CLASS_DOCK_WINDOW: &str = "dock-window";
const CLASS_DOCK_CONTENT: &str = "dock-content";
const CLASS_DOCK_SEPARATOR: &str = "dock-separator";
const CLASS_TRIGGER_BOX: &str = "trigger-box";
const CLASS_TASKBAR_ITEM: &str = "taskbar-item";
const CLASS_TASKBAR_ICON: &str = "taskbar-icon";
const CLASS_LAUNCHER_BUTTON: &str = "launcher-button";
const CLASS_LAUNCHER_POPOVER: &str = "launcher-popover";
const CLASS_LAUNCHER_LIST: &str = "launcher-list";
const CLASS_LAUNCHER_ITEM: &str = "launcher-item";
const CLASS_PINNED_APP: &str = "pinned-app";
const CLASS_OPEN_APP: &str = "open-app";
const CLASS_FOCUSED_APP: &str = "focused-app";
const CLASS_CONTEXT_MENU: &str = "context-menu";
const CLASS_CONTEXT_MENU_ITEM: &str = "context-menu-item";
const CLASS_SECTION_MENU: &str = "section-menu";
const CLASS_SECTION_FAVORITES: &str = "section-favorites";
const CLASS_SECTION_TASKS: &str = "section-tasks";

// --- UI Models ---

/// Handle to the UI widgets for state updates.
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

// --- Initialization & Setup ---

pub fn initialize_styling() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("style.css"));
    
    if let Some(proj_dirs) = crate::config::get_project_dirs() {
        let css_path = proj_dirs.config_dir().join("style.css");
        if css_path.exists() {
            if let Ok(content) = fs::read_to_string(&css_path) {
                provider.load_from_data(&content);
            }
        }
    }

    if let Some(display) = Display::default() {
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
    window.set_layer(Layer::Top);
    window.set_namespace(Some(DOCK_NAMESPACE));
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_keyboard_mode(KeyboardMode::OnDemand);
}

fn apply_background_color(widget: &impl IsA<Widget>, color: &str) {
    let provider = CssProvider::new();
    let css = format!(".{} {{ background-color: {}; }}", CLASS_DOCK_CONTENT, color);
    provider.load_from_data(&css);
    widget.style_context().add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_USER);
}

// --- Layout Creation ---

fn create_layout(config: &Config) -> (Box, Box, Box, HashMap<String, Button>, Popover) {
    let content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .halign(gtk4::Align::Center)
        .margin_top(4)
        .margin_bottom(4)
        .build();
    content.add_css_class(CLASS_DOCK_CONTENT);

    // 1. Menu
    let (launcher_button, launcher_popover) = create_launcher();
    let menu_section = create_section(CLASS_SECTION_MENU, 0, &[&launcher_button]);
    content.append(&menu_section);
    content.append(&create_separator());

    // 2. Favorites
    let pins_box = Box::builder().orientation(Orientation::Horizontal).spacing(8).build();
    let pins_map = populate_pinned_apps(&pins_box, config);
    let favorites_section = create_section(CLASS_SECTION_FAVORITES, 8, &[&pins_box]);
    content.append(&favorites_section);
    content.append(&create_separator());

    // 3. Tasks
    let taskbar_box = Box::builder().orientation(Orientation::Horizontal).spacing(8).build();
    let tasks_section = create_section(CLASS_SECTION_TASKS, 8, &[&taskbar_box]);
    content.append(&tasks_section);

    (content, taskbar_box, pins_box, pins_map, launcher_popover)
}

fn create_section(class: &str, spacing: i32, children: &[&impl IsA<Widget>]) -> Box {
    let section = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(spacing)
        .build();
    section.add_css_class(class);
    for child in children {
        section.append(*child);
    }
    section
}

fn create_separator() -> Separator {
    let sep = Separator::new(Orientation::Vertical);
    sep.add_css_class(CLASS_DOCK_SEPARATOR);
    sep
}

// --- Component Builders ---

fn create_launcher() -> (Button, Popover) {
    let button = create_base_button(Some(LAUNCHER_ICON_SIZE), Some("open-menu-symbolic"));
    button.add_css_class(CLASS_LAUNCHER_BUTTON);

    let popover = Popover::builder().position(gtk4::PositionType::Top).autohide(true).build();
    popover.add_css_class(CLASS_LAUNCHER_POPOVER);
    popover.set_parent(&button);

    let launcher_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .width_request(300)
        .height_request(400)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();

    let search_entry = SearchEntry::builder().placeholder_text("Search applications...").build();
    launcher_box.append(&search_entry);

    let scrolled = ScrolledWindow::builder().propagate_natural_height(true).build();
    launcher_box.append(&scrolled);

    let list_box = ListBox::builder().selection_mode(gtk4::SelectionMode::None).build();
    list_box.add_css_class(CLASS_LAUNCHER_LIST);
    scrolled.set_child(Some(&list_box));

    let lb_clone = list_box.clone();
    let p_clone = popover.clone();
    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().to_lowercase();
        let apps: Vec<AppItem> = launcher::get_all_apps().into_iter()
            .filter(|a| a.name.to_lowercase().contains(&query)).collect();
        update_launcher_list(&lb_clone, &apps, &p_clone);
    });

    let lb_init = list_box.clone();
    let p_init = popover.clone();
    popover.connect_show(move |_| {
        update_launcher_list(&lb_init, &launcher::get_all_apps(), &p_init);
    });

    popover.set_child(Some(&launcher_box));
    
    let p_click = popover.clone();
    button.connect_clicked(move |_| p_click.popup());

    (button, popover)
}

fn update_launcher_list(list_box: &ListBox, apps: &[AppItem], popover: &Popover) {
    while let Some(child) = list_box.first_child() { list_box.remove(&child); }
    for app in apps {
        let row = ListBoxRow::new();
        row.add_css_class(CLASS_LAUNCHER_ITEM);
        let item_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(6)
            .margin_end(6)
            .build();
        
        let icon = Image::builder().pixel_size(LAUNCHER_ICON_SIZE).build();
        if let Some(gicon) = &app.icon { icon.set_from_gicon(gicon); }
        else { icon.set_icon_name(Some(FALLBACK_ICON_NAME)); }

        let label = Label::new(Some(&app.name));
        label.set_halign(gtk4::Align::Start);

        item_box.append(&icon);
        item_box.append(&label);
        row.set_child(Some(&item_box));

        let app_name = app.name.clone();
        let p_clone = popover.clone();
        row.connect_activate(move |_| {
            launcher::launch_app(&app_name);
            p_clone.popdown();
        });
        list_box.append(&row);
    }
}

fn create_base_button(icon_size: Option<i32>, icon_name: Option<&str>) -> Button {
    let button = Button::builder().has_frame(false).build();
    if let Some(size) = icon_size {
        let icon = Image::builder().pixel_size(size).build();
        if let Some(name) = icon_name { icon.set_icon_name(Some(name)); }
        button.set_child(Some(&icon));
    }
    button
}

fn populate_pinned_apps(container: &Box, config: &Config) -> HashMap<String, Button> {
    let mut map = HashMap::new();
    for class in &config.pinned_apps {
        let pin = create_pinned_button(class, config.icon_size);
        container.append(&pin);
        map.insert(class.to_lowercase(), pin);
    }
    map
}

fn create_pinned_button(class: &str, icon_size: i32) -> Button {
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

// --- Interaction Logic ---

fn setup_auto_hide(window: &ApplicationWindow, content: &Box, l_pop: &Popover, c_pop: &Popover, config: &Config) {
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

// --- Event Handling ---

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

fn update_taskbar(ui: &Rc<DockUI>, windows: Vec<WindowInfo>) {
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

fn clear_container(container: &Box) {
    while let Some(child) = container.first_child() { container.remove(&child); }
}

fn create_taskbar_item(ui: &Rc<DockUI>, win: &WindowInfo, is_focused: bool) -> Button {
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

// --- Context Menu ---

fn attach_context_menu(ui: &Rc<DockUI>, button: &Button, class: &str, is_pinned: bool) {
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
        let ui_close = ui.clone();
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

fn refresh_dock(ui: &Rc<DockUI>) {
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
    update_taskbar(ui, windows);
}

// --- Icon Resolution ---

fn resolve_gicon(class: &str) -> Option<gio::Icon> {
    if class.is_empty() { return None; }
    let class_lower = class.to_lowercase();
    if let Some(display) = Display::default() {
        let theme = IconTheme::for_display(&display);
        if theme.has_icon(&class_lower) { return Some(gio::ThemedIcon::new(&class_lower).upcast()); }
    }
    let apps = gio::AppInfo::all();
    for app in &apps {
        let id = std::panic::catch_unwind(|| app.id()).unwrap_or(None).map(|i| i.to_string().to_lowercase()).unwrap_or_default();
        let name = std::panic::catch_unwind(|| app.name().to_lowercase()).unwrap_or_default();
        if (!name.is_empty() && class_lower.contains(&name)) || (!id.is_empty() && class_lower.contains(&id)) {
            if let Ok(icon) = std::panic::catch_unwind(|| app.icon()) { if let Some(icon) = icon { return Some(icon); } }
        }
    }
    for app in apps {
        if let Ok(exec) = std::panic::catch_unwind(|| app.executable()) {
            let exec_str = exec.to_string_lossy().to_lowercase();
            if !exec_str.is_empty() && class_lower.contains(&exec_str) {
                if let Ok(icon) = std::panic::catch_unwind(|| app.icon()) { if let Some(icon) = icon { return Some(icon); } }
            }
        }
    }
    None
}
