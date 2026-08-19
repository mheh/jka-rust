//! MP `gentity_t` copied from Raven `codemp/game/g_local.h`.
//!
//! Type declaration source: `oracle/codemp/game/g_local.h:16`
//! Full struct layout source: `oracle/codemp/game/g_local.h:133-359`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_void, CStr};

use mp_bg::cstr_util::{cstr, cstr_to_str};
use mp_bg::public::item_id::ItemId;
use mp_bg::vehicles::vehicle_s::Vehicle_t;
use mp_qshared::common::mp::ent_fn_ids::{
    EntBlocked, EntDie, EntPain, EntReached, EntThink, EntTouch, EntUse, FnId,
};
use mp_qshared::common::mp::entity_id::EntityId;
use mp_qshared::common::mp::gentity::{
    material_t, moverState_t, HL_MAX, MAX_FAILED_NODES, NUM_BSETS, NUM_TIDS,
};
use mp_qshared::common::mp::qcommon::{entityState_t, parms_t, playerState_t};
use mp_qshared::shared::{entityShared_t, qboolean, vec3_t};

use crate::client::gclient::gclient_t;
use crate::g_spawn::translate_newlines;
use crate::npc::g_npc_t::gNPC_t;
use crate::world::game_context::GameContext;

/// Raven MP `gentity_t`.
///
/// Type declaration source: `oracle/codemp/game/g_local.h:16`
/// Full struct layout source: `oracle/codemp/game/g_local.h:133-359`
#[repr(C)]
#[derive(Debug)]
pub struct gentity_t {
    /// Entstate must be first, to correspond with the bg shared entity structure.
    /// Raven field source: `oracle/codemp/game/g_local.h:135`
    pub s: entityState_t,
    /// Ptr to playerstate if applicable (for bg ents).
    /// Raven field source: `oracle/codemp/game/g_local.h:136`
    pub playerState: *mut playerState_t,
    /// DEC-26 restored the real type, because `gentity_t` now lives in `mp_game`.
    /// Raven field source: `oracle/codemp/game/bg_vehicles.h:477` (via `g_local.h:137`)
    pub m_pVehicle: *mut Vehicle_t,
    /// G2 instance.
    /// Raven field source: `oracle/codemp/game/g_local.h:138`
    pub ghoul2: *mut c_void,
    /// Index locally (game/cgame) to anim data for this skel.
    /// Raven field source: `oracle/codemp/game/g_local.h:139`
    pub localAnimIndex: c_int,
    /// Needed for g2 collision.
    /// Raven field source: `oracle/codemp/game/g_local.h:140`
    pub modelScale: vec3_t,
    /// From here up must be the same as centity_t/bgEntity_t.
    ///
    /// Raven field source: `oracle/codemp/game/g_local.h:144`
    pub r: entityShared_t,
    /// ICARUS task IDs.
    /// Raven field source: `oracle/codemp/game/g_local.h:147`
    pub taskID: [c_int; NUM_TIDS],
    /// Raven field source: `oracle/codemp/game/g_local.h:148`
    pub parms: *mut parms_t,
    /// Raven field source: `oracle/codemp/game/g_local.h:149`
    pub behaviorSet: [*mut c_char; NUM_BSETS],
    /// Raven field source: `oracle/codemp/game/g_local.h:150`
    pub script_targetname: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_local.h:151`
    pub delayScriptTime: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:152`
    pub fullName: *mut c_char,
    /// ICARUS needs access to targetname.
    /// Raven field source: `oracle/codemp/game/g_local.h:155`
    pub targetname: *mut c_char,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/codemp/game/g_local.h:156`
    pub classname: *mut c_char,
    /// Set once per frame, if you've moved, and if someone asks.
    /// Raven field source: `oracle/codemp/game/g_local.h:159`
    pub waypoint: c_int,
    /// To make sure you don't double-back.
    /// Raven field source: `oracle/codemp/game/g_local.h:160`
    pub lastWaypoint: c_int,
    /// ALWAYS valid - used for tracking someone you lost.
    /// Raven field source: `oracle/codemp/game/g_local.h:161`
    pub lastValidWaypoint: c_int,
    /// Debouncer - so don't keep checking every waypoint in existance every frame that you can't find one.
    /// Raven field source: `oracle/codemp/game/g_local.h:162`
    pub noWaypointTime: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:163`
    pub combatPoint: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:164`
    pub failedWaypoints: [c_int; MAX_FAILED_NODES],
    /// Raven field source: `oracle/codemp/game/g_local.h:165`
    pub failedWaypointCheckTime: c_int,
    /// NPCs need to know when they're getting roff'd.
    /// Raven field source: `oracle/codemp/game/g_local.h:167`
    pub next_roff_time: c_int,
    /// DO NOT MODIFY ANYTHING ABOVE THIS, THE SERVER EXPECTS THE FIELDS IN THAT ORDER.
    ///
    /// DEC-26 restored the real type, because `gentity_t` now lives in `mp_game`.
    /// Raven field source: `oracle/codemp/game/g_local.h:173`
    pub client: *mut gclient_t,
    /// DEC-26 restored the real type, because `gentity_t` now lives in `mp_game`.
    /// Raven field source: `oracle/codemp/game/g_local.h:175` (`b_public.h`)
    pub NPC: *mut gNPC_t,
    /// Makes them look for another enemy on the same team if the one they're after can't be hit.
    /// Raven field source: `oracle/codemp/game/g_local.h:176`
    pub cantHitEnemyCounter: c_int,
    /// See note in cg_local.h.
    /// Raven field source: `oracle/codemp/game/g_local.h:178`
    pub noLumbar: qboolean,
    /// Raven field source: `oracle/codemp/game/g_local.h:180`
    pub inuse: qboolean,
    /// Used by NPCs.
    /// Raven field source: `oracle/codemp/game/g_local.h:182`
    pub lockCount: c_int,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/codemp/game/g_local.h:184`
    pub spawnflags: c_int,
    /// Damage will be ignored if it comes from this team.
    /// Raven field source: `oracle/codemp/game/g_local.h:186`
    pub teamnodmg: c_int,
    // Raven's `roffname` and `rofftarget` (`g_local.h:188-189`) are gone.
    // `rofftarget` had zero readers repo-wide.
    // `roffname` only ever fed the resolved `roffid` int.
    // So both spawn keys became `F_IGNORE`, parsed and silently discarded, and the ICARUS `roffname` store dropped.
    // This is a private-tail change with no ABI impact.
    /// Owned copy of the QuakeEd healing-class name. `""` marks it absent.
    /// Raven's `char *` distinguished presence only by `!= 0`, never by NULL versus empty.
    /// Raven field source: `oracle/codemp/game/g_local.h:191`
    pub healingclass: String,
    /// Owned copy of the QuakeEd healing sound name. `""` marks it absent.
    /// Raven field source: `oracle/codemp/game/g_local.h:192`
    pub healingsound: String,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/codemp/game/g_local.h:193`
    pub healingrate: c_int,
    /// Debounce for generic object healing.
    /// Raven field source: `oracle/codemp/game/g_local.h:194`
    pub healingDebounce: c_int,
    /// Owned copy of the QuakeEd owner-tag name. `""` marks it absent.
    /// Raven field source: `oracle/codemp/game/g_local.h:196`
    pub ownername: String,
    /// Raven field source: `oracle/codemp/game/g_local.h:198`
    pub objective: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:199`
    pub side: c_int,
    /// Set to index to pass through (+1) for missiles.
    /// Raven field source: `oracle/codemp/game/g_local.h:201`
    pub passThroughNum: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:203`
    pub aimDebounceTime: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:204`
    pub painDebounceTime: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:205`
    pub attackDebounceTime: c_int,
    /// Only useable by this team, never target this team.
    /// Raven field source: `oracle/codemp/game/g_local.h:206`
    pub alliedTeam: c_int,
    /// If roffname != NULL then set on spawn.
    /// Raven field source: `oracle/codemp/game/g_local.h:208`
    pub roffid: c_int,
    /// If true, FreeEntity will only unlink.
    /// Raven field source: `oracle/codemp/game/g_local.h:210`
    pub neverFree: qboolean,
    /// FL_* variables.
    /// Raven field source: `oracle/codemp/game/g_local.h:213`
    pub flags: c_int,
    /// Owned copy of the QuakeEd model name. `None` maps to Raven NULL.
    /// Readers distinguish an unset model from an empty one, for example the brush-model guard in `g_mover`.
    /// Raven field source: `oracle/codemp/game/g_local.h:215`
    pub model: Option<String>,
    /// Owned copy of the QuakeEd secondary model name. `""` marks it absent.
    /// Raven field source: `oracle/codemp/game/g_local.h:216`
    pub model2: String,
    /// Level.time when the object was freed.
    /// Raven field source: `oracle/codemp/game/g_local.h:217`
    pub freetime: c_int,
    /// Events will be cleared EVENT_VALID_MSEC after set.
    /// Raven field source: `oracle/codemp/game/g_local.h:219`
    pub eventTime: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:220`
    pub freeAfterEvent: qboolean,
    /// Raven field source: `oracle/codemp/game/g_local.h:221`
    pub unlinkAfterEvent: qboolean,
    /// If true, it can be pushed by movers and fall off edges.
    /// Raven field source: `oracle/codemp/game/g_local.h:223`
    pub physicsObject: qboolean,
    /// 1.0 = continuous bounce, 0.0 = no bounce.
    /// Raven field source: `oracle/codemp/game/g_local.h:225`
    pub physicsBounce: f32,
    /// Brushes with this content value will be collided against when moving.
    /// Raven field source: `oracle/codemp/game/g_local.h:226`
    pub clipmask: c_int,
    /// Owned NPC species name for NPC_spawners. `None` maps to Raven NULL.
    /// The code distinguishes a never-set spawner from one whose type resolved to `""`.
    /// Only used by NPC_spawners.
    /// Raven field source: `oracle/codemp/game/g_local.h:230`
    pub NPC_type: Option<String>,
    /// Owned copy of the NPC_spawner's target name. `""` marks it absent.
    /// Only used by NPC_spawners.
    /// Raven field source: `oracle/codemp/game/g_local.h:231`
    pub NPC_targetname: String,
    /// Owned copy of the NPC_spawner's target name. `""` marks it absent.
    /// Only used by NPC_spawners.
    /// Raven field source: `oracle/codemp/game/g_local.h:232`
    pub NPC_target: String,
    /// Raven field source: `oracle/codemp/game/g_local.h:235`
    pub moverState: moverState_t,
    /// Raven field source: `oracle/codemp/game/g_local.h:236`
    pub soundPos1: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:237`
    pub sound1to2: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:238`
    pub sound2to1: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:239`
    pub soundPos2: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:240`
    pub soundLoop: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:241`
    pub parent: Option<EntityId>,
    /// Raven field source: `oracle/codemp/game/g_local.h:242`
    pub nextTrain: Option<EntityId>,
    /// Raven field source: `oracle/codemp/game/g_local.h:243`
    pub prevTrain: Option<EntityId>,
    /// Raven field source: `oracle/codemp/game/g_local.h:244`
    pub pos1: vec3_t,
    /// Raven field source: `oracle/codemp/game/g_local.h:244`
    pub pos2: vec3_t,
    /// For NPCs.
    /// Raven field source: `oracle/codemp/game/g_local.h:247`
    pub pos3: vec3_t,
    /// Owned copy of the QuakeEd message text. Writing it translates `\n` escapes to real linefeeds.
    /// `None` maps to Raven NULL. Readers distinguish an unset message from an empty one.
    /// Raven field source: `oracle/codemp/game/g_local.h:249`
    pub message: Option<String>,
    /// Body queue sinking, etc.
    /// Raven field source: `oracle/codemp/game/g_local.h:251`
    pub timestamp: c_int,
    /// Set in editor, -1 = up, -2 = down.
    /// Raven field source: `oracle/codemp/game/g_local.h:253`
    pub angle: f32,
    /// Owned copy of the QuakeEd target name. `None` maps to Raven NULL.
    /// Readers distinguish an unset target from an empty one.
    /// Raven field source: `oracle/codemp/game/g_local.h:254`
    pub target: Option<String>,
    /// Owned copy of the QuakeEd secondary target name. `None` maps to Raven NULL.
    /// Raven field source: `oracle/codemp/game/g_local.h:255`
    pub target2: Option<String>,
    /// Owned copy of the QuakeEd tertiary target name. `""` marks it absent.
    /// Raven field source: `oracle/codemp/game/g_local.h:256`
    pub target3: String,
    /// Owned copy of the QuakeEd quaternary target name. `""` marks it absent.
    /// Raven field source: `oracle/codemp/game/g_local.h:257`
    pub target4: String,
    /// Owned copy of the siege-item target name. `""` marks it absent.
    /// Mainly added for siege items.
    /// Raven field source: `oracle/codemp/game/g_local.h:258`
    pub target5: String,
    /// Owned copy of the siege-item target name. `""` marks it absent.
    /// Mainly added for siege items.
    /// Raven field source: `oracle/codemp/game/g_local.h:259`
    pub target6: String,
    /// Owned copy of the QuakeEd team tag. `None` maps to Raven NULL.
    /// Readers distinguish an unset team from an empty one.
    /// Raven field source: `oracle/codemp/game/g_local.h:261`
    pub team: Option<String>,
    /// Owned copy of the shader-remap source name. `""` marks it absent.
    /// Raven field source: `oracle/codemp/game/g_local.h:262`
    pub targetShaderName: String,
    /// Owned copy of the shader-remap destination name. `""` marks it absent.
    /// Raven field source: `oracle/codemp/game/g_local.h:263`
    pub targetShaderNewName: String,
    /// Raven field source: `oracle/codemp/game/g_local.h:264`
    pub target_ent: Option<EntityId>,
    /// Owned copy of the door close-target name. `None` maps to Raven NULL.
    /// Readers null-check it before use.
    /// Raven field source: `oracle/codemp/game/g_local.h:266`
    pub closetarget: Option<String>,
    /// Owned copy of the door open-target name. `None` maps to Raven NULL.
    /// Raven field source: `oracle/codemp/game/g_local.h:267`
    pub opentarget: Option<String>,
    /// Owned copy of the pain-target name. `None` maps to Raven NULL.
    /// Raven field source: `oracle/codemp/game/g_local.h:268`
    pub paintarget: Option<String>,
    /// Owned copy of the siege goal-target name. `""` marks it absent.
    /// Raven field source: `oracle/codemp/game/g_local.h:270`
    pub goaltarget: String,
    /// Owned copy of the siege ideal-class name. `""` marks it absent.
    /// Raven field source: `oracle/codemp/game/g_local.h:271`
    pub idealclass: String,
    /// Raven field source: `oracle/codemp/game/g_local.h:273`
    pub radius: f32,
    /// Used as a base for crosshair health display.
    /// Raven field source: `oracle/codemp/game/g_local.h:275`
    pub maxHealth: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:277`
    pub speed: f32,
    /// Raven field source: `oracle/codemp/game/g_local.h:278`
    pub movedir: vec3_t,
    /// Raven field source: `oracle/codemp/game/g_local.h:279`
    pub mass: f32,
    /// Raven field source: `oracle/codemp/game/g_local.h:280`
    pub setTime: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:283`
    pub nextthink: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:284`
    pub think: FnId<EntThink>,
    /// Movers call this when hitting endpoint.
    /// Raven field source: `oracle/codemp/game/g_local.h:285`
    pub reached: FnId<EntReached>,
    /// Raven field source: `oracle/codemp/game/g_local.h:286`
    pub blocked: FnId<EntBlocked>,
    /// Raven field source: `oracle/codemp/game/g_local.h:287`
    pub touch: FnId<EntTouch>,
    /// Raven field source: `oracle/codemp/game/g_local.h:288`
    pub use_: FnId<EntUse>,
    /// Raven field source: `oracle/codemp/game/g_local.h:289`
    pub pain: FnId<EntPain>,
    /// Raven field source: `oracle/codemp/game/g_local.h:290`
    pub die: FnId<EntDie>,
    /// Raven field source: `oracle/codemp/game/g_local.h:292`
    pub pain_debounce_time: c_int,
    /// Wind tunnel.
    /// Raven field source: `oracle/codemp/game/g_local.h:293`
    pub fly_sound_debounce_time: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:294`
    pub last_move_time: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:297`
    pub health: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:298`
    pub takedamage: qboolean,
    /// Raven field source: `oracle/codemp/game/g_local.h:299`
    pub material: material_t,
    /// Raven field source: `oracle/codemp/game/g_local.h:301`
    pub damage: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:302`
    pub dflags: c_int,
    /// Quad will increase this without increasing radius.
    /// Raven field source: `oracle/codemp/game/g_local.h:303`
    pub splashDamage: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:304`
    pub splashRadius: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:305`
    pub methodOfDeath: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:306`
    pub splashMethodOfDeath: c_int,
    /// Damage accumulated on different body locations.
    /// Raven field source: `oracle/codemp/game/g_local.h:308`
    pub locationDamage: [c_int; HL_MAX],
    /// Raven field source: `oracle/codemp/game/g_local.h:310`
    pub count: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:311`
    pub bounceCount: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:312`
    pub alt_fire: qboolean,
    /// Raven field source: `oracle/codemp/game/g_local.h:314`
    pub chain: Option<EntityId>,
    /// Raven field source: `oracle/codemp/game/g_local.h:315`
    pub enemy: Option<EntityId>,
    /// Raven field source: `oracle/codemp/game/g_local.h:316`
    pub lastEnemy: Option<EntityId>,
    /// Raven field source: `oracle/codemp/game/g_local.h:317`
    pub activator: Option<EntityId>,
    /// Next entity in team.
    /// Raven field source: `oracle/codemp/game/g_local.h:318`
    pub teamchain: Option<EntityId>,
    /// Master of the team.
    /// Raven field source: `oracle/codemp/game/g_local.h:319`
    pub teammaster: Option<EntityId>,
    /// Raven field source: `oracle/codemp/game/g_local.h:321`
    pub watertype: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:322`
    pub waterlevel: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:324`
    pub noise_index: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:327`
    pub wait: f32,
    /// Raven field source: `oracle/codemp/game/g_local.h:328`
    pub random: f32,
    /// Raven field source: `oracle/codemp/game/g_local.h:329`
    pub delay: c_int,
    /// Generic values used by various entities for different purposes.
    /// Raven field source: `oracle/codemp/game/g_local.h:332`
    pub genericValue1: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:333`
    pub genericValue2: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:334`
    pub genericValue3: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:335`
    pub genericValue4: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:336`
    pub genericValue5: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:337`
    pub genericValue6: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:338`
    pub genericValue7: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:339`
    pub genericValue8: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:340`
    pub genericValue9: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:341`
    pub genericValue10: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:342`
    pub genericValue11: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:343`
    pub genericValue12: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:344`
    pub genericValue13: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:345`
    pub genericValue14: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:346`
    pub genericValue15: c_int,
    /// Owned copy of the QuakeEd sound-set name. `""` marks it absent.
    /// Raven field source: `oracle/codemp/game/g_local.h:348`
    pub soundSet: String,
    /// Raven field source: `oracle/codemp/game/g_local.h:350`
    pub isSaberEntity: qboolean,
    /// If entity takes damage, redirect to...
    /// Raven field source: `oracle/codemp/game/g_local.h:352`
    pub damageRedirect: c_int,
    /// This entity number.
    /// Raven field source: `oracle/codemp/game/g_local.h:353`
    pub damageRedirectTo: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:355`
    pub epVelocity: vec3_t,
    /// Raven field source: `oracle/codemp/game/g_local.h:356`
    pub epGravFactor: f32,
    /// For bonus items.
    /// Raven's `gitem_t *item` only ever needs the table index, so this field holds the [`ItemId`],
    /// with `Option` for the C NULL, following the `FnId<EntThink>` precedent (private tail, no ABI pin).
    /// Raven field source: `oracle/codemp/game/g_local.h:358`
    pub item: Option<ItemId>,
}

// Layout parity contract.
// `gentity_t` carries pointers, so its layout is arch-dependent.
// The literal offsets are pinned to the host-64-bit build.
// Only `offset_of(s) == 0` is arch-independent.
// `m_pVehicle`, `client`, and `NPC` carry their real pointee types (`Vehicle_t`, `gclient_t`, `gNPC_t`), restored per DEC-26.
// Each is one pointer wide, so these offsets stay unchanged from the earlier `*mut c_void` form.
// Source: `oracle/codemp/game/g_local.h:133-359`
//
// The engine pins only the SHARED PREFIX: `s`, then `r` (`entityShared_t`), up through `next_roff_time`,
// per the "DO NOT MODIFY ANYTHING ABOVE THIS" comment.
// The engine learns the full stride at runtime through `trap_LocateGameData`, so the private tail is free.
// Several tail fields have since diverged from Raven layout.
// The 10 stored `gentity_t*` fields (`parent` through `teammaster`) became `Option<EntityId>`.
// The owned-`String` migration flipped tail string fields to `String` and deleted `roffname` and `rofftarget`,
// all past `next_roff_time`.
// The asserts below keep only the fixed-prefix fields, each one before the first diverged field.
// `client` still sits immediately after the prefix, so its offset stays unchanged.
const _: () = assert!(core::mem::offset_of!(gentity_t, s) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, r) == 576);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, taskID) == 688);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, client) == 976);

/// Write payload for [`GameContext::ent_set`].
/// One variant exists per writable prefix string slot, so a single choke point owns the slot-pointer
/// transaction for every prefix write in game code.
/// An `Option` payload carries Raven's NULL: `None` maps to a NULL slot.
/// A `str` payload is copied into the level-lifetime prefix-string arena ([`GameWorld::prefixStrings`])
/// with the same `\n` translation Raven's `x = G_NewString(...)` performed.
pub enum PrefixSet<'a> {
    /// `classname` slot.
    /// Always present in Raven, through `x = G_NewString(...)`.
    Classname(&'a str),
    /// `targetname` slot.
    /// `None` maps to Raven NULL.
    Targetname(Option<&'a str>),
    /// `fullName` slot.
    /// `None` maps to Raven NULL.
    FullName(Option<&'a str>),
    /// `script_targetname` slot.
    /// `None` maps to Raven NULL.
    ScriptTargetname(Option<&'a str>),
    /// `behaviorSet[i]` slot.
    /// `None` maps to Raven NULL.
    BehaviorSet(usize, Option<&'a str>),
    /// `classname` slot written straight from a `'static` C literal, bypassing the prefix-string arena.
    /// This is Raven's `x = "noclass"` / `"freed"` path.
    ClassnameStatic(&'static CStr),
}

/// Selects a prefix pool-pointer slot for [`gentity_t::alias_from`].
/// These are the sites that copy Raven's shared allocation, one pool buffer reachable
/// from two entities' slots, instead of making a fresh copy.
pub enum PrefixSlot {
    /// `targetname` slot.
    Targetname,
    /// `fullName` slot.
    FullName,
    /// `behaviorSet[i]` slot.
    BehaviorSet(usize),
}

impl gentity_t {
    /// Aliases a prefix pool pointer from `src`'s slot into `self`'s same slot.
    /// This is a raw pointer copy that matches Raven's shared-allocation semantics.
    /// The C `dst->x = src->x` left both slots pointing at the one `G_Alloc` buffer.
    /// Prefix slots stay `*mut c_char` (drop-in ABI), so this uses a pointer copy.
    /// `G_FindTeams` uses this: the master inherits the slave's `targetname`.
    /// `NPC_Spawn_Do` also uses this: the spawner clones its template's slots.
    ///
    /// # Safety
    /// The aliased pointer stays valid regardless of either entity's lifetime.
    /// The pointed-at bytes are owned by the level-lifetime, append-only prefix arena ([`GameWorld::prefixStrings`]).
    /// This arena never drops an entry on entity free, matching Raven's never-freed `G_Alloc` pool.
    /// So the copy is sound until level teardown.
    pub unsafe fn alias_from(&mut self, src: &gentity_t, slot: PrefixSlot) {
        match slot {
            PrefixSlot::Targetname => self.targetname = src.targetname,
            PrefixSlot::FullName => self.fullName = src.fullName,
            PrefixSlot::BehaviorSet(i) => self.behaviorSet[i] = src.behaviorSet[i],
        }
    }

