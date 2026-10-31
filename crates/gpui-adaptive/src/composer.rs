//! Writing a message: to someone in a channel, or to an agent.
//!
//! A text area that grows with what is written (up to eight lines), the
//! tools around it (attach, priority, schedule) and send. A tool may be a
//! menu that shows what is chosen ("Ask first ▾", the model): the agent's
//! mode. Things can stand above it: a reply-to line, an agent's plan, the
//! commands matching what is typed. While the other side works, send is a
//! stop button ([`Composer::stop`]); while it cannot take a message, send
//! is disabled ([`Composer::can_send`]).
//!
//! One card on a pointer and on a finger: what stands above, the field, and
//! under it the tools and send.
//! **Pointer**: a hint about the keys beside the tools; a menu opens under
//! its button. Enter sends and Shift+Enter breaks the line.
//! **Touch**: no hint; a menu opens as a sheet. The composer rides on the
//! keyboard (it is padded by [`crate::Insets::typing`] by whoever holds the
//! page). Enter breaks the line and the send button sends, as on a phone;
//! the keyboard is for prose.
//!
//! The draft is the owner's: [`ComposerState::text`] reads it, `clear`
//! empties it after sending.

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::input::{InputEvent, RopeExt as _, Textarea, TextareaState};
use gpui_kit::component::{IconName, Sizable};
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, App, Context, ElementId, Entity, EventEmitter, FocusHandle, Focusable, RenderOnce, SharedString,
    Subscription, Window, div, rems,
};

use std::rc::Rc;

use crate::colors::ActiveColors;
use crate::command::{Command, Handler};
use crate::env::{Env, InputMode, input_mode};
use crate::prim::{Surface, Text, column, pressable, row, surface, testable};
use crate::scale::ActiveScale;
use crate::sheet::Sheet;

/// What a composer tells its owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComposerEvent {
    /// The draft changed.
    Change,
    /// The person sent the draft.
    Send,
}

/// The retained state of a composer: the draft.
pub struct ComposerState {
    text: Entity<TextareaState>,
    _subscription: Subscription,
}

impl EventEmitter<ComposerEvent> for ComposerState {}

impl ComposerState {
    pub fn new(placeholder: impl Into<SharedString>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let placeholder: SharedString = placeholder.into();
        let text = cx.new(|cx| TextareaState::new(window, cx).auto_grow(1, 8).placeholder(placeholder));
        let subscription = cx.subscribe(&text, |_: &mut Self, text, event: &InputEvent, cx| {
            gpui_android_glue::keyboard::follow(
                text.entity_id(),
                event,
                gpui_android_glue::keyboard::Purpose::Message,
            );
            match event {
                InputEvent::Change => cx.emit(ComposerEvent::Change),
                // On a pointer Enter sends; Shift+Enter is a new line, which
                // the text area inserts itself.
                InputEvent::PressEnter { shift: false, secondary: false }
                    if input_mode(cx) == InputMode::Pointer =>
                {
                    cx.emit(ComposerEvent::Send)
                }
                _ => {}
            }
        });
        ComposerState { text, _subscription: subscription }
    }

    /// The draft, trimmed of the line break Enter may have left.
    pub fn text(&self, cx: &App) -> String {
        self.text.read(cx).value().trim_end_matches('\n').to_string()
    }

    pub fn is_empty(&self, cx: &App) -> bool {
        self.text(cx).trim().is_empty()
    }

    pub fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.text.update(cx, |text, cx| text.set_value("", window, cx));
    }

    /// Replaces the draft (a picked command) and puts the cursor at its end,
    /// where the person goes on typing.
    pub fn set_text(&mut self, text: impl Into<SharedString>, window: &mut Window, cx: &mut Context<Self>) {
        let text: SharedString = text.into();
        self.text.update(cx, |area, cx| {
            area.set_value(text, window, cx);
            let end = area.text().offset_to_position(area.text().len());
            area.set_cursor_position(end, window, cx);
        });
    }

    fn send(&mut self, cx: &mut Context<Self>) {
        cx.emit(ComposerEvent::Send);
    }
}

impl Focusable for ComposerState {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.text.read(cx).focus_handle(cx)
    }
}

/// A tool on the composer.
#[derive(Clone)]
enum Tool {
    Command(Command),
    /// A choice shown by what is chosen: `shown`'s label is the current
    /// value, `options` are the values, `chosen` the key of the current one.
    Menu { shown: Command, options: Vec<Command>, chosen: Option<ElementId> },
}

/// Draws a [`ComposerState`] with its tools.
#[derive(IntoElement)]
pub struct Composer {
    state: Entity<ComposerState>,
    tools: Vec<Tool>,
    above: Vec<AnyElement>,
    stop: Option<Command>,
    can_send: bool,
    hint: Option<SharedString>,
}

impl Composer {
    pub fn new(state: &Entity<ComposerState>) -> Self {
        Composer { state: state.clone(), tools: Vec::new(), above: Vec::new(), stop: None, can_send: true, hint: None }
    }

    /// A tool: attach, priority, schedule.
    pub fn tool(mut self, command: Command) -> Self {
        self.tools.push(Tool::Command(command));
        self
    }

    /// A tool that is a choice, shown by what is chosen: `shown` carries the
    /// icon and the current value as its label, `options` run as they are
    /// picked, `chosen` is the id of the current option (marked). **Pointer**:
    /// a menu under the button. **Touch**: a sheet of the options.
    pub fn menu(mut self, shown: Command, options: Vec<Command>, chosen: Option<ElementId>) -> Self {
        self.tools.push(Tool::Menu { shown, options, chosen });
        self
    }

