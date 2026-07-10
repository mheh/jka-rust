#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_int, c_uint};

/// Raven `netField_t` — one entry in the entity/playerstate delta-coder field
/// table (`name`, byte `offset` into the struct, `bits` = 0 for float).
///
/// Modeled for this build's `!_XBOX && !FINAL_BUILD` config: the XBOX-only
/// `realSize` field is absent; the non-FINAL_BUILD `mCount` profiling counter
/// is present (`MSG_ReportChangeVectors` reads/clears it).
/// Type definition source: `oracle/codemp/qcommon/msg.cpp:838-844`
pub struct netField_t {
    pub name: &'static str,
    pub offset: c_int,
    pub bits: c_int, // 0 = float
    pub mCount: c_uint,
}
