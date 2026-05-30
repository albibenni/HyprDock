use gtk4::prelude::*;
use gtk4::{Application, glib};
use tokio::sync::mpsc;
use hyprdock::{hypr, ui, config};

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("io.github.albibenni.hyprdock")
        .build();

    app.connect_activate(|app| {
        // 1. Load configuration from file
        let config = config::load_config();

        // 2. Initialize CSS styling
        ui::initialize_styling();

        // 3. Create a Tokio unbounded channel
        let (tx, mut rx) = mpsc::unbounded_channel::<hypr::HyprEvent>();

        // 4. Build the UI and get the UI handle
        let ui_handle = ui::build_ui(app, &config);

        // 5. Start the Hyprland listener in a standard thread, passing the sender
        hypr::start_listener(tx);

        // 6. Spawn a local async task on the GTK MainContext to listen for messages
        glib::MainContext::default().spawn_local(async move {
            while let Some(event) = rx.recv().await {
                // This block executes safely on the main UI thread
                ui::handle_event(event, &ui_handle);
            }
        });
    });

    app.run()
}
