# ListRow, SectionHeader, Collection

`gpui_adaptive::list` · [index](../README.md)

These are lists of things: conversations, channels, records, projects, settings.

| | Pointer | Touch |
|---|---|---|
| `ListRow` | dense: 1.75 rem, or 2.75 rem with a second line; the theme's hover; rounded | Material list item: 56 dp, or 72 dp with a second line; the pressed state layer |
| selected | the sidebar's accent | the secondary container |
| commands | hover bar and right click ([ItemActions](item-actions.md)) | long press → sheet |
| `SectionHeader` | small, semibold, muted | small, semibold, in the primary colour |
| a tone on the header | the tone's colour ("Needs you" in warning) | the same |
| `supporting` | a third line, small and muted; in a tone's colour with its icon | the same, in a three-line item |
| `signal` | a stripe down the row's edge for what waits or works, a dot or a spinner under the trailing text | the dot or the spinner only |
| `Collection` | rows with a 1 px gap and padding | rows edge to edge |

| Pointer | Touch |
|---|---|
| ![](../images/desktop-lists.png) | ![](../images/phone-lists.png) |

## API

```rust
Collection::new("channels")
    .section(SectionHeader::new("Favourites"), favourites.iter().map(|c| {
        ListRow::new(("channel", c.id), c.name.clone())
            .leading(Avatar::new(("channel-face", c.id), c.initials()))
            .subtitle(c.last_message.clone())
            .trailing(Count::new(c.unread).urgent(c.mentioned))
            .unread(c.unread > 0)
            .muted(c.muted)
            .selected(Some(c.id) == open)
            .actions(channel_actions(c))
            .on_activate(move |window, cx| open_channel(c.id, window, cx))
    }))
    .section(SectionHeader::new("Channels"), others.iter().map(channel_row))
```

A grouped list is a collection of `section`s, each a header and its rows: the grouping is said once, never by putting headers between rows by hand. A section without rows is not drawn. aiddiction's inbox is one section per thing a conversation may need of the person, in `crate::inbox`'s order:

```rust
let header = SectionHeader::new(group.title());
let header = if group == Group::NeedsYou { header.tone(Tone::Warning) } else { header };
list = list.section(header, run.iter().map(|item| session_row(item)));
```

- `.supporting(text)` is a third line under the subtitle: the last message, what went wrong. `.supporting_tone(Tone::Danger)` says it in the tone's colour with its icon.
- `.signal(Some(Signal::Waiting(Tone::Warning)))` says what the row's state asks of the person: `Waiting(tone)` (an agent's question, a failure), `Busy` (something works on it now), `New` (unread). It is the stripe and the dot of aiddiction's inbox rows.

- `.body(element)` puts something to act on inside the row, under its text: the answers to an agent's question in an inbox ([Ask](ask.md)), Accept and Decline for an invitation. A press in the body is its own and does not open the row. With a body, the leading and trailing parts align to the top.
- Inside a [Screen](screen.md) the screen scrolls. A collection that is a column of its own says `.scrolls(true)`; on touch its scroll is locked to its axis.
- Every row needs a stable id from the domain, never a position: positions move when rows are inserted.
- Thousands of rows belong in gpui-kit's `v_virtual_list`, with `ListRow`s as its items.

## States

| State | Look |
|---|---|
| `unread` | bold title |
| `muted` | dimmed |
| `selected` | container colour, where the open item stands beside the list; not on a compact window, which shows the list alone |
| loading | [Skeleton](busy.md) rows |
| empty | [Empty](empty.md) |

## Tests

`tests/ui.rs` presses rows in both skins in the navigator tests. `a_grouped_list_keeps_its_sections_order_in_both_skins` checks that rows come in their sections' order, that a section without rows takes no room, and that a row with a supporting line and a signal still opens. `an_answer_inside_a_row_does_not_open_the_row` checks, in both skins, that a press on an answer runs the answer only, and a press on the row's text opens it.

## In a form

`Collection::boxed(height)` is a list inside a [Form](form.md): a framed box of a fixed height that scrolls, so the form keeps its size (and its buttons their place) as the rows change.
