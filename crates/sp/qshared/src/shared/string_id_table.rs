#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int};

/// Raven `stringID_table_t` (`stringID_table_s`) — a name/id lookup entry used by
/// `GetIDForString`/`GetStringForID`.
///
/// `name` is Raven's `char *`; the struct crosses the ABI seam so it keeps the raw
/// pointer and `#[repr(C)]` layout.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:2617-2621`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct stringID_table_t {
    pub name: *mut c_char,
    pub id: c_int,
}

// Pointer-width dependent: 8-byte `name` + 4-byte `id` + 4-byte tail padding on
// 64-bit targets.
const _: () = {
    assert!(core::mem::size_of::<stringID_table_t>() == 16);
};
