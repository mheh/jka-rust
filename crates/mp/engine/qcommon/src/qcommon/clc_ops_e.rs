#![allow(non_camel_case_types, non_snake_case)]

/// Raven `clc_ops_e` — client to server message opcodes.
///
/// Type definition source: `oracle/oracle/codemp/qcommon/qcommon.h:256-263`
#[repr(i32)]
pub enum clc_ops_e {
    clc_bad = 0,
    clc_nop = 1,
    clc_move = 2,             // [[usercmd_t]
    clc_moveNoDelta = 3,      // [[usercmd_t]
    clc_clientCommand = 4,    // [string] message
    clc_EOF = 5,
}
