//! `mp_bg` — MP "both games" shared code (`bg_*`), depended on by game + cgame.
//!
//! //TODO: Port module mp_bg (only the types needed by the game-code migration
//! are ported so far).

pub mod public;

pub use public::{
    team_t, MAX_SPAWN_VARS, MAX_SPAWN_VARS_CHARS, TEAM_BLUE, TEAM_FREE, TEAM_NUM_TEAMS, TEAM_RED,
    TEAM_SPECTATOR,
};
