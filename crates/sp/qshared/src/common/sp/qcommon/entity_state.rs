//! SP `entityState_t` copied from Raven `code/game/q_shared.h`.
//!
//! Source: `oracle/code/game/q_shared.h:2441-2516`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use crate::shared::{qboolean, trajectory_t, vec3_t};

/// Raven SP `entityState_t`.
///
/// Type definition source: `oracle/code/game/q_shared.h:2448-2516`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct entityState_t {
    /// Entity index.
    pub number: c_int,
    /// entityType_t.
    pub eType: c_int,
    pub eFlags: c_int,
    /// For calculating position.
    pub pos: trajectory_t,
    /// For calculating angles.
    pub apos: trajectory_t,
    pub time: c_int,
    pub time2: c_int,
    pub origin: vec3_t,
    pub origin2: vec3_t,
    pub angles: vec3_t,
    pub angles2: vec3_t,
    /// Shotgun sources, etc.
    pub otherEntityNum: c_int,
    pub otherEntityNum2: c_int,
    /// -1 = in air.
    pub groundEntityNum: c_int,
    /// r + (g<<8) + (b<<16) + (intensity<<24).
    pub constantLight: c_int,
    /// Constantly loop this sound.
    pub loopSound: c_int,
    pub modelindex: c_int,
    pub modelindex2: c_int,
    pub modelindex3: c_int,
    /// 0 to (MAX_CLIENTS - 1), for players and corpses.
    pub clientNum: c_int,
    pub frame: c_int,
    /// For client side prediction, gi.linkentity sets this properly.
    pub solid: c_int,
    /// Impulse events -- muzzle flashes, footsteps, etc.
    pub event: c_int,
    pub eventParm: c_int,
    /// Bit flags.
    pub powerups: c_int,
    /// Determines weapon and flash model, etc.
    pub weapon: c_int,
    pub legsAnim: c_int,
    pub legsAnimTimer: c_int,
    pub torsoAnim: c_int,
    pub torsoAnimTimer: c_int,
    /// Scale players.
    pub scale: c_int,
    pub saberInFlight: qboolean,
    pub saberActive: qboolean,
    pub vehicleAngles: vec3_t,
    pub vehicleArmor: c_int,
    /// 0 if not in a vehicle, otherwise the client number.
    pub m_iVehicleNum: c_int,
    /// Used to scale models in any axis.
    pub modelScale: vec3_t,
    pub radius: c_int,
    pub boltInfo: c_int,
    pub isPortalEnt: qboolean,
}

const _: () = assert!(core::mem::size_of::<entityState_t>() == 272);
const _: () = assert!(core::mem::offset_of!(entityState_t, number) == 0);
const _: () = assert!(core::mem::offset_of!(entityState_t, pos) == 12);
const _: () = assert!(core::mem::offset_of!(entityState_t, origin) == 92);
const _: () = assert!(core::mem::offset_of!(entityState_t, saberInFlight) == 220);
const _: () = assert!(core::mem::offset_of!(entityState_t, modelScale) == 248);
const _: () = assert!(core::mem::offset_of!(entityState_t, isPortalEnt) == 268);
