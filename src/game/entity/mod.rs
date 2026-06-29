//! The server-side game entity: `gentity_s` / `gentity_t` (Raven `g_local.h`).
//!
//! Home of the engine-visible master struct `gentity_t` (`gentity_s`). Unlike the
//! q_shared.h networked structs, **this carries pointers** (raw `*mut`, function
//! pointers, `gentity_t*` links), so its layout is **arch-dependent** — the
//! `entityState_t s; playerState_t *playerState; …` prefix shifts every following
//! offset by the pointer-width delta on 64- vs 32-bit. The literal `size_of`/
//! `offset_of` asserts are therefore gated `#[cfg(target_pointer_width = "64")]`;
//! only `offset_of(s) == 0` is arch-independent. Mirrors upstream `codemp/game/g_local.h`.
//!
//! Migration target: `crate::modules::mp::game::entity`.
//! Source: `oracle/oracle/codemp/game/g_local.h:52`

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// TODO: NOT-PORTED
use crate::bg::gitem_t;
// TODO: NOT-PORTED
use crate::abi::{entityShared_t, parms_t, Vehicle_t, MAX_FAILED_NODES, NUM_BSETS, NUM_TIDS};
// TODO: NOT-PORTED
use crate::game::npc::gNPC_t;
// TODO: NOT-PORTED
use crate::shared::{entityState_t, material_t, playerState_t, qboolean, trace_t, vec3_t};
use core::ffi::{c_char, c_int, c_void};

use super::client::gclient_s;

/// `FL_*` game entity flags.
///
/// Raven: gentity->flags.
/// Source: `oracle/oracle/codemp/game/g_local.h:52`
pub const FL_GODMODE: c_int = 0x00000010;
pub const FL_NOTARGET: c_int = 0x00000020;
pub const FL_TEAMSLAVE: c_int = 0x00000400; // not the first on the team
pub const FL_NO_KNOCKBACK: c_int = 0x00000800;
pub const FL_DROPPED_ITEM: c_int = 0x00001000;
pub const FL_NO_BOTS: c_int = 0x00002000; // spawn point not for bot use
pub const FL_NO_HUMANS: c_int = 0x00004000; // spawn point just for bots
pub const FL_FORCE_GESTURE: c_int = 0x00008000; // force gesture on client
pub const FL_INACTIVE: c_int = 0x00010000; // inactive
pub const FL_NAVGOAL: c_int = 0x00020000; // for npc nav stuff
pub const FL_DONT_SHOOT: c_int = 0x00040000;
pub const FL_SHIELDED: c_int = 0x00080000;
pub const FL_UNDYING: c_int = 0x00100000; // takes damage down to 1, but never dies

//ex-eFlags -rww (note: FL_BOUNCE intentionally shares FL_UNDYING's value in the original)
pub const FL_BOUNCE: c_int = 0x00100000; // for missiles
pub const FL_BOUNCE_HALF: c_int = 0x00200000; // for missiles
pub const FL_BOUNCE_SHRAPNEL: c_int = 0x00400000; // special shrapnel flag

//vehicle game-local stuff -rww
pub const FL_VEH_BOARDING: c_int = 0x00800000; // special shrapnel flag

//breakable flags -rww
pub const FL_DMG_BY_SABER_ONLY: c_int = 0x01000000; //only take dmg from saber
pub const FL_DMG_BY_HEAVY_WEAP_ONLY: c_int = 0x02000000; //only take dmg from explosives

pub const FL_BBRUSH: c_int = 0x04000000; //I am a breakable brush

/// `moverState_t`.
///
/// Raven: movers are things like doors, plats, buttons, etc.
/// Source: `oracle/oracle/codemp/game/g_local.h:88`
pub type moverState_t = c_int;
pub const MOVER_POS1: moverState_t = 0;
pub const MOVER_POS2: moverState_t = 1;
pub const MOVER_1TO2: moverState_t = 2;
pub const MOVER_2TO1: moverState_t = 3;

