//! A question before something that cannot be undone: leave a channel,
//! remove a member, delete an account.
//!
//! Both skins open a gpui-kit dialog through
//! `gpui_android_glue::back::open_dialog`, so that a phone's back closes it.
//! The touch skin gives the buttons Material's height. The confirming button
//! names the action ("Leave channel"), never "OK", and a destructive one is
//! never the Enter default: Enter cancels it.
//!
//! For what can be undone, do not ask: do it and offer
//! [`crate::Notice::action`] "Undo" (Coding Guides, "no confirmation dialogs
//! for reversible, low-risk actions").

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::dialog::{DialogAction, DialogClose, DialogFooter};
use gpui_kit::component::{ActiveTheme, Sizable};
use gpui_kit::prelude::*;
use gpui_kit::{App, ElementId, SharedString, Window, div};

use crate::command::Command;
use crate::env::{InputMode, input_mode};

/// A question with a confirming command and Cancel.
#[derive(Clone)]
pub struct Confirm {
    id: ElementId,
    title: SharedString,
    body: Option<SharedString>,
    confirm: Option<Command>,
    cancel: SharedString,
}

impl Confirm {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Confirm { id: id.into(), title: title.into(), body: None, confirm: None, cancel: "Cancel".into() }
    }

    /// What will happen, in a sentence or two.
    pub fn body(mut self, body: impl Into<SharedString>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// The command confirming runs. Mark it destructive when it destroys.
    pub fn confirm(mut self, command: Command) -> Self {
        self.confirm = Some(command);
        self
    }

    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel = label.into();
        self
    }

    pub fn open(self, window: &mut Window, cx: &mut App) {
        let touch = input_mode(cx) == InputMode::Touch;
        gpui_android_glue::back::open_dialog(window, cx, move |dialog, _, cx| {
            let muted = cx.theme().muted_foreground;
            let confirm = self.confirm.clone();
            let (label, destructive) = confirm
                .as_ref()
                .map(|c| (c.label().clone(), c.is_destructive()))
                .unwrap_or_else(|| ("OK".into(), false));
            let ok = Button::new((self.id.clone(), "confirm")).label(label);
            let ok = if destructive { ok.danger() } else { ok.primary() };
            let cancel = Button::new((self.id.clone(), "cancel")).label(self.cancel.clone());
            let (ok, cancel) = if touch { (ok.large(), cancel.large().ghost()) } else { (ok, cancel) };
            let run = confirm.clone();
            dialog
                .title(self.title.clone())
                .children(self.body.clone().map(|body| div().text_color(muted).child(body)))
                .footer(
                    DialogFooter::new()
                        .child(DialogClose::new().child(cancel))
                        .child(DialogAction::new().child(ok)),
                )
                .on_ok(move |_, window, cx| {
                    if let Some(command) = &run {
                        let command = command.clone();
                        window.defer(cx, move |window, cx| command.run(window, cx));
                    }
                    true
                })
        });
    }
}
