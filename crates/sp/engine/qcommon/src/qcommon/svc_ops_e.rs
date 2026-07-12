#![allow(non_camel_case_types, non_snake_case)]

/// Raven `svc_ops_e` — server-to-client operation codes.
///
/// Raven: .
/// Type definition source: `oracle/code/qcommon/qcommon.h:207-216`
#[repr(i32)]
pub enum svc_ops_e {
    svc_bad,
    svc_nop,
    svc_gamestate,
    svc_configstring,  // [short] [string] only in gamestate messages
    svc_baseline,      // only in gamestate messages
    svc_serverCommand, // [string] to be executed by client game module
    svc_download,      // [short] size [size bytes]
    svc_snapshot,
}
