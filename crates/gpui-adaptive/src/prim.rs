//! What an application builds its own parts from, when no intent is the
//! part: an agent's step, a board, a diff. Built of these and of gpui's
//! `div`, sized by [`crate::Scale`] and painted by [`crate::Colors`], such a
//! part looks like the rest in either skin without naming one.
//!
//! - [`row`], [`column()`]: flex boxes, the room between things from the scale;
//! - [`surface`]: a card, a panel or a well;
//! - [`pressable`]: what can be pressed, with the skin's feedback, and
//!   [`testable`] to find it in a UI test;
//! - [`scroll_y`], [`scroll_x`]: a scroll area locked to its axis for a finger;
//! - [`tooltip`], [`spinner`];
//! - [`Text`], [`Markdown`], [`Meter`].

use std::sync::Arc;

use gpui_kit::base::InteractiveElementExt as _;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::text::TextView;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::Sizable as _;
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, AnyView, App, Div, ElementId, FontWeight, RenderOnce, SharedString, Stateful, Window, div, relative, rems};

use crate::banner::Tone;
use crate::colors::ActiveColors;
use crate::env::{InputMode, input_mode};
use crate::scale::ActiveScale;

/// The skin in use; the width does not matter to what is here.
fn env(cx: &App) -> crate::env::Env {
    crate::env::Env::new(input_mode(cx), crate::env::WidthClass::Expanded)
}

/// Things side by side, centred across.
pub fn row() -> Div {
    div().flex().flex_row().items_center()
}

/// Things one under another.
pub fn column() -> Div {
    div().flex().flex_col()
}

/// What a [`surface`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    /// A card on the page: bordered, the container's corners.
    Card,
    /// A panel holding cards: filled with the sunken colour.
    Panel,
    /// A well for text a machine wrote: sunken, the control's corners.
    Well,
}

/// A box drawn as `kind`; lay it out and fill it as any `div`.
pub fn surface(kind: Surface, cx: &App) -> Div {
    let (colors, scale) = (cx.colors(), cx.scale());
    let base = div().flex().flex_col();
    match kind {
        Surface::Card => base.border_1().border_color(colors.border).rounded(scale.radius_container).bg(colors.raised),
        Surface::Panel => base.rounded(scale.radius_container).bg(colors.sunken),
        Surface::Well => base.rounded(scale.radius_control).bg(colors.sunken),
    }
}

/// A row that can be pressed: hover and press colours for a pointer,
/// Material's pressed layer for a finger. Finish it with [`testable`] for a
/// UI test to find it by its id.
pub fn pressable(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    crate::touch::pressable(id, env(cx), cx)
}

/// `element`, findable by its id in a UI test (gpui-kit's test driver):
/// the last call on it, as it is no longer a `div` after.
pub fn testable(element: Stateful<Div>) -> AnyElement {
    use gpui_kit::base::TestSupportExt as _;
    element.test_support().into_any_element()
}

/// A vertical scroll area filling its parent.
pub fn scroll_y(id: impl Into<ElementId>, cx: &App, content: impl IntoElement) -> AnyElement {
    crate::touch::scroll_y(id, env(cx), content)
}

/// A row that scrolls sideways when it is wider than its parent: a
/// scrollbar for a pointer, a swipe locked to its axis for a finger.
pub fn scroll_x(id: impl Into<ElementId>, cx: &App, content: Div) -> AnyElement {
    let area = content.id(id.into());
    match input_mode(cx) {
        InputMode::Pointer => area.overflow_x_scrollbar().into_any_element(),
        InputMode::Touch => area.overflow_x_scroll().lock_scroll_axis().into_any_element(),
    }
}

/// A tooltip saying `text`, for gpui's `.tooltip(…)`.
pub fn tooltip(text: impl Into<SharedString>) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    let text = text.into();
    move |window, cx| Tooltip::new(text.clone()).build(window, cx)
}

/// Work under way, the size of a line of text.
pub fn spinner() -> AnyElement {
    Spinner::new().small().into_any_element()
}

/// How big text is, and what it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Title,
    Body,
    Small,
    Mono,
}

/// A run of text in one of the skin's sizes.
#[derive(IntoElement)]
pub struct Text {
    text: SharedString,
    role: Role,
    muted: bool,
    tone: Option<Tone>,
    strong: bool,
    truncate: bool,
    clamp: Option<usize>,
    grow: bool,
    center: bool,
}

