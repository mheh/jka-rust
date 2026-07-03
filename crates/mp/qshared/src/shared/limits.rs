//! MP `q_shared.h` per-level limit constants.

#![allow(non_camel_case_types)]

/// Raven `MAX_CLIENTS` — absolute client limit (non-Xbox build).
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1985`
pub const MAX_CLIENTS: usize = 32;

/// Raven `MAX_GENTITIES` — the entity-array size.
///
/// Relocated to `mp_qshared` per its oracle home + workspace-architecture Tier-0
/// (was mis-placed in `mp_engine_server` by the mechanical type-port; slice-0
/// wiring task). `mp_engine_server` still carries its own copy pending dedupe.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1996,2004`
pub const MAX_GENTITIES: usize = 1024;

/// Raven `MAX_STRING_CHARS` — max length of a string passed to `Cmd_TokenizeString`.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:380`
pub const MAX_STRING_CHARS: usize = 1024;
