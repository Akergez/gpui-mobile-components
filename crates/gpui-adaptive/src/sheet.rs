//! A short task or a set of choices over the page.
//!
//! **Touch**: a sheet over the bottom of the window, the way a phone has one
//! (money-manager's `bottom_sheet.rs`, generalised). A handle across its top
//! drags it: up to make it as tall as the window allows, down past
//! [`PUT_AWAY_AT`] rems to put it away; short of that it settles back. The
//! window is dimmed behind it and a press there closes it, and so do Escape
//! and the phone's back (each sheet is a layer of `gpui_android_glue::back`).
//! Sheets stack: one opened from a sheet opens over it, and closing it
//! uncovers the one under it as it was. What the sheet holds keeps its state
//! the whole way, because the sheet moves by its height over a few frames
//! rather than by re-creating what it holds. It stays above the keyboard and
//! the navigation bar.
//!
//! **Pointer**: the content in a gpui-kit dialog, opened through
//! `gpui_android_glue::back::open_dialog` so that a tablet's back closes it.
//!
//! The sheets are drawn by the window's [`crate::AppFrame`]; a window without
//! one shows none.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::{ActiveTheme, FocusTrapElement, WindowExt, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyView, App, Bounds, Context, DispatchPhase, FocusHandle, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Render, SharedString, TouchDragEvent, TouchPhase, Window,
    WindowId, canvas, div, px,
};

use crate::command::{Command, in_offer_order};
use crate::env::{InputMode, input_mode};
use crate::insets::Insets;
use crate::layers;
use crate::palette::ActivePalette;
use crate::scale::ActiveScale;
use crate::touch;

/// How far down, in rems, the handle has to be dragged for the sheet to go
/// away when it is let go; short of that it settles back.
pub const PUT_AWAY_AT: f32 = 5.0;

/// The least of the window, in rems, left showing above a sheet.
const GAP_ABOVE: f32 = 2.0;

/// Opening, closing and inspecting sheets.
pub struct Sheet;

/// How a sheet with a title is shown: [`Sheet::open_with`].
#[derive(Clone, Debug, Default)]
pub struct SheetOptions {
    /// What it is, over its content: "Settings", "Browse channels".
    pub title: Option<SharedString>,
    /// How wide the dialog is on a pointer; a sheet is the window's width.
    pub width: Option<gpui_kit::Rems>,
    /// Whether a pointer's dialog has no close button, for content that has
    /// its own way out (a switcher that closes on a choice or on Escape).
    pub no_close_button: bool,
}

impl Sheet {
    /// Opens `content` over the page: a bottom sheet over any that is open
    /// (touch), or a dialog (pointer). `content` draws its own title and its
    /// own commitment; [`crate::Form`] is a form in one.
    pub fn open(content: impl Into<AnyView>, window: &mut Window, cx: &mut App) {
        let content = content.into();
        match input_mode(cx) {
            InputMode::Touch => open_bottom(content, window, cx),
            InputMode::Pointer => {
                gpui_android_glue::back::open_dialog(window, cx, move |dialog, _, _| {
                    dialog.child(content.clone())
                });
            }
        }
    }

    /// Opens `content` under a title, at a width for a pointer: what a
    /// panel of the application's own is (its settings, a list to browse).
    pub fn open_with(content: impl Into<AnyView>, options: SheetOptions, window: &mut Window, cx: &mut App) {
        let content = content.into();
        match input_mode(cx) {
            InputMode::Touch => {
                let titled = cx.new(|_| Titled { title: options.title, content });
                open_bottom(titled.into(), window, cx)
            }
            InputMode::Pointer => {
                gpui_android_glue::back::open_dialog(window, cx, move |dialog, window, _| {
                    let dialog = match options.title.clone() {
                        Some(title) => dialog.title(title),
                        None => dialog,
                    };
                    let dialog = match options.width {
                        Some(width) => dialog.w(width.to_pixels(window.rem_size())),
                        None => dialog,
                    };
                    dialog.close_button(!options.no_close_button).child(content.clone())
                });
            }
        }
    }

    /// Closes the topmost sheet (or the dialog, on a pointer): it slides
    /// down out of the window, and the focus goes back to what had it.
    pub fn close(window: &mut Window, cx: &mut App) {
        match input_mode(cx) {
            InputMode::Touch => close_bottom(window, cx),
            InputMode::Pointer => window.close_dialog(cx),
        }
    }

    /// Whether a sheet (or, on a pointer, a dialog) is open over `window`.
    pub fn is_open(window: &mut Window, cx: &mut App) -> bool {
        match input_mode(cx) {
            InputMode::Touch => layers::top_sheet(window, cx).is_some(),
            InputMode::Pointer => window.has_active_dialog(cx),
        }
    }

