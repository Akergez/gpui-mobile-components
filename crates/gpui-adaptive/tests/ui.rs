//! UI integration tests: the same screens in both skins, driven through real
//! pointer events in headless windows. Each test sets the input mode the way
//! an application's entry point does, and the window's size picks the width
//! class.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use gpui_adaptive::{
    AppFrame, Choice, ChoiceEvent, ChoiceOption, ChoiceState, Command, Destination, Destinations, InputMode,
    ListRow, Navigator, NavigatorState, Page, Route, Screen, Sheet, Toggle,
};
use gpui_kit::component::IconName;
use gpui_kit::base::TestSupportExt as _;
use gpui_kit::prelude::*;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{
    AnyElement, AppContext, Bounds, Context, Entity, IntoElement, ParentElement, PlatformInput, Render, SharedString,
    Styled, TestAppContext, TouchEvent, TouchId, TouchPhase, Window, WindowBounds, WindowOptions, AnyWindowHandle, div,
    point, px, size,
};

const PHONE: (f32, f32) = (400., 800.);
const DESKTOP: (f32, f32) = (1280., 800.);

fn setup(input: InputMode, cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_adaptive::init(input, cx);
    });
}

fn open<V: Render>(
    (w, h): (f32, f32),
    cx: &mut TestAppContext,
    build: impl FnOnce(&mut Window, &mut Context<V>) -> V + 'static,
) -> (AnyWindowHandle, Entity<V>) {
    cx.update(|cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(Default::default(), size(px(w), px(h))))),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, move |window, cx| cx.new(|cx| build(window, cx))).expect("open window")
    })
}

// A screen with a primary command: a button on a pointer, a FAB on a phone.

struct Tasks {
    created: Rc<Cell<u32>>,
}

impl Render for Tasks {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let created = self.created.clone();
        AppFrame::new("Tasks").child(
            Screen::new("tasks", "Tasks")
                .root(true)
                .command(
                    Command::new("new-task", "New task")
                        .icon(IconName::Plus)
                        .primary()
                        .on_trigger(move |_, _| created.set(created.get() + 1)),
                )
                .content(div().child("Nothing yet")),
        )
    }
}

fn primary_command_runs(input: InputMode, window_size: (f32, f32), cx: &mut TestAppContext) {
    setup(input, cx);
    let created = Rc::new(Cell::new(0));
    let shared = created.clone();
    let (handle, _) = open(window_size, cx, move |_, _| Tasks { created: shared });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("new-task", cx);
    })
    .unwrap();
    assert_eq!(created.get(), 1);
}

#[gpui_kit::test]
fn the_primary_command_is_a_button_that_runs_on_a_pointer(cx: &mut TestAppContext) {
    primary_command_runs(InputMode::Pointer, DESKTOP, cx);
}

#[gpui_kit::test]
fn the_primary_command_is_a_fab_that_runs_on_a_phone(cx: &mut TestAppContext) {
    primary_command_runs(InputMode::Touch, PHONE, cx);
}

// Destinations and a navigator: choose a section, open a page, go back.

#[derive(Clone, Debug, PartialEq)]
enum Mail {
    Inbox,
    Message(u32),
    Settings,
}

impl Route for Mail {
    fn companion(&self) -> Option<Self> {
        match self {
            Mail::Message(_) => Some(Mail::Inbox),
            _ => None,
        }
    }
}

struct MailApp {
    navigator: Entity<NavigatorState<Mail>>,
}

impl Render for MailApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let navigator = self.navigator.clone();
        let state = self.navigator.read(cx);
        let section = if matches!(state.top(), Mail::Settings) { "settings" } else { "inbox" };
        let sections = Destinations::new("sections", section)
            .destination(Destination::new("inbox", "Inbox", IconName::Inbox).count(2, true))
            .destination(Destination::new("settings", "Settings", IconName::Settings).pinned())
            .on_select({
                let navigator = navigator.clone();
                move |key, _, cx| {
                    let route = if key == "settings" { Mail::Settings } else { Mail::Inbox };
                    navigator.update(cx, |navigator, cx| navigator.replace_root(route, cx));
                }
            });
        let bar = sections.bar(window, cx);
        let pages = Navigator::new("pages", &self.navigator, {
            let navigator = navigator.clone();
            move |route, page, window, cx| render_page(route, page, &navigator, window, cx)
        })
        .bar(bar);
        AppFrame::new("Mail").edge_to_edge(true).child(sections.content(pages))
    }
}

fn render_page(
    route: &Mail,
    page: Page,
    navigator: &Entity<NavigatorState<Mail>>,
    _: &mut Window,
    _: &mut gpui_kit::App,
) -> AnyElement {
    match route {
        Mail::Inbox => {
            let navigator = navigator.clone();
            Screen::new("inbox", "Inbox")
                .root(true)
                .up(page.up())
                .content(ListRow::new("message-1", "Hello").subtitle("From Ada").on_activate(move |_, cx| {
                    navigator.update(cx, |navigator, cx| navigator.push(Mail::Message(1), cx));
                }))
                .into_any_element()
        }
        Mail::Message(id) => Screen::new("message", format!("Message {id}")).up(page.up()).into_any_element(),
        Mail::Settings => Screen::new("settings-page", "Settings").root(true).into_any_element(),
    }
}

fn open_mail(input: InputMode, window_size: (f32, f32), cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<NavigatorState<Mail>>) {
    setup(input, cx);
    let navigator = cx.update(|cx| cx.new(|_| NavigatorState::new(Mail::Inbox)));
    let shared = navigator.clone();
    let (handle, _) = open(window_size, cx, move |_, _| MailApp { navigator: shared });
    (handle, navigator)
}

