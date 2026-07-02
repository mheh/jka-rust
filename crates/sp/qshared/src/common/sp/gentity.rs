//! SP `gentity_t` copied from Raven `code/game/g_shared.h`.
//!
//! Type declaration source: `oracle/oracle/code/game/g_public.h:51`
//! Full struct layout source: `oracle/oracle/code/game/g_shared.h:514-825`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use crate::common::sp::qcommon::{entityState_t, gitem_t, parms_t};
use crate::shared::{qboolean, qhandle_t, vec3_t, vec4_t};

/// Raven SP `NUM_TIDS` (from `taskID_e`).
///
/// Definition source: `oracle/oracle/code/game/g_shared.h:20-34`
pub const NUM_TIDS: usize = 10;

/// Raven SP `NUM_BSETS` (from `bSet_e`).
///
/// Definition source: `oracle/oracle/code/game/bset.h:1-24`
pub const NUM_BSETS: usize = 17;

/// Raven SP `HL_MAX` (from `hitloc_e`).
///
/// Definition source: `oracle/oracle/code/game/hitlocs.h:4-31`
pub const HL_MAX: usize = 23;

/// Raven SP `MAX_INHAND_WEAPONS`.
///
/// Definition source: `oracle/oracle/code/game/g_shared.h:509`
pub const MAX_INHAND_WEAPONS: usize = 2;

// Raven's `#ifdef GAME_INCLUDE` block (g_shared.h:497-506) `#define`s
// `thinkFunc_t`/`clThinkFunc_t`/`reachedFunc_t`/`blockedFunc_t`/`touchFunc_t`/
// `useFunc_t`/`painFunc_t`/`dieFunc_t` down to plain `int` for exactly this
// struct — they hold indices into the real function-pointer enums declared in
// `g_functions.h`, not function pointers themselves. The `e_*Func` fields below
// are `c_int` to match.

/// Raven SP `moverState_t`.
///
/// The real named enum already exists (`sp_game::shared::mover_state_t`), but
/// `gentity_t` lives in `sp_qshared`, below the game tier in the crate graph, so
/// it cannot be referenced here. `c_int` is ABI-identical (both 4 bytes).
/// Type definition source: `oracle/oracle/code/game/g_local.h`
//TODO: Port moverState_t (cross-tier: real enum lives in sp_game)
pub type moverState_t = c_int;

/// Raven SP `material_t`.
///
/// The real named enum already exists (`sp_game::shared::material_t`), but
/// `gentity_t` lives in `sp_qshared`, below the game tier in the crate graph, so
/// it cannot be referenced here. `c_int` is ABI-identical (both 4 bytes).
/// Type definition source: `oracle/oracle/code/game/g_shared.h:37-58`
//TODO: Port material_t (cross-tier: real enum lives in sp_game)
pub type material_t = c_int;

/// Raven SP `team_t`.
///
/// The real named enum already exists (`sp_game::teams::team_t`), but
/// `gentity_t` lives in `sp_qshared`, below the game tier in the crate graph, so
/// it cannot be referenced here. `c_int` is ABI-identical (both 4 bytes).
/// Type definition source: `oracle/oracle/code/game/teams.h:4-13`
//TODO: Port team_t (cross-tier: real enum lives in sp_game)
pub type team_t = c_int;

/// Anonymous union for `gentity_t` (Raven: `union { char *roff; char *fxFile; };`
/// — anonymous in the header, needs a field name to exist in Rust).
///
/// Raven: the roff file to use, if there is one / name of the external effect file.
/// Type definition source: `oracle/oracle/code/game/g_shared.h:607-611`
#[repr(C)]
#[derive(Clone, Copy)]
pub union gentity_t_uRoff {
    pub roff: *mut c_char,
    pub fxFile: *mut c_char,
}