    /// Decodes the live `classname` slot. NULL maps to `""`.
    /// The engine (ICARUS) writes the slot, so the code decodes it fresh every call and never caches it.
    pub fn classname_str(&self) -> String {
        if self.classname.is_null() {
            String::new()
        } else {
            unsafe { cstr_to_str(self.classname) }
        }
    }

    /// Decodes the live `targetname` slot. `None` maps to Raven NULL.
    pub fn targetname_str(&self) -> Option<String> {
        prefix_slot_str(self.targetname)
    }

    /// Decodes the live `fullName` slot. `None` maps to Raven NULL.
    pub fn fullname_str(&self) -> Option<String> {
        prefix_slot_str(self.fullName)
    }

    /// Decodes the live `script_targetname` slot. `None` maps to Raven NULL.
    pub fn script_targetname_str(&self) -> Option<String> {
        prefix_slot_str(self.script_targetname)
    }

    /// Decodes the live `behaviorSet[i]` slot. `None` maps to Raven NULL.
    pub fn behavior_set_str(&self, i: usize) -> Option<String> {
        prefix_slot_str(self.behaviorSet[i])
    }

    /// Drops every owned-`String` tail field with `mem::take`, so each becomes an empty `String`.
    /// This leaves the byte image safe to zero wholesale.
    /// [`Self::seat_owned_strings`] pairs with this method to bracket the `memset`-equivalent
    /// `write_bytes` in `G_FreeEntity`.
    /// Later batches extend this set as more tail fields migrate to owned strings.
    /// Raven has no counterpart here: Raven's fields were pool pointers, cleared by the `memset` itself.
    pub fn take_owned_strings(&mut self) {
        let _ = core::mem::take(&mut self.healingclass);
        let _ = core::mem::take(&mut self.healingsound);
        let _ = core::mem::take(&mut self.ownername);
        let _ = core::mem::take(&mut self.NPC_type);
        let _ = core::mem::take(&mut self.NPC_targetname);
        let _ = core::mem::take(&mut self.NPC_target);
        let _ = core::mem::take(&mut self.target5);
        let _ = core::mem::take(&mut self.target6);
        let _ = core::mem::take(&mut self.targetShaderName);
        let _ = core::mem::take(&mut self.targetShaderNewName);
        let _ = core::mem::take(&mut self.goaltarget);
        let _ = core::mem::take(&mut self.idealclass);
        let _ = core::mem::take(&mut self.target);
        let _ = core::mem::take(&mut self.target2);
        let _ = core::mem::take(&mut self.target3);
        let _ = core::mem::take(&mut self.target4);
        let _ = core::mem::take(&mut self.team);
        let _ = core::mem::take(&mut self.message);
        let _ = core::mem::take(&mut self.model);
        let _ = core::mem::take(&mut self.model2);
        let _ = core::mem::take(&mut self.soundSet);
        let _ = core::mem::take(&mut self.closetarget);
        let _ = core::mem::take(&mut self.opentarget);
        let _ = core::mem::take(&mut self.paintarget);
    }

