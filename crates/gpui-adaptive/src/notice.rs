//! A short message that goes away by itself, with at most one action:
//! "Record deleted — Undo".
//!
//! **Pointer**: a gpui-kit notification at the window's corner, which stacks
//! with others; with an action it stays until dismissed, as the toolkit's do.
//! **Touch**: a Material snackbar on the inverse surface, above the
//! navigation bar and the keyboard, one at a time (a new one replaces the
//! one shown), gone after [`crate::Scale::notice`].
//!
//! Undo is the reason this exists: the Coding Guides ask for no confirmation
//! before a reversible action, and an Undo afterwards instead.

use std::rc::Rc;

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::notification::{Notification, NotificationType};
use gpui_kit::component::{ActiveTheme, Icon, IconName, Sizable, WindowExt, h_flex};
use gpui_kit::prelude::*;
use gpui_kit::{App, Context, Render, SharedString, Window, WindowId, div};

use crate::banner::Tone;
use crate::env::{InputMode, input_mode};
use crate::insets::Insets;
use crate::layers;
use crate::palette::ActivePalette;
use crate::scale::ActiveScale;

type Act = Rc<dyn Fn(&mut Window, &mut App)>;

/// A message to show briefly.
#[derive(Clone)]
pub struct Notice {
    message: SharedString,
    tone: Tone,
    action: Option<(SharedString, Act)>,
}

impl Notice {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Notice { message: message.into(), tone: Tone::Info, action: None }
    }

    /// What kind of news it is. `Danger` for a failure the person should
    /// know about ("Could not reach the server"); the rest are quiet.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// A failure: `tone(Tone::Danger)`.
    pub fn error(message: impl Into<SharedString>) -> Self {
        Self::new(message).tone(Tone::Danger)
    }

    /// The one thing to do about it, such as "Undo". Choosing it puts the
    /// notice away.
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        act: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), Rc::new(act)));
        self
    }

    pub fn message(&self) -> &SharedString {
        &self.message
    }

    /// Shows it over `window`.
    pub fn push(self, window: &mut Window, cx: &mut App) {
        match input_mode(cx) {
            InputMode::Pointer => {
                let kind = match self.tone {
                    Tone::Info => NotificationType::Info,
                    Tone::Success => NotificationType::Success,
                    Tone::Warning => NotificationType::Warning,
                    Tone::Danger => NotificationType::Error,
                };
                let mut note = Notification::new().message(self.message.clone()).with_type(kind);
                if let Some((label, act)) = self.action {
                    note = note.action(move |_, _, cx| {
                        let (act, note) = (act.clone(), cx.entity());
                        Button::new("notice-action").label(label.clone()).primary().on_click(
                            move |_, window, cx| {
                                note.update(cx, |note, cx| note.dismiss(window, cx));
                                act(window, cx);
                            },
                        )
                    });
                }
                window.push_notification(note, cx);
            }
            InputMode::Touch => {
                let window_id = window.window_handle().window_id();
                let life = cx.scale().notice;
                let snackbar = cx.new(|_| Snackbar { notice: self, window: window_id });
                let id = snackbar.entity_id();
                layers::set_snackbar(window, snackbar, cx);
                cx.spawn(async move |cx| {
                    cx.background_executor().timer(life).await;
                    cx.update(|cx| layers::remove_snackbar(window_id, id, cx));
                })
                .detach();
            }
        }
    }
}

/// The snackbar a [`Notice`] is on a phone.
pub(crate) struct Snackbar {
    notice: Notice,
    window: WindowId,
}

impl Render for Snackbar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let scale = cx.scale();
        let insets = Insets::of(window);
        let action = self.notice.action.clone().map(|(label, act)| {
            let (window_id, id) = (self.window, cx.entity_id());
            div()
                .id("snackbar-action")
                .px_3()
                .h(scale.target)
                .flex()
                .items_center()
                .cursor_pointer()
                .text_color(palette.inverse_primary)
                .font_weight(gpui_kit::FontWeight::MEDIUM)
                .child(label)
                .on_click(move |_, window, cx| {
                    layers::remove_snackbar(window_id, id, cx);
                    act(window, cx);
                })
                .test_support()
        });
        div()
            .absolute()
            .left_0()
            .right_0()
            .bottom(insets.typing() + scale.navigation_bar.to_pixels(window.rem_size()))
            .px_3()
            .child(
                h_flex()
                    .occlude()
                    .min_h(scale.target)
                    .pl_4()
                    .pr_1()
                    .rounded(gpui_kit::rems(0.25))
                    .bg(palette.inverse_surface)
                    .text_color(palette.inverse_on_surface)
                    .text_size(scale.small)
                    .shadow_md()
                    .gap_3()
                    .children(match self.notice.tone {
                        Tone::Info => None,
                        Tone::Success => Some(Icon::new(IconName::CircleCheck).small().text_color(cx.theme().success)),
                        Tone::Warning => Some(Icon::new(IconName::TriangleAlert).small().text_color(cx.theme().warning)),
                        Tone::Danger => Some(Icon::new(IconName::CircleX).small().text_color(cx.theme().danger)),
                    })
                    .child(div().flex_1().py_3().child(self.notice.message.clone()))
                    .children(action),
            )
    }
}
