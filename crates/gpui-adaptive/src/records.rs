//! Rows of data with columns: a table when the window has room for one, a
//! list of rows with the key fields when it does not (money-manager's
//! records).
//!
//! **Pointer, Medium and up**: a table with a header row; a column marked
//! sortable sorts by a click on its header (the owner sorts: it is told
//! which column and keeps the order), numbers align right in tabular
//! figures, a row is pressed to open it.
//! **Compact, or Touch**: each record as a [`crate::ListRow`] from the
//! columns given the roles of title, subtitle and trailing; the sort is the
//! owner's chips.
//!
//! The rows are the owner's data, each with a stable id; the cells are
//! strings it has already formatted. A record may carry a [`Tile`], drawn
//! before its title in either form, and [`Pill`]s (its tags). [`Records::fill`] lays out only the rows
//! in view, in a scroll area of its own: a ledger of years of records.

use std::rc::Rc;

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::{ActiveTheme, Icon, IconName, Sizable, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, ElementId, FontWeight, Hsla, Rems, RenderOnce, SharedString, Window, div, uniform_list};

use crate::env::{Env, InputMode, WidthClass};
use crate::list::ListRow;
use crate::prim::{Pill, Tile, TileSize};
use crate::scale::ActiveScale;
use crate::touch;

/// Where a [`Records::fill`] is scrolled to; kept by the owner, so that it
/// can bring a record into view (`scroll_to_item(ix, ScrollStrategy::Center)`).
pub type RecordsScroll = gpui_kit::UniformListScrollHandle;
pub use gpui_kit::ScrollStrategy;

/// What a column is to a compact row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColumnRole {
    /// The row's title.
    Title,
    /// The line under the title.
    Subtitle,
    /// The end of the row: an amount, a time.
    Trailing,
    /// Only in the table.
    #[default]
    TableOnly,
}

/// A column.
#[derive(Clone)]
pub struct Column {
    key: SharedString,
    title: SharedString,
    width: Option<Rems>,
    grow: f32,
    numeric: bool,
    muted: bool,
    sortable: bool,
    role: ColumnRole,
}

impl Column {
    pub fn new(key: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Column {
            key: key.into(),
            title: title.into(),
            width: None,
            grow: 1.,
            numeric: false,
            muted: false,
            sortable: false,
            role: ColumnRole::TableOnly,
        }
    }

    /// A fixed width; without one the column takes a share of what is left.
    pub fn width(mut self, width: Rems) -> Self {
        self.width = Some(width);
        self
    }

    /// How large a share of what is left the column takes, against the
    /// others without a width (1 each by default): the name a row is read
    /// by gets more than a short id.
    pub fn grow(mut self, parts: f32) -> Self {
        self.grow = parts;
        self
    }

    /// Right-aligned, tabular figures.
    pub fn numeric(mut self) -> Self {
        self.numeric = true;
        self
    }

    /// In the secondary text colour: an id, a comment.
    pub fn muted(mut self) -> Self {
        self.muted = true;
        self
    }

    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    pub fn role(mut self, role: ColumnRole) -> Self {
        self.role = role;
        self
    }

    fn sized(&self, cell: gpui_kit::Div) -> gpui_kit::Div {
        match self.width {
            Some(width) => cell.w(width).flex_none(),
            None => cell.flex_grow(self.grow).flex_shrink(1.).flex_basis(gpui_kit::relative(0.)).min_w_0(),
        }
    }
}

/// One record: its id and its cells, one per column, formatted.
#[derive(Clone)]
pub struct Record {
    id: ElementId,
    cells: Vec<SharedString>,
    tile: Option<(Icon, Hsla)>,
    pills: Vec<Pill>,
}

impl Record {
    pub fn new(id: impl Into<ElementId>, cells: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        Record { id: id.into(), cells: cells.into_iter().map(Into::into).collect(), tile: None, pills: Vec::new() }
    }

    /// What the record is of, as a [`Tile`] before its title: a category's
    /// icon in its colour.
    pub fn tile(mut self, icon: impl Into<Icon>, color: Hsla) -> Self {
        self.tile = Some((icon.into(), color));
        self
    }

    /// What the record is tagged with, as [`Pill`]s: after the subtitle in
    /// a list row, after the title in a table's row. Neither grows for them.
    pub fn pills(mut self, pills: impl IntoIterator<Item = Pill>) -> Self {
        self.pills.extend(pills);
        self
    }

