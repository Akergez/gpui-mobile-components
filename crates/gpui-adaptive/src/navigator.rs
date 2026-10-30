//! Pages: side by side when there is room, a stack when there is not.
//!
//! The application describes its pages as [`Route`]s. A route may name its
//! *companion*, the page shown beside it when the window is wide enough: a
//! conversation's companion is the list of conversations, a record's is the
//! records. matterfast's `layout/split.rs` ("on a phone the list is the main
//! screen, as it is in Telegram: a conversation is a page laid over it") and
//! aiddiction's page stack are this one intent.
//!
//! - **Medium and Expanded**: the companion as a column of
//!   [`Route::column_width`], the page beside it.
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
    RenderOnce, Rems, Window, div, ease_out_quint, relative, rems,
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

/// Draws a [`NavigatorState`]'s pages.
#[derive(IntoElement)]
pub struct Navigator<R: Route> {
    id: ElementId,
    state: Entity<NavigatorState<R>>,
    render_page: RenderPage<R>,
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
        Navigator { id: id.into(), state: state.clone(), render_page: Rc::new(render_page) }
    }
}

impl<R: Route> RenderOnce for Navigator<R> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let state = self.state.read(cx);
        let top = state.top().clone();
        let roomy = env.width() != WidthClass::Compact;
        // A page and its companion beside it, or a page and what stands beside
        // it when nothing else does.
        let pair = match (top.companion(), top.wide_detail()) {
            (Some(companion), _) if roomy => Some((companion, top.clone())),
            (None, Some(detail)) if roomy => Some((top.clone(), detail)),
            _ => None,
        };
        let columns = pair.is_some();
        let (generation, change) = (state.generation, state.change);
        let back_target = state.back_target();
        self.state.update(cx, |state, cx| state.sync_back(!columns, cx));

        let weak = self.state.downgrade();
        let up: Handler = Rc::new(move |_, cx| {
            weak.update(cx, |navigator, cx| {
                navigator.back(cx);
            })
            .ok();
        });

        if let Some((companion, top)) = pair {
            let theme = cx.theme();
            let border = theme.border;
            let width = companion.column_width();
            let column = (self.render_page)(&companion, Page { up: None, column: true }, window, cx);
            let page = (self.render_page)(&top, Page { up: None, column: false }, window, cx);
            return h_flex()
                .id(self.id)
                .size_full()
                .child(div().flex_none().w(width).h_full().border_r_1().border_color(border).child(column))
                .child(div().flex_1().min_w_0().h_full().child(page))
                .into_any_element();
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
