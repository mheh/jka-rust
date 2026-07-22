//! MP `gentity_t` copied from Raven `codemp/game/g_local.h`.
//!
//! Type declaration source: `oracle/codemp/game/g_local.h:16`
//! Full struct layout source: `oracle/codemp/game/g_local.h:133-359`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

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
use crate::npc::g_npc_t::gNPC_t;

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
    /// Real type restored per DEC-26 (`gentity_t` now lives in `mp_game`).
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
    /// Real type restored per DEC-26 (`gentity_t` now lives in `mp_game`).
    /// Raven field source: `oracle/codemp/game/g_local.h:173`
    pub client: *mut gclient_t,
    /// Real type restored per DEC-26 (`gentity_t` now lives in `mp_game`).
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
    // Raven `roffname`/`rofftarget` (`g_local.h:188-189`) deleted: `rofftarget`
    // had zero readers repo-wide and `roffname` only ever fed the resolved
    // `roffid` int, so both spawn keys became `F_IGNORE` (parse silently) and the
    // ICARUS `roffname` store dropped. Private tail — no ABI impact.
    /// Owned copy of the QuakeEd healing-class name; `""` ≡ absent (Raven `char *`,
    /// distinguished only by `!= 0`, never by NULL-vs-empty).
    /// Raven field source: `oracle/codemp/game/g_local.h:191`
    pub healingclass: String,
    /// Owned copy of the QuakeEd healing sound name; `""` ≡ absent.
    /// Raven field source: `oracle/codemp/game/g_local.h:192`
    pub healingsound: String,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/codemp/game/g_local.h:193`
    pub healingrate: c_int,
    /// Debounce for generic object healing.
    /// Raven field source: `oracle/codemp/game/g_local.h:194`
    pub healingDebounce: c_int,
    /// Owned copy of the QuakeEd owner-tag name; `""` ≡ absent.
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
    /// Raven field source: `oracle/codemp/game/g_local.h:215`
    pub model: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_local.h:216`
    pub model2: *mut c_char,
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
    /// Owned NPC species name for NPC_spawners; `None` ≡ Raven NULL (the code
    /// distinguishes a never-set spawner from one whose type resolved to `""`).
    /// Only used by NPC_spawners.
    /// Raven field source: `oracle/codemp/game/g_local.h:230`
    pub NPC_type: Option<String>,
    /// Owned copy of the NPC_spawner's target name; `""` ≡ absent.
    /// Only used by NPC_spawners.
    /// Raven field source: `oracle/codemp/game/g_local.h:231`
    pub NPC_targetname: String,
    /// Owned copy of the NPC_spawner's target name; `""` ≡ absent.
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
    /// Raven field source: `oracle/codemp/game/g_local.h:249`
    pub message: *mut c_char,
    /// Body queue sinking, etc.
    /// Raven field source: `oracle/codemp/game/g_local.h:251`
    pub timestamp: c_int,
    /// Set in editor, -1 = up, -2 = down.
    /// Raven field source: `oracle/codemp/game/g_local.h:253`
    pub angle: f32,
    /// Raven field source: `oracle/codemp/game/g_local.h:254`
    pub target: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_local.h:255`
    pub target2: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_local.h:256`
    pub target3: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_local.h:257`
    pub target4: *mut c_char,
    /// Owned copy of the siege-item target name; `""` ≡ absent.
    /// Mainly added for siege items.
    /// Raven field source: `oracle/codemp/game/g_local.h:258`
    pub target5: String,
    /// Owned copy of the siege-item target name; `""` ≡ absent.
    /// Mainly added for siege items.
    /// Raven field source: `oracle/codemp/game/g_local.h:259`
    pub target6: String,
    /// Raven field source: `oracle/codemp/game/g_local.h:261`
    pub team: *mut c_char,
    /// Owned copy of the shader-remap source name; `""` ≡ absent.
    /// Raven field source: `oracle/codemp/game/g_local.h:262`
    pub targetShaderName: String,
    /// Owned copy of the shader-remap destination name; `""` ≡ absent.
    /// Raven field source: `oracle/codemp/game/g_local.h:263`
    pub targetShaderNewName: String,
    /// Raven field source: `oracle/codemp/game/g_local.h:264`
    pub target_ent: Option<EntityId>,
    /// Raven field source: `oracle/codemp/game/g_local.h:266`
    pub closetarget: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_local.h:267`
    pub opentarget: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_local.h:268`
    pub paintarget: *mut c_char,
    /// Owned copy of the siege goal-target name; `""` ≡ absent.
    /// Raven field source: `oracle/codemp/game/g_local.h:270`
    pub goaltarget: String,
    /// Owned copy of the siege ideal-class name; `""` ≡ absent.
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
    /// Raven field source: `oracle/codemp/game/g_local.h:348`
    pub soundSet: *mut c_char,
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
    /// For bonus items. Raven `gitem_t *item` — only the table index is ever
    /// needed, so this holds the [`ItemId`] (`Option` for the C NULL), following
    /// the `FnId<EntThink>` precedent (private tail, no ABI pin).
    /// Raven field source: `oracle/codemp/game/g_local.h:358`
    pub item: Option<ItemId>,
}

// Layout parity contract. `gentity_t` carries pointers, so its layout is
// arch-dependent; the literal offsets are pinned to the host-64-bit build (only
// `offset_of(s) == 0` is arch-independent). `m_pVehicle`/`client`/`NPC` carry
// their real pointee types (`Vehicle_t`/`gclient_t`/`gNPC_t`, restored per
// DEC-26); each is one pointer wide, so these offsets are unchanged from the
// earlier `*mut c_void` form.
// Source: `oracle/codemp/game/g_local.h:133-359`
//
// The engine only pins the SHARED PREFIX (`s`, then `r`/`entityShared_t`, up
// through `next_roff_time`, per the "DO NOT MODIFY ANYTHING ABOVE THIS" comment)
// and learns the full stride at runtime via `trap_LocateGameData`, so the
// private tail is free. Several tail fields have since diverged from Raven
// layout: the 10 stored `gentity_t*` (`parent`..`teammaster`) became
// `Option<EntityId>`, and the owned-`String` migration flipped tail string
// fields to `String` (and deleted `roffname`/`rofftarget`) — all past
// `next_roff_time`. Only the fixed-prefix asserts below (every one BEFORE the
// first diverged field) are kept; `client` still sits immediately after the
// prefix, so its offset is unchanged.
const _: () = assert!(core::mem::offset_of!(gentity_t, s) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, r) == 576);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, taskID) == 688);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, client) == 976);

impl gentity_t {
    /// Drops every owned-`String` tail field (`mem::take` → empty `String`),
    /// leaving the byte image safe to wholesale-zero. Paired with
    /// [`Self::seat_owned_strings`] to bracket the `memset`-equivalent
    /// `write_bytes` in `G_FreeEntity` — later batches EXTEND this set as more
    /// tail fields migrate to owned strings. No Raven counterpart (Raven's fields
    /// were pool pointers, cleared by the `memset` itself).
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
    }

    /// Seats a fresh empty `String` into every owned-`String` tail field of a
    /// freshly-zeroed entity image, overwriting the invalid all-zero `String`
    /// bytes WITHOUT dropping them (`ptr::write`). Mirrors `zeroed_clients`'s
    /// per-slot `String` install; the arena constructor and the `G_FreeEntity`
    /// zero dance both call it. `p` must point at a live (possibly zeroed)
    /// allocation for one `gentity_t`.
    ///
    /// # Safety
    /// `p` is a valid, aligned, writable pointer to one `gentity_t` whose owned
    /// `String` slots may hold invalid (zeroed) bytes that must not be dropped.
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
    }
}

// `gentity_t` is no longer `ZeroValid`: the owned-`String` tail fields make an
// all-zero image an invalid value (a zeroed `String` has a null data pointer).
// Wholesale-zero construction now goes through `zeroed_entities()` (arena) and
// the `take`/`seat_owned_strings` dance (`G_FreeEntity`), which install valid
// empty `String`s into those slots. The seven fn-ID dispatch fields (`think`..
// `die`) are still `FnId<EntXxx>` — a `#[repr(transparent)]` wrapper over
// `Option<NonZeroU8>` whose zeroed bytes decode as `None` ("no handler") by
// construction, matching Raven's NULL fn pointers; `fn_id_niche_tests` is the
// regression lock.

