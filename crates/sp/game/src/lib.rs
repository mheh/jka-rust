//! `sp_game` — SP server-game module (`g_*`).
//!
//! //TODO: Port module sp_game (dependency types ported first; SP places
//! `team_t`/`class_t` in `teams.h` and spawn-var limits in `g_local.h`, so they
//! live here rather than in `sp_bg`).

#![allow(non_camel_case_types, non_snake_case)]

pub mod ai;
pub mod local;
pub mod npc;
pub mod teams;
