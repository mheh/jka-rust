#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::c_int;

/// Raven `K_CHAR_FLAG` — or'd onto a keynum to mark a raw character event
/// (distinguished from a plain keynum) when passed to `UI_KEY_EVENT`.
///
/// Source: `oracle/codemp/ui/keycodes.h:344-347`
pub const K_CHAR_FLAG: c_int = 1024;
