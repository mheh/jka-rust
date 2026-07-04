//! MP `Vehicle_t` copied from Raven `codemp/game/bg_vehicles.h`.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_vehicles.h:477-623`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_int, c_void};

use mp_qshared::common::mp::qcommon::usercmd_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::{qboolean, vec3_t};

use crate::public::bg_entity::bgEntity_t;
use crate::vehicles::vehicle_info_t::vehicleInfo_t;
use crate::vehicles::{vehTurretStatus_t, vehWeaponStatus_t};

/// Raven `VEH_MAX_PASSENGERS` — max passengers a vehicle can carry.
///
/// Source: `oracle/oracle/codemp/game/bg_vehicles.h:489` (array bound).
pub const VEH_MAX_PASSENGERS: usize = 10;

/// Raven `MAX_VEHICLE_EXHAUSTS` — max exhaust tags per vehicle.
///
/// Source: `oracle/oracle/codemp/game/bg_vehicles.h:522` (array bound).
pub const MAX_VEHICLE_EXHAUSTS: usize = 12;

/// Raven `MAX_VEHICLE_MUZZLES` — max weapon muzzles per vehicle.
///
/// Source: `oracle/oracle/codemp/game/bg_vehicles.h:523` (array bound).
pub const MAX_VEHICLE_MUZZLES: usize = 12;

/// Raven `MAX_VEHICLE_TURRETS` — max gunner turrets per vehicle.
///
/// Source: `oracle/oracle/codemp/game/bg_vehicles.h:525` (array bound).
pub const MAX_VEHICLE_TURRETS: usize = 2;

/// Raven `MAX_VEHICLE_WEAPONS` — max weapon slots per vehicle.
///
/// Source: `oracle/oracle/codemp/game/bg_vehicles.h:617` (array bound).
pub const MAX_VEHICLE_WEAPONS: usize = 2;

/// Raven `MAX_VEHICLES` — size of the `g_vehicleInfo` table.
///
/// Source: `oracle/oracle/codemp/game/bg_vehicles.h:365`
pub const MAX_VEHICLES: usize = 16;

/// Raven `VEHICLE_BASE`/`VEHICLE_NONE` — `g_vehicleInfo[]` index sentinels.
///
/// Source: `oracle/oracle/codemp/game/bg_vehicles.h:366-367`
pub const VEHICLE_BASE: c_int = 0;
pub const VEHICLE_NONE: c_int = -1;

/// Raven `Vehicle_t` — per-vehicle runtime state shared by SP-derived vehicle
/// code running in MP.
///
/// Raven: (unnamed struct comment) — "this stuff is a little bit different
/// from SP, because I am lazy -rww".
/// Type definition source: `oracle/oracle/codemp/game/bg_vehicles.h:477-623`
#[repr(C)]
pub struct Vehicle_t {
    /// The entity who pilots/drives this vehicle.
    ///
    /// Raven: NOTE: This is redundant (since m_pParentEntity->owner _should_
    /// be the pilot). This makes things clearer though.
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:481`
    pub m_pPilot: *mut bgEntity_t,

    /// Raven: if spawnflag to die without pilot and this < level.time then die.
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:483`
    pub m_iPilotTime: c_int,
    /// Raven: index to last pilot
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:484`
    pub m_iPilotLastIndex: c_int,
    /// Raven: qtrue once the vehicle gets its first pilot
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:485`
    pub m_bHasHadPilot: qboolean,

    /// The passengers of this vehicle.
    ///
    /// Raven: `//bgEntity_t **m_ppPassengers;`
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:489`
    pub m_ppPassengers: [*mut bgEntity_t; VEH_MAX_PASSENGERS],

    /// Raven: the droid unit NPC for this vehicle, if any
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:492`
    pub m_pDroidUnit: *mut bgEntity_t,

    /// The number of passengers currently in this vehicle.
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:495`
    pub m_iNumPassengers: c_int,

    /// The entity from which this NPC comes from.
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:498`
    pub m_pParentEntity: *mut bgEntity_t,