    pub fn id(&self) -> &ElementId {
        &self.id
    }
}

type OnActivate = Rc<dyn Fn(&ElementId, &mut Window, &mut App)>;
type OnSort = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// A table or a list of records.
#[derive(IntoElement)]
pub struct Records {
    id: ElementId,
    columns: Vec<Column>,
    records: Rc<Vec<Record>>,
    sorted_by: Option<(SharedString, bool)>,
    selected: Option<ElementId>,
    on_activate: Option<OnActivate>,
    on_sort: Option<OnSort>,
    fill: Option<RecordsScroll>,
}

impl Records {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Records {
            id: id.into(),
            columns: Vec::new(),
            records: Rc::new(Vec::new()),
            sorted_by: None,
            selected: None,
            on_activate: None,
            on_sort: None,
            fill: None,
        }
    }

    pub fn column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    pub fn records(mut self, records: impl IntoIterator<Item = Record>) -> Self {
        Rc::make_mut(&mut self.records).extend(records);
        self
    }

    /// The records as the owner keeps them between frames, without copying
    /// them into every frame.
    pub fn shared(mut self, records: Rc<Vec<Record>>) -> Self {
        self.records = records;
        self
    }

    /// The column the owner sorted by, and whether ascending: the header
    /// shows it.
    pub fn sorted_by(mut self, key: impl Into<SharedString>, ascending: bool) -> Self {
        self.sorted_by = Some((key.into(), ascending));
        self
    }

    pub fn selected(mut self, id: Option<ElementId>) -> Self {
        self.selected = id;
        self
    }

    pub fn on_activate(mut self, f: impl Fn(&ElementId, &mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(f));
        self
    }

    /// A sortable column's header was pressed.
    pub fn on_sort(mut self, f: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_sort = Some(Rc::new(f));
        self
    }

    /// Fills what holds it and scrolls by itself, laying out only the rows
    /// in view: for more records than a page of a few hundred. Inside a
    /// [`crate::Screen`], say `scrolls(false)`. The rows must be of one
    /// height, which they are unless some have a subtitle and some not.
    pub fn fill(mut self, scroll: &RecordsScroll) -> Self {
        self.fill = Some(scroll.clone());
        self
    }
}

/// What every row is drawn from, shared with the rows a virtual list asks
/// for frame by frame.
#[derive(Clone)]
struct Rows {
    columns: Rc<Vec<Column>>,
    records: Rc<Vec<Record>>,
    selected: Option<ElementId>,
    on_activate: Option<OnActivate>,
}

impl Rows {
    fn cell(&self, record: &Record, role: ColumnRole) -> Option<SharedString> {
        let at = self.columns.iter().position(|column| column.role == role)?;
        record.cells.get(at).cloned()
    }

    fn list_row(&self, record: &Record) -> AnyElement {
        let title = self.cell(record, ColumnRole::Title).unwrap_or_default();
        let mut row = ListRow::new(record.id.clone(), title).selected(self.selected.as_ref() == Some(&record.id));
        if let Some((icon, color)) = record.tile.clone() {
            row = row.leading(Tile::new(icon, color).size(TileSize::Leading));
        }
        if let Some(subtitle) = self.cell(record, ColumnRole::Subtitle) {
            row = row.subtitle(subtitle);
        }
        row = row.pills(record.pills.iter().cloned());
        if let Some(trailing) = self.cell(record, ColumnRole::Trailing) {
            row = row.trailing(div().font_weight(FontWeight::SEMIBOLD).child(trailing));
        }
        if let Some(on_activate) = self.on_activate.clone() {
            let id = record.id.clone();
            row = row.on_activate(move |window, cx| on_activate(&id, window, cx));
        }
        row.into_any_element()
    }

