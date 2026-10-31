//! A sentence the page says in its own voice: what a list is for, what a
//! form is doing ("Checking…"), what went wrong, that a section has nothing
//! in it yet.
//!
//! Both skins write it as secondary text in the skin's small size; a tone
//! colours it. It is not a [`crate::Banner`], which is a state of the whole
//! page, nor an [`crate::Empty`], which stands for a page with nothing on it.

use gpui_kit::component::{ActiveTheme, Icon, IconName, Sizable, h_flex};
use gpui_kit::prelude::*;
use gpui_kit::{App, RenderOnce, SharedString, Window, div};

use crate::banner::Tone;
use crate::env::{Env, InputMode};
use crate::palette::ActivePalette;
use crate::scale::ActiveScale;

#[derive(IntoElement)]
pub struct Note {
    text: SharedString,
    tone: Option<Tone>,
    truncate: bool,
}

impl Note {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Note { text: text.into(), tone: None, truncate: false }
    }

    /// What the note is about: `Danger` for an error, with its icon.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }

    /// One line, cut with an ellipsis: an address, a path.
    pub fn truncate(mut self) -> Self {
        self.truncate = true;
        self
    }
}

impl RenderOnce for Note {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let muted = match env.input() {
            InputMode::Pointer => cx.theme().muted_foreground,
            InputMode::Touch => cx.palette().on_surface_variant,
        };
        let color = self.tone.map_or(muted, |tone| tone.color(cx));
        let icon = match self.tone {
            Some(Tone::Danger) => Some(IconName::CircleX),
            Some(Tone::Warning) => Some(IconName::TriangleAlert),
            _ => None,
        };
        let text = div().min_w_0().when(self.truncate, |text| text.truncate()).child(self.text);
        h_flex()
            .w_full()
            .min_w_0()
            .items_start()
            .gap_1p5()
            .text_size(scale.small)
            .text_color(color)
            .children(icon.map(|icon| div().pt_0p5().child(Icon::new(icon).xsmall())))
            .child(text.flex_1())
    }
}
