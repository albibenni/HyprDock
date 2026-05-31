# HyprDock Architecture & Design

HyprDock is a modern, lightweight dock for Hyprland (and other Wayland compositors supporting `wlr-layer-shell`), built with Rust and GTK4.

## Core Technologies
- **Rust**: Language of choice for safety and performance (Edition 2024).
- **GTK4**: For the user interface, providing a modern and accessible widget toolkit.
- **gtk4-layer-shell**: To integrate the GTK window into the Wayland layer-shell protocol.
- **hyprland-rs**: For interacting with the Hyprland IPC socket to listen for events.
- **Tokio/Threads**: For asynchronous event handling and background tasks.

## Architecture

The application follows a modular, event-driven architecture designed for high performance and low resource usage.

### 1. Main Thread (GTK Event Loop)
Manages the entire UI lifecycle and user interactions.
- **UI Module (`src/ui/`)**: Highly modularized components (Taskbar, Launcher, Context Menus).
- **Auto-Hide Logic**: Sophisticated, debounced mouse tracking with a 1.5s grace period and popover visibility coordination.
- **Overlay Mode**: Optimized window placement that avoids expensive workspace reflows by floating over active windows.

### 2. Background Thread (Hyprland Listener)
Connected via `hyprland-rs` to the compositor's IPC socket.
- **Event Streaming**: Listens for `activewindow`, `workspace`, `openwindow`, and `closewindow` events.
- **Channel Communication**: Sends events to the Main Thread via asynchronous channels to trigger UI refreshes without blocking.

### 3. State Management
- **DockUI State**: Shared handle (`Rc<DockUI>`) that manages widget references, configuration, and real-time state (like active window addresses).
- **Configuration**: TOML-based system with automatic disk persistence for dynamic updates (like pinning favorites).

## UI Component Breakdown
- **Launcher**: Button-based application menu with fuzzy search and robust `gio::AppInfo` launching.
- **Taskbar**: Dynamic grouping of pinned applications and active windows with three-tier visual status indicators.
- **Context Menus**: Right-click management for favorites and application control (Close App).
- **Visuals**: CSS-driven styling with hardware-accelerated slide animations and macOS-inspired aesthetics.
