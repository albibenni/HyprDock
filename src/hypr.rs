use hyprland::data::{Client, Clients};
use hyprland::dispatch::*;
use hyprland::event_listener::EventListener;
use hyprland::shared::{Address, HyprData, WorkspaceType};
use tokio::sync::mpsc::UnboundedSender;

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
}

/// Starts the Hyprland event listener and sends initial state.
pub fn start_listener(tx: UnboundedSender<HyprEvent>) {
    send_window_list(&tx);

    std::thread::spawn(move || {
        let mut listener = EventListener::new();
        register_handlers(&mut listener, tx);

        if let Err(e) = listener.start_listener() {
            eprintln!("Hyprland listener error: {}", e);
        }
    });
}

fn register_handlers(listener: &mut EventListener, tx: UnboundedSender<HyprEvent>) {
    // Workspace changes
    let tx_ws = tx.clone();
    listener.add_workspace_change_handler(move |id| {
        let _ = tx_ws.send(HyprEvent::WorkspaceChanged(format_workspace_id(id)));
    });

    // Active window title changes
    let tx_active = tx.clone();
    listener.add_active_window_change_handler(move |data| {
        let title = data.map(|d| d.window_title);
        let _ = tx_active.send(HyprEvent::ActiveWindowChanged(title));
    });

    // Full window list refreshes on structural changes
    register_refresh_handlers(listener, tx);
}

fn register_refresh_handlers(listener: &mut EventListener, tx: UnboundedSender<HyprEvent>) {
    listener.add_window_open_handler({
        let tx = tx.clone();
        move |_| send_window_list(&tx)
    });
    listener.add_window_close_handler({
        let tx = tx.clone();
        move |_| send_window_list(&tx)
    });
    listener.add_window_moved_handler({
        let tx = tx.clone();
        move |_| send_window_list(&tx)
    });
    listener.add_active_window_change_handler({
        let tx = tx.clone();
        move |_| send_window_list(&tx)
    });
}

fn send_window_list(tx: &UnboundedSender<HyprEvent>) {
    if let Ok(clients) = Clients::get() {
        let windows: Vec<WindowInfo> = clients.into_iter().map(WindowInfo::from).collect();
        let _ = tx.send(HyprEvent::WindowListUpdate(windows));
    }
}

fn format_workspace_id(id: WorkspaceType) -> String {
    match id {
        WorkspaceType::Regular(n) => n,
        WorkspaceType::Special(Some(n)) => n,
        _ => "Unknown".to_string(),
    }
}

/// Dispatches a focus command to Hyprland for the given window address.
pub fn focus_window(address: &str) {
    let addr = Address::new(address);
    let _ = Dispatch::call(DispatchType::FocusWindow(WindowIdentifier::Address(addr)));
}
