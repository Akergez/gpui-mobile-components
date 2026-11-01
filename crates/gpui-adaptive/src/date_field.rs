//! A date, typed or picked (money-manager's `date_field.rs`).
//!
//! The field is a line of text first: a date is four or eight key presses,
//! a calendar a dozen taps. It reads `2026-10-12`, `12.10.2026`, `12.10`
//! (this year) and `12` (this month). The calendar is one button away for
//! when the day of the week is what is known, and the two are one value:
//! typing a date moves the calendar to it, picking a day writes it in.
//!
//! **Pointer**: the calendar opens in a popup from the button beside the field.
//! **Touch**: the calendar opens in a sheet; the field's keyboard is digits.

use chrono::{Datelike, Local, NaiveDate};
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::calendar::{Calendar, CalendarEvent, CalendarState, Date};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::popover::Popover;
use gpui_kit::component::{IconName, Sizable, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, App, Context, Entity, EventEmitter, Render, RenderOnce, SharedString, Subscription, Window, div,
};

use crate::env::{Env, InputMode, input_mode};
use crate::scale::ActiveScale;
use crate::sheet::Sheet;

/// How a date is written back into the field.
pub const FORMAT: &str = "%Y-%m-%d";

/// Reads a typed date; short forms are read against `near`.
pub fn parse_date(text: &str, near: NaiveDate) -> Option<NaiveDate> {
    let text = text.trim();
    if let Ok(date) = NaiveDate::parse_from_str(text, FORMAT) {
        return Some(date);
    }
    let parts: Vec<u32> = text
        .split(['.', '/', ' '])
        .filter(|part| !part.is_empty())
        .map(|part| part.trim().parse().ok())
        .collect::<Option<_>>()?;
    match parts[..] {
        [day] => NaiveDate::from_ymd_opt(near.year(), near.month(), day),
        [day, month] => NaiveDate::from_ymd_opt(near.year(), month, day),
        [day, month, year] => {
            let year = if year < 100 { 2000 + year } else { year };
            NaiveDate::from_ymd_opt(i32::try_from(year).ok()?, month, day)
        }
        _ => None,
    }
}

/// What a date field tells its owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateFieldEvent {
    /// A whole date was typed or picked.
    Change(NaiveDate),
}

/// The retained state of a date field.
pub struct DateFieldState {
    input: Entity<InputState>,
    calendar: Entity<CalendarState>,
    date: Option<NaiveDate>,
    near: NaiveDate,
    open: bool,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<DateFieldEvent> for DateFieldState {}

impl DateFieldState {
    pub fn new(date: Option<NaiveDate>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let near = date.unwrap_or_else(|| Local::now().date_naive());
        let text = date.map(|date| date.format(FORMAT).to_string()).unwrap_or_default();
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("YYYY-MM-DD").default_value(text));
        let calendar = cx.new(|cx| {
            let mut calendar = CalendarState::new(window, cx);
            calendar.set_date(near, window, cx);
            calendar
        });
        let subscriptions = vec![
            cx.subscribe_in(&input, window, |this: &mut Self, input, event: &InputEvent, window, cx| {
                gpui_android_glue::keyboard::follow(input.entity_id(), event, gpui_android_glue::keyboard::Purpose::Number);
                if !matches!(event, InputEvent::Change) {
                    return;
                }
                // What is typed is the value: while it is not a date, there
                // is none, and a form asking for one says so rather than
                // saving the last date that was.
                let typed = parse_date(&input.read(cx).value(), this.near);
                if typed == this.date {
                    return;
                }
                this.date = typed;
                if let Some(date) = typed {
                    this.calendar.update(cx, |calendar, cx| calendar.set_date(date, window, cx));
                    cx.emit(DateFieldEvent::Change(date));
                }
                cx.notify();
            }),
            cx.subscribe_in(&calendar, window, |this: &mut Self, _, event: &CalendarEvent, window, cx| {
                let CalendarEvent::Selected(Date::Single(Some(date))) = event else { return };
                this.pick(*date, window, cx);
            }),
        ];
        DateFieldState { input, calendar, date, near, open: false, _subscriptions: subscriptions }
    }

    /// The date in the field, or nothing while what is typed there is not
    /// one.
    pub fn date(&self) -> Option<NaiveDate> {
        self.date
    }

