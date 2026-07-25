//! `BgState` — the session-lifetime bg working state.
//!
//! Raven scattered the bg tier's session-lifetime tables across six `.c` files
//! as file-scope statics (§B3 forbids that here). This struct owns them in one
//! place. `GameWorld` owns the one game-side instance and reaches it as
//! `world.bg_state`; a cgame `BgState` would be owned there later. The tables
//! are filled by their loaders (`BG_ParseAnimationFile`, `BG_VehicleLoadParms`,
//! `BG_ParseSaberParms`, item registration) in later passes — here they are the
//! owned containers, empty until loaded.
//!
//! The one member that must be bit-exact from day one is the faithful LCG RNG
//! (`rng`); see [`Rng`].
#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::{c_char, c_int};

use crate::prelude::*;
use mp_qshared::shared::com_parse::QSharedScratch;

use super::rng::Rng;
use crate::bg_panimate::MAX_ANIM_FILES;
use crate::public::anim_number::animNumber_t;
use crate::public::bg_loaded_anim::bgLoadedAnim_t;
use crate::public::bg_loaded_events::bgLoadedEvents_t;
use crate::public::saber_move_data::saberMoveData_t;
use crate::public::saber_move_data_table::saberMoveData;
use crate::saga::siege_team_t::MAX_SIEGE_TEAMS;
use crate::vehicles::vehicle_s::MAX_VEHICLES;
use mp_qshared::shared::limits::MAX_VEH_WEAPONS;

/// The bg tier's session-lifetime state, owned by `GameWorld`.
///
/// Raven's fixed static tables become owned collections (porting-rules §9:
/// pools/tables → `Vec`/`Box`). Each field cites its owning Raven static.
pub struct BgState {
    /// The faithful LCG RNG. The single parity-critical member.
    /// Source: `oracle/codemp/game/q_math.c:1432`
    pub rng: Rng,

    /// `q_shared.c` file-static parse/format scratch (safe-state Stage 3):
    /// the COM parser session state + token buffer, and the `va`/
    /// `Info_ValueForKey` rotating return buffers. Lives on `BgState` (not a
    /// game-only scratch struct) because both tiers parse through it — bg's
    /// saber/vehicle loaders take `&mut BgState` and never see `GameContext`.
    /// Buffer-rotation index semantics preserved exactly.
    /// Source: `oracle/codemp/game/q_shared.c` file statics.
    pub qs: QSharedScratch,

    // --- `bg_panimate.c` animation tables ---
    /// Raven `bgLoadedAnim_t bgAllAnims[MAX_ANIM_FILES]` — per-model animation
    /// sets, each pointing at a heap `animation_t` array.
    /// Source: `oracle/codemp/game/bg_panimate.c:1702`
    pub bgAllAnims: Vec<bgLoadedAnim_t>,
    /// Raven `int bgNumAllAnims = 2` — next free slot in `bgAllAnims` (0 is
    /// always humanoid, 1 always rockettrooper; see `new()` for why the init
    /// value matters).
    /// Source: `oracle/codemp/game/bg_panimate.c:1703`
    pub bgNumAllAnims: c_int,
    /// Raven `bgLoadedEvents_t bgAllEvents[MAX_ANIM_FILES]`.
    /// Source: `oracle/codemp/game/bg_panimate.c:2166`
    pub bgAllEvents: Vec<bgLoadedEvents_t>,
    /// Raven `int bgNumAnimEvents = 1` (first one is null/default).
    /// Source: `oracle/codemp/game/bg_panimate.c:2167`
    pub bgNumAnimEvents: c_int,
    /// Raven `animation_t bgHumanoidAnimations[MAX_TOTALANIMATIONS]` — the only
    /// statically-allocated animation set.
    /// Source: `oracle/codemp/game/bg_panimate.c:1672`
    pub bgHumanoidAnimations: Vec<animation_t>,
    /// Raven `char BGPAFtext[60000]` — animation-config parse scratch.
    /// Source: `oracle/codemp/game/bg_panimate.c:1669`
    pub BGPAFtext: Vec<u8>,
    /// Raven `qboolean BGPAFtextLoaded`.
    /// Source: `oracle/codemp/game/bg_panimate.c:1671`
    pub BGPAFtextLoaded: qboolean,

