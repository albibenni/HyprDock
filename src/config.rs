use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_auto_hide")]
    pub auto_hide: bool,

    #[serde(default = "default_pinned_apps")]
    pub pinned_apps: Vec<String>,

    #[serde(default = "default_icon_size")]
    pub icon_size: i32,

    #[serde(default = "default_exclusive_zone")]
    pub exclusive_zone: i32,

    #[serde(default = "default_trigger_height")]
    pub trigger_height: i32,

    #[serde(default = "default_background_color")]
    pub background_color: String,
}

fn default_auto_hide() -> bool { true }
fn default_pinned_apps() -> Vec<String> {
    vec![
        "firefox".to_string(),
        "alacritty".to_string(),
        "thunar".to_string(),
    ]
}
fn default_icon_size() -> i32 { 32 }
fn default_exclusive_zone() -> i32 { 60 }
fn default_trigger_height() -> i32 { 10 }
fn default_background_color() -> String { "rgba(20, 20, 30, 0.15)".to_string() }

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_hide: default_auto_hide(),
            pinned_apps: default_pinned_apps(),
            icon_size: default_icon_size(),
            exclusive_zone: default_exclusive_zone(),
            trigger_height: default_trigger_height(),
            background_color: default_background_color(),
        }
    }
}

pub fn load_config() -> Config {
    let mut config = Config::default();

    if let Some(proj_dirs) = get_project_dirs() {
        let config_dir = proj_dirs.config_dir();
        let config_path = config_dir.join("config.toml");

        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                match toml::from_str::<Config>(&content) {
                    Ok(loaded) => config = loaded,
                    Err(e) => eprintln!("Failed to parse config file (using defaults): {}", e),
                }
            }
        } else {
            ensure_config_dir(proj_dirs.config_dir());
        }
    }

    config
}

pub fn get_project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("io", "github.albibenni", "hyprdock")
}

fn ensure_config_dir(path: &Path) {
    let _ = fs::create_dir_all(path);
}
