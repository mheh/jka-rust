#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_ulong;

/// Raven `UINT4` — "defines a four byte word" (RSA MD4 reference typedef).
///
/// Type definition source: `oracle/codemp/qcommon/md4.cpp:17`
pub type UINT4 = c_ulong;