    // --- `bg_saberLoad.c` saber-parm tables ---
    /// Raven `char SaberParms[MAX_SABER_DATA_SIZE]` — accumulated saber `.sab`
    /// text kept for lazy re-parse.
    /// Source: `oracle/codemp/game/bg_saberLoad.c:44`
    pub SaberParms: Vec<u8>,
    /// Raven `char bgSaberParseTBuffer[MAX_SABER_DATA_SIZE]` — per-file read
    /// scratch during saber-parm loading.
    /// Source: `oracle/codemp/game/bg_saberLoad.c:2736`
    pub bgSaberParseTBuffer: Vec<u8>,

    // --- `bg_saber.c` saber-move tables ---
    /// Raven `saberMoveData_t saberMoveData[LS_MOVE_MAX]` — per-move animation
    /// and chaining data (a static const array in C, stored here as a static ref).
    /// Source: `oracle/codemp/game/bg_saber.c:120-321`
    pub saberMoveData: &'static [saberMoveData_t],

    // --- `bg_vehicleLoad.c` vehicle tables ---
    /// Raven `vehWeaponInfo_t g_vehWeaponInfo[MAX_VEH_WEAPONS]`.
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:103`
    pub g_vehWeaponInfo: Vec<vehWeaponInfo_t>,
    /// Raven `int numVehicleWeapons = 1` (first one is null/default).
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:104`
    pub numVehicleWeapons: c_int,
    /// Raven `vehicleInfo_t g_vehicleInfo[MAX_VEHICLES]`.
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:106`
    pub g_vehicleInfo: Vec<vehicleInfo_t>,
    /// Raven `int numVehicles = 0` (first one is null/default).
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:107`
    pub numVehicles: c_int,
    /// Raven `char VehWeaponParms[MAX_VEH_WEAPON_DATA_SIZE]` — accumulated
    /// `.vwp` text scratch buffer (§B3: file-scope static -> owned field).
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:69`
    pub VehWeaponParms: Vec<c_char>,
    /// Raven `char VehicleParms[MAX_VEHICLE_DATA_SIZE]` — accumulated `.veh`
    /// text scratch buffer.
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:70`
    pub VehicleParms: Vec<c_char>,

    // --- `bg_saga.c` siege class tables ---
    /// Raven `siegeClass_t bgSiegeClasses[MAX_SIEGE_CLASSES]` — siege gametype
    /// player class definitions, loaded from siege config files.
    /// Source: `oracle/codemp/game/bg_saga.c:38`
    pub bgSiegeClasses: Vec<siegeClass_t>,
    /// Raven `int bgNumSiegeClasses = 0` — count of loaded siege classes.
    /// Source: `oracle/codemp/game/bg_saga.c:39`
    pub bgNumSiegeClasses: c_int,

    // --- `bg_saga.c` siege team tables (module-scope mutable state, now owned by BgState) ---
    /// Raven `siegeTeam_t bgSiegeTeams[MAX_SIEGE_TEAMS]` — siege team definitions.
    /// Source: `oracle/codemp/game/bg_saga.c:41`
    pub bgSiegeTeams: Vec<siegeTeam_t>,
    /// Raven `int bgNumSiegeTeams = 0` — count of loaded siege teams.
    /// Source: `oracle/codemp/game/bg_saga.c:42`
    pub bgNumSiegeTeams: c_int,
    /// Raven `siegeTeam_t *team1Theme` — theme team for side 1 (points into
    /// `bgSiegeTeams`; NULL until set).
    /// Source: `oracle/codemp/game/bg_saga.c:35`
    pub team1Theme: *mut siegeTeam_t,
    /// Raven `siegeTeam_t *team2Theme` — theme team for side 2.
    /// Source: `oracle/codemp/game/bg_saga.c:36`
    pub team2Theme: *mut siegeTeam_t,
    /// Raven `extern char siege_info[MAX_SIEGE_INFO_SIZE]` — accumulated siege
    /// config text kept for lazy re-parse (`.siege` file contents).
    /// Source: `oracle/codemp/game/bg_saga.h:112`
    pub siege_info: Vec<u8>,
    /// Raven `extern int siege_valid` — whether `siege_info` currently holds a
    /// loaded siege config.
    /// Source: `oracle/codemp/game/bg_saga.h:113`
    pub siege_valid: c_int,

