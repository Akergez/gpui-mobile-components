//! Pages: side by side when there is room, a stack when there is not.
//!
//! The application describes its pages as [`Route`]s. A route may name its
//! *companion*, the page shown beside it when the window is wide enough: a
//! conversation's companion is the list of conversations, a record's is the
//! records. matterfast's places (a list, a conversation, its thread: on a
//! phone the list is the main screen, as in Telegram) and aiddiction's
//! page stack are this one intent.
//!
//! - **Medium and Expanded**: the page with its companion beside it, and
//!   the companion's companion beside that while there is room: two
//!   columns on a medium window, three on an expanded one (a list, a
//!   conversation, a thread). One column takes what the others leave
//!   ([`Route::grows`], the rightmost by default); the others are
//!   [`Route::column_width`] wide, or as wide as the person dragged them
//!   ([`Navigator::resizable`]). A page that [`Route::floats`] is laid over
//!   the right of the columns instead of taking one: results to glance at.
//! - **Compact**: the top of the stack. A page coming in slides from the
//!   side on a finger and fades in on a pointer. While back would go
//!   somewhere, the navigator is a layer of `gpui_android_glue::back`: the
//!   phone's back gesture moves the page aside with the finger over the page
//!   it goes to (predictive back), and on a finger a swipe to the right does
//!   the same unless the page scrolls sideways ([`Route::scrolls_sideways`]).
//!
//! The state is a plain stack of routes, so a UI scenario can start on any
//! page by building the stack it wants.

use std::rc::Rc;
use std::time::Duration;

use gpui_kit::component::{ActiveTheme, h_flex};
use gpui_kit::prelude::*;
use gpui_kit::{
    Animation, AnimationExt as _, AnyElement, App, Context, ElementId, Entity, EventEmitter,
    Pixels, Render, RenderOnce, Rems, Window, div, ease_out_quint, px, relative, rems,
};

use crate::command::Handler;
use crate::env::{Env, InputMode, WidthClass};
use crate::scale::ActiveScale;

/// A page of the application.
pub trait Route: Clone + PartialEq + 'static {
    /// The page shown beside this one when there is room for both.
    fn companion(&self) -> Option<Self> {
        None
    }

    /// What stands beside this page when there is room and nothing else
    /// does: the "Pick a conversation" of an inbox, the first settings page
    /// beside the list of settings. Only for a page without a companion; on
    /// a compact window it is not shown, and back never goes to it.
    fn wide_detail(&self) -> Option<Self> {
        None
    }

    /// How wide this page is when it is the column beside another.
    fn column_width(&self) -> Rems {
        rems(22.)
    }

    /// Whether this column takes the room the others leave: a conversation
    /// between its list and its thread. When no column shown says so, the
    /// rightmost does.
    fn grows(&self) -> bool {
        false
    }

    /// Whether, when there is room for columns, this page is laid over the
    /// right of them rather than taking one: search results, glanced at and
    /// put away, which should not push the conversation aside. On a compact
    /// window it is a page like any other.
    fn floats(&self) -> bool {
        false
    }

    /// Whether something in the page scrolls sideways (a diff, a board, a
    /// wide table). Such a page gives up the swipe to the right for back,
    /// which would take that direction from it.
    fn scrolls_sideways(&self) -> bool {
        false
    }
}

/// What changed the stack last: decides how the new top comes in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Change {
    Push,
    Pop,
    Replace,
}

/// Emitted after the stack changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigatorEvent {
    Changed,
}

/// The stack of pages. Keep it in the view that owns the window's content.
pub struct NavigatorState<R: Route> {
    stack: Vec<R>,
    /// Counts changes: the key of the transition of the current top.
    generation: usize,
    change: Change,
    back: Option<gpui_android_glue::back::Guard>,
}

impl<R: Route> EventEmitter<NavigatorEvent> for NavigatorState<R> {}

impl<R: Route> NavigatorState<R> {
    pub fn new(root: R) -> Self {
        NavigatorState { stack: vec![root], generation: 0, change: Change::Replace, back: None }
    }

    /// The page on top.
    pub fn top(&self) -> &R {
        self.stack.last().expect("a navigator always has a page")
    }

