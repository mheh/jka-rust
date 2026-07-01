#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_qshared::shared::qboolean;

use super::column_info_s::columnInfo_t;

// Raven `#define MAX_LB_COLUMNS 16`.
// Source: `oracle/oracle/codemp/ui/ui_shared.h:164`
const MAX_LB_COLUMNS: usize = 16;

/// Raven `listBoxDef_s` — list box layout/state definition.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_shared.h:172-186`
#[repr(C)]
pub struct listBoxDef_s {
    pub startPos: c_int,
    pub endPos: c_int,
    pub drawPadding: c_int,
    pub cursorPos: c_int,
    pub elementWidth: f32,
    pub elementHeight: f32,
    pub elementStyle: c_int,
    pub numColumns: c_int,
    pub columnInfo: [columnInfo_t; MAX_LB_COLUMNS],
    pub doubleClick: *const c_char,
    pub notselectable: qboolean,
    // JLF MPMOVED
    pub scrollhidden: qboolean,
}

/// Raven `listBoxDef_t` — `typedef struct listBoxDef_s listBoxDef_t`.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_shared.h:172-186`
pub type listBoxDef_t = listBoxDef_s;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<listBoxDef_t>() == 240);
const _: () = assert!(core::mem::offset_of!(listBoxDef_t, startPos) == 0);
const _: () = assert!(core::mem::offset_of!(listBoxDef_t, endPos) == 4);
const _: () = assert!(core::mem::offset_of!(listBoxDef_t, drawPadding) == 8);
const _: () = assert!(core::mem::offset_of!(listBoxDef_t, cursorPos) == 12);
const _: () = assert!(core::mem::offset_of!(listBoxDef_t, elementWidth) == 16);
const _: () = assert!(core::mem::offset_of!(listBoxDef_t, elementHeight) == 20);
const _: () = assert!(core::mem::offset_of!(listBoxDef_t, elementStyle) == 24);
const _: () = assert!(core::mem::offset_of!(listBoxDef_t, numColumns) == 28);
const _: () = assert!(core::mem::offset_of!(listBoxDef_t, columnInfo) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(listBoxDef_t, doubleClick) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(listBoxDef_t, notselectable) == 232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(listBoxDef_t, scrollhidden) == 236);
