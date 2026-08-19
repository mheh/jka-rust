//! MP `w_saber.h` free-standing `#define` constants and the anonymous force-jump-direction enum.
//!
//! The geometry and event subset shared with the bg tier lives in
//! `mp_qshared::common::mp::qcommon::saber::w_saber_consts` and is re-exported below.
//! It includes `SEF_LOCK_WON`, `SABER_RADIUS_STANDARD`, `SABERMINS_*`/`SABERMAXS_*`, and `SABER_MIN_THROW_DIST`.
//! The game-only constants stay here.
//!
//! Source: `oracle/codemp/game/w_saber.h`

use core::ffi::c_int;

pub use mp_qshared::common::mp::qcommon::saber::w_saber_consts::{
    SABERMAXS_X, SABERMAXS_Y, SABERMAXS_Z, SABERMINS_X, SABERMINS_Y, SABERMINS_Z,
    SABER_MIN_THROW_DIST, SABER_RADIUS_STANDARD, SEF_LOCK_WON,
};

// saberEventFlags: `saberInfo_t`/entity saber-hit-this-frame bits.
pub const SEF_HITENEMY: c_int = 0x1; //Hit the enemy
pub const SEF_HITOBJECT: c_int = 0x2; //Hit some other object
pub const SEF_HITWALL: c_int = 0x4; //Hit a wall
pub const SEF_PARRIED: c_int = 0x8; //Parried a saber swipe
pub const SEF_DEFLECTED: c_int = 0x10; //Deflected a missile or saberInFlight
pub const SEF_BLOCKED: c_int = 0x20; //Was blocked by a parry
/// Raven `SEF_EVENTS`. Source: `oracle/codemp/game/w_saber.h:10`
pub const SEF_EVENTS: c_int =
    SEF_HITENEMY | SEF_HITOBJECT | SEF_HITWALL | SEF_PARRIED | SEF_DEFLECTED | SEF_BLOCKED;
pub const SEF_LOCKED: c_int = 0x40; //Sabers locked with someone else
pub const SEF_INWATER: c_int = 0x80; //Saber is in water

//saberEntityState
pub const SES_LEAVING: c_int = 1;
pub const SES_HOVERING: c_int = 1; //2
pub const SES_RETURNING: c_int = 1; //3
//This is a hack because ATM the saberEntityState is only non-0 if out or 0 if in, and we
//at least want NPCs knowing when their saber is out regardless.

pub const JSF_AMBUSH: c_int = 16; //ambusher Jedi

pub const SABER_REFLECT_MISSILE_CONE: f32 = 0.2;

// `FORCE_POWER_MAX` already lives in `mp_qshared::shared::force_powers`, glob re-exported by the prelude.
// It is not redefined here, to avoid an ambiguous-glob collision.
pub const MAX_GRIP_DISTANCE: c_int = 256;
pub const MAX_TRICK_DISTANCE: c_int = 512;
pub const FORCE_JUMP_CHARGE_TIME: c_int = 6400;
pub const GRIP_DRAIN_AMOUNT: c_int = 30;
pub const FORCE_LIGHTNING_RADIUS: c_int = 300;
pub const MAX_DRAIN_DISTANCE: c_int = 512;

/// Raven force-jump-direction `typedef enum { FJ_FORWARD, ... };` is declared with no trailing type name, an anonymous typedef with no effect in C.
/// These are ordinary file-scope enum constants, ported as plain `c_int` consts per enum-vs-alias fidelity.
///
/// Source: `oracle/codemp/game/w_saber.h:35-41`
pub const FJ_FORWARD: c_int = 0;
pub const FJ_BACKWARD: c_int = 1;
pub const FJ_RIGHT: c_int = 2;
pub const FJ_LEFT: c_int = 3;
pub const FJ_UP: c_int = 4;
