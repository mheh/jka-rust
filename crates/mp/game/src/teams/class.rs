//! MP `class_t` — NPC class enumeration.
//!
//! `teams.h` is included by `q_shared.h`, so `class_t` is shared server<->game
//! surface: moved to `mp_qshared::common::mp::game::class_t` (Q3_INFINITE
//! migration precedent) so the engine server can reach it, and re-exported here
//! so `mp_game`'s own call sites (and the `prelude` glob) keep resolving.
//!
//! Type definition source: `oracle/codemp/game/teams.h:17-77`

#![allow(non_camel_case_types)]

pub use mp_qshared::common::mp::game::class_t::class_t;