    // --- `bg_misc.c` string pool ---
    /// Raven `static char bg_pool[MAX_POOL_SIZE]` — the `BG_Alloc` bump pool.
    /// Source: `oracle/codemp/game/bg_misc.c:3324`
    pub bg_pool: Vec<u8>,
    /// Raven `static int bg_poolSize = 0` — bump allocation point.
    /// Source: `oracle/codemp/game/bg_misc.c:3325`
    pub bg_poolSize: c_int,
    /// Raven `static int bg_poolTail = MAX_POOL_SIZE`.
    /// Source: `oracle/codemp/game/bg_misc.c:3326`
    pub bg_poolTail: c_int,

    // --- `bg_pmove.c` cross-frame debug counter ---
    /// Raven `int c_pmove = 0` — the PmoveSingle journal counter; genuine
    /// cross-frame state, so it's an owned field.
    /// Source: `oracle/codemp/game/bg_pmove.c:57`
    pub c_pmove: c_int,

    // --- game-cvar mirrors (bg code cannot reach GameCvars) ---
    /// Mirror of the `bg_fighterAltControl` cvar's `.integer`, written by the
    /// game-tier cvar register/update path for `BG_UnrestrainedPitchRoll`.
    /// Raven read the `vmCvar_t` global directly (`extern vmCvar_t
    /// bg_fighterAltControl`).
    /// Source: `oracle/codemp/game/bg_pmove.c:7783`
    pub bg_fighterAltControl: c_int,
}

impl BgState {
    /// A freshly zeroed session state with the LCG seeded to Raven's
    /// `holdrand = 0x89abcdef`; all tables empty until their loaders run.
    /// The `BG_Alloc` pool gets the QAGAME arm's `MAX_POOL_SIZE`; a module
    /// with a different Raven pool arm (ui: 512000, DEC-36 addendum 11) uses
    /// [`BgState::with_pool_size`].
    pub fn new() -> Self {
        Self::with_pool_size(crate::bg_misc::MAX_POOL_SIZE)
    }

