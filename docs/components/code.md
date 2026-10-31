# Code: text a machine wrote

`gpui_adaptive::code` · [index](../README.md)

`Code` shows a program's output as it was written: an agent's last lines of stderr, what a sign-in on the server prints. Both skins draw a monospaced block on the sunken colour, in the small size.

## API

```rust
Code::new(("stderr", instance.id), tail)
Code::new("sign-in-output", output).max_height(rems(10.))  // scrolls past it, locked to its axis
```

A diff is not here: what an edit did reads differently in each application. aiddiction draws its own from a `Well` [surface](primitives.md) and mono [`Text`](primitives.md).
