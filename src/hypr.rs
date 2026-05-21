use hyprland::event_listener::EventListener;

pub fn start_listener() {
    std::thread::spawn(|| {
        let mut listener = EventListener::new();

        listener.add_workspace_change_handler(|id| {
            println!("Workspace changed to: {:?}", id);
        });

        listener.add_active_window_change_handler(|data| {
            if let Some(data) = data {
                println!("Active window: {}", data.window_class);
            }
        });

        if let Err(e) = listener.start_listener() {
            eprintln!("Hyprland listener error: {}", e);
        }
    });
}
