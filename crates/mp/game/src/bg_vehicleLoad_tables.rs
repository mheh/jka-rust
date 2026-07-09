//! Vehicle/vehicle-weapon `.veh`/`.vwp` field-descriptor tables and file-scope
//! constants (`vehWeaponFields`, `vehicleFields`, `VehicleTable`,
//! `NUM_VWEAP_PARMS`, `MAX_VEH_WEAPON_DATA_SIZE`, `MAX_VEHICLE_DATA_SIZE`).
//!
//! Raven declared these as bare file-scope statics in `bg_vehicleLoad.c`
//! (porting-rules: `#define` -> `const`, pool tables -> owned/const arrays);
//! kept in their own file (rather than folded into `bg_vehicleLoad.rs`) since
//! they are data, not ported function bodies. `VFOFS`/`VWFOFS` byte offsets
//! are computed with `core::mem::offset_of!` against the already-ported
//! `vehicleInfo_t`/`vehWeaponInfo_t` layouts rather than hand-derived, so they
//! stay correct if those layouts ever change.
//!
//! `_JK2MP` is always true in this crate (jampgame), so the `#ifdef _JK2MP`
//! branch of `vehicleFields` (radarIcon/dmgIndic*/icon_*/health_* fields) is
//! the live one; the SP `#else` branch (bare `radarIcon`/`armorLowFX`/
//! `armorGoneFX`) is dropped per porting-rules §20.
//!
//! Source: `oracle/codemp/game/bg_vehicleLoad.c:66-671`
#![allow(non_upper_case_globals, non_snake_case)]

use mp_bg::vehicles::{
    turretStats_t, vehFieldType_t, vehField_t, vehWeaponInfo_t, vehWeaponStats_t, vehicleInfo_t,
};
use mp_qshared::shared::string_id_table::stringID_table_t;

/// Raven `MAX_VEH_WEAPON_DATA_SIZE` — `VehWeaponParms` scratch-buffer size.
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:66`
pub const MAX_VEH_WEAPON_DATA_SIZE: usize = 0x20000;
/// Raven `MAX_VEHICLE_DATA_SIZE` — `VehicleParms` scratch-buffer size.
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:67`
pub const MAX_VEHICLE_DATA_SIZE: usize = 0x80000;

/// Raven `NUM_VWEAP_PARMS` — entry count of `vehWeaponFields` (must match the
/// number of parseable `vehWeaponStats_t` fields, per the oracle's own
/// "*** IMPORTANT!!! ***" comment).
/// Source: `oracle/codemp/game/bg_vehicles.h:66`
pub const NUM_VWEAP_PARMS: usize = 25;

// Byte-offset helpers for the array-of-struct fields (`weapon[n].x`,
// `weapMuzzle[n]`, `turret[n].x`) that Raven's `VFOFS`/`VWFOFS` macros reach
// with plain C pointer arithmetic.
const OFS_WEAPON: usize = core::mem::offset_of!(vehicleInfo_t, weapon);
const SZ_VWS: usize = core::mem::size_of::<vehWeaponStats_t>();
const OFS_MUZZLE: usize = core::mem::offset_of!(vehicleInfo_t, weapMuzzle);
const OFS_TURRET: usize = core::mem::offset_of!(vehicleInfo_t, turret);
const SZ_TURRET: usize = core::mem::size_of::<turretStats_t>();

