# Note: a sentence the page says

`gpui_adaptive::note` · [index](../README.md)

A note is what a page says in its own voice: what a list is for ("What the server can start in a project…"), what a form is doing ("Checking…"), why a section is empty, a path under a step. It is secondary text in the skin's small size.

A [Banner](banner.md) is a state of the whole page and an [Empty](empty.md) stands for a page with nothing on it. A note is neither.

## API

```rust
Note::new("Reading the server's folders…")
Note::new(error).tone(Tone::Danger)   // the danger colour and its icon
Note::new(path).truncate()            // one line, cut with an ellipsis
```