#[cfg(test)]
mod fn_id_niche_tests {
    use super::*;
    use core::mem::{align_of, size_of, MaybeUninit};

    /// The `FnId<EntXxx>` handler fields must stay 1 byte / align 1 — the same
    /// size the legacy `Option<EntThink>` fields had — so `gentity_t`'s layout
    /// is unchanged. (`Option<NonZeroU8>` is 1 byte via the niche.)
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

    /// The whole-bug-class regression lock: in a fully byte-zeroed `gentity_t`
    /// image, all seven handler fields read as `None`. Before the `FnId` refactor
    /// this decoded as `Some(variant 0)` (e.g. `touch == HolocronTouch`).
    ///
    /// `gentity_t` is no longer `ZeroValid` (its owned `String` tail makes an
    /// all-zero image an invalid value), so the handler bytes are read off a
    /// zeroed `MaybeUninit` through raw pointers — never materializing a
    /// `gentity_t` value, which would be UB on the zeroed `String` slots.
    #[test]
    fn zeroed_gentity_reads_all_handlers_none() {
        use core::ptr::addr_of;
        let z = MaybeUninit::<gentity_t>::zeroed();
        let p = z.as_ptr();
        // SAFETY: `p` points at zeroed, correctly-aligned storage; each
        // `FnId<EntXxx>` field is one byte whose all-zero pattern is the valid
        // `None` encoding, so reading it out is sound (no `String` slot touched).
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
