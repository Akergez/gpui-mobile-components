//! What a frame is drawn for: the way the person points ([`InputMode`]) and
//! how much room the window has ([`WidthClass`]).
//!
//! The two are decided in different places on purpose. The input mode is the
//! application's to say once, at start: a phone's entry point says
//! [`InputMode::Touch`], a desktop's `main` says [`InputMode::Pointer`], and
//! a test or a preview may say either. It decides the skin (targets, hover
//! or ripple, popover or sheet) and which backend a stateful intent builds,
//! so it does not change under a running window.
//!
//! The width class is read off the window on every frame, in rems: what
//! decides whether a sidebar and a table fit is how many characters fit, and
//! that changes with the interface's zoom as much as with the window
//! (money-manager's `shell/layout.rs`, whose bounds these are). Nothing about
//! it is stored.

use gpui_kit::{App, Global, Window};

/// Below this many rems a window is [`WidthClass::Compact`]: one pane, the
/// navigation along the bottom.
pub const COMPACT_BELOW: f32 = 44.0;

/// From this many rems a window is [`WidthClass::Expanded`]: a sidebar with
/// its labels and columns side by side.
pub const EXPANDED_FROM: f32 = 66.0;

/// How the person points at the interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InputMode {
    /// A mouse or a trackpad, and a keyboard: gpui-kit as its Design Guides
    /// draw it.
    Pointer,
    /// A finger: Material 3, with targets of 48 dp, ripples, sheets and
    /// gestures.
    Touch,
}

/// How much room the window has, in rems.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WidthClass {
    /// Below [`COMPACT_BELOW`]: one pane at a time.
    Compact,
    /// Between the two bounds: an icon rail and two panes.
    Medium,
    /// From [`EXPANDED_FROM`]: a sidebar with labels, columns side by side.
    Expanded,
}

impl WidthClass {
    /// The class of a window `width` rems wide. The bounds belong to the
    /// wider class: 44.0 is already [`WidthClass::Medium`].
    pub fn for_rems(width: f32) -> Self {
        if width < COMPACT_BELOW {
            WidthClass::Compact
        } else if width < EXPANDED_FROM {
            WidthClass::Medium
        } else {
            WidthClass::Expanded
        }
    }

    /// The class `window` has room for at its current rem.
    pub fn of(window: &Window) -> Self {
        Self::for_rems(window.viewport_size().width / window.rem_size())
    }

    /// What a UI scenario or a log line calls it.
    pub fn name(self) -> &'static str {
        match self {
            WidthClass::Compact => "compact",
            WidthClass::Medium => "medium",
            WidthClass::Expanded => "expanded",
        }
    }
}

/// The input mode of the application, set by [`crate::init`].
pub(crate) struct Settings {
    pub(crate) input: InputMode,
}

impl Global for Settings {}

/// What the frame being drawn is for. Read it in `render` with [`Env::of`];
/// never keep it, since the width class changes as the window is resized.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Env {
    input: InputMode,
    width: WidthClass,
}

impl Env {
    /// The environment of `window` now. Before [`crate::init`] the input mode
    /// is [`InputMode::Pointer`].
    pub fn of(window: &Window, cx: &App) -> Self {
        Env { input: input_mode(cx), width: WidthClass::of(window) }
    }

    /// An environment given outright: for tests and for drawing one skin
    /// inside another, as the gallery does.
    pub fn new(input: InputMode, width: WidthClass) -> Self {
        Env { input, width }
    }

    pub fn input(&self) -> InputMode {
        self.input
    }

    pub fn width(&self) -> WidthClass {
        self.width
    }

    /// Whether this is the touch skin. Feature code should rarely ask: every
    /// use of this outside the design system is a place where an intent is
    /// missing.
    pub fn is_touch(&self) -> bool {
        self.input == InputMode::Touch
    }

    /// Whether only one pane fits.
    pub fn is_compact(&self) -> bool {
        self.width == WidthClass::Compact
    }
}

/// The application's input mode.
pub fn input_mode(cx: &App) -> InputMode {
    cx.try_global::<Settings>().map_or(InputMode::Pointer, |settings| settings.input)
}

/// Changes the input mode of a running application: for the gallery's switch
/// and for tests. Stateful intents built before keep the backend they built;
/// make them again after this.
pub fn set_input_mode(input: InputMode, cx: &mut App) {
    cx.set_global(Settings { input });
    crate::scale::Scale::install(input, cx);
    cx.refresh_windows();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_width_class_changes_exactly_at_its_bounds() {
        assert_eq!(WidthClass::for_rems(24.0), WidthClass::Compact);
        assert_eq!(WidthClass::for_rems(43.9), WidthClass::Compact);
        assert_eq!(WidthClass::for_rems(44.0), WidthClass::Medium);
        assert_eq!(WidthClass::for_rems(65.9), WidthClass::Medium);
        assert_eq!(WidthClass::for_rems(66.0), WidthClass::Expanded);
        assert_eq!(WidthClass::for_rems(160.0), WidthClass::Expanded);
    }

    #[test]
    fn width_classes_are_ordered_by_room() {
        assert!(WidthClass::Compact < WidthClass::Medium);
        assert!(WidthClass::Medium < WidthClass::Expanded);
    }
}
