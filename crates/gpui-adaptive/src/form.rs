//! A titled set of fields with Cancel and one commitment: "New expense —
//! Add expense".
//!
//! The fields are the owner's view (its [`crate::TextField`]s,
//! [`crate::Choice`]s, [`crate::DateField`]s, laid out as it likes), so
//! that the form keeps its state wherever it is shown. `commit` saves it and
//! answers whether it was acceptable; the form goes away only if it was.
//! The owner shows what is wrong with [`crate::TextFieldState::set_error`].
//!
//! **Pointer**: a gpui-kit dialog (aiddiction's `ui/forms.rs`): the title,
//! the fields, Cancel and the commitment, Enter commits, Escape cancels. On
//! a tablet the dialog moves up with the keyboard.
//! **Touch**: a bottom sheet (money-manager's `open_form`): the title at
//! the top with the commitment beside it, the fields scrolling under it, the
//! whole sheet above the keyboard. Back closes it.

use std::rc::Rc;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::dialog::{DialogAction, DialogClose, DialogFooter};
use gpui_kit::component::{ActiveTheme, Sizable, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{AnyView, App, Context, FocusHandle, Render, SharedString, Window, div};

use crate::env::{InputMode, input_mode};
use crate::insets::Insets;
use crate::scale::ActiveScale;
use crate::sheet::Sheet;

type Commit = Rc<dyn Fn(&mut Window, &mut App) -> bool>;

/// A form to open over the page.
pub struct Form {
    title: SharedString,
    fields: AnyView,
    commit_label: SharedString,
    commit: Option<Commit>,
    first: Option<FocusHandle>,
}

impl Form {
    /// `fields` is the owner's view of the fields.
    pub fn new(title: impl Into<SharedString>, fields: impl Into<AnyView>) -> Self {
        Form { title: title.into(), fields: fields.into(), commit_label: "Save".into(), commit: None, first: None }
    }

    /// The one commitment: its label ("Add expense") and what it does,
    /// answering whether the form was acceptable. Without it the form is a
    /// panel with Done.
    pub fn commit(
        mut self,
        label: impl Into<SharedString>,
        commit: impl Fn(&mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.commit_label = label.into();
        self.commit = Some(Rc::new(commit));
        self
    }

    /// What gets the focus once the form is up.
    pub fn focus(mut self, first: FocusHandle) -> Self {
        self.first = Some(first);
        self
    }

    pub fn open(self, window: &mut Window, cx: &mut App) {
        let first = self.first.clone();
        match input_mode(cx) {
            InputMode::Pointer => open_dialog(self, window, cx),
            InputMode::Touch => {
                let sheet = cx.new(|_| FormSheet { form: self });
                Sheet::open(sheet, window, cx);
            }
        }
        if let Some(first) = first {
            first.focus(window, cx);
        }
    }
}

/// How far down the window the dialog rests, as a share of its height.
const DIALOG_RESTS_AT: f32 = 0.1;

fn open_dialog(form: Form, window: &mut Window, cx: &mut App) {
    let Form { title, fields, commit_label, commit, .. } = form;
    gpui_android_glue::back::open_dialog(window, cx, move |dialog, window, _| {
        // With a keyboard up (a tablet), the dialog moves to the top of what
        // is left and its fields scroll inside that.
        let insets = Insets::of(window);
        let height = window.viewport_size().height;
        let rem = window.rem_size();
        let top = if insets.keyboard > gpui_kit::Pixels::ZERO { insets.top + rem } else { height * DIALOG_RESTS_AT };
        let room = (height - insets.keyboard - top - rem * 8.).max(rem * 6.);
        let body = div()
            .id("form-body")
            .max_h(room)
            .overflow_y_scroll()
            .child(fields.clone());
        let footer = match commit.clone() {
            Some(_) => DialogFooter::new()
                .child(DialogClose::new().child(Button::new("form-cancel").label("Cancel")))
                .child(DialogAction::new().child(Button::new("form-commit").label(commit_label.clone()).primary())),
            None => DialogFooter::new()
                .child(DialogClose::new().child(Button::new("form-done").label("Done").primary())),
        };
        let commit = commit.clone();
        dialog
            .margin_top(top)
            .title(title.clone())
            .child(body)
            .footer(footer)
            .on_ok(move |_, window, cx| commit.as_ref().is_none_or(|commit| commit(window, cx)))
    });
}

/// A form in a bottom sheet.
struct FormSheet {
    form: Form,
}

impl Render for FormSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let scale = cx.scale();
        let border = cx.theme().border;
        let commit = self.form.commit.clone().map(|commit| {
            Button::new("form-commit")
                .label(self.form.commit_label.clone())
                .primary()
                .large()
                .on_click(move |_, window, cx| {
                    if commit(window, cx) {
                        Sheet::close(window, cx);
                    }
                })
        });
        let dismiss = if commit.is_some() { "Cancel" } else { "Done" };
        v_flex()
            .flex_grow(1.)
            .min_h_0()
            .child(
                h_flex()
                    .flex_none()
                    .px_4()
                    .pb_3()
                    .gap_2()
                    .child(div().flex_1().text_size(scale.title).child(self.form.title.clone()))
                    .child(Button::new("form-cancel").label(dismiss).ghost().large().on_click(|_, window, cx| {
                        Sheet::close(window, cx)
                    }))
                    .children(commit),
            )
            .child(
                div()
                    .id("form-body")
                    .flex_grow(1.)
                    .flex_shrink(1.)
                    .min_h_0()
                    .border_t_1()
                    .border_color(border)
                    .overflow_y_scroll()
                    .child(div().px_4().py_4().child(self.form.fields.clone())),
            )
    }
}
