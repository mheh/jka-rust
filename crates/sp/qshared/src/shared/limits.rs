//! SP `q_shared.h` per-level limit constants.

#![allow(non_camel_case_types)]

/// Raven SP `MAX_CLIENTS` — single-player, so 1 (MP is 32; the old MP value
/// `128` is left commented out in the SP source).
///
/// Source: `oracle/oracle/code/game/q_shared.h:1447`
pub const MAX_CLIENTS: usize = 1;

/// Raven SP `MAX_GENTITIES` — the entity-array size (`1<<GENTITYNUM_BITS`).
///
/// Tier-0 home per its oracle header, mirroring the MP relocation
/// (state-ownership; `sp_engine_server` still carries its own copy pending
/// dedupe).
///
/// Source: `oracle/oracle/code/game/q_shared.h:1450-1451`
pub const MAX_GENTITIES: usize = 1024;

/// Raven SP `MAX_STRING_CHARS` — max length of a string passed to `Cmd_TokenizeString`.
///
/// Source: `oracle/oracle/code/game/q_shared.h:206`
pub const MAX_STRING_CHARS: usize = 1024;
