//! Two to four modes, exactly one of which is on: Expenses or Income, Day or
//! Week.
//!
//! **Pointer**: gpui-kit's segmented look, a muted track with the chosen
//! segment raised on it.
//! **Touch**: the same shape at a finger's size: a pill-shaped tonal track
//! as tall as a control, the chosen segment a pill of the secondary
//! container inside it, concentric with the track. The primary is left to
//! the page's one primary action.
//!
//! Controlled: the owner keeps which segment is on.

use std::rc::Rc;

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::{ActiveTheme, Icon, Sizable, h_flex};
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, ElementId, FontWeight, RenderOnce, SharedString, Window, div, rems};

use crate::env::{Env, InputMode};
use crate::palette::ActivePalette;
use crate::scale::ActiveScale;

type OnChange = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Segments {
    id: ElementId,
    segments: Vec<(SharedString, SharedString, Option<Icon>)>,
    selected: SharedString,
    on_change: Option<OnChange>,
    fill: bool,
}

impl Segments {
    /// `selected` is the key of the segment on.
    pub fn new(id: impl Into<ElementId>, selected: impl Into<SharedString>) -> Self {
        Segments {
            id: id.into(),
            segments: Vec::new(),
            selected: selected.into(),
            on_change: None,
            fill: true,
        }
    }

    /// Whether the group takes the width it is given and divides it among
    /// its segments (the default: a switch that spans its page or card).
    /// With `false` the group hugs its segments — for a row beside something
    /// else, where a full width would leave the rest of the row nothing.
    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    pub fn segment(mut self, key: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        self.segments.push((key.into(), label.into(), None));
        self
    }

    pub fn segment_with_icon(
        mut self,
        key: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: impl Into<Icon>,
    ) -> Self {
        self.segments.push((key.into(), label.into(), Some(icon.into())));
        self
    }

    pub fn on_change(mut self, on_change: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(on_change));
        self
    }
}

impl RenderOnce for Segments {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let palette = cx.palette();
        let theme = cx.theme();
        let touch = env.input() == InputMode::Touch;
        let fill = self.fill;
        // A switch across its page stays put while the page scrolls: a
        // cached part, its height a control's and its width the page's.
        if touch && fill {
            let look = TrackLook {
                segments: self.segments.iter().map(|(key, label, _)| (key.clone(), label.clone())).collect(),
                selected: self.selected.clone(),
                palette,
                scale: scale.clone(),
            };
            let handlers = TrackHandlers {
                icons: self.segments.iter().map(|(_, _, icon)| icon.clone()).collect(),
                on_change: self.on_change.clone(),
            };
            let style = gpui_kit::StyleRefinement::default().w_full().h(scale.control);
            return crate::cache::part((self.id.clone(), "track"), look, handlers, draw_track, style, window, cx);
        }
        let segments = self.segments.into_iter().map(|(key, label, icon)| {
            let on = key == self.selected;
            let on_change = self.on_change.clone();
            let chosen = key.clone();
            let segment = h_flex()
                .id(ElementId::Name(key))
                .justify_center()
                .gap_1p5()
                .cursor_pointer()
                .text_size(scale.small)
                .font_weight(FontWeight::MEDIUM)
                .children(icon.map(|icon| icon.small()))
                .child(label)
                .on_click(move |_, window, cx| {
                    if !on && let Some(on_change) = &on_change {
                        on_change(&chosen, window, cx);
                    }
                });
            if touch {
                segment
                    .when(fill, |segment| segment.flex_1())
                    .h_full()
                    .px_4()
                    .rounded_full()
                    .when(on, |s| s.bg(palette.secondary_container).text_color(palette.on_secondary_container))
                    .when(!on, |s| s.text_color(palette.on_surface_variant))
                    .test_support()
                    .into_any_element()
            } else {
                segment
                    .h(rems(1.5))
                    .px_3()
                    .rounded(theme.radius)
                    .when(on, |s| s.bg(theme.background).text_color(theme.foreground).shadow_xs())
                    .when(!on, |s| s.text_color(theme.muted_foreground))
                    .test_support()
                    .into_any_element()
            }
        });
        if touch {
            let track = h_flex()
                .id(self.id)
                .h(scale.control)
                .p(rems(0.25))
                .gap(rems(0.25))
                .rounded_full()
                .bg(palette.surface_high);
            let track = if fill { track.w_full() } else { track };
            track.children(segments).into_any_element()
        } else {
            div()
                .id(self.id)
                .flex()
                .flex_row()
                .p_0p5()
                .gap_0p5()
                .rounded(theme.radius * 1.3)
                .bg(theme.muted)
                .children(segments)
                .into_any_element()
        }
    }
}

/// What a touch switch across its page shows; see [`crate::cache`].
#[derive(PartialEq)]
struct TrackLook {
    segments: Vec<(SharedString, SharedString)>,
    selected: SharedString,
    palette: crate::palette::Palette,
    scale: crate::scale::Scale,
}

struct TrackHandlers {
    /// Told apart by their segment's key, not compared.
    icons: Vec<Option<Icon>>,
    on_change: Option<OnChange>,
}

fn draw_track(
    look: &TrackLook,
    handlers: &TrackHandlers,
    _: &mut Window,
    cx: &mut gpui_kit::Context<crate::cache::Part<TrackLook, TrackHandlers>>,
) -> AnyElement {
    let (palette, scale) = (look.palette, &look.scale);
    let track = cx.entity().downgrade();
    h_flex()
        .size_full()
        .p(rems(0.25))
        .gap(rems(0.25))
        .rounded_full()
        .bg(palette.surface_high)
        .children(look.segments.iter().zip(&handlers.icons).map(|((key, label), icon)| {
            let on = *key == look.selected;
            let (track, chosen) = (track.clone(), key.clone());
            h_flex()
                .id(ElementId::Name(key.clone()))
                .flex_1()
                .h_full()
                .px_4()
                .justify_center()
                .gap_1p5()
                .cursor_pointer()
                .rounded_full()
                .text_size(scale.small)
                .font_weight(FontWeight::MEDIUM)
                .when(on, |s| s.bg(palette.secondary_container).text_color(palette.on_secondary_container))
                .when(!on, |s| s.text_color(palette.on_surface_variant))
                .children(icon.clone().map(|icon| icon.small()))
                .child(label.clone())
                // The handler of the frame of the press.
                .on_click(move |_, window, cx| {
                    if on {
                        return;
                    }
                    let on_change = track
                        .upgrade()
                        .and_then(|track| track.read(cx).handlers().and_then(|h| h.on_change.clone()));
                    if let Some(on_change) = on_change {
                        on_change(&chosen, window, cx);
                    }
                })
                .test_support()
                .into_any_element()
        }))
        .into_any_element()
}
