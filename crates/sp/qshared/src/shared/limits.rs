//! SP `q_shared.h` per-level limit constants.

#![allow(non_camel_case_types)]

/// Raven SP `MAX_CLIENTS` — single-player, so 1 (MP is 32; the old MP value
/// `128` is left commented out in the SP source).
///
/// Source: `oracle/oracle/code/game/q_shared.h:1447`
pub const MAX_CLIENTS: usize = 1;

/// Raven SP `MAX_STRING_CHARS` — max length of a string passed to `Cmd_TokenizeString`.
///
/// Source: `oracle/oracle/code/game/q_shared.h:206`
pub const MAX_STRING_CHARS: usize = 1024;
