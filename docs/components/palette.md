# Palette: Material roles that survive a Zed theme

`gpui_adaptive::palette` · [index](../README.md)

Both skins paint from `cx.theme()` first. While the scheme is worn, the theme is the Material You scheme of `gpui-adaptive-colors`; otherwise it is a Zed theme the person chose.

Material 3 also has roles the gpui-kit theme has no place for:

- the container a FAB is filled with;
- the pill of the selected destination;
- an avatar's fill;
- the inverse surface of a snackbar.

`cx.scheme()` has all of them, but it keeps describing the scheme after a Zed theme has been put on, so a FAB would be the scheme's green in a Catppuccin window. The touch skin therefore reads `Palette`:

- **while the scheme is worn**, the scheme's roles (`AdaptiveColors::worn(mode)` is `None`);
- **while a theme is worn**, or without `gpui-adaptive-colors` at all, the nearest the theme has. Containers are the theme's accent colours laid thinly over its background, and the snackbar is the foreground used as a fill.

| Field | From the scheme | From a theme |
|---|---|---|
| `surface`, `on_surface`, `on_surface_variant` | surface roles | `background`, `foreground`, `muted_foreground` |
| `surface_low` / `container` / `high` / `highest` | surface containers | `sidebar` / `popover` / `muted` / `accent` |
| `primary`, `on_primary` | primary | `primary`, `primary_foreground` |
| `primary_container` (FAB, own bubble) | primary-container | `primary` at 16% over `background` |
| `secondary_container` (selected pill, row) | secondary-container | `sidebar_accent` |
| `tertiary_container` (avatar) | tertiary-container | `info` at 18% over `background` |
| `error`, `on_error` | error | `danger`, `danger_foreground` |
| `outline`, `outline_variant` | outline | `input`, `border` |
| `inverse_surface`, `inverse_on_surface`, `inverse_primary` | inverse roles | `foreground`, `background`, `primary` at 60% |
| `pressed` | on-surface at 12% | foreground at 12% |
| `scrim` | theme `overlay` | theme `overlay` |

## API

```rust
use gpui_adaptive::ActivePalette as _;
let palette = cx.palette();
div().bg(palette.primary_container).text_color(palette.on_primary_container)
```

## Rules

- Never paint from `cx.scheme()` directly in an application. Use the theme, or this palette.
- No literal colours. Every value comes from the scheme or the theme.
