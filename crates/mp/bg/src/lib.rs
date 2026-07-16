//! `mp_bg` — MP "both games" shared code (`bg_*`), depended on by game + cgame.
//!
//! //TODO: Port module mp_bg (only the types needed by the game-code migration
//! are ported so far).
//! Source: oracle/codemp/game/bg_public.h:3

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod local;
pub mod public;
pub mod saga;
pub mod siege;
pub mod vehicles;
pub mod weapons;

// The bg function modules (`bg_*.c`) and their state channel, migrated down from
// `mp_game` (safe-state S5-6). `prelude` is the landing prelude these modules
// open with (`use crate::prelude::*;`); `cstr_util` is the seam-string subset and
// `com_parse` the parse-trio twins they consume.
pub mod prelude;

pub mod bg_channel;
pub mod com_parse;
pub mod cstr_util;

pub mod bg_g2_utils;
pub mod bg_lib;
pub mod bg_misc;
pub mod bg_panimate;
pub mod bg_pmove;
pub mod bg_saber;
pub mod bg_saberLoad;
pub mod bg_saga;
pub mod bg_slidemove;
pub mod bg_vehicleLoad;
pub mod bg_vehicleLoad_tables;

pub use public::{
    team_t, MAX_SPAWN_VARS, MAX_SPAWN_VARS_CHARS, TEAM_BLUE, TEAM_FREE, TEAM_NUM_TEAMS, TEAM_RED,
    TEAM_SPECTATOR,
};
