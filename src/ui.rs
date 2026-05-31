use crate::config::Config;
use crate::hypr::{HyprEvent, WindowInfo};
use crate::launcher::{self, AppItem};
use gtk4::gdk::Display;
use gtk4::gio;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, Button, CssProvider, EventControllerMotion, IconTheme,
    Image, Label, ListBox, ListBoxRow, Orientation, Popover, Revealer,
    RevealerTransitionType, ScrolledWindow, SearchEntry, Separator,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

// --- Constants: Layout ---
const DOCK_NAMESPACE: &str = "hyprdock";

// --- Constants: Icons ---
const LAUNCHER_ICON_SIZE: i32 = 38;
const FALLBACK_ICON_NAME: &str = "application-x-executable";

// --- Constants: CSS Classes ---
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

// --- UI Components ---

/// Handle to the UI widgets for state updates.
pub struct DockUI {
    pub window: ApplicationWindow,
    pub taskbar_box: Box,
    pub pins_box: Box,
    pub pins_map: HashMap<String, Button>,
    pub launcher_popover: Popover,
    pub active_address: RefCell<Option<String>>,
    pub last_windows: RefCell<Vec<WindowInfo>>,
    pub config: Config,
}

/// Initializes styling by loading internal and optional external CSS.
pub fn initialize_styling() {
    let provider = CssProvider::new();
    load_internal_css(&provider);
    load_external_css(&provider);
    apply_css_to_display(&provider);
}

fn load_internal_css(provider: &CssProvider) {
    provider.load_from_data(include_str!("style.css"));
}

fn load_external_css(provider: &CssProvider) {
    if let Some(proj_dirs) = crate::config::get_project_dirs() {
        let css_path = proj_dirs.config_dir().join("style.css");
        if css_path.exists() {
            if let Ok(content) = fs::read_to_string(&css_path) {
                provider.load_from_data(&content);
            }
        }
    }
}

fn apply_css_to_display(provider: &CssProvider) {
    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

/// Main entry point for building the Dock UI.
pub fn build_ui(app: &Application, config: &Config) -> Rc<DockUI> {
    let window = create_window(app);
    setup_layer_shell(&window);
    window.add_css_class(CLASS_DOCK_WINDOW);

    let (content, taskbar_box, pins_box, pins_map, launcher_popover) = create_dock_content_layout(config);

    // Dynamic background color from config
    let color_provider = CssProvider::new();
    let css = format!(".{} {{ background-color: {}; }}", CLASS_DOCK_CONTENT, config.background_color);
    color_provider.load_from_data(&css);
    content.style_context().add_provider(&color_provider, gtk4::STYLE_PROVIDER_PRIORITY_USER);

    if config.auto_hide {
        setup_auto_hide_behavior(&window, &content, &launcher_popover, config);
    } else {
        setup_static_behavior(&window, &content, config);
    }

    window.present();

    Rc::new(DockUI {
        window,
        taskbar_box,
        pins_box,
        pins_map,
        launcher_popover,
        active_address: RefCell::new(None),
        last_windows: RefCell::new(Vec::new()),
        config: config.clone(),
    })
}

fn create_window(app: &Application) -> ApplicationWindow {
    ApplicationWindow::builder()
        .application(app)
        .title("hyprdock")
        .build()
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

fn create_dock_content_layout(config: &Config) -> (Box, Box, Box, HashMap<String, Button>, Popover) {
    let content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(gtk4::Align::Center)
        .margin_top(4) // Spacing from screen edge
        .margin_bottom(4)
        .build();
    content.add_css_class(CLASS_DOCK_CONTENT);

    // Launcher
    let (launcher_button, launcher_popover) = create_launcher();
    content.append(&launcher_button);

    // Separator between launcher and apps
    let launcher_sep = Separator::new(Orientation::Vertical);
    launcher_sep.add_css_class(CLASS_DOCK_SEPARATOR);
    content.append(&launcher_sep);

    // Pinned Apps (Preferred)
    let pins_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let pins_map = populate_pinned_apps(&pins_box, config);
    content.append(&pins_box);

    // Vertical Separator (macOS style)
    let separator = Separator::new(Orientation::Vertical);
    separator.add_css_class(CLASS_DOCK_SEPARATOR);
    content.append(&separator);

    // Taskbar
    let taskbar_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    content.append(&taskbar_box);

    (content, taskbar_box, pins_box, pins_map, launcher_popover)
}

fn populate_pinned_apps(container: &Box, config: &Config) -> HashMap<String, Button> {
    let mut map = HashMap::new();
    for class in &config.pinned_apps {
        let pin = create_pinned_app_button(class, config.icon_size);
        container.append(&pin);
        map.insert(class.to_lowercase(), pin);
    }
    map
}

fn create_pinned_app_button(class: &str, icon_size: i32) -> Button {
    let button = Button::builder()
        .has_frame(false)
        .build();
    button.add_css_class(CLASS_TASKBAR_ITEM);
    button.add_css_class(CLASS_PINNED_APP);
    button.set_tooltip_text(Some(class));

    let icon = Image::builder()
        .pixel_size(icon_size)
        .build();
    icon.add_css_class(CLASS_TASKBAR_ICON);

    if let Some(gicon) = resolve_gicon(class) {
        icon.set_from_gicon(&gicon);
    } else {
        icon.set_icon_name(Some(FALLBACK_ICON_NAME));
    }

    button.set_child(Some(&icon));

    let class_clone = class.to_string();
    button.connect_clicked(move |_| {
        if let Some(addr) = crate::hypr::get_first_window_by_class(&class_clone) {
            crate::hypr::focus_window(&addr);
        } else {
            crate::launcher::launch_app_by_class(&class_clone);
        }
    });

    button
}

fn create_launcher() -> (Button, Popover) {
    let button = Button::builder()
        .has_frame(false)
        .build();
    button.add_css_class(CLASS_LAUNCHER_BUTTON);

    let icon = Image::builder()
        .icon_name("open-menu-symbolic")
        .pixel_size(LAUNCHER_ICON_SIZE)
        .build();
    button.set_child(Some(&icon));

    let popover = Popover::builder()
        .position(gtk4::PositionType::Top)
        .autohide(true)
        .build();
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

    let search_entry = SearchEntry::builder()
        .placeholder_text("Search applications...")
        .build();
    launcher_box.append(&search_entry);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .propagate_natural_height(true)
        .build();
    launcher_box.append(&scrolled);

    let list_box = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .build();
    list_box.add_css_class(CLASS_LAUNCHER_LIST);
    scrolled.set_child(Some(&list_box));

    // Search logic
    let list_box_clone = list_box.clone();
    let popover_clone = popover.clone();
    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().to_lowercase();
        let filtered_apps: Vec<AppItem> = launcher::get_all_apps()
            .into_iter()
            .filter(|app| app.name.to_lowercase().contains(&query))
            .collect();
        populate_launcher_list(&list_box_clone, &filtered_apps, &popover_clone);
    });

    // Populate LAZILY when opened
    let list_box_init = list_box.clone();
    let popover_init = popover.clone();
    popover.connect_show(move |_| {
        let apps = launcher::get_all_apps();
        populate_launcher_list(&list_box_init, &apps, &popover_init);
    });

    popover.set_child(Some(&launcher_box));
    
    let popover_click = popover.clone();
    button.connect_clicked(move |_| {
        popover_click.popup();
    });

    (button, popover)
}

