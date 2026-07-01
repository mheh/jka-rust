#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_float, c_int, c_void};

use sp_qshared::shared::{qboolean, vec4_t};

use super::window_def_t::windowDef_t;

/// Raven `Window` — alias for `windowDef_t` used by menu/item headers.
///
/// Type definition source: `oracle/oracle/code/ui/ui_shared.h:146`
pub type Window = windowDef_t;

/// Raven `MAX_MENUITEMS`.
///
/// Source: `oracle/oracle/code/ui/ui_shared.h:271`
pub const MAX_MENUITEMS: usize = 150;

/// Raven `menuDef_t` — a UI menu definition (window plus its items).
///
/// Type definition source: `oracle/oracle/code/ui/ui_shared.h:427-459`
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
    //JLFACCEPT MPMOVED
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
    //TODO: Port itemDef_s
    // Source: oracle/oracle/code/ui/ui_shared.h:374-425
    /// items this menu contains
    pub items: [*mut c_void; MAX_MENUITEMS],
    /// when next item should appear
    pub appearanceTime: c_float,
    /// current item displayed
    pub appearanceCnt: c_int,
    pub appearanceIncrement: c_float,
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
    /// ( optional ) style, normal and shadowed are it for now
    pub descTextStyle: c_int,
}

const _: () = assert!(core::mem::size_of::<menuDef_t>() == 1568);
const _: () = assert!(core::mem::offset_of!(menuDef_t, window) == 0);
const _: () = assert!(core::mem::offset_of!(menuDef_t, font) == 208);
const _: () = assert!(core::mem::offset_of!(menuDef_t, fullScreen) == 216);
const _: () = assert!(core::mem::offset_of!(menuDef_t, itemCount) == 220);
const _: () = assert!(core::mem::offset_of!(menuDef_t, fontIndex) == 224);
const _: () = assert!(core::mem::offset_of!(menuDef_t, cursorItem) == 228);
const _: () = assert!(core::mem::offset_of!(menuDef_t, fadeCycle) == 232);
const _: () = assert!(core::mem::offset_of!(menuDef_t, fadeClamp) == 236);
const _: () = assert!(core::mem::offset_of!(menuDef_t, fadeAmount) == 240);
const _: () = assert!(core::mem::offset_of!(menuDef_t, onOpen) == 248);
const _: () = assert!(core::mem::offset_of!(menuDef_t, onClose) == 256);
const _: () = assert!(core::mem::offset_of!(menuDef_t, onAccept) == 264);
const _: () = assert!(core::mem::offset_of!(menuDef_t, onESC) == 272);
const _: () = assert!(core::mem::offset_of!(menuDef_t, soundName) == 280);
const _: () = assert!(core::mem::offset_of!(menuDef_t, focusColor) == 288);
const _: () = assert!(core::mem::offset_of!(menuDef_t, disableColor) == 304);
const _: () = assert!(core::mem::offset_of!(menuDef_t, items) == 320);
const _: () = assert!(core::mem::offset_of!(menuDef_t, appearanceTime) == 1520);
const _: () = assert!(core::mem::offset_of!(menuDef_t, appearanceCnt) == 1524);
const _: () = assert!(core::mem::offset_of!(menuDef_t, appearanceIncrement) == 1528);
const _: () = assert!(core::mem::offset_of!(menuDef_t, descX) == 1532);
const _: () = assert!(core::mem::offset_of!(menuDef_t, descY) == 1536);
const _: () = assert!(core::mem::offset_of!(menuDef_t, descColor) == 1540);
const _: () = assert!(core::mem::offset_of!(menuDef_t, descAlignment) == 1556);
const _: () = assert!(core::mem::offset_of!(menuDef_t, descScale) == 1560);
const _: () = assert!(core::mem::offset_of!(menuDef_t, descTextStyle) == 1564);
