//! `BgState` — the session-lifetime bg working state (pass-3 ruling 12).
//!
//! Raven scattered the bg tier's session-lifetime tables across six `.c` files
//! as file-scope statics (§B3 forbids that here). This struct owns them in one
//! place. `GameWorld` owns the one game-side instance and reaches it as
//! `world.bg_state`; a cgame `BgState` would be owned there later. The tables
//! are filled by their loaders (`BG_ParseAnimationFile`, `BG_VehicleLoadParms`,
//! `BG_ParseSaberParms`, item registration) in later passes — here they are the
//! owned containers, empty until loaded.
//!
//! The one member that must be bit-exact from day one is the fork-3 RNG
//! (`rng`); see [`Rng`].
#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::c_int;

use crate::prelude::*;

use super::rng::Rng;
use mp_bg::public::bg_loaded_anim::bgLoadedAnim_t;
use mp_bg::public::bg_loaded_events::bgLoadedEvents_t;

/// The bg tier's session-lifetime state, owned by `GameWorld` (ruling 12).
///
/// Raven's fixed static tables become owned collections (porting-rules §9:
/// pools/tables → `Vec`/`Box`). Each field cites its owning Raven static.
pub struct BgState {
    /// Fork-3 LCG (ruling 15). The single parity-critical member.
    /// Source: `oracle/oracle/codemp/game/q_math.c:1432`
    pub rng: Rng,

    // --- `bg_panimate.c` animation tables ---
    /// Raven `bgLoadedAnim_t bgAllAnims[MAX_ANIM_FILES]` — per-model animation
    /// sets, each pointing at a heap `animation_t` array.
    /// Source: `oracle/oracle/codemp/game/bg_panimate.c:1702`
    pub bgAllAnims: Vec<bgLoadedAnim_t>,
    /// Raven `int bgNumAllAnims` — count of loaded entries in `bgAllAnims`.
    /// Source: `oracle/oracle/codemp/game/bg_panimate.c:1703`
    pub bgNumAllAnims: c_int,
    /// Raven `bgLoadedEvents_t bgAllEvents[MAX_ANIM_FILES]`.
    /// Source: `oracle/oracle/codemp/game/bg_panimate.c:2166`
    pub bgAllEvents: Vec<bgLoadedEvents_t>,
    /// Raven `int bgNumAnimEvents = 1` (first one is null/default).
    /// Source: `oracle/oracle/codemp/game/bg_panimate.c:2167`
    pub bgNumAnimEvents: c_int,
    /// Raven `animation_t bgHumanoidAnimations[MAX_TOTALANIMATIONS]` — the only
    /// statically-allocated animation set.
    /// Source: `oracle/oracle/codemp/game/bg_panimate.c:1672`
    pub bgHumanoidAnimations: Vec<animation_t>,
    /// Raven `char BGPAFtext[60000]` — animation-config parse scratch.
    /// Source: `oracle/oracle/codemp/game/bg_panimate.c:1669`
    pub BGPAFtext: Vec<u8>,
    /// Raven `qboolean BGPAFtextLoaded`.
    /// Source: `oracle/oracle/codemp/game/bg_panimate.c:1671`
    pub BGPAFtextLoaded: qboolean,

    // --- `bg_saberLoad.c` saber-parm tables ---
    /// Raven `char SaberParms[MAX_SABER_DATA_SIZE]` — accumulated saber `.sab`
    /// text kept for lazy re-parse.
    /// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:44`
    pub SaberParms: Vec<u8>,
    /// Raven `char bgSaberParseTBuffer[MAX_SABER_DATA_SIZE]` — per-file read
    /// scratch during saber-parm loading.
    /// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2736`
    pub bgSaberParseTBuffer: Vec<u8>,

    // --- `bg_vehicleLoad.c` vehicle tables ---
    /// Raven `vehWeaponInfo_t g_vehWeaponInfo[MAX_VEH_WEAPONS]`.
    /// Source: `oracle/oracle/codemp/game/bg_vehicleLoad.c:103`
    pub g_vehWeaponInfo: Vec<vehWeaponInfo_t>,
    /// Raven `int numVehicleWeapons = 1` (first one is null/default).
    /// Source: `oracle/oracle/codemp/game/bg_vehicleLoad.c:104`
    pub numVehicleWeapons: c_int,
    /// Raven `vehicleInfo_t g_vehicleInfo[MAX_VEHICLES]`.
    /// Source: `oracle/oracle/codemp/game/bg_vehicleLoad.c:106`
    pub g_vehicleInfo: Vec<vehicleInfo_t>,
    /// Raven `int numVehicles = 0` (first one is null/default).
    /// Source: `oracle/oracle/codemp/game/bg_vehicleLoad.c:107`
    pub numVehicles: c_int,

    // --- `bg_saga.c` siege class tables ---
    /// Raven `siegeClass_t bgSiegeClasses[MAX_SIEGE_CLASSES]` — siege gametype
    /// player class definitions, loaded from siege config files.
    /// Source: `oracle/oracle/codemp/game/bg_saga.c:38`
    pub bgSiegeClasses: Vec<siegeClass_t>,
    /// Raven `int bgNumSiegeClasses = 0` — count of loaded siege classes.
    /// Source: `oracle/oracle/codemp/game/bg_saga.c:39`
    pub bgNumSiegeClasses: c_int,

    // --- `bg_misc.c` string pool ---
    /// Raven `static char bg_pool[MAX_POOL_SIZE]` — the `BG_Alloc` bump pool.
    /// Source: `oracle/oracle/codemp/game/bg_misc.c:3324`
    pub bg_pool: Vec<u8>,
    /// Raven `static int bg_poolSize = 0` — bump allocation point.
    /// Source: `oracle/oracle/codemp/game/bg_misc.c:3325`
    pub bg_poolSize: c_int,
    /// Raven `static int bg_poolTail = MAX_POOL_SIZE`.
    /// Source: `oracle/oracle/codemp/game/bg_misc.c:3326`
    pub bg_poolTail: c_int,

    // --- `bg_pmove.c` cross-frame debug counter ---
    /// Raven `int c_pmove = 0` — the PmoveSingle journal counter (fork ruling 5:
    /// genuine cross-frame state → owned field).
    /// Source: `oracle/oracle/codemp/game/bg_pmove.c:57`
    pub c_pmove: c_int,
}

impl BgState {
    /// A freshly zeroed session state with the LCG seeded to Raven's
    /// `holdrand = 0x89abcdef`; all tables empty until their loaders run.
    pub fn new() -> Self {
        Self {
            rng: Rng::new(),
            bgAllAnims: Vec::new(),
            bgNumAllAnims: 0,
            bgAllEvents: Vec::new(),
            // Raven initialises this to 1 (first entry is the null/default).
            bgNumAnimEvents: 1,
            bgHumanoidAnimations: Vec::new(),
            BGPAFtext: Vec::new(),
            BGPAFtextLoaded: QFALSE,
            SaberParms: Vec::new(),
            bgSaberParseTBuffer: Vec::new(),
            g_vehWeaponInfo: Vec::new(),
            // Raven initialises to 1 (first entry is the null/default).
            numVehicleWeapons: 1,
            g_vehicleInfo: Vec::new(),
            numVehicles: 0,
            bgSiegeClasses: Vec::new(),
            bgNumSiegeClasses: 0,
            bg_pool: Vec::new(),
            bg_poolSize: 0,
            bg_poolTail: 0,
            c_pmove: 0,
        }
    }
}

impl Default for BgState {
    fn default() -> Self {
        Self::new()
    }
}
