//! Work under way: [`Busy`] while it lasts (indeterminate or by fraction),
//! [`Skeleton`] rows where content is about to be.
//!
//! Both are gpui-kit's components (`Spinner`, `Progress`, `Skeleton`) sized
//! for the skin. A page that loads shows skeleton rows in the shape of what
//! comes, not a spinner in the middle of nothing.

use gpui_kit::component::progress::Progress;
use gpui_kit::component::skeleton::Skeleton as KitSkeleton;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{ActiveTheme, Sizable, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{App, ElementId, RenderOnce, SharedString, Window, div, relative, rems};

use crate::env::Env;
use crate::scale::ActiveScale;

/// Work under way.
#[derive(IntoElement)]
pub struct Busy {
    id: ElementId,
    fraction: Option<f32>,
    label: Option<SharedString>,
}

impl Busy {
    /// Work of unknown length: a spinner.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Busy { id: id.into(), fraction: None, label: None }
    }

    /// Work `fraction` (0 to 1) done: a bar.
    pub fn fraction(mut self, fraction: f32) -> Self {
        self.fraction = Some(fraction.clamp(0., 1.));
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for Busy {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let muted = cx.theme().muted_foreground;
        let label = self.label.map(|label| div().text_size(scale.small).text_color(muted).child(label));
        match self.fraction {
            Some(fraction) => v_flex()
                .w_full()
                .gap_1()
                .child(Progress::new(self.id).value(fraction * 100.))
                .children(label)
                .into_any_element(),
            None => {
                let spinner = Spinner::new();
                let spinner = if env.is_touch() { spinner.large() } else { spinner.small() };
                h_flex().id(self.id).gap_2().child(spinner).children(label).into_any_element()
            }
        }
    }
}

/// Placeholder rows in the shape of a list about to load.
#[derive(IntoElement)]
pub struct Skeleton {
    rows: usize,
    avatar: bool,
}

impl Skeleton {
    pub fn rows(rows: usize) -> Self {
        Skeleton { rows, avatar: false }
    }

    /// Rows that will have a face or an icon before them.
    pub fn with_avatar(mut self) -> Self {
        self.avatar = true;
        self
    }
}

impl RenderOnce for Skeleton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let pad = if env.is_touch() { rems(1.) } else { rems(0.5) };
        v_flex().w_full().children((0..self.rows).map(|row| {
            let width = [0.85, 0.6, 0.75, 0.5][row % 4];
            h_flex()
                .h(scale.row_two_line)
                .px(pad)
                .gap_3()
                .when(self.avatar, |row| row.child(KitSkeleton::new().size(rems(2.)).rounded_full()))
                .child(
                    v_flex()
                        .flex_1()
                        .gap_1p5()
                        .child(KitSkeleton::new().h(rems(0.75)).w(relative(width * 0.5)))
                        .child(KitSkeleton::new().h(rems(0.75)).w(relative(width))),
                )
        }))
    }
}
