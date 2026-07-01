//! MP team/class types (`teams.h`).

pub mod class;
pub mod npcteam;

pub use class::class_t;
pub use npcteam::{
    npcteam_t, NPCTEAM_ENEMY, NPCTEAM_FREE, NPCTEAM_NEUTRAL, NPCTEAM_NUM_TEAMS, NPCTEAM_PLAYER,
};
