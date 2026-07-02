#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::gentity::gentity_t;
use sp_qshared::common::sp::qcommon::usercmd::usercmd_t;
use sp_qshared::common::sp::trace_t::trace_t;
use sp_qshared::shared::{qboolean, vec3_t};

use super::muzzle::Muzzle;
use super::vehicle_info_t::vehicleInfo_t;
use super::veh_turret_status_t::vehTurretStatus_t;
use super::veh_weapon_status_t::vehWeaponStatus_t;

/// `MAX_VEHICLE_EXHAUSTS`.
/// Source: `oracle/oracle/code/game/G_Vehicles.h:83`
const MAX_VEHICLE_EXHAUSTS: usize = 4;
/// `MAX_VEHICLE_MUZZLES`.
/// Source: `oracle/oracle/code/game/G_Vehicles.h:80`
const MAX_VEHICLE_MUZZLES: usize = 10;
/// `MAX_VEHICLE_WEAPONS`.
/// Source: `oracle/oracle/code/game/G_Vehicles.h:86`
const MAX_VEHICLE_WEAPONS: usize = 2;
/// `MAX_VEHICLE_TURRETS`.
/// Source: `oracle/oracle/code/game/G_Vehicles.h:87`
const MAX_VEHICLE_TURRETS: usize = 2;

/// Raven `Vehicle_t` — runtime state for a spawned vehicle instance.
///
/// Type definition source: `oracle/oracle/code/game/G_Vehicles.h:510-621`
#[repr(C)]
pub struct Vehicle_t {
	// The entity who pilots/drives this vehicle.
	// NOTE: This is redundant (since m_pParentEntity->owner _should_ be the pilot). This makes things clearer though.
	pub m_pPilot: *mut gentity_t,

	pub m_iPilotTime: i32, //if spawnflag to die without pilot and this < level.time then die.
	pub m_bHasHadPilot: qboolean, //qtrue once the vehicle gets its first pilot

	//the droid unit NPC for this vehicle, if any
	pub m_pDroidUnit: *mut gentity_t,

	// The entity from which this NPC comes from.
	pub m_pParentEntity: *mut gentity_t,

	// If not zero, how long to wait before we can do anything with the vehicle (we're getting on still).
	// -1 = board from left, -2 = board from right, -3 = jump/quick board.  -4 & -5 = throw off existing pilot
	pub m_iBoarding: i32,

	// Used to check if we've just started the boarding process
	pub m_bWasBoarding: bool,

	// The speed the vehicle maintains while boarding occurs (often zero)
	pub m_vBoardingVelocity: vec3_t,

	// Time modifier (must only be used in ProcessMoveCommands() and ProcessOrientCommands() and is updated in Update()).
	pub m_fTimeModifier: f32,

	// Ghoul2 Animation info.
	// NOTE: Since each vehicle has their own model instance, these bolts must be local to each vehicle as well.
	pub m_iLeftWingBone: i32,
	pub m_iRightWingBone: i32,
	//int m_iDriverTag;
	pub m_iExhaustTag: [i32; MAX_VEHICLE_EXHAUSTS],
	pub m_iMuzzleTag: [i32; MAX_VEHICLE_MUZZLES],
	pub m_iDroidUnitTag: i32,
	pub m_iGunnerViewTag: [i32; MAX_VEHICLE_TURRETS], //Where to put the view origin of the gunner (index)

	// This vehicles weapon muzzles.
	pub m_Muzzles: [Muzzle; MAX_VEHICLE_MUZZLES],

	// The user commands structure.
	pub m_ucmd: usercmd_t,

	// The direction an entity will eject from the vehicle towards.
	pub m_EjectDir: i32,

	// Flags that describe the vehicles behavior.
	// Raven's `unsigned long` is 8 bytes under the LP64 layout this struct was
	// asserted against (matches the 8-byte alignment gap before this field).
	pub m_ulFlags: u64,

	// NOTE: Vehicle Type ID, Orientation, and Armor MUST be transmitted over the net.

	// Current angles of this vehicle.
	pub m_vOrientation: vec3_t,

	// How long you have strafed left or right (increments every frame that you strafe to right, decrements every frame you strafe left)
	pub m_fStrafeTime: i32,

	// Previous angles of this vehicle.
	pub m_vPrevOrientation: vec3_t,

	// When control is lost on a speeder, current angular velocity is stored here and applied until landing
	pub m_vAngularVelocity: f32,

	pub m_vFullAngleVelocity: vec3_t,

	// Current armor and shields of your vehicle (explodes if armor to 0).
	pub m_iArmor: i32,   //hull strength - STAT_HEALTH on NPC
	pub m_iShields: i32, //energy shielding - STAT_ARMOR on NPC

	// Timer for all cgame-FX...? ex: exhaust?
	pub m_iLastFXTime: i32,

	// When to die.
	pub m_iDieTime: i32,

	// This pointer is to a valid VehicleInfo (which could be an animal, speeder, fighter, whatever). This
	// contains the functions actually used to do things to this specific kind of vehicle as well as shared
	// information (max speed, type, etc...).
	pub m_pVehicleInfo: *mut vehicleInfo_t,

	// This trace tells us if we're within landing height.
	pub m_LandTrace: trace_t,

	//bitflag of surfaces that have broken off
	pub m_iRemovedSurfaces: i32,

	// the last time this vehicle fired a turbo burst
	pub m_iTurboTime: i32,

	//how long it should drop like a rock for after freed from SUSPEND
	pub m_iDropTime: i32,

	pub m_iSoundDebounceTimer: i32,

	//last time we incremented the shields
	pub lastShieldInc: i32,

	//so we don't hold it down and toggle it back and forth
	pub linkWeaponToggleHeld: qboolean,

	//info about our weapons (linked, ammo, etc.)
	pub weaponStatus: [vehWeaponStatus_t; MAX_VEHICLE_WEAPONS],
	pub turretStatus: [vehTurretStatus_t; MAX_VEHICLE_TURRETS],

	//the guy who was previously the pilot
	pub m_pOldPilot: *mut gentity_t,

	// don't need these in mp
	pub m_safeJumpMountTime: i32,
	pub m_safeJumpMountRightDot: f32,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<Vehicle_t>() == 1760);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_pPilot) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iPilotTime) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_bHasHadPilot) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_pDroidUnit) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_pParentEntity) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iBoarding) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_bWasBoarding) == 36);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_vBoardingVelocity) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_fTimeModifier) == 52);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iLeftWingBone) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iRightWingBone) == 60);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iExhaustTag) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iMuzzleTag) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iDroidUnitTag) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iGunnerViewTag) == 124);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_Muzzles) == 132);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_ucmd) == 452);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_EjectDir) == 480);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_ulFlags) == 488);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_vOrientation) == 496);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_fStrafeTime) == 508);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_vPrevOrientation) == 512);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_vAngularVelocity) == 524);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_vFullAngleVelocity) == 528);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iArmor) == 540);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iShields) == 544);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iLastFXTime) == 548);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iDieTime) == 552);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_pVehicleInfo) == 560);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_LandTrace) == 568);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iRemovedSurfaces) == 1648);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iTurboTime) == 1652);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iDropTime) == 1656);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_iSoundDebounceTimer) == 1660);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, lastShieldInc) == 1664);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, linkWeaponToggleHeld) == 1668);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, weaponStatus) == 1672);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, turretStatus) == 1704);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_pOldPilot) == 1744);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_safeJumpMountTime) == 1752);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_safeJumpMountRightDot) == 1756);
