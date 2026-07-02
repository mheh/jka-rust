#![allow(non_camel_case_types, non_snake_case)]

/// Raven `xcommand_t` — function pointer type for console commands.
///
/// Raven: .
/// Type definition source: `oracle/oracle/code/qcommon/qcommon.h:272-272`
pub type xcommand_t = extern "C" fn();
