//! What is drawn over a window's pages and under the toolkit's own dialogs
//! and popups: the stack of touch sheets and the snackbar. One stack per
//! window, drawn by that window's [`crate::AppFrame`].

use std::collections::HashMap;

use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, Entity, EntityId, Global, Window, WindowId};

use crate::menu::FloatingMenu;
use crate::notice::Snackbar;
use crate::sheet::BottomSheet;

#[derive(Default)]
struct Layers {
    sheets: HashMap<WindowId, Vec<Entity<BottomSheet>>>,
    snackbars: HashMap<WindowId, Entity<Snackbar>>,
    menus: HashMap<WindowId, FloatingMenu>,
}

impl Global for Layers {}

pub(crate) fn init(cx: &mut App) {
    if cx.try_global::<Layers>().is_none() {
        cx.set_global(Layers::default());
    }
}

fn id(window: &Window) -> WindowId {
    window.window_handle().window_id()
}

pub(crate) fn push_sheet(window: &Window, sheet: Entity<BottomSheet>, cx: &mut App) {
    init(cx);
    cx.global_mut::<Layers>().sheets.entry(id(window)).or_default().push(sheet);
    cx.refresh_windows();
}

pub(crate) fn remove_sheet(window: WindowId, sheet: EntityId, cx: &mut App) {
    if let Some(sheets) = cx.global_mut::<Layers>().sheets.get_mut(&window) {
        sheets.retain(|open| open.entity_id() != sheet);
    }
    cx.refresh_windows();
}

/// The topmost sheet of `window` that is not on its way out.
pub(crate) fn top_sheet(window: &Window, cx: &App) -> Option<Entity<BottomSheet>> {
    let sheets = cx.try_global::<Layers>()?.sheets.get(&id(window))?;
    sheets.iter().rev().find(|sheet| !sheet.read(cx).is_closing()).cloned()
}

pub(crate) fn set_snackbar(window: &Window, snackbar: Entity<Snackbar>, cx: &mut App) {
    init(cx);
    cx.global_mut::<Layers>().snackbars.insert(id(window), snackbar);
    cx.refresh_windows();
}

pub(crate) fn remove_snackbar(window: WindowId, snackbar: EntityId, cx: &mut App) {
    let snackbars = &mut cx.global_mut::<Layers>().snackbars;
    if snackbars.get(&window).is_some_and(|shown| shown.entity_id() == snackbar) {
        snackbars.remove(&window);
    }
    cx.refresh_windows();
}

pub(crate) fn set_menu(window: &Window, menu: FloatingMenu, cx: &mut App) {
    init(cx);
    cx.global_mut::<Layers>().menus.insert(id(window), menu);
    cx.refresh_windows();
}

pub(crate) fn remove_menu(window: WindowId, cx: &mut App) {
    cx.global_mut::<Layers>().menus.remove(&window);
    cx.refresh_windows();
}

pub(crate) fn has_menu(window: &Window, cx: &App) -> bool {
    cx.try_global::<Layers>().is_some_and(|layers| layers.menus.contains_key(&id(window)))
}

/// The layers of `window`, to draw over everything else it draws.
pub(crate) fn render(window: &Window, cx: &App) -> Vec<AnyElement> {
    let Some(layers) = cx.try_global::<Layers>() else {
        return Vec::new();
    };
    let window_ref = window;
    let window = id(window);
    let mut drawn: Vec<AnyElement> = layers
        .sheets
        .get(&window)
        .into_iter()
        .flatten()
        .map(|sheet| sheet.clone().into_any_element())
        .collect();
    if let Some(snackbar) = layers.snackbars.get(&window) {
        drawn.push(snackbar.clone().into_any_element());
    }
    if let Some(menu) = layers.menus.get(&window) {
        drawn.push(menu.render(window_ref));
    }
    drawn
}
