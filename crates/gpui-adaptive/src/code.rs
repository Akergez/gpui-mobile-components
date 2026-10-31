//! Text a machine wrote, shown as it was written: a program's output (an
//! agent's last lines of stderr, what a sign-in on a server prints).
//!
//! Both skins draw a monospaced block on the muted surface, in the small
//! size. A block that may be long has a height it scrolls in, locked to its
//! axis for a finger.

use gpui_kit::component::{ActiveTheme, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{App, ElementId, Rems, RenderOnce, SharedString, Window, div};

use crate::env::Env;
use crate::scale::ActiveScale;
use crate::touch;

/// A program's output.
#[derive(IntoElement)]
pub struct Code {
    id: ElementId,
    text: SharedString,
    max_height: Option<Rems>,
}

impl Code {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Code { id: id.into(), text: text.into(), max_height: None }
    }

    /// The most the block grows to; past it, it scrolls.
    pub fn max_height(mut self, max_height: Rems) -> Self {
        self.max_height = Some(max_height);
        self
    }
}

impl RenderOnce for Code {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let block = block(cx).child(self.text);
        match self.max_height {
            Some(max) => div().w_full().max_h(max).child(touch::scroll_y(self.id, env, block)).into_any_element(),
            None => div().id(self.id).w_full().child(block).into_any_element(),
        }
    }
}

/// The monospaced block both draw on.
fn block(cx: &App) -> gpui_kit::Div {
    let theme = cx.theme();
    v_flex()
        .w_full()
        .p_2()
        .rounded(cx.scale().radius_control)
        .bg(theme.muted)
        .font_family(theme.mono_font_family.clone())
        .text_size(cx.scale().small)
}
