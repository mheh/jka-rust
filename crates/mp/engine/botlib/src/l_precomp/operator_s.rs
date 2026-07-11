#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `operator_t` — an operator node in the preprocessor expression
/// evaluator (`PC_EvaluateTokens`). File-local to `l_precomp.cpp`; never crosses
/// the ABI seam.
///
/// Type definition source: `oracle/codemp/botlib/l_precomp.cpp:1652-1658`
#[repr(C)]
pub struct operator_t {
    pub mOperator: c_int,
    pub priority: c_int,
    pub parentheses: c_int,
    pub prev: *mut operator_t,
    pub next: *mut operator_t,
}

pub type operator_s = operator_t;
