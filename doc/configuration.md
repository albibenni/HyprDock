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
| `transparent` | Boolean | `false` | If set to `true`, the dock background will be completely transparent. |

## Example `config.toml`

```toml
auto_hide = true
pinned_apps = ["firefox", "kitty", "code", "spotify"]
icon_size = 48
exclusive_zone = 70
trigger_height = 8
transparent = true
```

## Custom Styling

You can also provide a custom CSS file at `~/.config/hyprdock/style.css` to override any default styles.
