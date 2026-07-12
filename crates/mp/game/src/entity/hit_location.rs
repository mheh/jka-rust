//! MP hit-location constants (`HL_*`).
//!
//! Raven anonymous enum ending in `HL_MAX`; sizes `gentity_t::locationDamage`
//! (qshared also exposes `HL_MAX` as a `usize` for that array).
//! Source: `oracle/codemp/game/g_local.h:98-123`

use core::ffi::c_int;

pub const HL_NONE: c_int = 0;
pub const HL_FOOT_RT: c_int = 1;
pub const HL_FOOT_LT: c_int = 2;
pub const HL_LEG_RT: c_int = 3;
pub const HL_LEG_LT: c_int = 4;
pub const HL_WAIST: c_int = 5;
pub const HL_BACK_RT: c_int = 6;
pub const HL_BACK_LT: c_int = 7;
pub const HL_BACK: c_int = 8;
pub const HL_CHEST_RT: c_int = 9;
pub const HL_CHEST_LT: c_int = 10;
pub const HL_CHEST: c_int = 11;
pub const HL_ARM_RT: c_int = 12;
pub const HL_ARM_LT: c_int = 13;
pub const HL_HAND_RT: c_int = 14;
pub const HL_HAND_LT: c_int = 15;
pub const HL_HEAD: c_int = 16;
pub const HL_GENERIC1: c_int = 17;
pub const HL_GENERIC2: c_int = 18;
pub const HL_GENERIC3: c_int = 19;
pub const HL_GENERIC4: c_int = 20;
pub const HL_GENERIC5: c_int = 21;
pub const HL_GENERIC6: c_int = 22;
/// `c_int`-typed dual of the canonical `mp_qshared::common::mp::gentity::HL_MAX`
/// (`usize`), feeding `HL_*` enum-index match arms.
pub const HL_MAX: c_int = mp_qshared::common::mp::gentity::HL_MAX as c_int;