    /// Seats a fresh empty `String` into every owned-`String` tail field of a freshly-zeroed entity image.
    /// This overwrites the invalid all-zero `String` bytes without dropping them, through `ptr::write`.
    /// This mirrors `zeroed_clients`'s per-slot `String` install.
    /// The arena constructor and the `G_FreeEntity` zero dance both call it.
    /// `p` must point at a live, possibly zeroed, allocation for one `gentity_t`.
    ///
    /// # Safety
    /// `p` is a valid, aligned, writable pointer to one `gentity_t`.
    /// Its owned `String` slots may hold invalid, zeroed bytes that must not be dropped.
    pub unsafe fn seat_owned_strings(p: *mut Self) {
        core::ptr::write(core::ptr::addr_of_mut!((*p).healingclass), String::new());
        core::ptr::write(core::ptr::addr_of_mut!((*p).healingsound), String::new());
        core::ptr::write(core::ptr::addr_of_mut!((*p).ownername), String::new());
        core::ptr::write(core::ptr::addr_of_mut!((*p).NPC_type), None);
        core::ptr::write(core::ptr::addr_of_mut!((*p).NPC_targetname), String::new());
        core::ptr::write(core::ptr::addr_of_mut!((*p).NPC_target), String::new());
        core::ptr::write(core::ptr::addr_of_mut!((*p).target5), String::new());
        core::ptr::write(core::ptr::addr_of_mut!((*p).target6), String::new());
        core::ptr::write(core::ptr::addr_of_mut!((*p).targetShaderName), String::new());
        core::ptr::write(core::ptr::addr_of_mut!((*p).targetShaderNewName), String::new());
        core::ptr::write(core::ptr::addr_of_mut!((*p).goaltarget), String::new());
        core::ptr::write(core::ptr::addr_of_mut!((*p).idealclass), String::new());
        core::ptr::write(core::ptr::addr_of_mut!((*p).target), None);
        core::ptr::write(core::ptr::addr_of_mut!((*p).target2), None);
        core::ptr::write(core::ptr::addr_of_mut!((*p).target3), String::new());
        core::ptr::write(core::ptr::addr_of_mut!((*p).target4), String::new());
        core::ptr::write(core::ptr::addr_of_mut!((*p).team), None);
        core::ptr::write(core::ptr::addr_of_mut!((*p).message), None);
        core::ptr::write(core::ptr::addr_of_mut!((*p).model), None);
        core::ptr::write(core::ptr::addr_of_mut!((*p).model2), String::new());
        core::ptr::write(core::ptr::addr_of_mut!((*p).soundSet), String::new());
        core::ptr::write(core::ptr::addr_of_mut!((*p).closetarget), None);
        core::ptr::write(core::ptr::addr_of_mut!((*p).opentarget), None);
        core::ptr::write(core::ptr::addr_of_mut!((*p).paintarget), None);
    }
}

impl GameContext<'_> {
    /// Stores `s` into the level-lifetime prefix-string arena ([`GameWorld::prefixStrings`]).
    /// This reproduces Raven `G_NewString`'s `\n`-escape translation and returns a `*mut c_char` into
    /// that owned `CString`'s stable heap buffer for a prefix slot.
    /// This method replaces `G_NewString`.
    /// The copy is owned for the level's lifetime and is never freed on entity free, matching
    /// Raven's never-freed `G_Alloc` pool.
    /// Taking the pointer before moving the `CString` into the `Vec` is sound.
    /// The move relocates only the `CString` struct, not its heap buffer, so the pointer stays valid
    /// across this call and every later push.
    /// Source: replaces `oracle/codemp/game/g_spawn.c:724-749` (`G_NewString`).
    pub fn prefix_string(&mut self, s: &str) -> *mut c_char {
        let s_c = cstr(&translate_newlines(s));
        let ptr = s_c.as_ptr() as *mut c_char;
        self.world.prefixStrings.push(s_c);
        ptr
    }

