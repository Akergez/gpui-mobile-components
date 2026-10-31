//! A line of text to type, with its label, its error, and what it is for.
//!
//! The field says what it is for with a [`Purpose`], the glue's: on a phone
//! that shapes the keyboard (digits for an amount, no corrections for a
//! login, capitals and corrections for prose) through
//! `gpui_android_glue::keyboard::follow`, and a tap raises the keyboard even
//! when the field kept its focus (`keyboard::on_tap`). A password is masked.
//!
//! **Pointer**: gpui-kit's input at its small size, the label over it.
//! **Touch**: the input at its large size (a finger's height), the label
//! over it, the error under it.
//!
//! A field of several lines ([`TextFieldState::multiline`]) grows with what
//! is written, between its bounds: details in Markdown, an environment.
//!
//! A field the owner drives with keys while it types — a switcher's
//! results, the completions of what is written — takes Up, Down, Enter, Tab
//! and Escape first with [`TextField::on_key`]; what it does not take goes
//! on to the text. The owner can read and move the cursor, read the
//! selection and replace it: a completion put in where the cursor is, a
//! pasted address wrapped around the selected words.
//!
//! The state is retained: make a [`TextFieldState`] in the owner's `new`,
//! hold it, draw `TextField::new(&state)` in `render`.

use std::rc::Rc;

use gpui_kit::component::input::{
    Enter, Escape, IndentInline, Input, InputEvent, InputState, MoveDown, MoveUp, Textarea, TextareaState,
};
use gpui_kit::component::{ActiveTheme, Icon, Sizable, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, App, ClipboardItem, Context, Entity, EventEmitter, FocusHandle, Focusable, RenderOnce,
    SharedString, Subscription, Window, div,
};

pub use gpui_android_glue::keyboard::Purpose;

use crate::env::Env;
use crate::scale::ActiveScale;

/// What a field tells its owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldEvent {
    /// The text changed.
    Change,
    /// Enter (or the keyboard's action key) was pressed: move on or commit.
    Submit,
    /// The field took the focus.
    Focus,
    /// The field lost the focus.
    Blur,
}

/// A key a [`TextField::on_key`] handler is offered before the text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldKey {
    Up,
    Down,
    /// Enter without Shift.
    Enter,
    Tab,
    Escape,
}

/// Takes a key before the text does: `true` when it was used.
type OnKey = Rc<dyn Fn(FieldKey, &mut Window, &mut App) -> bool>;

/// Looks at what is pasted before the text does: `true` when it was used
/// (an image attached, an address made a link).
type OnPaste = Rc<dyn Fn(&ClipboardItem, &mut Window, &mut App) -> bool>;

/// Runs `$body` with `$input` bound to the field's one-line or multi-line
/// input state: the two have the same methods.
macro_rules! either {
    ($text:expr, $input:ident => $body:expr) => {
        match $text {
            Text::Line($input) => $body,
            Text::Lines($input) => $body,
        }
    };
}

/// The text the field keeps: a line, or lines.
enum Text {
    Line(Entity<InputState>),
    Lines(Entity<TextareaState>),
}

/// The retained state of a text field.
pub struct TextFieldState {
    input: Text,
    purpose: Purpose,
    error: Option<SharedString>,
    _subscription: Subscription,
}

impl EventEmitter<FieldEvent> for TextFieldState {}

