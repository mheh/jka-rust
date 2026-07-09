#![allow(non_camel_case_types, non_snake_case)]

/// Raven `xcommand_t` — function pointer for console commands.
///
/// Type definition source: `oracle/codemp/qcommon/qcommon.h:363-363`
pub type xcommand_t = extern "C" fn();