    /// The single prefix-slot write choke point (design §4d-bis).
    /// Only one borrow exists at a time.
    /// The arena copy happens first, through [`Self::prefix_string`], with no trap.
    /// The slot write then happens through a fresh entity borrow.
    /// So the slot is never observable in a partial state at any trap boundary, and no `&mut` into
    /// the arena coexists with the world borrow.
    pub fn ent_set(&mut self, id: EntityId, field: PrefixSet) {
        match field {
            PrefixSet::Classname(name) => {
                let s = self.prefix_string(name);
                self.entity_mut(id).classname = s;
            }
            PrefixSet::ClassnameStatic(name) => {
                self.entity_mut(id).classname = name.as_ptr() as *mut c_char;
            }
            PrefixSet::Targetname(name) => {
                let s = new_string_or_null(self, name);
                self.entity_mut(id).targetname = s;
            }
            PrefixSet::FullName(name) => {
                let s = new_string_or_null(self, name);
                self.entity_mut(id).fullName = s;
            }
            PrefixSet::ScriptTargetname(name) => {
                let s = new_string_or_null(self, name);
                self.entity_mut(id).script_targetname = s;
            }
            PrefixSet::BehaviorSet(i, name) => {
                let s = new_string_or_null(self, name);
                self.entity_mut(id).behaviorSet[i] = s;
            }
        }
    }
}

