//! The colours an application paints its own parts with, by role, in the
//! skin in use: what `cx.theme()` is to gpui-kit.
//!
//! An intent knows which of the theme's colours (pointer) or the
//! [`crate::Palette`]'s roles (touch) it wants. A part an application makes
//! of its own (an agent's step, a board) asks for a role instead, and gets
//! the right colour in either skin: `raised` is the theme's background card
//! on a pointer and Material's surface container on a finger.

use gpui_kit::component::ActiveTheme;
use gpui_kit::{App, Hsla, SharedString};

use crate::banner::Tone;
use crate::env::{InputMode, input_mode};
use crate::palette::ActivePalette;

/// The roles, resolved for the skin in use.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Colors {
    /// Running text.
    pub text: Hsla,
    /// Secondary text: details, metadata, what is quiet.
    pub muted: Hsla,
    /// The page.
    pub surface: Hsla,
    /// A card on the page.
    pub raised: Hsla,
    /// A well in the page: a code block, a board's column.
    pub sunken: Hsla,
    pub border: Hsla,
    /// The application's colour: a progress, the thing that is on.
    pub accent: Hsla,
    pub on_accent: Hsla,
    /// Under a pointer that is over something pressable.
    pub hover: Hsla,
    /// Under something being pressed.
    pub pressed: Hsla,
    /// The person's own message.
    pub mine: Hsla,
    pub on_mine: Hsla,
    pub success: Hsla,
    pub warning: Hsla,
    pub danger: Hsla,
    pub info: Hsla,
    /// The font for text a machine wrote.
    pub mono_font: SharedString,
}

impl Colors {
    pub fn global(cx: &App) -> Colors {
        let theme = cx.theme();
        let (success, warning, danger, info) = (theme.success, theme.warning, theme.danger, theme.info);
        let mono_font = theme.mono_font_family.clone();
        match input_mode(cx) {
            InputMode::Pointer => Colors {
                text: theme.foreground,
                muted: theme.muted_foreground,
                surface: theme.background,
                raised: theme.background,
                sunken: theme.muted,
                border: theme.border,
                accent: theme.primary,
                on_accent: theme.primary_foreground,
                hover: theme.accent,
                pressed: theme.list_active,
                mine: theme.secondary,
                on_mine: theme.secondary_foreground,
                success,
                warning,
                danger,
                info,
                mono_font,
            },
            InputMode::Touch => {
                let palette = cx.palette();
                Colors {
                    text: palette.on_surface,
                    muted: palette.on_surface_variant,
                    surface: palette.surface,
                    raised: palette.surface,
                    sunken: palette.surface_container,
                    border: palette.outline_variant,
                    accent: palette.primary,
                    on_accent: palette.on_primary,
                    hover: palette.pressed,
                    pressed: palette.pressed,
                    mine: palette.primary_container,
                    on_mine: palette.on_primary_container,
                    success,
                    warning,
                    danger: palette.error,
                    info,
                    mono_font,
                }
            }
        }
    }

    /// The colour of a tone.
    pub fn tone(&self, tone: Tone) -> Hsla {
        match tone {
            Tone::Info => self.info,
            Tone::Success => self.success,
            Tone::Warning => self.warning,
            Tone::Danger => self.danger,
        }
    }
}

/// Reads the colours by role, the way `cx.theme()` reads the theme.
pub trait ActiveColors {
    fn colors(&self) -> Colors;
}

impl ActiveColors for App {
    fn colors(&self) -> Colors {
        Colors::global(self)
    }
}
