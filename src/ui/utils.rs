use gtk4::prelude::*;
use gtk4::{Button, Image, Widget, CssProvider};
use gtk4::gdk::Display;
use gtk4::gio;
use gtk4::IconTheme;
use crate::ui::constants::*;

pub fn create_base_button(icon_size: Option<i32>, icon_name: Option<&str>) -> Button {
    let button = Button::builder().has_frame(false).build();
    if let Some(size) = icon_size {
        let icon = Image::builder().pixel_size(size).build();
        if let Some(name) = icon_name { icon.set_icon_name(Some(name)); }
        button.set_child(Some(&icon));
    }
    button
}

pub fn apply_background_color(widget: &impl IsA<Widget>, color: &str) {
    let provider = CssProvider::new();
    let css = format!(".{} {{ background-color: {}; }}", CLASS_DOCK_CONTENT, color);
    provider.load_from_data(&css);
    widget.style_context().add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_USER);
}

pub fn clear_container(container: &gtk4::Box) {
    while let Some(child) = container.first_child() { container.remove(&child); }
}

pub fn resolve_gicon(class: &str) -> Option<gio::Icon> {
    if class.is_empty() { return None; }
    let class_lower = class.to_lowercase();
    
    // 1. Direct Icon Theme Lookup (Fast Path)
    if let Some(display) = Display::default() {
        let theme = IconTheme::for_display(&display);
        
        // Try exact class
        if theme.has_icon(&class_lower) {
            return Some(gio::ThemedIcon::new(&class_lower).upcast());
        }
        
        // Try removing common suffixes (e.g., "code-url-handler" -> "code")
        let clean_class = class_lower
            .replace("-url-handler", "")
            .replace(".desktop", "")
            .replace("org.gnome.", "")
            .replace("com.visualstudio.", "");
            
        if theme.has_icon(&clean_class) {
            return Some(gio::ThemedIcon::new(&clean_class).upcast());
        }
    }

    // 2. Scan Desktop Entries (AppInfo)
    let apps = gio::AppInfo::all();
    
    // First pass: Exact or very close matches on ID/Name
    for app in &apps {
        let id = std::panic::catch_unwind(|| app.id()).unwrap_or(None)
            .map(|i| i.to_string().to_lowercase())
            .unwrap_or_default();
        let name = std::panic::catch_unwind(|| app.name().to_lowercase()).unwrap_or_default();
        
        if id == class_lower || id == format!("{}.desktop", class_lower) || name == class_lower {
            if let Ok(icon) = std::panic::catch_unwind(|| app.icon()) {
                if let Some(icon) = icon { return Some(icon); }
            }
        }
    }

    // Second pass: Fuzzy matches on ID/Name
    for app in &apps {
        let id = std::panic::catch_unwind(|| app.id()).unwrap_or(None)
            .map(|i| i.to_string().to_lowercase())
            .unwrap_or_default();
        let name = std::panic::catch_unwind(|| app.name().to_lowercase()).unwrap_or_default();
        
        if (!id.is_empty() && (id.contains(&class_lower) || class_lower.contains(&id))) 
           || (!name.is_empty() && (name.contains(&class_lower) || class_lower.contains(&name))) {
            if let Ok(icon) = std::panic::catch_unwind(|| app.icon()) {
                if let Some(icon) = icon { return Some(icon); }
            }
        }
    }

    // Third pass: Executable name match
    for app in apps {
        if let Ok(exec) = std::panic::catch_unwind(|| app.executable()) {
            let exec_str = exec.to_string_lossy().to_lowercase();
            if !exec_str.is_empty() && (exec_str.contains(&class_lower) || class_lower.contains(&exec_str)) {
                if let Ok(icon) = std::panic::catch_unwind(|| app.icon()) {
                    if let Some(icon) = icon { return Some(icon); }
                }
            }
        }
    }

    println!("HyprDock: Warning - Could not resolve icon for class: {}", class);
    None
}
