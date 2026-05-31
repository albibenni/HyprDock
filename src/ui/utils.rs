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
