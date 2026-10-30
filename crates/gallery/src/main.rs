//! Every intent of gpui-adaptive in one window, in either skin.
//!
//! ```sh
//! build-aux/cargo.sh run -p gallery                                  # pointer, desktop size
//! GALLERY_INPUT=touch GALLERY_SIZE=400x800 build-aux/cargo.sh run -p gallery   # a phone
//! ```
//!
//! `GPUI_KEYBOARD=300` pretends an on-screen keyboard is up.

use chrono::NaiveDate;
use gpui_adaptive::records::ColumnRole;
use gpui_adaptive::{
    AppFrame, Ask, Avatar, Banner, Busy, Choice, ChoiceOption, ChoiceState, Collection, Column, Command, Composer,
    ComposerEvent, ComposerState, Confirm, DateField, DateFieldState, Destination, Destinations, Dock, Empty, Form,
    InputMode, ItemActions, ListRow, Lookup, LookupEvent, LookupState, Navigator, NavigatorState, Notice, Page,
    Presence, Record, Records, Route, Screen, SectionHeader, Segments, Skeleton, TextField, TextFieldState, Toggle,
    Tone,
};
use gpui_adaptive_colors::{Color, Options};
use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::{IconName, TitleBar, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, App, AppContext, Bounds, Context, Entity, Pixels, Render, SharedString, Subscription, Window,
    WindowBounds, WindowOptions, div, px, rems, size,
};

/// The pages of the gallery: one per group of intents, and a detail page
/// to show the navigator's columns and stack.
#[derive(Clone, Debug, PartialEq)]
enum Gallery {
    Lists,
    /// What stands beside the list when no one is open.
    Pick,
    Person(&'static str),
    Input,
    Feedback,
    Records,
}

impl Route for Gallery {
    fn companion(&self) -> Option<Self> {
        match self {
            Gallery::Person(_) => Some(Gallery::Lists),
            _ => None,
        }
    }

    fn wide_detail(&self) -> Option<Self> {
        matches!(self, Gallery::Lists).then_some(Gallery::Pick)
    }
}

const PEOPLE: [(&str, &str, Presence); 4] = [
    ("Lena Petrova", "Pushed the reconnect fix", Presence::Online),
    ("Mikk Tamm", "That explains the hangs", Presence::Away),
    ("Sara Okafor", "So parse the track id instead", Presence::Online),
    ("Anton Ivanov", "Confirmed against rtcd", Presence::Offline),
];

struct Shell {
    navigator: Entity<NavigatorState<Gallery>>,
    agent: Entity<ChoiceState>,
    amount: Entity<TextFieldState>,
    date: Entity<DateFieldState>,
    composer: Entity<ComposerState>,
    lookup: Entity<LookupState>,
    kind: SharedString,
    sync: bool,
    sent: Vec<String>,
    _subscriptions: Vec<Subscription>,
}

impl Shell {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let root = match std::env::var("GALLERY_ROUTE").as_deref() {
            Ok("input") => Gallery::Input,
            Ok("feedback") => Gallery::Feedback,
            Ok("records") => Gallery::Records,
            _ => Gallery::Lists,
        };
        let navigator = cx.new(|_| NavigatorState::new(root));
        if std::env::var("GALLERY_ROUTE").as_deref() == Ok("person") {
            navigator.update(cx, |navigator, cx| navigator.push(Gallery::Person("Lena Petrova"), cx));
        }
        if let Ok(demo) = std::env::var("GALLERY_DEMO") {
            // After the first frames, as a person would: the window's layers
            // and the toolkit's root exist by then.
            cx.spawn_in(window, async move |this, cx| {
                cx.background_executor().timer(std::time::Duration::from_millis(400)).await;
                this.update_in(cx, |shell, window, cx| shell.demo(&demo, window, cx)).ok();
            })
            .detach();
        }
        let agents = vec![
            ChoiceOption::new("claude", "Claude Code").detail("Anthropic"),
            ChoiceOption::new("codex", "Codex").detail("OpenAI"),
            ChoiceOption::new("gemini", "Gemini CLI").detail("Google"),
        ];
        let agent = cx.new(|cx| ChoiceState::new(agents, Some("claude".into()), window, cx));
        let amount = cx.new(|cx| TextFieldState::amount(window, cx));
        let date = cx.new(|cx| DateFieldState::new(NaiveDate::from_ymd_opt(2026, 10, 12), window, cx));
        let composer = cx.new(|cx| ComposerState::new("Write a message…", window, cx));
        let lookup = cx.new(|cx| LookupState::new("Add people", true, window, cx));
        let subscriptions = vec![
            cx.subscribe_in(&composer, window, |this, composer, event: &ComposerEvent, window, cx| {
                if *event == ComposerEvent::Send {
                    let text = composer.read(cx).text(cx);
                    if !text.trim().is_empty() {
                        this.sent.push(text);
                        composer.update(cx, |composer, cx| composer.clear(window, cx));
                        cx.notify();
                    }
                }
            }),
            cx.subscribe(&lookup, |_, lookup, event: &LookupEvent, cx| {
                if let LookupEvent::Query(query) = event {
                    let query = query.to_lowercase();
                    let results = PEOPLE
                        .iter()
                        .filter(|(name, ..)| !query.is_empty() && name.to_lowercase().contains(&query))
                        .map(|(name, ..)| ChoiceOption::new(*name, *name))
                        .collect();
                    lookup.update(cx, |lookup, cx| lookup.set_results(results, cx));
                }
            }),
        ];
        Shell {
            navigator,
            agent,
            amount,
            date,
            composer,
            lookup,
            kind: "expense".into(),
            sync: true,
            sent: Vec::new(),
            _subscriptions: subscriptions,
        }
    }