#[gpui_kit::test]
fn on_a_phone_a_message_is_a_page_over_the_inbox_and_up_goes_back(cx: &mut TestAppContext) {
    let (handle, navigator) = open_mail(InputMode::Touch, PHONE, cx);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("up").is_none(), "the inbox is a root: no way up");
        window.click("message-1", cx);
        window.render_frame(cx);
        assert!(window.try_find("message-1").is_none(), "the message covers the inbox on a phone");
        window.click("up", cx);
    })
    .unwrap();
    cx.update(|cx| assert_eq!(navigator.read(cx).top(), &Mail::Inbox));
}

#[gpui_kit::test]
fn on_a_desktop_the_message_opens_beside_the_inbox(cx: &mut TestAppContext) {
    let (handle, navigator) = open_mail(InputMode::Pointer, DESKTOP, cx);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("message-1", cx);
        window.render_frame(cx);
        assert!(window.try_find("message-1").is_some(), "the inbox stays as the column beside the message");
        assert!(window.try_find("up").is_none(), "with the inbox beside it, the message has no way up");
    })
    .unwrap();
    cx.update(|cx| assert_eq!(navigator.read(cx).top(), &Mail::Message(1)));
}

#[gpui_kit::test]
fn choosing_a_destination_replaces_the_stack(cx: &mut TestAppContext) {
    for (input, window_size) in [(InputMode::Pointer, DESKTOP), (InputMode::Touch, PHONE)] {
        let (handle, navigator) = open_mail(input, window_size, cx);
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("settings", cx);
        })
        .unwrap();
        cx.update(|cx| assert_eq!(navigator.read(cx).stack(), &[Mail::Settings], "{input:?}"));
    }
}

// A choice: a select on a pointer, a sheet of options on a phone.

struct Picker {
    agent: Entity<ChoiceState>,
    chosen: Rc<Cell<Option<&'static str>>>,
    _subscription: gpui_kit::Subscription,
}

impl Picker {
    fn new(chosen: Rc<Cell<Option<&'static str>>>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let options = vec![ChoiceOption::new("claude", "Claude Code"), ChoiceOption::new("codex", "Codex")];
        let agent = cx.new(|cx| ChoiceState::new(options, Some("claude".into()), window, cx));
        let seen = chosen.clone();
        let subscription = cx.subscribe(&agent, move |_, _, event: &ChoiceEvent, _| {
            let ChoiceEvent::Change(key) = event;
            seen.set(if key.as_ref() == "codex" { Some("codex") } else { Some("claude") });
        });
        Picker { agent, chosen, _subscription: subscription }
    }
}

impl Render for Picker {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let _ = &self.chosen;
        AppFrame::new("Pick").child(div().p_4().child(Choice::new(&self.agent).id("agent").label("Agent")))
    }
}

#[gpui_kit::test]
fn on_a_phone_a_choice_opens_a_sheet_and_choosing_closes_it(cx: &mut TestAppContext) {
    setup(InputMode::Touch, cx);
    let chosen = Rc::new(Cell::new(None));
    let shared = chosen.clone();
    let (handle, picker) = open(PHONE, cx, move |window, cx| Picker::new(shared, window, cx));
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(!Sheet::is_open(window, cx));
        window.click("agent", cx);
        window.render_frame(cx);
        assert!(Sheet::is_open(window, cx), "the field opens its options in a sheet");
    })
    .unwrap();
    // The sheet slides up over a quarter of a second, by the clock.
    settle(handle, cx);
    cx.update_window(handle, |_, window, cx| {
        assert!(window.find("codex").visible(), "the options are in view once the sheet is up");
        window.click("codex", cx);
        window.render_frame(cx);
    })
    .unwrap();
    assert_eq!(chosen.get(), Some("codex"));
    cx.update(|cx| assert_eq!(picker.read(cx).agent.read(cx).selected().map(SharedString::as_ref), Some("codex")));
}

/// Lets a sheet's motion finish: it is timed by the clock, as on a phone.
fn settle(handle: AnyWindowHandle, cx: &mut TestAppContext) {
    for _ in 0..2 {
        std::thread::sleep(gpui_adaptive::Scale::touch().sheet_open + std::time::Duration::from_millis(40));
        cx.update_window(handle, |_, window, cx| window.render_frame(cx)).unwrap();
    }
}

// A toggle: the whole row is the target on a phone.

struct Settings {
    on: bool,
}

impl Render for Settings {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity();
        AppFrame::new("Settings").child(
            div().p_4().child(
                Toggle::new("sync", "Sync on start", self.on)
                    .immediate(true)
                    .on_change(move |on, _, cx| this.update(cx, |this, cx| {
                        this.on = on;
                        cx.notify();
                    })),
            ),
        )
    }
}

#[gpui_kit::test]
fn a_toggle_changes_from_its_control_in_both_skins(cx: &mut TestAppContext) {
    for (input, window_size) in [(InputMode::Pointer, DESKTOP), (InputMode::Touch, PHONE)] {
        setup(input, cx);
        let (handle, settings) = open(window_size, cx, |_, _| Settings { on: false });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("sync", cx);
        })
        .unwrap();
        cx.update(|cx| assert!(settings.read(cx).on, "{input:?}"));
    }
}

// What stands beside a root page when there is room.

#[derive(Clone, Debug, PartialEq)]
enum Chats {
    Inbox,
    Nothing,
}

impl Route for Chats {
    fn wide_detail(&self) -> Option<Self> {
        matches!(self, Chats::Inbox).then_some(Chats::Nothing)
    }
}

