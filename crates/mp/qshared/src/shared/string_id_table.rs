#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int};

/// Raven `stringID_table_t` (`stringID_table_s`) — a name/id lookup entry used
/// by `GetIDForString`/`GetStringForID`.
///
/// `name` is Raven's `char *` (an owned/static C string pointer); the struct
/// crosses the ABI seam so it keeps the raw pointer and `#[repr(C)]` layout.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:3154-3158`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct stringID_table_t {
    pub name: *mut c_char,
    pub id: c_int,
}

// `name` is always initialized from a `'static` C-string literal (Raven's
// `stringID_table_t` tables — `setTable`, `TeamTable`, `ClassTable`, … — are
// `static` arrays of string literals cast to `char *`) and never mutated, so
// sharing such a table across threads is sound despite the raw pointer. Same
// treatment as `BG_field_t`.
unsafe impl Sync for stringID_table_t {}

// Pointer-width dependent: 8-byte `name` + 4-byte `id` + 4-byte tail padding on
// 64-bit targets.
const _: () = {
    assert!(core::mem::size_of::<stringID_table_t>() == 16);
};
