//! A page: a title, the way up, the page's commands and its content.
//!
//! **Pointer**: a toolbar row. The title and the subtitle share one line,
//! the commands are buttons with labels while the window is wide enough and
//! icons with tooltips when it is not, the rare ones are under a "⋯" menu,
//! and the primary command is the primary button at the end.
//!
//! **Touch**: a Material top app bar. The title is over the subtitle, at
//! most two commands are icons, the rest are in a sheet behind "⋮", and the
//! primary command is a floating action button over the content's corner,
//! extended with its label on a root page. A root page has the large title.
//!
//! The content is the page's one vertical scroll area, which the screen
//! owns: locked to its axis for a finger. A page that scrolls in its own way
//! (a feed, a virtual list) says [`Screen::scrolls`] `false` and fills the
//! space it is given.

use std::rc::Rc;

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::DropdownMenu as _;
use gpui_kit::component::{ActiveTheme, IconName, Sizable, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, ElementId, FontWeight, RenderOnce, SharedString, Window, div};

use crate::command::{Command, CommandRole};
use crate::env::{Env, InputMode, WidthClass};
use crate::list::Signal;
use crate::palette::ActivePalette;
use crate::scale::ActiveScale;
use crate::sheet::Sheet;
use crate::touch;

type Up = Rc<dyn Fn(&mut Window, &mut App)>;

/// A page with a title and commands.
#[derive(IntoElement)]
pub struct Screen {
    id: ElementId,
    title: SharedString,
    subtitle: Option<SharedString>,
    status: Option<(SharedString, Signal)>,
    up: Option<Up>,
    root: bool,
    commands: Vec<Command>,
    banner: Option<AnyElement>,
    content: Option<AnyElement>,
    scrolls: bool,
    column: bool,
    in_title_bar: bool,
}

impl Screen {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Screen {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            status: None,
            up: None,
            root: false,
            commands: Vec::new(),
            banner: None,
            content: None,
            scrolls: true,
            column: false,
            in_title_bar: false,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// What the page's subject is doing now, as a short word: "Needs you",
    /// "Working". **Pointer**: a tinted pill after the title, with the
    /// signal's dot or spinner. **Touch**: not shown; a phone's bar has no
    /// room, and the page itself says it (the question's card, the stop
    /// button).
    pub fn status(mut self, status: Option<(SharedString, Signal)>) -> Self {
        self.status = status;
        self
    }

    /// The way up, when there is one: [`crate::Page::up`] gives it inside a
    /// [`crate::Navigator`]. Drawn as an arrow before the title.
    pub fn up(mut self, up: Option<crate::command::Handler>) -> Self {
        self.up = up;
        self
    }

    /// Takes what the [`crate::Navigator`] says about the page: its way up,
    /// and whether it is a column beside another (where commands are icons,
    /// there being no room for labels).
    pub fn page(mut self, page: &crate::Page) -> Self {
        self.up = page.up();
        self.column = page.is_column();
        self
    }

    /// Draws the page's bar as the window's title bar, as aiddiction does:
    /// one row with the window's controls, rather than a title bar and the
    /// page's bar under it. Use it with `AppFrame::title_bar(false)`. Not for
    /// a page shown as a column (its bar stays in the column), and a phone's
    /// touch skin keeps its app bar under the status bar.
    pub fn in_title_bar(mut self, in_title_bar: bool) -> Self {
        self.in_title_bar = in_title_bar;
        self
    }

    /// A section's first page: the large title on a phone.
    pub fn root(mut self, root: bool) -> Self {
        self.root = root;
        self
    }

    /// Adds a command. The one marked [`Command::primary`] becomes the
    /// primary button or the FAB; [`Command::overflow`] ones start in the
    /// menu.
    pub fn command(mut self, command: Command) -> Self {
        self.commands.push(command);
        self
    }

    pub fn commands(mut self, commands: impl IntoIterator<Item = Command>) -> Self {
        self.commands.extend(commands);
        self
    }

