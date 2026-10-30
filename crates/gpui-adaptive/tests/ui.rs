//! UI integration tests: the same screens in both skins, driven through real
//! pointer events in headless windows. Each test sets the input mode the way
//! an application's entry point does, and the window's size picks the width
//! class.

use std::cell::Cell;
use std::rc::Rc;

use gpui_adaptive::{
    AppFrame, Choice, ChoiceEvent, ChoiceOption, ChoiceState, Command, Destination, Destinations, InputMode,
    ListRow, Navigator, NavigatorState, Page, Route, Screen, Sheet, Toggle,
};
use gpui_kit::component::IconName;
use gpui_kit::base::TestSupportExt as _;
use gpui_kit::prelude::*;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{
    AnyElement, AppContext, Bounds, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    TestAppContext, Window, WindowBounds, WindowOptions, AnyWindowHandle, div, px, size,
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
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let navigator = self.navigator.clone();
        let state = self.navigator.read(cx);
        let section = if matches!(state.top(), Mail::Settings) { "settings" } else { "inbox" };
        let at_root = state.is_at_root();
        let pages = Navigator::new("pages", &self.navigator, {
            let navigator = navigator.clone();
            move |route, page, window, cx| render_page(route, page, &navigator, window, cx)
        });
        AppFrame::new("Mail").edge_to_edge(true).child(
            Destinations::new("sections", section)
                .destination(Destination::new("inbox", "Inbox", IconName::Inbox).count(2, true))
                .destination(Destination::new("settings", "Settings", IconName::Settings).pinned())
                .on_select(move |key, _, cx| {
                    let route = if key == "settings" { Mail::Settings } else { Mail::Inbox };
                    navigator.update(cx, |navigator, cx| navigator.replace_root(route, cx));
                })
                .bar_visible(at_root)
                .content(pages),
        )
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
