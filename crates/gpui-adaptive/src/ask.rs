//! A question with a few answers, asked where it arises: an agent asking to
//! write a file (aiddiction's permission requests, in the conversation and
//! in the inbox row), a bot's message with buttons (matterfast's
//! interactive messages).
//!
//! Unlike [`crate::Confirm`] it does not stop the page: it is a card in the
//! flow, answered when the person gets to it. [`Ask::open`] asks it in a
//! dialog instead, for a question that must be answered before going on.
//!
//! Each answer is a [`Command`]: the primary one is filled, a destructive
//! one is in the danger colour, the rest are outlined. On a pointer the
//! answers are small buttons at the card's end; on a finger they are a
//! finger's height and wrap onto more lines when they do not fit.

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::dialog::DialogFooter;
use gpui_kit::component::{ActiveTheme, Icon, IconName, Sizable, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{App, ElementId, RenderOnce, SharedString, Window, div};

use crate::banner::Tone;
use crate::command::{Command, CommandRole};
use crate::env::{Env, InputMode, input_mode};
use crate::scale::ActiveScale;

/// A question and its answers.
#[derive(IntoElement, Clone)]
pub struct Ask {
    id: ElementId,
    question: SharedString,
    detail: Option<SharedString>,
    tone: Tone,
    answers: Vec<Command>,
}

impl Ask {
    pub fn new(id: impl Into<ElementId>, question: impl Into<SharedString>) -> Self {
        Ask { id: id.into(), question: question.into(), detail: None, tone: Tone::Warning, answers: Vec::new() }
    }

    /// What exactly is asked about: the command, the file, the bot.
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// `Warning` (the default) for asking for permission, `Info` for an
    /// ordinary question.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// An answer, in the order shown. Mark the expected one `primary()`.
    pub fn answer(mut self, command: Command) -> Self {
        self.answers.push(command);
        self
    }

    fn button(command: &Command, touch: bool) -> Button {
        let run = command.clone();
        let button = Button::new(command.id().clone())
            .label(command.label().clone())
            .on_click(move |_, window, cx| run.run(window, cx));
        let button = match command.role() {
            CommandRole::Primary => button.primary(),
            CommandRole::Destructive => button.danger(),
            _ => button.outline(),
        };
        if touch { button } else { button.small() }
    }

    /// Asks it in a dialog that only an answer (or back) closes. The answer
    /// runs once the dialog has gone.
    pub fn open(self, window: &mut Window, cx: &mut App) {
        let touch = input_mode(cx) == InputMode::Touch;
        gpui_android_glue::back::open_dialog(window, cx, move |dialog, _, cx| {
            let muted = cx.theme().muted_foreground;
            let buttons = self.answers.iter().map(|command| {
                let run = command.clone();
                Ask::button(command, touch).on_click(move |_, window, cx| {
                    let run = run.clone();
                    gpui_kit::component::WindowExt::close_dialog(window, cx);
                    window.defer(cx, move |window, cx| run.run(window, cx));
                })
            });
            dialog
                .title(self.question.clone())
                .children(self.detail.clone().map(|detail| div().text_color(muted).child(detail)))
                .footer(DialogFooter::new().children(buttons))
        });
    }
}

impl RenderOnce for Ask {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let touch = env.is_touch();
        let theme = cx.theme();
        let scale = cx.scale();
        let tint = match self.tone {
            Tone::Info => theme.info,
            Tone::Success => theme.success,
            Tone::Warning => theme.warning,
            Tone::Danger => theme.danger,
        };
        let (muted, background) = (theme.muted_foreground, theme.background);
        let answers = h_flex()
            .flex_wrap()
            .gap_2()
            .when(!touch, |answers| answers.justify_end())
            .children(self.answers.iter().map(|command| Ask::button(command, touch)));
        v_flex()
            .id(self.id)
            .w_full()
            .p_3()
            .gap_2()
            .rounded(scale.radius_container)
            .border_1()
            .border_color(tint.opacity(0.6))
            .bg(tint.opacity(0.1).blend(background.opacity(0.9)))
            .child(
                h_flex()
                    .gap_2()
                    .items_start()
                    .child(Icon::new(IconName::TriangleAlert).small().text_color(tint))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(div().text_size(scale.body).child(self.question))
                            .children(self.detail.map(|detail| {
                                div().text_size(scale.small).text_color(muted).child(detail)
                            })),
                    ),
            )
            .child(answers)
    }
}
