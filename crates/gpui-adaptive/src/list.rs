//! Lists: [`ListRow`], [`SectionHeader`] and the [`Collection`] they scroll
//! in. A grouped list (an inbox by what each conversation needs, records by
//! day) is a collection of [`Collection::section`]s: the grouping is said
//! once, and each skin draws it its way.
//!
//! **Pointer**: dense rows (1.75 rem, 2.75 with a second line) with the
//! theme's hover, keyboard-free to the row itself, commands on hover and on
//! a right click.
//!
//! **Touch**: Material list items: 56 dp for one line, 72 for two, a
//! pressed state layer, and the commands behind a long press.
//!
//! Every row needs a stable id from the domain (`("channel", channel.id)`),
//! never a position: positions move when rows are inserted. A list of
//! thousands belongs in a virtual list (gpui-kit's `v_virtual_list`), with
//! `ListRow`s as its items.

use std::rc::Rc;

use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{ActiveTheme, Icon, IconName, Sizable, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, ElementId, FontWeight, MouseButton, Rems, RenderOnce, SharedString, Window, div, px, rems};

use crate::banner::Tone;
use crate::env::{Env, InputMode};
use crate::item_actions::ItemActions;
use crate::palette::ActivePalette;
use crate::scale::ActiveScale;
use crate::touch;

type Activate = Rc<dyn Fn(&mut Window, &mut App)>;

/// What a row's state asks of the person, seen at a glance down the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    /// It waits on the person: an agent's question (`Warning`), a failure
    /// (`Danger`).
    Waiting(Tone),
    /// Something is working on it now.
    Busy,
    /// Something new in it the person has not seen.
    New,
}

/// One item of a list.
#[derive(IntoElement)]
pub struct ListRow {
    id: ElementId,
    leading: Option<AnyElement>,
    title: SharedString,
    subtitle: Option<SharedString>,
    pills: Vec<crate::prim::Pill>,
    supporting: Option<SharedString>,
    supporting_tone: Option<Tone>,
    trailing: Option<AnyElement>,
    signal: Option<Signal>,
    body: Option<AnyElement>,
    selected: bool,
    unread: bool,
    muted: bool,
    on_activate: Option<Activate>,
    actions: ItemActions,
}

impl ListRow {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        ListRow {
            id: id.into(),
            leading: None,
            title: title.into(),
            subtitle: None,
            pills: Vec::new(),
            supporting: None,
            supporting_tone: None,
            trailing: None,
            signal: None,
            body: None,
            selected: false,
            unread: false,
            muted: false,
            on_activate: None,
            actions: ItemActions::new(),
        }
    }

    /// An icon or an [`crate::Avatar`] before the title.
    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.leading = Some(leading.into_any_element());
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// [`crate::Pill`]s after the subtitle, on its line: the tags of a
    /// record. What does not fit is cut off at the row's end; the row does
    /// not grow for them.
    pub fn pills(mut self, pills: impl IntoIterator<Item = crate::prim::Pill>) -> Self {
        self.pills.extend(pills);
        self
    }

    /// A third line under the subtitle: the last message of a conversation,
    /// what went wrong in it.
    pub fn supporting(mut self, supporting: impl Into<SharedString>) -> Self {
        self.supporting = Some(supporting.into());
        self
    }

    /// Says the supporting line in a tone: `Danger` for an error, with its
    /// icon.
    pub fn supporting_tone(mut self, tone: Tone) -> Self {
        self.supporting_tone = Some(tone);
        self
    }

    /// The row's state: see [`Signal`]. **Pointer**: a stripe down the row's
    /// edge for what waits or works, and a dot or a spinner under the
    /// trailing text. **Touch**: the dot or the spinner only.
    pub fn signal(mut self, signal: Option<Signal>) -> Self {
        self.signal = signal;
        self
    }

    /// What ends the row: a time, an amount, a [`crate::Count`].
    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    /// Something to act on inside the row, under its text: the answers to
    /// an agent's question in an inbox ([`crate::Ask`]), Accept and Decline
    /// for an invitation. A press in it is its own: it does not open the
    /// row.
    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }

    /// The row is the one open now, beside the list. A compact window shows
    /// one page at a time, so there it is not marked: the list is all there
    /// is (and is what back's gesture shows under the page).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Something new in it: the title in bold.
    pub fn unread(mut self, unread: bool) -> Self {
        self.unread = unread;
        self
    }

    /// Asked to be quiet: dimmed.
    pub fn muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }

    pub fn on_activate(mut self, on_activate: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(on_activate));
        self
    }

    /// The row's commands: see [`ItemActions`].
    pub fn actions(mut self, actions: ItemActions) -> Self {
        self.actions = actions;
        self
    }
}