/// Hit-location enum.
///
/// Raven: anonymous enum ending in `HL_MAX`; sizes `gentity_t::locationDamage`.
/// Source: `oracle/oracle/codemp/game/g_local.h:98`
pub const HL_NONE: c_int = 0;
pub const HL_FOOT_RT: c_int = 1;
pub const HL_FOOT_LT: c_int = 2;
pub const HL_LEG_RT: c_int = 3;
pub const HL_LEG_LT: c_int = 4;
pub const HL_WAIST: c_int = 5;
pub const HL_BACK_RT: c_int = 6;
pub const HL_BACK_LT: c_int = 7;
pub const HL_BACK: c_int = 8;
pub const HL_CHEST_RT: c_int = 9;
pub const HL_CHEST_LT: c_int = 10;
pub const HL_CHEST: c_int = 11;
pub const HL_ARM_RT: c_int = 12;
pub const HL_ARM_LT: c_int = 13;
pub const HL_HAND_RT: c_int = 14;
pub const HL_HAND_LT: c_int = 15;
pub const HL_HEAD: c_int = 16;
pub const HL_GENERIC1: c_int = 17;
pub const HL_GENERIC2: c_int = 18;
pub const HL_GENERIC3: c_int = 19;
pub const HL_GENERIC4: c_int = 20;
pub const HL_GENERIC5: c_int = 21;
pub const HL_GENERIC6: c_int = 22;
pub const HL_MAX: c_int = 23;

