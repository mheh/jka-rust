#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;
use sp_qshared::shared::qboolean;

/// Raven `qkey_t` — per-key autorepeat/binding state.
///
/// Type definition source: `oracle/code/client/keys.h:3-7`
#[repr(C)]
pub struct qkey_t {
    pub down: qboolean,
    pub repeats: i32, // if > 1, it is autorepeating
    pub binding: *mut c_char,
}

const _: () = assert!(core::mem::size_of::<qkey_t>() == 16);
const _: () = assert!(core::mem::offset_of!(qkey_t, down) == 0);
const _: () = assert!(core::mem::offset_of!(qkey_t, repeats) == 4);
const _: () = assert!(core::mem::offset_of!(qkey_t, binding) == 8);