fn populate_launcher_list(list_box: &ListBox, apps: &[AppItem], popover: &Popover) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

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

        let icon = Image::builder()
            .pixel_size(LAUNCHER_ICON_SIZE)
            .build();
        if let Some(gicon) = &app.icon {
            icon.set_from_gicon(gicon);
        } else {
            icon.set_icon_name(Some(FALLBACK_ICON_NAME));
        }

        let label = Label::new(Some(&app.name));
        label.set_halign(gtk4::Align::Start);

        item_box.append(&icon);
        item_box.append(&label);
        row.set_child(Some(&item_box));

        let app_name = app.name.clone();
        let popover_clone = popover.clone();
        row.connect_activate(move |_| {
            launcher::launch_app(&app_name);
            popover_clone.popdown();
        });

        list_box.append(&row);
    }
}

fn setup_static_behavior(window: &ApplicationWindow, content: &Box, config: &Config) {
    window.set_exclusive_zone(config.exclusive_zone);
    window.set_child(Some(content));
}

fn setup_auto_hide_behavior(window: &ApplicationWindow, content: &Box, popover: &Popover, config: &Config) {
    window.set_exclusive_zone(0);

    let revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideUp)
        .reveal_child(false)
        .child(content)
        .build();

    let trigger_box = Box::builder()
        .height_request(config.trigger_height)
        .build();
    trigger_box.add_css_class(CLASS_TRIGGER_BOX);

    let root_box = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    root_box.append(&revealer);
    root_box.append(&trigger_box);
    window.set_child(Some(&root_box));

    attach_auto_hide_controllers(window, &revealer, &trigger_box, &root_box, popover, config.exclusive_zone);
}

fn attach_auto_hide_controllers(
    window: &ApplicationWindow,
    revealer: &Revealer,
    trigger: &Box,
    root: &Box,
    popover: &Popover,
    exclusive_zone: i32,
) {
    let enter_controller = EventControllerMotion::new();
    let win_clone = window.clone();
    let rev_clone = revealer.clone();
    enter_controller.connect_enter(move |_, _, _| {
        if !rev_clone.reveals_child() {
            rev_clone.set_reveal_child(true);
            win_clone.set_exclusive_zone(exclusive_zone);
        }
    });
    trigger.add_controller(enter_controller);

    let leave_controller = EventControllerMotion::new();
    let win_clone = window.clone();
    let rev_clone = revealer.clone();
    let popover_clone = popover.clone();
    leave_controller.connect_leave(move |_| {
        // ONLY hide if the launcher popover is NOT visible
        if rev_clone.reveals_child() && !popover_clone.is_visible() {
            rev_clone.set_reveal_child(false);
            win_clone.set_exclusive_zone(0);
        }
    });
    root.add_controller(leave_controller);
}