    fn table_row(&self, record: &Record, env: Env, cx: &App) -> AnyElement {
        let theme = cx.theme();
        let scale = cx.scale();
        let selected = self.selected.as_ref() == Some(&record.id);
        let on_activate = self.on_activate.clone();
        let id = record.id.clone();
        let muted = theme.muted_foreground;
        let height = if record.tile.is_some() { scale.row_two_line } else { scale.row };
        touch::pressable(record.id.clone(), env, cx)
            .w_full()
            .h(height)
            .px_2()
            .gap_3()
            .border_b_1()
            .border_color(theme.table_row_border)
            .text_size(scale.body)
            .when(selected, |row| row.bg(theme.table_active))
            .children(self.columns.iter().zip(record.cells.iter()).map(|(column, cell)| {
                let tile = (column.role == ColumnRole::Title)
                    .then(|| record.tile.clone())
                    .flatten()
                    .map(|(icon, color)| Tile::new(icon, color).size(TileSize::Small));
                let pills = if column.role == ColumnRole::Title { record.pills.clone() } else { Vec::new() };
                column
                    .sized(h_flex().gap_2())
                    .when(column.numeric, |cell| cell.justify_end().font_weight(FontWeight::MEDIUM))
                    .when(column.muted, |cell| cell.text_color(muted))
                    .when(!pills.is_empty(), |cell| cell.overflow_hidden())
                    .children(tile)
                    .child(div().min_w_0().truncate().child(cell.clone()))
                    .children(pills)
            }))
            .when_some(on_activate, |row, on_activate| row.on_click(move |_, window, cx| on_activate(&id, window, cx)))
            .test_support()
            .into_any_element()
    }
}

impl Records {
    fn rows(&self) -> Rows {
        Rows {
            columns: Rc::new(self.columns.clone()),
            records: self.records.clone(),
            selected: self.selected.clone(),
            on_activate: self.on_activate.clone(),
        }
    }

    fn header(&self, cx: &App) -> AnyElement {
        let theme = cx.theme();
        let scale = cx.scale();
        h_flex()
            .flex_none()
            .w_full()
            .h(scale.row)
            .px_2()
            .gap_3()
            .bg(theme.table_head)
            .text_color(theme.table_head_foreground)
            .text_size(scale.small)
            .font_weight(FontWeight::SEMIBOLD)
            .border_b_1()
            .border_color(theme.table_row_border)
            .children(self.columns.iter().map(|column| {
                let sort = self.sorted_by.as_ref().filter(|(key, _)| key == &column.key).map(|(_, ascending)| *ascending);
                let on_sort = self.on_sort.clone().filter(|_| column.sortable);
                let key = column.key.clone();
                column
                    .sized(h_flex().gap_1())
                    .id((self.id.clone(), column.key.clone()))
                    .when(column.numeric, |cell| cell.justify_end())
                    .when(on_sort.is_some(), |cell| cell.cursor_pointer())
                    .child(column.title.clone())
                    .children(sort.map(|ascending| {
                        Icon::new(if ascending { IconName::ChevronUp } else { IconName::ChevronDown }).xsmall()
                    }))
                    .when_some(on_sort, |cell, on_sort| cell.on_click(move |_, window, cx| on_sort(&key, window, cx)))
                    .test_support()
            }))
            .into_any_element()
    }
}

impl RenderOnce for Records {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let table = env.input() == InputMode::Pointer && env.width() != WidthClass::Compact;
        let rows = self.rows();
        let header = table.then(|| self.header(cx));
        let Some(scroll) = self.fill.clone() else {
            let drawn: Vec<AnyElement> = rows
                .records
                .iter()
                .map(|record| if table { rows.table_row(record, env, cx) } else { rows.list_row(record) })
                .collect();
            return v_flex().w_full().children(header).children(drawn).into_any_element();
        };
        let count = rows.records.len();
        let list = uniform_list((self.id.clone(), "rows"), count, move |range, _, cx| {
            range
                .filter_map(|ix| rows.records.get(ix))
                .map(|record| if table { rows.table_row(record, env, cx) } else { rows.list_row(record) })
                .collect::<Vec<_>>()
        })
        .size_full()
        .track_scroll(&scroll);
        let body = match env.input() {
            InputMode::Pointer => div().relative().flex_1().min_h_0().child(list).vertical_scrollbar(&scroll),
            InputMode::Touch => {
                // A sideways swipe over the list is the page's, not its own:
                // the gpui-mobile rule for a one-axis scroll.
                let mut list = list;
                list.style().restrict_scroll_to_axis = Some(true);
                div().flex_1().min_h_0().child(list)
            }
        };
        v_flex().size_full().children(header).child(body).into_any_element()
    }
}
