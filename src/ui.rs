use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, Orientation};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("HyprDock")
        .build();

    // Initialize layer shell
    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.set_namespace(Some("hyprdock"));

    // Anchor to bottom
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    // Reserve space
    window.set_exclusive_zone(50);

    // UI Layout
    let content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .halign(gtk4::Align::Center)
        .margin_top(5)
        .margin_bottom(5)
        .build();

    for i in 1..=5 {
        let button = Button::with_label(&format!("App {}", i));
        button.connect_clicked(move |_| {
            println!("Button {} clicked!", i);
        });
        content.append(&button);
    }

    window.set_child(Some(&content));
    window.present();
}