/// `gentity_s` / `gentity_t`.
///
/// Raven: rww - entstate must be first, to correspond with the bg shared entity structure.
/// Raven: From here up must be the same as centity_t/bgEntity_t.
/// Raven: DO NOT MODIFY ANYTHING ABOVE THIS, THE SERVER EXPECTS THE FIELDS IN THAT ORDER!
/// Source: `oracle/oracle/codemp/game/g_local.h:133`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gentity_s {
    //rww - entstate must be first, to correspond with the bg shared entity structure
    pub s: entityState_t,                // communicated by server to clients
    pub playerState: *mut playerState_t, //ptr to playerstate if applicable (for bg ents)
    pub m_pVehicle: *mut Vehicle_t,      //vehicle data
    pub ghoul2: *mut c_void,             //g2 instance
    pub localAnimIndex: c_int,           //index locally (game/cgame) to anim data for this skel
    pub modelScale: vec3_t,              //needed for g2 collision

    //From here up must be the same as centity_t/bgEntity_t
    pub r: entityShared_t, // shared by both the server system and game

    //rww - these are shared icarus things. They must be in this order as well in relation to the entityshared structure.
    pub taskID: [c_int; NUM_TIDS],
    pub parms: *mut parms_t,
    pub behaviorSet: [*mut c_char; NUM_BSETS],
    pub script_targetname: *mut c_char,
    pub delayScriptTime: c_int,
    pub fullName: *mut c_char,

    //rww - targetname and classname are now shared as well. ICARUS needs access to them.
    pub targetname: *mut c_char,
    pub classname: *mut c_char, // set in QuakeEd

    //rww - and yet more things to share. This is because the nav code is in the exe because it's all C++.
    pub waypoint: c_int, //Set once per frame, if you've moved, and if someone asks
    pub lastWaypoint: c_int, //To make sure you don't double-back
    pub lastValidWaypoint: c_int, //ALWAYS valid -used for tracking someone you lost
    pub noWaypointTime: c_int, //Debouncer - so don't keep checking every waypoint in existance every frame that you can't find one
    pub combatPoint: c_int,
    pub failedWaypoints: [c_int; MAX_FAILED_NODES],
    pub failedWaypointCheckTime: c_int,

    pub next_roff_time: c_int, //rww - npc's need to know when they're getting roff'd

    // DO NOT MODIFY ANYTHING ABOVE THIS, THE SERVER
    // EXPECTS THE FIELDS IN THAT ORDER!
    //================================
    pub client: *mut gclient_s, // NULL if not a client

    pub NPC: *mut gNPC_t,           //Only allocated if the entity becomes an NPC
    pub cantHitEnemyCounter: c_int, //HACK - Makes them look for another enemy on the same team if the one they're after can't be hit

    pub noLumbar: qboolean, //see note in cg_local.h

    pub inuse: qboolean,

    pub lockCount: c_int, //used by NPCs

    pub spawnflags: c_int, // set in QuakeEd

    pub teamnodmg: c_int, // damage will be ignored if it comes from this team

    pub roffname: *mut c_char,   // set in QuakeEd
    pub rofftarget: *mut c_char, // set in QuakeEd

    pub healingclass: *mut c_char, //set in quakeed
    pub healingsound: *mut c_char, //set in quakeed
    pub healingrate: c_int,        //set in quakeed
    pub healingDebounce: c_int,    //debounce for generic object healing shiz

    pub ownername: *mut c_char,

    pub objective: c_int,
    pub side: c_int,

    pub passThroughNum: c_int, // set to index to pass through (+1) for missiles

    pub aimDebounceTime: c_int,
    pub painDebounceTime: c_int,
    pub attackDebounceTime: c_int,
    pub alliedTeam: c_int, // only useable by this team, never target this team

    pub roffid: c_int, // if roffname != NULL then set on spawn

    pub neverFree: qboolean, // if true, FreeEntity will only unlink
    // bodyque uses this
    pub flags: c_int, // FL_* variables

    pub model: *mut c_char,
    pub model2: *mut c_char,
    pub freetime: c_int, // level.time when the object was freed

    pub eventTime: c_int, // events will be cleared EVENT_VALID_MSEC after set
    pub freeAfterEvent: qboolean,
    pub unlinkAfterEvent: qboolean,

    pub physicsObject: qboolean, // if true, it can be pushed by movers and fall off edges
    // all game items are physicsObjects,
    pub physicsBounce: f32, // 1.0 = continuous bounce, 0.0 = no bounce
    pub clipmask: c_int,    // brushes with this content value will be collided against
    // when moving.  items and corpses do not collide against
    // players, for instance

    //Only used by NPC_spawners
    pub NPC_type: *mut c_char,
    pub NPC_targetname: *mut c_char,
    pub NPC_target: *mut c_char,

    // movers
    pub moverState: moverState_t,
    pub soundPos1: c_int,
    pub sound1to2: c_int,
    pub sound2to1: c_int,
    pub soundPos2: c_int,
    pub soundLoop: c_int,
    pub parent: *mut gentity_s,
    pub nextTrain: *mut gentity_s,
    pub prevTrain: *mut gentity_s,
    pub pos1: vec3_t,
    pub pos2: vec3_t,

    //for npc's
    pub pos3: vec3_t,

    pub message: *mut c_char,

    pub timestamp: c_int, // body queue sinking, etc

    pub angle: f32, // set in editor, -1 = up, -2 = down
    pub target: *mut c_char,
    pub target2: *mut c_char,
    pub target3: *mut c_char, //For multiple targets, not used for firing/triggering/using, though, only for path branches
    pub target4: *mut c_char, //For multiple targets, not used for firing/triggering/using, though, only for path branches
    pub target5: *mut c_char, //mainly added for siege items
    pub target6: *mut c_char, //mainly added for siege items

    pub team: *mut c_char,
    pub targetShaderName: *mut c_char,
    pub targetShaderNewName: *mut c_char,
    pub target_ent: *mut gentity_s,

    pub closetarget: *mut c_char,
    pub opentarget: *mut c_char,
    pub paintarget: *mut c_char,

    pub goaltarget: *mut c_char,
    pub idealclass: *mut c_char,

    pub radius: f32,

    pub maxHealth: c_int, //used as a base for crosshair health display

    pub speed: f32,
    pub movedir: vec3_t,
    pub mass: f32,
    pub setTime: c_int,

    //Think Functions
    pub nextthink: c_int,
    pub think: Option<unsafe extern "C" fn(*mut gentity_s)>,
    pub reached: Option<unsafe extern "C" fn(*mut gentity_s)>, // movers call this when hitting endpoint
    pub blocked: Option<unsafe extern "C" fn(*mut gentity_s, *mut gentity_s)>,
    pub touch: Option<unsafe extern "C" fn(*mut gentity_s, *mut gentity_s, *mut trace_t)>,
    pub r#use: Option<unsafe extern "C" fn(*mut gentity_s, *mut gentity_s, *mut gentity_s)>,
    pub pain: Option<unsafe extern "C" fn(*mut gentity_s, *mut gentity_s, c_int)>,
    pub die:
        Option<unsafe extern "C" fn(*mut gentity_s, *mut gentity_s, *mut gentity_s, c_int, c_int)>,

    pub pain_debounce_time: c_int,
    pub fly_sound_debounce_time: c_int, // wind tunnel
    pub last_move_time: c_int,

    //Health and damage fields
    pub health: c_int,
    pub takedamage: qboolean,
    pub material: material_t,

    pub damage: c_int,
    pub dflags: c_int,
    pub splashDamage: c_int, // quad will increase this without increasing radius
    pub splashRadius: c_int,
    pub methodOfDeath: c_int,
    pub splashMethodOfDeath: c_int,

    pub locationDamage: [c_int; HL_MAX as usize], // Damage accumulated on different body locations

    pub count: c_int,
    pub bounceCount: c_int,
    pub alt_fire: qboolean,

    pub chain: *mut gentity_s,
    pub enemy: *mut gentity_s,
    pub lastEnemy: *mut gentity_s,
    pub activator: *mut gentity_s,
    pub teamchain: *mut gentity_s,  // next entity in team
    pub teammaster: *mut gentity_s, // master of the team

    pub watertype: c_int,
    pub waterlevel: c_int,

    pub noise_index: c_int,

    // timing variables
    pub wait: f32,
    pub random: f32,
    pub delay: c_int,

    //generic values used by various entities for different purposes.
    pub genericValue1: c_int,
    pub genericValue2: c_int,
    pub genericValue3: c_int,
    pub genericValue4: c_int,
    pub genericValue5: c_int,
    pub genericValue6: c_int,
    pub genericValue7: c_int,
    pub genericValue8: c_int,
    pub genericValue9: c_int,
    pub genericValue10: c_int,
    pub genericValue11: c_int,
    pub genericValue12: c_int,
    pub genericValue13: c_int,
    pub genericValue14: c_int,
    pub genericValue15: c_int,

    pub soundSet: *mut c_char,

    pub isSaberEntity: qboolean,

    pub damageRedirect: c_int,   //if entity takes damage, redirect to..
    pub damageRedirectTo: c_int, //this entity number

    pub epVelocity: vec3_t,
    pub epGravFactor: f32,

    pub item: *mut gitem_t, // for bonus items
}

/// `gentity_t`.
///
/// Raven: `typedef struct gentity_s gentity_t`.
/// Source: `oracle/oracle/codemp/game/g_local.h:16`
pub type gentity_t = gentity_s;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<gentity_t>() == 1832);
const _: () = assert!(core::mem::offset_of!(gentity_t, s) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, r) == 576);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, taskID) == 688);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, client) == 976);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, moverState) == 1176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, think) == 1440);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, material) == 1516);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, locationDamage) == 1544);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, item) == 1824);

pub const DAMAGEREDIRECT_HEAD: c_int = 1;
pub const DAMAGEREDIRECT_RLEG: c_int = 2;
pub const DAMAGEREDIRECT_LLEG: c_int = 3;