struct ChatsApp {
    navigator: Entity<NavigatorState<Chats>>,
}

impl Render for ChatsApp {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        AppFrame::new("Chats").child(Navigator::new("pages", &self.navigator, |route, page, _, _| match route {
            Chats::Inbox => Screen::new("inbox", "Inbox").page(&page).content(ListRow::new("chat-1", "Hello")).into_any_element(),
            Chats::Nothing => div().id("placeholder").test_support().into_any_element(),
        }))
    }
}

#[gpui_kit::test]
fn a_root_page_has_its_wide_detail_beside_it_only_where_there_is_room(cx: &mut TestAppContext) {
    for (input, window_size, shown) in [(InputMode::Pointer, DESKTOP, true), (InputMode::Touch, PHONE, false)] {
        setup(input, cx);
        let navigator = cx.update(|cx| cx.new(|_| NavigatorState::new(Chats::Inbox)));
        let shared = navigator.clone();
        let (handle, _) = open(window_size, cx, move |_, _| ChatsApp { navigator: shared });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert_eq!(window.try_find("placeholder").is_some(), shown, "{input:?}");
            assert!(window.try_find("chat-1").is_some(), "the inbox itself is always there");
            assert!(window.try_find("up").is_none(), "the inbox is a root either way");
        })
        .unwrap();
        cx.update(|cx| assert!(!navigator.read(cx).has_back(), "back never goes to the wide detail"));
    }
}

// A rail with a header, where a sidebar would fit.

struct Railed {
    chosen: Rc<Cell<bool>>,
}

impl Render for Railed {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let chosen = self.chosen.clone();
        AppFrame::new("Rail").edge_to_edge(true).child(
            Destinations::new("sections", "inbox")
                .prefer_rail(true)
                .header(gpui_adaptive::Avatar::new("server", "H").square())
                .destination(Destination::new("inbox", "Inbox", IconName::Inbox))
                .destination(Destination::new("settings", "Settings", IconName::Settings).pinned())
                .on_select(move |key, _, _| chosen.set(key == "settings"))
                .content(div().child("page")),
        )
    }
}

#[gpui_kit::test]
fn a_rail_can_be_preferred_on_a_desktop_and_carries_the_header(cx: &mut TestAppContext) {
    setup(InputMode::Pointer, cx);
    let chosen = Rc::new(Cell::new(false));
    let shared = chosen.clone();
    let (handle, _) = open(DESKTOP, cx, move |_, _| Railed { chosen: shared });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let rail = window.find("settings").bounds();
        assert!(rail.size.width < px(80.), "an icon of a rail, not a sidebar's row: {:?}", rail.size.width);
        assert!(window.find("server").bounds().top() < rail.top(), "the header heads the rail");
        window.click("settings", cx);
    })
    .unwrap();
    assert!(chosen.get());
}

// The page's bar as the window's title bar.

struct Titled {
    reviewed: Rc<Cell<bool>>,
}

impl Render for Titled {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let reviewed = self.reviewed.clone();
        AppFrame::new("Titled").title_bar(false).child(
            Screen::new("project", "Sample repo")
                .in_title_bar(true)
                .command(Command::new("review", "Review").icon(IconName::Eye).on_trigger(move |_, _| reviewed.set(true)))
                .content(div().child("board")),
        )
    }
}

#[gpui_kit::test]
fn a_screen_can_be_the_window_title_bar(cx: &mut TestAppContext) {
    setup(InputMode::Pointer, cx);
    let reviewed = Rc::new(Cell::new(false));
    let shared = reviewed.clone();
    let (handle, _) = open(DESKTOP, cx, move |_, _| Titled { reviewed: shared });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let review = window.find("review").bounds();
        assert!(review.bottom() < px(48.), "the page's commands are in the one bar at the top");
        window.click("review", cx);
    })
    .unwrap();
    assert!(reviewed.get());
}

// Answers inside a row, and a question asked in the flow.

struct Inbox {
    opened: Rc<Cell<u32>>,
    answered: Rc<Cell<u32>>,
}

impl Render for Inbox {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let (opened, answered) = (self.opened.clone(), self.answered.clone());
        AppFrame::new("Inbox").child(
            ListRow::new("session", "Write hello.txt?")
                .subtitle("Sample repo · Claude")
                .body(
                    gpui_adaptive::Ask::new("permission", "Write hello.txt")
                        .answer(Command::new("allow", "Allow").primary().on_trigger(move |_, _| answered.set(answered.get() + 1)))
                        .answer(Command::new("reject", "Reject")),
                )
                .on_activate(move |_, _| opened.set(opened.get() + 1)),
        )
    }
}

#[gpui_kit::test]
fn an_answer_inside_a_row_does_not_open_the_row(cx: &mut TestAppContext) {
    for (input, window_size) in [(InputMode::Pointer, DESKTOP), (InputMode::Touch, PHONE)] {
        setup(input, cx);
        let (opened, answered) = (Rc::new(Cell::new(0)), Rc::new(Cell::new(0)));
        let (o, a) = (opened.clone(), answered.clone());
        let (handle, _) = open(window_size, cx, move |_, _| Inbox { opened: o, answered: a });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("allow", cx);
            window.render_frame(cx);
        })
        .unwrap();
        assert_eq!((answered.get(), opened.get()), (1, 0), "{input:?}: the answer, and only the answer");
        cx.update_window(handle, |_, window, cx| {
            let row = window.find("session").bounds();
            window.click_at("session", gpui_kit::point(px(8.), px(8.)), cx);
            let _ = row;
        })
        .unwrap();
        assert_eq!(opened.get(), 1, "{input:?}: a press on the row's text opens it");
    }
}

