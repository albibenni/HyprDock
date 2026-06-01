use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Deserialize, Serialize, Clone)]
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

    #[serde(default = "default_trigger_width")]
    pub trigger_width: i32,

    #[serde(default = "default_background_color")]
    pub background_color: String,

    #[serde(default = "default_overlay")]
    pub overlay: bool,
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
fn default_trigger_height() -> i32 { 2 }
fn default_trigger_width() -> i32 { 100 }
fn default_background_color() -> String { "rgba(20, 20, 30, 0.15)".to_string() }
fn default_overlay() -> bool { true }

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_hide: default_auto_hide(),
            pinned_apps: default_pinned_apps(),
            icon_size: default_icon_size(),
            exclusive_zone: default_exclusive_zone(),
            trigger_height: default_trigger_height(),
            trigger_width: default_trigger_width(),
            background_color: default_background_color(),
            overlay: default_overlay(),
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

pub fn save_config(config: &Config) {
    if let Some(proj_dirs) = get_project_dirs() {
        let config_dir = proj_dirs.config_dir();
        let config_path = config_dir.join("config.toml");

        ensure_config_dir(config_dir);

        match toml::to_string_pretty(config) {
            Ok(content) => {
                if let Err(e) = fs::write(&config_path, content) {
                    eprintln!("Failed to write config file: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to serialize config: {}", e),
        }
    }
}

pub fn get_project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("io", "github.albibenni", "hyprdock")
}

fn ensure_config_dir(path: &Path) {
    let _ = fs::create_dir_all(path);
}
