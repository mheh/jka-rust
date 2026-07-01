#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use sp_qshared::shared::qboolean;

// Raven `#define MAX_MULTI_CVARS 64//32`.
// Source: `oracle/oracle/code/ui/ui_shared.h:81`
const MAX_MULTI_CVARS: usize = 64;

/// Raven `multiDef_s` — multi-value cvar list/string/value tables for combo-box items.
///
/// Type definition source: `oracle/oracle/code/ui/ui_shared.h:83-89`
#[repr(C)]
pub struct multiDef_s {
    pub cvarList: [*const c_char; MAX_MULTI_CVARS],
    pub cvarStr: [*const c_char; MAX_MULTI_CVARS],
    pub cvarValue: [f32; MAX_MULTI_CVARS],
    pub count: c_int,
    pub strDef: qboolean,
}

/// Raven `multiDef_t` — `typedef struct multiDef_s multiDef_t`.
///
/// Type definition source: `oracle/oracle/code/ui/ui_shared.h:83-89`
pub type multiDef_t = multiDef_s;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<multiDef_t>() == 1288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(multiDef_t, cvarList) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(multiDef_t, cvarStr) == 512);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(multiDef_t, cvarValue) == 1024);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(multiDef_t, count) == 1280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(multiDef_t, strDef) == 1284);