impl TextFieldState {
    pub fn new(purpose: Purpose, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).masked(purpose == Purpose::Password));
        let subscription = cx.subscribe(&input, move |this: &mut Self, input, event: &InputEvent, cx| {
            this.on_input(input.entity_id(), event, cx)
        });
        TextFieldState { input: Text::Line(input), purpose, error: None, _subscription: subscription }
    }

    /// A field of several lines, growing from `min` to `max` lines with what
    /// is written. Enter breaks the line.
    pub fn multiline(purpose: Purpose, min: usize, max: usize, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextareaState::new(window, cx).auto_grow(min, max));
        let subscription = cx.subscribe(&input, move |this: &mut Self, input, event: &InputEvent, cx| {
            this.on_input(input.entity_id(), event, cx)
        });
        TextFieldState { input: Text::Lines(input), purpose, error: None, _subscription: subscription }
    }

    /// What to ask, in a few lines: the first message of a conversation. On a
    /// pointer Enter commits the form (Shift+Enter breaks the line), as in a
    /// conversation; on a finger Enter breaks the line and the form's
    /// button commits.
    pub fn prompt(min: usize, max: usize, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let pointer = crate::env::input_mode(cx) == crate::env::InputMode::Pointer;
        let input = cx.new(|cx| TextareaState::new(window, cx).auto_grow(min, max).submit_on_enter(pointer));
        let subscription = cx.subscribe(&input, move |this: &mut Self, input, event: &InputEvent, cx| {
            if pointer && matches!(event, InputEvent::PressEnter { shift: false, .. }) {
                cx.emit(FieldEvent::Submit);
            }
            this.on_input(input.entity_id(), event, cx)
        });
        TextFieldState { input: Text::Lines(input), purpose: Purpose::Message, error: None, _subscription: subscription }
    }

    fn on_input(&mut self, input: gpui_kit::EntityId, event: &InputEvent, cx: &mut Context<Self>) {
        gpui_android_glue::keyboard::follow(input, event, self.purpose);
        match event {
            InputEvent::Change => {
                self.error = None;
                cx.emit(FieldEvent::Change);
                cx.notify();
            }
            InputEvent::PressEnter { .. } if matches!(self.input, Text::Line(_)) => cx.emit(FieldEvent::Submit),
            InputEvent::Focus => cx.emit(FieldEvent::Focus),
            InputEvent::Blur => cx.emit(FieldEvent::Blur),
            _ => {}
        }
    }

    /// A field for an amount: the keyboard of digits with the decimal
    /// separator.
    pub fn amount(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::new(Purpose::Number, window, cx)
    }

    /// What the empty field shows.
    pub fn set_placeholder(&mut self, placeholder: impl Into<SharedString>, window: &mut Window, cx: &mut Context<Self>) {
        let placeholder: SharedString = placeholder.into();
        match &self.input {
            Text::Line(input) => input.update(cx, |input, cx| input.set_placeholder(placeholder, window, cx)),
            Text::Lines(input) => input.update(cx, |input, cx| input.set_placeholder(placeholder, window, cx)),
        }
    }

    pub fn value(&self, cx: &App) -> SharedString {
        match &self.input {
            Text::Line(input) => input.read(cx).value(),
            Text::Lines(input) => input.read(cx).value(),
        }
    }

    pub fn set_value(&mut self, value: impl Into<SharedString>, window: &mut Window, cx: &mut Context<Self>) {
        let value: SharedString = value.into();
        match &self.input {
            Text::Line(input) => input.update(cx, |input, cx| input.set_value(value, window, cx)),
            Text::Lines(input) => input.update(cx, |input, cx| input.set_value(value, window, cx)),
        }
    }

    /// The number typed, read with either decimal separator and with spaces
    /// between thousands; `None` while it is not a number.
    pub fn number(&self, cx: &App) -> Option<f64> {
        parse_number(&self.value(cx))
    }

    /// What is wrong with the value, shown under the field until it is
    /// changed. `None` clears it.
    pub fn set_error(&mut self, error: Option<impl Into<SharedString>>, cx: &mut Context<Self>) {
        self.error = error.map(Into::into);
        cx.notify();
    }

    pub fn error(&self) -> Option<&SharedString> {
        self.error.as_ref()
    }

    pub fn purpose(&self) -> Purpose {
        self.purpose
    }

    /// Puts the focus in the field, the cursor where it was.
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        either!(&self.input, input => input.update(cx, |input, cx| input.focus(window, cx)))
    }

    pub fn is_focused(&self, window: &Window, cx: &App) -> bool {
        self.focus_handle(cx).is_focused(window)
    }

    /// Where the cursor is, in bytes of the value.
    pub fn cursor(&self, cx: &App) -> usize {
        either!(&self.input, input => input.read(cx).cursor())
    }

    /// Puts the cursor at `offset`, in bytes of the value, selecting nothing.
    pub fn set_cursor(&mut self, offset: usize, cx: &mut Context<Self>) {
        either!(&self.input, input => input.update(cx, |input, cx| input.set_selected_range(offset..offset, cx)))
    }

    /// What is selected; empty when nothing is.
    pub fn selected_text(&self, cx: &App) -> String {
        either!(&self.input, input => input.read(cx).selected_text().to_string())
    }

    /// Puts `text` in place of the selection (or at the cursor), as typing
    /// would: it can be undone.
    pub fn replace_selection(&mut self, text: impl Into<SharedString>, window: &mut Window, cx: &mut Context<Self>) {
        let text: SharedString = text.into();
        either!(&self.input, input => input.update(cx, |input, cx| input.replace(text, window, cx)))
    }

    /// The input inside a one-line field, for what gpui-kit's own API
    /// offers beyond this.
    pub fn input(&self) -> Option<&Entity<InputState>> {
        match &self.input {
            Text::Line(input) => Some(input),
            Text::Lines(_) => None,
        }
    }
}

