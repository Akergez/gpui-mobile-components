//! Who and how many: a face with its presence, and a count of what waits.
//!
//! Made once so that the same person looks the same in a sidebar, a feed and
//! an inbox (matterfast's `kit.rs`), and so that counts stay semantic and
//! scarce: neutral or primary for a number, the danger colour only for what
//! waits on the person (Coding Guides, "keep Badge variants semantic").

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::ActiveTheme;
use gpui_kit::prelude::*;
use gpui_kit::{App, ElementId, FontWeight, ImageSource, RenderOnce, Rems, SharedString, Window, div, img, rems};

use crate::env::{Env, InputMode};
use crate::palette::ActivePalette;

/// Whether someone is there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presence {
    Online,
    Away,
    Busy,
    Offline,
}

/// A face: a picture, or initials on a container colour; with a dot for
/// presence. 2 rem on a pointer, 2.5 rem for a finger, unless sized.
#[derive(IntoElement)]
pub struct Avatar {
    id: ElementId,
    initials: SharedString,
    picture: Option<ImageSource>,
    presence: Option<Presence>,
    size: Option<Rems>,
    square: bool,
    hue: Option<f32>,
}

impl Avatar {
    /// `id` is the person's or the thing's own (`("user", user.id)`).
    pub fn new(id: impl Into<ElementId>, initials: impl Into<SharedString>) -> Self {
        Avatar { id: id.into(), initials: initials.into(), picture: None, presence: None, size: None, square: false, hue: None }
    }

    pub fn picture(mut self, picture: impl Into<ImageSource>) -> Self {
        self.picture = Some(picture.into());
        self
    }

    pub fn presence(mut self, presence: Presence) -> Self {
        self.presence = Some(presence);
        self
    }

    pub fn size(mut self, size: Rems) -> Self {
        self.size = Some(size);
        self
    }

    /// Rounded square corners: a team, a server, a project rather than a
    /// person.
    /// Its own colour rather than the scheme's tertiary container: a hue
    /// (0–1) derived from the thing's id, so that each project or person
    /// keeps one colour everywhere. Light or dark as the theme is.
    pub fn hue(mut self, hue: f32) -> Self {
        self.hue = Some(hue);
        self
    }

    pub fn square(mut self) -> Self {
        self.square = true;
        self
    }
}

impl RenderOnce for Avatar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let palette = cx.palette();
        let theme = cx.theme();
        let size = self.size.unwrap_or(match env.input() {
            InputMode::Pointer => rems(2.),
            InputMode::Touch => rems(2.5),
        });
        let dot = self.presence.map(|presence| {
            let color = match presence {
                Presence::Online => theme.success,
                Presence::Away => theme.warning,
                Presence::Busy => theme.danger,
                Presence::Offline => theme.muted_foreground,
            };
            div()
                .absolute()
                .right(gpui_kit::px(-1.))
                .bottom(gpui_kit::px(-1.))
                .size(rems(size.0 * 0.32))
                .rounded_full()
                .border_2()
                .border_color(theme.background)
                .bg(color)
        });
        let dark = theme.mode.is_dark();
        let (container, on_container) = match self.hue {
            Some(hue) => (
                gpui_kit::hsla(hue, 0.55, if dark { 0.26 } else { 0.88 }, 1.),
                gpui_kit::hsla(hue, 0.6, if dark { 0.8 } else { 0.3 }, 1.),
            ),
            None => (palette.tertiary_container, palette.on_tertiary_container),
        };
        let radius = if self.square { rems(size.0 * 0.28) } else { rems(size.0) };
        let face = match self.picture {
            Some(picture) => img(picture).size_full().rounded(radius).into_any_element(),
            None => div()
                .size_full()
                .rounded(radius)
                .flex()
                .items_center()
                .justify_center()
                .bg(container)
                .text_color(on_container)
                .text_size(rems(size.0 * 0.4))
                .font_weight(FontWeight::SEMIBOLD)
                .child(self.initials)
                .into_any_element(),
        };
        div()
            .id(self.id)
            .relative()
            .flex_none()
            .size(size)
            .child(face)
            .children(dot)
            .test_support()
    }
}

/// A count of what waits: unread messages, conversations that need you.
/// Nothing is drawn for zero.
#[derive(IntoElement)]
pub struct Count {
    count: usize,
    urgent: bool,
}

impl Count {
    pub fn new(count: usize) -> Self {
        Count { count, urgent: false }
    }

    /// Something waits on the person: a mention, a question. The danger
    /// colour; everything else is the primary.
    pub fn urgent(mut self, urgent: bool) -> Self {
        self.urgent = urgent;
        self
    }
}

impl RenderOnce for Count {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let (bg, fg) = if self.urgent {
            (theme.danger, theme.danger_foreground)
        } else {
            (theme.primary, theme.primary_foreground)
        };
        let text = if self.count > 99 { "99+".to_string() } else { self.count.to_string() };
        div().when(self.count > 0, |count| {
            count
                .min_w(rems(1.125))
                .h(rems(1.125))
                .px(rems(0.3))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(bg)
                .text_color(fg)
                .text_size(rems(0.6875))
                .font_weight(FontWeight::SEMIBOLD)
                .child(text)
        })
    }
}
