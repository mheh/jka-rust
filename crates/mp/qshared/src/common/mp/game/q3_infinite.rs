//! MP `g_public.h` `Q3_INFINITE`.
//!
//! NAV-D3 / RULING 39d migration: moved here from `mp_game`
//! (`crates/mp/game/src/g_public_consts.rs:14`) so the engine-side nav code
//! (`mp_engine_server`, which cannot reach `mp_game`) shares the single
//! referee-compared definition.
//!
//! Source: `oracle/codemp/game/g_public.h:9`

use core::ffi::c_int;

/// Raven `Q3_INFINITE`.
///
/// Source: `oracle/codemp/game/g_public.h:9`
pub const Q3_INFINITE: c_int = 16777216;
