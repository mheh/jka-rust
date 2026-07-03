//! `mp_game` — MP server-game module (`g_*`), the game-side of the QVM boundary.
//!
//! The core `g_local.h` data model is ported (client/entity/level + AI/teams/npc
//! types), verified against oracle with size/offset asserts. //TODO: Port the
//! gameplay logic (g_*.c functions).

#![allow(non_camel_case_types, non_snake_case)]

pub mod ai;
pub mod botai;
pub mod client;
pub mod entity;
pub mod level;
pub mod npc;
pub mod saber;
pub mod say;
pub mod teams;
pub mod trap;
pub mod world;

pub use world::{EntityId, GameContext, GameWorld};
