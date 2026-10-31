//! One option out of a known list: a project, an agent, a category, an
//! account, a theme.
//!
//! **Pointer**: gpui-kit's `Select`, searchable from eight options on, with
//! its keyboard navigation and type-ahead.
//! **Touch**: a field a finger presses, which opens the options as rows of a
//! sheet with the chosen one marked; choosing one closes it.
//!
//! The backend is chosen once, in [`ChoiceState::new`], by the application's
//! input mode. The value is controlled by the state: the owner subscribes to
//! [`ChoiceEvent::Change`] and may set it with [`ChoiceState::set_selected`].

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::IndexPath;
use gpui_kit::component::searchable_list::SearchableListItem;
use gpui_kit::component::select::{SearchableVec, Select, SelectEvent, SelectState};
use gpui_kit::component::{ActiveTheme, Icon, IconName, Sizable, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{
    App, Context, ElementId, Entity, EventEmitter, Render, RenderOnce, SharedString, Subscription, Window, div,
};

use crate::env::{Env, InputMode, input_mode};
use crate::palette::ActivePalette;
use crate::scale::ActiveScale;
use crate::sheet::Sheet;
use crate::touch;

/// From this many options on, the pointer's select gets a search field.
pub const SEARCHABLE_FROM: usize = 8;

/// An option.
#[derive(Clone, Debug, PartialEq)]
pub struct ChoiceOption {
    key: SharedString,
    label: SharedString,
    detail: Option<SharedString>,
}

impl ChoiceOption {
    pub fn new(key: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        ChoiceOption { key: key.into(), label: label.into(), detail: None }
    }

    /// A second, quieter line: where a project is, what an agent is.
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn key(&self) -> &SharedString {
        &self.key
    }

    pub fn label(&self) -> &SharedString {
        &self.label
    }
}

impl SearchableListItem for ChoiceOption {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.label.clone()
    }

    fn value(&self) -> &SharedString {
        &self.key
    }
}

/// What a choice tells its owner.
#[derive(Clone, Debug, PartialEq)]
pub enum ChoiceEvent {
    /// The person chose the option with this key.
    Change(SharedString),
}

enum Backend {
    Pointer { select: Entity<SelectState<SearchableVec<ChoiceOption>>>, _subscription: Subscription },
    Touch,
}

/// The retained state of a choice.
pub struct ChoiceState {
    options: Vec<ChoiceOption>,
    selected: Option<SharedString>,
    backend: Backend,
}

impl EventEmitter<ChoiceEvent> for ChoiceState {}

impl ChoiceState {
    pub fn new(
        options: Vec<ChoiceOption>,
        selected: Option<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::build(options, selected, false, window, cx)
    }

    /// A choice the pointer finds by typing, however few the options: a
    /// list that grows with what the person has (projects, agents), so that
    /// many are as quick to pick as two.
    pub fn searchable(
        options: Vec<ChoiceOption>,
        selected: Option<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::build(options, selected, true, window, cx)
    }

    fn build(
        options: Vec<ChoiceOption>,
        selected: Option<SharedString>,
        search: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let backend = match input_mode(cx) {
            InputMode::Touch => Backend::Touch,
            InputMode::Pointer => {
                let index = selected
                    .as_ref()
                    .and_then(|key| options.iter().position(|option| &option.key == key))
                    .map(|row| IndexPath::default().row(row));
                let searchable = search || options.len() >= SEARCHABLE_FROM;
                let items = SearchableVec::new(options.clone());
                let select = cx.new(|cx| SelectState::new(items, index, window, cx).searchable(searchable));
                let subscription = cx.subscribe(
                    &select,
                    |this: &mut Self, _, event: &SelectEvent<SearchableVec<ChoiceOption>>, cx| {
                        let SelectEvent::Confirm(Some(key)) = event else { return };
                        this.choose(key.clone(), cx);
                    },
                );
                Backend::Pointer { select, _subscription: subscription }
            }
        };
        ChoiceState { options, selected, backend }
    }

    pub fn options(&self) -> &[ChoiceOption] {
        &self.options
    }

    pub fn selected(&self) -> Option<&SharedString> {
        self.selected.as_ref()
    }

    pub fn selected_option(&self) -> Option<&ChoiceOption> {
        let key = self.selected.as_ref()?;
        self.options.iter().find(|option| &option.key == key)
    }

    /// Sets the chosen option from the owner, without an event.
    pub fn set_selected(&mut self, key: Option<SharedString>, window: &mut Window, cx: &mut Context<Self>) {
        self.selected = key.clone();
        if let Backend::Pointer { select, .. } = &self.backend
            && let Some(key) = key
        {
            select.update(cx, |select, cx| select.set_selected_value(&key, window, cx));
        }
        cx.notify();
    }

