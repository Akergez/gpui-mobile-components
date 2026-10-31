//! A conversation as it is read: a [`Feed`] of [`Message`]s, newest at the
//! bottom, that follows what comes in; and a [`Fold`], a quiet line that
//! unfolds into what most readers skip (an agent's thinking).
//!
//! **Pointer**: the feed reads in a column at most [`READING_WIDTH`] wide,
//! centred in the page, with gpui-kit's scrollbar. Your own message is a
//! bubble at the end in the secondary colour; the other side's is text on
//! the page, in Markdown.
//! **Touch**: the same column, edge to edge on a phone; your own bubble is
//! Material's primary container.
//!
//! The feed is virtual: only what is on screen is drawn, so a long
//! conversation costs what a short one does. The owner keeps its
//! [`ListState`] (from [`Feed::state`]) and tells it what changed with
//! `splice`, as with gpui's `list`.

use std::rc::Rc;
use std::sync::Arc;

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::{ActiveTheme, Icon, IconName, Sizable, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, App, ElementId, FollowMode, ListAlignment, ListState, RenderOnce, SharedString, Window, div, list, px, relative,
    rems,
};

use crate::banner::Tone;
use crate::command::Handler;
use crate::empty::Empty;
use crate::env::{Env, InputMode};
use crate::palette::ActivePalette;
use crate::prim::Markdown;
use crate::scale::ActiveScale;

/// How wide a conversation reads at most, in rems.
pub const READING_WIDTH: f32 = 48.;

/// What a link in a message opens; gpui-kit's text view wants it `Send`.
type OnLink = dyn Fn(&str, &mut Window, &mut App) + Send + Sync;

type RenderItem = Rc<dyn Fn(usize, &mut Window, &mut App) -> AnyElement>;

/// The messages of a conversation, newest at the bottom.
#[derive(IntoElement)]
pub struct Feed {
    state: ListState,
    render_item: RenderItem,
    empty: Option<Empty>,
}

impl Feed {
    /// A feed's state: aligned to the bottom, following the newest message
    /// until the reader scrolls away from it.
    pub fn state() -> ListState {
        let state = ListState::new(0, ListAlignment::Bottom, px(400.));
        state.set_follow_mode(FollowMode::Tail);
        state
    }

    /// `render_item` draws the item at an index; an item with nothing to
    /// show (a step drawn by the one before it) is an empty element.
    pub fn new(state: &ListState, render_item: impl Fn(usize, &mut Window, &mut App) -> AnyElement + 'static) -> Self {
        Feed { state: state.clone(), render_item: Rc::new(render_item), empty: None }
    }

    /// What the feed says while it has no messages.
    pub fn empty(mut self, empty: Option<Empty>) -> Self {
        self.empty = empty;
        self
    }
}

impl RenderOnce for Feed {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let render_item = self.render_item.clone();
        let rows = list(self.state.clone(), move |index, window, cx| {
            let item = render_item(index, window, cx);
            h_flex()
                .w_full()
                .justify_center()
                .child(div().w_full().max_w(rems(READING_WIDTH)).px_4().py_1p5().child(item))
                .into_any_element()
        })
        .size_full()
        .py_3();
        div()
            .relative()
            .flex_1()
            .min_h_0()
            .w_full()
            .children(self.empty.map(|empty| div().absolute().inset_0().child(empty)))
            .child(div().size_full().child(rows).vertical_scrollbar(&self.state))
    }
}

/// Whose a message is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Mine,
    Theirs,
    Aside,
}

/// One message in a [`Feed`].
#[derive(IntoElement)]
pub struct Message {
    id: ElementId,
    side: Side,
    text: SharedString,
    tone: Option<Tone>,
    before: Option<AnyElement>,
    on_link: Option<Arc<OnLink>>,
}

impl Message {
    /// What the person wrote: a bubble at the end.
    pub fn mine(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self::of(id, Side::Mine, text)
    }

    /// What the other side wrote, in Markdown. Its text can be selected.
    pub fn theirs(id: impl Into<ElementId>, markdown: impl Into<SharedString>) -> Self {
        Self::of(id, Side::Theirs, markdown)
    }

    /// Something said about the conversation, not in it: "The agent
    /// stopped." Centred and small; [`Message::tone`] colours it.
    pub fn aside(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self::of(id, Side::Aside, text)
    }

    fn of(id: impl Into<ElementId>, side: Side, text: impl Into<SharedString>) -> Self {
        Message { id: id.into(), side, text: text.into(), tone: None, before: None, on_link: None }
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }

    /// What stands before the text: a [`Fold`] with the thinking that led to
    /// it.
    pub fn before(mut self, before: Option<impl IntoElement>) -> Self {
        self.before = before.map(IntoElement::into_any_element);
        self
    }

    /// What a link in the text opens.
    pub fn on_link(mut self, on_link: impl Fn(&str, &mut Window, &mut App) + Send + Sync + 'static) -> Self {
        self.on_link = Some(Arc::new(on_link));
        self
    }
}

impl RenderOnce for Message {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        match self.side {
            Side::Mine => {
                let (bg, fg) = match env.input() {
                    InputMode::Pointer => (cx.theme().secondary, cx.theme().secondary_foreground),
                    InputMode::Touch => (cx.palette().primary_container, cx.palette().on_primary_container),
                };
                h_flex()
                    .id(self.id)
                    .justify_end()
                    .child(div().max_w(relative(0.85)).px_4().py_2p5().rounded(px(18.)).bg(bg).text_color(fg).child(self.text))
                    .into_any_element()
            }
            Side::Theirs => {
                let text = (!self.text.is_empty()).then(|| {
                    let markdown = Markdown::new(self.id.clone(), self.text.clone());
                    match self.on_link.clone() {
                        Some(on_link) => markdown.on_link(move |url, window, cx| on_link(url, window, cx)),
                        None => markdown,
                    }
                });
                v_flex().gap_1().children(self.before).children(text).into_any_element()
            }
            Side::Aside => {
                let color = self.tone.map_or(cx.theme().muted_foreground, |tone| tone.color(cx));
                div()
                    .id(self.id)
                    .w_full()
                    .text_center()
                    .text_size(scale.small)
                    .text_color(color)
                    .child(self.text)
                    .into_any_element()
            }
        }
    }
}

/// A quiet line that unfolds into what it stands for: "Thinking".
/// Controlled: the owner keeps whether it is open.
#[derive(IntoElement)]
pub struct Fold {
    id: ElementId,
    label: SharedString,
    open: bool,
    on_toggle: Option<Handler>,
    children: Vec<AnyElement>,
}

impl Fold {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Fold { id: id.into(), label: label.into(), open: false, on_toggle: None, children: Vec::new() }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn on_toggle(mut self, on_toggle: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(on_toggle));
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for Fold {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let on_toggle = self.on_toggle.clone();
        let line = h_flex()
            .id(self.id.clone())
            .test_support()
            .gap_1()
            .cursor_pointer()
            .when(env.is_touch(), |line| line.min_h(scale.target))
            .child(Icon::new(if self.open { IconName::ChevronDown } else { IconName::ChevronRight }).xsmall())
            .child(self.label)
            .when_some(on_toggle, |line, on_toggle| line.on_click(move |_, window, cx| on_toggle(window, cx)));
        v_flex()
            .text_size(scale.small)
            .text_color(cx.theme().muted_foreground)
            .child(line)
            .when(self.open, |fold| fold.child(div().pl_4().children(self.children)))
    }
}
