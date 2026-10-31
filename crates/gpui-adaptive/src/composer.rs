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
//! **Pointer**: a card: what stands above, the field, and under it the tools,
//! a hint about the keys and send. Enter sends and Shift+Enter breaks the
//! line.
//! **Touch**: the composer rides on the keyboard (it is padded by
//! [`crate::Insets::typing`] by whoever holds the page). Enter breaks the
//! line and the send button sends, as on a phone; the tools fold into a
//! "+" that opens them in a sheet; the keyboard is for prose.
//!
//! The draft is the owner's: [`ComposerState::text`] reads it, `clear`
//! empties it after sending.

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::input::{InputEvent, Textarea, TextareaState};
use gpui_kit::component::{ActiveTheme, Disableable as _, IconName, Sizable, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, App, Context, ElementId, Entity, EventEmitter, FocusHandle, Focusable, RenderOnce, SharedString,
    Subscription, Window, div,
};

use std::rc::Rc;

use crate::command::{Command, Handler};
use crate::env::{Env, InputMode, input_mode};
use crate::palette::ActivePalette;
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

    pub fn set_text(&mut self, text: impl Into<SharedString>, window: &mut Window, cx: &mut Context<Self>) {
        let text: SharedString = text.into();
        self.text.update(cx, |area, cx| area.set_value(text, window, cx));
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
        let (muted, mono) = (cx.theme().muted_foreground, cx.theme().mono_font_family.clone());
        let rows = self.0.into_iter().map(|suggestion| {
            let on_pick = suggestion.on_pick.clone();
            let row = crate::touch::pressable(suggestion.id, env, cx)
                .gap_2()
                .px_2()
                .rounded(scale.radius_control)
                .min_h(if env.is_touch() { scale.target } else { scale.row })
                .text_size(scale.small)
                .child(div().flex_none().when(suggestion.code, |label| label.font_family(mono.clone())).child(suggestion.label))
                .children(suggestion.detail.map(|detail| div().min_w_0().truncate().text_color(muted).child(detail)))
                .on_click(move |_, window, cx| on_pick(window, cx));
            crate::prim::testable(row)
        });
        v_flex().w_full().px_1().py_1().children(rows)
    }
}

impl Tool {
    /// The tool on a pointer: an icon button, or the current value with a
    /// menu of the others.
    fn pointer(&self, env: Env) -> AnyElement {
        match self {
            Tool::Command(command) => command.icon_button(env).into_any_element(),
            Tool::Menu { shown, options, chosen } => {
                let (options, chosen) = (options.clone(), chosen.clone());
                let button = Button::new(shown.id().clone()).ghost().xsmall().label(shown.label().clone()).dropdown_caret(true);
                let button = match shown.icon_ref() {
                    Some(icon) => button.icon(icon.clone()),
                    None => button,
                };
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

    /// The tool as a row of the touch sheet: a menu opens a sheet of its own.
    fn touch(&self) -> Command {
        match self {
            Tool::Command(command) => command.clone(),
            Tool::Menu { shown, options, .. } => {
                let (title, options) = (shown.label().clone(), options.clone());
                let mut row = Command::new(shown.id().clone(), shown.label().clone())
                    .on_trigger(move |window, cx| Sheet::actions(Some(title.clone()), options.clone(), window, cx));
                if let Some(icon) = shown.icon_ref() {
                    row = row.icon(icon.clone());
                }
                row
            }
        }
    }
}

impl RenderOnce for Composer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let theme = cx.theme();
        let scale = cx.scale();
        let text = self.state.read(cx).text.clone();
        let empty = self.state.read(cx).is_empty(cx);
        let state = self.state.clone();
        let send = match &self.stop {
            Some(stop) => {
                let stop = stop.clone();
                Button::new(stop.id().clone())
                    .icon(IconName::Pause)
                    .accessibility_label(stop.label().clone())
                    .tooltip(stop.label().clone())
                    .on_click(move |_, window, cx| stop.run(window, cx))
            }
            None => Button::new(("composer-send", self.state.entity_id()))
                .icon(IconName::ArrowUp)
                .primary()
                .disabled(empty || !self.can_send)
                .accessibility_label("Send")
                .tooltip("Send")
                .on_click(move |_, _, cx| state.update(cx, |state, cx| state.send(cx))),
        };
        let area = Textarea::new(&text).appearance(false);
        let area = gpui_android_glue::keyboard::on_tap(area, &text, cx);
        match env.input() {
            InputMode::Pointer => {
                let (border, muted, background) = (theme.border, theme.muted_foreground, theme.background);
                v_flex()
                    .w_full()
                    .rounded(gpui_kit::rems(1.125))
                    .border_1()
                    .border_color(border)
                    .bg(background)
                    .overflow_hidden()
                    .children(self.above.into_iter().map(|above| div().border_b_1().border_color(border).child(above)))
                    .child(div().px_1().pt_1().child(area))
                    .child(
                        h_flex()
                            .gap_1()
                            .px_2()
                            .pb_2()
                            .children(self.tools.iter().map(|tool| tool.pointer(env)))
                            .children(self.hint.map(|hint| div().text_size(scale.small).text_color(muted).child(hint)))
                            .child(div().flex_1())
                            .child(send.small()),
                    )
                    .into_any_element()
            }
            InputMode::Touch => {
                let palette = cx.palette();
                let tools: Vec<Command> = self.tools.iter().map(Tool::touch).collect();
                let more = (!tools.is_empty()).then(|| {
                    Button::new(("composer-tools", self.state.entity_id()))
                        .ghost()
                        .large()
                        .icon(IconName::Plus)
                        .accessibility_label("More")
                        .on_click(move |_, window, cx| Sheet::actions(None, tools.clone(), window, cx))
                });
                v_flex()
                    .w_full()
                    .bg(palette.surface_container)
                    .children(self.above)
                    .child(
                        h_flex()
                            .w_full()
                            .items_end()
                            .gap_1()
                            .pl_1()
                            .pr_2()
                            .py_1p5()
                            .children(more)
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .min_h(scale.target)
                                    .px_4()
                                    .py_2()
                                    .rounded(gpui_kit::rems(1.5))
                                    .bg(palette.surface_highest)
                                    .text_size(scale.body)
                                    .child(area),
                            )
                            .child(send.large()),
                    )
                    .into_any_element()
            }
        }
    }
}
