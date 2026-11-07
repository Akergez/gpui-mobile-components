//! What an application builds its own parts from, when no intent is the
//! part: an agent's step, a board, a diff. Built of these and of gpui's
//! `div`, sized by [`crate::Scale`] and painted by [`crate::Colors`], such a
//! part looks like the rest in either skin without naming one.
//!
//! - [`row`], [`column()`]: flex boxes, the room between things from the scale;
//! - [`surface`]: a card, a panel or a well;
//! - [`pressable`]: what can be pressed, with the skin's feedback;
//!   [`key_pressable`] reached by the keyboard too; [`testable`] to find it
//!   in a UI test;
//! - [`scroll_y`], [`scroll_x`]: a scroll area locked to its axis for a finger;
//!   [`Scrolls`] for the application's own scrolling element;
//! - [`tooltip`], [`spinner`], [`rem_size`];
//! - [`Text`], [`Markdown`], [`Meter`], [`Tile`], [`Pill`].

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

/// What a rem is in pixels, where there is no window to ask: the size the
/// person's text is set to. For a length that has to be a number of pixels
/// (a list row's height, measured ahead).
pub fn rem_size(cx: &App) -> gpui_kit::Pixels {
    use gpui_kit::component::ActiveTheme as _;
    cx.theme().font_size
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

/// A [`pressable`] the keyboard reaches too: Tab stops on it, Enter or
/// Space presses it, and a ring shows where the keyboard is. For the cells
/// of a grid to choose from (a form's categories, icons), which are not a
/// list's rows and would otherwise be a pointer's or a finger's alone.
pub fn key_pressable(id: impl Into<ElementId>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
    let id = id.into();
    let focus = window.use_keyed_state(id.clone(), cx, |_, cx| cx.focus_handle()).read(cx).clone();
    let ring = cx.colors().focus;
    pressable(id, cx)
        .track_focus(&focus)
        .tab_stop(true)
        .border_1()
        .border_color(gpui_kit::transparent_black())
        .focus_visible(move |style| style.border_color(ring))
        // A press of the pointer or the finger does not leave the ring.
        .on_mouse_down(gpui_kit::MouseButton::Left, |_, window, _| window.prevent_default())
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

/// Scrolling along one axis on an element the application lays out itself:
/// a list of bounded height inside a popup, a strip of tabs. The axis is
/// locked, so that a finger's swipe across it moves what is around it
/// instead (the gpui-mobile rule). Track a scroll handle on it as usual.
pub trait Scrolls: Sized {
    /// Scrolls up and down, and only so.
    fn scrolls_y(self) -> Self;
    /// Scrolls sideways, and only so.
    fn scrolls_x(self) -> Self;
}

impl Scrolls for Stateful<Div> {
    fn scrolls_y(self) -> Self {
        self.overflow_y_scroll().lock_scroll_axis()
    }

    fn scrolls_x(self) -> Self {
        self.overflow_x_scroll().lock_scroll_axis()
    }
}

/// `holder` with a scrollbar for the gpui `list` in it: a feed of messages
/// too long to lay out at once. A finger scrolls it without one.
pub fn list_scrollbar(holder: Div, list: &gpui_kit::ListState, cx: &App) -> Div {
    match input_mode(cx) {
        InputMode::Pointer => holder.vertical_scrollbar(list),
        InputMode::Touch => holder,
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

/// What a right click on a link in [`Markdown`] asks for, at the point it
/// landed (in window coordinates).
type OnLinkMenu = dyn Fn(&str, gpui_kit::Point<gpui_kit::Pixels>, &mut Window, &mut App) + Send + Sync;

/// Text in Markdown, which can be selected.
///
/// An application's own syntax — its custom emoji, its mentions — comes in
/// through [`crate::markdown`]'s plugins, set with [`Markdown::extensions`].
#[derive(IntoElement)]
pub struct Markdown {
    id: ElementId,
    text: SharedString,
    selectable: bool,
    inherit_size: bool,
    extensions: Option<crate::markdown::MarkdownExtensions>,
    on_link: Option<Arc<OnLink>>,
    on_link_menu: Option<Arc<OnLinkMenu>>,
}

impl Markdown {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Markdown {
            id: id.into(),
            text: text.into(),
            selectable: true,
            inherit_size: false,
            extensions: None,
            on_link: None,
            on_link_menu: None,
        }
    }

    /// Whether the text can be selected (it can): not in a row a long
    /// press opens, where selecting would take the press.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// The size of the text around it rather than the skin's body size: a
    /// message in a feed the application sizes itself.
    pub fn inherit_size(mut self) -> Self {
        self.inherit_size = true;
        self
    }

    /// The application's own syntax. Build the extensions once and clone
    /// them: a clone keeps its revision, which lets the text keep what it
    /// parsed from one frame to the next.
    pub fn extensions(mut self, extensions: crate::markdown::MarkdownExtensions) -> Self {
        self.extensions = Some(extensions);
        self
    }

    /// What a right click on a link does instead of opening it: a menu at
    /// the point (follow, copy), for [`crate::Menu::open_at`].
    pub fn on_link_menu(
        mut self,
        on_link_menu: impl Fn(&str, gpui_kit::Point<gpui_kit::Pixels>, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_link_menu = Some(Arc::new(on_link_menu));
        self
    }

    pub fn on_link(mut self, on_link: impl Fn(&str, &mut Window, &mut App) + Send + Sync + 'static) -> Self {
        self.on_link = Some(Arc::new(on_link));
        self
    }
}

impl RenderOnce for Markdown {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let size = cx.scale().body;
        let (on_link, on_link_menu) = (self.on_link, self.on_link_menu);
        let clicks = on_link.is_some() || on_link_menu.is_some();
        div().when(!self.inherit_size, |text| text.text_size(size)).child(
            TextView::markdown(self.id, self.text)
                .selectable(self.selectable)
                .when_some(self.extensions, |view, extensions| view.markdown_extensions(extensions))
                .when(clicks, |view| {
                    view.on_link_click(move |url, event, window, cx| match (event, &on_link_menu) {
                        (gpui_kit::ClickEvent::Mouse(click), Some(menu))
                            if click.up.button == gpui_kit::MouseButton::Right =>
                        {
                            menu(url, click.up.position, window, cx)
                        }
                        _ => {
                            if let Some(on_link) = &on_link {
                                on_link(url, window, cx)
                            }
                        }
                    })
                }),
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

/// How large a [`Tile`] is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TileSize {
    /// Beside a name in a dense row: a table's cell.
    Small,
    /// Leading a [`crate::ListRow`]: small in a pointer's dense row, medium
    /// in a finger's.
    #[default]
    Leading,
    /// Leading a card, or a cell of a grid to choose from.
    Medium,
    /// The preview at the top of an editor.
    Large,
}

/// An icon on a tint of its colour: what identifies a thing of the person's
/// own making (a category, a label) wherever it appears.
#[derive(IntoElement)]
pub struct Tile {
    icon: gpui_kit::component::Icon,
    color: gpui_kit::Hsla,
    size: TileSize,
}

impl Tile {
    pub fn new(icon: impl Into<gpui_kit::component::Icon>, color: gpui_kit::Hsla) -> Self {
        Tile { icon: icon.into(), color, size: TileSize::Leading }
    }

    pub fn size(mut self, size: TileSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for Tile {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let scale = cx.scale();
        let tile = div()
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .bg(self.color.opacity(0.16))
            .text_color(self.color);
        let size = match (self.size, input_mode(cx)) {
            (TileSize::Leading, InputMode::Pointer) => TileSize::Small,
            (TileSize::Leading, InputMode::Touch) => TileSize::Medium,
            (size, _) => size,
        };
        match size {
            TileSize::Small | TileSize::Leading => tile.size(rems(1.5)).rounded(rems(scale.radius_container.0 * 0.6)).child(self.icon.xsmall()),
            TileSize::Medium => tile.size(rems(2.5)).rounded(scale.radius_container).child(self.icon),
            TileSize::Large => tile.size(rems(3.)).rounded(rems(scale.radius_container.0 * 1.5)).child(self.icon.large()),
        }
    }
}

/// A small label of a thing of the person's own making, its icon in its
/// colour on a tint of it: a tag on a record. Quieter than a [`Tile`] and
/// read beside other text, so a row can carry several on its second line
/// without growing.
#[derive(IntoElement, Clone)]
pub struct Pill {
    icon: Option<gpui_kit::component::Icon>,
    label: SharedString,
    color: gpui_kit::Hsla,
}

impl Pill {
    pub fn new(label: impl Into<SharedString>, color: gpui_kit::Hsla) -> Self {
        Pill { icon: None, label: label.into(), color }
    }

    pub fn icon(mut self, icon: impl Into<gpui_kit::component::Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// What it says.
    pub fn label(&self) -> &SharedString {
        &self.label
    }
}

impl RenderOnce for Pill {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let touch = input_mode(cx) == InputMode::Touch;
        let text = cx.colors().text;
        div()
            .flex_none()
            .flex()
            .items_center()
            .gap(rems(0.25))
            .h(rems(if touch { 1.25 } else { 1.125 }))
            .px(rems(0.375))
            .rounded(rems(0.375))
            .bg(self.color.opacity(0.14))
            .text_size(rems(0.6875))
            .line_height(rems(1.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(text.opacity(0.86))
            .children(self.icon.map(|icon| icon.xsmall().text_color(self.color)))
            .child(self.label)
    }
}