    /// Offers `commands` as a sheet of rows under `title`: what a long press
    /// opens on a phone. Choosing one closes the sheet, then runs it. On a
    /// pointer it is a dialog of the same rows; a pointer usually offers
    /// these as a menu instead ([`crate::ItemActions`]).
    pub fn actions(
        title: Option<SharedString>,
        commands: Vec<Command>,
        window: &mut Window,
        cx: &mut App,
    ) {
        let view = cx.new(|_| ActionList { title, commands: in_offer_order(&commands) });
        Self::open(view, window, cx);
    }
}

/// Content under a title, in a sheet.
struct Titled {
    title: Option<SharedString>,
    content: AnyView,
}

impl Render for Titled {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let scale = cx.scale();
        v_flex()
            .flex_grow(1.)
            .min_h_0()
            .when_some(self.title.clone(), |sheet, title| {
                sheet.child(div().flex_none().px_4().pb_3().text_size(scale.title).child(title))
            })
            .child(v_flex().flex_grow(1.).min_h_0().child(self.content.clone()))
    }
}

/// A list of commands in a sheet.
struct ActionList {
    title: Option<SharedString>,
    commands: Vec<Command>,
}

impl Render for ActionList {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let env = crate::Env::of(window, cx);
        let palette = cx.palette();
        let scale = cx.scale();
        let rows = self.commands.iter().map(|command| {
            let chosen = command.clone();
            let danger = command.is_destructive();
            touch::pressable(command.id().clone(), env, cx)
                .h(scale.row)
                .px_4()
                .gap_4()
                .when(command.is_disabled(), |row| row.opacity(0.4))
                .when(danger, |row| row.text_color(palette.error))
                .children(command.icon_ref().cloned())
                .child(div().flex_1().child(command.label().clone()))
                .when(command.is_checked() == Some(true), |row| {
                    row.child(gpui_kit::component::Icon::new(gpui_kit::component::IconName::Check))
                })
                .on_click(move |_, window, cx| {
                    let chosen = chosen.clone();
                    Sheet::close(window, cx);
                    // After the sheet has let go of the focus, so that an
                    // action is dispatched from what had it before.
                    window.defer(cx, move |window, cx| chosen.run(window, cx));
                })
                .test_support()
        });
        v_flex()
            .pb_2()
            .when_some(self.title.clone(), |list, title| {
                list.child(
                    div()
                        .px_4()
                        .pb_2()
                        .text_size(scale.small)
                        .text_color(palette.on_surface_variant)
                        .child(title),
                )
            })
            .children(rows)
    }
}

/// A sheet over the bottom of the window. Opened by [`Sheet::open`] in the
/// touch skin; kept and drawn by the window's [`crate::AppFrame`].
pub struct BottomSheet {
    content: AnyView,
    window: WindowId,
    focus: FocusHandle,
    /// What had the focus before the sheet opened, which gets it back.
    previous: Option<FocusHandle>,
    drag: Option<Drag>,
    /// Whether it has been pulled up to all the room there is.
    expanded: bool,
    closing: bool,
    motion: Option<Motion>,
    /// How tall what it holds is, with the handle and the room at the
    /// bottom: its height at rest. Measured as it is drawn.
    natural: Rc<Cell<Pixels>>,
    /// All the room there is above the bottom of the window, as last drawn.
    full: Pixels,
    _back: gpui_android_glue::back::Guard,
}

/// A drag of the handle: where it started and how tall the sheet was then.
#[derive(Clone, Copy)]
struct Drag {
    from: Pixels,
    height: Pixels,
    at: Pixels,
}

/// The sheet going somewhere over a few frames.
#[derive(Clone, Copy)]
struct Motion {
    /// The height it started from; `None` for out of sight below.
    from: Option<Pixels>,
    started: Instant,
    takes: Duration,
}

impl Motion {
    fn new(from: Option<Pixels>, takes: Duration) -> Self {
        Motion { from, started: Instant::now(), takes }
    }

    /// How far along it is, eased out: fast at first, slow into place.
    fn progress(&self) -> f32 {
        if self.takes.is_zero() {
            return 1.;
        }
        let t = (self.started.elapsed().as_secs_f32() / self.takes.as_secs_f32()).clamp(0., 1.);
        1. - (1. - t).powi(3)
    }

    fn done(&self) -> bool {
        self.started.elapsed() >= self.takes
    }
}

