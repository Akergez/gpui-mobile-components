//! One thing the person can do, described once and drawn wherever it is
//! offered: a toolbar button, a floating action button, a menu item, a row of
//! an action sheet.
//!
//! The Coding Guides ask for "one logical desktop command, modelled once": a
//! button, a menu item and a key binding dispatch the same Action, so that the
//! entry points cannot disagree. A [`Command`] is that value. Its shortcut is
//! not stored here: a menu reads it from the keymap for the action, so the
//! menu, the tooltip and the binding say the same thing.
//!
//! A command does its work in one of two ways. With [`Command::action`] it
//! dispatches a GPUI action from the focused element, which the owning view
//! handles with `on_action`; this is what gets a key binding and a shortcut
//! in menus. With [`Command::on_trigger`] it calls a closure, for what has no
//! binding and no focus to dispatch from (a row's "Delete" in a sheet).

use std::rc::Rc;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::{PopupMenu, PopupMenuItem};
use gpui_kit::component::{Disableable, Icon, Selectable as _, Sizable};
use gpui_kit::prelude::*;
use gpui_kit::{Action, App, ElementId, RenderOnce, SharedString, Window};

use crate::env::{Env, InputMode};

/// What a command is to the place it is offered in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CommandRole {
    /// An ordinary command: a button while there is room, then the overflow.
    #[default]
    Normal,
    /// The page's one primary command: the primary button, or the FAB.
    Primary,
    /// It destroys something: drawn in the danger colour, last in a list,
    /// never the Enter default.
    Destructive,
    /// Rare enough to live in the overflow menu from the start.
    Overflow,
}

/// Something to do with the window and the application: a command's work,
/// the way up.
pub type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

type Trigger = Handler;

/// One logical command. Cheap to clone: build it in `render`, from state.
pub struct Command {
    id: ElementId,
    label: SharedString,
    icon: Option<Icon>,
    action: Option<Box<dyn Action>>,
    trigger: Option<Trigger>,
    role: CommandRole,
    disabled: bool,
    checked: Option<bool>,
}

impl Clone for Command {
    fn clone(&self) -> Self {
        Command {
            id: self.id.clone(),
            label: self.label.clone(),
            icon: self.icon.clone(),
            action: self.action.as_ref().map(|action| action.boxed_clone()),
            trigger: self.trigger.clone(),
            role: self.role,
            disabled: self.disabled,
            checked: self.checked,
        }
    }
}

