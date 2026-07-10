#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `INFO_CHANGE_MIN_INTERVAL` — 6 seconds is reasonable I suppose.
///
/// Type definition source: `oracle/codemp/server/sv_client.cpp:1502`
pub const INFO_CHANGE_MIN_INTERVAL: c_int = 6000;

/// Raven `INFO_CHANGE_MAX_COUNT` — only allow 3 changes within the 6 seconds.
///
/// Type definition source: `oracle/codemp/server/sv_client.cpp:1503`
pub const INFO_CHANGE_MAX_COUNT: c_int = 3;