    pub fn stack(&self) -> &[R] {
        &self.stack
    }

    /// Whether the top is the first page of its section.
    pub fn is_at_root(&self) -> bool {
        self.stack.len() == 1 && self.top().companion().is_none()
    }

    /// Whether back goes anywhere: a page under the top, or the top's
    /// companion.
    pub fn has_back(&self) -> bool {
        self.stack.len() > 1 || self.top().companion().is_some()
    }

    /// The page back goes to.
    pub fn back_target(&self) -> Option<R> {
        if self.stack.len() > 1 {
            self.stack.get(self.stack.len() - 2).cloned()
        } else {
            self.top().companion()
        }
    }

    /// Opens `route` over the top. Pushing the page already on top does
    /// nothing.
    pub fn push(&mut self, route: R, cx: &mut Context<Self>) {
        if self.top() == &route {
            return;
        }
        // A page and its companion are one place when there is room: opening
        // another conversation from the list replaces the conversation.
        if self.stack.len() > 1 && route.companion().is_some() && route.companion() == self.top().companion() {
            self.stack.pop();
        }
        self.stack.push(route);
        self.changed(Change::Push, cx);
    }

    /// Puts `route` in place of the whole stack: choosing a destination.
    pub fn replace_root(&mut self, route: R, cx: &mut Context<Self>) {
        if self.stack.len() == 1 && self.top() == &route {
            return;
        }
        self.stack = vec![route];
        self.changed(Change::Replace, cx);
    }

    /// Keeps only the pages `keep` says yes to: what a closed conversation
    /// does to the history. The first page stays if nothing else would.
    pub fn retain(&mut self, keep: impl Fn(&R) -> bool, cx: &mut Context<Self>) {
        let before = self.stack.len();
        let first = self.stack[0].clone();
        let top = self.top().clone();
        self.stack.retain(|route| keep(route));
        if self.stack.is_empty() {
            self.stack.push(first);
        }
        if self.stack.len() != before {
            let change = if self.top() == &top { Change::Replace } else { Change::Pop };
            self.changed(change, cx);
        }
    }

    /// Makes the stack `stack`, as the application's own state says what is
    /// open: a conversation, a thread over it. Nothing happens if it is
    /// already so; a longer stack with the same start comes in as a push, a
    /// shorter one as a pop.
    pub fn set_stack(&mut self, stack: Vec<R>, cx: &mut Context<Self>) {
        assert!(!stack.is_empty(), "a navigator always has a page");
        if stack == self.stack {
            return;
        }
        let change = if stack.len() > self.stack.len() && stack.starts_with(&self.stack) {
            Change::Push
        } else if stack.len() < self.stack.len() && self.stack.starts_with(&stack) {
            Change::Pop
        } else {
            Change::Replace
        };
        self.stack = stack;
        self.changed(change, cx);
    }

    /// How far the page in front has been moved aside by a gesture of back
    /// under way, as a share of its width; none without one. For a UI
    /// scenario to check that the page follows the finger.
    pub fn gesture_offset(&self, window: &mut Window, cx: &mut App) -> Option<f32> {
        self.back.as_ref()?.gesture(window, cx).map(|gesture| gesture.offset())
    }

    /// Goes back a page, answering whether there was one.
    pub fn back(&mut self, cx: &mut Context<Self>) -> bool {
        if self.stack.len() > 1 {
            self.stack.pop();
        } else if let Some(companion) = self.top().companion() {
            self.stack = vec![companion];
        } else {
            return false;
        }
        self.changed(Change::Pop, cx);
        true
    }

    fn changed(&mut self, change: Change, cx: &mut Context<Self>) {
        self.generation += 1;
        self.change = change;
        cx.emit(NavigatorEvent::Changed);
        cx.notify();
    }

    /// Keeps a layer of back exactly while back goes somewhere in a stack.
    fn sync_back(&mut self, stacked: bool, cx: &mut Context<Self>) {
        let wanted = stacked && self.has_back();
        if wanted == self.back.is_some() {
            return;
        }
        self.back = wanted.then(|| {
            let this = cx.weak_entity();
            gpui_android_glue::back::push(move |_, cx| {
                this.update(cx, |navigator, cx| navigator.back(cx)).unwrap_or(false)
            })
        });
    }
}

