//! Something that outlives the page it started on: a call (matterfast's
//! call dock), a sync in progress, music playing.
//!
//! **Medium and up**: pinned at the bottom of the navigation column, under
//! the sidebar's destinations or the list, so it does not move as pages
//! change.
//! **Compact**: a strip across the bottom of the page, above the navigation
//! bar, in a container colour, a finger's height.
//!
//! The dock draws its content as the owner gives it; [`Dock::place`] tells
//! the owner where to put it for the window's width.

use gpui_kit::component::ActiveTheme;
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, ElementId, RenderOnce, Window, div};

use crate::env::{Env, WidthClass};
use crate::palette::ActivePalette;
use crate::scale::ActiveScale;

/// Where a dock goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockPlace {
    /// At the foot of the navigation column.
    Column,
    /// Across the bottom of the page.
    Page,
}

#[derive(IntoElement)]
pub struct Dock {
    id: ElementId,
    content: Option<AnyElement>,
}

impl Dock {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Dock { id: id.into(), content: None }
    }

    pub fn child(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    /// Where the dock belongs in `window` now.
    pub fn place(window: &Window) -> DockPlace {
        if WidthClass::of(window) == WidthClass::Compact { DockPlace::Page } else { DockPlace::Column }
    }
}

impl RenderOnce for Dock {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let theme = cx.theme();
        let palette = cx.palette();
        let page = Dock::place(window) == DockPlace::Page;
        div()
            .id(self.id)
            .flex_none()
            .w_full()
            .min_h(if env.is_touch() { scale.target } else { scale.control })
            .px_3()
            .py_2()
            .border_t_1()
            .when(page, |dock| dock.bg(palette.surface_high).border_color(palette.outline_variant))
            .when(!page, |dock| dock.bg(theme.sidebar).border_color(theme.sidebar_border))
            .children(self.content)
    }
}