/// Returns an arena copy of `name` when present, through [`GameContext::prefix_string`] with its
/// `\n` translation, or a NULL slot when absent.
/// This is the shared body behind [`PrefixSet`]'s `Option` arms, mirroring Raven's
/// `x = value ? G_NewString(value) : NULL`.
fn new_string_or_null(ctx: &mut GameContext, name: Option<&str>) -> *mut c_char {
    match name {
        Some(s) => ctx.prefix_string(s),
        None => core::ptr::null_mut(),
    }
}

/// Decodes a live prefix `*mut c_char` slot into an owned `String`. NULL maps to `None`.
/// Raven distinguishes an unset slot from an empty one.
/// The `Option`-returning `_str` readers share this function.
fn prefix_slot_str(slot: *mut c_char) -> Option<String> {
    if slot.is_null() {
        None
    } else {
        Some(unsafe { cstr_to_str(slot) })
    }
}

// `gentity_t` is no longer `ZeroValid`.
// The owned-`String` tail fields make an all-zero image an invalid value, because a zeroed `String`
// has a null data pointer.
// Wholesale-zero construction now goes through `zeroed_entities()` (arena) and the
// `take`/`seat_owned_strings` dance in `G_FreeEntity`, which install valid empty `String`s into those slots.
// The seven fn-ID dispatch fields (`think` through `die`) are still `FnId<EntXxx>`, a
// `#[repr(transparent)]` wrapper over `Option<NonZeroU8>`.
// Its zeroed bytes decode as `None`, meaning no handler, by construction, matching Raven's NULL fn pointers.
// `fn_id_niche_tests` is the regression lock.

