//! MP `q_shared.h` per-level limit constants.

#![allow(non_camel_case_types)]

/// Raven `MAX_CLIENTS` — absolute client limit (non-Xbox build).
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1985`
pub const MAX_CLIENTS: usize = 32;

/// Raven `MAX_STRING_CHARS` — max length of a string passed to `Cmd_TokenizeString`.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:380`
pub const MAX_STRING_CHARS: usize = 1024;