    /// Opens what `GALLERY_DEMO` names, for screenshots.
    fn demo(&mut self, demo: &str, window: &mut Window, cx: &mut Context<Self>) {
        match demo {
            "form" => self.open_form(window, cx),
            "notice" => Notice::new("Record deleted").action("Undo", |_, _| {}).push(window, cx),
            "confirm" => Confirm::new("leave-confirm", "Leave #design-crit?")
                .body("You will stop getting its messages.")
                .confirm(Command::new("leave-yes", "Leave channel").destructive())
                .open(window, cx),
            "actions" => ItemActions::new()
                .title("Lena Petrova")
                .quick(Command::new("reply", "Reply").icon(IconName::Undo2))
                .command(Command::new("copy", "Copy name").icon(IconName::Copy))
                .command(Command::new("delete", "Delete").icon(IconName::Delete).destructive())
                .open_sheet(window, cx),
            "lookup" => Lookup::open(&self.lookup, window, cx),
            "error" => Notice::error("Could not reach the server").push(window, cx),
            _ => {}
        }
    }

    fn open_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (amount, date, agent) = (self.amount.clone(), self.date.clone(), self.agent.clone());
        let fields = cx.new(|_| FormFields { amount: amount.clone(), date, agent });
        Form::new("New expense", fields)
            .commit("Add expense", move |window, cx| {
                let ok = amount.read(cx).number(cx).is_some();
                if !ok {
                    amount.update(cx, |field, cx| field.set_error(Some("Type an amount"), cx));
                } else {
                    Notice::new("Expense added").push(window, cx);
                }
                ok
            })
            .open(window, cx);
    }

    fn render_page(&self, route: &Gallery, page: Page, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match route {
            Gallery::Lists => self.render_lists(page, cx),
            Gallery::Pick => screen("pick", "")
                .page(&page)
                .scrolls(false)
                .content(Empty::new(IconName::Inbox, "Pick a conversation").body("Or start a new one."))
                .into_any_element(),
            Gallery::Person(name) => self.render_person(name, page, cx),
            Gallery::Input => self.render_input(window, cx),
            Gallery::Feedback => self.render_feedback(cx),
            Gallery::Records => self.render_records(),
        }
    }

    fn render_lists(&self, page: Page, cx: &mut Context<Self>) -> AnyElement {
        let navigator = self.navigator.clone();
        let open_now = match self.navigator.read(cx).top() {
            Gallery::Person(name) => Some(*name),
            _ => None,
        };
        let rows = PEOPLE.iter().map(|(name, said, presence)| {
            let initials: String = name.split(' ').filter_map(|word| word.chars().next()).collect();
            let navigator = navigator.clone();
            let open = *name;
            ListRow::new(SharedString::from(format!("person-{name}")), *name)
                .leading(Avatar::new(SharedString::from(format!("avatar-{name}")), initials).presence(*presence))
                .subtitle(*said)
                .trailing("14:05")
                .unread(*presence == Presence::Online)
                .when(*name == "Lena Petrova", |row| {
                    row.body(
                        Ask::new("permission", "Write hello.txt")
                            .detail("Claude wants to create a file in Sample repo")
                            .answer(Command::new("allow", "Allow").primary().on_trigger(|window, cx| {
                                Notice::new("Allowed").push(window, cx)
                            }))
                            .answer(Command::new("reject", "Reject")),
                    )
                })
                .selected(open_now == Some(*name))
                .actions(
                    ItemActions::new()
                        .title(*name)
                        .quick(Command::new(SharedString::from(format!("reply-{name}")), "Reply").icon(IconName::Undo2))
                        .command(Command::new(SharedString::from(format!("copy-{name}")), "Copy name").icon(IconName::Copy))
                        .command(
                            Command::new(SharedString::from(format!("delete-{name}")), "Delete")
                                .icon(IconName::Delete)
                                .destructive()
                                .on_trigger(|window, cx| {
                                    Notice::new("Conversation deleted")
                                        .action("Undo", |window, cx| Notice::new("Restored").push(window, cx))
                                        .push(window, cx)
                                }),
                        ),
                )
                .on_activate(move |_, cx| navigator.update(cx, |navigator, cx| navigator.push(Gallery::Person(open), cx)))
        });
        screen("lists", "People")
            .subtitle("Collection, ListRow, ItemActions")
            .root(true)
            .page(&page)
            .command(
                Command::new("add-people", "Add people")
                    .icon(IconName::Plus)
                    .primary()
                    .on_trigger({
                        let lookup = self.lookup.clone();
                        move |window, cx| Lookup::open(&lookup, window, cx)
                    }),
            )
            .command(Command::new("search", "Search").icon(IconName::Search))
            .command(Command::new("mark-read", "Mark all read").icon(IconName::Check).overflow())
            .content(Collection::new("people").child(SectionHeader::new("Direct messages")).children(rows))
            .into_any_element()
    }

    fn render_person(&self, name: &'static str, page: Page, cx: &mut Context<Self>) -> AnyElement {
        let sent = self.sent.iter().map(|text| {
            div().p_2().child(text.clone())
        });
        let content = v_flex()
            .size_full()
            .child(div().flex_1().min_h_0().p_4().children(sent))
            .child(
                Composer::new(&self.composer)
                    .tool(Command::new("attach", "Attach files").icon(Lucide::Paperclip))
                    .tool(Command::new("priority", "Priority").icon(IconName::CircleAlert)),
            );
        let _ = cx;
        screen("person", name)
            .subtitle("Navigator page with a Composer")
            .page(&page)
            .scrolls(false)
            .banner(Some(
                Banner::new("call", "3 people are in a call")
                    .tone(Tone::Success)
                    .action(Command::new("join", "Join").on_trigger(|window, cx| Notice::new("Joined").push(window, cx))),
            ))
            .content(content)
            .into_any_element()
    }

    fn render_input(&self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let this = cx.entity();
        let open_form = {
            let this = this.clone();
            move |window: &mut Window, cx: &mut App| this.update(cx, |shell, cx| shell.open_form(window, cx))
        };
        let content = v_flex()
            .p_4()
            .gap_4()
            .max_w(rems(36.))
            .child(
                Segments::new("kind", self.kind.clone())
                    .segment("expense", "Expenses")
                    .segment("income", "Income")
                    .on_change({
                        let this = this.clone();
                        move |key, _, cx| this.update(cx, |this, cx| {
                            this.kind = key.clone();
                            cx.notify();
                        })
                    }),
            )
            .child(TextField::new(&self.amount).label("Amount").suffix("₽").help("Either separator works"))
            .child(DateField::new(&self.date).label("Date"))
            .child(Choice::new(&self.agent).label("Agent"))
            .child(
                Toggle::new("sync", "Sync on start", self.sync)
                    .description("Pull changes when the window opens")
                    .immediate(true)
                    .on_change(move |on, _, cx| this.update(cx, |this, cx| {
                        this.sync = on;
                        cx.notify();
                    })),
            );
        screen("input", "Input")
            .subtitle("TextField, DateField, Choice, Segments, Toggle, Form")
            .root(true)
            .command(Command::new("new-expense", "New expense").icon(IconName::Plus).primary().on_trigger(open_form))
            .content(content)
            .into_any_element()
    }

    fn render_feedback(&self, _: &mut Context<Self>) -> AnyElement {
        let content = v_flex()
            .p_4()
            .gap_4()
            .child(
                Ask::new("trust", "Trust this folder?")
                    .detail("The agent will run commands in ~/aiddiction-projects/sample")
                    .answer(Command::new("trust-yes", "Trust").primary())
                    .answer(Command::new("trust-no", "Not now")),
            )
            .child(Busy::new("sync-busy").label("Syncing…"))
            .child(Busy::new("upload").fraction(0.4).label("Uploading 4 of 10"))
            .child(Skeleton::rows(2).with_avatar())
            .child(div().h(rems(18.)).child(
                Empty::new(IconName::Inbox, "Nothing needs you")
                    .body("Conversations that ask you something appear here.")
                    .action(Command::new("new-chat", "New chat").icon(IconName::Plus)),
            ))
            .child(Dock::new("call-dock").child("In call · development · 12:04"));
        screen("feedback", "Feedback")
            .subtitle("Notice, Confirm, Banner, Busy, Skeleton, Empty, Dock")
            .root(true)
            .banner(Some(Banner::new("offline", "Server offline · retrying").tone(Tone::Warning).icon(IconName::TriangleAlert)))
            .command(
                Command::new("notice", "Show notice")
                    .icon(IconName::Bell)
                    .on_trigger(|window, cx| {
                        Notice::new("Record deleted")
                            .action("Undo", |window, cx| Notice::new("Record restored").push(window, cx))
                            .push(window, cx)
                    }),
            )
            .command(
                Command::new("fail", "Show error")
                    .icon(IconName::CircleX)
                    .overflow()
                    .on_trigger(|window, cx| Notice::error("Could not reach the server").push(window, cx)),
            )
            .command(
                Command::new("leave", "Leave channel…")
                    .icon(Lucide::LogOut)
                    .destructive()
                    .on_trigger(|window, cx| {
                        Confirm::new("leave-confirm", "Leave #design-crit?")
                            .body("You will stop getting its messages.")
                            .confirm(
                                Command::new("leave-yes", "Leave channel")
                                    .destructive()
                                    .on_trigger(|window, cx| Notice::new("Left #design-crit").push(window, cx)),
                            )
                            .open(window, cx)
                    }),
            )
            .content(content)
            .into_any_element()
    }

    fn render_records(&self) -> AnyElement {
        let rows = [
            ("412", "12 Oct", "Groceries", "Weekly shop", "−1 240"),
            ("411", "12 Oct", "Pharmacy", "", "−380"),
            ("410", "11 Oct", "Transport", "Metro card", "−60"),
            ("409", "10 Oct", "Salary", "October", "+95 000"),
        ];
        screen("records", "Records")
            .subtitle("A table with room, a list without")
            .root(true)
            .content(
                Records::new("records-table")
                    .column(Column::new("date", "Date").width(rems(6.)).sortable().role(ColumnRole::Subtitle))
                    .column(Column::new("category", "Category").role(ColumnRole::Title))
                    .column(Column::new("comment", "Comment"))
                    .column(Column::new("amount", "Amount").width(rems(7.)).numeric().role(ColumnRole::Trailing))
                    .sorted_by("date", false)
                    .records(rows.iter().map(|(id, date, category, comment, amount)| {
                        Record::new(SharedString::from(format!("record-{id}")), [*date, *category, *comment, *amount])
                    })),
            )
            .into_any_element()
    }
}