impl Text {
    fn of(role: Role, text: impl Into<SharedString>) -> Self {
        Text { text: text.into(), role, muted: false, tone: None, strong: false, truncate: false, clamp: None, grow: false, center: false }
    }

    /// A heading: a card's title.
    pub fn title(text: impl Into<SharedString>) -> Self {
        Self::of(Role::Title, text)
    }

    /// Running text.
    pub fn body(text: impl Into<SharedString>) -> Self {
        Self::of(Role::Body, text)
    }

    /// Secondary text: a detail, a label in a dense row.
    pub fn small(text: impl Into<SharedString>) -> Self {
        Self::of(Role::Small, text)
    }

    /// Text a machine wrote, or that is typed as it is: "/review".
    pub fn mono(text: impl Into<SharedString>) -> Self {
        Self::of(Role::Mono, text)
    }

    pub fn muted(mut self) -> Self {
        self.muted = true;
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }

    pub fn strong(mut self) -> Self {
        self.strong = true;
        self
    }

    /// One line, cut with an ellipsis.
    pub fn truncate(mut self) -> Self {
        self.truncate = true;
        self
    }

    /// At most `lines` lines.
    pub fn clamp(mut self, lines: usize) -> Self {
        self.clamp = Some(lines);
        self
    }

    /// Takes the room left in its row (and may shrink below its text).
    pub fn grow(mut self) -> Self {
        self.grow = true;
        self
    }

    pub fn center(mut self) -> Self {
        self.center = true;
        self
    }
}

impl RenderOnce for Text {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let (colors, scale) = (cx.colors(), cx.scale());
        let size = match self.role {
            Role::Title | Role::Body => scale.body,
            Role::Small | Role::Mono => scale.small,
        };
        let color = match (self.tone, self.muted) {
            (Some(tone), _) => colors.tone(tone),
            (None, true) => colors.muted,
            (None, false) => colors.text,
        };
        div()
            .min_w_0()
            .text_size(size)
            .text_color(color)
            .when(self.role == Role::Title || self.strong, |text| text.font_weight(FontWeight::SEMIBOLD))
            .when(self.role == Role::Mono, |text| text.font_family(colors.mono_font.clone()))
            .when(self.truncate, |text| text.truncate())
            .when_some(self.clamp, |text, lines| text.line_clamp(lines))
            .when(self.grow, |text| text.flex_1())
            .when(!self.grow, |text| text.flex_none())
            .when(self.center, |text| text.w_full().text_center())
            .child(self.text)
    }
}

/// What a link in [`Markdown`] opens; gpui-kit's text view wants it `Send`.
type OnLink = dyn Fn(&str, &mut Window, &mut App) + Send + Sync;

/// Text in Markdown, which can be selected.
#[derive(IntoElement)]
pub struct Markdown {
    id: ElementId,
    text: SharedString,
    on_link: Option<Arc<OnLink>>,
}

impl Markdown {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Markdown { id: id.into(), text: text.into(), on_link: None }
    }

    pub fn on_link(mut self, on_link: impl Fn(&str, &mut Window, &mut App) + Send + Sync + 'static) -> Self {
        self.on_link = Some(Arc::new(on_link));
        self
    }
}

impl RenderOnce for Markdown {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let size = cx.scale().body;
        let on_link = self.on_link;
        div().text_size(size).child(
            TextView::markdown(self.id, self.text)
                .selectable(true)
                .when_some(on_link, |view, on_link| view.on_link_click(move |url, _, window, cx| on_link(url, window, cx))),
        )
    }
}

/// How far along something is, as a short bar: a plan's items done.
#[derive(IntoElement)]
pub struct Meter {
    fraction: f32,
}

impl Meter {
    pub fn new(fraction: f32) -> Self {
        Meter { fraction: fraction.clamp(0., 1.) }
    }
}

impl RenderOnce for Meter {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        div()
            .flex_none()
            .w(rems(4.))
            .h(rems(0.25))
            .rounded_full()
            .bg(colors.sunken)
            .child(div().h_full().rounded_full().bg(colors.accent).w(relative(self.fraction)))
    }
}
