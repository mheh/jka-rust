#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;
use core::ffi::c_long;

/// Raven `value_t` — a value node in the preprocessor expression evaluator
/// (`PC_EvaluateTokens`). File-local to `l_precomp.cpp`; never crosses the ABI
/// seam, so it keeps an idiomatic-but-faithful field order.
///
/// Type definition source: `oracle/codemp/botlib/l_precomp.cpp:1660-1666`
#[repr(C)]
pub struct value_t {
    /// `signed long int intvalue`
    pub intvalue: c_long,
    /// `double floatvalue`
    pub floatvalue: f64,
    pub parentheses: c_int,
    pub prev: *mut value_t,
    pub next: *mut value_t,
}

pub type value_s = value_t;
