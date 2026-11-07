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
pub mod chips;
pub mod choice;
pub mod code;
pub mod color_field;
pub mod colors;
pub mod command;
pub mod composer;
pub mod confirm;
pub mod date_field;
pub mod destinations;
pub mod dock;
pub mod empty;
pub mod env;
pub mod feed;
pub mod field;
pub mod form;
pub mod frame;
pub mod identity;
pub mod insets;
pub mod item_actions;
mod layers;
pub mod list;
pub mod lookup;
pub mod menu;
pub mod navigator;
pub mod note;
pub mod notice;
pub mod palette;
pub mod popup;
mod cache;
pub mod prim;
pub mod records;
pub mod scale;
pub mod screen;
pub mod segments;
pub mod setting;
pub mod sheet;
pub mod stack;
pub mod toggle;
mod touch;

pub use ask::Ask;
pub use banner::{Banner, Tone};
pub use busy::{Busy, Skeleton};
pub use chips::Chips;
pub use choice::{Choice, ChoiceEvent, ChoiceOption, ChoiceState};
pub use code::Code;
pub use color_field::{ColorField, ColorFieldEvent, ColorFieldState};
pub use colors::{ActiveColors, Colors};
pub use command::{Command, CommandButton, CommandRole, Handler};
pub use composer::{Composer, ComposerEvent, ComposerState, Suggestion};
pub use confirm::Confirm;
pub use date_field::{DateField, DateFieldEvent, DateFieldState};
pub use destinations::{Destination, Destinations};
pub use dock::{Dock, DockPlace};
pub use empty::Empty;
pub use env::{Env, InputMode, WidthClass, input_mode, set_input_mode};
pub use feed::{Feed, Fold, Message};
pub use field::{FieldEvent, FieldKey, Purpose, TextField, TextFieldState};
pub use form::{Fields, Form, Labelled};
pub use frame::AppFrame;
pub use identity::{Avatar, Count, Presence};
pub use insets::Insets;
pub use item_actions::ItemActions;
pub use list::{Collection, ListRow, SectionHeader, Signal};
pub use lookup::{Lookup, LookupEvent, LookupState};
pub use menu::Menu;
pub use navigator::{Navigator, NavigatorEvent, NavigatorState, Page, Route};
pub use note::Note;
pub use notice::Notice;
pub use palette::{ActivePalette, Palette};
pub use popup::Popup;
pub use prim::{Markdown, Meter, Pill, Surface, Text, Tile, TileSize};
pub use records::{Column, ColumnRole, Record, Records, RecordsScroll};
pub use scale::{ActiveScale, Scale};
pub use screen::Screen;
pub use segments::Segments;
pub use setting::{Setting, Swatches};
pub use sheet::{Sheet, SheetOptions};
pub use stack::Stack;
pub use toggle::Toggle;

use gpui_kit::App;

/// The system's icons: gpui-kit's set, sized with [`Sizable`].
pub use gpui_kit::component::{Icon, IconName, Sizable};

/// An application's own Markdown syntax for [`Markdown::extensions`]: a
/// plugin that finds its custom emoji or its mentions in the text and draws
/// them inline.
pub mod markdown {
    pub use gpui_kit::component::text::{
        InlineElement, InlineRenderContext, MarkdownExtensions, MarkdownNode, MarkdownParseContext,
        MarkdownPlugin, markdown_ast,
    };
}

/// Charts: gpui-kit's bars, lines and rings, painted with the series of
/// `cx.colors().chart` or the application's own colours (a category's).
/// What a chart says should also be written out beside it, for a finger
/// has no pointer to hover with.
pub mod chart {
    pub use gpui_kit::component::chart::{BarChart, LineChart, PieChart};
}

/// The theme an application lets the person choose: light or dark, the
/// catalogue of themes it can wear (Zed's, through `gpui-zed-themes`), the
/// font. What `gpui-adaptive-colors` and the intents paint with follows the
/// theme worn; an application asks for colours with [`ActiveColors`], not
/// here.
pub mod theme {
    pub use gpui_kit::component::{Theme, ThemeConfig, ThemeMode};

    /// The name of the theme worn: what a settings page shows as chosen.
    pub fn name(cx: &gpui_kit::App) -> gpui_kit::SharedString {
        Theme::global(cx).theme_name().clone()
    }
}

/// The keys a text field turns into actions, for a handler around the
/// window's content: Escape that closes what is in front even while a field
/// has the focus (capture it, and let it go on when nothing was closed).
pub mod keys {
    pub use gpui_kit::component::input::Escape;
}

/// The whole Lucide catalog, for an icon the system's set does not name.
pub use gpui_kit::assets::IconName as Lucide;

/// The icons' files: the application's asset source, or what its own
/// source falls back to.
pub use gpui_kit::assets::AllAssets;

/// Sets the application's input mode and the scale that goes with it. Call
/// once, after `gpui_kit::init`, `gpui_adaptive_colors::init` and
/// `gpui_android_glue::init`.
pub fn init(input: InputMode, cx: &mut App) {
    cx.set_global(env::Settings { input });
    scale::Scale::install(input, cx);
    layers::init(cx);
}
