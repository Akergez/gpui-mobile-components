# Popup: something small beside what opened it

`gpui_adaptive::popup` · [index](../README.md)

`Popup` is a button that opens content of the application's own over the page: a palette of reactions, who is in a call, the account and its teams.

| Pointer | Touch |
|---|---|
| A gpui-kit popover under the button, closed by a press outside it, by Escape, or by what it holds once the choice is made | The same content in a [sheet](sheet.md), which back and a drag down close as well |

## API

```rust
Popup::new("account", Command::new("account-button", "Account and teams"), move |close, window, cx| {
    // built each time it is drawn; `close` ends it, as a choice made in it usually does
    column().child(teams(close.clone(), cx)).into_any_element()
})
.face(avatar)   // a face in the button instead of the command's icon
```

The button is the [command](command.md): its icon (or `.face(..)`), its label as the tooltip. The content is a closure handed a `Handler` that closes the popup.
