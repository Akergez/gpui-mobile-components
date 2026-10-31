//! Commands behind a button: a toolbar's "More", a channel's menu, an
//! application's main menu.
//!
//! **Pointer**: the button opens a gpui-kit dropdown menu, the groups apart
//! by a separator, each command with its key binding and its check.
//! **Touch**: the button opens the commands as a sheet of rows
//! ([`Sheet::actions`]), destructive ones last.
//!
//! The button is a [`Command`] too: its icon, and its label as the tooltip
//! (or as the text, without an icon). What the button itself would do is
//! not run; it opens the menu.
//!
//! [`Menu::open_at`] opens commands where no button is: at the point a
//! right click landed on a link inside a message, once the text has said
//! which link it was. **Pointer**: a popup menu there, drawn by the
//! window's [`crate::AppFrame`], closed by a choice, a press outside or
//! Escape. **Touch**: the sheet of rows.

use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenu};
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, App, DismissEvent, Entity, Focusable as _, Pixels, Point, RenderOnce, SharedString, Subscription,
    Window, anchored, deferred, div, px,
};

use crate::command::{Command, in_offer_order};
use crate::env::{Env, InputMode, input_mode};
use crate::layers;
use crate::sheet::Sheet;

/// Commands offered from one button.
#[derive(IntoElement)]
pub struct Menu {
    button: Command,
    title: Option<SharedString>,
    groups: Vec<Vec<Command>>,
    choice: bool,
}

impl Menu {
    pub fn new(button: Command) -> Self {
        Menu { button, title: None, groups: vec![Vec::new()], choice: false }
    }

    /// The menu is a choice, its button showing what is chosen: outlined,
    /// with a caret, so that it reads as something to change rather than
    /// as text. An integration's select, a form's field of a few options.
    pub fn choice(mut self) -> Self {
        self.choice = true;
        self
    }

    /// What the sheet says over the commands, on a phone.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn command(mut self, command: Command) -> Self {
        self.groups.last_mut().expect("there is always a group").push(command);
        self
    }

    pub fn commands(mut self, commands: impl IntoIterator<Item = Command>) -> Self {
        self.groups.last_mut().expect("there is always a group").extend(commands);
        self
    }

    /// Starts a new group: what follows is set apart in the menu.
    pub fn separator(mut self) -> Self {
        if self.groups.last().is_some_and(|group| !group.is_empty()) {
            self.groups.push(Vec::new());
        }
        self
    }

    /// Opens `commands` at `position`, in window coordinates: a menu for a
    /// point where there is no button. The focus goes back where it was
    /// once it closes.
    pub fn open_at(
        title: Option<SharedString>,
        commands: Vec<Command>,
        position: Point<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) {
        if input_mode(cx) == InputMode::Touch {
            Sheet::actions(title, commands, window, cx);
            return;
        }
        let back = window.focused(cx);
        let commands = in_offer_order(&commands);
        let menu = PopupMenu::build(window, cx, move |menu, _, _| {
            let menu = match back.clone() {
                Some(back) => menu.action_context(back),
                None => menu,
            };
            commands.iter().fold(menu, |menu, command| command.add_to(menu))
        });
        let window_id = window.window_handle().window_id();
        let dismiss = window.subscribe(&menu, cx, move |_, _: &DismissEvent, _, cx| {
            layers::remove_menu(window_id, cx);
        });
        // Focused, so that Escape and the arrow keys reach it.
        menu.focus_handle(cx).focus(window, cx);
        layers::set_menu(window, FloatingMenu { position, menu, _dismiss: dismiss }, cx);
    }

    /// Whether a menu [`Menu::open_at`] opened is up in `window`.
    pub fn is_open_at(window: &Window, cx: &App) -> bool {
        layers::has_menu(window, cx)
    }

    fn all(&self) -> Vec<Command> {
        self.groups.iter().flatten().cloned().collect()
    }
}

impl RenderOnce for Menu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let choice = self.choice;
        let drawn = |button: &Command| match button.icon_ref() {
            _ if choice => button.button(env).outline().dropdown_caret(true),
            Some(_) => button.icon_button(env),
            None => button.button(env).ghost(),
        };
        if env.is_touch() {
            let (title, commands) = (self.title.clone(), self.all());
            let opens = self
                .button
                .clone()
                .on_trigger(move |window, cx| Sheet::actions(title.clone(), commands.clone(), window, cx));
            return drawn(&opens).into_any_element();
        }
        let groups = self.groups;
        drawn(&self.button)
            .dropdown_menu(move |menu, _, _| {
                let mut menu = menu;
                for (index, group) in groups.iter().filter(|group| !group.is_empty()).enumerate() {
                    if index > 0 {
                        menu = menu.separator();
                    }
                    menu = group.iter().fold(menu, |menu, command| command.add_to(menu));
                }
                menu
            })
            .into_any_element()
    }
}

/// A menu [`Menu::open_at`] opened, where it was asked for.
pub(crate) struct FloatingMenu {
    position: Point<Pixels>,
    menu: Entity<PopupMenu>,
    _dismiss: Subscription,
}

impl FloatingMenu {
    /// Over everything else in `window`, at its point.
    pub(crate) fn render(&self, window: &Window) -> AnyElement {
        let size = window.bounds().size;
        deferred(
            anchored().child(
                div()
                    .w(size.width)
                    .h(size.height)
                    // What is under it must not scroll away from the point
                    // the menu is about.
                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                    .child(
                        anchored()
                            .position(self.position)
                            .snap_to_window_with_margin(px(8.))
                            .child(self.menu.clone()),
                    ),
            ),
        )
        .with_priority(gpui_kit::base::POPUP_PRIORITY)
        .into_any_element()
    }
}
