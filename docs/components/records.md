# Records: a table with room, a list without

`gpui_adaptive::records` · [index](../README.md)

`Records` shows rows of data with columns. It is money-manager's transactions, generalised.

| Pointer, Medium and up | Compact, or Touch |
|---|---|
| a table: a header row, numbers right-aligned in a medium weight, the selected row in the table's active colour; a sortable column sorts by a click on its header, with an arrow for the direction | a [ListRow](list.md) per record, built from the columns given the roles of title, subtitle and trailing |

| Pointer | Touch |
|---|---|
| ![](../images/desktop-records.png) | ![](../images/phone-records.png) |

## API

```rust
Records::new("transactions")
    .column(Column::new("date", "Date").width(rems(6.)).sortable().role(ColumnRole::Subtitle))
    .column(Column::new("category", "Category").role(ColumnRole::Title))
    .column(Column::new("comment", "Comment"))                       // table only
    .column(Column::new("amount", "Amount").width(rems(7.)).numeric().role(ColumnRole::Trailing))
    .sorted_by("date", false)
    .selected(open.map(|id| ElementId::from(("record", id))))
    .records(rows.iter().map(|r| Record::new(("record", r.id), [r.date(), r.category(), r.comment(), r.amount()])))
    .on_sort(move |column, _, cx| this.update(cx, |this, cx| this.sort_by(column, cx)))
    .on_activate(move |id, window, cx| open_record(id, window, cx))
```

The owner sorts: it is told which column was pressed, keeps the order, and says which column it sorted by. The cells are strings the owner has already formatted, including number grouping and currency. This is meant for pages of a few hundred rows. For thousands, use gpui-kit's virtualising `DataTable`.
