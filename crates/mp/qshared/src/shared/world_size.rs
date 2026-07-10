//! MP `q_shared.h` `WORLD_SIZE`.
//!
//! NAV-D3 / RULING 39d migration: moved here from `mp_game`
//! (`crates/mp/game/src/NPC_combat.rs:2736`) so the engine-side nav code
//! (`mp_engine_server`) shares the single referee-compared definition.
//!
//! Raven `WORLD_SIZE` = `MAX_WORLD_COORD - MIN_WORLD_COORD` = `64*1024 - (-64*1024)`.
//!
//! Source: `oracle/codemp/game/q_shared.h:18-20`

/// Raven `WORLD_SIZE`.
///
/// Source: `oracle/codemp/game/q_shared.h:20`
pub const WORLD_SIZE: f32 = 131072.0;