    /// A standing state of the page under the bar: a [`crate::Banner`].
    pub fn banner(mut self, banner: Option<impl IntoElement>) -> Self {
        self.banner = banner.map(|banner| banner.into_any_element());
        self
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    /// Whether the screen puts the content in its scroll area (the default),
    /// or the content fills the page and scrolls itself.
    pub fn scrolls(mut self, scrolls: bool) -> Self {
        self.scrolls = scrolls;
        self
    }

    fn split(&self) -> (Option<Command>, Vec<Command>, Vec<Command>) {
        let primary = self.commands.iter().find(|c| c.role() == CommandRole::Primary).cloned();
        let (overflow, shown): (Vec<_>, Vec<_>) = self
            .commands
            .iter()
            .filter(|c| c.role() != CommandRole::Primary)
            .cloned()
            .partition(|c| matches!(c.role(), CommandRole::Overflow | CommandRole::Destructive));
        (primary, shown, overflow)
    }

    fn render_content(&mut self, env: Env) -> AnyElement {
        let content = self.content.take().unwrap_or_else(|| div().into_any_element());
        if self.scrolls {
            touch::scroll_y((self.id.clone(), "content"), env, content)
        } else {
            div().size_full().child(content).into_any_element()
        }
    }

    fn render_pointer(mut self, env: Env, cx: &mut App) -> AnyElement {
        let theme = cx.theme();
        let (border, muted) = (theme.border, theme.muted_foreground);
        let scale = cx.scale();
        let (primary, shown, overflow) = self.split();
        let roomy = env.width() == WidthClass::Expanded && !self.column;
        let up = self.up.clone().map(|up| {
            Button::new("up")
                .ghost()
                .small()
                .icon(IconName::ArrowLeft)
                .tooltip("Back")
                .on_click(move |_, window, cx| up(window, cx))
        });
        let more = (!overflow.is_empty()).then(|| {
            let overflow = overflow.clone();
            Button::new("more")
                .ghost()
                .small()
                .icon(IconName::Ellipsis)
                .tooltip("More")
                .dropdown_menu(move |menu, _, _| overflow.iter().fold(menu, |menu, c| c.add_to(menu)))
        });
        let status = self.status.clone().map(|(label, signal)| {
            let tint = match signal {
                Signal::Waiting(tone) => tone.color(cx),
                Signal::Busy | Signal::New => cx.theme().primary,
            };
            let mark = match signal {
                Signal::Busy => gpui_kit::component::spinner::Spinner::new().xsmall().into_any_element(),
                _ => div().size(gpui_kit::rems(0.5)).rounded_full().bg(tint).into_any_element(),
            };
            h_flex()
                .flex_none()
                .gap_1p5()
                .h(scale.control)
                .px_2p5()
                .rounded_full()
                .bg(tint.opacity(0.14))
                .text_color(tint)
                .text_size(scale.small)
                .font_weight(FontWeight::SEMIBOLD)
                .child(mark)
                .child(label)
        });
        let content = self.render_content(env);
        let in_title_bar = self.in_title_bar && !self.column;
        // A column beside a page in the window's title bar lines up with it:
        // one strip across the window, the column's part its own.
        let beside_title_bar = self.in_title_bar && self.column;
        let (title_bar_bg, title_bar_border) = (cx.theme().title_bar, cx.theme().title_bar_border);
        let bar = h_flex()
                    .flex_none()
                    .when(!in_title_bar && !beside_title_bar, |bar| bar.h(scale.bar).px_3().border_b_1().border_color(border))
                    .when(beside_title_bar, |bar| {
                        bar.h(gpui_kit::component::TITLE_BAR_HEIGHT)
                            .pl_3()
                            .pr_1()
                            .bg(title_bar_bg)
                            .border_b_1()
                            .border_color(title_bar_border)
                    })
                    .when(in_title_bar, |bar| bar.size_full().pr_2())
                    .gap_2()
                    .children(up)
                    .child(
                        h_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_2()
                            .items_baseline()
                            .child(
                                div()
                                    .flex_none()
                                    .max_w(gpui_kit::relative(0.7))
                                    .truncate()
                                    .text_size(scale.title)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(self.title.clone()),
                            )
                            .children(self.subtitle.clone().map(|subtitle| {
                                div().min_w_0().truncate().text_size(scale.small).text_color(muted).child(subtitle)
                            })),
                    )
                    .children(status)
                    .children(shown.iter().map(|c| {
                        if roomy { c.button(env).into_any_element() } else { c.icon_button(env).into_any_element() }
                    }))
                    .children(primary.map(|c| if roomy { c.button(env) } else { c.icon_button(env).primary() }))
                    .children(more);
        let bar = if in_title_bar { title_bar(bar, cx) } else { bar.into_any_element() };
        v_flex()
            .size_full()
            .child(bar)
            .children(self.banner)
            .child(div().flex_1().min_h_0().child(content))
            .into_any_element()
    }

    fn render_touch(mut self, env: Env, cx: &mut App) -> AnyElement {
        let palette = cx.palette();
        let scale = cx.scale();
        let (primary, mut shown, mut overflow) = self.split();
        // Two icons fit beside the title; the rest go behind "⋮".
        if shown.len() > 2 {
            overflow.splice(0..0, shown.split_off(2));
        }
        let up = self.up.clone().map(|up| {
            Button::new("up")
                .ghost()
                .large()
                .icon(IconName::ArrowLeft)
                .accessibility_label("Back")
                .on_click(move |_, window, cx| up(window, cx))
        });
        let more = (!overflow.is_empty()).then(|| {
            let (overflow, title) = (overflow.clone(), self.title.clone());
            Button::new("more")
                .ghost()
                .large()
                .icon(IconName::EllipsisVertical)
                .accessibility_label("More")
                .on_click(move |_, window, cx| Sheet::actions(Some(title.clone()), overflow.clone(), window, cx))
        });
        let large = self.root && self.up.is_none();
        let heading = v_flex()
            .flex_1()
            .min_w_0()
            .when(self.up.is_none(), |heading| heading.pl_4())
            .when(!large, |heading| {
                heading
                    .child(div().truncate().text_size(scale.title).child(self.title.clone()))
                    .children(self.subtitle.clone().map(|subtitle| {
                        div().truncate().text_size(scale.small).text_color(palette.on_surface_variant).child(subtitle)
                    }))
            });
        let large_title = large.then(|| {
            v_flex()
                .px_4()
                .pb_3()
                .child(div().text_size(scale.title_root).child(self.title.clone()))
                .children(self.subtitle.clone().map(|subtitle| {
                    div().text_size(scale.small).text_color(palette.on_surface_variant).child(subtitle)
                }))
        });
        let fab = primary.map(|command| fab(&command, self.root, cx));
        let content = self.render_content(env);
        // A desktop window drawn in the touch skin (a preview) still needs its
        // controls; a phone has none.
        let chrome = (self.in_title_bar && !self.column && !gpui_kit::is_mobile())
            .then(|| title_bar(div(), cx));
        v_flex()
            .size_full()
            .children(chrome)
            .bg(palette.surface)
            .text_color(palette.on_surface)
            .child(
                h_flex()
                    .flex_none()
                    .h(scale.bar)
                    .px_1()
                    .gap_1()
                    .children(up)
                    .child(heading)
                    .children(shown.iter().map(|c| c.icon_button(env)))
                    .children(more),
            )
            .children(large_title)
            .children(self.banner)
            .child(div().flex_1().min_h_0().relative().child(content).children(fab))
            .into_any_element()
    }
}

/// `content` in the window's title bar: gpui-kit's on a desktop, a strip
/// under the status bar on a phone.
fn title_bar(content: impl IntoElement, cx: &App) -> AnyElement {
    let frame = gpui_android_glue::chrome::Frame { below_status_bar: true, ..Default::default() };
    // The toolkit's title bar is as wide as its content at the least, and
    // would push the window's controls out for a long subtitle. Laid over it
    // the content takes the room there is, and cuts the subtitle.
    let content = div().absolute().inset_0().flex().items_center().child(content);
    gpui_android_glue::chrome::title_frame(content, frame, cx)
}

/// The floating action button: the container colour of Material's FAB,
/// 56 dp, 16 dp from the corner; with its label on a root page.
fn fab(command: &Command, extended: bool, cx: &App) -> AnyElement {
    let palette = cx.palette();
    let scale = cx.scale();
    let chosen = command.clone();
    div()
        .id(command.id().clone())
        .absolute()
        .right_4()
        .bottom_4()
        .h(scale.fab)
        .min_w(scale.fab)
        .px_4()
        .gap_3()
        .flex()
        .items_center()
        .justify_center()
        .rounded(gpui_kit::rems(1.))
        .bg(palette.primary_container)
        .text_color(palette.on_primary_container)
        .shadow_lg()
        .cursor_pointer()
        .when(command.is_disabled(), |fab| fab.opacity(0.4))
        .active(move |style| style.opacity(0.88))
        .children(command.icon_ref().cloned().map(|icon| icon.large()))
        .when(extended || command.icon_ref().is_none(), |fab| {
            fab.child(div().font_weight(FontWeight::SEMIBOLD).child(command.label().clone()))
        })
        .on_click(move |_, window, cx| chosen.run(window, cx))
        .test_support()
        .into_any_element()
}

impl RenderOnce for Screen {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        match env.input() {
            InputMode::Pointer => self.render_pointer(env, cx),
            InputMode::Touch => self.render_touch(env, cx),
        }
    }
}
