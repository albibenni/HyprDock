use crate::config::Config;
use crate::hypr::{HyprEvent, WindowInfo};
use crate::launcher::{self, AppItem};
use gtk4::gdk::Display;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, Button, CssProvider, EventControllerMotion, IconTheme,
    Image, Label, ListBox, ListBoxRow, MenuButton, Orientation, Popover, Revealer,
    RevealerTransitionType, ScrolledWindow, SearchEntry,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::fs;

// --- Constants: Layout ---
const DEFAULT_EXCLUSIVE_ZONE: i32 = 50;
const TRIGGER_ZONE_HEIGHT: i32 = 3;
const CONTENT_SPACING: i32 = 10;
const CONTENT_MARGIN: i32 = 8;
const DOCK_NAMESPACE: &str = "hyprdock";

// --- Constants: Icons ---
const DEFAULT_ICON_SIZE: i32 = 24;
const LAUNCHER_ICON_SIZE: i32 = 32;
const FALLBACK_ICON_NAME: &str = "application-x-executable";

// --- Constants: CSS Classes ---
const CLASS_DOCK_WINDOW: &str = "dock-window";
const CLASS_DOCK_CONTENT: &str = "dock-content";
const CLASS_STATUS_LABEL: &str = "status-label";
const CLASS_TRIGGER_BOX: &str = "trigger-box";
const CLASS_TASKBAR_ITEM: &str = "taskbar-item";
const CLASS_TASKBAR_ICON: &str = "taskbar-icon";
const CLASS_LAUNCHER_BUTTON: &str = "launcher-button";
const CLASS_LAUNCHER_POPOVER: &str = "launcher-popover";
const CLASS_LAUNCHER_LIST: &str = "launcher-list";
const CLASS_LAUNCHER_ITEM: &str = "launcher-item";

// --- UI Components ---

