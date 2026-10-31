# TextField: a line of text, with a label, an error and a purpose

`gpui_adaptive::field` · [index](../README.md)

`TextField` is a line of text to type. It also covers amounts: the proposal's `Amount` is `TextFieldState::amount()` with a currency `suffix`.

| Pointer | Touch |
|---|---|
| gpui-kit `Input` at its small size, the label over it | the `Input` at its large size (a finger's height), the label over it |

Under the field: the error while there is one, otherwise the help line.

![](../images/phone-input.png)

## Purpose

The field says what it is for with the glue's `Purpose`. On a phone, `keyboard::follow` shapes the keyboard from it, and `keyboard::on_tap` raises the keyboard even when the field kept its focus.

| Purpose | Keyboard |
|---|---|
| `Plain` | plain |
| `Message` | prose: capitals, corrections |
| `Address` | URLs, hosts |
| `Login` | a name or e-mail address to sign in with |
| `Password` | learns nothing; the field is masked |
| `Number` | digits with the decimal separator |

## API

```rust
// in the owner's new():
let amount = cx.new(|cx| TextFieldState::amount(window, cx));
let name = cx.new(|cx| TextFieldState::new(Purpose::Plain, window, cx));
cx.subscribe(&amount, |this, _, event: &FieldEvent, cx| match event {
    FieldEvent::Change => {}
    FieldEvent::Submit => {} // Enter, or the keyboard's action key
    FieldEvent::Focus | FieldEvent::Blur => {}
});

// in render():
TextField::new(&self.amount).label("Amount").suffix("₽").help("Either separator works")

// reading and validating:
let value = amount.read(cx).number(cx);          // reads "1 240,50", "1240.5", "-60"
amount.update(cx, |f, cx| f.set_error(Some("Type an amount"), cx));
```

Typing clears the error. `input()` exposes the inner gpui-kit `InputState` of a one-line field for anything beyond this.

- A `Password` field is masked, with an eye to look at it before it is sent.
- `TextFieldState::multiline(purpose, min, max, ..)` is a field of several lines growing from `min` to `max` (details in Markdown, an environment): Enter breaks the line.
- `TextFieldState::prompt(min, max, ..)` is what to ask, in a few lines (a conversation's first message): on a pointer Enter commits the form and Shift+Enter breaks the line; on a finger Enter breaks the line and the form's button commits.
- `set_placeholder` and `set_value` work on either.
- `.icon(icon)` starts a one-line field with an icon (a magnifier for a search); `.cleanable()` adds a cross that empties it.

## A field that drives a list

A switcher's results, the completions of what is written, a search's hints: the field keeps the focus and the keys move through the list.

```rust
TextField::new(&self.search).icon(IconName::Search).on_key(move |key, window, cx| match key {
    FieldKey::Up => { step(-1); true }       // taken: the text never sees it
    FieldKey::Down => { step(1); true }
    FieldKey::Enter | FieldKey::Tab if list_open => { pick(cx); true }
    _ => false,                              // goes on to the text as usual
})
```

`on_key` is offered Up, Down, Enter (without Shift), Tab and Escape before the text, as actions on their way down. Enter is taken there, before the field could treat it as typing.

`.on_paste(|item, window, cx| -> bool)` looks at what is pasted first: `true` when it was used (an image attached, an address made a link) and the text gets nothing.

The owner can work with the text where the person is:

| | |
|---|---|
| `focus(window, cx)`, `is_focused(window, cx)` | put the focus in it; ask whether it has it |
| `cursor(cx)`, `set_cursor(offset, cx)` | where the cursor is, in bytes of the value; move it |
| `selected_text(cx)`, `replace_selection(text, window, cx)` | what is selected; put text in its place as typing would (undoable) |

matterfast's composers are built so: `TextFieldState::prompt`, a completion list driven by `on_key`, a completion put in at the cursor, a pasted address wrapped around the selected words.

## Tests

`tests/ui.rs`: `a_field_offers_the_keys_that_move_a_list_before_the_text`.

`field::tests` covers number parsing: either separator, spaces and no-break spaces between thousands, empty, and not a number.
