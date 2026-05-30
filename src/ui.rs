use crate::hypr::HyprEvent;
use crate::config::Config;
use gtk4::gdk::Display;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, CssProvider, EventControllerMotion, Label, Orientation,
    Revealer, RevealerTransitionType,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::fs;

// --- Constants: Layout ---
const DEFAULT_EXCLUSIVE_ZONE: i32 = 50;
const TRIGGER_ZONE_HEIGHT: i32 = 1;
const CONTENT_SPACING: i32 = 20;
const CONTENT_MARGIN: i32 = 10;
const DOCK_NAMESPACE: &str = "hyprdock";

// --- Constants: CSS Classes ---
const CLASS_DOCK_WINDOW: &str = "dock-window";
const CLASS_DOCK_CONTENT: &str = "dock-content";
const CLASS_STATUS_LABEL: &str = "status-label";
const CLASS_TRIGGER_BOX: &str = "trigger-box";

// --- UI Construction ---

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
pub fn build_ui(app: &Application, config: &Config) -> (ApplicationWindow, Label) {
    let window = create_window(app);
    setup_layer_shell(&window);
    window.add_css_class(CLASS_DOCK_WINDOW);

    let (content, status_label) = create_dock_content();

    if config.auto_hide {
        setup_auto_hide(&window, &content);
    } else {
        setup_static_dock(&window, &content);
    }

    window.present();
    (window, status_label)
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
}

fn create_dock_content() -> (Box, Label) {
    let content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(CONTENT_SPACING)
        .halign(gtk4::Align::Center)
        .margin_top(CONTENT_MARGIN)
        .margin_bottom(CONTENT_MARGIN)
        .build();
    content.add_css_class(CLASS_DOCK_CONTENT);

    let status_label = Label::new(Some("Waiting for Hyprland..."));
    status_label.add_css_class(CLASS_STATUS_LABEL);
    content.append(&status_label);

    (content, status_label)
}

fn setup_static_dock(window: &ApplicationWindow, content: &Box) {
    window.set_exclusive_zone(DEFAULT_EXCLUSIVE_ZONE);
    window.set_child(Some(content));
}

fn setup_auto_hide(window: &ApplicationWindow, content: &Box) {
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
        rev_clone.set_reveal_child(true);
        win_clone.set_exclusive_zone(DEFAULT_EXCLUSIVE_ZONE);
    });
    trigger.add_controller(enter_controller);

    let leave_controller = EventControllerMotion::new();
    let win_clone = window.clone();
    let rev_clone = revealer.clone();
    leave_controller.connect_leave(move |_| {
        rev_clone.set_reveal_child(false);
        win_clone.set_exclusive_zone(0);
    });
    root.add_controller(leave_controller);
}

// --- Event Handling ---

pub fn handle_event(event: HyprEvent, label: &Label) {
    match event {
        HyprEvent::WorkspaceChanged(ws) => {
            label.set_text(&format!("Workspace: {}", ws));
        }
        HyprEvent::ActiveWindowChanged(win) => {
            label.set_text(&format!("Active: {}", win));
        }
    }
}
