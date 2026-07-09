#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use sp_qshared::shared::qboolean;

use super::item_def_s::itemDef_t;

/// Raven `commandDef_t` — a named script command with its handler.
///
/// Type definition source: `oracle/code/ui/ui_shared.h:477-482`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct commandDef_t {
    pub name: *const c_char,
    pub handler: Option<unsafe extern "C" fn(item: *mut itemDef_t, args: *mut *mut c_char) -> qboolean>,
}

const _: () = assert!(core::mem::size_of::<commandDef_t>() == 16);
const _: () = assert!(core::mem::offset_of!(commandDef_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(commandDef_t, handler) == 8);
