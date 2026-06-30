//! MP `gentity_t` copied from Raven `codemp/game/g_local.h`.
//!
//! Type declaration source: `oracle/oracle/codemp/game/g_local.h:16`
//! Full struct layout source: `oracle/oracle/codemp/game/g_local.h:133-359`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use crate::common::mp::qcommon::{entityState_t, gitem_t, parms_t, playerState_t};
use crate::common::mp::trace_t::trace_t;
use crate::shared::{entityShared_t, qboolean, vec3_t};

/// Raven MP `NUM_TIDS`.
///
/// Definition source: `oracle/oracle/codemp/game/g_public.h:623-638`
pub const NUM_TIDS: usize = 10;

/// Raven MP `NUM_BSETS`.
///
/// Definition source: `oracle/oracle/codemp/game/g_public.h:642-663`
pub const NUM_BSETS: usize = 17;

/// Raven MP `MAX_FAILED_NODES`.
///
/// Definition source: `oracle/oracle/codemp/game/g_public.h:673`
pub const MAX_FAILED_NODES: usize = 8;

/// Raven MP `HL_MAX`.
///
/// Definition source: `oracle/oracle/codemp/game/g_local.h:99-123`
pub const HL_MAX: usize = 23;

/// Raven MP `moverState_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/g_local.h:89-94`
pub type moverState_t = c_int;

/// Raven MP `material_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:990`
pub type material_t = c_int;

