//! `mp_game` — MP server-game module (`g_*`), the game-side of the QVM boundary.
//!
//! //TODO: Port module mp_game (dependency types for the dormant `gclient_s` /
//! `level_locals_t` scaffolding are ported first; the structs themselves follow).

#![allow(non_camel_case_types, non_snake_case)]

pub mod ai;
pub mod client;
pub mod npc;
pub mod teams;
