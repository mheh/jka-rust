//! `mp_bg` — MP "both games" shared code (`bg_*`), depended on by game + cgame.
//!
//! //TODO: Port module mp_bg (only the types needed by the game-code migration
//! are ported so far).

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod local;
pub mod public;
pub mod saga;
pub mod siege;
pub mod vehicles;
pub mod weapons;

pub use public::{
    team_t, MAX_SPAWN_VARS, MAX_SPAWN_VARS_CHARS, TEAM_BLUE, TEAM_FREE, TEAM_NUM_TEAMS, TEAM_RED,
    TEAM_SPECTATOR,
};