// --- Event Handling ---

/// Routes Hyprland events to the appropriate UI update logic.
pub fn handle_event(event: HyprEvent, ui: &Rc<DockUI>) {
    match event {
        HyprEvent::WorkspaceChanged(_) => {
            // Nothing to do for now
        }
        HyprEvent::ActiveWindowChanged(addr) => {
            *ui.active_address.borrow_mut() = addr;
            let windows = ui.last_windows.borrow().clone();
            update_taskbar(ui, windows);
        }
        HyprEvent::WindowListUpdate(windows) => {
            *ui.last_windows.borrow_mut() = windows.clone();
            update_taskbar(ui, windows);
        }
        HyprEvent::Error(err) => {
            eprintln!("Hyprland Error: {}", err);
        }
    }
}

fn update_taskbar(ui: &DockUI, windows: Vec<WindowInfo>) {
    clear_container(&ui.taskbar_box);
    let active_addr = ui.active_address.borrow();

    // Reset all pinned apps states
    for button in ui.pins_map.values() {
        button.remove_css_class(CLASS_OPEN_APP);
        button.remove_css_class(CLASS_FOCUSED_APP);
    }

    for win in windows {
        let is_focused = active_addr.as_ref() == Some(&win.address);
        let class_lower = win.class.to_lowercase();

        if let Some(pinned_btn) = ui.pins_map.get(&class_lower) {
            // Update pinned app state
            pinned_btn.add_css_class(CLASS_OPEN_APP);
            if is_focused {
                pinned_btn.add_css_class(CLASS_FOCUSED_APP);
            }
        } else {
            // Add to taskbar section if not pinned
            let item = create_taskbar_item(&win, is_focused, ui.config.icon_size);
            ui.taskbar_box.append(&item);
        }
    }
}

fn clear_container(container: &Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

fn create_taskbar_item(win: &WindowInfo, is_focused: bool, icon_size: i32) -> Button {
    let button = Button::builder()
        .has_frame(false)
        .build();
    button.add_css_class(CLASS_TASKBAR_ITEM);
    button.add_css_class(CLASS_OPEN_APP);
    if is_focused {
        button.add_css_class(CLASS_FOCUSED_APP);
    }
    button.set_tooltip_text(Some(&win.title));

    let icon = Image::builder()
        .pixel_size(icon_size)
        .build();
    icon.add_css_class(CLASS_TASKBAR_ICON);

    if let Some(gicon) = resolve_gicon(&win.class) {
        icon.set_from_gicon(&gicon);
    } else {
        icon.set_icon_name(Some(FALLBACK_ICON_NAME));
    }

    button.set_child(Some(&icon));

    let addr = win.address.clone();
    button.connect_clicked(move |_| {
        crate::hypr::focus_window(&addr);
    });

    button
}

fn resolve_gicon(class: &str) -> Option<gio::Icon> {
    if class.is_empty() { return None; }
    let class_lower = class.to_lowercase();
    
    // 1. Fast path: Exact theme lookup
    if let Some(display) = Display::default() {
        let icon_theme = IconTheme::for_display(&display);
        if icon_theme.has_icon(&class_lower) {
            return Some(gio::ThemedIcon::new(&class_lower).upcast());
        }
    }

    // 2. Defensive fuzzy scan of all installed apps
    let apps = gio::AppInfo::all();
    for app in &apps {
        let id = std::panic::catch_unwind(|| app.id()).unwrap_or(None)
            .map(|i| i.to_string().to_lowercase())
            .unwrap_or_default();
        let name = std::panic::catch_unwind(|| app.name().to_lowercase()).unwrap_or_default();
        
        // Match if the app name or ID is part of the window class (common for Web Apps)
        if (!name.is_empty() && class_lower.contains(&name)) || (!id.is_empty() && class_lower.contains(&id)) {
            if let Ok(icon) = std::panic::catch_unwind(|| app.icon()) {
                if let Some(icon) = icon { return Some(icon); }
            }
        }
    }

    // 3. Match by executable name
    for app in apps {
        if let Ok(exec) = std::panic::catch_unwind(|| app.executable()) {
            let exec_str = exec.to_string_lossy().to_lowercase();
            if !exec_str.is_empty() && class_lower.contains(&exec_str) {
                if let Ok(icon) = std::panic::catch_unwind(|| app.icon()) {
                    if let Some(icon) = icon { return Some(icon); }
                }
            }
        }
    }

    None
}
