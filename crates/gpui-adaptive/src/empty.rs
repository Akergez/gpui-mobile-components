//! Nothing to show yet, and the one thing to do about it: "Nothing needs
//! you — New chat".
//!
//! **Pointer**: a muted icon, a line and a button, centred, quiet.
//! **Touch**: Material's empty state: the icon on a large container, a title,
//! a sentence, and a tonal button.

use gpui_kit::component::{ActiveTheme, Icon, Sizable, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{App, RenderOnce, SharedString, Window, div, rems};

use crate::command::Command;
use crate::env::{Env, InputMode};
use crate::palette::ActivePalette;
use crate::scale::ActiveScale;

#[derive(IntoElement)]
pub struct Empty {
    icon: Icon,
    title: SharedString,
    body: Option<SharedString>,
    action: Option<Command>,
}

impl Empty {
    pub fn new(icon: impl Into<Icon>, title: impl Into<SharedString>) -> Self {
        Empty { icon: icon.into(), title: title.into(), body: None, action: None }
    }

    pub fn body(mut self, body: impl Into<SharedString>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn action(mut self, command: Command) -> Self {
        self.action = Some(command);
        self
    }
}

impl RenderOnce for Empty {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let palette = cx.palette();
        let muted = cx.theme().muted_foreground;
        let figure = match env.input() {
            InputMode::Pointer => div().text_color(muted).child(self.icon.large()),
            InputMode::Touch => div()
                .size(rems(6.))
                .rounded(rems(2.))
                .flex()
                .items_center()
                .justify_center()
                .bg(palette.secondary_container)
                .text_color(palette.on_secondary_container)
                .child(self.icon.size(rems(2.75))),
        };
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_3()
            .p_6()
            .child(figure)
            .child(
                div()
                    .text_size(if env.is_touch() { scale.title } else { scale.body })
                    .font_weight(gpui_kit::FontWeight::MEDIUM)
                    .child(self.title),
            )
            .children(self.body.map(|body| {
                div().max_w(rems(24.)).text_center().text_size(scale.small).text_color(muted).child(body)
            }))
            .children(self.action.map(|command| command.button(env)))
    }
}