/// Raven `vehField_t vehWeaponFields[NUM_VWEAP_PARMS]`.
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:138-164`
pub const vehWeaponFields: [vehField_t; NUM_VWEAP_PARMS] = [
    vehField_t {
        name: c"name".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, name) as i32,
        r#type: vehFieldType_t::VF_LSTRING,
    },
    vehField_t {
        name: c"projectile".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, bIsProjectile) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"hasGravity".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, bHasGravity) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"ionWeapon".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, bIonWeapon) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"saberBlockable".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, bSaberBlockable) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"muzzleFX".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, iMuzzleFX) as i32,
        r#type: vehFieldType_t::VF_EFFECT_CLIENT,
    },
    vehField_t {
        name: c"model".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, iModel) as i32,
        r#type: vehFieldType_t::VF_MODEL_CLIENT,
    },
    vehField_t {
        name: c"shotFX".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, iShotFX) as i32,
        r#type: vehFieldType_t::VF_EFFECT_CLIENT,
    },
    vehField_t {
        name: c"impactFX".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, iImpactFX) as i32,
        r#type: vehFieldType_t::VF_EFFECT_CLIENT,
    },
    vehField_t {
        name: c"g2MarkShader".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, iG2MarkShaderHandle) as i32,
        r#type: vehFieldType_t::VF_SHADER,
    },
    vehField_t {
        name: c"g2MarkSize".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, fG2MarkSize) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"loopSound".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, iLoopSound) as i32,
        r#type: vehFieldType_t::VF_SOUND_CLIENT,
    },
    vehField_t {
        name: c"speed".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, fSpeed) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"homing".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, fHoming) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"homingFOV".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, fHomingFOV) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"lockOnTime".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, iLockOnTime) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"damage".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, iDamage) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"splashDamage".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, iSplashDamage) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"splashRadius".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, fSplashRadius) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"ammoPerShot".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, iAmmoPerShot) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"health".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, iHealth) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"width".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, fWidth) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"height".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, fHeight) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"lifetime".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, iLifeTime) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"explodeOnExpire".as_ptr(),
        ofs: core::mem::offset_of!(vehWeaponInfo_t, bExplodeOnExpire) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
];

/// Raven `vehField_t vehicleFields[]` — sentinel-terminated (`.ofs == -1`).
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:445-668`
pub const vehicleFields: [vehField_t; 175] = [
    vehField_t {
        name: c"name".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, name) as i32,
        r#type: vehFieldType_t::VF_LSTRING,
    },
    vehField_t {
        name: c"type".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, r#type) as i32,
        r#type: vehFieldType_t::VF_VEHTYPE,
    },
    vehField_t {
        name: c"numHands".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, numHands) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"lookPitch".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, lookPitch) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"lookYaw".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, lookYaw) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"length".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, length) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"width".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, width) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"height".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, height) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"centerOfGravity".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, centerOfGravity) as i32,
        r#type: vehFieldType_t::VF_VECTOR,
    },
    vehField_t {
        name: c"speedMax".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, speedMax) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"turboSpeed".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, turboSpeed) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"speedMin".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, speedMin) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"speedIdle".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, speedIdle) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"accelIdle".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, accelIdle) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"acceleration".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, acceleration) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"decelIdle".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, decelIdle) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"throttleSticks".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, throttleSticks) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"strafePerc".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, strafePerc) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"bankingSpeed".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, bankingSpeed) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"pitchLimit".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, pitchLimit) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"rollLimit".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, rollLimit) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"braking".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, braking) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"mouseYaw".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, mouseYaw) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"mousePitch".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, mousePitch) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"turningSpeed".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, turningSpeed) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"turnWhenStopped".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, turnWhenStopped) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"traction".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, traction) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"friction".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, friction) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"maxSlope".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, maxSlope) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"speedDependantTurning".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, speedDependantTurning) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"mass".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, mass) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"armor".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, armor) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"shields".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, shields) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"shieldRechargeMS".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, shieldRechargeMS) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"toughness".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, toughness) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"malfunctionArmorLevel".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, malfunctionArmorLevel) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"surfDestruction".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, surfDestruction) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"model".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, model) as i32,
        r#type: vehFieldType_t::VF_LSTRING,
    },
    vehField_t {
        name: c"skin".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, skin) as i32,
        r#type: vehFieldType_t::VF_LSTRING,
    },
    vehField_t {
        name: c"g2radius".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, g2radius) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"riderAnim".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, riderAnim) as i32,
        r#type: vehFieldType_t::VF_ANIM,
    },
    vehField_t {
        name: c"droidNPC".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, droidNPC) as i32,
        r#type: vehFieldType_t::VF_LSTRING,
    },
    vehField_t {
        name: c"radarIcon".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, radarIconHandle) as i32,
        r#type: vehFieldType_t::VF_SHADER_NOMIP,
    },
    vehField_t {
        name: c"dmgIndicFrame".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, dmgIndicFrameHandle) as i32,
        r#type: vehFieldType_t::VF_SHADER_NOMIP,
    },
    vehField_t {
        name: c"dmgIndicShield".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, dmgIndicShieldHandle) as i32,
        r#type: vehFieldType_t::VF_SHADER_NOMIP,
    },
    vehField_t {
        name: c"dmgIndicBackground".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, dmgIndicBackgroundHandle) as i32,
        r#type: vehFieldType_t::VF_SHADER_NOMIP,
    },
    vehField_t {
        name: c"icon_front".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iconFrontHandle) as i32,
        r#type: vehFieldType_t::VF_SHADER_NOMIP,
    },
    vehField_t {
        name: c"icon_back".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iconBackHandle) as i32,
        r#type: vehFieldType_t::VF_SHADER_NOMIP,
    },
    vehField_t {
        name: c"icon_right".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iconRightHandle) as i32,
        r#type: vehFieldType_t::VF_SHADER_NOMIP,
    },
    vehField_t {
        name: c"icon_left".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iconLeftHandle) as i32,
        r#type: vehFieldType_t::VF_SHADER_NOMIP,
    },
    vehField_t {
        name: c"crosshairShader".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, crosshairShaderHandle) as i32,
        r#type: vehFieldType_t::VF_SHADER_NOMIP,
    },
    vehField_t {
        name: c"shieldShader".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, shieldShaderHandle) as i32,
        r#type: vehFieldType_t::VF_SHADER,
    },
    vehField_t {
        name: c"health_front".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, health_front) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"health_back".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, health_back) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"health_right".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, health_right) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"health_left".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, health_left) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"soundOn".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, soundOn) as i32,
        r#type: vehFieldType_t::VF_SOUND,
    },
    vehField_t {
        name: c"soundOff".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, soundOff) as i32,
        r#type: vehFieldType_t::VF_SOUND,
    },
    vehField_t {
        name: c"soundLoop".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, soundLoop) as i32,
        r#type: vehFieldType_t::VF_SOUND,
    },
    vehField_t {
        name: c"soundTakeOff".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, soundTakeOff) as i32,
        r#type: vehFieldType_t::VF_SOUND,
    },
    vehField_t {
        name: c"soundEngineStart".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, soundEngineStart) as i32,
        r#type: vehFieldType_t::VF_SOUND_CLIENT,
    },
    vehField_t {
        name: c"soundSpin".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, soundSpin) as i32,
        r#type: vehFieldType_t::VF_SOUND,
    },
    vehField_t {
        name: c"soundTurbo".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, soundTurbo) as i32,
        r#type: vehFieldType_t::VF_SOUND,
    },
    vehField_t {
        name: c"soundHyper".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, soundHyper) as i32,
        r#type: vehFieldType_t::VF_SOUND_CLIENT,
    },
    vehField_t {
        name: c"soundLand".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, soundLand) as i32,
        r#type: vehFieldType_t::VF_SOUND,
    },
    vehField_t {
        name: c"soundFlyBy".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, soundFlyBy) as i32,
        r#type: vehFieldType_t::VF_SOUND_CLIENT,
    },
    vehField_t {
        name: c"soundFlyBy2".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, soundFlyBy2) as i32,
        r#type: vehFieldType_t::VF_SOUND_CLIENT,
    },
    vehField_t {
        name: c"soundShift1".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, soundShift1) as i32,
        r#type: vehFieldType_t::VF_SOUND,
    },
    vehField_t {
        name: c"soundShift2".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, soundShift2) as i32,
        r#type: vehFieldType_t::VF_SOUND,
    },
    vehField_t {
        name: c"soundShift3".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, soundShift3) as i32,
        r#type: vehFieldType_t::VF_SOUND,
    },
    vehField_t {
        name: c"soundShift4".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, soundShift4) as i32,
        r#type: vehFieldType_t::VF_SOUND,
    },
    vehField_t {
        name: c"exhaustFX".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iExhaustFX) as i32,
        r#type: vehFieldType_t::VF_EFFECT_CLIENT,
    },
    vehField_t {
        name: c"turboFX".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iTurboFX) as i32,
        r#type: vehFieldType_t::VF_EFFECT_CLIENT,
    },
    vehField_t {
        name: c"turboStartFX".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iTurboStartFX) as i32,
        r#type: vehFieldType_t::VF_EFFECT,
    },
    vehField_t {
        name: c"trailFX".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iTrailFX) as i32,
        r#type: vehFieldType_t::VF_EFFECT_CLIENT,
    },
    vehField_t {
        name: c"impactFX".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iImpactFX) as i32,
        r#type: vehFieldType_t::VF_EFFECT_CLIENT,
    },
    vehField_t {
        name: c"explodeFX".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iExplodeFX) as i32,
        r#type: vehFieldType_t::VF_EFFECT,
    },
    vehField_t {
        name: c"wakeFX".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iWakeFX) as i32,
        r#type: vehFieldType_t::VF_EFFECT_CLIENT,
    },
    vehField_t {
        name: c"dmgFX".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iDmgFX) as i32,
        r#type: vehFieldType_t::VF_EFFECT_CLIENT,
    },
    vehField_t {
        name: c"injureFX".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iInjureFX) as i32,
        r#type: vehFieldType_t::VF_EFFECT_CLIENT,
    },
    vehField_t {
        name: c"noseFX".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iNoseFX) as i32,
        r#type: vehFieldType_t::VF_EFFECT_CLIENT,
    },
    vehField_t {
        name: c"lwingFX".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iLWingFX) as i32,
        r#type: vehFieldType_t::VF_EFFECT_CLIENT,
    },
    vehField_t {
        name: c"rwingFX".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, iRWingFX) as i32,
        r#type: vehFieldType_t::VF_EFFECT_CLIENT,
    },
    vehField_t {
        name: c"weap1".as_ptr(),
        ofs: (OFS_WEAPON + 0 * SZ_VWS + core::mem::offset_of!(vehWeaponStats_t, ID)) as i32,
        r#type: vehFieldType_t::VF_WEAPON,
    },
    vehField_t {
        name: c"weap2".as_ptr(),
        ofs: (OFS_WEAPON + 1 * SZ_VWS + core::mem::offset_of!(vehWeaponStats_t, ID)) as i32,
        r#type: vehFieldType_t::VF_WEAPON,
    },
    vehField_t {
        name: c"weap1Delay".as_ptr(),
        ofs: (OFS_WEAPON + 0 * SZ_VWS + core::mem::offset_of!(vehWeaponStats_t, delay)) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"weap2Delay".as_ptr(),
        ofs: (OFS_WEAPON + 1 * SZ_VWS + core::mem::offset_of!(vehWeaponStats_t, delay)) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"weap1Link".as_ptr(),
        ofs: (OFS_WEAPON + 0 * SZ_VWS + core::mem::offset_of!(vehWeaponStats_t, linkable)) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"weap2Link".as_ptr(),
        ofs: (OFS_WEAPON + 1 * SZ_VWS + core::mem::offset_of!(vehWeaponStats_t, linkable)) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"weap1Aim".as_ptr(),
        ofs: (OFS_WEAPON + 0 * SZ_VWS + core::mem::offset_of!(vehWeaponStats_t, aimCorrect)) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"weap2Aim".as_ptr(),
        ofs: (OFS_WEAPON + 1 * SZ_VWS + core::mem::offset_of!(vehWeaponStats_t, aimCorrect)) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"weap1AmmoMax".as_ptr(),
        ofs: (OFS_WEAPON + 0 * SZ_VWS + core::mem::offset_of!(vehWeaponStats_t, ammoMax)) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"weap2AmmoMax".as_ptr(),
        ofs: (OFS_WEAPON + 1 * SZ_VWS + core::mem::offset_of!(vehWeaponStats_t, ammoMax)) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"weap1AmmoRechargeMS".as_ptr(),
        ofs: (OFS_WEAPON + 0 * SZ_VWS + core::mem::offset_of!(vehWeaponStats_t, ammoRechargeMS))
            as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"weap2AmmoRechargeMS".as_ptr(),
        ofs: (OFS_WEAPON + 1 * SZ_VWS + core::mem::offset_of!(vehWeaponStats_t, ammoRechargeMS))
            as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"weap1SoundNoAmmo".as_ptr(),
        ofs: (OFS_WEAPON + 0 * SZ_VWS + core::mem::offset_of!(vehWeaponStats_t, soundNoAmmo))
            as i32,
        r#type: vehFieldType_t::VF_SOUND_CLIENT,
    },
    vehField_t {
        name: c"weap2SoundNoAmmo".as_ptr(),
        ofs: (OFS_WEAPON + 1 * SZ_VWS + core::mem::offset_of!(vehWeaponStats_t, soundNoAmmo))
            as i32,
        r#type: vehFieldType_t::VF_SOUND_CLIENT,
    },
    vehField_t {
        name: c"weapMuzzle1".as_ptr(),
        ofs: (OFS_MUZZLE + 0 * 4) as i32,
        r#type: vehFieldType_t::VF_WEAPON,
    },
    vehField_t {
        name: c"weapMuzzle2".as_ptr(),
        ofs: (OFS_MUZZLE + 1 * 4) as i32,
        r#type: vehFieldType_t::VF_WEAPON,
    },
    vehField_t {
        name: c"weapMuzzle3".as_ptr(),
        ofs: (OFS_MUZZLE + 2 * 4) as i32,
        r#type: vehFieldType_t::VF_WEAPON,
    },
    vehField_t {
        name: c"weapMuzzle4".as_ptr(),
        ofs: (OFS_MUZZLE + 3 * 4) as i32,
        r#type: vehFieldType_t::VF_WEAPON,
    },
    vehField_t {
        name: c"weapMuzzle5".as_ptr(),
        ofs: (OFS_MUZZLE + 4 * 4) as i32,
        r#type: vehFieldType_t::VF_WEAPON,
    },
    vehField_t {
        name: c"weapMuzzle6".as_ptr(),
        ofs: (OFS_MUZZLE + 5 * 4) as i32,
        r#type: vehFieldType_t::VF_WEAPON,
    },
    vehField_t {
        name: c"weapMuzzle7".as_ptr(),
        ofs: (OFS_MUZZLE + 6 * 4) as i32,
        r#type: vehFieldType_t::VF_WEAPON,
    },
    vehField_t {
        name: c"weapMuzzle8".as_ptr(),
        ofs: (OFS_MUZZLE + 7 * 4) as i32,
        r#type: vehFieldType_t::VF_WEAPON,
    },
    vehField_t {
        name: c"weapMuzzle9".as_ptr(),
        ofs: (OFS_MUZZLE + 8 * 4) as i32,
        r#type: vehFieldType_t::VF_WEAPON,
    },
    vehField_t {
        name: c"weapMuzzle10".as_ptr(),
        ofs: (OFS_MUZZLE + 9 * 4) as i32,
        r#type: vehFieldType_t::VF_WEAPON,
    },
    vehField_t {
        name: c"landingHeight".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, landingHeight) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"gravity".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, gravity) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"hoverHeight".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, hoverHeight) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"hoverStrength".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, hoverStrength) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"waterProof".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, waterProof) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"bouyancy".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, bouyancy) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"fuelMax".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, fuelMax) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"fuelRate".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, fuelRate) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turboDuration".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, turboDuration) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turboRecharge".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, turboRecharge) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"visibility".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, visibility) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"loudness".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, loudness) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"explosionRadius".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, explosionRadius) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"explosionDamage".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, explosionDamage) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"maxPassengers".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, maxPassengers) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"hideRider".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, hideRider) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"killRiderOnDeath".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, killRiderOnDeath) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"flammable".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, flammable) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"explosionDelay".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, explosionDelay) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"cameraOverride".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, cameraOverride) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"cameraRange".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, cameraRange) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"cameraVertOffset".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, cameraVertOffset) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"cameraHorzOffset".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, cameraHorzOffset) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"cameraPitchOffset".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, cameraPitchOffset) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"cameraFOV".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, cameraFOV) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"cameraAlpha".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, cameraAlpha) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"cameraPitchDependantVertOffset".as_ptr(),
        ofs: core::mem::offset_of!(vehicleInfo_t, cameraPitchDependantVertOffset) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"turret1Weap".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, iWeapon)) as i32,
        r#type: vehFieldType_t::VF_WEAPON,
    },
    vehField_t {
        name: c"turret1Delay".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, iDelay)) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret1AmmoMax".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, iAmmoMax)) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret1AmmoRechargeMS".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, iAmmoRechargeMS))
            as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret1YawBone".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, yawBone)) as i32,
        r#type: vehFieldType_t::VF_LSTRING,
    },
    vehField_t {
        name: c"turret1PitchBone".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, pitchBone)) as i32,
        r#type: vehFieldType_t::VF_LSTRING,
    },
    vehField_t {
        name: c"turret1YawAxis".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, yawAxis)) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret1PitchAxis".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, pitchAxis)) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret1ClampYawL".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, yawClampLeft))
            as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"turret1ClampYawR".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, yawClampRight))
            as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"turret1ClampPitchU".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, pitchClampUp))
            as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"turret1ClampPitchD".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, pitchClampDown))
            as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"turret1Muzzle1".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, iMuzzle) + 0 * 4)
            as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret1Muzzle2".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, iMuzzle) + 1 * 4)
            as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret1TurnSpeed".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, fTurnSpeed)) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"turret1AI".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, bAI)) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"turret1AILead".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, bAILead)) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"turret1AIRange".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, fAIRange)) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"turret1PassengerNum".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, passengerNum))
            as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret1GunnerViewTag".as_ptr(),
        ofs: (OFS_TURRET + 0 * SZ_TURRET + core::mem::offset_of!(turretStats_t, gunnerViewTag))
            as i32,
        r#type: vehFieldType_t::VF_LSTRING,
    },
    vehField_t {
        name: c"turret2Weap".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, iWeapon)) as i32,
        r#type: vehFieldType_t::VF_WEAPON,
    },
    vehField_t {
        name: c"turret2Delay".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, iDelay)) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret2AmmoMax".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, iAmmoMax)) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret2AmmoRechargeMS".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, iAmmoRechargeMS))
            as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret2YawBone".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, yawBone)) as i32,
        r#type: vehFieldType_t::VF_LSTRING,
    },
    vehField_t {
        name: c"turret2PitchBone".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, pitchBone)) as i32,
        r#type: vehFieldType_t::VF_LSTRING,
    },
    vehField_t {
        name: c"turret2YawAxis".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, yawAxis)) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret2PitchAxis".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, pitchAxis)) as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret2ClampYawL".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, yawClampLeft))
            as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"turret2ClampYawR".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, yawClampRight))
            as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"turret2ClampPitchU".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, pitchClampUp))
            as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"turret2ClampPitchD".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, pitchClampDown))
            as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"turret2Muzzle1".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, iMuzzle) + 0 * 4)
            as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret2Muzzle2".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, iMuzzle) + 1 * 4)
            as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret2TurnSpeed".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, fTurnSpeed)) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"turret2AI".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, bAI)) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"turret2AILead".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, bAILead)) as i32,
        r#type: vehFieldType_t::VF_BOOL,
    },
    vehField_t {
        name: c"turret2AIRange".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, fAIRange)) as i32,
        r#type: vehFieldType_t::VF_FLOAT,
    },
    vehField_t {
        name: c"turret2PassengerNum".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, passengerNum))
            as i32,
        r#type: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: c"turret2GunnerViewTag".as_ptr(),
        ofs: (OFS_TURRET + 1 * SZ_TURRET + core::mem::offset_of!(turretStats_t, gunnerViewTag))
            as i32,
        r#type: vehFieldType_t::VF_LSTRING,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: -1,
        r#type: vehFieldType_t::VF_INT,
    },
];

/// Raven `stringID_table_t VehicleTable[VH_NUM_VEHICLES+1]`.
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:671-679`
pub const VehicleTable: [stringID_table_t; 7] = [
    stringID_table_t {
        name: c"VH_NONE".as_ptr() as *mut core::ffi::c_char,
        id: 0,
    },
    stringID_table_t {
        name: c"VH_WALKER".as_ptr() as *mut core::ffi::c_char,
        id: 1,
    },
    stringID_table_t {
        name: c"VH_FIGHTER".as_ptr() as *mut core::ffi::c_char,
        id: 2,
    },
    stringID_table_t {
        name: c"VH_SPEEDER".as_ptr() as *mut core::ffi::c_char,
        id: 3,
    },
    stringID_table_t {
        name: c"VH_ANIMAL".as_ptr() as *mut core::ffi::c_char,
        id: 4,
    },
    stringID_table_t {
        name: c"VH_FLIER".as_ptr() as *mut core::ffi::c_char,
        id: 5,
    },
    stringID_table_t {
        name: core::ptr::null_mut(),
        id: -1,
    },
];
