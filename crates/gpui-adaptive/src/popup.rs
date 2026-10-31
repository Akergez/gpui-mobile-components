//! Something small to do or see beside what opened it: a palette of
//! reactions, who is in a call, the account and its teams.
//!
//! **Pointer**: a gpui-kit popover under the button, closed by a press
//! outside it, by Escape, or by what it holds once the choice is made.
//! **Touch**: the same content in a [`Sheet`], which back and a drag down
//! close as well.
//!
//! The content is built each time it is drawn, and is handed how to close
//! the popup: a choice made in it usually ends it.

use std::rc::Rc;

use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::component::popover::Popover;
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, Context, DismissEvent, ElementId, Render, RenderOnce, Window, div};

use crate::command::{Command, Handler};
use crate::env::Env;
use crate::scale::ActiveScale;
use crate::sheet::Sheet;

/// Builds what a popup holds, given what closes it.
type Content = Rc<dyn Fn(&Handler, &mut Window, &mut App) -> AnyElement>;

/// A button that opens content over the page.
#[derive(IntoElement)]
pub struct Popup {
    id: ElementId,
    button: Command,
    face: Option<AnyElement>,
    content: Content,
}

impl Popup {
    /// `button` is what opens it: its icon, its label as the tooltip.
    pub fn new(
        id: impl Into<ElementId>,
        button: Command,
        content: impl Fn(&Handler, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Popup { id: id.into(), button, face: None, content: Rc::new(content) }
    }

    /// Draws `face` in the button instead of the command's icon: a person's
    /// avatar, say.
    pub fn face(mut self, face: impl IntoElement) -> Self {
        self.face = Some(face.into_any_element());
        self
    }
}

impl RenderOnce for Popup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let content = self.content.clone();
        let button = if env.is_touch() {
            self.button.clone().on_trigger(move |window, cx| {
                let content = content.clone();
                Sheet::open(cx.new(|_| PopupSheet { content }), window, cx);
            })
        } else {
            self.button.clone()
        };
        let drawn = match (self.face, button.icon_ref()) {
            (Some(face), _) => button.button(env).ghost().label("").child(face).tooltip(button.label().clone()),
            (None, Some(_)) => button.icon_button(env),
            (None, None) => button.button(env).ghost(),
        };
        if env.is_touch() {
            return drawn.into_any_element();
        }
        let content = self.content;
        Popover::new(self.id)
            .trigger(drawn)
            .content(move |_, window, cx: &mut Context<_>| {
                let popover = cx.entity();
                let close: Handler = Rc::new(move |_, cx| popover.update(cx, |_, cx| cx.emit(DismissEvent)));
                content(&close, window, cx)
            })
            .into_any_element()
    }
}

/// A popup's content in a sheet, on a phone.
struct PopupSheet {
    content: Content,
}

impl Render for PopupSheet {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let close: Handler = Rc::new(Sheet::close);
        let scale = cx.scale();
        div().px(scale.space_l).pb(scale.space_l).child((self.content)(&close, window, cx))
    }
}
