//! A standing state of the page, under its bar: "3 people are in a call —
//! Join", "Server offline · retrying", "Sign in to continue".
//!
//! A banner stays as long as the state does, unlike a [`crate::Notice`]. Its
//! tone is semantic and scarce: neutral information, success, a warning, or
//! danger. Both skins draw a strip across the page; the touch strip is a
//! finger's height and its action is a text button.

use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::component::{ActiveTheme, Icon, Sizable, h_flex};
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, ElementId, Hsla, RenderOnce, SharedString, Window, div};

use crate::command::Command;
use crate::env::Env;
use crate::scale::ActiveScale;

/// What a banner says about the page.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Info,
    Success,
    Warning,
    Danger,
}

impl Tone {
    /// The theme's colour for the tone.
    pub fn color(self, cx: &App) -> Hsla {
        let theme = cx.theme();
        match self {
            Tone::Info => theme.info,
            Tone::Success => theme.success,
            Tone::Warning => theme.warning,
            Tone::Danger => theme.danger,
        }
    }
}

/// A strip under the page's bar.
#[derive(IntoElement)]
pub struct Banner {
    id: ElementId,
    tone: Tone,
    text: SharedString,
    icon: Option<Icon>,
    extra: Option<AnyElement>,
    action: Option<Command>,
}

impl Banner {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Banner { id: id.into(), tone: Tone::Info, text: text.into(), icon: None, extra: None, action: None }
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Something to show before the action: the faces in a call, a spinner.
    pub fn extra(mut self, extra: impl IntoElement) -> Self {
        self.extra = Some(extra.into_any_element());
        self
    }

    pub fn action(mut self, command: Command) -> Self {
        self.action = Some(command);
        self
    }
}

impl RenderOnce for Banner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let tint = self.tone.color(cx);
        let action = self.action.map(|command| {
            let button = command.button(env);
            if env.is_touch() { button.ghost().into_any_element() } else { button.small().into_any_element() }
        });
        h_flex()
            .id(self.id)
            .flex_none()
            .w_full()
            .min_h(if env.is_touch() { scale.target } else { scale.control })
            .px_3()
            .py_1()
            .gap_2()
            .bg(tint.opacity(0.14))
            .border_b_1()
            .border_color(tint.opacity(0.35))
            .text_size(scale.small)
            .children(self.icon.map(|icon| icon.small().text_color(tint)))
            .child(div().flex_1().min_w_0().child(self.text))
            .children(self.extra)
            .children(action)
    }
}
