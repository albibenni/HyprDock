use gtk4::gio;
use gtk4::prelude::*;

#[derive(Clone, Debug)]
pub struct AppItem {
    pub name: String,
    pub exec: String,
    pub icon: Option<gio::Icon>,
}

pub fn get_all_apps() -> Vec<AppItem> {
    println!("Launcher: Fetching all apps...");
    let mut items = Vec::new();
    
    let apps = gio::AppInfo::all();
    for app in apps {
        // Wrap every call that could return a null pointer in GLib/C
        let should_show = std::panic::catch_unwind(|| app.should_show()).unwrap_or(false);
        if !should_show { continue; }

        let name = std::panic::catch_unwind(|| app.display_name().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
            
        let exec = std::panic::catch_unwind(|| {
            app.executable().to_string_lossy().into_owned()
        }).unwrap_or_else(|_| "unknown".to_string());

        let icon = std::panic::catch_unwind(|| app.icon()).unwrap_or(None);

        items.push(AppItem { name, exec, icon });
    }
    
    println!("Launcher: Found {} apps", items.len());
    items
}

pub fn launch_app(app_name: &str) {
    let apps = gio::AppInfo::all();
    for app in apps {
        let name = std::panic::catch_unwind(|| app.display_name().to_string()).unwrap_or_default();
        if name == app_name {
            let _ = app.launch(&[], gio::AppLaunchContext::NONE);
            return;
        }
    }
}

pub fn launch_app_by_class(class: &str) {
    let class_lower = class.to_lowercase();
    let apps = gio::AppInfo::all();
    
    for app in apps {
        let id = std::panic::catch_unwind(|| app.id()).unwrap_or(None)
            .map(|i| i.to_string().to_lowercase())
            .unwrap_or_default();
            
        let exec = std::panic::catch_unwind(|| app.executable().to_string_lossy().to_lowercase())
            .unwrap_or_default();
            
        if (!id.is_empty() && id.contains(&class_lower)) || exec.contains(&class_lower) {
            let _ = app.launch(&[], gio::AppLaunchContext::NONE);
            return;
        }
    }
}
