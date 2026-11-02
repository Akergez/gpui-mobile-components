//! Parts of a window that stay put while something beside them moves: a
//! page's header, the navigation bar. GPUI builds, lays out and paints the
//! whole window again on every frame of a scroll; a part drawn here is a
//! cached view instead, taken from the previous frame until what it shows
//! changes. On a phone that is a good share of each frame.
//!
//! A part is told two things every frame: its look, compared with the
//! previous frame's (a change draws it again), and its handlers, kept as
//! they come without drawing anything. What the part draws reaches its
//! handlers through the part at the moment of a press, so a part reused from
//! an earlier frame never runs a closure of that frame. The look must hold
//! everything that is seen, colours and sizes included.
//!
//! A cached view is laid out from its style alone, not measured: a part is
//! given its size.

use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, Context, ElementId, Entity, Render, StyleRefinement, Window, div};

/// How a part is drawn from its look and its handlers.
pub(crate) type Draw<V, H> = fn(&V, &H, &mut Window, &mut Context<Part<V, H>>) -> AnyElement;

pub(crate) struct Part<V: 'static, H: 'static> {
    look: Option<V>,
    handlers: Option<H>,
    draw: Draw<V, H>,
}

impl<V: 'static, H: 'static> Part<V, H> {
    /// The handlers as of the latest frame.
    pub(crate) fn handlers(&self) -> Option<&H> {
        self.handlers.as_ref()
    }
}

impl<V: 'static, H: 'static> Render for Part<V, H> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match (&self.look, &self.handlers) {
            (Some(look), Some(handlers)) => (self.draw)(look, handlers, window, cx),
            _ => div().into_any_element(),
        }
    }
}

/// The part kept under `key`, at `style`'s size: drawn again when `look`
/// differs from the previous frame's, reused otherwise.
pub(crate) fn part<V: PartialEq + 'static, H: 'static>(
    key: impl Into<ElementId>,
    look: V,
    handlers: H,
    draw: Draw<V, H>,
    style: StyleRefinement,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let entity: Entity<Part<V, H>> =
        window.use_keyed_state(key.into(), cx, |_, _| Part { look: None, handlers: None, draw });
    entity.update(cx, |part, cx| {
        part.handlers = Some(handlers);
        if part.look.as_ref() != Some(&look) {
            part.look = Some(look);
            cx.notify();
        }
    });
    entity.cached(style).into_any_element()
}
