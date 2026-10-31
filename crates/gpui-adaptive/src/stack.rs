//! The parts of a page one under another: a project's agents, its board and
//! its conversations; a settings page's settings.
//!
//! Both skins centre the parts in a column at most [`Stack::width`] wide
//! (56 rem by default), with the page's margins and the scale's `space_xl`
//! between the parts.
//! A part that is wider than the column (a board) scrolls in it.

use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, RenderOnce, Rems, Window, rems};

use crate::env::Env;
use crate::scale::ActiveScale;

#[derive(IntoElement)]
pub struct Stack {
    width: Rems,
    children: Vec<AnyElement>,
}

impl Default for Stack {
    fn default() -> Self {
        Stack { width: rems(56.), children: Vec::new() }
    }
}

impl Stack {
    pub fn new() -> Self {
        Self::default()
    }

    /// The widest the column is.
    pub fn width(mut self, width: Rems) -> Self {
        self.width = width;
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children.extend(children.into_iter().map(IntoElement::into_any_element));
        self
    }
}

impl RenderOnce for Stack {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let column = v_flex().w_full().min_w_0().max_w(self.width).gap(scale.space_xl).children(self.children);
        h_flex().w_full().justify_center().px_4().py_4().when(env.is_compact(), |page| page.py_2()).child(column)
    }
}