    /// [`BgState::new`] with an explicit `BG_Alloc` pool size — Raven sized
    /// `bg_pool[MAX_POOL_SIZE]` per module (`#define` arms in bg_misc.c), so
    /// each hosting module passes its own arm (§F20 duplicate-don't-unify).
    /// Source: `oracle/codemp/game/bg_misc.c:3311-3316`
    pub fn with_pool_size(pool_size: c_int) -> Self {
        Self {
            qs: QSharedScratch::zeroed(),
            rng: Rng::new(),
            // Sized like Raven's fixed `bgLoadedAnim_t bgAllAnims[MAX_ANIM_FILES]` /
            // `bgLoadedEvents_t bgAllEvents[MAX_ANIM_FILES]` zeroed statics
            // (loaders index them directly rather than push/grow).
            // `bgLoadedAnim_t` owns a `String` (`filename`), so it is seeded via
            // its `Default` (empty name + null `anims`), not `mem::zeroed()`.
            bgAllAnims: vec![bgLoadedAnim_t::default(); MAX_ANIM_FILES as usize],
            // Raven initialises this to 2: slot 0 is always the humanoid set and
            // slot 1 always rockettrooper, so dynamically-parsed sets start at 2
            // (`BG_ParseAnimationFile`'s `nextIndex = bgNumAllAnims`). Starting at
            // 0 lets the first non-humanoid parse (e.g. a t2_trip swoop, which
            // spawns during map load before any player humanoid) grab slot 0,
            // overwrite the humanoid set, and latch `BGPAFtextLoaded`, breaking
            // every player's animations.
            bgNumAllAnims: 2,
            // `bgLoadedEvents_t` owns a `String` (`filename`); seeded via its
            // `Default`, not `mem::zeroed()`.
            bgAllEvents: vec![bgLoadedEvents_t::default(); MAX_ANIM_FILES as usize],
            // Raven initialises this to 1 (first entry is the null/default).
            bgNumAnimEvents: 1,
            // Sized like Raven's fixed `animation_t bgHumanoidAnimations[
            // MAX_TOTALANIMATIONS]` zeroed static (the only statically-allocated
            // animation set): `BG_ParseAnimationFile` receives `.as_mut_ptr()` as
            // `animset` and writes MAX_ANIMATIONS entries (with face/legs code
            // indexing up to MAX_TOTALANIMATIONS), so an empty `Vec` gives a
            // dangling pointer. Same fixed-array pre-size convention as
            // `bgAllAnims`/`g_vehicleInfo` above.
            bgHumanoidAnimations: vec![
                unsafe { core::mem::zeroed() };
                animNumber_t::MAX_TOTALANIMATIONS as usize
            ],
            BGPAFtext: Vec::new(),
            BGPAFtextLoaded: qfalse,
            SaberParms: Vec::new(),
            bgSaberParseTBuffer: Vec::new(),
            saberMoveData: &saberMoveData,
            // Sized like Raven's fixed `vehWeaponInfo_t g_vehWeaponInfo[MAX_VEH_WEAPONS]`
            // / `vehicleInfo_t g_vehicleInfo[MAX_VEHICLES]` zeroed statics: the
            // loaders index them directly at fixed slots (e.g. `g_vehicleInfo[
            // VEHICLE_BASE]`, `bg_vehicleLoad.rs:856`) rather than push/grow, so
            // an empty `Vec` would panic on the first index. Same fixed-array
            // pre-size convention as `bgAllAnims`/`VehicleParms` above.
            g_vehWeaponInfo: (0..MAX_VEH_WEAPONS)
                .map(|_| unsafe { core::mem::zeroed() })
                .collect(),
            // Raven initialises to 1 (first entry is the null/default).
            numVehicleWeapons: 1,
            g_vehicleInfo: (0..MAX_VEHICLES)
                .map(|_| unsafe { core::mem::zeroed() })
                .collect(),
            numVehicles: 0,
            // Sized like Raven's fixed `char[MAX_VEH_WEAPON_DATA_SIZE/
            // MAX_VEHICLE_DATA_SIZE]` scratch buffers (loaders index them
            // directly rather than push/grow).
            VehWeaponParms: vec![0; crate::bg_vehicleLoad_tables::MAX_VEH_WEAPON_DATA_SIZE],
            VehicleParms: vec![0; crate::bg_vehicleLoad_tables::MAX_VEHICLE_DATA_SIZE],
            // Sized like Raven's fixed `siegeClass_t bgSiegeClasses[MAX_SIEGE_CLASSES]`
            // zeroed static: consumers index it directly (still-parked loader leaves
            // it unpopulated), so an empty `Vec` would panic where C reads zeros.
            // Same fixed-array pre-size convention as `g_vehicleInfo`/`bgAllAnims`.
            bgSiegeClasses: (0..MAX_SIEGE_CLASSES)
                .map(|_| siegeClass_t::default())
                .collect(),
            bgNumSiegeClasses: 0,
            // Sized like Raven's fixed `siegeTeam_t bgSiegeTeams[MAX_SIEGE_TEAMS]`
            // zeroed static: the team loader indexes `[bgNumSiegeTeams]` directly
            // (bg_saga.c pattern), so an empty `Vec` panics on the first parse.
            bgSiegeTeams: (0..MAX_SIEGE_TEAMS)
                .map(|_| unsafe { core::mem::zeroed() })
                .collect(),
            bgNumSiegeTeams: 0,
            team1Theme: core::ptr::null_mut(),
            team2Theme: core::ptr::null_mut(),
            // Sized like Raven's fixed `char[MAX_SIEGE_INFO_SIZE]` scratch buffer
            // (loaders index it directly rather than push/grow).
            siege_info: vec![0; 16384],
            siege_valid: 0,
            // Sized like Raven's fixed `char bg_pool[MAX_POOL_SIZE]` zeroed static
            // (`BG_Alloc`/`BG_TempAlloc` write through `.as_mut_ptr()` to a fixed
            // extent, so an empty `Vec` gives a dangling pointer). `bg_poolTail`
            // mirrors Raven's static initializer `bg_poolTail = MAX_POOL_SIZE`
            // (bg_misc.c:3326) — not 0 — so the descending temp-alloc arithmetic
            // starts at the top of the pool. Same fixed-array pre-size convention
            // as `bgHumanoidAnimations`/`g_vehicleInfo` above.
            bg_pool: vec![0; pool_size as usize],
            bg_poolSize: 0,
            bg_poolTail: pool_size,
            c_pmove: 0,
            bg_fighterAltControl: 0,
        }
    }
}

impl Default for BgState {
    fn default() -> Self {
        Self::new()
    }
}
