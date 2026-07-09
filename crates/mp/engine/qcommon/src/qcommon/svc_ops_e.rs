#![allow(non_camel_case_types, non_snake_case)]

/// Raven `svc_ops_e` — server to client message opcodes.
///
/// Type definition source: `oracle/codemp/qcommon/qcommon.h:233-250`
#[repr(i32)]
pub enum svc_ops_e {
    svc_bad = 0,
    svc_nop = 1,
    svc_gamestate = 2,
    svc_configstring = 3,    // [short] [string] only in gamestate messages
    svc_baseline = 4,        // only in gamestate messages
    svc_serverCommand = 5,   // [string] to be executed by client game module
    svc_download = 6,        // [short] size [size bytes]
    svc_snapshot = 7,
    svc_setgame = 8,
    svc_mapchange = 9,
    svc_EOF = 10,
}