fn open_bottom(content: AnyView, window: &mut Window, cx: &mut App) {
    let previous = window.focused(cx);
    let opening = cx.scale().sheet_open;
    let window_id = window.window_handle().window_id();
    let sheet = cx.new(|cx| BottomSheet {
        content,
        window: window_id,
        focus: cx.focus_handle(),
        previous,
        drag: None,
        expanded: false,
        closing: false,
        motion: Some(Motion::new(None, opening)),
        natural: Rc::new(Cell::new(Pixels::ZERO)),
        full: Pixels::ZERO,
        _back: gpui_android_glue::back::push(|window, cx| {
            let open = layers::top_sheet(window, cx).is_some();
            close_bottom(window, cx);
            open
        }),
    });
    layers::push_sheet(window, sheet, cx);
}

fn close_bottom(window: &mut Window, cx: &mut App) {
    let Some(sheet) = layers::top_sheet(window, cx) else {
        return;
    };
    let closing = cx.scale().sheet_close;
    let previous = sheet.update(cx, |sheet, cx| {
        let from = sheet.height_now();
        sheet.closing = true;
        sheet.drag = None;
        sheet.motion = Some(Motion::new(Some(from), closing));
        cx.notify();
        sheet.previous.clone()
    });
    if let Some(previous) = previous {
        previous.focus(window, cx);
    }
    let (window_id, id) = (sheet.read(cx).window, sheet.entity_id());
    cx.spawn(async move |cx| {
        cx.background_executor().timer(closing).await;
        cx.update(|cx| layers::remove_sheet(window_id, id, cx));
    })
    .detach();
}

/// Where the sheet stands at `height`, given how tall what it holds is:
/// the least height to give it, if more than its own, and how far below
/// the bottom of the window it goes.
fn placement(height: Pixels, natural: Pixels) -> (Option<Pixels>, Pixels) {
    if height > natural { (Some(height), Pixels::ZERO) } else { (None, natural - height) }
}

impl BottomSheet {
    pub(crate) fn is_closing(&self) -> bool {
        self.closing
    }

    fn resting(&self) -> Pixels {
        if self.expanded { self.full } else { self.natural.get() }
    }

    fn below_window(&self) -> Pixels {
        self.natural.get().min(self.full) - self.full - px(48.)
    }

    fn target(&self) -> Pixels {
        if self.closing { self.below_window() } else { self.resting() }
    }

    fn height_now(&self) -> Pixels {
        if let Some(drag) = self.drag {
            return drag.at;
        }
        let target = self.target();
        let Some(motion) = self.motion else {
            return target;
        };
        let from = motion.from.unwrap_or(self.below_window());
        from + (target - from) * motion.progress()
    }

    fn start_drag(&mut self, y: Pixels, cx: &mut Context<Self>) {
        if self.closing {
            return;
        }
        let height = self.height_now();
        self.motion = None;
        self.drag = Some(Drag { from: y, height, at: height });
        cx.notify();
    }

    fn drag_to(&mut self, y: Pixels, cx: &mut Context<Self>) {
        let full = self.full;
        if let Some(drag) = &mut self.drag {
            drag.at = (drag.height - (y - drag.from)).min(full);
            cx.notify();
        }
    }

    fn let_go(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(drag) = self.drag.take() else {
            return;
        };
        let natural = self.natural.get();
        if natural - drag.at > window.rem_size() * PUT_AWAY_AT {
            self.drag = Some(drag);
            window.defer(cx, close_bottom);
            return;
        }
        // Past halfway to the top it goes all the way up; short of that,
        // back to its own height.
        self.expanded = self.full > natural && drag.at > (natural + self.full) / 2.;
        self.motion = Some(Motion::new(Some(drag.at), cx.scale().settle));
        cx.notify();
    }

