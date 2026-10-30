# gpui-adaptive: intents for desktop and phone

`gpui-adaptive` is a design system for gpui-kit applications that run on a desktop and on a phone: aiddiction, matterfast and money-manager. You write a screen once, against what the person is doing: a [`Screen`](components/screen.md) with [`Command`](components/command.md)s, rows of a [`Collection`](components/list.md), a [`Choice`](components/choice.md). Each intent draws itself in two skins:

- **Pointer** is gpui-kit as its Design Guides draw it: dense, hover, menus, dialogs, keyboard first.
- **Touch** is Material 3: 48 dp targets, a pressed state layer, a navigation bar, a FAB, bottom sheets and gestures.

| Desktop | Phone |
|---|---|
| ![desktop](images/desktop-person.png) | ![phone](images/phone-lists.png) |

## How a frame chooses its skin

[`Env`](components/env.md) answers two questions:

| Axis | Values | Decided by | Decides |
|---|---|---|---|
| `InputMode` | `Pointer`, `Touch` | the application's entry point, once (`gpui_adaptive::init`) | the skin, and which backend a stateful intent builds |
| `WidthClass` | `Compact` < 44 rem ≤ `Medium` < 66 rem ≤ `Expanded` | the window, every frame, in rems | layout: bar, rail or sidebar; one pane or columns; list or table |

A tablet gets columns with touch feedback. A narrow desktop window gets one pane with hover and menus.

## What it stands on

Nothing here is a new engine:

- **Colours** come from `gpui-adaptive-colors`: a Material You scheme from one seed, or a Zed theme the person wears. Everything is read through `cx.theme()`. The touch skin's extra roles come from [`Palette`](components/palette.md), which falls back to the theme while a Zed theme is worn.
- **Phone plumbing** comes from `gpui-android-glue`: back (layers, predictive back, swipe back), the keyboard (insets, purpose, raise on tap) and the system bars. See [Insets](components/insets.md).
- **Behaviour** is gpui-kit's wherever it has it: `Select`, `Dialog`, `Notification`, `Input`, `Calendar`, `Switch`, `Checkbox`, `Spinner`, `Progress`, `Skeleton`.

## Setup

```rust
gpui_kit::application().with_assets(gpui_kit::assets::Assets).run(|cx| {
    gpui_kit::init(cx);
    gpui_adaptive_colors::init(Options::new(Color::from_u32(0x0b57d0)), cx);
    gpui_android_glue::init(cx);                 // `launch::run` does it on Android
    gpui_adaptive::init(InputMode::Pointer, cx); // InputMode::Touch from android_main
    // open the window; its root view draws an AppFrame
});
```

## The components

**Foundations**
- [Env](components/env.md): `InputMode`, `WidthClass`, `Env::of`
- [Scale](components/scale.md): lengths and durations of each skin
- [Palette](components/palette.md): Material roles that survive a Zed theme
- [Insets](components/insets.md): status bar, navigation bar, keyboard
- [AppFrame](components/app-frame.md): the window's frame and its layers

**Structure**
- [Destinations](components/destinations.md): sections as a bar, rail or sidebar
- [Screen](components/screen.md): a page with a title and commands
- [Navigator](components/navigator.md): columns or a stack, with predictive back
- [Sheet](components/sheet.md): a bottom sheet or a dialog
- [Dock](components/dock.md): something that outlives the page

**Actions**
- [Command](components/command.md): one action, many entry points
- [ItemActions](components/item-actions.md): hover bar and context menu, or long press
- [Confirm](components/confirm.md): a question before something irreversible
- [Ask](components/ask.md): a question with a few answers, in the flow or in a dialog
- [Notice](components/notice.md): a toast or a snackbar, with Undo

**Input**
- [TextField](components/text-field.md): text and amounts, with purpose and errors
- [Form](components/form.md): fields with one commitment
- [Choice](components/choice.md): one of a known list
- [Lookup](components/lookup.md): one or several from a directory
- [Segments](components/segments.md): two to four exclusive modes
- [Toggle](components/toggle.md): on or off
- [DateField](components/date-field.md): a date, typed or picked
- [Composer](components/composer.md): writing a message

**Content**
- [ListRow, SectionHeader, Collection](components/list.md)
- [Records](components/records.md): a table or a list

**Feedback**
- [Banner](components/banner.md): a standing state of the page
- [Busy and Skeleton](components/busy.md): work under way
- [Empty](components/empty.md): nothing yet, and what to do
- [Avatar and Count](components/identity.md): who, and how many

## Rules for feature code

1. Do not ask for the platform. `Env::of(window, cx).is_touch()` is the one escape hatch, and every use of it in a feature is a missing intent.
2. Give repeated things stable ids from the domain (`("channel", id)`), never positions.
3. Model each command once, as a [`Command`](components/command.md). Its button, menu item, FAB and key binding then cannot disagree.
4. Keep every way of input: typing a date as well as picking it, a shortcut as well as a button, a menu as well as a gesture.
5. Test each screen in both skins (see [Testing](testing.md)).

## What stays in the applications

The proposal in Claude Design also had the surfaces below. Each serves one application, so they are not intents of the design system. Each application builds them itself, from the intents:

| Surface | Application | Built from |
|---|---|---|
| Message feed | matterfast | gpui-kit's `Message` and `MessageScroller`; [ItemActions](components/item-actions.md) for hover and long press; [Screen](components/screen.md) `.scrolls(false)`; [Composer](components/composer.md) |
| Agent conversation | aiddiction | the same, plus [Confirm](components/confirm.md)-style inline answers for permission requests |
| Task board | aiddiction | [Route](components/navigator.md)`::scrolls_sideways` so swipe back does not take the board's direction; [ItemActions](components/item-actions.md) "Move to…" on touch |
| Diff review | aiddiction | `diffreview-ui`; `Route::scrolls_sideways`; [Sheet](components/sheet.md) for a line's note on touch |
| Charts | money-manager | gpui-kit `chart`; [Segments](components/segments.md) for the month; colours `chart_1`–`chart_5` from the theme |

The chats in matterfast and aiddiction look alike, but what they hold differs (posts with reactions and threads, against agent runs with tools and plans). The shared parts (`Message`, `Bubble`, `MessageScroller`) are gpui-kit's already.

## Not done yet

These are general, and belong here:

- Swipe actions on rows, and selection mode (a long press entering multi-select).
- A ripple from the touch point. The touch skin currently uses Material's pressed state layer.
