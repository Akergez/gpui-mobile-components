# Composer: writing a message

`gpui_adaptive::composer` · [index](../README.md)

The composer writes a message to someone in a channel (matterfast) or to an agent (aiddiction). It is a text area that grows with what is written, up to eight lines, with tools around it and send. Things can stand above it: what is replied to, an agent's plan.

| | Pointer | Touch |
|---|---|---|
| Tools | icon buttons beside the field | folded into a + that opens them in a [sheet](sheet.md) |
| Enter | sends; Shift+Enter breaks the line | breaks the line; the send button sends, as on a phone |
| Keyboard | — | `Purpose::Message`: capitals and corrections; the composer rides on it (the page pads by `Insets::typing()`) |
| Send | disabled while the draft is empty | the same |

| Pointer | Touch |
|---|---|
| ![](../images/desktop-person.png) | ![](../images/phone-person.png) |

## API

```rust
let composer = cx.new(|cx| ComposerState::new("Write a message…", window, cx));
cx.subscribe_in(&composer, window, |this, composer, event: &ComposerEvent, window, cx| {
    if *event == ComposerEvent::Send {
        let text = composer.read(cx).text(cx);
        this.post(text, cx);
        composer.update(cx, |c, cx| c.clear(window, cx));
    }
});

Composer::new(&self.composer)
    .above(reply_to_line)
    .tool(Command::new("attach", "Attach files").icon(Lucide::Paperclip).action(Attach))
    .tool(Command::new("priority", "Priority").icon(IconName::CircleAlert).action(SetPriority))
```

The draft belongs to the owner: `text()` reads it and `clear()` empties it after sending. Completion lists for mentions and commands remain the application's for now. Draw them with `above`.
