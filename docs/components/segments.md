# Segments: two to four exclusive modes

`gpui_adaptive::segments` · [index](../README.md)

Use `Segments` for choices like Expenses or Income, Day or Week, Mentions or Saved.

| Pointer | Touch |
|---|---|
| gpui-kit's segmented look: a muted track with the chosen segment raised on it | Material 3's connected button group, 40 dp tall; the chosen segment is filled with the primary and fully rounded |

| Pointer | Touch |
|---|---|
| ![](../images/desktop-input.png) | ![](../images/phone-input.png) |

## API

```rust
Segments::new("kind", self.kind.clone())
    .segment("expense", "Expenses")
    .segment_with_icon("income", "Income", IconName::ArrowDown)
    .on_change(move |key, _, cx| this.update(cx, |this, cx| { this.kind = key.clone(); cx.notify(); }))
```

The value is controlled: the owner keeps the selected key. `on_change` is not called for the segment already on. Each segment's element id is its key.

## When not to use

- More than four options, or options that are not all visible at once: use [Choice](choice.md).
- Navigating between sections: use [Destinations](destinations.md).
