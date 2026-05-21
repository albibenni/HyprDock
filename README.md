# hyprdock

A modern dock for Hyprland written in Rust.

## Features
- GTK4 UI
- Layer Shell support (stays on top, reserves screen space)
- Hyprland IPC integration (listens for workspace and window changes)

## Getting Started

### Prerequisites
Ensure you have the following installed:
- Rust (stable)
- GTK4 development libraries
- GTK4 Layer Shell development libraries

On Arch Linux:
```bash
sudo pacman -S gtk4 gtk4-layer-shell
```

### Installation
1. Clone the repository
2. Run with cargo:
```bash
cargo run
```

## Architecture
See [GEMINI.md](GEMINI.md) for detailed architecture and design goals.