impl Focusable for TextFieldState {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        match &self.input {
            Text::Line(input) => input.read(cx).focus_handle(cx),
            Text::Lines(input) => input.read(cx).focus_handle(cx),
        }
    }
}

/// Reads "1 240,50", "1240.5" and "-60" alike.
pub fn parse_number(text: &str) -> Option<f64> {
    let cleaned: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{a0}' && *c != '\u{202f}')
        .map(|c| if c == ',' { '.' } else { c })
        .collect();
    if cleaned.is_empty() {
        return None;
    }
    cleaned.parse().ok()
}

/// Draws a [`TextFieldState`].
#[derive(IntoElement)]
pub struct TextField {
    state: Entity<TextFieldState>,
    label: Option<SharedString>,
    help: Option<SharedString>,
    icon: Option<Icon>,
    suffix: Option<AnyElement>,
    cleanable: bool,
    disabled: bool,
    on_key: Option<OnKey>,
    on_paste: Option<OnPaste>,
}

impl TextField {
    pub fn new(state: &Entity<TextFieldState>) -> Self {
        TextField {
            state: state.clone(),
            label: None,
            help: None,
            icon: None,
            suffix: None,
            cleanable: false,
            disabled: false,
            on_key: None,
            on_paste: None,
        }
    }

    /// What starts a one-line field: a magnifier for a search.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// A cross that empties a one-line field while it holds something: a
    /// search to start again.
    pub fn cleanable(mut self) -> Self {
        self.cleanable = true;
        self
    }

    /// Offers Up, Down, Enter, Tab and Escape to `on_key` before the text:
    /// the keys that move through a list the field drives. What it returns
    /// `false` for reaches the text as usual.
    pub fn on_key(mut self, on_key: impl Fn(FieldKey, &mut Window, &mut App) -> bool + 'static) -> Self {
        self.on_key = Some(Rc::new(on_key));
        self
    }

