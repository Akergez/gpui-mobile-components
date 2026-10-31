//! On or off.
//!
//! **Pointer**: a checkbox in a form, where the change is committed with the
//! form; a switch for a setting that acts at once ([`Toggle::immediate`]).
//! **Touch**: always a switch, at the end of a row a finger can press
//! anywhere on.
//!
//! The value is controlled: pass the current value, receive the requested
//! one in `on_change`, store it, render again.

use std::rc::Rc;

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{ActiveTheme, Disableable as _, Sizable, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{App, ElementId, RenderOnce, SharedString, Window, div};

use crate::env::{Env, InputMode};
use crate::scale::ActiveScale;

type OnChange = Rc<dyn Fn(bool, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Toggle {
    id: ElementId,
    label: SharedString,
    description: Option<SharedString>,
    on: bool,
    immediate: bool,
    disabled: bool,
    on_change: Option<OnChange>,
}

impl Toggle {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, on: bool) -> Self {
        Toggle {
            id: id.into(),
            label: label.into(),
            description: None,
            on,
            immediate: false,
            disabled: false,
            on_change: None,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The change takes effect at once (a setting), rather than with a form.
    pub fn immediate(mut self, immediate: bool) -> Self {
        self.immediate = immediate;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, on_change: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(on_change));
        self
    }
}

impl RenderOnce for Toggle {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let muted = cx.theme().muted_foreground;
        let on_change = self.on_change.clone();
        let change = move |value: &bool, window: &mut Window, cx: &mut App| {
            if let Some(on_change) = &on_change {
                on_change(*value, window, cx);
            }
        };
        let text = v_flex()
            .flex_1()
            .min_w_0()
            .child(div().text_size(scale.body).child(self.label.clone()))
            .children(self.description.clone().map(|d| div().text_size(scale.small).text_color(muted).child(d)));
        match env.input() {
            // The description under the label, where the box's own label
            // ends: what the option means is read with it.
            InputMode::Pointer if !self.immediate => v_flex()
                .gap_0p5()
                .child(
                    Checkbox::new(self.id)
                        .checked(self.on)
                        .label(self.label)
                        .disabled(self.disabled)
                        .on_click(change),
                )
                .children(self.description.map(|d| {
                    div().pl(gpui_kit::rems(1.5)).text_size(scale.small).text_color(muted).child(d)
                }))
                .into_any_element(),
            InputMode::Pointer => h_flex()
                .w_full()
                .gap_3()
                .child(text)
                .child(Switch::new(self.id).checked(self.on).disabled(self.disabled).on_click(change))
                .into_any_element(),
            InputMode::Touch => {
                let (on, on_change, disabled) = (self.on, self.on_change.clone(), self.disabled);
                h_flex()
                    .id((self.id.clone(), "row"))
                    .w_full()
                    .min_h(scale.row)
                    .gap_4()
                    .cursor_pointer()
                    .child(text)
                    .child(Switch::new(self.id).checked(on).disabled(disabled).large())
                    .when(!disabled, |row| {
                        row.on_click(move |_, window, cx| {
                            if let Some(on_change) = &on_change {
                                on_change(!on, window, cx);
                            }
                        })
                    })
                    .test_support()
                    .into_any_element()
            }
        }
    }
}
