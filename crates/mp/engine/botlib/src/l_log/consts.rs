#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `MAX_LOGFILENAMESIZE` — max size of `logfile_s::filename`.
///
/// Source: `oracle/codemp/botlib/l_log.cpp:24`
pub const MAX_LOGFILENAMESIZE: c_int = 1024;