    /// Puts the keyboard in the field.
    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        self.input.update(cx, |input, cx| input.focus(window, cx));
    }

    /// Sets the date from the owner, without an event.
    pub fn set_date(&mut self, date: NaiveDate, window: &mut Window, cx: &mut Context<Self>) {
        self.date = Some(date);
        self.input.update(cx, |input, cx| input.set_value(date.format(FORMAT).to_string(), window, cx));
        self.calendar.update(cx, |calendar, cx| calendar.set_date(date, window, cx));
        cx.notify();
    }

    fn pick(&mut self, date: NaiveDate, window: &mut Window, cx: &mut Context<Self>) {
        let changed = self.date != Some(date);
        self.date = Some(date);
        self.open = false;
        self.input.update(cx, |input, cx| input.set_value(date.format(FORMAT).to_string(), window, cx));
        if changed {
            cx.emit(DateFieldEvent::Change(date));
        }
        if input_mode(cx) == InputMode::Touch {
            Sheet::close(window, cx);
        }
        cx.notify();
    }

    fn toggle_calendar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match input_mode(cx) {
            InputMode::Pointer => {
                self.open = !self.open;
                cx.notify();
            }
            InputMode::Touch => {
                let calendar = self.calendar.clone();
                let sheet = cx.new(|_| CalendarSheet { calendar });
                Sheet::open(sheet, window, cx);
            }
        }
    }
}

/// The calendar in a sheet.
struct CalendarSheet {
    calendar: Entity<CalendarState>,
}

impl Render for CalendarSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let scale = cx.scale();
        v_flex()
            .px_4()
            .pb_4()
            .gap_3()
            .child(div().text_size(scale.title).child("Select date"))
            .child(Calendar::new(&self.calendar).large().w_full())
    }
}

/// Draws a [`DateFieldState`].
#[derive(IntoElement)]
pub struct DateField {
    state: Entity<DateFieldState>,
    label: Option<SharedString>,
}

impl DateField {
    pub fn new(state: &Entity<DateFieldState>) -> Self {
        DateField { state: state.clone(), label: None }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for DateField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let state = self.state.read(cx);
        let (input, calendar, open) = (state.input.clone(), state.calendar.clone(), state.open);
        let entity = self.state.clone();
        let id = self.state.entity_id();
        let field: AnyElement = if env.is_touch() {
            // A finger's calendar is a sheet, from a button at the field's end.
            let button = Button::new(("date-calendar", id))
                .ghost()
                .icon(IconName::Calendar)
                .accessibility_label("Calendar")
                .with_size(gpui_kit::component::Size::Medium)
                .on_click(move |_, window, cx| entity.update(cx, |state, cx| state.toggle_calendar(window, cx)));
            let field = Input::new(&input).suffix(button).large();
            gpui_android_glue::keyboard::on_tap(field, &input, cx).into_any_element()
        } else {
            // A pointer's calendar opens from the button beside the field, over
            // whatever is around it: a form's dialog keeps its size, and a day
            // picked is a press inside the popup, not outside the dialog.
            let popover = Popover::new(("date-calendar-popup", id))
                .anchor(gpui_kit::Anchor::TopRight)
                .trigger(
                    Button::new(("date-calendar", id))
                        .outline()
                        .small()
                        .icon(IconName::Calendar)
                        .accessibility_label("Calendar"),
                )
                .open(open)
                .on_open_change(move |open, _, cx| {
                    let open = *open;
                    entity.update(cx, |state, cx| {
                        state.open = open;
                        cx.notify();
                    });
                })
                // In rems, so that the days still fit their popup when the
                // text is drawn larger.
                .content(move |_, _, _| Calendar::new(&calendar).small().border_0().p_0().w(gpui_kit::rems(15.)));
            h_flex()
                .w_full()
                .gap_1()
                .child(div().flex_1().min_w_0().child(gpui_android_glue::keyboard::on_tap(Input::new(&input).small(), &input, cx)))
                .child(popover)
                .into_any_element()
        };
        v_flex()
            .w_full()
            .gap_1()
            .children(self.label.map(|label| {
                div().text_size(scale.small).font_weight(gpui_kit::FontWeight::MEDIUM).child(label)
            }))
            .child(field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_forms_are_read_against_the_date_near() {
        let near = NaiveDate::from_ymd_opt(2026, 10, 30).unwrap();
        let day = |y, m, d| NaiveDate::from_ymd_opt(y, m, d);
        assert_eq!(parse_date("2026-10-12", near), day(2026, 10, 12));
        assert_eq!(parse_date("12.10.2026", near), day(2026, 10, 12));
        assert_eq!(parse_date("12.10.26", near), day(2026, 10, 12));
        assert_eq!(parse_date("4.10", near), day(2026, 10, 4));
        assert_eq!(parse_date("7", near), day(2026, 10, 7));
        assert_eq!(parse_date("31.02", near), None);
        assert_eq!(parse_date("tomorrow", near), None);
    }
}
