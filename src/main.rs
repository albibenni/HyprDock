use gtk4::prelude::*;
use gtk4::Application;
use HyprDock::{hypr, ui};

fn main() -> gtk4::glib::ExitCode {
    let app = Application::builder()
        .application_id("io.github.albibenni.hyprdock")
        .build();

    app.connect_activate(ui::build_ui);

    // Start Hyprland event listener
    hypr::start_listener();

    app.run()
}
