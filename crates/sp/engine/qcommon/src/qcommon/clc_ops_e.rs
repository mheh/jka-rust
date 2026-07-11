#![allow(non_camel_case_types, non_snake_case)]

/// Raven `clc_ops_e` — client-to-server operation codes.
///
/// Raven: .
/// Type definition source: `oracle/code/qcommon/qcommon.h:222-227`
#[repr(i32)]
pub enum clc_ops_e {
    clc_bad,
    clc_nop,
    clc_move,          // [[usercmd_t]
    clc_clientCommand, // [string] message
}
