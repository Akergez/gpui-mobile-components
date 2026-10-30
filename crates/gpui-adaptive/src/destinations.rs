//! The application's sections and the way between them.
//!
//! | Width    | Pointer                          | Touch                                   |
//! |----------|----------------------------------|-----------------------------------------|
//! | Compact  | a bar along the bottom           | Material's navigation bar               |
//! | Medium   | an icon rail with tooltips       | a navigation rail, the primary on top   |
//! | Expanded | a sidebar with labels and counts | a navigation rail with labels           |
//!
//! The navigation bar is only on a section's first page: deeper pages have
//! the whole height, and back. The content is drawn beside the rail or
//! above the bar, and kept clear of the system's navigation bar and the
//! keyboard: the bar goes under the keyboard, the content stays above it.
//! Use it inside an [`crate::AppFrame`] that is `edge_to_edge`.
//!
//! Choosing a destination calls `on_select` with its key; the owner usually
//! puts the section's root page in its [`crate::NavigatorState`].

use std::rc::Rc;

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::{ActiveTheme, Icon, Sizable, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, ElementId, FontWeight, RenderOnce, SharedString, Window, div, rems};

use crate::command::Command;
use crate::env::{Env, InputMode, WidthClass};
use crate::identity::Count;
use crate::insets::Insets;
use crate::palette::ActivePalette;
use crate::scale::ActiveScale;

/// One section.
#[derive(Clone)]
pub struct Destination {
    key: SharedString,
    label: SharedString,
    icon: Icon,
    count: usize,
    urgent: bool,
    pinned: bool,
}

impl Destination {
    /// `key` is the section's stable name ("inbox"), `label` what is shown.
    pub fn new(key: impl Into<SharedString>, label: impl Into<SharedString>, icon: impl Into<Icon>) -> Self {
        Destination {
            key: key.into(),
            label: label.into(),
            icon: icon.into(),
            count: 0,
            urgent: false,
            pinned: false,
        }
    }

    /// How many things wait in it; `urgent` when they wait on the person.
    pub fn count(mut self, count: usize, urgent: bool) -> Self {
        self.count = count;
        self.urgent = urgent;
        self
    }

    /// At the far end of a rail or sidebar, apart from the others: Settings.
    pub fn pinned(mut self) -> Self {
        self.pinned = true;
        self
    }

    pub fn key(&self) -> &SharedString {
        &self.key
    }
}

type OnSelect = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// The sections, around the page of the one selected.
#[derive(IntoElement)]
pub struct Destinations {
    id: ElementId,
    destinations: Vec<Destination>,
    selected: SharedString,
    on_select: Option<OnSelect>,
    header: Option<AnyElement>,
    primary: Option<Command>,
    content: Option<AnyElement>,
    bar_visible: bool,
    prefer_rail: bool,
}

impl Destinations {
    pub fn new(id: impl Into<ElementId>, selected: impl Into<SharedString>) -> Self {
        Destinations {
            id: id.into(),
            destinations: Vec::new(),
            selected: selected.into(),
            on_select: None,
            header: None,
            primary: None,
            content: None,
            bar_visible: true,
            prefer_rail: false,
        }
    }

    pub fn destination(mut self, destination: Destination) -> Self {
        self.destinations.push(destination);
        self
    }

