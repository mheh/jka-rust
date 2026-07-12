#![allow(non_camel_case_types, non_snake_case)]

/// Raven `HMAX` — maximum symbol.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:1055`
pub const HMAX: i32 = 256;

/// Raven `NYT` — Not Yet Transmitted.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:1044`
pub const NYT: i32 = HMAX;

/// Raven `INTERNAL_NODE`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:1045`
pub const INTERNAL_NODE: i32 = HMAX + 1;

/// Raven `SV_ENCODE_START`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:1089`
pub const SV_ENCODE_START: i32 = 4;

/// Raven `SV_DECODE_START`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:1090`
pub const SV_DECODE_START: i32 = 12;

/// Raven `CL_ENCODE_START`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:1091`
pub const CL_ENCODE_START: i32 = 12;

/// Raven `CL_DECODE_START`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:1092`
pub const CL_DECODE_START: i32 = 4;
