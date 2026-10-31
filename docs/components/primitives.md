# Primitives: what an application builds its own parts from

`gpui_adaptive::prim`, `gpui_adaptive::colors` · [index](../README.md)

An application has parts no intent is: aiddiction's agent steps, its plan, its board, its diffs. It builds them itself, from the primitives below and gpui's `div`, and never from gpui-kit's components. Such a part then looks like the rest in either skin without asking which one it is in.

| Primitive | What it is |
|---|---|
| `cx.colors()` ([`Colors`]) | the colours by role, resolved for the skin: `text`, `muted`, `surface`, `raised`, `sunken`, `border`, `accent`/`on_accent`, `hover`, `pressed`, `mine`/`on_mine`, the tones, `mono_font` |
| `cx.scale()` ([Scale](scale.md)) | lengths: `target`, `row`, the radii, the type sizes, and the room between things: `space_xs` to `space_xl` |
| `Icon`, `IconName`, `Sizable` | the system's icons (gpui-kit's set), `.xsmall()`/`.small()` |
| `Lucide` | the whole Lucide catalog, for an icon the system's set does not name; `AllAssets` is where their files are, for the application's asset source |
| `row()`, `column()` | flex boxes; the gap is the scale's |
| `surface(Surface::Card \| Panel \| Well, cx)` | a card (bordered, raised), a panel (sunken, holds cards), a well (sunken, for machine text) |
| `pressable(id, cx)` | a row that can be pressed: hover and press colours on a pointer, the pressed layer on a finger. `testable(el)` finishes it so a UI test finds it by its id |
| `scroll_y(id, cx, content)`, `scroll_x(id, cx, row)` | a scroll area, locked to its axis for a finger |
| `Scrolls`: `.scrolls_y()`, `.scrolls_x()` | scrolling along one axis on an element laid out by the application (a list of bounded height in a popup, a strip of tabs), the axis locked |
| `list_scrollbar(holder, &list_state, cx)` | a scrollbar for the gpui `list` in `holder` (a feed too long to lay out at once); a finger scrolls without one |
| `tooltip(text)` | for gpui's `.tooltip(…)` |
| `spinner()` | work under way, the size of a line of text |
| `Text::title \| body \| small \| mono(text)` | text in the skin's sizes; `.muted()`, `.tone(t)`, `.strong()`, `.truncate()`, `.clamp(n)`, `.grow()`, `.center()` |
| `Markdown::new(id, text)` | Markdown that can be selected; `.on_link(f)`, `.on_link_menu(f)` (a right click on a link, with the point: for [`Menu::open_at`](menu.md)), `.selectable(false)`, `.inherit_size()` (the size of what holds it, not the body size), `.extensions(..)` |
| `Meter::new(fraction)` | a short bar of how far along something is |

| Colours | Pointer | Touch |
|---|---|---|
| `raised` / `sunken` | the theme's background / `muted` | `surface` / `surface_container` |
| `mine` | `secondary` | `primary_container` |
| `hover` / `pressed` | `accent` / `list_active` | the pressed layer, both |

## An application's own part

aiddiction's agent step (`ui/work.rs`):

```rust
let head = pressable(id, cx)
    .gap(scale.space_s)
    .px(scale.space_m)
    .min_h(scale.target)
    .on_click(move |_, window, cx| on_toggle(window, cx))
    .child(Icon::new(IconName::SquareTerminal).small().text_color(colors.muted))
    .child(Text::small(title).truncate().grow())
    .child(status_mark);
surface(Surface::Card, cx).child(head).when(open, |card| card.child(output))
```

## An application's own Markdown

Custom emoji, mentions: a plugin of `gpui_adaptive::markdown` (`MarkdownPlugin`, `InlineElement`, …) finds them in the text and draws them inline. Build the `MarkdownExtensions` once and clone them into every `Markdown::extensions`: a clone keeps its revision, which lets the text keep what it parsed from one frame to the next.

## Theme and keys

- `gpui_adaptive::theme`: what a settings page lets the person choose: `ThemeMode`, `Theme` (the font, changing the mode), `ThemeConfig` (a catalogue of Zed themes), `theme::name(cx)`. Colours to paint with come from `cx.colors()`, not from here.
- `gpui_adaptive::keys::Escape`: for a handler around the window's content that closes what is in front even while a field has the focus.
- `AppFrame::window_options()`: the window options that go with the frame's title bar.

## Keeping to it

An application can make the rule a check. matterfast's `tests/design_system.rs` fails on any `gpui_kit::component`, `gpui_kit::base` or `gpui_kit::assets` in its sources; its `clippy.toml` names the components one by one, with what to use instead. aiddiction's `crates/aiddiction/clippy.toml` disallows gpui-kit's `h_flex`, `v_flex`, `ActiveTheme::theme`, `Button`, `Spinner`, `TextView`, `Tooltip`, `TitleBar`, `Input`, `Select` and `ListItem`, each with the intent or primitive to use instead. Keep the file in the application's crate, not at the workspace root: crates of the workspace that are gpui-kit components themselves (`diffreview-ui`) must not get it.

## Tests

`tests/ui.rs`:
- `a_fold_a_swatch_and_a_pressable_answer_a_press_in_both_skins`
- `the_colours_by_role_follow_the_skin`