/// The fields of the form in the Input page.
struct FormFields {
    amount: Entity<TextFieldState>,
    date: Entity<DateFieldState>,
    agent: Entity<ChoiceState>,
}

impl Render for FormFields {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(TextField::new(&self.amount).label("Amount").suffix("₽"))
            .child(DateField::new(&self.date).label("Date"))
            .child(Choice::new(&self.agent).label("Category"))
    }
}

impl Render for Shell {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.navigator.read(cx);
        let section: SharedString = match state.stack().first() {
            Some(Gallery::Input) => "input",
            Some(Gallery::Feedback) => "feedback",
            Some(Gallery::Records) => "records",
            _ => "lists",
        }
        .into();
        let at_root = state.stack().len() == 1;
        let this = cx.entity();
        let navigator = self.navigator.clone();
        let pages = Navigator::new("pages", &self.navigator, move |route, page, window, cx| {
            this.update(cx, |shell, cx| shell.render_page(route, page, window, cx))
        });
        let aiddiction = std::env::var("GALLERY_AIDDICTION").is_ok();
        let destinations = Destinations::new("sections", section)
            .prefer_rail(aiddiction)
            .when(aiddiction, |destinations| {
                destinations.header(Avatar::new("server", "H").square().presence(Presence::Online))
            });
        AppFrame::new("gpui-adaptive gallery").edge_to_edge(true).title_bar(!aiddiction).child(
            destinations
                .destination(Destination::new("lists", "Lists", IconName::Inbox).count(2, true))
                .destination(Destination::new("input", "Input", Lucide::SquarePen))
                .destination(Destination::new("records", "Records", Lucide::Table))
                .destination(Destination::new("feedback", "Feedback", IconName::Bell).count(3, false))
                .on_select(move |key, _, cx| {
                    let route = match key.as_ref() {
                        "input" => Gallery::Input,
                        "feedback" => Gallery::Feedback,
                        "records" => Gallery::Records,
                        _ => Gallery::Lists,
                    };
                    navigator.update(cx, |navigator, cx| navigator.replace_root(route, cx));
                })
                .bar_visible(at_root)
                .content(pages),
        )
    }
}