    /// Offers what is pasted to `on_paste` before the text: `true` when it
    /// was used, and then the text gets nothing.
    pub fn on_paste(mut self, on_paste: impl Fn(&ClipboardItem, &mut Window, &mut App) -> bool + 'static) -> Self {
        self.on_paste = Some(Rc::new(on_paste));
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// A line under the field that helps, shown while there is no error.
    pub fn help(mut self, help: impl Into<SharedString>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// What ends the field: a currency, a unit.
    pub fn suffix(mut self, suffix: impl IntoElement) -> Self {
        self.suffix = Some(suffix.into_any_element());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let theme = cx.theme();
        let (muted, danger) = (theme.muted_foreground, theme.danger);
        let state = self.state.read(cx);
        let (error, password) = (state.error.clone(), state.purpose == Purpose::Password);
        let input: AnyElement = match &state.input {
            Text::Line(input_state) => {
                let input_state = input_state.clone();
                let input = Input::new(&input_state).disabled(self.disabled).cleanable(self.cleanable);
                // A password can be looked at before it is sent.
                let input = if password { input.mask_toggle() } else { input };
                let input = match self.icon {
                    Some(icon) => input.prefix(div().text_color(muted).child(icon)),
                    None => input,
                };
                let input = match self.on_paste.clone() {
                    Some(on_paste) => input.on_paste(move |item, window, cx| on_paste(item, window, cx)),
                    None => input,
                };
                let input = match self.suffix {
                    Some(suffix) => input.suffix(div().text_color(muted).child(suffix)),
                    None => input,
                };
                let input = if env.is_touch() { input.large() } else { input.small() };
                gpui_android_glue::keyboard::on_tap(input, &input_state, cx).into_any_element()
            }
            Text::Lines(input_state) => {
                let input_state = input_state.clone();
                let area = Textarea::new(&input_state).disabled(self.disabled);
                let area = match self.on_paste.clone() {
                    Some(on_paste) => area.on_paste(move |item, window, cx| on_paste(item, window, cx)),
                    None => area,
                };
                let area = if env.is_touch() { area.large() } else { area.small() };
                gpui_android_glue::keyboard::on_tap(area, &input_state, cx).into_any_element()
            }
        };
        let under = match (error, self.help) {
            (Some(error), _) => Some(div().text_size(scale.small).text_color(danger).child(error)),
            (None, Some(help)) => Some(div().text_size(scale.small).text_color(muted).child(help)),
            (None, None) => None,
        };
        let field = v_flex()
            .w_full()
            .gap_1()
            .children(self.label.map(|label| {
                div().text_size(scale.small).font_weight(gpui_kit::FontWeight::MEDIUM).child(label)
            }))
            .child(input)
            .children(under);
        match self.on_key {
            Some(on_key) => offer_keys(field, on_key),
            None => field,
        }
    }
}

/// Offers the keys that move through a list to `on_key`, on their way down
/// to the text: taken there, they never reach it. Enter is taken as an
/// action, before the text sees the key: left to arrive as the field's own
/// "submitted", it would already have been typed.
fn offer_keys(field: gpui_kit::Div, on_key: OnKey) -> gpui_kit::Div {
    let offer = move |key: FieldKey| {
        let on_key = on_key.clone();
        move |window: &mut Window, cx: &mut App| {
            if on_key(key, window, cx) {
                cx.stop_propagation();
            }
        }
    };
    let (up, down, enter, tab, escape) =
        (offer(FieldKey::Up), offer(FieldKey::Down), offer(FieldKey::Enter), offer(FieldKey::Tab), offer(FieldKey::Escape));
    field
        .capture_action(move |_: &MoveUp, window, cx| up(window, cx))
        .capture_action(move |_: &MoveDown, window, cx| down(window, cx))
        .capture_action(move |action: &Enter, window, cx| {
            if !action.shift {
                enter(window, cx)
            }
        })
        .capture_action(move |_: &IndentInline, window, cx| tab(window, cx))
        .capture_action(move |_: &Escape, window, cx| escape(window, cx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_read_with_either_separator_and_grouped_thousands() {
        assert_eq!(parse_number("1 240,50"), Some(1240.5));
        assert_eq!(parse_number("1240.5"), Some(1240.5));
        assert_eq!(parse_number("-60"), Some(-60.));
        assert_eq!(parse_number("12\u{a0}400"), Some(12400.));
        assert_eq!(parse_number(""), None);
        assert_eq!(parse_number("abc"), None);
    }
}
