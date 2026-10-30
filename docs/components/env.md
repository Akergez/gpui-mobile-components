# Env: input mode and width class

`gpui_adaptive::env` · [index](../README.md)

`Env` describes what the current frame is drawn for. Every intent reads it in `render`, and feature code should almost never need to.

## The two axes

| | `InputMode` | `WidthClass` |
|---|---|---|
| Values | `Pointer`, `Touch` | `Compact`, `Medium`, `Expanded` |
| Set by | `gpui_adaptive::init(input, cx)`, once | the window's width, every frame |
| Measured in | — | rems (`viewport_size().width / rem_size()`) |
| Bounds | — | `COMPACT_BELOW = 44.0`, `EXPANDED_FROM = 66.0`. A bound belongs to the wider class. |
| Decides | the skin, and stateful backends | layout |

The width is measured in rems, not pixels, because what decides whether a sidebar and a table fit is how many characters fit. That changes with the interface's zoom (the theme's base font) as much as with the window. The bounds are money-manager's (`shell/layout.rs`).

The input mode does not change while windows are open. Stateful intents ([`Choice`](choice.md)) build their backend in `new` from it. `set_input_mode` exists for tests and previews; rebuild stateful intents after calling it.

## API

```rust
gpui_adaptive::init(InputMode::Touch, cx);     // android_main's start
gpui_adaptive::init(InputMode::Pointer, cx);   // main()

let env = Env::of(window, cx);
env.input();        // InputMode
env.width();        // WidthClass
env.is_touch();
env.is_compact();

WidthClass::of(window);
WidthClass::for_rems(52.0);                    // Medium
gpui_adaptive::input_mode(cx);
gpui_adaptive::set_input_mode(InputMode::Touch, cx); // tests, previews
```

## Which combinations happen

| | Compact | Medium | Expanded |
|---|---|---|---|
| **Pointer** | narrow desktop window: one pane, bottom bar, menus | icon rail, columns | sidebar with labels, columns. *Today's desktop.* |
| **Touch** | navigation bar, stack, FAB, sheets. *Today's phone.* | rail, columns, sheets | rail with labels, columns |

## Rules

- Read `Env` in `render`. Never store it: the window can be resized at any moment.
- Feature code that branches on `is_touch()` is missing an intent. Add one here instead.

## Replaces

`aiddiction::ui::is_phone`, `money-manager::shell::layout::Layout`, and matterfast's width checks.

## Tests

`env::tests` checks the bounds exactly (43.9 is Compact, 44.0 is Medium, 66.0 is Expanded) and the order of the classes.
