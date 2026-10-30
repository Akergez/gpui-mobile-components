//! The colours the touch skin paints with, beyond the theme's.
//!
//! Both skins read the theme first: `cx.theme()` is the scheme of
//! `gpui-adaptive-colors` while the scheme is worn, and a Zed theme while one
//! is. Material 3 also has roles the theme has no place for: the containers a
//! FAB, a navigation pill and an avatar are filled with, the inverse surface a
//! snackbar is. `cx.scheme()` has them, but it goes on describing the scheme
//! after a Zed theme has been put on, and a FAB in the scheme's green on a
//! Catppuccin window would be wrong. So the touch skin reads [`Palette`]:
//! the scheme's roles while the scheme is worn, otherwise the nearest the
//! active theme has.

use gpui_adaptive_colors::{ActiveScheme, AdaptiveColors};
use gpui_kit::component::ActiveTheme;
use gpui_kit::{App, Hsla};

/// What the touch skin fills and writes with. Every colour comes from the
/// scheme or the theme in use; nothing is a literal.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct Palette {
    pub surface: Hsla,
    pub on_surface: Hsla,
    pub on_surface_variant: Hsla,
    /// Bars, sheets.
    pub surface_low: Hsla,
    /// The navigation bar, menus.
    pub surface_container: Hsla,
    /// Dialogs, filled fields.
    pub surface_high: Hsla,
    pub surface_highest: Hsla,
    pub primary: Hsla,
    pub on_primary: Hsla,
    /// The FAB, your own message.
    pub primary_container: Hsla,
    pub on_primary_container: Hsla,
    /// The selected destination's pill, a selected row.
    pub secondary_container: Hsla,
    pub on_secondary_container: Hsla,
    /// Avatars without a picture.
    pub tertiary_container: Hsla,
    pub on_tertiary_container: Hsla,
    pub error: Hsla,
    pub on_error: Hsla,
    pub outline: Hsla,
    pub outline_variant: Hsla,
    /// The snackbar.
    pub inverse_surface: Hsla,
    pub inverse_on_surface: Hsla,
    pub inverse_primary: Hsla,
    /// What a press lays over what is pressed: the content colour at 12%.
    pub pressed: Hsla,
    /// The window behind a sheet or a dialog.
    pub scrim: Hsla,
}

impl Palette {
    /// The palette in use: the scheme's while it is worn, the theme's when a
    /// theme of the person's choosing is.
    pub fn global(cx: &App) -> Palette {
        let theme = cx.theme();
        let scheme_worn = cx
            .try_global::<AdaptiveColors>()
            .is_some_and(|colors| colors.worn(theme.mode).is_none());
        if scheme_worn { Self::from_scheme(cx) } else { Self::from_theme(cx) }
    }

    fn from_scheme(cx: &App) -> Palette {
        let scheme = cx.scheme();
        let theme = cx.theme();
        Palette {
            surface: scheme.surface,
            on_surface: scheme.on_surface,
            on_surface_variant: scheme.on_surface_variant,
            surface_low: scheme.surface_container_low,
            surface_container: scheme.surface_container,
            surface_high: scheme.surface_container_high,
            surface_highest: scheme.surface_container_highest,
            primary: scheme.primary,
            on_primary: scheme.on_primary,
            primary_container: scheme.primary_container,
            on_primary_container: scheme.on_primary_container,
            secondary_container: scheme.secondary_container,
            on_secondary_container: scheme.on_secondary_container,
            tertiary_container: scheme.tertiary_container,
            on_tertiary_container: scheme.on_tertiary_container,
            error: scheme.error,
            on_error: scheme.on_error,
            outline: scheme.outline,
            outline_variant: scheme.outline_variant,
            inverse_surface: scheme.inverse_surface,
            inverse_on_surface: scheme.inverse_on_surface,
            inverse_primary: scheme.inverse_primary,
            pressed: scheme.on_surface.opacity(0.12),
            scrim: theme.overlay,
        }
    }

    /// The nearest a theme without Material's roles has: the containers are
    /// the theme's accent colours laid thinly over its background.
    fn from_theme(cx: &App) -> Palette {
        let theme = cx.theme();
        let over = |color: Hsla, amount: f32| mix(theme.background, color, amount);
        Palette {
            surface: theme.background,
            on_surface: theme.foreground,
            on_surface_variant: theme.muted_foreground,
            surface_low: theme.sidebar,
            surface_container: theme.popover,
            surface_high: theme.muted,
            surface_highest: theme.accent,
            primary: theme.primary,
            on_primary: theme.primary_foreground,
            primary_container: over(theme.primary, 0.16),
            on_primary_container: theme.foreground,
            secondary_container: theme.sidebar_accent,
            on_secondary_container: theme.sidebar_accent_foreground,
            tertiary_container: over(theme.info, 0.18),
            on_tertiary_container: theme.foreground,
            error: theme.danger,
            on_error: theme.danger_foreground,
            outline: theme.input,
            outline_variant: theme.border,
            inverse_surface: theme.foreground,
            inverse_on_surface: theme.background,
            inverse_primary: over(theme.primary, 0.6),
            pressed: theme.foreground.opacity(0.12),
            scrim: theme.overlay,
        }
    }
}

/// `color` laid over `base` at `amount`, opaque.
fn mix(base: Hsla, color: Hsla, amount: f32) -> Hsla {
    let (base, color) = (base.to_rgb(), color.to_rgb());
    let lerp = |a: f32, b: f32| a + (b - a) * amount;
    gpui_kit::Rgba { r: lerp(base.r, color.r), g: lerp(base.g, color.g), b: lerp(base.b, color.b), a: 1. }.into()
}

/// Reads the palette in use, the way `cx.theme()` reads the theme.
pub trait ActivePalette {
    fn palette(&self) -> Palette;
}

impl ActivePalette for App {
    fn palette(&self) -> Palette {
        Palette::global(self)
    }
}
