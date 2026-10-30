//! Pieces both skins' renderers share: a pressable surface with the right
//! feedback for the input, and a long press taken before anything under the
//! finger sees it.


use gpui_kit::component::ActiveTheme;
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, App, DispatchPhase, ElementId, HitboxBehavior, LongPressEvent, Stateful, Div,
    TouchPhase, canvas, div,
};

use crate::env::{Env, InputMode};
use crate::palette::ActivePalette;

/// A horizontal surface that can be pressed: rows, destinations, sheet rows.
///
/// A pointer gets the theme's hover (8%) and press colours; a finger gets
/// Material's pressed state layer (12% of the content colour) and no hover,
/// which a finger does not have.
pub(crate) fn pressable(id: impl Into<ElementId>, env: Env, cx: &App) -> Stateful<Div> {
    let row = div().id(id.into()).flex().flex_row().items_center().cursor_pointer();
    match env.input() {
        InputMode::Pointer => {
            let theme = cx.theme();
            let (hover, active) = (theme.list_hover, theme.list_active);
            row.hover(move |style| style.bg(hover)).active(move |style| style.bg(active))
        }
        InputMode::Touch => {
            let pressed = cx.palette().pressed;
            row.active(move |style| style.bg(pressed))
        }
    }
}

/// An invisible layer over its parent (which must be `relative`) that takes a
/// long press started inside it, on its way down, ahead of text selection
/// and of the parent's own click: matterfast's `post_sheet.rs`. Nothing
/// happens while a dialog is open.
pub(crate) fn long_press(on_press: crate::command::Handler) -> AnyElement {
    canvas(
        |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
        move |_, hitbox, window, _| {
            let on_press = on_press.clone();
            window.on_mouse_event(move |event: &LongPressEvent, phase, window, cx| {
                if phase != DispatchPhase::Capture
                    || event.phase != TouchPhase::Started
                    || !hitbox.bounds.contains(&event.start_position)
                {
                    return;
                }
                if gpui_kit::component::WindowExt::has_active_dialog(window, cx) {
                    return;
                }
                window.prevent_default();
                cx.stop_propagation();
                on_press(window, cx);
            });
        },
    )
    .absolute()
    .size_full()
    .into_any_element()
}

/// A vertical scroll area filling its parent: gpui-kit's scrollbar for a
/// pointer; for a finger, a scroll locked to its axis, so that a sideways
/// swipe over something inside it does not move the page too (the
/// gpui-mobile rule).
pub(crate) fn scroll_y(id: impl Into<ElementId>, env: Env, content: impl IntoElement) -> AnyElement {
    let area = div().id(id.into()).size_full();
    match env.input() {
        InputMode::Pointer => {
            use gpui_kit::component::scroll::ScrollableElement as _;
            area.overflow_y_scrollbar().child(content).into_any_element()
        }
        InputMode::Touch => {
            use gpui_kit::base::InteractiveElementExt as _;
            area.overflow_y_scroll().lock_scroll_axis().child(content).into_any_element()
        }
    }
}
