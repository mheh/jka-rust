#![allow(non_camel_case_types, non_snake_case)]

/// Raven `MAX_USERMOVE` — max magnitude of a bot elementary-action move command.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:23`
pub const MAX_USERMOVE: ::core::ffi::c_int = 400;

/// Raven `MAX_COMMANDARGUMENTS` — max arguments in a bot console command string.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:24`
pub const MAX_COMMANDARGUMENTS: ::core::ffi::c_int = 10;

/// Raven `ACTION_JUMPEDLASTFRAME` — elementary-action flag bit.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:25`
pub const ACTION_JUMPEDLASTFRAME: ::core::ffi::c_int = 0x0800000;
