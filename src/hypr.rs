use hyprland::event_listener::EventListener;
use hyprland::shared::WorkspaceType;
use tokio::sync::mpsc::UnboundedSender;
use std::panic;
use std::time::Duration;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct WindowInfo {
    pub address: String,
    pub title: String,
    pub class: String,
    #[serde(rename = "workspace")]
    workspace_data: WorkspaceRef,
}

#[derive(Clone, Debug, Deserialize)]
struct WorkspaceRef {
    pub id: i32,
}

impl WindowInfo {
    pub fn workspace(&self) -> String {
        self.workspace_data.id.to_string()
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
    
    // Initial sync
    std::thread::spawn(move || {
        let mut attempts = 0;
        while attempts < 5 {
            if let Ok(windows) = get_clients_via_hyprctl() {
                let _ = tx_init.send(HyprEvent::WindowListUpdate(windows));
                return;
            }
            attempts += 1;
            std::thread::sleep(Duration::from_millis(500));
        }
    });

    std::thread::spawn(move || {
        let result = panic::catch_unwind(panic::AssertUnwindSafe(move || {
            let mut listener = EventListener::new();
            
            let tx_ws = tx.clone();
            listener.add_workspace_change_handler(move |id| {
                let _ = tx_ws.send(HyprEvent::WorkspaceChanged(format_workspace_id(id)));
            });

            let tx_active = tx.clone();
            listener.add_active_window_change_handler(move |data| {
                let title = data.map(|d| d.window_title);
                let _ = tx_active.send(HyprEvent::ActiveWindowChanged(title));
            });

            // Structural changes -> refresh list
            let tx_refresh = tx.clone();
            let refresh = move || {
                if let Ok(windows) = get_clients_via_hyprctl() {
                    let _ = tx_refresh.send(HyprEvent::WindowListUpdate(windows));
                }
            };

            let r1 = refresh.clone();
            listener.add_window_open_handler(move |_| r1());
            let r2 = refresh.clone();
            listener.add_window_close_handler(move |_| r2());
            let r3 = refresh.clone();
            listener.add_window_moved_handler(move |_| r3());
            let r4 = refresh;
            listener.add_active_window_change_handler(move |_| r4());

            let _ = listener.start_listener();
        }));

        if let Err(_) = result {
            eprintln!("Hyprland listener thread exited.");
        }
    });
}

fn get_clients_via_hyprctl() -> Result<Vec<WindowInfo>, String> {
    let output = std::process::Command::new("hyprctl")
        .args(["clients", "-j"])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("hyprctl failed".to_string());
    }

    serde_json::from_slice::<Vec<WindowInfo>>(&output.stdout).map_err(|e| e.to_string())
}

fn format_workspace_id(id: WorkspaceType) -> String {
    match id {
        WorkspaceType::Regular(n) => n,
        WorkspaceType::Special(Some(n)) => n,
        _ => "Unknown".to_string(),
    }
}

pub fn focus_window(address: &str) {
    let _ = std::process::Command::new("hyprctl")
        .args(["dispatch", "focuswindow", &format!("address:{}", address)])
        .status();
}

pub fn get_first_window_by_class(class: &str) -> Option<String> {
    if let Ok(windows) = get_clients_via_hyprctl() {
        let class_lower = class.to_lowercase();
        windows.into_iter()
            .find(|w| w.class.to_lowercase() == class_lower)
            .map(|w| w.address)
    } else {
        None
    }
}
