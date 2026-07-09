//! `rank_t`.
//!
//! Raven: "sigh... had to move in here for groupInfo".
//! Source: `oracle/codemp/game/ai.h:29-41`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum rank_t {
    RANK_CIVILIAN = 0,
    RANK_CREWMAN = 1,
    RANK_ENSIGN = 2,
    RANK_LT_JG = 3,
    RANK_LT = 4,
    RANK_LT_COMM = 5,
    RANK_COMMANDER = 6,
    RANK_CAPTAIN = 7,
}
pub use rank_t::*;
