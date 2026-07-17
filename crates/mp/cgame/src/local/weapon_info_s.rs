#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_float, c_int};

use mp_bg::public::item_id::ItemId;

use super::centity_s::centity_t;
use mp_qshared::shared::{fxHandle_t, qboolean, qhandle_t, sfxHandle_t, vec3_t};

/// Raven `weaponInfo_t`.
///
/// Module-internal (`cg_local.h`) — never crosses the engine seam, so the
/// layout is free: Raven's `gitem_t *item` holds the table [`ItemId`], and the
/// transcription-era `#[repr(C)]`/offset asserts are retired (DEC-31).
/// Type definition source: `oracle/codemp/cgame/cg_local.h:652-702`
pub struct weaponInfo_t {
    pub registered: qboolean,
    pub item: Option<ItemId>,

    /// the hands don't actually draw, they just position the weapon
    pub handsModel: qhandle_t,
    /// this is the pickup model
    pub weaponModel: qhandle_t,
    /// this is the in-view model used by the player
    pub viewModel: qhandle_t,
    pub barrelModel: qhandle_t,
    pub flashModel: qhandle_t,

    /// so it will rotate centered instead of by tag
    pub weaponMidpoint: vec3_t,

    pub flashDlight: c_float,
    pub flashDlightColor: vec3_t,

    pub weaponIcon: qhandle_t,
    pub ammoIcon: qhandle_t,

    pub ammoModel: qhandle_t,

    /// fast firing weapons randomly choose
    pub flashSound: [sfxHandle_t; 4],
    pub firingSound: sfxHandle_t,
    pub chargeSound: sfxHandle_t,
    pub muzzleEffect: fxHandle_t,
    pub missileModel: qhandle_t,
    pub missileSound: sfxHandle_t,
    pub missileTrailFunc:
        Option<unsafe extern "C" fn(cent: *mut centity_t, wi: *const weaponInfo_t)>,
    pub missileDlight: c_float,
    pub missileDlightColor: vec3_t,
    pub missileRenderfx: c_int,
    pub missileHitSound: sfxHandle_t,

    pub altFlashSound: [sfxHandle_t; 4],
    pub altFiringSound: sfxHandle_t,
    pub altChargeSound: sfxHandle_t,
    pub altMuzzleEffect: fxHandle_t,
    pub altMissileModel: qhandle_t,
    pub altMissileSound: sfxHandle_t,
    pub altMissileTrailFunc:
        Option<unsafe extern "C" fn(cent: *mut centity_t, wi: *const weaponInfo_t)>,
    pub altMissileDlight: c_float,
    pub altMissileDlightColor: vec3_t,
    pub altMissileRenderfx: c_int,
    pub altMissileHitSound: sfxHandle_t,

    pub selectSound: sfxHandle_t,

    pub readySound: sfxHandle_t,
    pub trailRadius: c_float,
    pub wiTrailTime: c_float,
}
