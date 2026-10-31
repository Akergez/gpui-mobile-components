//! The lengths and durations of a skin: how tall a control is, how round a
//! sheet is, how long a page takes to come in.
//!
//! Every length is in rems, so that both skins follow the theme's base font,
//! which is the application's zoom (Coding Guides, "Base font is the
//! application zoom control"). The pointer scale is gpui-kit's own; the touch
//! scale is Material 3's, read at 16 px to the rem. gpui-kit's theme cannot
//! store a scale of the application's own (`apply_semantic_tokens` keeps no
//! custom spacing), so the scale is a global of this crate, set by
//! [`crate::init`] and [`crate::set_input_mode`].

use std::time::Duration;

use gpui_kit::{App, Global, Rems, rems};

use crate::env::InputMode;

/// The scale of the skin in use.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Scale {
    /// A button's height.
    pub control: Rems,
    /// The least a pressable thing is in both directions.
    pub target: Rems,
    /// A one-line row of a list.
    pub row: Rems,
    /// A row with a title and a line under it.
    pub row_two_line: Rems,
    /// The bar across the top of a page.
    pub bar: Rems,
    /// The navigation bar along the bottom, without the system's inset.
    pub navigation_bar: Rems,
    /// The rail down the side.
    pub rail: Rems,
    /// The sidebar with labels.
    pub sidebar: Rems,
    /// The floating action button.
    pub fab: Rems,
    /// Corners of buttons and fields.
    pub radius_control: Rems,
    /// Corners of cards and menus.
    pub radius_container: Rems,
    /// Corners of sheets and dialogs.
    pub radius_sheet: Rems,
    /// The page title's size; the root page's when it differs.
    pub title: Rems,
    pub title_root: Rems,
    /// Running text.
    pub body: Rems,
    /// Secondary text: subtitles, metadata.
    pub small: Rems,
    /// Room between things, from the least (an icon and its label) to the
    /// most (the parts of a page).
    pub space_xs: Rems,
    pub space_s: Rems,
    pub space_m: Rems,
    pub space_l: Rems,
    pub space_xl: Rems,
    /// A page coming in or going.
    pub page: Duration,
    /// A sheet coming up, going down, settling where a drag let it go.
    pub sheet_open: Duration,
    pub sheet_close: Duration,
    pub settle: Duration,
    /// How long a notice stays before it goes by itself.
    pub notice: Duration,
}

impl Scale {
    /// gpui-kit's scale: 28 px controls, dense rows, short fades.
    pub fn pointer() -> Self {
        Scale {
            control: rems(1.75),
            target: rems(1.5),
            row: rems(1.75),
            row_two_line: rems(2.75),
            bar: rems(2.75),
            navigation_bar: rems(3.5),
            rail: rems(3.75),
            sidebar: rems(14.),
            fab: rems(3.5),
            radius_control: rems(0.375),
            radius_container: rems(0.5),
            radius_sheet: rems(0.625),
            title: rems(0.875),
            title_root: rems(0.875),
            body: rems(0.8125),
            small: rems(0.75),
            space_xs: rems(0.25),
            space_s: rems(0.5),
            space_m: rems(0.75),
            space_l: rems(1.),
            space_xl: rems(1.5),
            page: Duration::from_millis(120),
            sheet_open: Duration::from_millis(160),
            sheet_close: Duration::from_millis(120),
            settle: Duration::from_millis(120),
            notice: Duration::from_secs(6),
        }
    }

    /// Material 3's scale: 40 dp buttons, 48 dp targets, 56 dp rows and
    /// fields, large radii, longer motion.
    pub fn touch() -> Self {
        Scale {
            control: rems(2.5),
            target: rems(3.),
            row: rems(3.5),
            row_two_line: rems(4.5),
            bar: rems(4.),
            navigation_bar: rems(5.),
            rail: rems(5.),
            sidebar: rems(17.5),
            fab: rems(3.5),
            radius_control: rems(1.25),
            radius_container: rems(0.75),
            radius_sheet: rems(1.75),
            title: rems(1.25),
            title_root: rems(1.75),
            body: rems(1.),
            small: rems(0.875),
            space_xs: rems(0.25),
            space_s: rems(0.5),
            space_m: rems(1.),
            space_l: rems(1.5),
            space_xl: rems(2.),
            page: Duration::from_millis(300),
            sheet_open: Duration::from_millis(260),
            sheet_close: Duration::from_millis(200),
            settle: Duration::from_millis(180),
            notice: Duration::from_secs(4),
        }
    }

    /// The scale for `input`.
    pub fn for_input(input: InputMode) -> Self {
        match input {
            InputMode::Pointer => Self::pointer(),
            InputMode::Touch => Self::touch(),
        }
    }

    pub(crate) fn install(input: InputMode, cx: &mut App) {
        cx.set_global(Self::for_input(input));
    }

    /// The scale in use; the pointer's before [`crate::init`].
    pub fn global(cx: &App) -> Scale {
        cx.try_global::<Scale>().cloned().unwrap_or_else(Scale::pointer)
    }
}

impl Global for Scale {}

/// Reads the scale in use, the way `cx.theme()` reads the theme.
pub trait ActiveScale {
    fn scale(&self) -> Scale;
}

impl ActiveScale for App {
    fn scale(&self) -> Scale {
        Scale::global(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_touch_target_is_at_least_48_dp_and_larger_than_a_pointers() {
        let (touch, pointer) = (Scale::touch(), Scale::pointer());
        assert!(touch.target.0 * 16. >= 48.);
        assert!(touch.target.0 > pointer.target.0);
        assert!(touch.row.0 > pointer.row.0);
    }
}
