# Toggle: on or off

`gpui_adaptive::toggle` · [index](../README.md)

| | Pointer | Touch |
|---|---|---|
| in a form (committed with it) | gpui-kit `Checkbox` with its label, the description under it | `Switch` at the end of a row; the whole row is the target |
| `immediate(true)` (a setting that acts at once) | the label and description, a `Switch` at the end | the same row |

## API

```rust
Toggle::new("sync", "Sync on start", self.sync)
    .description("Pull changes when the window opens")
    .immediate(true)
    .disabled(!self.signed_in)
    .on_change(move |on, _, cx| this.update(cx, |this, cx| { this.sync = on; cx.notify(); }))
```

The value is controlled: pass the current value, receive the requested one, store it, render again (Coding Guides, "prefer controlled values"). The control's element id is the toggle's id.

## Tests

`tests/ui.rs`: `a_toggle_changes_from_its_control_in_both_skins` presses the control in each skin and checks the owner's value.
