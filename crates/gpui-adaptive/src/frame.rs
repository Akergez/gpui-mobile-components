//! The window's frame: its title bar (or the room the status bar takes on a
//! phone), the content, and the layers drawn over everything — the touch
//! sheets and the snackbar.
//!
//! Put one [`AppFrame`] at the top of each window, under gpui-kit's `Root`.
//! The bar across the top is `gpui_android_glue::chrome::title_frame`: the
//! toolkit's title bar on a desktop, a plain strip under the status bar on a
//! phone with a pointer. In the touch skin on a phone there is no strip at
//! all: the pages' own app bars sit right under the status bar.

use gpui_kit::component::{ActiveTheme, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, IntoElement, RenderOnce, SharedString, Window, div};

use crate::env::{Env, InputMode};
use crate::insets::Insets;
use crate::layers;

/// The frame of a window.
#[derive(IntoElement)]
pub struct AppFrame {
    title: SharedString,
    bar: Option<AnyElement>,
    content: Option<AnyElement>,
    edge_to_edge: bool,
    title_bar: bool,
}

impl AppFrame {
    /// How a window with a frame is opened: without the system's
    /// decorations, the frame draws the title bar.
    pub fn window_options() -> gpui_kit::WindowOptions {
        gpui_kit::component::TitleBar::window_options()
    }

    /// A frame whose title bar shows `title`.
    pub fn new(title: impl Into<SharedString>) -> Self {
        AppFrame { title: title.into(), bar: None, content: None, edge_to_edge: false, title_bar: true }
    }

    /// What the title bar holds after the title on a desktop: a search box,
    /// the window's commands. A phone has no title bar to hold it.
    pub fn bar(mut self, bar: impl IntoElement) -> Self {
        self.bar = Some(bar.into_any_element());
        self
    }

    /// The window's content: usually [`crate::Destinations`] or a
    /// [`crate::Navigator`].
    pub fn child(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    /// Whether the frame draws the window's title bar (the default). Off when
    /// the pages draw it themselves with `Screen::in_title_bar`.
    pub fn title_bar(mut self, title_bar: bool) -> Self {
        self.title_bar = title_bar;
        self
    }

    /// Says that the content keeps clear of the navigation bar and the
    /// keyboard itself, as [`crate::Destinations`] does. Without it the frame
    /// keeps the room at the bottom for them.
    pub fn edge_to_edge(mut self, edge_to_edge: bool) -> Self {
        self.edge_to_edge = edge_to_edge;
        self
    }
}

impl RenderOnce for AppFrame {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let insets = Insets::of(window);
        let theme = cx.theme();
        let (background, foreground, muted) = (theme.background, theme.foreground, theme.muted_foreground);
        let top: AnyElement = if env.input() == InputMode::Touch && gpui_kit::is_mobile() {
            div().flex_none().h(insets.top).into_any_element()
        } else if !self.title_bar {
            div().into_any_element()
        } else {
            let content = h_flex()
                .w_full()
                .gap_3()
                .child(div().text_sm().font_weight(gpui_kit::FontWeight::SEMIBOLD).child(self.title))
                .children(self.bar.map(|bar| div().flex_1().min_w_0().text_color(muted).child(bar)));
            let frame = gpui_android_glue::chrome::Frame { below_status_bar: true, ..Default::default() };
            gpui_android_glue::chrome::title_frame(content, frame, cx)
        };
        let bottom = if self.edge_to_edge { gpui_kit::Pixels::ZERO } else { insets.typing() };
        v_flex()
            .size_full()
            .relative()
            .bg(background)
            .text_color(foreground)
            .child(top)
            .child(div().flex_1().min_h_0().relative().pb(bottom).children(self.content))
            .children(layers::render(window, cx))
    }
}
