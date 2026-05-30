use hyprland::data::{Client, Clients};
use hyprland::dispatch::*;
use hyprland::event_listener::EventListener;
use hyprland::shared::{Address, HyprData, WorkspaceType};
use tokio::sync::mpsc::UnboundedSender;
use std::panic;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct WindowInfo {
    pub address: String,
    pub title: String,
    pub class: String,
    pub workspace: String,
}

impl From<Client> for WindowInfo {
    fn from(client: Client) -> Self {
        Self {
            address: client.address.to_string(),
            title: client.title,
            class: client.class,
            workspace: client.workspace.id.to_string(),
        }
    }
}

pub enum HyprEvent {
    WorkspaceChanged(String),
    ActiveWindowChanged(Option<String>),
    WindowListUpdate(Vec<WindowInfo>),
    Error(String),
}

/// Starts the Hyprland event listener and sends initial state.
pub fn start_listener(tx: UnboundedSender<HyprEvent>) {
    let tx_init = tx.clone();
    
    // Initial sync in a loop until successful (handles slow startup)
    std::thread::spawn(move || {
        let mut attempts = 0;
        while attempts < 10 {
            if let Ok(_) = safe_send_window_list(&tx_init) {
                return;
            }
            attempts += 1;
            std::thread::sleep(Duration::from_millis(500));
        }
        let _ = tx_init.send(HyprEvent::Error("Hyprland connection timeout".to_string()));
    });

    std::thread::spawn(move || {
        // We wrap the entire listener logic to catch panics from the library
        let result = panic::catch_unwind(panic::AssertUnwindSafe(move || {
            let mut listener = EventListener::new();
            register_handlers(&mut listener, tx.clone());

            if let Err(e) = listener.start_listener() {
                let _ = tx.send(HyprEvent::Error(format!("Hyprland listener error: {}", e)));
            }
        }));

        if let Err(_) = result {
            eprintln!("Hyprland listener thread panicked. This usually means the Hyprland socket could not be found.");
        }
    });
}

fn safe_send_window_list(tx: &UnboundedSender<HyprEvent>) -> Result<(), String> {
    let tx_clone = tx.clone();
    let result = panic::catch_unwind(panic::AssertUnwindSafe(move || {
        if let Ok(clients) = Clients::get() {
            let windows: Vec<WindowInfo> = clients.into_iter().map(WindowInfo::from).collect();
            let _ = tx_clone.send(HyprEvent::WindowListUpdate(windows));
            Ok(())
        } else {
            Err("Failed to get clients".to_string())
        }
    }));

    match result {
        Ok(res) => res,
        Err(_) => Err("Library panicked".to_string()),
    }
}

fn register_handlers(listener: &mut EventListener, tx: UnboundedSender<HyprEvent>) {
    let tx_ws = tx.clone();
    listener.add_workspace_change_handler(move |id| {
        let _ = tx_ws.send(HyprEvent::WorkspaceChanged(format_workspace_id(id)));
    });

    let tx_active = tx.clone();
    listener.add_active_window_change_handler(move |data| {
        let title = data.map(|d| d.window_title);
        let _ = tx_active.send(HyprEvent::ActiveWindowChanged(title));
    });

    register_refresh_handlers(listener, tx);
}

fn register_refresh_handlers(listener: &mut EventListener, tx: UnboundedSender<HyprEvent>) {
    listener.add_window_open_handler({
        let tx = tx.clone();
        move |_| { let _ = safe_send_window_list(&tx); }
    });
    listener.add_window_close_handler({
        let tx = tx.clone();
        move |_| { let _ = safe_send_window_list(&tx); }
    });
    listener.add_window_moved_handler({
        let tx = tx.clone();
        move |_| { let _ = safe_send_window_list(&tx); }
    });
    listener.add_active_window_change_handler({
        let tx = tx.clone();
        move |_| { let _ = safe_send_window_list(&tx); }
    });
}

fn format_workspace_id(id: WorkspaceType) -> String {
    match id {
        WorkspaceType::Regular(n) => n,
        WorkspaceType::Special(Some(n)) => n,
        _ => "Unknown".to_string(),
    }
}

pub fn focus_window(address: &str) {
    let addr = Address::new(address);
    let _ = panic::catch_unwind(panic::AssertUnwindSafe(move || {
        let _ = Dispatch::call(DispatchType::FocusWindow(WindowIdentifier::Address(addr.clone())));
    }));
}

pub fn get_first_window_by_class(class: &str) -> Option<String> {
    let class_lower = class.to_lowercase();
    panic::catch_unwind(panic::AssertUnwindSafe(|| {
        if let Ok(clients) = Clients::get() {
            clients.into_iter()
                .find(|c| c.class.to_lowercase() == class_lower)
                .map(|c| c.address.to_string())
        } else {
            None
        }
    })).unwrap_or(None)
}