/// How a page is shown: what [`Navigator`] tells the page it draws.
#[derive(Clone)]
pub struct Page {
    up: Option<Handler>,
    column: bool,
}

impl Page {
    /// The way up, if the page has one where it is shown: give it to
    /// [`crate::Screen::up`]. None for a page with its companion beside it.
    pub fn up(&self) -> Option<Handler> {
        self.up.clone()
    }

    /// Whether the page is the column beside another, rather than the page
    /// in front.
    pub fn is_column(&self) -> bool {
        self.column
    }
}

type RenderPage<R> = Rc<dyn Fn(&R, Page, &mut Window, &mut App) -> AnyElement>;

/// How wide a column is, given the window's width.
type WidthOf<R> = Rc<dyn Fn(&R, Pixels, &App) -> Pixels>;

/// A column dragged to a width, or given back its own (`None`).
type OnResize<R> = Rc<dyn Fn(&R, Option<Pixels>, &mut App)>;

/// Draws a [`NavigatorState`]'s pages.
#[derive(IntoElement)]
pub struct Navigator<R: Route> {
    id: ElementId,
    state: Entity<NavigatorState<R>>,
    render_page: RenderPage<R>,
    resizable: Option<(WidthOf<R>, OnResize<R>)>,
}

impl<R: Route> Navigator<R> {
    /// `render_page` draws one route; it may be called for the page under
    /// the top too, while a back gesture shows it, so it must not change
    /// anything.
    pub fn new(
        id: impl Into<ElementId>,
        state: &Entity<NavigatorState<R>>,
        render_page: impl Fn(&R, Page, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Navigator { id: id.into(), state: state.clone(), render_page: Rc::new(render_page), resizable: None }
    }

    /// Lets the person size the columns that do not grow by dragging their
    /// edge; a double click gives a column back its own width. `width_of`
    /// says how wide a column is (what it was dragged to, kept within
    /// reason, or a share of the window), `on_resize` hears a drag.
    pub fn resizable(
        mut self,
        width_of: impl Fn(&R, Pixels, &App) -> Pixels + 'static,
        on_resize: impl Fn(&R, Option<Pixels>, &mut App) + 'static,
    ) -> Self {
        self.resizable = Some((Rc::new(width_of), Rc::new(on_resize)));
        self
    }
}

/// How many columns a window of this class has room for.
fn room(width: WidthClass) -> usize {
    match width {
        WidthClass::Compact => 1,
        WidthClass::Medium => 2,
        WidthClass::Expanded => 3,
    }
}

/// What is shown when there is room for `room` columns: the columns, left
/// to right, and the page laid over them. One column and nothing over it
/// is the stack.
fn layout<R: Route>(top: &R, room: usize) -> (Vec<R>, Option<R>) {
    let floating = (room > 1 && top.floats() && top.companion().is_some()).then(|| top.clone());
    let mut columns = Vec::new();
    let mut next = match &floating {
        Some(top) => top.companion(),
        None => Some(top.clone()),
    };
    while let Some(route) = next {
        if columns.len() == room {
            break;
        }
        next = route.companion();
        columns.push(route);
    }
    columns.reverse();
    if room > 1
        && let [only] = columns.as_slice()
        && let Some(detail) = only.wide_detail()
    {
        columns.push(detail);
    }
    (columns, floating)
}

/// The edge of a column being dragged: its place among the columns.
#[derive(Clone, Copy)]
struct ColumnEdge(usize);

/// What follows the pointer while an edge is dragged: nothing; the column
/// moving is the feedback.
struct EdgeGhost;

impl Render for EdgeGhost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui_kit::Empty
    }
}

/// How wide the strip is that drags a column's edge, over the border.
const GRIP: f32 = 7.;

