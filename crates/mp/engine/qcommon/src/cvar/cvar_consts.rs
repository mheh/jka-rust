#![allow(non_camel_case_types, non_snake_case)]

/// Raven `MAX_CVARS` — capacity of the engine's `cvar_indexes` slot arena
/// (`Cvar_Get` errors past it, matching Raven's static-table overflow).
/// Source: oracle/codemp/qcommon/cvar.cpp:10
pub const MAX_CVARS: usize = 1224;