/// Raven MP `gentity_t`.
///
/// Type declaration source: `oracle/oracle/codemp/game/g_local.h:16`
/// Full struct layout source: `oracle/oracle/codemp/game/g_local.h:133-359`
#[repr(C)]
#[derive(Debug)]
pub struct gentity_t {
    /// Entstate must be first, to correspond with the bg shared entity structure.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:135`
    pub s: entityState_t,
    /// Ptr to playerstate if applicable (for bg ents).
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:136`
    pub playerState: *mut playerState_t,
    // FIXME: create type `Vehicle_t`.
    // Raven field source: `oracle/oracle/codemp/game/g_local.h:137`
    // pub m_pVehicle: *mut Vehicle_t,
    /// Placeholder for `Vehicle_t *m_pVehicle` until `Vehicle_t` is ported.
    pub m_pVehicle: *mut c_void,
    /// G2 instance.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:138`
    pub ghoul2: *mut c_void,
    /// Index locally (game/cgame) to anim data for this skel.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:139`
    pub localAnimIndex: c_int,
    /// Needed for g2 collision.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:140`
    pub modelScale: vec3_t,
    /// From here up must be the same as centity_t/bgEntity_t.
    ///
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:144`
    pub r: entityShared_t,
    /// ICARUS task IDs.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:147`
    pub taskID: [c_int; NUM_TIDS],
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:148`
    pub parms: *mut parms_t,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:149`
    pub behaviorSet: [*mut c_char; NUM_BSETS],
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:150`
    pub script_targetname: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:151`
    pub delayScriptTime: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:152`
    pub fullName: *mut c_char,
    /// ICARUS needs access to targetname.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:155`
    pub targetname: *mut c_char,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:156`
    pub classname: *mut c_char,
    /// Set once per frame, if you've moved, and if someone asks.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:159`
    pub waypoint: c_int,
    /// To make sure you don't double-back.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:160`
    pub lastWaypoint: c_int,
    /// ALWAYS valid - used for tracking someone you lost.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:161`
    pub lastValidWaypoint: c_int,
    /// Debouncer - so don't keep checking every waypoint in existance every frame that you can't find one.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:162`
    pub noWaypointTime: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:163`
    pub combatPoint: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:164`
    pub failedWaypoints: [c_int; MAX_FAILED_NODES],
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:165`
    pub failedWaypointCheckTime: c_int,
    /// NPCs need to know when they're getting roff'd.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:167`
    pub next_roff_time: c_int,
    /// DO NOT MODIFY ANYTHING ABOVE THIS, THE SERVER EXPECTS THE FIELDS IN THAT ORDER.
    ///
    // FIXME: create type `gclient_s`.
    // Raven field source: `oracle/oracle/codemp/game/g_local.h:173`
    // pub client: *mut gclient_s,
    /// Placeholder for `struct gclient_s *client` until `gclient_s` is ported.
    pub client: *mut c_void,
    // FIXME: create type `gNPC_t`.
    // Raven field source: `oracle/oracle/codemp/game/g_local.h:175`
    // pub NPC: *mut gNPC_t,
    /// Placeholder for `gNPC_t *NPC` until `gNPC_t` is ported.
    pub NPC: *mut c_void,
    /// Makes them look for another enemy on the same team if the one they're after can't be hit.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:176`
    pub cantHitEnemyCounter: c_int,
    /// See note in cg_local.h.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:178`
    pub noLumbar: qboolean,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:180`
    pub inuse: qboolean,
    /// Used by NPCs.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:182`
    pub lockCount: c_int,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:184`
    pub spawnflags: c_int,
    /// Damage will be ignored if it comes from this team.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:186`
    pub teamnodmg: c_int,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:188`
    pub roffname: *mut c_char,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:189`
    pub rofftarget: *mut c_char,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:191`
    pub healingclass: *mut c_char,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:192`
    pub healingsound: *mut c_char,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:193`
    pub healingrate: c_int,
    /// Debounce for generic object healing.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:194`
    pub healingDebounce: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:196`
    pub ownername: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:198`
    pub objective: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:199`
    pub side: c_int,
    /// Set to index to pass through (+1) for missiles.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:201`
    pub passThroughNum: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:203`
    pub aimDebounceTime: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:204`
    pub painDebounceTime: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:205`
    pub attackDebounceTime: c_int,
    /// Only useable by this team, never target this team.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:206`
    pub alliedTeam: c_int,
    /// If roffname != NULL then set on spawn.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:208`
    pub roffid: c_int,
    /// If true, FreeEntity will only unlink.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:210`
    pub neverFree: qboolean,
    /// FL_* variables.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:213`
    pub flags: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:215`
    pub model: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:216`
    pub model2: *mut c_char,
    /// Level.time when the object was freed.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:217`
    pub freetime: c_int,
    /// Events will be cleared EVENT_VALID_MSEC after set.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:219`
    pub eventTime: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:220`
    pub freeAfterEvent: qboolean,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:221`
    pub unlinkAfterEvent: qboolean,
    /// If true, it can be pushed by movers and fall off edges.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:223`
    pub physicsObject: qboolean,
    /// 1.0 = continuous bounce, 0.0 = no bounce.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:225`
    pub physicsBounce: f32,
    /// Brushes with this content value will be collided against when moving.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:226`
    pub clipmask: c_int,
    /// Only used by NPC_spawners.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:230`
    pub NPC_type: *mut c_char,
    /// Only used by NPC_spawners.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:231`
    pub NPC_targetname: *mut c_char,
    /// Only used by NPC_spawners.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:232`
    pub NPC_target: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:235`
    pub moverState: moverState_t,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:236`
    pub soundPos1: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:237`
    pub sound1to2: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:238`
    pub sound2to1: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:239`
    pub soundPos2: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:240`
    pub soundLoop: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:241`
    pub parent: *mut gentity_t,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:242`
    pub nextTrain: *mut gentity_t,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:243`
    pub prevTrain: *mut gentity_t,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:244`
    pub pos1: vec3_t,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:244`
    pub pos2: vec3_t,
    /// For NPCs.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:247`
    pub pos3: vec3_t,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:249`
    pub message: *mut c_char,
    /// Body queue sinking, etc.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:251`
    pub timestamp: c_int,
    /// Set in editor, -1 = up, -2 = down.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:253`
    pub angle: f32,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:254`
    pub target: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:255`
    pub target2: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:256`
    pub target3: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:257`
    pub target4: *mut c_char,
    /// Mainly added for siege items.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:258`
    pub target5: *mut c_char,
    /// Mainly added for siege items.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:259`
    pub target6: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:261`
    pub team: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:262`
    pub targetShaderName: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:263`
    pub targetShaderNewName: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:264`
    pub target_ent: *mut gentity_t,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:266`
    pub closetarget: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:267`
    pub opentarget: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:268`
    pub paintarget: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:270`
    pub goaltarget: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:271`
    pub idealclass: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:273`
    pub radius: f32,
    /// Used as a base for crosshair health display.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:275`
    pub maxHealth: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:277`
    pub speed: f32,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:278`
    pub movedir: vec3_t,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:279`
    pub mass: f32,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:280`
    pub setTime: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:283`
    pub nextthink: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:284`
    pub think: Option<unsafe extern "C" fn(self_: *mut gentity_t)>,
    /// Movers call this when hitting endpoint.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:285`
    pub reached: Option<unsafe extern "C" fn(self_: *mut gentity_t)>,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:286`
    pub blocked: Option<unsafe extern "C" fn(self_: *mut gentity_t, other: *mut gentity_t)>,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:287`
    pub touch: Option<
        unsafe extern "C" fn(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t),
    >,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:288`
    pub use_: Option<
        unsafe extern "C" fn(
            self_: *mut gentity_t,
            other: *mut gentity_t,
            activator: *mut gentity_t,
        ),
    >,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:289`
    pub pain: Option<
        unsafe extern "C" fn(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int),
    >,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:290`
    pub die: Option<
        unsafe extern "C" fn(
            self_: *mut gentity_t,
            inflictor: *mut gentity_t,
            attacker: *mut gentity_t,
            damage: c_int,
            mod_: c_int,
        ),
    >,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:292`
    pub pain_debounce_time: c_int,
    /// Wind tunnel.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:293`
    pub fly_sound_debounce_time: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:294`
    pub last_move_time: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:297`
    pub health: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:298`
    pub takedamage: qboolean,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:299`
    pub material: material_t,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:301`
    pub damage: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:302`
    pub dflags: c_int,
    /// Quad will increase this without increasing radius.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:303`
    pub splashDamage: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:304`
    pub splashRadius: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:305`
    pub methodOfDeath: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:306`
    pub splashMethodOfDeath: c_int,
    /// Damage accumulated on different body locations.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:308`
    pub locationDamage: [c_int; HL_MAX],
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:310`
    pub count: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:311`
    pub bounceCount: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:312`
    pub alt_fire: qboolean,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:314`
    pub chain: *mut gentity_t,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:315`
    pub enemy: *mut gentity_t,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:316`
    pub lastEnemy: *mut gentity_t,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:317`
    pub activator: *mut gentity_t,
    /// Next entity in team.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:318`
    pub teamchain: *mut gentity_t,
    /// Master of the team.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:319`
    pub teammaster: *mut gentity_t,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:321`
    pub watertype: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:322`
    pub waterlevel: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:324`
    pub noise_index: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:327`
    pub wait: f32,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:328`
    pub random: f32,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:329`
    pub delay: c_int,
    /// Generic values used by various entities for different purposes.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:332`
    pub genericValue1: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:333`
    pub genericValue2: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:334`
    pub genericValue3: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:335`
    pub genericValue4: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:336`
    pub genericValue5: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:337`
    pub genericValue6: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:338`
    pub genericValue7: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:339`
    pub genericValue8: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:340`
    pub genericValue9: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:341`
    pub genericValue10: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:342`
    pub genericValue11: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:343`
    pub genericValue12: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:344`
    pub genericValue13: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:345`
    pub genericValue14: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:346`
    pub genericValue15: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:348`
    pub soundSet: *mut c_char,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:350`
    pub isSaberEntity: qboolean,
    /// If entity takes damage, redirect to...
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:352`
    pub damageRedirect: c_int,
    /// This entity number.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:353`
    pub damageRedirectTo: c_int,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:355`
    pub epVelocity: vec3_t,
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:356`
    pub epGravFactor: f32,
    /// For bonus items.
    /// Raven field source: `oracle/oracle/codemp/game/g_local.h:358`
    pub item: *mut gitem_t,
}
