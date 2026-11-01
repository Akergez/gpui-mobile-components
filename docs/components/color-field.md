# ColorField: a colour the person picks

`gpui_adaptive::color_field` · [index](../README.md)

Use `ColorField` for a colour of the person's own making: a category's, a label's. A few featured colours are a press away, any other comes from the palettes or is typed as hex.

| Pointer | Touch |
|---|---|
| gpui-kit's colour picker from a swatch button | the same, its button a finger's height |

## API

```rust
// in new():
let color = cx.new(|cx| ColorFieldState::new(stored.as_deref().and_then(parse_hex), window, cx));
cx.subscribe(&color, |this, _, ColorFieldEvent::Change(picked), cx| {
    this.color = picked.map(to_hex);   // `#rrggbb`, how it is stored
    cx.notify();
});

// in render():
ColorField::new("category-color", &self.color).label("Color").featured(PALETTE.iter().filter_map(|c| parse_hex(c)))
```

- An unset field (`None`) is the thing's own default (money-manager's category colour follows its name). Say so beside the field with a [Note](note.md).
- `color_field::to_hex(color)` gives `#rrggbb` without the opacity; `color_field::parse_hex(text)` reads it back.

## Used by

money-manager's category form.