/// Anonymous union for `gentity_t` (Raven: `union { qboolean trigger_formation;
/// qboolean misc_dlight_active; qboolean has_bounced; };`).
///
/// Type definition source: `oracle/oracle/code/game/g_shared.h:668-673`
#[repr(C)]
#[derive(Clone, Copy)]
pub union gentity_t_uTriggerFormation {
    pub trigger_formation: qboolean,
    pub misc_dlight_active: qboolean,
    /// For thermal Det. we force at least one bounce to happen before it can do
    /// proximity checks.
    pub has_bounced: qboolean,
}

/// Anonymous union for `gentity_t` (Raven: `union { int wpIndex; int fxID; };`).
///
/// Type definition source: `oracle/oracle/code/game/g_shared.h:795-799`
#[repr(C)]
#[derive(Clone, Copy)]
pub union gentity_t_uWpIndex {
    pub wpIndex: c_int,
    /// id of the external effect file.
    pub fxID: c_int,
}

/// Anonymous union for `gentity_t` (Raven: `union { vec4_t finalRGBA; vec3_t
/// pos4; vec3_t modelAngles; };`).
///
/// Type definition source: `oracle/oracle/code/game/g_shared.h:805-810`
#[repr(C)]
#[derive(Clone, Copy)]
pub union gentity_t_uFinalRGBA {
    pub finalRGBA: vec4_t,
    pub pos4: vec3_t,
    /// For brush entities with an attached md3 model, as an offset to the
    /// brush's angles.
    pub modelAngles: vec3_t,
}

