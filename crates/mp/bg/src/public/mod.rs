//! MP `bg_public.h` shared game definitions.

pub mod gametype;
pub mod powerup;
pub mod spawn;
pub mod team;

pub use spawn::{MAX_SPAWN_VARS, MAX_SPAWN_VARS_CHARS};
pub use team::{team_t, TEAM_BLUE, TEAM_FREE, TEAM_NUM_TEAMS, TEAM_RED, TEAM_SPECTATOR};