impl Command {
    /// A command with a stable `id` (derive it from the domain: `("delete",
    /// record.id)`) and a label in the imperative ("Delete record").
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Command {
            id: id.into(),
            label: label.into(),
            icon: None,
            action: None,
            trigger: None,
            role: CommandRole::Normal,
            disabled: false,
            checked: None,
        }
    }

    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Dispatches `action` when chosen. Menus show its key binding.
    pub fn action(mut self, action: impl Action) -> Self {
        self.action = Some(Box::new(action));
        self
    }

    /// Calls `trigger` when chosen. Takes precedence over [`Self::action`].
    pub fn on_trigger(mut self, trigger: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.trigger = Some(Rc::new(trigger));
        self
    }

    pub fn primary(mut self) -> Self {
        self.role = CommandRole::Primary;
        self
    }

    pub fn destructive(mut self) -> Self {
        self.role = CommandRole::Destructive;
        self
    }

    pub fn overflow(mut self) -> Self {
        self.role = CommandRole::Overflow;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// A command that is a state as well as an action: "Mute", a priority
    /// among several. Shown checked in a menu and a sheet, selected as a
    /// button.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Whether the command is a state, and if so whether it is on.
    pub fn is_checked(&self) -> Option<bool> {
        self.checked
    }

    pub fn id(&self) -> &ElementId {
        &self.id
    }

    pub fn label(&self) -> &SharedString {
        &self.label
    }

    pub fn icon_ref(&self) -> Option<&Icon> {
        self.icon.as_ref()
    }

    pub fn role(&self) -> CommandRole {
        self.role
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn is_destructive(&self) -> bool {
        self.role == CommandRole::Destructive
    }

    /// Does the command's work now, as if it had been chosen. Nothing when it
    /// is disabled.
    pub fn run(&self, window: &mut Window, cx: &mut App) {
        if self.disabled {
            return;
        }
        if let Some(trigger) = &self.trigger {
            trigger(window, cx);
        } else if let Some(action) = &self.action {
            window.dispatch_action(action.boxed_clone(), cx);
        }
    }

    /// The command as a button with its label (and icon), sized for `env`.
    pub fn button(&self, env: Env) -> Button {
        let command = self.clone();
        let button = Button::new(self.id.clone())
            .label(self.label.clone())
            .disabled(self.disabled)
            .selected(self.checked == Some(true))
            .on_click(move |_, window, cx| command.run(window, cx));
        let button = match self.icon.clone() {
            Some(icon) => button.icon(icon),
            None => button,
        };
        let button = match self.role {
            CommandRole::Primary => button.primary(),
            CommandRole::Destructive => button.danger(),
            _ => button,
        };
        match env.input() {
            InputMode::Pointer => button.small(),
            InputMode::Touch => button.large(),
        }
    }

    /// The command as an icon alone, its label in the tooltip (with the key
    /// binding, when it has an action): ghost, filled when it is primary or
    /// destructive. Falls back to the label when it has no icon.
    pub fn icon_button(&self, env: Env) -> Button {
        let Some(icon) = self.icon.clone() else {
            return self.button(env).ghost();
        };
        let command = self.clone();
        let button = Button::new(self.id.clone())
            .icon(icon)
            .disabled(self.disabled)
            .selected(self.checked == Some(true))
            .accessibility_label(self.label.clone())
            .on_click(move |_, window, cx| command.run(window, cx));
        // Ghost unless it is the page's one primary command or it destroys:
        // those are filled, so that they read as what they are.
        let button = match self.role {
            CommandRole::Primary => button.primary(),
            CommandRole::Destructive => button.danger(),
            _ => button.ghost(),
        };
        let button = match &self.action {
            Some(action) => button.tooltip_with_action(self.label.clone(), action.as_ref(), None),
            None => button.tooltip(self.label.clone()),
        };
        match env.input() {
            InputMode::Pointer => button.small(),
            InputMode::Touch => button.large(),
        }
    }

    /// Adds the command to a popup menu: with its key binding when it has an
    /// action, checked when it is a state.
    pub fn add_to(&self, menu: PopupMenu) -> PopupMenu {
        let item = PopupMenuItem::new(self.label.clone())
            .disabled(self.disabled)
            .checked(self.checked == Some(true));
        let item = match self.icon.clone() {
            Some(icon) => item.icon(icon),
            None => item,
        };
        let item = match (&self.trigger, &self.action) {
            (None, Some(action)) => item.action(action.boxed_clone()),
            _ => {
                let command = self.clone();
                item.on_click(move |_, window, cx| command.run(window, cx))
            }
        };
        menu.item(item)
    }
}

impl Command {
    /// The command as a button in the layout, drawn for the skin it is
    /// drawn in: [`Self::button`] without having to ask for the [`Env`].
    pub fn as_button(self) -> CommandButton {
        CommandButton { command: self, icon_only: false, ghost: false, full_width: false, dense: false }
    }

    /// The command as an icon in the layout: [`Self::icon_button`].
    pub fn as_icon_button(self) -> CommandButton {
        CommandButton { command: self, icon_only: true, ghost: true, full_width: false, dense: false }
    }
}

/// A [`Command`] placed in the layout as a button.
#[derive(IntoElement)]
pub struct CommandButton {
    command: Command,
    icon_only: bool,
    ghost: bool,
    full_width: bool,
    dense: bool,
}

impl CommandButton {
    /// Without a fill until hovered: a toolbar's button among others.
    pub fn ghost(mut self) -> Self {
        self.ghost = true;
        self
    }

    /// Smaller on a pointer, inside a chip or a dense row: an attachment's
    /// "Remove". A finger's target does not shrink.
    pub fn dense(mut self) -> Self {
        self.dense = true;
        self
    }

    /// As wide as what holds it: a form's one commitment on a phone.
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }
}

impl RenderOnce for CommandButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let button = if self.icon_only { self.command.icon_button(env) } else { self.command.button(env) };
        let button = if self.ghost && self.command.role == CommandRole::Normal { button.ghost() } else { button };
        let button = if self.dense && !env.is_touch() { button.xsmall() } else { button };
        if self.full_width { button.w_full() } else { button }
    }
}

/// Puts destructive commands last, keeping the order of the rest: how every
/// list of commands is shown.
pub fn in_offer_order(commands: &[Command]) -> Vec<Command> {
    let (mut kept, destructive): (Vec<_>, Vec<_>) =
        commands.iter().cloned().partition(|command| !command.is_destructive());
    kept.extend(destructive);
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destructive_commands_are_offered_last_and_the_rest_keep_their_order() {
        let commands = [
            Command::new("delete", "Delete").destructive(),
            Command::new("edit", "Edit"),
            Command::new("copy", "Copy"),
        ];
        let order: Vec<_> = in_offer_order(&commands).iter().map(|c| c.label().to_string()).collect();
        assert_eq!(order, ["Edit", "Copy", "Delete"]);
    }

    #[test]
    fn a_clone_keeps_the_role_and_the_disabled_state() {
        let command = Command::new("x", "X").primary().disabled(true);
        let clone = command.clone();
        assert_eq!(clone.role(), CommandRole::Primary);
        assert!(clone.is_disabled());
    }
}