struct Blank;

impl Render for Blank {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        AppFrame::new("Blank").child(div())
    }
}

#[gpui_kit::test]
fn a_question_asked_in_a_dialog_closes_with_its_answer(cx: &mut TestAppContext) {
    for (input, window_size) in [(InputMode::Pointer, DESKTOP), (InputMode::Touch, PHONE)] {
        setup(input, cx);
        let answered = Rc::new(Cell::new(false));
        let shared = answered.clone();
        let (handle, _) = open(window_size, cx, |_, _| Blank);
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            gpui_adaptive::Ask::new("trust", "Trust this folder?")
                .detail("The agent will run commands in it.")
                .answer(Command::new("trust-yes", "Trust").primary().on_trigger(move |_, _| shared.set(true)))
                .answer(Command::new("trust-no", "Not now"))
                .open(window, cx);
            window.render_frame(cx);
            assert!(gpui_kit::component::WindowExt::has_active_dialog(window, cx));
            window.click("trust-yes", cx);
            window.render_frame(cx);
            assert!(!gpui_kit::component::WindowExt::has_active_dialog(window, cx), "{input:?}: the answer closes it");
        })
        .unwrap();
        cx.run_until_parked();
        assert!(answered.get(), "{input:?}: and then runs");
    }
}

// A grouped list: the grouping said once, rows in their sections' order.

struct Grouped {
    opened: Rc<Cell<u32>>,
}

impl Render for Grouped {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let opened = self.opened.clone();
        AppFrame::new("Grouped").child(
            Screen::new("grouped", "Inbox").content(
                gpui_adaptive::Collection::new("conversations")
                    .section(
                        gpui_adaptive::SectionHeader::new("Needs you").tone(gpui_adaptive::Tone::Warning),
                        [ListRow::new("asking", "Write hello.txt?")
                            .subtitle("Sample repo · Claude")
                            .supporting("The agent asks to write a file")
                            .signal(Some(gpui_adaptive::Signal::Waiting(gpui_adaptive::Tone::Warning)))
                            .on_activate(move |_, _| opened.set(opened.get() + 1))],
                    )
                    .section(gpui_adaptive::SectionHeader::new("Working"), Vec::<ListRow>::new())
                    .section(
                        gpui_adaptive::SectionHeader::new("Recent"),
                        [ListRow::new("older", "Fix the build").signal(Some(gpui_adaptive::Signal::Busy))],
                    ),
            ),
        )
    }
}

#[gpui_kit::test]
fn a_grouped_list_keeps_its_sections_order_in_both_skins(cx: &mut TestAppContext) {
    for (input, window_size) in [(InputMode::Pointer, DESKTOP), (InputMode::Touch, PHONE)] {
        setup(input, cx);
        let opened = Rc::new(Cell::new(0));
        let shared = opened.clone();
        let (handle, _) = open(window_size, cx, move |_, _| Grouped { opened: shared });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            let (asking, older) = (window.find("asking").bounds(), window.find("older").bounds());
            assert!(asking.origin.y < older.origin.y, "{input:?}: Needs you comes before Recent");
            // The empty Working section has no heading: nothing between them
            // but Recent's own.
            assert!(older.origin.y - (asking.origin.y + asking.size.height) < px(64.), "{input:?}: no empty section");
            window.click("asking", cx);
        })
        .unwrap();
        assert_eq!(opened.get(), 1, "{input:?}: a row with a third line and a signal still opens");
    }
}

// A form whose commitment finishes later closes itself when it does.

struct Fields;

impl Render for Fields {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child("Address")
    }
}

#[gpui_kit::test]
fn a_form_can_be_closed_once_its_work_is_done(cx: &mut TestAppContext) {
    for (input, window_size) in [(InputMode::Pointer, DESKTOP), (InputMode::Touch, PHONE)] {
        setup(input, cx);
        let (handle, _) = open(window_size, cx, |_, _| Blank);
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            let fields = cx.new(|_| Fields);
            gpui_adaptive::Form::new("Add server", fields).commit("Add", |_, _| false).open(window, cx);
            window.render_frame(cx);
            let open = gpui_kit::component::WindowExt::has_active_dialog(window, cx) || Sheet::is_open(window, cx);
            assert!(open, "{input:?}: the form is up");
            gpui_adaptive::Form::close(window, cx);
        })
        .unwrap();
        cx.run_until_parked();
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            let open = gpui_kit::component::WindowExt::has_active_dialog(window, cx) || Sheet::is_open(window, cx);
            assert!(!open, "{input:?}: and gone");
        })
        .unwrap();
    }
}

// Options that follow another field: the agents of the chosen project.

struct Agents {
    agent: Entity<ChoiceState>,
}

impl Render for Agents {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        AppFrame::new("Agents").child(Choice::new(&self.agent).id("agent"))
    }
}

#[gpui_kit::test]
fn a_choices_options_can_be_replaced(cx: &mut TestAppContext) {
    for (input, window_size) in [(InputMode::Pointer, DESKTOP), (InputMode::Touch, PHONE)] {
        setup(input, cx);
        let (handle, view) = open(window_size, cx, |window, cx| {
            let options = vec![ChoiceOption::new("claude", "Claude Code"), ChoiceOption::new("codex", "Codex")];
            Agents { agent: cx.new(|cx| ChoiceState::searchable(options, Some("claude".into()), window, cx)) }
        });
        cx.update_window(handle, |_, window, cx| {
            let agent = view.read(cx).agent.clone();
            agent.update(cx, |agent, cx| {
                agent.set_options(vec![ChoiceOption::new("fake", "Fake agent")], Some("fake".into()), window, cx)
            });
            window.render_frame(cx);
            let agent = agent.read(cx);
            assert_eq!(agent.options().len(), 1, "{input:?}");
            assert_eq!(agent.selected_option().map(|option| option.label().clone()), Some("Fake agent".into()), "{input:?}");
        })
        .unwrap();
    }
}