impl RenderOnce for ListRow {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let palette = cx.palette();
        let theme = cx.theme();
        let touch = env.input() == InputMode::Touch;
        let has_body = self.body.is_some();
        // Over two lines the row's parts line up at its top.
        let tall = has_body || self.supporting.is_some();
        let pills = std::mem::take(&mut self.pills);
        if self.subtitle.is_none() && !pills.is_empty() {
            self.subtitle = Some(SharedString::default());
        }
        let height = match (&self.subtitle, &self.supporting) {
            (_, Some(_)) => rems(scale.row_two_line.0 + if touch { 1.25 } else { 1. }),
            (Some(_), None) => scale.row_two_line,
            (None, None) => scale.row,
        };
        let (selected_bg, selected_fg, muted_fg) = if touch {
            (palette.secondary_container, palette.on_secondary_container, palette.on_surface_variant)
        } else {
            (theme.sidebar_accent, theme.sidebar_accent_foreground, theme.muted_foreground)
        };
        let title = div()
            .truncate()
            .text_size(scale.body)
            .when(self.unread, |title| title.font_weight(FontWeight::BOLD))
            .child(self.title);
        let text = v_flex()
            .flex_1()
            .min_w_0()
            .child(title)
            .children(self.subtitle.map(|subtitle| {
                let subtitle = div().truncate().text_size(scale.small).text_color(muted_fg).child(subtitle);
                if pills.is_empty() {
                    return subtitle;
                }
                h_flex()
                    .min_w_0()
                    .gap_1()
                    .overflow_hidden()
                    .child(subtitle.flex_none().max_w(gpui_kit::relative(0.6)))
                    .children(pills)
            }))
            .children(self.supporting.map(|supporting| {
                let tone = self.supporting_tone;
                let color = tone.map_or(muted_fg, |tone| tone.color(cx));
                let icon = match tone {
                    Some(Tone::Danger) => Some(IconName::CircleX),
                    Some(Tone::Warning) => Some(IconName::TriangleAlert),
                    _ => None,
                };
                h_flex()
                    .gap_1p5()
                    .text_size(scale.small)
                    .text_color(color)
                    .children(icon.map(|icon| Icon::new(icon).xsmall()))
                    .child(div().flex_1().min_w_0().truncate().child(supporting))
            }))
            .children(self.body.map(|body| {
                // The row opens on a click it gets the press of; a press in
                // the body stops here.
                div()
                    .pt_1p5()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(body)
            }));
        let signal_color = |signal: Signal| match signal {
            Signal::Waiting(tone) => tone.color(cx),
            Signal::Busy | Signal::New => if touch { palette.primary } else { theme.primary },
        };
        let indicator = self.signal.map(|signal| match signal {
            Signal::Busy => Spinner::new().xsmall().into_any_element(),
            _ => div().size(rems(0.5)).rounded_full().bg(signal_color(signal)).into_any_element(),
        });
        let stripe = self.signal.filter(|signal| !touch && *signal != Signal::New).map(|signal| {
            div().absolute().left(px(2.)).top_2().bottom_2().w(px(3.)).rounded_full().bg(signal_color(signal))
        });
        let trailing = (self.trailing.is_some() || indicator.is_some()).then(|| {
            v_flex()
                .flex_none()
                .items_end()
                .gap_1p5()
                .text_size(scale.small)
                .text_color(muted_fg)
                .children(self.trailing)
                .children(indicator)
        });
        let on_activate = self.on_activate.clone();
        let row = touch::pressable(self.id, env, cx)
            .w_full()
            .min_h(height)
            .when(touch, |row| row.px_4().gap_4().py_2())
            .when(!touch, |row| row.px_2().gap_2().rounded(scale.radius_control))
            .relative()
            .when(tall, |row| row.items_start())
            .when(tall && !touch, |row| row.py_1p5())
            .when(self.selected && !env.is_compact(), |row| row.bg(selected_bg).text_color(selected_fg))
            .when(self.muted, |row| row.opacity(0.55))
            .children(self.leading)
            .children(stripe)
            .child(text)
            .children(trailing)
            .when_some(on_activate, |row, on_activate| row.on_click(move |_, window, cx| on_activate(window, cx)));
        self.actions.attach(row, env, cx)
    }
}