    /// While the other side works: send becomes `stop`.
    pub fn stop(mut self, stop: Option<Command>) -> Self {
        self.stop = stop;
        self
    }

    /// Whether a message can go now (the agent is connected).
    pub fn can_send(mut self, can_send: bool) -> Self {
        self.can_send = can_send;
        self
    }

    /// A line about the keys, beside the tools on a pointer: "/ for
    /// commands". A finger has no use for it.
    pub fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// What stands over the field: what is replied to, an agent's plan, the
    /// commands matching what is typed. Each call adds one, top to bottom.
    pub fn above(mut self, above: impl IntoElement) -> Self {
        self.above.push(above.into_any_element());
        self
    }

    /// What can complete what is typed: the commands matching "/re", the
    /// people matching "@an". A band over the field, one row each; none, no
    /// band.
    pub fn suggestions(mut self, suggestions: Vec<Suggestion>) -> Self {
        if !suggestions.is_empty() {
            self.above.push(Suggestions(suggestions).into_any_element());
        }
        self
    }
}

/// A completion of what is typed in a [`Composer`].
pub struct Suggestion {
    id: ElementId,
    label: SharedString,
    detail: Option<SharedString>,
    code: bool,
    on_pick: Handler,
}

impl Suggestion {
    /// `on_pick` puts it in the draft (`ComposerState::set_text`).
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, on_pick: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Suggestion { id: id.into(), label: label.into(), detail: None, code: false, on_pick: Rc::new(on_pick) }
    }

    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// The label is something typed as it is: "/review".
    pub fn code(mut self) -> Self {
        self.code = true;
        self
    }
}

/// The band of suggestions.
#[derive(IntoElement)]
struct Suggestions(Vec<Suggestion>);

impl RenderOnce for Suggestions {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let rows = self.0.into_iter().map(|suggestion| {
            let on_pick = suggestion.on_pick.clone();
            let label = if suggestion.code { Text::mono(suggestion.label) } else { Text::small(suggestion.label) };
            let row = pressable(suggestion.id, cx)
                .gap(scale.space_s)
                .px(scale.space_s)
                .rounded(scale.radius_control)
                .min_h(if env.is_touch() { scale.target } else { scale.row })
                .child(div().flex_none().child(label))
                .children(suggestion.detail.map(|detail| Text::small(detail).muted().truncate()))
                .on_click(move |_, window, cx| on_pick(window, cx));
            testable(row)
        });
        column().w_full().p(scale.space_xs).children(rows)
    }
}

impl Tool {
    /// The tool on the composer: an icon button, or the current value that
    /// opens the others: a menu under it on a pointer, a sheet on a finger.
    fn button(&self, env: Env) -> AnyElement {
        match self {
            Tool::Command(command) => command.icon_button(env).into_any_element(),
            Tool::Menu { shown, options, chosen } => {
                let button = Button::new(shown.id().clone()).ghost().xsmall().label(shown.label().clone()).dropdown_caret(true);
                let button = match shown.icon_ref() {
                    Some(icon) => button.icon(icon.clone()),
                    None => button,
                };
                let options = options.clone();
                if env.is_touch() {
                    let title = shown.label().clone();
                    return button
                        .on_click(move |_, window, cx| Sheet::actions(Some(title.clone()), options.clone(), window, cx))
                        .into_any_element();
                }
                let chosen = chosen.clone();
                button
                    .dropdown_menu(move |menu, _, _| {
                        options.iter().fold(menu, |menu, option| {
                            let run = option.clone();
                            menu.item(
                                PopupMenuItem::new(option.label().clone())
                                    .checked(chosen.as_ref() == Some(option.id()))
                                    .on_click(move |_, window, cx| run.run(window, cx)),
                            )
                        })
                    })
                    .into_any_element()
            }
        }
    }
}

impl RenderOnce for Composer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let (colors, scale) = (cx.colors(), cx.scale());
        let text = self.state.read(cx).text.clone();
        let empty = self.state.read(cx).is_empty(cx);
        let state = self.state.clone();
        // Send is round, `inset` from the card's edges, and the card's corner
        // is round about the same centre: inset plus half the button.
        let (size, inset) = (scale.control, scale.space_s);
        let corner = rems(inset.0 + size.0 / 2.);
        let send = match &self.stop {
            Some(stop) => stop.clone().icon(IconName::Pause).icon_button(env).primary(),
            None => Command::new(("composer-send", self.state.entity_id()), "Send")
                .icon(IconName::ArrowUp)
                .disabled(empty || !self.can_send)
                .on_trigger(move |_, cx| state.update(cx, |state, cx| state.send(cx)))
                .icon_button(env)
                .primary(),
        };
        let area = Textarea::new(&text).appearance(false);
        let area = gpui_android_glue::keyboard::on_tap(area, &text, cx);
        // A finger has no use for a line about the keys.
        let hint = self.hint.filter(|_| !env.is_touch());
        surface(Surface::Card, cx)
            .w_full()
            .rounded(corner)
            .overflow_hidden()
            .children(self.above.into_iter().map(|above| div().border_b_1().border_color(colors.border).child(above)))
            .child(div().px(inset).pt(inset).child(area))
            .child(
                row()
                    .gap(scale.space_xs)
                    .pl(inset)
                    .pr(inset)
                    .pb(inset)
                    .children(self.tools.iter().map(|tool| tool.button(env)))
                    .children(hint.map(|hint| Text::small(hint).muted().truncate()))
                    .child(div().flex_1())
                    .child(send.w(size).h(size).rounded_full()),
            )
    }
}