    pub fn on_select(mut self, on_select: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(on_select));
        self
    }

    /// What heads the sidebar or the rail: the account, team or server
    /// switcher. In a rail it should be no wider than a rail's item (an
    /// [`crate::Avatar`]). A compact window shows none: the root page's app
    /// bar carries the switcher there.
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    /// The application's primary command, at the top of a rail or sidebar.
    /// On a compact window the page's [`crate::Screen`] draws it as its FAB.
    pub fn primary(mut self, command: Command) -> Self {
        self.primary = Some(command);
        self
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    /// A rail even where a sidebar fits: for an application whose sections'
    /// pages bring their own list column (aiddiction's inbox, projects and
    /// settings), so that a sidebar would be a second list beside it.
    pub fn prefer_rail(mut self, prefer_rail: bool) -> Self {
        self.prefer_rail = prefer_rail;
        self
    }

    /// Whether the compact window's navigation bar shows: on the section's
    /// first page only, usually `navigator.is_at_root()`.
    pub fn bar_visible(mut self, visible: bool) -> Self {
        self.bar_visible = visible;
        self
    }

    fn select(&self, key: SharedString) -> impl Fn(&gpui_kit::ClickEvent, &mut Window, &mut App) + 'static {
        let on_select = self.on_select.clone();
        move |_, window, cx| {
            if let Some(on_select) = &on_select {
                on_select(&key, window, cx);
            }
        }
    }

    fn render_bar(&self, env: Env, insets: Insets, rem: gpui_kit::Pixels, cx: &App) -> AnyElement {
        let palette = cx.palette();
        let scale = cx.scale();
        let theme = cx.theme();
        let (bar_bg, border) = match env.input() {
            InputMode::Touch => (palette.surface_container, palette.surface_container),
            InputMode::Pointer => (theme.sidebar, theme.sidebar_border),
        };
        h_flex()
            .flex_none()
            .w_full()
            .h(scale.navigation_bar.to_pixels(rem) + insets.bars)
            .pb(insets.bars)
            .bg(bar_bg)
            .border_t_1()
            .border_color(border)
            .children(self.destinations.iter().map(|destination| {
                let selected = destination.key == self.selected;
                let (pill_bg, pill_fg) = if selected {
                    (palette.secondary_container, palette.on_secondary_container)
                } else {
                    (gpui_kit::transparent_black(), palette.on_surface_variant)
                };
                v_flex()
                    .id(ElementId::Name(destination.key.clone()))
                    .flex_1()
                    .h_full()
                    .items_center()
                    .justify_center()
                    .gap_1()
                    .cursor_pointer()
                    .child(
                        div()
                            .relative()
                            .w(rems(4.))
                            .h(rems(2.))
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(pill_bg)
                            .text_color(pill_fg)
                            .child(destination.icon.clone())
                            .child(
                                div()
                                    .absolute()
                                    .top(rems(-0.25))
                                    .left(rems(2.25))
                                    .child(Count::new(destination.count).urgent(destination.urgent)),
                            ),
                    )
                    .child(
                        div()
                            .text_size(rems(0.75))
                            .when(selected, |label| label.font_weight(FontWeight::BOLD).text_color(palette.on_surface))
                            .when(!selected, |label| label.text_color(palette.on_surface_variant))
                            .child(destination.label.clone()),
                    )
                    .on_click(self.select(destination.key.clone()))
                .test_support()
            }))
            .into_any_element()
    }

    fn render_rail(&mut self, env: Env, labels: bool, cx: &App) -> AnyElement {
        let header = self.header.take();
        let palette = cx.palette();
        let scale = cx.scale();
        let theme = cx.theme();
        let touch = env.is_touch();
        let (rail_bg, border) = if touch { (palette.surface_container, palette.surface_container) } else { (theme.sidebar, theme.sidebar_border) };
        let item = |destination: &Destination| {
            let selected = destination.key == self.selected;
            let (bg, fg) = if selected {
                (palette.secondary_container, palette.on_secondary_container)
            } else {
                (gpui_kit::transparent_black(), palette.on_surface_variant)
            };
            let icon_box = div()
                .relative()
                .w(if touch { rems(3.5) } else { rems(2.5) })
                .h(if touch { rems(2.) } else { rems(2.5) })
                .rounded(if touch { rems(1.) } else { scale.radius_control })
                .flex()
                .items_center()
                .justify_center()
                .bg(bg)
                .text_color(fg)
                .child(destination.icon.clone())
                .child(
                    div().absolute().top(rems(-0.25)).right(rems(-0.25)).child(Count::new(destination.count).urgent(destination.urgent)),
                );
            let hover = theme.list_hover;
            v_flex()
                .id(ElementId::Name(destination.key.clone()))
                .items_center()
                .gap_1()
                .cursor_pointer()
                .when(!touch && !selected, |item| item.hover(move |style| style.bg(hover)).rounded(scale.radius_control))
                .child(icon_box)
                .when(labels || touch, |item| {
                    item.child(
                        div()
                            .text_size(rems(0.75))
                            .when(selected, |label| label.font_weight(FontWeight::BOLD))
                            .child(destination.label.clone()),
                    )
                })
                .when(!touch, |item| {
                    let label = destination.label.clone();
                    item.tooltip(move |window, cx| gpui_kit::component::tooltip::Tooltip::new(label.clone()).build(window, cx))
                })
                .on_click(self.select(destination.key.clone()))
                .test_support()
        };
        let primary = self.primary.as_ref().map(|command| {
            if touch {
                command.icon_button(env).with_size(gpui_kit::component::Size::Large).into_any_element()
            } else {
                command.icon_button(env).into_any_element()
            }
        });
        let header_gap = header.is_some().then(|| div().w(rems(1.5)).h_px().bg(border));
        v_flex()
            .flex_none()
            .w(scale.rail)
            .h_full()
            .items_center()
            .gap_3()
            .py_3()
            .bg(rail_bg)
            .border_r_1()
            .border_color(border)
            .children(header)
            .children(header_gap)
            .children(primary)
            .children(self.destinations.iter().filter(|d| !d.pinned).map(item))
            .child(div().flex_1())
            .children(self.destinations.iter().filter(|d| d.pinned).map(item))
            .into_any_element()
    }

    fn render_sidebar(&mut self, env: Env, cx: &App) -> AnyElement {
        let header = self.header.take();
        let theme = cx.theme();
        let scale = cx.scale();
        let row = |destination: &Destination| {
            let selected = destination.key == self.selected;
            let (hover, accent, accent_fg) = (theme.list_hover, theme.sidebar_accent, theme.sidebar_accent_foreground);
            h_flex()
                .id(ElementId::Name(destination.key.clone()))
                .h(scale.row)
                .px_2()
                .gap_2()
                .rounded(scale.radius_control)
                .cursor_pointer()
                .when(selected, |row| row.bg(accent).text_color(accent_fg).font_weight(FontWeight::SEMIBOLD))
                .when(!selected, |row| row.hover(move |style| style.bg(hover)))
                .child(destination.icon.clone().small())
                .child(div().flex_1().truncate().child(destination.label.clone()))
                .child(Count::new(destination.count).urgent(destination.urgent))
                .on_click(self.select(destination.key.clone()))
                .test_support()
        };
        let primary = self.primary.as_ref().map(|command| command.button(env).w_full());
        v_flex()
            .flex_none()
            .w(scale.sidebar)
            .h_full()
            .p_2()
            .gap_0p5()
            .bg(theme.sidebar)
            .text_color(theme.sidebar_foreground)
            .border_r_1()
            .border_color(theme.sidebar_border)
            .children(header.map(|header| div().pb_2().child(header)))
            .children(primary.map(|primary| div().pb_2().child(primary)))
            .children(self.destinations.iter().filter(|d| !d.pinned).map(row))
            .child(div().flex_1())
            .children(self.destinations.iter().filter(|d| d.pinned).map(row))
            .into_any_element()
    }
}

