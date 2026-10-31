//! A settings page: [`Setting`]s, each what it is beside its control, and
//! [`Swatches`], a colour picked from a few.
//!
//! **Pointer**: a setting's title and its line of explanation on the left,
//! its control on the right; where they do not fit side by side the control
//! goes under them.
//! **Touch**: Material's settings item: the title in the body size, the
//! explanation under it, the control at the end or under, a finger's height
//! at least. A swatch is 40 dp across.
//!
//! Every setting takes effect as it is chosen: a settings page has no Save.

use std::rc::Rc;

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{ActiveTheme, Selectable as _, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, ElementId, Hsla, RenderOnce, SharedString, Window, div, rems};

use crate::env::{Env, InputMode};
use crate::palette::ActivePalette;
use crate::scale::ActiveScale;

/// One setting.
#[derive(IntoElement)]
pub struct Setting {
    title: SharedString,
    detail: Option<SharedString>,
    control: Option<AnyElement>,
    below: Option<AnyElement>,
}

impl Setting {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Setting { title: title.into(), detail: None, control: None, below: None }
    }

    /// What the setting does, in a line.
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// What sets it: a [`crate::Segments`], a [`crate::Choice`], a toggle, a
    /// command's button. Beside the title while there is room.
    pub fn control(mut self, control: impl IntoElement) -> Self {
        self.control = Some(control.into_any_element());
        self
    }

    /// What sets it when it needs the row's width: [`Swatches`], a browser.
    pub fn below(mut self, below: impl IntoElement) -> Self {
        self.below = Some(below.into_any_element());
        self
    }
}

impl RenderOnce for Setting {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let muted = match env.input() {
            InputMode::Pointer => cx.theme().muted_foreground,
            InputMode::Touch => cx.palette().on_surface_variant,
        };
        let text = v_flex()
            .flex_1()
            .min_w(rems(14.))
            .child(div().text_size(scale.body).child(self.title))
            .children(self.detail.map(|detail| div().text_size(scale.small).text_color(muted).child(detail)));
        v_flex()
            .w_full()
            .gap_2()
            .when(env.is_touch(), |setting| setting.min_h(scale.row_two_line).justify_center())
            .child(h_flex().flex_wrap().gap_x_4().gap_y_2().child(text).children(self.control))
            .children(self.below)
    }
}

type OnPick = Rc<dyn Fn(Option<&SharedString>, &mut Window, &mut App)>;

/// A colour picked from a few, or none of them (the system's).
/// Controlled: the owner keeps which is picked.
#[derive(IntoElement)]
pub struct Swatches {
    id: ElementId,
    selected: Option<SharedString>,
    unset: Option<SharedString>,
    swatches: Vec<(SharedString, SharedString, Hsla)>,
    on_pick: Option<OnPick>,
}

impl Swatches {
    /// `selected` is the key of the swatch picked; none is the unset one.
    pub fn new(id: impl Into<ElementId>, selected: Option<impl Into<SharedString>>) -> Self {
        Swatches { id: id.into(), selected: selected.map(Into::into), unset: None, swatches: Vec::new(), on_pick: None }
    }

    /// A first choice that picks no colour: "System".
    pub fn unset(mut self, label: impl Into<SharedString>) -> Self {
        self.unset = Some(label.into());
        self
    }

    /// A colour, named for its tooltip and for a screen reader.
    pub fn swatch(mut self, key: impl Into<SharedString>, name: impl Into<SharedString>, color: Hsla) -> Self {
        self.swatches.push((key.into(), name.into(), color));
        self
    }

    pub fn on_pick(mut self, on_pick: impl Fn(Option<&SharedString>, &mut Window, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(on_pick));
        self
    }
}

impl RenderOnce for Swatches {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let (ring, gap) = (cx.theme().foreground, cx.theme().background);
        let size = if env.is_touch() { rems(2.5) } else { rems(1.5) };
        let unset = self.unset.map(|label| {
            let on_pick = self.on_pick.clone();
            let button = Button::new((self.id.clone(), "unset")).label(label).outline().selected(self.selected.is_none());
            let button = if env.is_touch() { button } else { button.small() };
            button.on_click(move |_, window, cx| {
                if let Some(on_pick) = &on_pick {
                    on_pick(None, window, cx);
                }
            })
        });
        let swatches = self.swatches.into_iter().map(|(key, name, color)| {
            let on = self.selected.as_ref() == Some(&key);
            let on_pick = self.on_pick.clone();
            div()
                .id(ElementId::Name(key.clone()))
                .test_support()
                .flex_none()
                .size(size)
                .rounded_full()
                .cursor_pointer()
                .when(on, |swatch| swatch.border_2().border_color(ring))
                .child(div().size_full().rounded_full().border_2().border_color(gap).bg(color))
                .tooltip(move |window, cx| Tooltip::new(name.clone()).build(window, cx))
                .on_click(move |_, window, cx| {
                    if let Some(on_pick) = &on_pick {
                        on_pick(Some(&key), window, cx);
                    }
                })
        });
        h_flex().id(self.id).flex_wrap().items_center().gap_2().children(unset).children(swatches)
    }
}
