//! Filter chips: which of several things a view is narrowed to, any number
//! of them at once, "All" for none. The categories a chart is of, the
//! labels a list shows.
//!
//! **Pointer**: small outlined toggles, the ones on in the selected fill.
//! **Touch**: Material 3 filter chips, 32 dp, a check on the ones on.
//!
//! Controlled: the owner keeps which are on and is told which was pressed.
//! A chip may carry a dot of its thing's colour, faded while others are
//! chosen and it is not.

use std::rc::Rc;

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::{ActiveTheme, Icon, IconName, Sizable, h_flex};
use gpui_kit::prelude::*;
use gpui_kit::{App, ElementId, FontWeight, Hsla, RenderOnce, SharedString, Window, div, rems};

use crate::env::{Env, InputMode};
use crate::palette::ActivePalette;
use crate::scale::ActiveScale;

type OnToggle = Rc<dyn Fn(Option<&SharedString>, &mut Window, &mut App)>;

struct Chip {
    key: SharedString,
    label: SharedString,
    on: bool,
    dot: Option<Hsla>,
}

/// A row of filter chips, wrapping onto more rows as needed.
#[derive(IntoElement)]
pub struct Chips {
    id: ElementId,
    all: Option<SharedString>,
    chips: Vec<Chip>,
    on_toggle: Option<OnToggle>,
}

impl Chips {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Chips { id: id.into(), all: None, chips: Vec::new(), on_toggle: None }
    }

    /// A first chip that stands for none chosen, which is on while none is:
    /// "All". Pressing it is told as `None`.
    pub fn all(mut self, label: impl Into<SharedString>) -> Self {
        self.all = Some(label.into());
        self
    }

    pub fn chip(mut self, key: impl Into<SharedString>, label: impl Into<SharedString>, on: bool) -> Self {
        self.chips.push(Chip { key: key.into(), label: label.into(), on, dot: None });
        self
    }

    /// The chip just added carries a dot of `color`.
    pub fn dot(mut self, color: Hsla) -> Self {
        if let Some(chip) = self.chips.last_mut() {
            chip.dot = Some(color);
        }
        self
    }

    /// A chip was pressed: its key, or `None` for "All".
    pub fn on_toggle(mut self, f: impl Fn(Option<&SharedString>, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Chips {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let touch = env.input() == InputMode::Touch;
        let scale = cx.scale();
        let theme = cx.theme();
        let palette = cx.palette();
        let none_on = self.chips.iter().all(|chip| !chip.on);
        let (on_bg, on_border, on_fg, off_fg, border) = if touch {
            let on = palette.secondary_container;
            (on, on, palette.on_secondary_container, palette.on_surface_variant, palette.outline_variant)
        } else {
            (theme.primary.opacity(0.12), theme.primary, theme.foreground, theme.foreground, theme.border)
        };
        let chip = |id: ElementId, label: SharedString, on: bool, dot: Option<Hsla>, key: Option<SharedString>| {
            let on_toggle = self.on_toggle.clone();
            h_flex()
                .id(id)
                .flex_none()
                .gap_1p5()
                .h(if touch { rems(2.) } else { scale.control })
                .px(if touch { rems(1.) } else { rems(0.625) })
                .when(touch, |chip| chip.rounded(rems(0.5)))
                .when(!touch, |chip| chip.rounded(theme.radius))
                .border_1()
                .cursor_pointer()
                .text_size(scale.small)
                .font_weight(FontWeight::MEDIUM)
                .when(on, |chip| chip.bg(on_bg).border_color(on_border).text_color(on_fg))
                .when(!on, |chip| chip.border_color(border).text_color(off_fg))
                .when(!touch && !on, |chip| chip.hover(|style| style.bg(theme.secondary_hover)))
                .when(touch && on, |chip| chip.child(Icon::new(IconName::Check).xsmall()))
                .children(dot.map(|color| {
                    let strength = if on || none_on { 1. } else { 0.35 };
                    div().flex_none().size(rems(0.625)).rounded_full().bg(color.opacity(strength))
                }))
                .child(label)
                .on_click(move |_, window, cx| {
                    if let Some(on_toggle) = &on_toggle {
                        on_toggle(key.as_ref(), window, cx);
                    }
                })
                .test_support()
        };
        let all = self.all.clone().map(|label| chip((self.id.clone(), SharedString::from("all")).into(), label, none_on, None, None));
        let chips: Vec<_> = self
            .chips
            .iter()
            .map(|c| chip((self.id.clone(), c.key.clone()).into(), c.label.clone(), c.on, c.dot, Some(c.key.clone())))
            .collect();
        h_flex().flex_wrap().gap(scale.space_s).children(all).children(chips)
    }
}
