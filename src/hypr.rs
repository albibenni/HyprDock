use hyprland::event_listener::EventListener;
use tokio::sync::mpsc::UnboundedSender;

/// Events that can be sent from the Hyprland thread to the GTK thread
pub enum HyprEvent {
    WorkspaceChanged(String),
    ActiveWindowChanged(String),
}

pub fn start_listener(tx: UnboundedSender<HyprEvent>) {
    std::thread::spawn(move || {
        let mut listener = EventListener::new();

        let tx_ws = tx.clone();
        listener.add_workspace_change_handler(move |id| {
            let _ = tx_ws.send(HyprEvent::WorkspaceChanged(format!("{:?}", id)));
        });

        let tx_win = tx.clone();
        listener.add_active_window_change_handler(move |data| {
            if let Some(data) = data {
                let _ = tx_win.send(HyprEvent::ActiveWindowChanged(data.window_title));
            }
        });

        if let Err(e) = listener.start_listener() {
            eprintln!("Hyprland listener error: {}", e);
        }
    });
}
