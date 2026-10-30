//! Intents for gpui-kit applications that run on a desktop and on a phone.
//!
//! A screen is written once, against what the person is doing: a [`Screen`]
//! with a title and [`Command`]s, a list of [`ListRow`]s, a [`Choice`] among
//! options. Each intent draws itself twice: in the **pointer** skin, which is
//! gpui-kit as its Design Guides draw it, and in the **touch** skin, which is
//! Material 3 with its targets, sheets and gestures. Which one is decided by
//! [`Env`]: the [`InputMode`] the application gives [`init`], and the
//! [`WidthClass`] the window has room for in this frame.
//!
//! Nothing here is a new engine. The colours are `gpui-adaptive-colors`'
//! (a Material You scheme, or a Zed theme the person wears) read through the
//! gpui-kit theme; what a phone needs beyond gpui-mobile (back, the keyboard,
//! the system's bars) is `gpui-android-glue`'s; and where gpui-kit already
//! has the behaviour (a select, a dialog, a notification) the pointer skin is
//! gpui-kit's own component.
//!
//! ```ignore
//! gpui_kit::init(cx);
//! gpui_adaptive_colors::init(Options::new(Color::from_u32(SEED)), cx);
//! gpui_android_glue::init(cx);              // `launch::run` does it on Android
//! gpui_adaptive::init(InputMode::Pointer, cx); // Touch from `android_main`
//! ```
//!
//! The documentation of every intent is in `docs/` beside this crate, one
//! page each, and on each type below.

pub mod ask;
pub mod banner;
pub mod busy;
pub mod choice;
pub mod command;
pub mod composer;
pub mod confirm;
pub mod date_field;
pub mod destinations;
pub mod dock;
pub mod empty;
pub mod env;
pub mod field;
pub mod form;
pub mod frame;
pub mod identity;
pub mod insets;
pub mod item_actions;
mod layers;
pub mod list;
pub mod lookup;
pub mod navigator;
pub mod notice;
pub mod palette;
pub mod records;
pub mod scale;
pub mod screen;
pub mod segments;
pub mod sheet;
pub mod toggle;
mod touch;

pub use ask::Ask;
pub use banner::{Banner, Tone};
pub use busy::{Busy, Skeleton};
pub use choice::{Choice, ChoiceEvent, ChoiceOption, ChoiceState};
pub use command::{Command, CommandRole, Handler};
pub use composer::{Composer, ComposerEvent, ComposerState};
pub use confirm::Confirm;
pub use date_field::{DateField, DateFieldEvent, DateFieldState};
pub use destinations::{Destination, Destinations};
pub use dock::{Dock, DockPlace};
pub use empty::Empty;
pub use env::{Env, InputMode, WidthClass, input_mode, set_input_mode};
pub use field::{FieldEvent, Purpose, TextField, TextFieldState};
pub use form::Form;
pub use frame::AppFrame;
pub use identity::{Avatar, Count, Presence};
pub use insets::Insets;
pub use item_actions::ItemActions;
pub use list::{Collection, ListRow, SectionHeader};
pub use lookup::{Lookup, LookupEvent, LookupState};
pub use navigator::{Navigator, NavigatorEvent, NavigatorState, Page, Route};
pub use notice::Notice;
pub use palette::{ActivePalette, Palette};
pub use records::{Column, ColumnRole, Record, Records};
pub use scale::{ActiveScale, Scale};
pub use screen::Screen;
pub use segments::Segments;
pub use sheet::Sheet;
pub use toggle::Toggle;

use gpui_kit::App;

/// Sets the application's input mode and the scale that goes with it. Call
/// once, after `gpui_kit::init`, `gpui_adaptive_colors::init` and
/// `gpui_android_glue::init`.
pub fn init(input: InputMode, cx: &mut App) {
    cx.set_global(env::Settings { input });
    scale::Scale::install(input, cx);
    layers::init(cx);
}
