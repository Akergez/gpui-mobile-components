# Command: one action, many entry points

`gpui_adaptive::command` · [index](../README.md)

A `Command` is one logical action: its id, label, icon, role, enabled state, and what it does. The same value becomes whatever the place it is offered in needs:

| Offered as | Pointer | Touch |
|---|---|---|
| a screen's primary | the primary button | the FAB |
| a screen's command | a button with its label, or an icon with a tooltip that shows the key binding | an app-bar icon (two at most) |
| overflow | a dropdown menu item with its key binding | a row of a sheet |
| an item's quick command | a button in the hover bar | the top of the long-press sheet |
| an item's command | a context-menu item | a row of the long-press sheet |

This is the Coding Guides' "model one logical desktop command once": a button, a menu item and a key binding run the same thing, so they cannot disagree. The shortcut is not stored on the command. Menus and tooltips read it from the keymap for its action.

## Doing the work

- `.action(SaveRecord)` dispatches a GPUI action from the focused element; the owning view handles it with `on_action`. Use this whenever the command has, or could have, a key binding.
- `.on_trigger(|window, cx| ...)` calls a closure. Use it for what has no binding and no focus to dispatch from, such as a row's "Delete" in a sheet. It takes precedence over the action.

`command.run(window, cx)` does nothing while the command is disabled.

## Roles

| Role | Meaning |
|---|---|
| `Normal` | shown while there is room, then in the overflow |
| `primary()` | the page's one default commitment: primary button or FAB |
| `destructive()` | danger colour, offered last, never the Enter default |
| `overflow()` | rare enough to start in the menu |

## API

```rust
Command::new(("delete", record.id), "Delete record")
    .icon(IconName::Delete)
    .destructive()
    .disabled(!record.is_editable())
    .on_trigger(move |window, cx| delete(record.id, window, cx));

command.button(env);       // a gpui-kit Button sized for the skin
command.icon_button(env);  // icon + tooltip (with the binding)
command.add_to(menu);      // into a PopupMenu
command.run(window, cx);
gpui_adaptive::command::in_offer_order(&commands); // destructive last
```

Ids come from the domain (`("delete", record.id)`), and a command's id becomes its element id.

## Tests

`command::tests` checks the offer order and that clones keep their role and state. `tests/ui.rs` covers the primary command in both skins.
