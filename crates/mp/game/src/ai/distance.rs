//! `distance_e`.
//!
//! Source: `oracle/codemp/game/ai.h:4-9`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum distance_e {
    DIST_MELEE = 0,
    DIST_LONG = 1,
}
pub use distance_e::*;