/// The strip over a column's edge, `at` from the left or the right of the
/// columns. Draws nothing until the pointer is on it.
fn edge(index: usize, at: Pixels, from_left: bool, on_reset: Handler, focus: gpui_kit::Hsla) -> AnyElement {
    let strip = div()
        .id(("navigator-edge", index))
        .absolute()
        .top_0()
        .bottom_0()
        .w(px(GRIP))
        .flex()
        .justify_center()
        .cursor_col_resize()
        .occlude()
        .group("navigator-edge")
        .child(div().w(px(2.)).h_full().group_hover("navigator-edge", move |mark| mark.bg(focus)))
        .on_drag(ColumnEdge(index), |_, _, _, cx| cx.new(|_| EdgeGhost))
        .on_click(move |click, window, cx| {
            if click.click_count() == 2 {
                on_reset(window, cx);
            }
        });
    let at = at - px(GRIP / 2.);
    if from_left { strip.left(at) } else { strip.right(at) }.into_any_element()
}

impl<R: Route> RenderOnce for Navigator<R> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let state = self.state.read(cx);
        let top = state.top().clone();
        let (columns, floating) = layout(&top, room(env.width()));
        let stacked = columns.len() == 1 && floating.is_none();
        let (generation, change) = (state.generation, state.change);
        let back_target = state.back_target();
        self.state.update(cx, |state, cx| state.sync_back(stacked, cx));

        let weak = self.state.downgrade();
        let up: Handler = Rc::new(move |_, cx| {
            weak.update(cx, |navigator, cx| {
                navigator.back(cx);
            })
            .ok();
        });

        if !stacked {
            return self.columns(columns, floating, up, window, cx);
        }

        let has_back = back_target.is_some();
        let page = (self.render_page)(
            &top,
            Page { up: has_back.then(|| up.clone()), column: false },
            window,
            cx,
        );
        let page = transition(self.id.clone(), generation, change, env, page, cx);
        let gesture = self.state.update(cx, |navigator, cx| {
            navigator.back.as_ref().and_then(|guard| guard.gesture(window, cx))
        });
        // What back goes to is drawn only while a gesture shows it.
        let under = gesture
            .and(back_target)
            .map(|target| (self.render_page)(&target, Page { up: None, column: false }, window, cx));
        gpui_android_glue::back::preview(div().id(self.id).size_full().child(page), gesture)
            .under(under)
            .swipe_back(env.is_touch() && has_back && !top.scrolls_sideways())
            .into_any_element()
    }
}