/// Handle to the UI widgets for state updates.
pub struct DockUI {
    pub window: ApplicationWindow,
    pub status_label: Label,
    pub taskbar_box: Box,
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
pub fn build_ui(app: &Application, config: &Config) -> DockUI {
    let window = create_window(app);
    setup_layer_shell(&window);
    window.add_css_class(CLASS_DOCK_WINDOW);

    let (content, status_label, taskbar_box) = create_dock_content_layout();

    if config.auto_hide {
        setup_auto_hide_behavior(&window, &content);
    } else {
        setup_static_behavior(&window, &content);
    }

    window.present();

    DockUI {
        window,
        status_label,
        taskbar_box,
    }
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

fn create_dock_content_layout() -> (Box, Label, Box) {
    let content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(CONTENT_SPACING)
        .halign(gtk4::Align::Center)
        .margin_top(CONTENT_MARGIN)
        .margin_bottom(CONTENT_MARGIN)
        .build();
    content.add_css_class(CLASS_DOCK_CONTENT);

    // Launcher
    let launcher_button = create_launcher_button();
    content.append(&launcher_button);

    let taskbar_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(5)
        .build();
    content.append(&taskbar_box);

    let status_label = Label::new(Some("HyprDock"));
    status_label.add_css_class(CLASS_STATUS_LABEL);
    content.append(&status_label);

    (content, status_label, taskbar_box)
}

fn create_launcher_button() -> MenuButton {
    let button = MenuButton::builder()
        .icon_name("start-here-symbolic")
        .build();
    button.add_css_class(CLASS_LAUNCHER_BUTTON);

    let popover = Popover::builder()
        .position(gtk4::PositionType::Top)
        .autohide(true)
        .build();
    popover.add_css_class(CLASS_LAUNCHER_POPOVER);

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

    // Populate initial list
    let apps = launcher::get_all_apps();
    populate_launcher_list(&list_box, &apps, &popover);

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

    popover.set_child(Some(&launcher_box));
    button.set_popover(Some(&popover));

    button
}

fn populate_launcher_list(list_box: &ListBox, apps: &[AppItem], popover: &Popover) {
    // Clear
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

fn setup_static_behavior(window: &ApplicationWindow, content: &Box) {
    window.set_exclusive_zone(DEFAULT_EXCLUSIVE_ZONE);
    window.set_child(Some(content));
}

fn setup_auto_hide_behavior(window: &ApplicationWindow, content: &Box) {
    window.set_exclusive_zone(0);

    let revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideUp)
        .reveal_child(false)
        .child(content)
        .build();

    let trigger_box = Box::builder()
        .height_request(TRIGGER_ZONE_HEIGHT)
        .build();
    trigger_box.add_css_class(CLASS_TRIGGER_BOX);

    let root_box = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    root_box.append(&revealer);
    root_box.append(&trigger_box);
    window.set_child(Some(&root_box));

    attach_auto_hide_controllers(window, &revealer, &trigger_box, &root_box);
}

fn attach_auto_hide_controllers(
    window: &ApplicationWindow,
    revealer: &Revealer,
    trigger: &Box,
    root: &Box,
) {
    let enter_controller = EventControllerMotion::new();
    let win_clone = window.clone();
    let rev_clone = revealer.clone();
    enter_controller.connect_enter(move |_, _, _| {
        if !rev_clone.reveals_child() {
            println!("Dock: Mouse entered trigger zone -> Revealing");
            rev_clone.set_reveal_child(true);
            win_clone.set_exclusive_zone(DEFAULT_EXCLUSIVE_ZONE);
        }
    });
    trigger.add_controller(enter_controller);

    let leave_controller = EventControllerMotion::new();
    let win_clone = window.clone();
    let rev_clone = revealer.clone();
    leave_controller.connect_leave(move |_| {
        if rev_clone.reveals_child() {
            println!("Dock: Mouse left dock area -> Hiding");
            rev_clone.set_reveal_child(false);
            win_clone.set_exclusive_zone(0);
        }
    });
    root.add_controller(leave_controller);
}

// --- Event Handling ---

/// Routes Hyprland events to the appropriate UI update logic.
pub fn handle_event(event: HyprEvent, ui: &DockUI) {
    match event {
        HyprEvent::WorkspaceChanged(ws) => {
            ui.status_label.set_text(&format!("Workspace: {}", ws));
        }
        HyprEvent::ActiveWindowChanged(win) => {
            let title = win.unwrap_or_else(|| "Desktop".to_string());
            ui.status_label.set_text(&title);
        }
        HyprEvent::WindowListUpdate(windows) => {
            update_taskbar(ui, windows);
        }
        HyprEvent::Error(err) => {
            ui.status_label.set_text(&format!("Error: {}", err));
        }
    }
}

fn update_taskbar(ui: &DockUI, windows: Vec<WindowInfo>) {
    clear_container(&ui.taskbar_box);

    for win in windows {
        let item = create_taskbar_item(&win);
        ui.taskbar_box.append(&item);
    }
}

fn clear_container(container: &Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

fn create_taskbar_item(win: &WindowInfo) -> Button {
    let button = Button::builder().build();
    button.add_css_class(CLASS_TASKBAR_ITEM);
    button.set_tooltip_text(Some(&win.title));

    let content_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();

    let icon_name = resolve_icon_name(&win.class);
    let icon = Image::builder()
        .icon_name(icon_name)
        .pixel_size(DEFAULT_ICON_SIZE)
        .build();
    icon.add_css_class(CLASS_TASKBAR_ICON);

    let label = Label::new(Some(&win.class));

    content_box.append(&icon);
    content_box.append(&label);
    button.set_child(Some(&content_box));

    let addr = win.address.clone();
    button.connect_clicked(move |_| {
        crate::hypr::focus_window(&addr);
    });

    button
}

fn resolve_icon_name(class: &str) -> String {
    let normalized = class.to_lowercase();
    let icon_theme = IconTheme::for_display(&Display::default().expect("Could not get default display"));

    if icon_theme.has_icon(&normalized) {
        normalized
    } else {
        FALLBACK_ICON_NAME.to_string()
    }
}
