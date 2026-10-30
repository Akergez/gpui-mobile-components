//! Two to four modes, exactly one of which is on: Expenses or Income, Day or
//! Week.
//!
//! **Pointer**: gpui-kit's segmented look, a muted track with the chosen
//! segment raised on it.
//! **Touch**: Material 3's connected button group, 40 dp tall, the chosen
//! segment filled with the primary and rounded fully.
//!
//! Controlled: the owner keeps which segment is on.

use std::rc::Rc;

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::{ActiveTheme, Icon, Sizable, h_flex};
use gpui_kit::prelude::*;
use gpui_kit::{App, ElementId, FontWeight, RenderOnce, SharedString, Window, div, rems};

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
}

impl Segments {
    /// `selected` is the key of the segment on.
    pub fn new(id: impl Into<ElementId>, selected: impl Into<SharedString>) -> Self {
        Segments { id: id.into(), segments: Vec::new(), selected: selected.into(), on_change: None }
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
        let count = self.segments.len();
        let touch = env.input() == InputMode::Touch;
        let segments = self.segments.into_iter().enumerate().map(|(ix, (key, label, icon))| {
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
                let (first, last) = (ix == 0, ix + 1 == count);
                let big = rems(1.25);
                let small = rems(0.5);
                segment
                    .flex_1()
                    .h(scale.control)
                    .px_4()
                    .bg(if on { palette.primary } else { palette.surface_high })
                    .text_color(if on { palette.on_primary } else { palette.on_surface })
                    .when(on, |s| s.rounded(big))
                    .when(!on, |s| {
                        s.rounded_tl(if first { big } else { small })
                            .rounded_bl(if first { big } else { small })
                            .rounded_tr(if last { big } else { small })
                            .rounded_br(if last { big } else { small })
                    })
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
            h_flex().id(self.id).w_full().gap(rems(0.125)).children(segments).into_any_element()
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
