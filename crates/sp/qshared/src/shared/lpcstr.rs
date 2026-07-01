#![allow(non_camel_case_types)]

use core::ffi::c_char;

/// Raven `LPCSTR` — Win32 const-string alias (`const char *`). SP-only.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:178`
pub type LPCSTR = *const c_char;
