use gtk4::gio;
use gtk4::prelude::*;

#[derive(Clone, Debug)]
pub struct AppItem {
    pub name: String,
    pub exec: String,
    pub icon: Option<gio::Icon>,
}

pub fn get_all_apps() -> Vec<AppItem> {
    gio::AppInfo::all()
        .into_iter()
        .filter(|app| app.should_show())
        .map(|app| AppItem {
            name: app.display_name().to_string(),
            exec: app.executable()
                .to_string_lossy()
                .into_owned(),
            icon: app.icon(),
        })
        .collect()
}

pub fn launch_app(app_name: &str) {
    let apps = gio::AppInfo::all();
    if let Some(app) = apps.into_iter().find(|a| a.display_name() == app_name) {
        let _ = app.launch(&[], gio::AppLaunchContext::NONE);
    }
}