#[cfg(test)]
mod fn_id_niche_tests {
    use super::*;
    use core::mem::{align_of, size_of, MaybeUninit};

    /// The `FnId<EntXxx>` handler fields must stay 1 byte, with align 1.
    /// This is the same size the legacy `Option<EntThink>` fields had, so `gentity_t`'s layout is unchanged.
    /// `Option<NonZeroU8>` is 1 byte through the niche optimization.
    #[test]
    fn handler_field_size_matches_legacy_one_byte() {
        assert_eq!(size_of::<FnId<EntThink>>(), 1);
        assert_eq!(align_of::<FnId<EntThink>>(), 1);
        assert_eq!(size_of::<FnId<EntReached>>(), 1);
        assert_eq!(size_of::<FnId<EntBlocked>>(), 1);
        assert_eq!(size_of::<FnId<EntTouch>>(), 1);
        assert_eq!(size_of::<FnId<EntUse>>(), 1);
        assert_eq!(size_of::<FnId<EntPain>>(), 1);
        assert_eq!(size_of::<FnId<EntDie>>(), 1);
    }

    /// The whole-bug-class regression lock.
    /// In a fully byte-zeroed `gentity_t` image, all seven handler fields read as `None`.
    /// Before the `FnId` refactor, this decoded as `Some(variant 0)`, for example `touch == HolocronTouch`.
    ///
    /// `gentity_t` is no longer `ZeroValid`, because its owned `String` tail makes an all-zero image
    /// an invalid value.
    /// So the handler bytes are read off a zeroed `MaybeUninit` through raw pointers.
    /// The code never materializes a `gentity_t` value, which would be UB on the zeroed `String` slots.
    #[test]
    fn zeroed_gentity_reads_all_handlers_none() {
        use core::ptr::addr_of;
        let z = MaybeUninit::<gentity_t>::zeroed();
        let p = z.as_ptr();
        // SAFETY: `p` points at zeroed, aligned storage.
        // Each `FnId<EntXxx>` field is one byte, and its all-zero pattern is the valid `None` encoding.
        // So reading it out is sound, because no `String` slot is touched.
        unsafe {
            assert!(addr_of!((*p).think).read().is_none());
            assert!(addr_of!((*p).reached).read().is_none());
            assert!(addr_of!((*p).blocked).read().is_none());
            assert!(addr_of!((*p).touch).read().is_none());
            assert!(addr_of!((*p).use_).read().is_none());
            assert!(addr_of!((*p).pain).read().is_none());
            assert!(addr_of!((*p).die).read().is_none());
        }
    }
}
