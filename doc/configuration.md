# HyprDock Configuration

HyprDock is configured via a TOML file located at `~/.config/hyprdock/config.toml`.

## Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `auto_hide` | Boolean | `true` | Whether the dock should automatically hide when not in use. |
| `pinned_apps` | List of Strings | `["firefox", "alacritty", "thunar"]` | List of application classes to pin to the dock. |
| `icon_size` | Integer | `32` | The size of the icons in the dock (in pixels). |
| `exclusive_zone` | Integer | `60` | The height of the reserved area on the screen edge (macOS-like spacing). |
| `trigger_height` | Integer | `10` | The height of the invisible trigger zone at the bottom of the screen. |
| `background_color` | String | `"rgba(20, 20, 30, 0.15)"` | The background color of the dock content. Supports any CSS color string. |
| `overlay` | Boolean | `true` | If set to `true`, the dock will overlap open windows when it appears. If `false`, it will push windows upward. |

## Example `config.toml`

```toml
auto_hide = true
pinned_apps = ["firefox", "kitty", "code", "spotify"]
icon_size = 48
exclusive_zone = 70
trigger_height = 8
background_color = "transparent"
```

## Custom Styling

You can also provide a custom CSS file at `~/.config/hyprdock/style.css` to override any default styles.

### Section Classes
The dock is divided into three main sections that you can style individually:
- `.section-menu`: The container for the launcher/menu button.
- `.section-favorites`: The container for your pinned application icons.
- `.section-tasks`: The container for unpinned, active application windows.