// A composer while the other side works: send is stop.

struct Talk {
    composer: Entity<gpui_adaptive::ComposerState>,
    stopped: Rc<Cell<u32>>,
}

impl Render for Talk {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let stopped = self.stopped.clone();
        AppFrame::new("Talk").child(
            gpui_adaptive::Composer::new(&self.composer)
                .stop(Some(Command::new("stop", "Stop").on_trigger(move |_, _| stopped.set(stopped.get() + 1))))
                .menu(
                    Command::new("mode", "Ask first").icon(IconName::Bot),
                    vec![Command::new("mode-ask", "Ask first"), Command::new("mode-auto", "Auto")],
                    Some("mode-ask".into()),
                ),
        )
    }
}

#[gpui_kit::test]
fn a_working_composer_stops_instead_of_sending(cx: &mut TestAppContext) {
    for (input, window_size) in [(InputMode::Pointer, DESKTOP), (InputMode::Touch, PHONE)] {
        setup(input, cx);
        let stopped = Rc::new(Cell::new(0));
        let shared = stopped.clone();
        let (handle, _) = open(window_size, cx, move |window, cx| Talk {
            composer: cx.new(|cx| gpui_adaptive::ComposerState::new("Ask the agent…", window, cx)),
            stopped: shared,
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("stop", cx);
        })
        .unwrap();
        assert_eq!(stopped.get(), 1, "{input:?}: the stop button stops");
    }
}

// A command in an item's hover bar is the command's, not the item's.

struct Card {
    opened: Rc<Cell<u32>>,
    moved: Rc<Cell<u32>>,
}

impl Render for Card {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (opened, moved) = (self.opened.clone(), self.moved.clone());
        let card = div()
            .id("card")
            .w(px(240.))
            .h(px(80.))
            .on_click(move |_, _, _| opened.set(opened.get() + 1));
        let actions = gpui_adaptive::ItemActions::new()
            .quick(Command::new("move", "Move to Done").icon(IconName::ChevronRight).on_trigger(move |_, _| moved.set(moved.get() + 1)));
        AppFrame::new("Card").child(actions.attach(card, gpui_adaptive::Env::of(window, cx), cx))
    }
}

#[gpui_kit::test]
fn a_command_in_the_hover_bar_does_not_open_the_item(cx: &mut TestAppContext) {
    setup(InputMode::Pointer, cx);
    let (opened, moved) = (Rc::new(Cell::new(0)), Rc::new(Cell::new(0)));
    let (o, m) = (opened.clone(), moved.clone());
    let (handle, _) = open(DESKTOP, cx, move |_, _| Card { opened: o, moved: m });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("move", cx);
    })
    .unwrap();
    assert_eq!((moved.get(), opened.get()), (1, 0), "the command, and only the command");
}

// A field of several lines keeps its lines.

#[gpui_kit::test]
fn a_multiline_field_keeps_its_lines(cx: &mut TestAppContext) {
    setup(InputMode::Pointer, cx);
    let (handle, _) = open(DESKTOP, cx, |_, _| Blank);
    cx.update_window(handle, |_, window, cx| {
        let field = cx.new(|cx| gpui_adaptive::TextFieldState::multiline(gpui_adaptive::Purpose::Message, 2, 6, window, cx));
        field.update(cx, |field, cx| field.set_value("A=1\nB=2", window, cx));
        assert_eq!(field.read(cx).value(cx).as_ref(), "A=1\nB=2");
        assert!(field.read(cx).input().is_none(), "no one-line input inside");
    })
    .unwrap();
}

// What completes the draft is offered over the composer and picked by a press.

struct Commands {
    composer: Entity<gpui_adaptive::ComposerState>,
    picked: Rc<Cell<u32>>,
}

impl Render for Commands {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let picked = self.picked.clone();
        let review = gpui_adaptive::Suggestion::new("command-review", "/review", move |_, _| picked.set(picked.get() + 1))
            .detail("Review the changes")
            .code();
        AppFrame::new("Talk").child(gpui_adaptive::Composer::new(&self.composer).suggestions(vec![review]))
    }
}

#[gpui_kit::test]
fn a_suggestion_over_the_composer_is_picked_by_a_press(cx: &mut TestAppContext) {
    for (input, window_size) in [(InputMode::Pointer, DESKTOP), (InputMode::Touch, PHONE)] {
        setup(input, cx);
        let picked = Rc::new(Cell::new(0));
        let shared = picked.clone();
        let (handle, _) = open(window_size, cx, move |window, cx| Commands {
            composer: cx.new(|cx| gpui_adaptive::ComposerState::new("Ask the agent…", window, cx)),
            picked: shared,
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("command-review", cx);
        })
        .unwrap();
        assert_eq!(picked.get(), 1, "{input:?}: the suggestion is picked");
    }
}

// A fold, a swatch and a primitive's pressable say they were pressed.

struct Parts {
    folded: Rc<Cell<u32>>,
    swatch: Rc<std::cell::RefCell<Vec<Option<SharedString>>>>,
    pressed: Rc<Cell<u32>>,
}

