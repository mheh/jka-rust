#![allow(non_camel_case_types, non_snake_case)]

/// Raven `sharedTraps_t` — shared VM trap opcode indices.
///
/// Type definition source: `oracle/codemp/qcommon/qcommon.h:281-300`
#[repr(i32)]
pub enum sharedTraps_t {
    TRAP_MEMSET = 100,
    TRAP_MEMCPY = 101,
    TRAP_STRNCPY = 102,
    TRAP_SIN = 103,
    TRAP_COS = 104,
    TRAP_ATAN2 = 105,
    TRAP_SQRT = 106,
    TRAP_MATRIXMULTIPLY = 107,
    TRAP_ANGLEVECTORS = 108,
    TRAP_PERPENDICULARVECTOR = 109,
    TRAP_FLOOR = 110,
    TRAP_CEIL = 111,
    TRAP_TESTPRINTINT = 112,
    TRAP_TESTPRINTFLOAT = 113,
    TRAP_ACOS = 114,
    TRAP_ASIN = 115,
}
