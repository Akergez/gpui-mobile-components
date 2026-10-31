# Menu: commands behind a button, or at a point

`gpui_adaptive::menu` · [index](../README.md)

`Menu` offers several [commands](command.md) from one button: a toolbar's "More", a channel's menu, an application's main menu.

| Pointer | Touch |
|---|---|
| The button opens a gpui-kit dropdown menu: groups apart by a separator, each command with its key binding and its check | The button opens the commands as a [sheet](sheet.md) of rows (`Sheet::actions`), destructive ones last |

The button is a `Command` too: its icon, and its label as the tooltip (or as the text, without an icon). What the button itself would do is not run; it opens the menu.

## API

```rust
Menu::new(Command::new("chat-more", "More").icon(IconName::Ellipsis))
    .title("Development")                       // over the sheet, on a phone
    .commands([mute, mark_read])
    .separator()                                 // a new group
    .command(leave.destructive())
```

- `Command::checked(bool)` marks the option that is on: a priority, a sort order.
- `.choice()`: the menu is a choice, its button showing what is chosen: outlined, with a caret, so that it reads as something to change rather than as text (an integration's select, a form field of a few options). Without it a button with no icon is a ghost button.

## At a point

`Menu::open_at(title, commands, position, window, cx)` opens commands where there is no button: at the point a right click landed on a link inside a message, once the text has said which link it was (`Markdown::on_link_menu`, see [Primitives](primitives.md)).

| Pointer | Touch |
|---|---|
| A popup menu at the point, drawn by the window's [AppFrame](app-frame.md) over everything else; closed by a choice, a press outside or Escape; the focus goes back where it was | The sheet of rows |

`Menu::is_open_at(window, cx)` says whether one is up, for a scenario to check.

## Tests

`tests/ui.rs`:
- `a_menu_at_a_point_is_a_popup_on_a_pointer`
- `a_menu_at_a_point_is_a_sheet_on_a_phone`