    /// The handle: a short bar across the top in a strip tall enough for a
    /// finger. A finger's drag is offered to it before it can become a
    /// scroll of what the sheet holds.
    fn render_handle(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let this = cx.entity().downgrade();
        let palette = cx.palette();
        div()
            .id("sheet-handle")
            .relative()
            .flex_none()
            .w_full()
            .h_8()
            .flex()
            .items_center()
            .justify_center()
            .child(div().w_8().h_1().rounded_full().bg(palette.on_surface_variant.opacity(0.4)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _, cx| this.start_drag(event.position.y, cx)),
            )
            .child(
                canvas(
                    |_, _, _| {},
                    move |bounds: Bounds<Pixels>, _, window, _| {
                        let this = this.clone();
                        window.on_mouse_event(move |event: &TouchDragEvent, phase, window, cx| {
                            if phase != DispatchPhase::Bubble {
                                return;
                            }
                            let taken = this
                                .update(cx, |sheet, cx| match event.phase {
                                    TouchPhase::Started => {
                                        if !bounds.contains(&event.start_position) {
                                            return false;
                                        }
                                        sheet.start_drag(event.position.y, cx);
                                        true
                                    }
                                    TouchPhase::Moved => {
                                        sheet.drag_to(event.position.y, cx);
                                        sheet.drag.is_some()
                                    }
                                    TouchPhase::Ended | TouchPhase::Cancelled => {
                                        let dragging = sheet.drag.is_some();
                                        sheet.let_go(window, cx);
                                        dragging
                                    }
                                })
                                .unwrap_or(false);
                            if taken {
                                window.prevent_default();
                                cx.stop_propagation();
                            }
                        });
                    },
                )
                .absolute()
                .size_full(),
            )
    }
}

impl Render for BottomSheet {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rem = window.rem_size();
        let insets = Insets::of(window);
        let bottom = insets.typing();
        let room = window.viewport_size().height - insets.keyboard - insets.top - rem * GAP_ABOVE;
        self.full = room.max(Pixels::ZERO) + bottom;

        if let Some(motion) = self.motion {
            if motion.done() {
                self.motion = None;
            } else {
                window.request_animation_frame();
            }
        }
        let natural = self.natural.get();
        let height = self.height_now();
        let (least, below) = placement(height, natural);
        let handle = self.render_handle(cx);
        let palette = cx.palette();
        let scale = cx.scale();
        // The window darkens as the sheet comes up and lightens as it goes.
        let shown = if self.full > Pixels::ZERO {
            (1. - f32::from(below) / f32::from(self.full)).clamp(0., 1.)
        } else {
            1.
        };
        let mut scrim = palette.scrim;
        scrim.a *= shown;

        // How tall what the sheet holds is, measured as it is laid out, while
        // nothing pulls it taller.
        let measured = self.natural.clone();
        let pulled = least.is_some();
        let measure = canvas(
            move |bounds: Bounds<Pixels>, _, _| {
                if !pulled {
                    measured.set(bounds.size.height + bottom + px(1.));
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .size_full();

        let surface = v_flex()
            .id("sheet-surface")
            .absolute()
            .left_0()
            .right_0()
            .bottom(-below)
            .when_some(least, |surface, least| surface.min_h(least))
            .max_h(self.full)
            .occlude()
            .bg(palette.surface_low)
            .text_color(palette.on_surface)
            .rounded_t(scale.radius_sheet)
            .shadow_xl()
            .pb(bottom)
            .child(
                v_flex()
                    .relative()
                    .flex_grow(1.)
                    .flex_shrink(1.)
                    .min_h_0()
                    .child(handle)
                    .child(v_flex().flex_grow(1.).flex_shrink(1.).min_h_0().child(self.content.clone()))
                    .child(measure),
            )
            .focus_trap("sheet-focus", &self.focus);

        // While the handle is held by the pointer, it is followed wherever it
        // goes over the window.
        let dragging = self.drag.is_some().then(|| {
            let this = cx.entity().downgrade();
            canvas(
                |_, _, _| {},
                move |_, _, window, _| {
                    let moved = this.clone();
                    window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                        if phase == DispatchPhase::Bubble {
                            moved.update(cx, |this, cx| this.drag_to(event.position.y, cx)).ok();
                        }
                    });
                    let released = this.clone();
                    window.on_mouse_event(move |_: &MouseUpEvent, phase, window, cx| {
                        if phase == DispatchPhase::Bubble {
                            released.update(cx, |this, cx| this.let_go(window, cx)).ok();
                        }
                    });
                },
            )
            .absolute()
            .size_full()
        });

        div()
            .id("sheet-scrim")
            .absolute()
            .inset_0()
            .occlude()
            .bg(scrim)
            .text_color(cx.theme().foreground)
            .on_mouse_down(MouseButton::Left, |_, window, cx| close_bottom(window, cx))
            .child(surface)
            .children(dragging)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sheet_taller_than_its_own_is_given_the_height_and_one_shorter_goes_below() {
        assert_eq!(placement(px(300.), px(300.)), (None, px(0.)));
        assert_eq!(placement(px(500.), px(300.)), (Some(px(500.)), px(0.)));
        assert_eq!(placement(px(200.), px(300.)), (None, px(100.)));
    }

    #[test]
    fn motion_eases_out_and_a_zero_duration_is_already_there() {
        assert_eq!(Motion::new(None, Duration::ZERO).progress(), 1.);
    }
}
