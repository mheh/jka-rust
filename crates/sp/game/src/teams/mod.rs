//! SP team/class types (`teams.h`). SP has no separate `npcteam_t`.

pub mod class;
pub mod team;

pub use class::class_t;
// SP `team_t` is a named enum; its variants are `team_t::TEAM_*` (unlike MP's
// `typedef int` + free `TEAM_*` consts).
pub use team::team_t;