    /// If not zero, how long to wait before we can do anything with the
    /// vehicle (we're getting on still).
    ///
    /// Raven: -1 = board from left, -2 = board from right, -3 = jump/quick
    /// board.  -4 & -5 = throw off existing pilot
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:502`
    pub m_iBoarding: c_int,

    /// Used to check if we've just started the boarding process
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:505`
    pub m_bWasBoarding: qboolean,

    /// The speed the vehicle maintains while boarding occurs (often zero)
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:508`
    pub m_vBoardingVelocity: vec3_t,

    /// Time modifier (must only be used in ProcessMoveCommands() and
    /// ProcessOrientCommands() and is updated in Update()).
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:511`
    pub m_fTimeModifier: f32,

    /// Ghoul2 Animation info.
    ///
    /// Raven: `//int m_iDriverTag;`
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:515`
    pub m_iLeftExhaustTag: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:516`
    pub m_iRightExhaustTag: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:517`
    pub m_iGun1Tag: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:518`
    pub m_iGun1Bone: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:519`
    pub m_iLeftWingBone: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:520`
    pub m_iRightWingBone: c_int,

    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:522`
    pub m_iExhaustTag: [c_int; MAX_VEHICLE_EXHAUSTS],
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:523`
    pub m_iMuzzleTag: [c_int; MAX_VEHICLE_MUZZLES],
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:524`
    pub m_iDroidUnitTag: c_int,
    /// Raven: Where to put the view origin of the gunner (index)
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:525`
    pub m_iGunnerViewTag: [c_int; MAX_VEHICLE_TURRETS],

    /// Raven: this stuff is a little bit different from SP, because I am lazy
    /// -rww
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:528`
    pub m_iMuzzleTime: [c_int; MAX_VEHICLE_MUZZLES],
    /// These are updated every frame and represent the current position for
    /// the specific muzzle.
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:530`
    pub m_vMuzzlePos: [vec3_t; MAX_VEHICLE_MUZZLES],
    /// These are updated every frame and represent the current direction for
    /// the specific muzzle.
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:530`
    pub m_vMuzzleDir: [vec3_t; MAX_VEHICLE_MUZZLES],

    /// This is how long to wait before being able to fire a specific muzzle
    /// again. This is based on the firing rate so that a firing rate of
    /// 10 rounds/sec would make this value initially 100 miliseconds.
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:534`
    pub m_iMuzzleWait: [c_int; MAX_VEHICLE_MUZZLES],

    /// The user commands structure.
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:537`
    pub m_ucmd: usercmd_t,

    /// The direction an entity will eject from the vehicle towards.
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:540`
    pub m_EjectDir: c_int,

    /// Flags that describe the vehicles behavior.
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:543`
    pub m_ulFlags: u64,

    /// NOTE: Vehicle Type ID, Orientation, and Armor MUST be transmitted over
    /// the net.
    ///
    /// The ID of the type of vehicle this is.
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:548`
    pub m_iVehicleTypeID: c_int,

    /// Current angles of this vehicle.
    ///
    /// Raven: `//vec3_t m_vOrientation;` — since we use the SP code for
    /// vehicles, I want to use this value, but I'm going to make it a
    /// pointer to a vec3_t in the playerstate for prediction's sake. -rww
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:552`
    pub m_vOrientation: *mut f32,

    /// How long you have strafed left or right (increments every frame that
    /// you strafe to right, decrements every frame you strafe left)
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:557`
    pub m_fStrafeTime: c_int,

    /// Previous angles of this vehicle.
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:560`
    pub m_vPrevOrientation: vec3_t,

    /// Previous viewangles of the rider
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:563`
    pub m_vPrevRiderViewAngles: vec3_t,

    /// When control is lost on a speeder, current angular velocity is stored
    /// here and applied until landing
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:566`
    pub m_vAngularVelocity: f32,

    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:568`
    pub m_vFullAngleVelocity: vec3_t,

    /// Current armor and shields of your vehicle (explodes if armor to 0).
    ///
    /// Raven: hull strength - STAT_HEALTH on NPC
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:571`
    pub m_iArmor: c_int,
    /// Raven: energy shielding - STAT_ARMOR on NPC
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:572`
    pub m_iShields: c_int,

