# Chips: what a view is narrowed to

`gpui_adaptive::chips` · [index](../README.md)

Filter chips say which of several things a view shows: the categories a chart is of, the labels a list keeps. Any number can be on at once; "All" stands for none chosen.

| Pointer | Touch |
|---|---|
| small outlined toggles; the ones on are tinted with the primary and bordered in it | Material 3 filter chips, 32 dp; the ones on are the secondary container with a check |

## API

```rust
shares.iter().fold(Chips::new("bars").all("All"), |chips, share| {
    chips.chip(share.key(), share.name.clone(), chosen.contains(&share.id)).dot(share.color)
})
.on_toggle(move |key, _, cx| match key {
    Some(key) => toggle(key, cx),   // a chip
    None => clear(cx),              // "All"
})
```

- Controlled: the owner keeps which chips are on and is told which was pressed.
- `.dot(color)` gives the chip just added a dot of its thing's colour. While some chips are on, the dots of the others fade, so the chosen ones stand out on the chart too.
- The row wraps onto more rows as needed.
- Each chip's element id is `(id, key)`, and "All"'s is `(id, "all")`.

## When not to use

- Exactly one of two to four modes: use [Segments](segments.md).
- One of a long list: use [Choice](choice.md).

## Used by

money-manager's charts: the categories the monthly bars and the running total are each of.

## Tests

`tests/ui.rs`: `a_chip_is_told_by_its_key_and_all_as_none_in_both_skins`.