impl Render for Parts {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (folded, swatch, pressed) = (self.folded.clone(), self.swatch.clone(), self.pressed.clone());
        let fold = gpui_adaptive::Fold::new("thought", "Thinking").on_toggle(move |_, _| folded.set(folded.get() + 1));
        let swatches = gpui_adaptive::Swatches::new("colors", None::<SharedString>)
            .unset("System")
            .swatch("blue", "Blue", gpui_kit::blue())
            .on_pick(move |key, _, _| swatch.borrow_mut().push(key.cloned()));
        let press = gpui_adaptive::prim::testable(
            gpui_adaptive::prim::pressable("press", cx)
                .child(gpui_adaptive::Text::body("Press"))
                .on_click(move |_, _, _| pressed.set(pressed.get() + 1)),
        );
        AppFrame::new("Parts").child(
            gpui_adaptive::Stack::new()
                .child(fold)
                .child(gpui_adaptive::Setting::new("Color").below(swatches))
                .child(press),
        )
    }
}

#[gpui_kit::test]
fn a_fold_a_swatch_and_a_pressable_answer_a_press_in_both_skins(cx: &mut TestAppContext) {
    for (input, window_size) in [(InputMode::Pointer, DESKTOP), (InputMode::Touch, PHONE)] {
        setup(input, cx);
        let (folded, swatch, pressed) = (Rc::new(Cell::new(0)), Rc::new(std::cell::RefCell::new(Vec::new())), Rc::new(Cell::new(0)));
        let shared = (folded.clone(), swatch.clone(), pressed.clone());
        let (handle, _) = open(window_size, cx, move |_, _| Parts { folded: shared.0, swatch: shared.1, pressed: shared.2 });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("thought", cx);
            window.click("blue", cx);
            window.click("press", cx);
        })
        .unwrap();
        assert_eq!(folded.get(), 1, "{input:?}: the fold toggles");
        assert_eq!(*swatch.borrow(), vec![Some(SharedString::from("blue"))], "{input:?}: the swatch is picked by its key");
        assert_eq!(pressed.get(), 1, "{input:?}: the pressable is pressed");
    }
}

// The colours by role follow the skin.

#[gpui_kit::test]
fn the_colours_by_role_follow_the_skin(cx: &mut TestAppContext) {
    use gpui_adaptive::{ActiveColors as _, ActivePalette as _};
    setup(InputMode::Touch, cx);
    cx.update(|cx| {
        let (colors, palette) = (cx.colors(), cx.palette());
        assert_eq!((colors.text, colors.mine), (palette.on_surface, palette.primary_container));
        gpui_adaptive::set_input_mode(InputMode::Pointer, cx);
        let theme = gpui_kit::component::ActiveTheme::theme(cx).clone();
        let colors = cx.colors();
        assert_eq!((colors.text, colors.mine), (theme.foreground, theme.secondary));
    });
}

// A field that drives a list takes the keys that move through it first.

struct Search {
    field: Entity<gpui_adaptive::TextFieldState>,
    ups: Rc<Cell<u32>>,
}

impl Render for Search {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let ups = self.ups.clone();
        AppFrame::new("Search").child(gpui_adaptive::TextField::new(&self.field).on_key(move |key, _, _| {
            if key != gpui_adaptive::FieldKey::Up {
                return false;
            }
            ups.set(ups.get() + 1);
            true
        }))
    }
}

#[gpui_kit::test]
fn a_field_offers_the_keys_that_move_a_list_before_the_text(cx: &mut TestAppContext) {
    use gpui_kit::component::input::{MoveDown, MoveUp};
    setup(InputMode::Pointer, cx);
    let ups = Rc::new(Cell::new(0));
    let shared = ups.clone();
    let (handle, view) = open(DESKTOP, cx, move |window, cx| Search {
        field: cx.new(|cx| gpui_adaptive::TextFieldState::new(gpui_adaptive::Purpose::Plain, window, cx)),
        ups: shared,
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let field = view.read(cx).field.clone();
        field.update(cx, |field, cx| field.focus(window, cx));
        window.render_frame(cx);
        window.dispatch_action(Box::new(MoveUp), cx);
        // Not taken: it goes on to the text, and is not counted.
        window.dispatch_action(Box::new(MoveDown), cx);
    })
    .unwrap();
    assert_eq!(ups.get(), 1);
}

// Commands at a point where there is no button: a link's menu.

struct LinkPage;

impl Render for LinkPage {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        AppFrame::new("Page").child(div().child("A link"))
    }
}

fn a_menu_opens_at_a_point(input: InputMode, window_size: (f32, f32), cx: &mut TestAppContext) -> bool {
    setup(input, cx);
    let (handle, _) = open(window_size, cx, |_, _| LinkPage);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let commands = vec![Command::new("open", "Open link"), Command::new("copy", "Copy link")];
        gpui_adaptive::Menu::open_at(None, commands, gpui_kit::point(px(100.), px(100.)), window, cx);
        window.render_frame(cx);
        match input {
            InputMode::Pointer => gpui_adaptive::Menu::is_open_at(window, cx),
            InputMode::Touch => Sheet::is_open(window, cx),
        }
    })
    .unwrap()
}

#[gpui_kit::test]
fn a_menu_at_a_point_is_a_popup_on_a_pointer(cx: &mut TestAppContext) {
    assert!(a_menu_opens_at_a_point(InputMode::Pointer, DESKTOP, cx));
}

#[gpui_kit::test]
fn a_menu_at_a_point_is_a_sheet_on_a_phone(cx: &mut TestAppContext) {
    assert!(a_menu_opens_at_a_point(InputMode::Touch, PHONE, cx));
}

// A FAB over a list: the press is the FAB's alone.

