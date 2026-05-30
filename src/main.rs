use gtk4::prelude::*;
use gtk4::{Application, glib};
use tokio::sync::mpsc;
use hyprdock::{hypr, ui, config};
use std::fs;
use std::env;
use std::path::Path;
use std::panic;

fn setup_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let payload = info.payload().downcast_ref::<&str>();
        let msg = payload.unwrap_or(&"Unknown panic");
        if msg.contains("assertion failed: !ptr.is_null()") { return; }
        default_hook(info);
    }));
}

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

    if actual_socket_dir.exists() {
        let is_valid = if expected_socket_dir.exists() {
            match fs::read_link(&expected_socket_dir) {
                Ok(target) => target == actual_socket_dir,
                Err(_) => false,
            }
        } else {
            false
        };

        if !is_valid {
            println!("Fixing Hyprland socket path: symlinking /tmp/hypr/{} to {}", signature, actual_socket_dir.display());
            let _ = fs::remove_dir_all(&expected_socket_dir);
            let _ = fs::create_dir_all("/tmp/hypr");
            #[cfg(unix)]
            {
                use std::os::unix::fs::symlink;
                let _ = symlink(actual_socket_dir, expected_socket_dir);
            }
        }
    }
}

fn main() -> glib::ExitCode {
    setup_panic_hook();
    fix_hyprland_socket();

    let app = Application::builder()
        .application_id("io.github.albibenni.hyprdock")
        .build();

    app.connect_activate(|app| {
        println!("HyprDock: Activating Version 0.1.0-Polished");
        let config = config::load_config();
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