/// Raven SP `gentity_t`.
///
/// Type declaration source: `oracle/oracle/code/game/g_public.h:51`
/// Full struct layout source: `oracle/oracle/code/game/g_shared.h:514-825`
#[repr(C)]
pub struct gentity_t {
    /// communicated by server to clients
    pub s: entityState_t,
    // Raven: `struct gclient_s *client` (g_shared.h:516). `gclient_s` lives in
    // the game tier (`sp_game`), which `sp_qshared` cannot depend on. `*mut
    // c_void` is ABI-identical to `*mut gclient_s` (both pointer-sized).
    //TODO: Port gclient_s (cross-tier: real struct lives in sp_game)
    // Source: oracle/oracle/code/game/g_shared.h:387
    /// NULL if not a player (unless it's NPC ( if (this->NPC != NULL) ) <sigh>... -slc)
    pub client: *mut c_void,
    pub inuse: qboolean,
    /// qfalse if not in any good cluster
    pub linked: qboolean,
    /// SVF_NOCLIENT, SVF_BROADCAST, etc
    pub svFlags: c_int,
    /// if false, assume an explicit mins / maxs bounding box
    /// only set by gi.SetBrushModel
    pub bmodel: qboolean,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    /// CONTENTS_TRIGGER, CONTENTS_SOLID, CONTENTS_BODY, etc
    /// a non-solid entity should set to 0
    pub contents: c_int,
    /// derived from mins/maxs and origin + rotation
    pub absmin: vec3_t,
    pub absmax: vec3_t,
    /// currentOrigin will be used for all collision detection and world linking.
    /// it will not necessarily be the same as the trajectory evaluation for the
    /// current time, because each entity must be moved one at a time after time
    /// is advanced to avoid simultanious collision issues
    pub currentOrigin: vec3_t,
    pub currentAngles: vec3_t,
    /// objects never interact with their owners, to
    /// prevent player missiles from immediately
    /// colliding with their owner
    pub owner: *mut gentity_t,
    //TODO: Port CGhoul2Info_v
    // Source: oracle/oracle/code/game/ghoul2_shared.h:326-329
    // Raven's `CGhoul2Info_v` wraps a single `int mItem` handle into the shared
    // ghoul2 instance array; ABI width is 4 bytes.
    pub ghoul2: c_int,
    /// needed for g2 collision
    pub modelScale: vec3_t,
    /// set in QuakeEd
    pub classname: *mut c_char,
    /// set in QuakeEd
    pub spawnflags: c_int,
    /// FL_* variables
    pub flags: c_int,
    /// Normal model, or legs model on tri-models
    pub model: *mut c_char,
    /// Torso model
    pub model2: *mut c_char,
    /// sv.time when the object was freed
    pub freetime: c_int,
    /// events will be cleared EVENT_VALID_MSEC after set
    pub eventTime: c_int,
    pub freeAfterEvent: qboolean,
    /// 1.0 = continuous bounce, 0.0 = no bounce
    pub physicsBounce: f32,
    /// brushes with this content value will be collided against
    /// when moving.  items and corpses do not collide against
    /// players, for instance
    pub clipmask: c_int,
    pub speed: f32,
    pub resultspeed: f32,
    pub lastMoveTime: c_int,
    pub movedir: vec3_t,
    /// Where you were last frame
    pub lastOrigin: vec3_t,
    /// Where you were looking last frame
    pub lastAngles: vec3_t,
    /// How heavy you are
    pub mass: f32,
    /// Last time you impacted something
    pub lastImpact: c_int,
    pub watertype: c_int,
    pub waterlevel: c_int,
    pub wupdate: i16,
    pub prev_waterlevel: i16,
    /// set in editor, -1 = up, -2 = down
    pub angle: f32,
    pub target: *mut c_char,
    /// For multiple targets, not used for firing/triggering/using, though, only
    /// for path branches
    pub target2: *mut c_char,
    /// For multiple targets, not used for firing/triggering/using, though, only
    /// for path branches
    pub target3: *mut c_char,
    /// For multiple targets, not used for firing/triggering/using, though, only
    /// for path branches
    pub target4: *mut c_char,
    pub targetJump: *mut c_char,
    pub targetname: *mut c_char,
    pub team: *mut c_char,
    /// the roff file to use, if there is one / name of the external effect file
    pub uRoff: gentity_t_uRoff,
    /// current roff frame we are playing
    pub roff_ctr: c_int,
    pub next_roff_time: c_int,
    /// timer for beam in/out effects.
    pub fx_time: c_int,
    /// Used to determine if it's time to call e_ThinkFunc again
    pub nextthink: c_int,
    /// Called once every game frame for every ent
    pub e_ThinkFunc: c_int,
    /// Think func for equivalent centity
    pub e_clThinkFunc: c_int,
    /// movers call this when hitting endpoint
    pub e_ReachedFunc: c_int,
    pub e_BlockedFunc: c_int,
    pub e_TouchFunc: c_int,
    /// Called by G_UseTargets
    pub e_UseFunc: c_int,
    /// Called by G_Damage when damage is taken
    pub e_PainFunc: c_int,
    /// Called by G_Damage when health reaches <= 0
    pub e_DieFunc: c_int,
    pub health: c_int,
    pub max_health: c_int,
    pub takedamage: qboolean,
    pub material: material_t,
    pub damage: c_int,
    pub dflags: c_int,
    /// quad will increase this without increasing radius
    pub splashDamage: c_int,
    pub splashRadius: c_int,
    pub methodOfDeath: c_int,
    pub splashMethodOfDeath: c_int,
    /// Damage accumulated on different body locations
    pub locationDamage: [c_int; HL_MAX],
    pub chain: *mut gentity_t,
    pub enemy: *mut gentity_t,
    pub activator: *mut gentity_t,
    /// next entity in team
    pub teamchain: *mut gentity_t,
    /// master of the team
    pub teammaster: *mut gentity_t,
    pub lastEnemy: *mut gentity_t,
    pub wait: f32,
    pub random: f32,
    pub delay: c_int,
    pub alt_fire: qboolean,
    pub count: c_int,
    pub bounceCount: c_int,
    /// wind tunnel
    pub fly_sound_debounce_time: c_int,
    pub painDebounceTime: c_int,
    pub disconnectDebounceTime: c_int,
    pub attackDebounceTime: c_int,
    pub pushDebounceTime: c_int,
    pub aimDebounceTime: c_int,
    pub useDebounceTime: c_int,
    /// `trigger_formation` / `misc_dlight_active` / `has_bounced`
    pub uTriggerFormation: gentity_t_uTriggerFormation,
    /// store contents of ents on spawn so nav system can restore them
    pub spawnContents: c_int,
    /// Set once per frame, if you've moved, and if someone asks
    pub waypoint: c_int,
    /// Used by doors and breakable things to know what edge goes through them
    pub wayedge: c_int,
    /// To make sure you don't double-back
    pub lastWaypoint: c_int,
    pub lastInAirTime: c_int,
    /// Debouncer - so don't keep checking every waypoint in existance every
    /// frame that you can't find one
    pub noWaypointTime: c_int,
    pub combatPoint: c_int,
    pub followPos: vec3_t,
    pub followPosRecalcTime: c_int,
    pub followPosWaypoint: c_int,
    pub loopAnim: qboolean,
    pub startFrame: c_int,
    pub endFrame: c_int,
    pub m_iIcarusID: c_int,
    pub taskID: [c_int; NUM_TIDS],
    pub parms: *mut parms_t,
    pub behaviorSet: [*mut c_char; NUM_BSETS],
    pub script_targetname: *mut c_char,
    pub delayScriptTime: c_int,
    /// Only used for local sets
    pub soundSet: *mut c_char,
    pub setTime: c_int,
    pub cameraGroup: *mut c_char,
    pub noDamageTeam: team_t,
    pub playerModel: i16,
    pub weaponModel: [i16; MAX_INHAND_WEAPONS],
    pub handRBolt: i16,
    pub handLBolt: i16,
    pub headBolt: i16,
    pub cervicalBolt: i16,
    pub chestBolt: i16,
    pub gutBolt: i16,
    pub torsoBolt: i16,
    pub crotchBolt: i16,
    pub motionBolt: i16,
    pub kneeLBolt: i16,
    pub kneeRBolt: i16,
    pub elbowLBolt: i16,
    pub elbowRBolt: i16,
    pub footLBolt: i16,
    pub footRBolt: i16,
    pub faceBone: i16,
    pub craniumBone: i16,
    pub cervicalBone: i16,
    pub thoracicBone: i16,
    pub upperLumbarBone: i16,
    pub lowerLumbarBone: i16,
    pub hipsBone: i16,
    pub motionBone: i16,
    pub rootBone: i16,
    pub footLBone: i16,
    pub footRBone: i16,
    pub humerusRBone: i16,
    /// For bones special to an entity
    pub genericBone1: i16,
    pub genericBone2: i16,
    pub genericBone3: i16,
    /// For bolts special to an entity
    pub genericBolt1: i16,
    pub genericBolt2: i16,
    pub genericBolt3: i16,
    pub genericBolt4: i16,
    pub genericBolt5: i16,
    pub cinematicModel: qhandle_t,
    // Raven: `Vehicle_t *m_pVehicle` (g_shared.h:758). `Vehicle_t` lives in the
    // game tier (`sp_game`), which `sp_qshared` cannot depend on. `*mut c_void`
    // is ABI-identical to `*mut Vehicle_t` (both pointer-sized).
    //TODO: Port Vehicle_t (cross-tier: real struct lives in sp_game)
    // Source: oracle/oracle/code/game/G_Vehicles.h:133
    /// The vehicle object.
    pub m_pVehicle: *mut c_void,
    // Raven: `gNPC_t *NPC` (g_shared.h:762). `gNPC_t` lives in the game tier
    // (`sp_game`), which `sp_qshared` cannot depend on. `*mut c_void` is
    // ABI-identical to `*mut gNPC_t` (both pointer-sized).
    //TODO: Port gNPC_t (cross-tier: real struct lives in sp_game)
    // Source: oracle/oracle/code/game/b_public.h:146-313
    /// Only allocated if the entity becomes an NPC
    pub NPC: *mut c_void,
    /// Used by squadpaths to locate owning NPC
    pub ownername: *mut c_char,
    /// HACK - Makes them look for another enemy on the same team if the one
    /// they're after can't be hit
    pub cantHitEnemyCounter: c_int,
    pub NPC_type: *mut c_char,
    pub NPC_targetname: *mut c_char,
    pub NPC_target: *mut c_char,
    pub moverState: moverState_t,
    pub soundPos1: c_int,
    pub sound1to2: c_int,
    pub sound2to1: c_int,
    pub soundPos2: c_int,
    pub soundLoop: c_int,
    pub nextTrain: *mut gentity_t,
    pub prevTrain: *mut gentity_t,
    pub pos1: vec3_t,
    pub pos2: vec3_t,
    pub pos3: vec3_t,
    pub sounds: c_int,
    pub closetarget: *mut c_char,
    pub opentarget: *mut c_char,
    pub paintarget: *mut c_char,
    /// for maglocks- actually get put on the trigger for the door
    pub lockCount: c_int,
    pub radius: f32,
    /// `wpIndex` / `fxID`
    pub uWpIndex: gentity_t_uWpIndex,
    pub noise_index: c_int,
    pub startRGBA: vec4_t,
    /// `finalRGBA` / `pos4` / `modelAngles`
    pub uFinalRGBA: gentity_t_uFinalRGBA,
    /// for bonus items -
    pub item: *mut gitem_t,
    /// Used by triggers to print a message when activated
    pub message: *mut c_char,
    pub lightLevel: f32,
    pub forcePushTime: c_int,
    /// who force-pulled me (so we don't damage them if we hit them)
    pub forcePuller: c_int,
}