struct Rows {
    created: Rc<Cell<u32>>,
    opened: Rc<Cell<u32>>,
}

impl Render for Rows {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let created = self.created.clone();
        let rows = (0..40u64).map(|ix| {
            let opened = self.opened.clone();
            ListRow::new(("row", ix), format!("Row {ix}")).on_activate(move |_, _| opened.set(opened.get() + 1))
        });
        AppFrame::new("Rows").child(
            Screen::new("rows", "Rows")
                .root(true)
                .command(
                    Command::new("new-row", "New row")
                        .icon(IconName::Plus)
                        .primary()
                        .on_trigger(move |_, _| created.set(created.get() + 1)),
                )
                .content(gpui_adaptive::Collection::new("rows-list").children(rows)),
        )
    }
}

#[gpui_kit::test]
fn a_fab_over_a_row_takes_the_press_alone(cx: &mut TestAppContext) {
    setup(InputMode::Touch, cx);
    let (created, opened) = (Rc::new(Cell::new(0)), Rc::new(Cell::new(0)));
    let shared = (created.clone(), opened.clone());
    let (handle, _) = open(PHONE, cx, move |_, _| Rows { created: shared.0, opened: shared.1 });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("new-row", cx);
    })
    .unwrap();
    assert_eq!(created.get(), 1, "the FAB runs its command");
    assert_eq!(opened.get(), 0, "the row under the FAB does not open");
}

// Filter chips say which chip was pressed, and "All" as none.

struct Filters {
    pressed: Rc<std::cell::RefCell<Vec<Option<SharedString>>>>,
}

impl Render for Filters {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let pressed = self.pressed.clone();
        AppFrame::new("Filters").child(
            gpui_adaptive::Chips::new("chips")
                .all("All")
                .chip("food", "Food", false)
                .dot(gpui_kit::hsla(0.1, 0.8, 0.5, 1.))
                .chip("rent", "Rent", true)
                .on_toggle(move |key, _, _| pressed.borrow_mut().push(key.cloned())),
        )
    }
}

#[gpui_kit::test]
fn a_chip_is_told_by_its_key_and_all_as_none_in_both_skins(cx: &mut TestAppContext) {
    for (input, window_size) in [(InputMode::Pointer, DESKTOP), (InputMode::Touch, PHONE)] {
        setup(input, cx);
        let pressed = Rc::new(std::cell::RefCell::new(Vec::new()));
        let shared = pressed.clone();
        let (handle, _) = open(window_size, cx, move |_, _| Filters { pressed: shared });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click((gpui_kit::ElementId::from("chips"), SharedString::from("food")), cx);
            window.click((gpui_kit::ElementId::from("chips"), SharedString::from("all")), cx);
        })
        .unwrap();
        assert_eq!(*pressed.borrow(), vec![Some(SharedString::from("food")), None], "{input:?}");
    }
}

// Records that fill the page lay out only the rows in view.

struct Ledger {
    scroll: gpui_adaptive::RecordsScroll,
    opened: Rc<std::cell::RefCell<Vec<gpui_kit::ElementId>>>,
}

impl Render for Ledger {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        use gpui_adaptive::{Column, ColumnRole, Record, Records};
        let opened = self.opened.clone();
        let records = (0..1000u64).map(|ix| {
            Record::new(("record", ix), [format!("{ix}"), format!("Item {ix}"), format!("{}.00", ix * 3)])
                .tile(IconName::Star, gpui_kit::hsla(0.6, 0.7, 0.5, 1.))
        });
        AppFrame::new("Ledger").child(
            Screen::new("ledger", "Ledger").root(true).scrolls(false).content(
                Records::new("ledger-records")
                    .column(Column::new("id", "ID").grow(1.).muted())
                    .column(Column::new("name", "Name").grow(3.).role(ColumnRole::Title))
                    .column(Column::new("amount", "Amount").grow(2.).numeric().role(ColumnRole::Trailing))
                    .records(records)
                    .fill(&self.scroll)
                    .on_activate(move |id, _, _| opened.borrow_mut().push(id.clone())),
            ),
        )
    }
}

#[gpui_kit::test]
fn records_that_fill_the_page_lay_out_only_the_rows_in_view_in_both_skins(cx: &mut TestAppContext) {
    for (input, window_size) in [(InputMode::Pointer, DESKTOP), (InputMode::Touch, PHONE)] {
        setup(input, cx);
        let opened = Rc::new(std::cell::RefCell::new(Vec::new()));
        let shared = opened.clone();
        let (handle, _) = open(window_size, cx, move |_, _| Ledger { scroll: Default::default(), opened: shared });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(window.try_find(("record", 2u64)).is_some(), "{input:?}: a row in view is laid out");
            assert!(window.try_find(("record", 900u64)).is_none(), "{input:?}: a row far below is not");
            window.click(("record", 2u64), cx);
        })
        .unwrap();
        assert_eq!(*opened.borrow(), vec![gpui_kit::ElementId::from(("record", 2u64))], "{input:?}");
    }
}

// A record's pills (its tags) ride on a line it already has: a row with
// them is as tall as a row without, so a list of one height stays one.

struct Tagged;

