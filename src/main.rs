use gtk4::prelude::*;
use gtk4::{Application, glib};
use tokio::sync::mpsc;
use hyprdock::{hypr, ui};

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("io.github.albibenni.hyprdock")
        .build();

    app.connect_activate(|app| {
        // 1. Basic configuration (could be loaded from TOML in the future)
        let config = ui::Config {
            auto_hide: true,
        };

        // 2. Create a Tokio unbounded channel
        let (tx, mut rx) = mpsc::unbounded_channel::<hypr::HyprEvent>();

        // 3. Build the UI and get a reference to the label we want to update
        let (_window, status_label) = ui::build_ui(app, &config);

        // 4. Start the Hyprland listener in a standard thread, passing the sender
        hypr::start_listener(tx);

        // 5. Spawn a local async task on the GTK MainContext to listen for messages
        glib::MainContext::default().spawn_local(async move {
            while let Some(event) = rx.recv().await {
                // This block executes safely on the main UI thread
                ui::handle_event(event, &status_label);
            }
        });
    });

    app.run()
}
