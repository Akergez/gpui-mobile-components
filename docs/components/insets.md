# Insets: the status bar, the navigation bar, the keyboard

`gpui_adaptive::insets` · [index](../README.md)

`Insets::of(window)` reads the parts of the window the system covers in this frame, from `gpui-android-glue`:

| Field | Glue | Meaning |
|---|---|---|
| `top` | `insets::system_bars().0` | the status bar |
| `bars` | `insets::system_bars().1` | the navigation bar, never the keyboard |
| `keyboard` | `keyboard::keyboard_inset(window)` | the keyboard as drawn now (with the bar under it); 0 when it is down |

The window lies under all of them edge to edge. Read them in `render` and never keep them: while the keyboard slides, the glue reports every frame and the window is drawn again, so whatever makes room moves with it and nothing has to be animated by hand. On a desktop all three are zero; `GPUI_KEYBOARD=300` pretends a 300 px keyboard is up.

Two helpers:

- `typing()` is `bars.max(keyboard)`: the room to keep under whatever is typed into.
- `keyboard_over_bars()` is how far the keyboard reaches above the navigation bar: what a page that already keeps the bar's room has to rise by.

## Who keeps the room

| Element | Keeps |
|---|---|
| [AppFrame](app-frame.md) | `top` on a phone in the touch skin; `typing()` at the bottom unless `edge_to_edge` |
| [Destinations](destinations.md), compact, bar shown | the bar pads `bars`; the content pads `keyboard_over_bars()`, so the bar goes under the keyboard |
| [Destinations](destinations.md), bar hidden, or rail/sidebar | the content pads `typing()` |
| [Sheet](sheet.md) | `typing()` under its content, and its full height ends at the keyboard |
| [Notice](notice.md) (snackbar) | stands above `typing()` and the navigation bar |
| [Form](form.md) on a pointer (tablet) | the dialog rises to the top while the keyboard is up |

## Related glue

- `keyboard::follow(id, event, Purpose)` shapes the keyboard for an input. [TextField](text-field.md), [DateField](date-field.md), [Composer](composer.md) and [Lookup](lookup.md) call it.
- `keyboard::on_tap(input, state, cx)` raises the keyboard on a tap even when the input kept its focus. The same intents use it.
