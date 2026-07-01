#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_void};

use sp_qshared::shared::qboolean;

/// Raven `commandDef_t` — a named script command with its handler.
///
/// Type definition source: `oracle/oracle/code/ui/ui_shared.h:477-482`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct commandDef_t {
    pub name: *const c_char,
    //TODO: Port itemDef_t
    // Source: oracle/oracle/code/ui/ui_shared.h:374-425
    pub handler: Option<unsafe extern "C" fn(item: *mut c_void, args: *mut *mut c_char) -> qboolean>,
}

const _: () = assert!(core::mem::size_of::<commandDef_t>() == 16);
const _: () = assert!(core::mem::offset_of!(commandDef_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(commandDef_t, handler) == 8);