// Layout parity contract. `gentity_t` carries pointers, so its layout is
// arch-dependent; only `offset_of!(s) == 0` is arch-independent. The `*mut
// c_void` placeholders for `client`/`m_pVehicle`/`NPC` occupy the same 8 bytes
// as their real pointee pointers, so these offsets hold regardless of those
// types being ported.
// Source: `oracle/oracle/code/game/g_shared.h:514-825`
const _: () = assert!(core::mem::offset_of!(gentity_t, s) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<gentity_t>() == 1496);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, client) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, inuse) == 280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, linked) == 284);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, svFlags) == 288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, bmodel) == 292);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, mins) == 296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, maxs) == 308);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, contents) == 320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, absmin) == 324);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, absmax) == 336);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, currentOrigin) == 348);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, currentAngles) == 360);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, owner) == 376);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, ghoul2) == 384);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, modelScale) == 388);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, classname) == 400);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, spawnflags) == 408);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, flags) == 412);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, model) == 416);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, model2) == 424);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, freetime) == 432);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, eventTime) == 436);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, freeAfterEvent) == 440);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, physicsBounce) == 444);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, clipmask) == 448);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, speed) == 452);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, resultspeed) == 456);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, lastMoveTime) == 460);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, movedir) == 464);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, lastOrigin) == 476);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, lastAngles) == 488);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, mass) == 500);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, lastImpact) == 504);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, watertype) == 508);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, waterlevel) == 512);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, wupdate) == 516);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, prev_waterlevel) == 518);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, angle) == 520);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, target) == 528);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, target2) == 536);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, target3) == 544);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, target4) == 552);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, targetJump) == 560);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, targetname) == 568);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, team) == 576);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, uRoff) == 584);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, roff_ctr) == 592);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, next_roff_time) == 596);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, fx_time) == 600);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, nextthink) == 604);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, e_ThinkFunc) == 608);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, e_clThinkFunc) == 612);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, e_ReachedFunc) == 616);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, e_BlockedFunc) == 620);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, e_TouchFunc) == 624);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, e_UseFunc) == 628);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, e_PainFunc) == 632);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, e_DieFunc) == 636);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, health) == 640);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, max_health) == 644);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, takedamage) == 648);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, material) == 652);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, damage) == 656);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, dflags) == 660);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, splashDamage) == 664);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, splashRadius) == 668);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, methodOfDeath) == 672);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, splashMethodOfDeath) == 676);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, locationDamage) == 680);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, chain) == 776);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, enemy) == 784);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, activator) == 792);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, teamchain) == 800);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, teammaster) == 808);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, lastEnemy) == 816);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, wait) == 824);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, random) == 828);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, delay) == 832);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, alt_fire) == 836);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, count) == 840);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, bounceCount) == 844);
#[cfg(target_pointer_width = "64")]
const _: () =
    assert!(core::mem::offset_of!(gentity_t, fly_sound_debounce_time) == 848);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, painDebounceTime) == 852);
