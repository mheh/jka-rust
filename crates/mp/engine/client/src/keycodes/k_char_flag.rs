#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::c_int;

/// Raven `K_CHAR_FLAG` — or'd onto a keynum to mark a raw character event, so
/// the menu code takes one path for key and char events.
///
/// Source: `oracle/codemp/client/keycodes.h:342-345`
pub const K_CHAR_FLAG: c_int = 1024;
