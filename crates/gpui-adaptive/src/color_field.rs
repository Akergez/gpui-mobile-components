//! A colour the person picks: a category's, a label's. A few featured ones a
//! press away, any other from the palettes or typed as hex.
//!
//! Both skins open gpui-kit's colour picker from a swatch button; on a
//! finger the button is a finger's height. Whether a colour is chosen at all
//! is the owner's: an unset field (`None`) is the thing's own default, which
//! the owner says beside it.

use gpui_kit::component::color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState};
use gpui_kit::component::{Colorize as _, Sizable, Size};
use gpui_kit::prelude::*;
use gpui_kit::{App, Context, ElementId, Entity, EventEmitter, Hsla, RenderOnce, SharedString, Subscription, Window, div};

use crate::env::{Env, InputMode};
use crate::field::label;

/// A colour chosen, or the choice undone.
#[derive(Clone, Debug, PartialEq)]
pub enum ColorFieldEvent {
    Change(Option<Hsla>),
}

/// The colour a [`ColorField`] holds.
pub struct ColorFieldState {
    picker: Entity<ColorPickerState>,
    _picked: Subscription,
}

impl EventEmitter<ColorFieldEvent> for ColorFieldState {}

impl ColorFieldState {
    pub fn new(color: Option<Hsla>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let picker = cx.new(|cx| {
            let state = ColorPickerState::new(window, cx);
            match color {
                Some(color) => state.default_value(color),
                None => state,
            }
        });
        let picked = cx.subscribe(&picker, |_, _, event: &ColorPickerEvent, cx| {
            let ColorPickerEvent::Change(color) = event;
            cx.emit(ColorFieldEvent::Change(*color));
        });
        ColorFieldState { picker, _picked: picked }
    }
}

/// `#rrggbb` of `color` without its opacity: how a colour is stored.
pub fn to_hex(color: Hsla) -> String {
    Hsla { a: 1.0, ..color }.to_hex().to_lowercase()
}

/// The colour `#rrggbb` (or `#rrggbbaa`) names.
pub fn parse_hex(text: &str) -> Option<Hsla> {
    Hsla::parse_hex(text).ok()
}

/// A swatch that opens the picker.
#[derive(IntoElement)]
pub struct ColorField {
    id: ElementId,
    state: Entity<ColorFieldState>,
    label: Option<SharedString>,
    featured: Vec<Hsla>,
}

impl ColorField {
    pub fn new(id: impl Into<ElementId>, state: &Entity<ColorFieldState>) -> Self {
        ColorField { id: id.into(), state: state.clone(), label: None, featured: Vec::new() }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The colours offered first, a press away.
    pub fn featured(mut self, colors: impl IntoIterator<Item = Hsla>) -> Self {
        self.featured = colors.into_iter().collect();
        self
    }
}

impl RenderOnce for ColorField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let size = match env.input() {
            InputMode::Pointer => Size::Medium,
            InputMode::Touch => Size::Large,
        };
        let picker = self.state.read(cx).picker.clone();
        let label_text = self.label.clone();
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .gap_1()
            .children(self.label.map(|text| label(text, cx)))
            .child(
                ColorPicker::new(&picker)
                    .featured_colors(self.featured)
                    .with_size(size)
                    .accessibility_label(label_text.unwrap_or_else(|| "Color".into())),
            )
    }
}
