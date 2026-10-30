//! One or several things out of a directory too large to list: people,
//! channels (matterfast's `dialogs/picker.rs`).
//!
//! The owner answers every query: it subscribes to [`LookupEvent::Query`],
//! searches (on the server, in a cache) and hands the results back with
//! [`LookupState::set_results`]. What has been chosen stays above the
//! results as chips, each with a cross to take it back.
//!
//! **Pointer**: a dialog with the search field, the chips and the results;
//! Done commits.
//! **Touch**: a bottom sheet, the field first and focused, above the
//! keyboard; a single choice commits at once, several commit with Done;
//! back cancels.

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{ActiveTheme, Icon, IconName, Sizable, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{App, Context, ElementId, Focusable as _, Entity, EventEmitter, Render, SharedString, Subscription, Window, div};

use crate::choice::ChoiceOption;
use crate::env::Env;
use crate::palette::ActivePalette;
use crate::scale::ActiveScale;
use crate::sheet::Sheet;
use crate::touch;

/// What a lookup tells its owner.
#[derive(Clone, Debug, PartialEq)]
pub enum LookupEvent {
    /// Search for this: answer with [`LookupState::set_results`].
    Query(SharedString),
    /// The person is done: these keys were chosen.
    Confirm(Vec<SharedString>),
}

/// The retained state of a lookup.
pub struct LookupState {
    title: SharedString,
    query: Entity<InputState>,
    results: Vec<ChoiceOption>,
    chosen: Vec<ChoiceOption>,
    multiple: bool,
    searching: bool,
    _subscription: Subscription,
}

impl EventEmitter<LookupEvent> for LookupState {}

impl LookupState {
    pub fn new(title: impl Into<SharedString>, multiple: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let query = cx.new(|cx| InputState::new(window, cx).placeholder("Search"));
        let subscription = cx.subscribe(&query, |this: &mut Self, query, event: &InputEvent, cx| {
            gpui_android_glue::keyboard::follow(query.entity_id(), event, gpui_android_glue::keyboard::Purpose::Plain);
            if matches!(event, InputEvent::Change) {
                this.searching = true;
                cx.emit(LookupEvent::Query(query.read(cx).value()));
                cx.notify();
            }
        });
        LookupState {
            title: title.into(),
            query,
            results: Vec::new(),
            chosen: Vec::new(),
            multiple,
            searching: false,
            _subscription: subscription,
        }
    }

    /// The answer to the last query.
    pub fn set_results(&mut self, results: Vec<ChoiceOption>, cx: &mut Context<Self>) {
        self.results = results;
        self.searching = false;
        cx.notify();
    }

    pub fn chosen(&self) -> &[ChoiceOption] {
        &self.chosen
    }

    fn toggle(&mut self, option: ChoiceOption, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(at) = self.chosen.iter().position(|chosen| chosen.key() == option.key()) {
            self.chosen.remove(at);
        } else if self.multiple {
            self.chosen.push(option);
        } else {
            self.chosen = vec![option];
            self.confirm(window, cx);
            return;
        }
        cx.notify();
    }

    fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(LookupEvent::Confirm(self.chosen.iter().map(|option| option.key().clone()).collect()));
        Sheet::close(window, cx);
    }
}

/// Opening a lookup.
pub struct Lookup;

impl Lookup {
    /// Opens `state` over the page, its field focused.
    pub fn open(state: &Entity<LookupState>, window: &mut Window, cx: &mut App) {
        let view = cx.new(|_| LookupView { state: state.clone() });
        Sheet::open(view, window, cx);
        let focus = state.read(cx).query.read(cx).focus_handle(cx);
        focus.focus(window, cx);
    }
}

struct LookupView {
    state: Entity<LookupState>,
}

impl Render for LookupView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let palette = cx.palette();
        let muted = cx.theme().muted_foreground;
        let state = self.state.read(cx);
        let entity = self.state.clone();
        let done = state.multiple.then(|| {
            let entity = entity.clone();
            Button::new("lookup-done")
                .label("Done")
                .primary()
                .with_size(if env.is_touch() { gpui_kit::component::Size::Large } else { gpui_kit::component::Size::Small })
                .on_click(move |_, window, cx| entity.update(cx, |state, cx| state.confirm(window, cx)))
        });
        let chips = state.chosen.iter().map(|option| {
            let (entity, option_) = (entity.clone(), option.clone());
            h_flex()
                .id((ElementId::from("lookup-chip"), option.key().clone()))
                .h(if env.is_touch() { gpui_kit::rems(2.) } else { gpui_kit::rems(1.5) })
                .px_2()
                .gap_1()
                .rounded(scale.radius_control)
                .bg(palette.secondary_container)
                .text_color(palette.on_secondary_container)
                .text_size(scale.small)
                .cursor_pointer()
                .child(option.label().clone())
                .child(Icon::new(IconName::Close).xsmall())
                .on_click(move |_, window, cx| entity.update(cx, |state, cx| state.toggle(option_.clone(), window, cx)))
                .test_support()
        });
        let rows = state.results.iter().map(|option| {
            let chosen = state.chosen.iter().any(|chosen| chosen.key() == option.key());
            let (entity, option_) = (entity.clone(), option.clone());
            touch::pressable(ElementId::Name(option.key().clone()), env, cx)
                .min_h(scale.row)
                .px_4()
                .gap_3()
                .child(div().flex_1().truncate().text_size(scale.body).child(option.label().clone()))
                .when(chosen, |row| row.child(Icon::new(IconName::Check)))
                .on_click(move |_, window, cx| entity.update(cx, |state, cx| state.toggle(option_.clone(), window, cx)))
                .test_support()
        });
        let empty = (state.results.is_empty() && !state.searching).then(|| {
            div().px_4().py_6().text_size(scale.small).text_color(muted).child("Type to search")
        });
        v_flex()
            .min_w(gpui_kit::rems(24.))
            .flex_grow(1.)
            .min_h_0()
            .gap_2()
            .child(
                h_flex()
                    .px_4()
                    .gap_2()
                    .child(div().flex_1().text_size(scale.title).child(state.title.clone()))
                    .children(done),
            )
            .child(div().px_4().child({
                let input = Input::new(&state.query).prefix(Icon::new(IconName::Search));
                if env.is_touch() { input.large() } else { input.small() }
            }))
            .when(!state.chosen.is_empty(), |view| view.child(h_flex().px_4().gap_1().flex_wrap().children(chips)))
            .child(
                div()
                    .id("lookup-results")
                    .flex_grow(1.)
                    .min_h(gpui_kit::rems(12.))
                    .overflow_y_scroll()
                    .child(v_flex().children(rows).children(empty)),
            )
    }
}
