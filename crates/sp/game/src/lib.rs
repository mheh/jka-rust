//! `sp_game` — SP server-game module (`g_*`).
//!
//! //TODO: Port module sp_game (dependency types ported first; SP places
//! `team_t`/`class_t` in `teams.h` and spawn-var limits in `g_local.h`, so they
//! live here rather than in `sp_bg`).

#![allow(non_camel_case_types, non_snake_case)]

pub mod ai;
pub mod bset;
pub mod bstate;
pub mod characters;
pub mod dmstates;
pub mod events;
pub mod fields;
pub mod functions;
pub mod game_context;
pub mod game_world;
pub mod gi;
pub mod local;
pub mod npc;
pub mod objectives;
pub mod roff;
pub mod saber;
pub mod say;
pub mod shared;
pub mod teams;
pub mod vehicles;
pub mod weapons;

pub use game_context::GameContext;
pub use game_world::GameWorld;
