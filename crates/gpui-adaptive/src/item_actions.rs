//! What can be done to one item: a message, a channel, a record, a task.
//!
//! **Pointer**: the quick commands in a small bar at the row's top right
//! while the pointer is over it (matterfast's `hover_actions.rs`), and every
//! command in a menu on a right click.
//!
//! **Touch**: a finger has no hover and no right button, so a long press
//! opens the commands in a sheet (matterfast's `post_sheet.rs`): the quick
//! ones first, then the rest, destructive last. The press is taken on its
//! way down, ahead of text selection and of the row's own tap.

use std::rc::Rc;

use gpui_kit::component::menu::ContextMenuExt as _;
use gpui_kit::component::{ActiveTheme, h_flex};
use gpui_kit::prelude::*;
use gpui_kit::base::TestSupportExt as _;
use gpui_kit::{AnyElement, App, Div, ElementId, SharedString, Stateful, Window, div};

use crate::command::{Command, in_offer_order};
use crate::env::{Env, InputMode};
use crate::sheet::Sheet;
use crate::touch;

/// The commands of one item.
#[derive(Clone, Default)]
pub struct ItemActions {
    title: Option<SharedString>,
    quick: Vec<Command>,
    commands: Vec<Command>,
    inline: bool,
}

impl ItemActions {
    pub fn new() -> Self {
        Self::default()
    }

    /// What the sheet says it is about: the item's name.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// A command frequent enough for the hover bar. Give it an icon.
    pub fn quick(mut self, command: Command) -> Self {
        self.quick.push(command);
        self
    }

    pub fn command(mut self, command: Command) -> Self {
        self.commands.push(command);
        self
    }

    /// The item sits among others on a line (a folder's tab in a strip):
    /// its own width, rather than its row's.
    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.quick.is_empty() && self.commands.is_empty()
    }

    /// Every command, in the order offered: quick ones, the rest, the
    /// destructive last.
    pub fn all(&self) -> Vec<Command> {
        let mut all = self.quick.clone();
        all.extend(self.commands.iter().cloned());
        in_offer_order(&all)
    }

    /// Opens the commands as a sheet now: what a long press does.
    pub fn open_sheet(&self, window: &mut Window, cx: &mut App) {
        Sheet::actions(self.title.clone(), self.all(), window, cx);
    }

    /// [`Self::attach`] where there is no window at hand: the width does not
    /// matter to what is attached, only the input does.
    pub fn wrap(self, item: Stateful<Div>, cx: &App) -> AnyElement {
        let env = Env::new(crate::env::input_mode(cx), crate::env::WidthClass::Expanded);
        self.attach(item, env, cx)
    }

    /// Attaches the commands to `item`, the pressable surface of the item.
    pub fn attach(self, mut item: Stateful<Div>, env: Env, cx: &App) -> AnyElement {
        if self.is_empty() {
            return item.test_support().into_any_element();
        }
        match env.input() {
            InputMode::Pointer => {
                let theme = cx.theme();
                let id = item.interactivity().element_id.clone().unwrap_or(ElementId::Name("item".into()));
                let group: SharedString = format!("item-actions-{id:?}").into();
                let bar = (!self.quick.is_empty()).then(|| {
                    h_flex()
                        .absolute()
                        .top_0p5()
                        .right_2()
                        .p_0p5()
                        .gap_0p5()
                        .rounded(theme.radius)
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.popover)
                        .shadow_sm()
                        .opacity(0.)
                        .group_hover(group.clone(), |style| style.opacity(1.))
                        // A press on a command is the command's: it does not
                        // open the item under it.
                        .on_mouse_down(gpui_kit::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .children(self.quick.iter().map(|command| command.icon_button(env)))
                });
                let all = self.all();
                let item = item.group(group).relative().children(bar).test_support();
                div()
                    .id((id, "actions"))
                    .when(self.inline, |holder| holder.flex_none())
                    .when(!self.inline, |holder| holder.w_full())
                    .child(item)
                    .context_menu(move |menu, _, _| all.iter().fold(menu, |menu, command| command.add_to(menu)))
                    .into_any_element()
            }
            InputMode::Touch => {
                let actions = Rc::new(self);
                item.relative()
                    .child(touch::long_press(Rc::new(move |window, cx| actions.open_sheet(window, cx))))
                    .test_support()
                    .into_any_element()
            }
        }
    }
}
