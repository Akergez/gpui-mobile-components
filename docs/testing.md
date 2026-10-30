# Testing

[index](README.md)

## Building on this machine

The host has no fontconfig or xkbcommon headers. `build-aux/cargo.sh` runs cargo in the `backseating-dev` distrobox (which shares `$HOME`) when the host lacks them, and plain cargo elsewhere:

```sh
build-aux/cargo.sh test --offline -p gpui-adaptive
build-aux/cargo.sh build --offline -p gallery
```

## What is tested

| Layer | Where | What |
|---|---|---|
| pure | `src/*::tests` | width bounds, scale, command order, number and date parsing, sheet placement and motion |
| GPUI context | `navigator::tests` | the stack: push, back, the companion fallback, siblings replacing each other |
| UI integration | `tests/ui.rs` | real clicks in headless windows, **in both skins**: the primary command as a button and as a FAB; a message beside the inbox on a desktop and over it on a phone, with up going back; destinations replacing the stack; a choice opening a sheet and choosing; a toggle |

Each UI test sets the input mode the way an application's entry point does (`gpui_adaptive::init`) and opens a window at a desktop size (1280×800) or a phone size (400×800). Custom pressable elements register themselves for queries with `.test_support()` (gpui-base's `TestSupportExt`); their element ids come from the domain: a command's id, a destination's key, an option's key.

The touch sheet moves by the clock, as on a phone. A test that opens one waits for `Scale::touch().sheet_open` before pressing inside it.

## Looking at it

The gallery shows every intent:

```sh
build-aux/cargo.sh run -p gallery
GALLERY_INPUT=touch GALLERY_SIZE=400x800 build-aux/cargo.sh run -p gallery
```

| Variable | Meaning |
|---|---|
| `GALLERY_INPUT` | `touch` or `pointer` |
| `GALLERY_SIZE` | `WxH`, for example `400x800` for a phone |
| `GALLERY_ROUTE` | `lists`, `person`, `input`, `records` or `feedback` |
| `GALLERY_DEMO` | `form`, `notice`, `confirm`, `actions` or `lookup`: opens it after the first frames |
| `GPUI_KEYBOARD` | a height in pixels: pretends an on-screen keyboard is up |

`build-aux/shots.sh` takes the screenshots in `docs/images/` under a private headless sway (the UI image the applications use):

```sh
build-aux/cargo.sh build --offline -p gallery
podman run --rm --userns=keep-id --cap-add=SYS_NICE --security-opt label=disable \
  -v "$PWD":/src -v "$HOME/.cargo/registry":"$HOME/.cargo/registry":ro -w /src \
  localhost/aiddiction-ui build-aux/shots.sh
```

The registry is mounted because a debug build reads gpui-kit's icons from it.

## What is not covered yet

- Touch gestures in scripted scenarios: dragging the sheet's handle, the long press, swipe back. These need raw `TouchEvent`s (see the gpui-mobile skill) or a device.
- Running on an Android device. The crate builds on the same glue and gpui-mobile revision as the applications, but has not been built for `aarch64-linux-android` here.
