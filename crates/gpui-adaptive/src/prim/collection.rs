//! Building blocks for application-owned collections and navigation.

use std::{cell::Cell, rc::Rc};

use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, App, Bounds, DispatchPhase, ElementId, ListAlignment, ListState, MouseButton,
    Pixels, Rems, ScrollWheelEvent, TouchPhase, Window, canvas, div, list, px,
};

use crate::{ActiveColors as _, ItemActions};

/// Retained scroll position and measurements for a lazy vertical list.
#[derive(Clone)]
pub struct VerticalListState(ListState);

impl Default for VerticalListState {
    fn default() -> Self {
        let overdraw = if cfg!(target_os = "android") {
            120.
        } else {
            320.
        };
        Self(ListState::new(0, ListAlignment::Top, px(overdraw)))
    }
}

impl VerticalListState {
    /// Rebuild measurements while retaining the logical scroll position.
    pub fn reset(&self, count: usize, row_height: Option<Rems>, rem: Pixels) {
        let top = self.0.logical_scroll_top();
        match row_height {
            Some(height) => self
                .0
                .reset_with_uniform_height(count, height.to_pixels(rem)),
            None => self.0.reset(count),
        }
        self.0.scroll_to(top);
    }

    /// Fills its parent, building only rows near the visible region.
    pub fn view(
        &self,
        render: impl Fn(usize, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> AnyElement {
        list(self.0.clone(), render).size_full().into_any_element()
    }
}

/// A selectable surface with arbitrary content, input feedback and item commands.
#[derive(IntoElement)]
pub struct SelectableSurface {
    id: ElementId,
    content: AnyElement,
    selected: bool,
    muted: bool,
    inline: bool,
    activate: crate::command::Handler,
    actions: ItemActions,
}

impl SelectableSurface {
    pub fn new(
        id: impl Into<ElementId>,
        content: impl IntoElement,
        activate: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            content: content.into_any_element(),
            selected: false,
            muted: false,
            inline: false,
            activate: Rc::new(activate),
            actions: ItemActions::new(),
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }

    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }

    pub fn actions(mut self, actions: ItemActions) -> Self {
        self.actions = actions;
        self
    }
}

impl gpui_kit::RenderOnce for SelectableSurface {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let focus = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            })
            .read(cx)
            .clone();
        let pressed = self.activate.clone();
        let focus_color = colors.focus;
        let surface = super::pressable(self.id, cx)
            .track_focus(&focus)
            .border_1()
            .border_color(gpui_kit::transparent_black())
            .focus_visible(move |style| style.border_color(focus_color))
            .when(self.inline, |item| item.flex_none().rounded_full())
            .when(!self.inline, |item| item.w_full().rounded_md())
            .when(self.selected, |item| item.bg(colors.nav_selected))
            .when(self.muted, |item| item.opacity(0.6))
            .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
            .on_click(move |_, window, cx| pressed(window, cx))
            .on_key_down(move |event, window, cx| {
                if !event.is_held && matches!(event.keystroke.key.as_str(), "enter" | "space") {
                    window.prevent_default();
                    cx.stop_propagation();
                    (self.activate)(window, cx);
                }
            })
            .child(self.content);
        let actions = if self.inline {
            self.actions.inline()
        } else {
            self.actions
        };
        actions.wrap(surface, cx)
    }
}

/// Retains the measured bounds of a viewport across frames.
#[derive(Clone, Default)]
pub struct ViewportBounds {
    area: Rc<Cell<Bounds<Pixels>>>,
    pan_inside: Rc<Cell<bool>>,
}

impl ViewportBounds {
    pub fn bounds(&self) -> Bounds<Pixels> {
        self.area.get()
    }

    /// An invisible layer measuring a relative parent.
    pub fn measure(&self) -> AnyElement {
        let bounds = self.area.clone();
        canvas(move |area, _, _| bounds.set(area), |_, _, _, _| ())
            .absolute()
            .size_full()
            .into_any_element()
    }
}