    /// Raven: mp-specific
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:575`
    pub m_iHitDebounce: c_int,

    /// Timer for all cgame-FX...? ex: exhaust?
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:578`
    pub m_iLastFXTime: c_int,

    /// When to die.
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:581`
    pub m_iDieTime: c_int,

    /// Raven `vehicleInfo_t *m_pVehicleInfo`.
    ///
    /// Raven: This pointer is to a valid VehicleInfo (which could be an
    /// animal, speeder, fighter, whatever). This contains the functions
    /// actually used to do things to this specific kind of vehicle as well
    /// as shared information (max speed, type, etc...).
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:586`
    pub m_pVehicleInfo: *mut vehicleInfo_t,

    /// This trace tells us if we're within landing height.
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:589`
    pub m_LandTrace: trace_t,

    /// TEMP: The wing angles (used to animate it).
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:592`
    pub m_vWingAngles: vec3_t,

    /// Raven: amount of damage done last impact
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:595`
    pub m_iLastImpactDmg: c_int,

    /// Raven: bitflag of surfaces that have broken off
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:598`
    pub m_iRemovedSurfaces: c_int,

    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:600`
    pub m_iDmgEffectTime: c_int,

    /// Raven: the last time this vehicle fired a turbo burst
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:603`
    pub m_iTurboTime: c_int,

    /// Raven: how long it should drop like a rock for after freed from
    /// SUSPEND
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:606`
    pub m_iDropTime: c_int,

    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:608`
    pub m_iSoundDebounceTimer: c_int,

    /// Raven: last time we incremented the shields
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:611`
    pub lastShieldInc: c_int,

    /// Raven: so we don't hold it down and toggle it back and forth
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:614`
    pub linkWeaponToggleHeld: qboolean,

    /// Raven: info about our weapons (linked, ammo, etc.)
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:617`
    pub weaponStatus: [vehWeaponStatus_t; MAX_VEHICLE_WEAPONS],
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:618`
    pub turretStatus: [vehTurretStatus_t; MAX_VEHICLE_TURRETS],

    /// Raven: the guy who was previously the pilot
    /// Raven field source: `oracle/oracle/codemp/game/bg_vehicles.h:621`
    pub m_pOldPilot: *mut bgEntity_t,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<Vehicle_t>() == 976);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_pPilot) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iPilotTime) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iPilotLastIndex) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_bHasHadPilot) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_ppPassengers) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_pDroidUnit) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iNumPassengers) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_pParentEntity) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iBoarding) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_bWasBoarding) == 132);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_vBoardingVelocity) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_fTimeModifier) == 148);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iLeftExhaustTag) == 152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iRightExhaustTag) == 156);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iGun1Tag) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iGun1Bone) == 164);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iLeftWingBone) == 168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iRightWingBone) == 172);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iExhaustTag) == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iMuzzleTag) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iDroidUnitTag) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iGunnerViewTag) == 276);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iMuzzleTime) == 284);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_vMuzzlePos) == 332);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_vMuzzleDir) == 476);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iMuzzleWait) == 620);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_ucmd) == 668);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_EjectDir) == 696);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_ulFlags) == 704);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iVehicleTypeID) == 712);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_vOrientation) == 720);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_fStrafeTime) == 728);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_vPrevOrientation) == 732);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_vPrevRiderViewAngles) == 744);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_vAngularVelocity) == 756);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_vFullAngleVelocity) == 760);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iArmor) == 772);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iShields) == 776);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iHitDebounce) == 780);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iLastFXTime) == 784);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iDieTime) == 788);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_pVehicleInfo) == 792);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_LandTrace) == 800);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_vWingAngles) == 848);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iLastImpactDmg) == 860);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iRemovedSurfaces) == 864);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iDmgEffectTime) == 868);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iTurboTime) == 872);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iDropTime) == 876);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iSoundDebounceTimer) == 880);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, lastShieldInc) == 884);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, linkWeaponToggleHeld) == 888);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, weaponStatus) == 892);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, turretStatus) == 924);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_pOldPilot) == 968);
