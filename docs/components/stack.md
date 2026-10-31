# Stack: the parts of a page

`gpui_adaptive::stack` · [index](../README.md)

A stack lays a page's parts one under another: a project's agents, its board and its conversations; a settings page's settings; a note over a list. Both skins centre the parts in a column at most 56 rem wide (`Stack::width`), with the page's margins and `space_xl` between the parts.

```rust
Screen::new("agents", "Agents").content(
    Stack::new()
        .child(Note::new("What the server can start in a project."))
        .child(Collection::new("agents-list").children(rows)),
)
```

A part wider than the column (a board) scrolls in it with `prim::scroll_x`.
