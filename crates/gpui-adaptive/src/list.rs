//! Lists: [`ListRow`], [`SectionHeader`] and the [`Collection`] they scroll
//! in.
//!
//! **Pointer**: dense rows (1.75 rem, 2.75 with a second line) with the
//! theme's hover, keyboard-free to the row itself, commands on hover and on
//! a right click.
//!
//! **Touch**: Material list items: 56 dp for one line, 72 for two, a
//! pressed state layer, and the commands behind a long press.
//!
//! Every row needs a stable id from the domain (`("channel", channel.id)`),
//! never a position: positions move when rows are inserted. A list of
//! thousands belongs in a virtual list (gpui-kit's `v_virtual_list`), with
//! `ListRow`s as its items.

use std::rc::Rc;

use gpui_kit::component::{ActiveTheme, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, ElementId, FontWeight, MouseButton, RenderOnce, SharedString, Window, div};

use crate::env::{Env, InputMode};
use crate::item_actions::ItemActions;
use crate::palette::ActivePalette;
use crate::scale::ActiveScale;
use crate::touch;

type Activate = Rc<dyn Fn(&mut Window, &mut App)>;

/// One item of a list.
#[derive(IntoElement)]
pub struct ListRow {
    id: ElementId,
    leading: Option<AnyElement>,
    title: SharedString,
    subtitle: Option<SharedString>,
    trailing: Option<AnyElement>,
    body: Option<AnyElement>,
    selected: bool,
    unread: bool,
    muted: bool,
    on_activate: Option<Activate>,
    actions: ItemActions,
}

impl ListRow {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        ListRow {
            id: id.into(),
            leading: None,
            title: title.into(),
            subtitle: None,
            trailing: None,
            body: None,
            selected: false,
            unread: false,
            muted: false,
            on_activate: None,
            actions: ItemActions::new(),
        }
    }

    /// An icon or an [`crate::Avatar`] before the title.
    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.leading = Some(leading.into_any_element());
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// What ends the row: a time, an amount, a [`crate::Count`].
    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    /// Something to act on inside the row, under its text: the answers to
    /// an agent's question in an inbox ([`crate::Ask`]), Accept and Decline
    /// for an invitation. A press in it is its own: it does not open the
    /// row.
    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }

    /// The row is the one open now.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Something new in it: the title in bold.
    pub fn unread(mut self, unread: bool) -> Self {
        self.unread = unread;
        self
    }

    /// Asked to be quiet: dimmed.
    pub fn muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }

    pub fn on_activate(mut self, on_activate: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(on_activate));
        self
    }

    /// The row's commands: see [`ItemActions`].
    pub fn actions(mut self, actions: ItemActions) -> Self {
        self.actions = actions;
        self
    }
}

impl RenderOnce for ListRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let palette = cx.palette();
        let theme = cx.theme();
        let touch = env.input() == InputMode::Touch;
        let has_body = self.body.is_some();
        let height = if self.subtitle.is_some() { scale.row_two_line } else { scale.row };
        let (selected_bg, selected_fg, muted_fg) = if touch {
            (palette.secondary_container, palette.on_secondary_container, palette.on_surface_variant)
        } else {
            (theme.sidebar_accent, theme.sidebar_accent_foreground, theme.muted_foreground)
        };
        let title = div()
            .truncate()
            .text_size(scale.body)
            .when(self.unread, |title| title.font_weight(FontWeight::BOLD))
            .child(self.title);
        let text = v_flex()
            .flex_1()
            .min_w_0()
            .child(title)
            .children(self.subtitle.map(|subtitle| {
                div().truncate().text_size(scale.small).text_color(muted_fg).child(subtitle)
            }))
            .children(self.body.map(|body| {
                // The row opens on a click it gets the press of; a press in
                // the body stops here.
                div()
                    .pt_1p5()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(body)
            }));
        let on_activate = self.on_activate.clone();
        let row = touch::pressable(self.id, env, cx)
            .w_full()
            .min_h(height)
            .when(touch, |row| row.px_4().gap_4().py_2())
            .when(!touch, |row| row.px_2().gap_2().rounded(scale.radius_control))
            .when(has_body, |row| row.items_start())
            .when(self.selected, |row| row.bg(selected_bg).text_color(selected_fg))
            .when(self.muted, |row| row.opacity(0.55))
            .children(self.leading)
            .child(text)
            .children(self.trailing.map(|trailing| div().flex_none().text_size(scale.small).child(trailing)))
            .when_some(on_activate, |row, on_activate| row.on_click(move |_, window, cx| on_activate(window, cx)));
        self.actions.attach(row, env, cx)
    }
}

/// A heading over a run of rows: "Favourites", "Today".
#[derive(IntoElement)]
pub struct SectionHeader {
    title: SharedString,
}

impl SectionHeader {
    pub fn new(title: impl Into<SharedString>) -> Self {
        SectionHeader { title: title.into() }
    }
}

impl RenderOnce for SectionHeader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        match env.input() {
            InputMode::Pointer => div()
                .px_2()
                .pt_3()
                .pb_1()
                .text_size(scale.small)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(cx.theme().muted_foreground)
                .child(self.title),
            InputMode::Touch => div()
                .px_4()
                .pt_4()
                .pb_1()
                .text_size(scale.small)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(cx.palette().primary)
                .child(self.title),
        }
    }
}

/// Rows in the page's scroll area: a [`ListRow`] after another, with
/// [`SectionHeader`]s between runs.
#[derive(IntoElement)]
pub struct Collection {
    id: ElementId,
    children: Vec<AnyElement>,
    scrolls: bool,
}

impl Collection {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Collection { id: id.into(), children: Vec::new(), scrolls: false }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children.extend(children.into_iter().map(|child| child.into_any_element()));
        self
    }

    /// Whether the collection scrolls by itself. Off by default: inside a
    /// [`crate::Screen`] the screen scrolls; a column of its own scrolls.
    pub fn scrolls(mut self, scrolls: bool) -> Self {
        self.scrolls = scrolls;
        self
    }
}

impl RenderOnce for Collection {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let list = v_flex()
            .w_full()
            .when(env.input() == InputMode::Pointer, |list| list.p_1().gap_px())
            .children(self.children);
        if self.scrolls { touch::scroll_y(self.id, env, list) } else { list.into_any_element() }
    }
}
