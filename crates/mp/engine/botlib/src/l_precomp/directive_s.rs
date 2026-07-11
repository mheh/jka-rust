#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use crate::l_precomp::source_s::source_t;
use crate::BotLib;

/// Raven `directive_t` — one row of the preprocessor directive dispatch table
/// (`directives[]`/`dollardirectives[]`, l_precomp.cpp:2535/2648). File-local to
/// `l_precomp.cpp`; never crosses the ABI seam.
///
/// Raven's handler is `int (*func)(source_t *source)`; the port threads `BotLib`
/// as the leading receiver (per the crate's `&mut BotLib` convention), so `func`
/// carries the extra parameter. `name` is a nullable C string pointer — the
/// `{NULL, NULL}` sentinel row terminates the table walk; `func` is never read
/// on that row (see the `directives`/`dollardirectives` sentinel stub).
///
/// Type definition source: `oracle/codemp/botlib/l_precomp.cpp:86-90`
#[repr(C)]
pub struct directive_t {
    pub name: *const c_char,
    pub func: fn(&mut BotLib, *mut source_t) -> c_int,
}

pub type directive_s = directive_t;
