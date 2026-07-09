#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_float, c_int};

use mp_qshared::shared::{qboolean, vec4_t};

use super::item_def_s::itemDef_t;
use super::window_def_t::windowDef_t;

/// Raven `Window` — alias for `windowDef_t` used by menu/item headers.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:146`
pub type Window = windowDef_t;

/// Raven `MAX_MENUITEMS`.
///
/// Source: `oracle/codemp/ui/ui_shared.h:17`
pub const MAX_MENUITEMS: usize = 256;

/// Raven `menuDef_t` — a UI menu definition (window plus its items).
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:307-336`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct menuDef_t {
    pub window: Window,
    /// font
    pub font: *const c_char,
    /// covers entire screen
    pub fullScreen: qboolean,
    /// number of items;
    pub itemCount: c_int,
    pub fontIndex: c_int,
    /// which item as the cursor
    pub cursorItem: c_int,
    pub fadeCycle: c_int,
    pub fadeClamp: c_float,
    pub fadeAmount: c_float,
    /// run when the menu is first opened
    pub onOpen: *const c_char,
    /// run when the menu is closed
    pub onClose: *const c_char,
    //JLFACCEPT
    /// run when menu is closed with acceptance
    pub onAccept: *const c_char,
    /// run when the menu is closed
    pub onESC: *const c_char,
    /// background loop sound for menu
    pub soundName: *const c_char,
    /// focus color for items
    pub focusColor: vec4_t,
    /// focus color for items
    pub disableColor: vec4_t,
    /// items this menu contains
    pub items: [*mut itemDef_t; MAX_MENUITEMS],
    /// X position of description
    pub descX: c_int,
    /// X position of description
    pub descY: c_int,
    /// description text color for items
    pub descColor: vec4_t,
    /// Description of alignment
    pub descAlignment: c_int,
    /// Description scale
    pub descScale: c_float,
    /// when next item should appear
    pub appearanceTime: c_float,
    /// current item displayed
    pub appearanceCnt: c_int,
    pub appearanceIncrement: c_float,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<menuDef_t>() == 2400);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, window) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, font) == 192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, fullScreen) == 200);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, itemCount) == 204);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, fontIndex) == 208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, cursorItem) == 212);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, fadeCycle) == 216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, fadeClamp) == 220);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, fadeAmount) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, onOpen) == 232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, onClose) == 240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, onAccept) == 248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, onESC) == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, soundName) == 264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, focusColor) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, disableColor) == 288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, items) == 304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, descX) == 2352);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, descY) == 2356);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, descColor) == 2360);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, descAlignment) == 2376);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, descScale) == 2380);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, appearanceTime) == 2384);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, appearanceCnt) == 2388);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuDef_t, appearanceIncrement) == 2392);
