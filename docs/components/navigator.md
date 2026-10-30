# Navigator: columns when there is room, a stack when not

`gpui_adaptive::navigator` · [index](../README.md)

The application describes its pages as `Route`s. A route may name its **companion**, the page shown beside it when the window is wide enough: a conversation's companion is the conversation list, and a record's is the records. matterfast's `layout/split.rs` ("on a phone the list is the main screen, as in Telegram") and aiddiction's page stack are both this one intent.

| Window | What is shown |
|---|---|
| Medium, Expanded | the companion as a column of `Route::column_width()` (22 rem by default), the page beside it; neither has a way up |
| Compact | the top of the stack, with a way up. A page comes in sliding from the side (touch, 300 ms) or fading (pointer, 120 ms). |

| Desktop: columns | Phone: stack |
|---|---|
| ![](../images/desktop-person.png) | ![](../images/phone-person.png) |

## Beside a root page: `wide_detail`

When nothing else stands beside a page, `Route::wide_detail()` names what does, if there is room. Examples: the inbox's "Pick a conversation", or the first settings page beside the list of settings (aiddiction's `shell.rs:364`). On a compact window it is not shown, and back never goes to it. Make it a [Screen](screen.md) too, so that a page bar in the title bar has a page to be over.

| Desktop | Phone |
|---|---|
| ![](../images/desktop-pick.png) | ![](../images/phone-lists.png) |

```rust
fn wide_detail(&self) -> Option<Self> {
    match self { Route::Inbox => Some(Route::NothingOpen), Route::Settings => Some(Route::Appearance), _ => None }
}
```

## Back

While the navigator is a stack and back goes somewhere, it holds a `gpui_android_glue::back` layer:

- **The phone's back** (the button or the predictive gesture) pops the page. Partway through the gesture the page moves aside with the finger over the page it goes to (`back::preview`, with the page under drawn only while there is a gesture).
- **On a finger, a swipe to the right** anywhere across the page does the same (`swipe_back`), unless `Route::scrolls_sideways()` says the page has something that scrolls sideways (a diff, a board, a wide table).
- **Up** in the [Screen](screen.md) does the same.

`back()` pops the top. If only one page is left and it has a companion (opened straight onto a message), back goes to the companion.

## API

```rust
#[derive(Clone, PartialEq)]
enum Route { Channels, Channel(ChannelId), Thread(PostId) }

impl gpui_adaptive::Route for Route {
    fn companion(&self) -> Option<Self> {
        match self { Route::Channel(_) => Some(Route::Channels), _ => None }
    }
    fn scrolls_sideways(&self) -> bool { false }
}

let navigator = cx.new(|_| NavigatorState::new(Route::Channels));
navigator.update(cx, |n, cx| n.push(Route::Channel(id), cx));   // open
navigator.update(cx, |n, cx| n.replace_root(Route::Settings, cx)); // a destination
navigator.update(cx, |n, cx| n.back(cx));                         // -> bool
navigator.update(cx, |n, cx| n.retain(|r| r != &Route::Chat(closed), cx)); // a closed chat leaves the history

Navigator::new("pages", &navigator, |route, page, window, cx| match route {
    Route::Channels => channel_list(page, cx),
    Route::Channel(id) => Screen::new("channel", name(id)).page(&page).into_any_element(),
    ..
})
```

- `push` of the page already on top does nothing. Opening a sibling that has the same companion replaces the page beside it, so the list → A → B → back sequence returns to the list, not to A.
- `retain` keeps only the pages it is told to (aiddiction drops a closed conversation from the history). The first page stays if nothing else would.
- `NavigatorState` emits `NavigatorEvent::Changed` and is a plain stack: a UI scenario can start on any page.
- `render_page` may run for the page under the top while a gesture shows it, so it must not change anything.

## Tests

- `navigator::tests`: back pops, then falls back to the companion; a sibling replaces the page beside its companion; `retain` drops closed pages and keeps a first one.
- `tests/ui.rs`: `a_root_page_has_its_wide_detail_beside_it_only_where_there_is_room`.
- `tests/ui.rs`: `on_a_phone_a_message_is_a_page_over_the_inbox_and_up_goes_back` and `on_a_desktop_the_message_opens_beside_the_inbox`.