    /// Puts other options in place of these, choosing `selected` among them
    /// without an event: the agents of the project chosen in the field
    /// above.
    pub fn set_options(
        &mut self,
        options: Vec<ChoiceOption>,
        selected: Option<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Backend::Pointer { select, .. } = &self.backend {
            let index = selected
                .as_ref()
                .and_then(|key| options.iter().position(|option| &option.key == key))
                .map(|row| IndexPath::default().row(row));
            let items = SearchableVec::new(options.clone());
            select.update(cx, |select, cx| {
                select.set_items(items, window, cx);
                select.set_selected_index(index, window, cx);
            });
        }
        self.options = options;
        self.selected = selected;
        cx.notify();
    }

    /// The person chose `key`.
    fn choose(&mut self, key: SharedString, cx: &mut Context<Self>) {
        if self.selected.as_ref() == Some(&key) {
            return;
        }
        self.selected = Some(key.clone());
        cx.emit(ChoiceEvent::Change(key));
        cx.notify();
    }
}

/// Draws a [`ChoiceState`].
#[derive(IntoElement)]
pub struct Choice {
    state: Entity<ChoiceState>,
    id: Option<ElementId>,
    label: Option<SharedString>,
    placeholder: SharedString,
}

impl Choice {
    pub fn new(state: &Entity<ChoiceState>) -> Self {
        Choice { state: state.clone(), id: None, label: None, placeholder: "Choose…".into() }
    }

    /// The field's id, for tests and scenarios; the state's identity
    /// otherwise.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// What shows while nothing is chosen.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
}

impl RenderOnce for Choice {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let scale = cx.scale();
        let state = self.state.read(cx);
        let label = self.label.clone().map(|label| {
            div().text_size(scale.small).font_weight(gpui_kit::FontWeight::MEDIUM).child(label)
        });
        let field = match &state.backend {
            Backend::Pointer { select, .. } => Select::new(select)
                .when_some(self.id.clone(), |select, id| select.id(id))
                .placeholder(self.placeholder.clone())
                .search_placeholder("Search")
                .small()
                .into_any_element(),
            Backend::Touch => {
                let palette = cx.palette();
                let shown = state.selected_option().map(|option| option.label.clone());
                let has_value = shown.is_some();
                let entity = self.state.clone();
                let title = self.label.clone();
                let id = self.id.clone().unwrap_or_else(|| ElementId::from(("choice", self.state.entity_id())));
                touch::pressable(id, env, cx)
                    .w_full()
                    .h(scale.row)
                    .px_4()
                    .gap_3()
                    .rounded(gpui_kit::rems(0.25))
                    .border_1()
                    .border_color(palette.outline)
                    .child(
                        div()
                            .flex_1()
                            .truncate()
                            .text_size(scale.body)
                            .when(!has_value, |text| text.text_color(palette.on_surface_variant))
                            .child(shown.unwrap_or(self.placeholder.clone())),
                    )
                    .child(Icon::new(IconName::ChevronDown).text_color(palette.on_surface_variant))
                    .on_click(move |_, window, cx| {
                        let sheet = cx.new(|_| ChoiceSheet { state: entity.clone(), title: title.clone() });
                        Sheet::open(sheet, window, cx);
                    })
                    .test_support()
                    .into_any_element()
            }
        };
        v_flex().w_full().gap_1().children(label).child(field)
    }
}

/// The options in a sheet.
struct ChoiceSheet {
    state: Entity<ChoiceState>,
    title: Option<SharedString>,
}

impl Render for ChoiceSheet {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let env = Env::of(window, cx);
        let palette = cx.palette();
        let scale = cx.scale();
        let muted = cx.theme().muted_foreground;
        let state = self.state.read(cx);
        let selected = state.selected.clone();
        let rows: Vec<_> = state
            .options
            .iter()
            .map(|option| {
                let on = selected.as_ref() == Some(&option.key);
                let (entity, key) = (self.state.clone(), option.key.clone());
                touch::pressable(ElementId::Name(option.key.clone()), env, cx)
                    .min_h(scale.row)
                    .px_6()
                    .gap_4()
                    .when(on, |row| row.bg(palette.secondary_container).text_color(palette.on_secondary_container))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(div().text_size(scale.body).child(option.label.clone()))
                            .children(option.detail.clone().map(|detail| {
                                div().text_size(scale.small).text_color(muted).child(detail)
                            })),
                    )
                    .when(on, |row| row.child(Icon::new(IconName::Check)))
                    .on_click(move |_, window, cx| {
                        entity.update(cx, |state, cx| state.choose(key.clone(), cx));
                        Sheet::close(window, cx);
                    })
                    .test_support()
            })
            .collect();
        v_flex()
            .flex_grow(1.)
            .min_h_0()
            .children(self.title.clone().map(|title| {
                h_flex().px_6().pb_2().text_size(scale.title).child(title)
            }))
            .child(
                div()
                    .id("choice-options")
                    .flex_grow(1.)
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(v_flex().pb_2().children(rows)),
            )
    }
}