impl<R: Route> Navigator<R> {
    /// The pages side by side, and the one laid over them.
    fn columns(self, columns: Vec<R>, floating: Option<R>, up: Handler, window: &mut Window, cx: &mut App) -> AnyElement {
        let (border, focus) = {
            let theme = cx.theme();
            (theme.border, theme.ring)
        };
        let window_width = window.viewport_size().width;
        let grows = columns.iter().position(Route::grows).unwrap_or(columns.len() - 1);
        let width = |route: &R, cx: &App| -> gpui_kit::AbsoluteLength {
            match &self.resizable {
                Some((width_of, _)) => width_of(route, window_width, cx).into(),
                None => route.column_width().into(),
            }
        };
        // The edges that can be dragged, and how far each is from the side
        // of the columns it is measured from: the sum of the fixed widths
        // between it and that side. Only with widths in pixels.
        let pixels = |route: &R, cx: &App| match &self.resizable {
            Some((width_of, _)) => width_of(route, window_width, cx),
            None => px(0.),
        };
        let mut edges = Vec::new();
        if let Some((_, on_resize)) = &self.resizable {
            let reset = |route: &R| -> Handler {
                let (route, on_resize) = (route.clone(), on_resize.clone());
                Rc::new(move |_, cx| on_resize(&route, None, cx))
            };
            let mut at = px(0.);
            for (index, route) in columns.iter().enumerate().take(grows) {
                at += pixels(route, cx);
                edges.push(edge(index, at, true, reset(route), focus));
            }
            let mut at = px(0.);
            for (index, route) in columns.iter().enumerate().skip(grows + 1).rev() {
                at += pixels(route, cx);
                edges.push(edge(index, at, false, reset(route), focus));
            }
            if let Some(over) = &floating {
                edges.push(edge(columns.len(), pixels(over, cx), false, reset(over), focus));
            }
        }

        let mut row = h_flex().id(self.id).relative().size_full();
        for (index, route) in columns.iter().enumerate() {
            let page = (self.render_page)(route, Page { up: None, column: index != grows }, window, cx);
            let column = div().h_full().child(page);
            row = row.child(match index.cmp(&grows) {
                std::cmp::Ordering::Less => column.flex_none().w(width(route, cx)).border_r_1().border_color(border),
                std::cmp::Ordering::Equal => column.flex_1().min_w_0(),
                std::cmp::Ordering::Greater => column.flex_none().w(width(route, cx)).border_l_1().border_color(border),
            });
        }
        if let Some(over) = &floating {
            let page = (self.render_page)(over, Page { up: Some(up), column: true }, window, cx);
            row = row.child(
                div().absolute().top_0().bottom_0().right_0().w(width(over, cx)).shadow_lg().occlude().child(page),
            );
        }
        if let Some((width_of, on_resize)) = self.resizable.clone() {
            // The order of the edges' indices: the columns, then the page over them.
            let routes: Vec<R> = columns.iter().cloned().chain(floating.clone()).collect();
            let count = columns.len();
            row = row.on_drag_move(move |moved: &gpui_kit::DragMoveEvent<ColumnEdge>, _, cx| {
                let ColumnEdge(index) = *moved.drag(cx);
                let Some(route) = routes.get(index) else { return };
                let sum = |routes: &[R], cx: &App| {
                    routes.iter().map(|route| width_of(route, window_width, cx)).fold(px(0.), |a, b| a + b)
                };
                let pointer = moved.event.position.x;
                // The columns between the dragged edge and its side keep
                // their widths.
                let dragged = if index < grows {
                    pointer - moved.bounds.left() - sum(&routes[..index], cx)
                } else if index < count {
                    moved.bounds.right() - sum(&routes[index + 1..count], cx) - pointer
                } else {
                    moved.bounds.right() - pointer
                };
                on_resize(route, Some(dragged.round()), cx);
            });
        }
        row.children(edges).into_any_element()
    }
}