impl RenderOnce for Destinations {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let insets = Insets::of(window);
        let content = self.content.take().unwrap_or_else(|| div().into_any_element());
        match env.width() {
            WidthClass::Compact => {
                let bar = self.bar_visible.then(|| self.render_bar(env, insets, window.rem_size(), cx));
                // With the bar, the bar keeps the system's room and goes under
                // the keyboard; without it the page keeps the room itself.
                let below = if self.bar_visible { insets.keyboard_over_bars() } else { insets.typing() };
                v_flex()
                    .id(self.id.clone())
                    .size_full()
                    .child(div().flex_1().min_h_0().relative().pb(below).child(content))
                    .children(bar)
                    .into_any_element()
            }
            WidthClass::Medium => h_flex()
                .id(self.id.clone())
                .size_full()
                .child(self.render_rail(env, false, cx))
                .child(div().flex_1().min_w_0().h_full().pb(insets.typing()).child(content))
                .into_any_element(),
            WidthClass::Expanded => {
                let side = match (env.is_touch(), self.prefer_rail) {
                    (true, _) => self.render_rail(env, true, cx),
                    (false, true) => self.render_rail(env, false, cx),
                    (false, false) => self.render_sidebar(env, cx),
                };
                h_flex()
                    .id(self.id.clone())
                    .size_full()
                    .child(side)
                    .child(div().flex_1().min_w_0().h_full().pb(insets.typing()).child(content))
                    .into_any_element()
            }
        }
    }
}
