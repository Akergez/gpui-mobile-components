# AppFrame: the window's frame

`gpui_adaptive::frame` · [index](../README.md)

Put one `AppFrame` at the top of each window's root view. It draws three things:

1. **The bar across the top.** This is `gpui_android_glue::chrome::title_frame`: gpui-kit's `TitleBar` on a desktop (window controls, drag area), with the title and optional `bar` content. In the touch skin on a phone there is no strip at all, only the status bar's room; each page's [Screen](screen.md) app bar sits right under it.
2. **The content**, usually [Destinations](destinations.md) or a [Navigator](navigator.md).
3. **The layers over everything**: the stack of touch [sheets](sheet.md) and the [snackbar](notice.md). gpui-kit's own dialogs, popovers and notifications are drawn by its `Root` above these. Without an `AppFrame` a window shows no touch sheets and no snackbars.

## API

```rust
impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        AppFrame::new("Matterfast")
            .bar(search_box)             // desktop title bar content after the title
            .edge_to_edge(true)          // content keeps clear of bars and keyboard itself
            .child(destinations)
    }
}
```

`title_bar(false)` leaves the title bar to the pages: each page's [Screen](screen.md) with `in_title_bar(true)` draws it (aiddiction's layout). On a phone in the touch skin it changes nothing.

`edge_to_edge(true)` says the content handles the bottom insets itself, as [Destinations](destinations.md) does. Without it the frame pads the bottom by `Insets::typing()`.

## Built on

- `gpui_android_glue::chrome::title_frame`
- `gpui_android_glue::insets`
- the crate's per-window layer stack (`layers.rs`)