/// The page on top, coming in: from the side and fading in on a finger, a
/// short fade on a pointer. Keyed by the change, so it plays once per change.
fn transition(id: ElementId, generation: usize, change: Change, env: Env, page: AnyElement, cx: &App) -> AnyElement {
    let duration: Duration = cx.scale().page;
    if generation == 0 || change == Change::Replace || duration.is_zero() {
        return page;
    }
    let slide = env.input() == InputMode::Touch;
    let from = match change {
        Change::Push => 0.12,
        Change::Pop => -0.06,
        Change::Replace => 0.,
    };
    div()
        .relative()
        .size_full()
        .child(page)
        .with_animation(
            ElementId::NamedInteger(format!("{id:?}-page").into(), generation as u64),
            Animation::new(duration).with_easing(ease_out_quint()),
            move |element, delta| {
                let element = element.opacity(0.4 + 0.6 * delta);
                if slide { element.left(relative(from * (1. - delta))) } else { element }
            },
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Mail {
        Inbox,
        Message(u32),
        Settings,
    }

    impl Route for Mail {
        fn companion(&self) -> Option<Self> {
            match self {
                Mail::Message(_) => Some(Mail::Inbox),
                _ => None,
            }
        }
    }

    /// A list, a conversation, a thread over it, results laid over it.
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Chat {
        List,
        Conversation,
        Thread,
        Search,
    }

    impl Route for Chat {
        fn companion(&self) -> Option<Self> {
            match self {
                Chat::List => None,
                Chat::Conversation => Some(Chat::List),
                Chat::Thread | Chat::Search => Some(Chat::Conversation),
            }
        }

        fn wide_detail(&self) -> Option<Self> {
            (*self == Chat::List).then_some(Chat::Conversation)
        }

        fn grows(&self) -> bool {
            *self == Chat::Conversation
        }

        fn floats(&self) -> bool {
            *self == Chat::Search
        }
    }

    #[test]
    fn as_many_columns_as_there_is_room_for_from_the_top_down() {
        assert_eq!(layout(&Chat::Thread, 3), (vec![Chat::List, Chat::Conversation, Chat::Thread], None));
        assert_eq!(layout(&Chat::Thread, 2), (vec![Chat::Conversation, Chat::Thread], None));
        assert_eq!(layout(&Chat::Thread, 1), (vec![Chat::Thread], None));
        // Nothing chosen yet: what stands beside the list where there is room.
        assert_eq!(layout(&Chat::List, 3), (vec![Chat::List, Chat::Conversation], None));
        assert_eq!(layout(&Chat::List, 1), (vec![Chat::List], None));
    }

    #[test]
    fn a_floating_page_lies_over_its_columns_and_is_a_page_on_a_phone() {
        assert_eq!(layout(&Chat::Search, 3), (vec![Chat::List, Chat::Conversation], Some(Chat::Search)));
        assert_eq!(layout(&Chat::Search, 1), (vec![Chat::Search], None));
    }

    #[gpui_kit::test]
    fn a_stack_set_from_the_applications_state_comes_in_as_it_changed(cx: &mut gpui_kit::TestAppContext) {
        let navigator = cx.new(|_| NavigatorState::new(Chat::List));
        navigator.update(cx, |navigator, cx| {
            navigator.set_stack(vec![Chat::List, Chat::Conversation, Chat::Thread], cx);
            assert_eq!(navigator.change, Change::Push);
            let generation = navigator.generation;
            navigator.set_stack(vec![Chat::List, Chat::Conversation, Chat::Thread], cx);
            assert_eq!(navigator.generation, generation, "the same stack changes nothing");
            navigator.set_stack(vec![Chat::List, Chat::Conversation], cx);
            assert_eq!(navigator.change, Change::Pop);
            assert!(navigator.back(cx));
            assert_eq!(navigator.stack(), &[Chat::List]);
        });
    }

    #[gpui_kit::test]
    fn back_pops_then_falls_back_to_the_companion(cx: &mut gpui_kit::TestAppContext) {
        let navigator = cx.new(|_| NavigatorState::new(Mail::Inbox));
        navigator.update(cx, |navigator, cx| {
            assert!(navigator.is_at_root());
            assert!(!navigator.has_back());
            navigator.push(Mail::Message(1), cx);
            assert_eq!(navigator.back_target(), Some(Mail::Inbox));
            assert!(navigator.back(cx));
            assert_eq!(navigator.top(), &Mail::Inbox);
            assert!(!navigator.back(cx));

            // Opened straight onto a message: back goes to its companion.
            navigator.replace_root(Mail::Message(7), cx);
            assert!(navigator.has_back());
            assert!(navigator.back(cx));
            assert_eq!(navigator.stack(), &[Mail::Inbox]);
        });
    }

    #[gpui_kit::test]
    fn retain_drops_closed_pages_and_keeps_a_first_page(cx: &mut gpui_kit::TestAppContext) {
        let navigator = cx.new(|_| NavigatorState::new(Mail::Inbox));
        navigator.update(cx, |navigator, cx| {
            navigator.push(Mail::Settings, cx);
            navigator.push(Mail::Message(3), cx);
            navigator.retain(|route| route != &Mail::Message(3), cx);
            assert_eq!(navigator.stack(), &[Mail::Inbox, Mail::Settings]);
            navigator.retain(|_| false, cx);
            assert_eq!(navigator.stack(), &[Mail::Inbox]);
        });
    }

    #[gpui_kit::test]
    fn opening_a_sibling_replaces_the_page_beside_the_same_companion(cx: &mut gpui_kit::TestAppContext) {
        let navigator = cx.new(|_| NavigatorState::new(Mail::Inbox));
        navigator.update(cx, |navigator, cx| {
            navigator.push(Mail::Message(1), cx);
            navigator.push(Mail::Message(2), cx);
            assert_eq!(navigator.stack(), &[Mail::Inbox, Mail::Message(2)]);
            navigator.push(Mail::Message(2), cx);
            assert_eq!(navigator.stack().len(), 2);
            navigator.replace_root(Mail::Settings, cx);
            assert_eq!(navigator.stack(), &[Mail::Settings]);
        });
    }
}