/// A heading over a run of rows: "Favourites", "Today".
#[derive(IntoElement)]
pub struct SectionHeader {
    title: SharedString,
    tone: Option<Tone>,
}

impl SectionHeader {
    pub fn new(title: impl Into<SharedString>) -> Self {
        SectionHeader { title: title.into(), tone: None }
    }

    /// A heading that says something about its rows: "Needs you" in the
    /// warning colour.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }
}

impl RenderOnce for SectionHeader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let tone = self.tone.map(|tone| tone.color(cx));
        match env.input() {
            InputMode::Pointer => div()
                .px_2()
                .pt_3()
                .pb_1()
                .text_size(scale.small)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(tone.unwrap_or(cx.theme().muted_foreground))
                .child(self.title),
            InputMode::Touch => div()
                .px_4()
                .pt_4()
                .pb_1()
                .text_size(scale.small)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(tone.unwrap_or(cx.palette().primary))
                .child(self.title),
        }
    }
}

/// Rows in the page's scroll area: [`ListRow`]s one after another, or in
/// [`Collection::section`]s under their headings.
#[derive(IntoElement)]
pub struct Collection {
    id: ElementId,
    children: Vec<AnyElement>,
    scrolls: bool,
    boxed: Option<Rems>,
}

impl Collection {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Collection { id: id.into(), children: Vec::new(), scrolls: false, boxed: None }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children.extend(children.into_iter().map(|child| child.into_any_element()));
        self
    }

    /// A group of rows under its heading: "Needs you" and the conversations
    /// that do. A section with no rows is not drawn.
    pub fn section(mut self, header: SectionHeader, rows: impl IntoIterator<Item = impl IntoElement>) -> Self {
        let rows: Vec<AnyElement> = rows.into_iter().map(|row| row.into_any_element()).collect();
        if !rows.is_empty() {
            self.children.push(Section { header, rows }.into_any_element());
        }
        self
    }

    /// Whether the collection scrolls by itself. Off by default: inside a
    /// [`crate::Screen`] the screen scrolls; a column of its own scrolls.
    pub fn scrolls(mut self, scrolls: bool) -> Self {
        self.scrolls = scrolls;
        self
    }

    /// A list inside a form: a framed box of a fixed `height` that scrolls,
    /// so that the form does not change size (and move its buttons) as the
    /// rows change.
    pub fn boxed(mut self, height: Rems) -> Self {
        self.boxed = Some(height);
        self.scrolls = true;
        self
    }
}

impl RenderOnce for Collection {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let list = v_flex()
            .w_full()
            .when(env.input() == InputMode::Pointer, |list| list.p_1().gap_px())
            .children(self.children);
        let list = if self.scrolls { touch::scroll_y(self.id, env, list) } else { list.into_any_element() };
        match self.boxed {
            Some(height) => {
                let scale = cx.scale();
                div().w_full().h(height).border_1().border_color(cx.theme().border).rounded(scale.radius_control).child(list).into_any_element()
            }
            None => list,
        }
    }
}

/// A heading and its rows, kept together.
#[derive(IntoElement)]
struct Section {
    header: SectionHeader,
    rows: Vec<AnyElement>,
}

impl RenderOnce for Section {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        v_flex()
            .w_full()
            .when(env.input() == InputMode::Pointer, |section| section.gap_px())
            .child(self.header)
            .children(self.rows)
    }
}