#[cfg(target_pointer_width = "64")]
const _: () =
    assert!(core::mem::offset_of!(gentity_t, disconnectDebounceTime) == 856);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, attackDebounceTime) == 860);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, pushDebounceTime) == 864);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, aimDebounceTime) == 868);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, useDebounceTime) == 872);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, uTriggerFormation) == 876);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, spawnContents) == 880);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, waypoint) == 884);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, wayedge) == 888);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, lastWaypoint) == 892);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, lastInAirTime) == 896);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, noWaypointTime) == 900);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, combatPoint) == 904);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, followPos) == 908);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, followPosRecalcTime) == 920);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, followPosWaypoint) == 924);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, loopAnim) == 928);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, startFrame) == 932);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, endFrame) == 936);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, m_iIcarusID) == 940);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, taskID) == 944);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, parms) == 984);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, behaviorSet) == 992);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, script_targetname) == 1128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, delayScriptTime) == 1136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, soundSet) == 1144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, setTime) == 1152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, cameraGroup) == 1160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, noDamageTeam) == 1168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, playerModel) == 1172);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, weaponModel) == 1174);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, handRBolt) == 1178);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, handLBolt) == 1180);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, headBolt) == 1182);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, cervicalBolt) == 1184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, chestBolt) == 1186);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, gutBolt) == 1188);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, torsoBolt) == 1190);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, crotchBolt) == 1192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, motionBolt) == 1194);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, kneeLBolt) == 1196);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, kneeRBolt) == 1198);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, elbowLBolt) == 1200);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, elbowRBolt) == 1202);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, footLBolt) == 1204);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, footRBolt) == 1206);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, faceBone) == 1208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, craniumBone) == 1210);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, cervicalBone) == 1212);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, thoracicBone) == 1214);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, upperLumbarBone) == 1216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, lowerLumbarBone) == 1218);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, hipsBone) == 1220);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, motionBone) == 1222);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, rootBone) == 1224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, footLBone) == 1226);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, footRBone) == 1228);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, humerusRBone) == 1230);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, genericBone1) == 1232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, genericBone2) == 1234);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, genericBone3) == 1236);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, genericBolt1) == 1238);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, genericBolt2) == 1240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, genericBolt3) == 1242);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, genericBolt4) == 1244);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, genericBolt5) == 1246);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, cinematicModel) == 1248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, m_pVehicle) == 1256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, NPC) == 1264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, ownername) == 1272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, cantHitEnemyCounter) == 1280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, NPC_type) == 1288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, NPC_targetname) == 1296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, NPC_target) == 1304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, moverState) == 1312);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, soundPos1) == 1316);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, sound1to2) == 1320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, sound2to1) == 1324);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, soundPos2) == 1328);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, soundLoop) == 1332);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, nextTrain) == 1336);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, prevTrain) == 1344);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, pos1) == 1352);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, pos2) == 1364);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, pos3) == 1376);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, sounds) == 1388);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, closetarget) == 1392);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, opentarget) == 1400);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, paintarget) == 1408);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, lockCount) == 1416);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, radius) == 1420);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, uWpIndex) == 1424);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, noise_index) == 1428);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, startRGBA) == 1432);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, uFinalRGBA) == 1448);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, item) == 1464);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, message) == 1472);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, lightLevel) == 1480);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, forcePushTime) == 1484);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, forcePuller) == 1488);
