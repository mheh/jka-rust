//! MP `w_saber.h` free-standing `#define` constants and the anonymous
//! force-jump-direction enum.
//!
//! Source: `oracle/codemp/game/w_saber.h`

use core::ffi::c_int;

// saberEventFlags — `saberInfo_t`/entity saber-hit-this-frame bits.
pub const SEF_HITENEMY: c_int = 0x1; // Hit the enemy
pub const SEF_HITOBJECT: c_int = 0x2; // Hit some other object
pub const SEF_HITWALL: c_int = 0x4; // Hit a wall
pub const SEF_PARRIED: c_int = 0x8; // Parried a saber swipe
pub const SEF_DEFLECTED: c_int = 0x10; // Deflected a missile or saberInFlight
pub const SEF_BLOCKED: c_int = 0x20; // Was blocked by a parry
/// Raven `SEF_EVENTS`. Source: `oracle/codemp/game/w_saber.h:10`
pub const SEF_EVENTS: c_int =
    SEF_HITENEMY | SEF_HITOBJECT | SEF_HITWALL | SEF_PARRIED | SEF_DEFLECTED | SEF_BLOCKED;
pub const SEF_LOCKED: c_int = 0x40; // Sabers locked with someone else
pub const SEF_INWATER: c_int = 0x80; // Saber is in water
pub const SEF_LOCK_WON: c_int = 0x100; // Won a saberLock

// saberEntityState — Raven note: hacky, only ever non-0/0 in practice.
pub const SES_LEAVING: c_int = 1;
pub const SES_HOVERING: c_int = 1; // 2 in comment, redefined to 1
pub const SES_RETURNING: c_int = 1; // 3 in comment, redefined to 1

/// Raven `JSF_AMBUSH` — ambusher Jedi.
pub const JSF_AMBUSH: c_int = 16;

pub const SABER_RADIUS_STANDARD: f32 = 3.0;
pub const SABER_REFLECT_MISSILE_CONE: f32 = 0.2;

// `FORCE_POWER_MAX` already lives in `mp_qshared::shared::force_powers` (glob
// re-exported by the prelude); not redefined here to avoid an ambiguous-glob
// collision.
pub const MAX_GRIP_DISTANCE: c_int = 256;
pub const MAX_TRICK_DISTANCE: c_int = 512;
pub const FORCE_JUMP_CHARGE_TIME: c_int = 6400;
pub const GRIP_DRAIN_AMOUNT: c_int = 30;
pub const FORCE_LIGHTNING_RADIUS: c_int = 300;
pub const MAX_DRAIN_DISTANCE: c_int = 512;

/// Raven's force-jump-direction `typedef enum { FJ_FORWARD, ... };` is
/// declared with no trailing type name (an anonymous typedef with no effect
/// in C), so these are ordinary file-scope enum constants; ported as plain
/// `c_int` consts per enum-vs-alias fidelity.
///
/// Source: `oracle/codemp/game/w_saber.h:35-41`
pub const FJ_FORWARD: c_int = 0;
pub const FJ_BACKWARD: c_int = 1;
pub const FJ_RIGHT: c_int = 2;
pub const FJ_LEFT: c_int = 3;
pub const FJ_UP: c_int = 4;

pub const SABERMINS_X: f32 = -3.0;
pub const SABERMINS_Y: f32 = -3.0;
pub const SABERMINS_Z: f32 = -3.0;
pub const SABERMAXS_X: f32 = 3.0;
pub const SABERMAXS_Y: f32 = 3.0;
pub const SABERMAXS_Z: f32 = 3.0;
pub const SABER_MIN_THROW_DIST: f32 = 80.0;
