//! MP `ai.h` squad-state values (`NPCInfo->squadState`).
//!
//! Raven declares these via a fully anonymous `enum { SQUAD_IDLE, ... }` (no
//! `typedef` name at all), so per enum-vs-alias fidelity this becomes a plain
//! `c_int` alias + consts rather than a named Rust enum.
//!
//! Source: `oracle/codemp/game/ai.h:18-26`

use core::ffi::c_int;

pub type squadState_t = c_int;

pub const SQUAD_IDLE: squadState_t = 0; //No target found, waiting
pub const SQUAD_STAND_AND_SHOOT: squadState_t = 1; //Standing in position and shoot (no cover)
pub const SQUAD_RETREAT: squadState_t = 2; //Running away from combat
pub const SQUAD_COVER: squadState_t = 3; //Under protective cover
pub const SQUAD_TRANSITION: squadState_t = 4; //Moving between points, not firing
pub const SQUAD_POINT: squadState_t = 5; //On point, laying down suppressive fire
pub const SQUAD_SCOUT: squadState_t = 6; //Poking out to draw enemy
pub const NUM_SQUAD_STATES: squadState_t = 7;
