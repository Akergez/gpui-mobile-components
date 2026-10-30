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
//! strings it has already formatted. A table of thousands of rows wants
//! gpui-kit's `DataTable`, which virtualises; this is for the pages of a few
//! hundred the applications have.

use std::rc::Rc;

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::{ActiveTheme, Icon, IconName, Sizable, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{App, ElementId, FontWeight, Rems, RenderOnce, SharedString, Window, div};

use crate::env::{Env, InputMode, WidthClass};
use crate::list::ListRow;
use crate::scale::ActiveScale;
use crate::touch;

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
    numeric: bool,
    sortable: bool,
    role: ColumnRole,
}

impl Column {
    pub fn new(key: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Column { key: key.into(), title: title.into(), width: None, numeric: false, sortable: false, role: ColumnRole::TableOnly }
    }

    /// A fixed width; without one the column takes a share of what is left.
    pub fn width(mut self, width: Rems) -> Self {
        self.width = Some(width);
        self
    }

    /// Right-aligned, tabular figures.
    pub fn numeric(mut self) -> Self {
        self.numeric = true;
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
}

/// One record: its id and its cells, one per column, formatted.
#[derive(Clone)]
pub struct Record {
    id: ElementId,
    cells: Vec<SharedString>,
}

impl Record {
    pub fn new(id: impl Into<ElementId>, cells: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        Record { id: id.into(), cells: cells.into_iter().map(Into::into).collect() }
    }
}

type OnActivate = Rc<dyn Fn(&ElementId, &mut Window, &mut App)>;
type OnSort = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// A table or a list of records.
#[derive(IntoElement)]
pub struct Records {
    id: ElementId,
    columns: Vec<Column>,
    records: Vec<Record>,
    sorted_by: Option<(SharedString, bool)>,
    selected: Option<ElementId>,
    on_activate: Option<OnActivate>,
    on_sort: Option<OnSort>,
}

impl Records {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Records { id: id.into(), columns: Vec::new(), records: Vec::new(), sorted_by: None, selected: None, on_activate: None, on_sort: None }
    }

    pub fn column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    pub fn records(mut self, records: impl IntoIterator<Item = Record>) -> Self {
        self.records.extend(records);
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

    fn cell(&self, record: &Record, role: ColumnRole) -> Option<SharedString> {
        let at = self.columns.iter().position(|column| column.role == role)?;
        record.cells.get(at).cloned()
    }

    fn render_list(self) -> gpui_kit::AnyElement {
        let rows: Vec<_> = self
            .records
            .iter()
            .map(|record| {
                let title = self.cell(record, ColumnRole::Title).unwrap_or_default();
                let mut row = ListRow::new(record.id.clone(), title).selected(self.selected.as_ref() == Some(&record.id));
                if let Some(subtitle) = self.cell(record, ColumnRole::Subtitle) {
                    row = row.subtitle(subtitle);
                }
                if let Some(trailing) = self.cell(record, ColumnRole::Trailing) {
                    row = row.trailing(div().font_weight(FontWeight::SEMIBOLD).child(trailing));
                }
                if let Some(on_activate) = self.on_activate.clone() {
                    let id = record.id.clone();
                    row = row.on_activate(move |window, cx| on_activate(&id, window, cx));
                }
                row
            })
            .collect();
        v_flex().w_full().children(rows).into_any_element()
    }

    fn render_table(self, env: Env, cx: &App) -> gpui_kit::AnyElement {
        let theme = cx.theme();
        let scale = cx.scale();
        let width = |column: &Column| column.width;
        let header = h_flex()
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
                h_flex()
                    .id((self.id.clone(), column.key.clone()))
                    .gap_1()
                    .when_some(width(column), |cell, width| cell.w(width).flex_none())
                    .when(width(column).is_none(), |cell| cell.flex_1().min_w_0())
                    .when(column.numeric, |cell| cell.justify_end())
                    .when(on_sort.is_some(), |cell| cell.cursor_pointer())
                    .child(column.title.clone())
                    .children(sort.map(|ascending| {
                        Icon::new(if ascending { IconName::ChevronUp } else { IconName::ChevronDown }).xsmall()
                    }))
                    .when_some(on_sort, |cell, on_sort| cell.on_click(move |_, window, cx| on_sort(&key, window, cx)))
                    .test_support()
            }));
        let rows = self.records.iter().map(|record| {
            let selected = self.selected.as_ref() == Some(&record.id);
            let on_activate = self.on_activate.clone();
            let id = record.id.clone();
            touch::pressable(record.id.clone(), env, cx)
                .w_full()
                .h(scale.row)
                .px_2()
                .gap_3()
                .border_b_1()
                .border_color(theme.table_row_border)
                .text_size(scale.body)
                .when(selected, |row| row.bg(theme.table_active))
                .children(self.columns.iter().zip(record.cells.iter()).map(|(column, cell)| {
                    div()
                        .truncate()
                        .when_some(width(column), |cell, width| cell.w(width).flex_none())
                        .when(width(column).is_none(), |cell| cell.flex_1().min_w_0())
                        .when(column.numeric, |cell| cell.text_right().font_weight(FontWeight::MEDIUM))
                        .child(cell.clone())
                }))
                .when_some(on_activate, |row, on_activate| row.on_click(move |_, window, cx| on_activate(&id, window, cx)))
                .test_support()
        });
        v_flex().w_full().child(header).children(rows).into_any_element()
    }
}

impl RenderOnce for Records {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let env = Env::of(window, cx);
        let table = env.input() == InputMode::Pointer && env.width() != WidthClass::Compact;
        if table { self.render_table(env, cx) } else { self.render_list() }
    }
}
