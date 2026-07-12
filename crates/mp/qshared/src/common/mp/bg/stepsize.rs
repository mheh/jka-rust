//! MP `bg_public.h` `STEPSIZE`.
//!
//! NAV-D3 / RULING 39d migration: moved here from `mp_game`
//! (`crates/mp/game/src/bg_slidemove.rs:37`) so the engine-side nav code
//! (`mp_engine_server`) shares the single referee-compared definition;
//! consumed by `npcnav`'s `WP_MINS` z-bound (`-24+STEPSIZE`,
//! `oracle/codemp/server/NPCNav/navigator.cpp:51`).
//!
//! Source: `oracle/codemp/game/bg_public.h:22`

/// Raven `STEPSIZE`.
///
/// Source: `oracle/codemp/game/bg_public.h:22`
pub const STEPSIZE: f32 = 18.0;
