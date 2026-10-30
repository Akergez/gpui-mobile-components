# Form: fields with one commitment

`gpui_adaptive::form` · [index](../README.md)

A form is a titled set of fields with Cancel and one commitment, such as "New expense — Add expense".

| Pointer | Touch |
|---|---|
| a gpui-kit dialog (aiddiction's `ui/forms.rs`): title, fields, Cancel and the commitment; Enter commits, Escape cancels; on a tablet the dialog moves to the top while the keyboard is up and its fields scroll | a [bottom sheet](sheet.md) (money-manager's `open_form`): the title with Cancel and the commitment beside it at the top, the fields scrolling under it, the sheet above the keyboard; back closes it |

| Pointer | Touch |
|---|---|
| ![](../images/desktop-form.png) | ![](../images/phone-form.png) |

## API

```rust
let fields = cx.new(|_| ExpenseFields { amount: self.amount.clone(), date: self.date.clone(), category: self.category.clone() });
let amount = self.amount.clone();
Form::new("New expense", fields)
    .commit("Add expense", move |window, cx| {
        let Some(value) = amount.read(cx).number(cx) else {
            amount.update(cx, |f, cx| f.set_error(Some("Type an amount"), cx));
            return false;               // the form stays open
        };
        save(value, cx);
        true                            // the form closes
    })
    .focus(first_field_focus)
    .open(window, cx);
```

The fields are the owner's own view: its [TextField](text-field.md)s, [Choice](choice.md)s and [DateField](date-field.md)s, laid out as it likes. The form therefore keeps its state wherever it is shown. Without `commit`, the form is a panel with Done.

## Rules

- **Name the commitment** ("Add expense", "Create channel"), not "OK" or "Submit".
- **Keep the commitment enabled** and say what is missing, rather than leaving a dead button.
- **Put errors under their fields**, not in a notice.

## Replaces

aiddiction `ui/forms.rs`, matterfast `dialogs/forms/*`, and money-manager `bottom_sheet::open_form`.