impl Render for Tagged {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        use gpui_adaptive::{Column, ColumnRole, Pill, Record, Records};
        let color = gpui_kit::hsla(0.9, 0.6, 0.5, 1.);
        AppFrame::new("Tagged").child(
            Screen::new("tagged", "Tagged").root(true).content(
                Records::new("tagged-records")
                    .column(Column::new("name", "Name").grow(3.).role(ColumnRole::Title))
                    .column(Column::new("date", "Date").grow(2.).role(ColumnRole::Subtitle))
                    .column(Column::new("amount", "Amount").grow(2.).numeric().role(ColumnRole::Trailing))
                    .records([
                        Record::new("plain", ["Food", "2026-10-01", "5.00"]).tile(IconName::Star, color),
                        Record::new("tagged", ["Food", "2026-10-02", "7.00"])
                            .tile(IconName::Star, color)
                            .pills([Pill::new("Trip", color).icon(IconName::Map), Pill::new("Work", color)]),
                    ]),
            ),
        )
    }
}

#[gpui_kit::test]
fn a_record_with_pills_is_as_tall_as_one_without_in_both_skins(cx: &mut TestAppContext) {
    for (input, window_size) in [(InputMode::Pointer, DESKTOP), (InputMode::Touch, PHONE)] {
        setup(input, cx);
        let (handle, _) = open(window_size, cx, |_, _| Tagged);
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            let (plain, tagged) = (window.find("plain").bounds(), window.find("tagged").bounds());
            assert_eq!(plain.size.height, tagged.size.height, "{input:?}");
        })
        .unwrap();
    }
}

// A date field holds the date typed, and none while what is typed is not one.

struct Day {
    date: Entity<gpui_adaptive::DateFieldState>,
}

impl Render for Day {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        AppFrame::new("Day").child(gpui_adaptive::DateField::new(&self.date).label("Date"))
    }
}

#[gpui_kit::test]
fn a_date_field_has_no_date_while_what_is_typed_is_not_one(cx: &mut TestAppContext) {
    setup(InputMode::Pointer, cx);
    let (handle, view) = open(DESKTOP, cx, |window, cx| Day {
        date: cx.new(|cx| gpui_adaptive::DateFieldState::new(None, window, cx)),
    });
    let date = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            let date = view.read(cx).date.clone();
            date.update(cx, |date, cx| date.focus(window, cx));
            window.input("2026-10-12", cx);
            date
        })
        .unwrap();
    let typed = cx.update(|cx| date.read(cx).date());
    assert_eq!(typed, chrono::NaiveDate::from_ymd_opt(2026, 10, 12));
    cx.update_window(handle, |_, window, cx| window.input("x", cx)).unwrap();
    assert_eq!(cx.update(|cx| date.read(cx).date()), None, "a date with more typed after it is not a date");
}

// A switcher is on the page in both skins.

struct Switching;

impl Render for Switching {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        AppFrame::new("Switching").child(
            Screen::new("switching", "Records")
                .root(true)
                .switcher(gpui_adaptive::Menu::new(Command::new("account", "Card · 12.00")).choice().command(Command::new("cash", "Cash")))
                .command(Command::new("sync", "Sync").icon(IconName::Inbox))
                .content(div().child("Nothing yet")),
        )
    }
}

#[gpui_kit::test]
fn a_switcher_is_on_the_page_in_both_skins(cx: &mut TestAppContext) {
    for (input, window_size) in [(InputMode::Pointer, DESKTOP), (InputMode::Touch, PHONE)] {
        setup(input, cx);
        let (handle, _) = open(window_size, cx, |_, _| Switching);
        let found = cx
            .update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                window.try_find("account").is_some()
            })
            .unwrap();
        assert!(found, "{input:?}");
    }
}

/// A finger at (x, y) in `phase`, drawn in the frame it arrives in.
fn touch(cx: &mut TestAppContext, handle: AnyWindowHandle, phase: TouchPhase, x: f32, y: f32) {
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            PlatformInput::Touch(TouchEvent {
                id: TouchId(1),
                phase,
                position: point(px(x), px(y)),
                predicted_position: None,
                force: None,
            }),
            cx,
        );
        window.render_frame(cx);
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(16));
}

/// The compact window's navigation bar is the first page's foot: it moves
/// with the page, so a gesture of back reveals it together with the page it
/// goes to instead of leaving it to appear when the gesture has ended.
#[gpui_kit::test]
fn a_phone_page_carries_the_bar_at_its_foot_and_a_gesture_reveals_it_with_the_page(cx: &mut TestAppContext) {
    let (handle, navigator) = open_mail(InputMode::Touch, PHONE, cx);
    let resting = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.find("inbox").bounds()
        })
        .unwrap();
    assert_eq!(f32::from(resting.left()), 0., "the inbox's own bar, in place");
    assert!(f32::from(resting.top()) > 700., "at the foot of the window: {resting:?}");

    // A page over the list has no bar of its own.
    cx.update_window(handle, |_, window, cx| {
        window.click("message-1", cx);
        window.render_frame(cx);
    })
    .unwrap();
    assert!(
        cx.update_window(handle, |_, window, _| window.try_find("inbox").is_none()).unwrap(),
        "the message covers the inbox and its bar"
    );

    // Part way back: the inbox under the finger carries its bar, moved with it.
    touch(cx, handle, TouchPhase::Started, 60., 300.);
    let mut dx = 0.;
    for _ in 0..5 {
        dx += 20.;
        touch(cx, handle, TouchPhase::Moved, 60. + dx, 300.);
    }
    let under = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.find("inbox").bounds()
        })
        .unwrap();
    assert!(f32::from(under.left()) < -20., "the bar went with the page under the finger: {under:?}");
    assert_eq!(f32::from(under.bottom()), 800., "and is still at the foot of the window: {under:?}");

    // The page comes back where it was, its bar with it.
    navigator.update(cx, |navigator, cx| {
        navigator.back(cx);
    });
    let back = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.find("inbox").bounds()
        })
        .unwrap();
    assert_eq!(back, resting, "nothing of the page moves when the gesture ends");
}
