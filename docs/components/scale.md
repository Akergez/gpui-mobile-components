# Scale: the lengths and durations of a skin

`gpui_adaptive::scale` · [index](../README.md)

`Scale` holds how tall a control is, how round a sheet is, and how long a page takes to come in. `init` installs it as a global for the input mode, and intents read it with `cx.scale()`.

Every length is in rems, so both skins follow the theme's base font, which is the application's zoom (Coding Guides, "Base font is the application zoom control"). gpui-kit's theme cannot store an application's own scales (`apply_semantic_tokens` keeps no custom spacing or elevation), so the scale is this crate's global.

| Token | Pointer | Touch (M3 at 16 px/rem) |
|---|---|---|
| `control` (button height) | 1.75 rem (28 px) | 2.5 rem (40 dp) |
| `target` (least pressable) | 1.5 rem | 3 rem (48 dp) |
| `row` / `row_two_line` | 1.75 / 2.75 rem | 3.5 / 4.5 rem (56 / 72 dp) |
| `bar` (page bar) | 2.75 rem | 4 rem (64 dp) |
| `navigation_bar` | 3.5 rem | 5 rem (80 dp) |
| `rail` / `sidebar` | 3.75 / 14 rem | 5 / 17.5 rem |
| `fab` | 3.5 rem | 3.5 rem (56 dp) |
| `radius_control` / `container` / `sheet` | 0.375 / 0.5 / 0.625 rem | 1.25 / 0.75 / 1.75 rem |
| `title` / `title_root` | 0.875 / 0.875 rem | 1.25 / 1.5 rem |
| `body` / `small` | 0.8125 / 0.75 rem | 1 / 0.875 rem |
| `page` (transition) | 120 ms | 300 ms |
| `sheet_open` / `sheet_close` / `settle` | 160 / 120 / 120 ms | 260 / 200 / 180 ms |
| `notice` (lifetime) | 6 s | 4 s |

## API

```rust
use gpui_adaptive::ActiveScale as _;
let scale = cx.scale();
div().h(scale.row).text_size(scale.body)
Scale::pointer(); Scale::touch(); Scale::for_input(InputMode::Touch);
```

## Rules

- Do not write `px(...)` in an application for a control's geometry. Take it from the scale or from the gpui-kit rem helpers.
- A length in pixels is right only for a physical boundary, such as the system's insets.

## Tests

`scale::tests` checks that a touch target is at least 48 dp and larger than a pointer's.
