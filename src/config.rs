use serde::Deserialize;
use directories::ProjectDirs;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub auto_hide: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { auto_hide: true }
    }
}

pub fn load_config() -> Config {
    let mut config = Config::default();

    if let Some(proj_dirs) = get_project_dirs() {
        let config_path = proj_dirs.config_dir().join("config.toml");
        
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(loaded) = toml::from_str::<Config>(&content) {
                    config = loaded;
                } else {
                    eprintln!("Failed to parse config file: {:?}", config_path);
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