/// A page; its bar is the window's title bar in the aiddiction style.
fn screen(id: &'static str, title: impl Into<SharedString>) -> Screen {
    Screen::new(id, title).in_title_bar(std::env::var("GALLERY_AIDDICTION").is_ok())
}

fn window_size() -> gpui_kit::Size<Pixels> {
    std::env::var("GALLERY_SIZE")
        .ok()
        .and_then(|value| {
            let (w, h) = value.split_once('x')?;
            Some(size(px(w.parse().ok()?), px(h.parse().ok()?)))
        })
        .unwrap_or_else(|| size(px(1200.), px(780.)))
}

fn main() {
    let input = match std::env::var("GALLERY_INPUT").as_deref() {
        Ok("touch") => InputMode::Touch,
        _ => InputMode::Pointer,
    };
    gpui_kit::application().with_assets(gpui_kit::assets::AllAssets).run(move |cx| {
        gpui_kit::init(cx);
        gpui_adaptive_colors::init(Options::new(Color::from_u32(0x0b57d0)), cx);
        gpui_android_glue::init(cx);
        gpui_adaptive::init(input, cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, window_size(), cx))),
            ..TitleBar::window_options()
        };
        gpui_kit::open_window(options, cx, |window, cx| cx.new(|cx| Shell::new(window, cx)))
            .expect("failed to open the gallery");
    });
}
