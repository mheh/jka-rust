#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `MAX_EPAIRKEY` — max length of a BSP entity epair key/value string.
///
/// Type definition source: `oracle/codemp/botlib/be_aas_bsp.h:41`
pub const MAX_EPAIRKEY: c_int = 128;
