//! Writing a message: to someone in a channel, or to an agent.
//!
//! A text area that grows with what is written (up to eight lines), the
//! tools around it (attach, priority, schedule, an agent's mode) and send.
//! Things can stand above it: a reply-to line, an agent's plan.
//!
//! **Pointer**: the tools beside the field; Enter sends and Shift+Enter
//! breaks the line.
//! **Touch**: the composer rides on the keyboard (it is padded by
//! [`crate::Insets::typing`] by whoever holds the page). Enter breaks the
//! line and the send button sends, as on a phone; the tools fold into a
//! "+" that opens them in a sheet; the keyboard is for prose.
//!
//! The draft is the owner's: [`ComposerState::text`] reads it, `clear`
//! empties it after sending.

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{InputEvent, Textarea, TextareaState};
use gpui_kit::component::{ActiveTheme, Disableable as _, IconName, Sizable, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, App, Context, Entity, EventEmitter, FocusHandle, Focusable, RenderOnce, SharedString,
    Subscription, Window, div,
};

use crate::command::Command;
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

/// Draws a [`ComposerState`] with its tools.
#[derive(IntoElement)]
pub struct Composer {
    state: Entity<ComposerState>,
    tools: Vec<Command>,
    above: Option<AnyElement>,
}

impl Composer {
    pub fn new(state: &Entity<ComposerState>) -> Self {
        Composer { state: state.clone(), tools: Vec::new(), above: None }
    }

    /// A tool: attach, priority, schedule, an agent's mode.
    pub fn tool(mut self, command: Command) -> Self {
        self.tools.push(command);
        self
    }

    /// What stands over the field: what is replied to, an agent's plan.
    pub fn above(mut self, above: impl IntoElement) -> Self {
        self.above = Some(above.into_any_element());
        self
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
        let send = Button::new(("composer-send", self.state.entity_id()))
            .icon(IconName::ArrowUp)
            .primary()
            .disabled(empty)
            .accessibility_label("Send")
            .on_click(move |_, _, cx| state.update(cx, |state, cx| state.send(cx)));
        let area = Textarea::new(&text).appearance(false);
        let area = gpui_android_glue::keyboard::on_tap(area, &text, cx);
        match env.input() {
            InputMode::Pointer => v_flex()
                .w_full()
                .p_2()
                .gap_1()
                .children(self.above)
                .child(
                    h_flex()
                        .w_full()
                        .items_end()
                        .gap_1()
                        .children(self.tools.iter().map(|tool| tool.icon_button(env)))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .px_2()
                                .py_1()
                                .rounded(theme.radius)
                                .border_1()
                                .border_color(theme.input)
                                .child(area),
                        )
                        .child(send.small()),
                )
                .into_any_element(),
            InputMode::Touch => {
                let palette = cx.palette();
                let tools = self.tools.clone();
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