/// A viewport-width layer shifted by a fraction of its width.
pub fn translated_layer(
    id: impl Into<ElementId>,
    bounds: &ViewportBounds,
    offset: f32,
    content: impl IntoElement,
) -> AnyElement {
    div()
        .id(id.into())
        .absolute()
        .top_0()
        .bottom_0()
        .left(bounds.bounds().size.width * offset)
        .w_full()
        .child(content)
        .into_any_element()
}

/// A pan sample in viewport pixels. The application decides what a pan means.
#[derive(Clone, Copy, Debug)]
pub struct PanSample {
    pub dx: f32,
    pub dy: f32,
    pub phase: TouchPhase,
}

/// A capture layer forwarding touch pans that start inside the measured viewport.
/// Return true to take the gesture away from underlying scroll areas.
pub fn touch_pan(
    bounds: ViewportBounds,
    on_pan: impl Fn(PanSample, &mut App) -> bool + 'static,
) -> AnyElement {
    let on_pan = Rc::new(on_pan);
    let inside = bounds.pan_inside.clone();
    canvas(
        |_, _, _| (),
        move |_, _, window, _| {
            let (bounds, on_pan, inside) = (bounds.clone(), on_pan.clone(), inside.clone());
            window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
                if phase != DispatchPhase::Capture {
                    return;
                }
                if event.touch_phase == TouchPhase::Started {
                    inside.set(bounds.bounds().contains(&event.position));
                }
                if !inside.get() {
                    return;
                }
                let delta = event.delta.pixel_delta(px(1.));
                if on_pan(
                    PanSample {
                        dx: f32::from(delta.x),
                        dy: f32::from(delta.y),
                        phase: event.touch_phase,
                    },
                    cx,
                ) {
                    window.refresh();
                    cx.stop_propagation();
                }
            });
        },
    )
    .absolute()
    .size_0()
    .into_any_element()
}

/// A critically damped, interruptible return to zero for a dragged offset.
#[derive(Clone, Copy)]
pub struct OffsetSpring {
    start: std::time::Instant,
    offset: f32,
    velocity: f32,
}

impl OffsetSpring {
    pub fn new(offset: f32, velocity: f32) -> Self {
        Self {
            start: std::time::Instant::now(),
            offset,
            velocity,
        }
    }

    /// Current offset. Returns zero once the remaining movement is imperceptible.
    pub fn advance(self, window: &mut Window) -> f32 {
        let stiffness = 18.0;
        let seconds = self.start.elapsed().as_secs_f32();
        let decay = (-stiffness * seconds).exp();
        let slope = self.velocity + stiffness * self.offset;
        let offset = (self.offset + slope * seconds) * decay;
        let velocity = (self.velocity - stiffness * slope * seconds) * decay;
        if offset.abs() < 0.001 && velocity.abs() < 0.015 {
            return 0.0;
        }
        window.request_animation_frame();
        offset
    }
}

/// A horizontal viewport with retained scroll position and item reveal.
#[derive(Default)]
pub struct HorizontalScroll(gpui_kit::ScrollHandle);

impl HorizontalScroll {
    /// The distance scrolled from the beginning of the strip.
    pub fn scrolled(&self) -> Pixels {
        -self.0.offset().x
    }

    /// Reveal the child at this index on the next layout.
    pub fn reveal(&self, index: usize) {
        self.0.scroll_to_item(index);
    }

    /// A single line of items, clipped and scrollable when it is too wide.
    pub fn view(
        &self,
        id: impl Into<ElementId>,
        content: impl IntoIterator<Item = AnyElement>,
    ) -> AnyElement {
        use super::Scrolls as _;
        super::row()
            .id(id.into())
            .flex_1()
            .min_w_0()
            .gap_1()
            .scrolls_x()
            .track_scroll(&self.0)
            .children(content)
            .into_any_element()
    }
}

/// A mark indicating unread activity without a count.
pub fn unread_mark(cx: &App) -> AnyElement {
    div()
        .flex_none()
        .size(gpui_kit::rems(0.5))
        .rounded_full()
        .bg(cx.colors().accent)
        .into_any_element()
}
