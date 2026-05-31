# HyprDock

A modern, lightweight, and highly customizable dock for Hyprland (and other Wayland compositors), built with Rust and GTK4.

![HyprDock Screenshot](./images/screenshot.png)

## Features

- **macOS-inspired Aesthetics**: Clean pill-shaped design with semi-transparent backgrounds and smooth slide-up animations.
- **Intelligent Auto-Hide**: Debounced, rock-solid auto-hide that stays open during tooltip displays and menu interaction.
- **Overlay Mode**: Floats over windows without shifting your workspace layout (no screen reflow).
- **Modular App Menu**: Faster, button-based application launcher with fuzzy search and visual click feedback.
- **Favorites Management**: Right-click any icon to pin to favorites or remove them instantly. Changes are persisted automatically.
- **Visual Status Indicators**: Three-tier dot system to distinguish between closed, open, and focused applications.
- **Application Control**: Right-click open apps to close them (all instances) directly from the dock.
- **Highly Configurable**: Comprehensive TOML configuration and support for custom CSS overrides.

## Documentation

- **[Configuration Guide](./doc/configuration.md)**: Full list of options and styling customization.
- **[Architecture](./doc/architecture.md)**: Overview of the internal design and technologies.

## Prerequisites

- `rustc` and `cargo` (Edition 2024)
- `libgtk-4-dev`
- `libgtk4-layer-shell-dev`
- `hyprland` compositor

## Installation

```bash
git clone https://github.com/albibenni/HyprDock.git
cd HyprDock
make build
```

## Running

```bash
make run
```

## License

MIT
