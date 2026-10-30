//! How much of the window the system covers: the status bar, the navigation
//! bar, and the on-screen keyboard, from `gpui-android-glue`.
//!
//! Read in `render`, every frame, never kept: while the keyboard slides the
//! glue reports every frame of it and the window is drawn again, so whatever
//! makes room for it moves with it with nothing to animate. On a desktop all
//! three are zero, unless `GPUI_KEYBOARD=<height>` pretends a keyboard.

use gpui_kit::{Pixels, Window};

/// The system's insets over `window` in this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    /// The status bar.
    pub top: Pixels,
    /// The navigation bar, never the keyboard.
    pub bars: Pixels,
    /// The keyboard as drawn now, the navigation bar under it included; zero
    /// while it is down.
    pub keyboard: Pixels,
}

impl Insets {
    pub fn of(window: &Window) -> Self {
        let (top, bars) = gpui_android_glue::insets::system_bars();
        Insets { top, bars, keyboard: gpui_android_glue::keyboard::keyboard_inset(window) }
    }

    /// The room to keep at the bottom under what is typed into: the keyboard
    /// or the navigation bar, whichever covers more.
    pub fn typing(&self) -> Pixels {
        self.bars.max(self.keyboard)
    }

    /// How far the keyboard reaches above the navigation bar: what a page
    /// that already keeps the bar's room has to rise by.
    pub fn keyboard_over_bars(&self) -> Pixels {
        (self.keyboard - self.bars).max(Pixels::ZERO)
    }
}
