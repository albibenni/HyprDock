use gtk4::prelude::*;
use gtk4::{Application, glib};
use tokio::sync::mpsc;
use hyprdock::{hypr, ui, config};
use std::fs;
use std::env;
use std::path::Path;

fn fix_hyprland_socket() {
    let signature = match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(s) => s,
        Err(_) => return,
    };

    let xdg_runtime_dir = match env::var("XDG_RUNTIME_DIR") {
        Ok(s) => s,
        Err(_) => return,
    };

    let actual_socket_dir = Path::new(&xdg_runtime_dir).join("hypr").join(&signature);
    let expected_socket_dir = Path::new("/tmp/hypr").join(&signature);

    if !expected_socket_dir.exists() && actual_socket_dir.exists() {
        println!("Fixing Hyprland socket path: symlinking /tmp/hypr/{} to {}", signature, actual_socket_dir.display());
        let _ = fs::create_dir_all("/tmp/hypr");
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let _ = symlink(actual_socket_dir, expected_socket_dir);
        }
    }
}

fn load_config() -> config::Config {
    let mut config = config::Config::default();

    if let Some(proj_dirs) = config::get_project_dirs() {
        let config_dir = proj_dirs.config_dir();
        let config_path = config_dir.join("config.toml");

        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(loaded) = toml::from_str::<config::Config>(&content) {
                    config = loaded;
                }
            }
        } else {
            let _ = fs::create_dir_all(config_dir);
        }
    }

    config
}

fn main() -> glib::ExitCode {
    // Attempt to fix the socket path before the library tries to use it
    fix_hyprland_socket();

    let app = Application::builder()
        .application_id("io.github.albibenni.hyprdock")
        .build();

    app.connect_activate(|app| {
        println!("HyprDock activating...");
        let config = load_config();
        ui::initialize_styling();

        let (tx, mut rx) = mpsc::unbounded_channel::<hypr::HyprEvent>();
        let ui_handle = ui::build_ui(app, &config);

        hypr::start_listener(tx);

        glib::MainContext::default().spawn_local(async move {
            while let Some(event) = rx.recv().await {
                ui::handle_event(event, &ui_handle);
            }
        });
    });

    app.run()
}
