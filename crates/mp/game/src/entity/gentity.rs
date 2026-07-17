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
    /// Set in QuakeEd.
    /// Raven field source: `oracle/codemp/game/g_local.h:188`
    pub roffname: *mut c_char,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/codemp/game/g_local.h:189`
    pub rofftarget: *mut c_char,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/codemp/game/g_local.h:191`
    pub healingclass: *mut c_char,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/codemp/game/g_local.h:192`
    pub healingsound: *mut c_char,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/codemp/game/g_local.h:193`
    pub healingrate: c_int,
    /// Debounce for generic object healing.
    /// Raven field source: `oracle/codemp/game/g_local.h:194`
    pub healingDebounce: c_int,
    /// Raven field source: `oracle/codemp/game/g_local.h:196`
    pub ownername: *mut c_char,
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
    /// Only used by NPC_spawners.
    /// Raven field source: `oracle/codemp/game/g_local.h:230`
    pub NPC_type: *mut c_char,
    /// Only used by NPC_spawners.
    /// Raven field source: `oracle/codemp/game/g_local.h:231`
    pub NPC_targetname: *mut c_char,
    /// Only used by NPC_spawners.
    /// Raven field source: `oracle/codemp/game/g_local.h:232`
    pub NPC_target: *mut c_char,
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
    /// Mainly added for siege items.
    /// Raven field source: `oracle/codemp/game/g_local.h:258`
    pub target5: *mut c_char,
    /// Mainly added for siege items.
    /// Raven field source: `oracle/codemp/game/g_local.h:259`
    pub target6: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_local.h:261`
    pub team: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_local.h:262`
    pub targetShaderName: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_local.h:263`
    pub targetShaderNewName: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_local.h:264`
    pub target_ent: Option<EntityId>,
    /// Raven field source: `oracle/codemp/game/g_local.h:266`
    pub closetarget: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_local.h:267`
    pub opentarget: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_local.h:268`
    pub paintarget: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_local.h:270`
    pub goaltarget: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_local.h:271`
    pub idealclass: *mut c_char,
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
// The 10 stored `gentity_t*` fields (`parent`..`teammaster`, all after
// `moverState`) became `Option<EntityId>`. Those pointers were never
// ABI-visible — the engine only pins the SHARED PREFIX (`s`, then
// `r`/`entityShared_t`, up through
// `next_roff_time`, per the "DO NOT MODIFY ANYTHING ABOVE THIS" comment) and
// learns the full stride at runtime via `trap_LocateGameData`. So the private
// tail (`size_of` and every offset at/after the first flipped field `parent`)
// is free and its literal asserts are dropped; only the fixed-prefix asserts
// below (all BEFORE `parent`) are kept.
const _: () = assert!(core::mem::offset_of!(gentity_t, s) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, r) == 576);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, taskID) == 688);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, client) == 976);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, moverState) == 1176);

// The STATE-D9 zeroed-construction contract (round-5 STATE-Q10 resolution):
// all-zero bytes are a valid gentity_t — the same property the layout asserts above
// pin and Raven's memset/static zero-init relies on.
// Source: oracle/codemp/game/g_shared.h (all-zero-valid #[repr(C)]; Raven memsets g_entities, g_main.c:978)
//
// The seven fn-ID dispatch fields (`think`, `reached`, `blocked`, `touch`,
// `use_`, `pain`, `die`) are `FnId<EntXxx>` — a `#[repr(transparent)]` wrapper
// over `Option<NonZeroU8>` (`ent_fn_ids.rs`). std guarantees `Option<NonZero*>`
// (and transparent structs around it) encode `None` as the all-zero bit
// pattern, so zeroed bytes decode as `None` ("no handler") *by construction*,
// matching Raven's C NULL-fn-pointer semantics. There is therefore no post-zero
// fixup: the earlier niche hazard (a bare `Option<EntXxx>` enum, whose `None`
// niche sat AFTER the last variant, so zeroed `touch` read as
// `Some(EntTouch::HolocronTouch)`) is now structurally impossible. The
// `fn_id_niche_tests` module below is the regression lock.
unsafe impl native_platform::ZeroValid for gentity_t {}

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

    /// The whole-bug-class regression lock: a fully byte-zeroed `gentity_t`
    /// reads all seven handler fields as `None`. Before the `FnId` refactor
    /// this decoded as `Some(variant 0)` (e.g. `touch == HolocronTouch`).
    #[test]
    fn zeroed_gentity_reads_all_handlers_none() {
        // SAFETY: `gentity_t: ZeroValid` — all-zero bytes are a valid value;
        // it holds no `Drop` types, so `forget` is a formality.
        let z: gentity_t = unsafe { MaybeUninit::zeroed().assume_init() };
        assert!(z.think.is_none());
        assert!(z.reached.is_none());
        assert!(z.blocked.is_none());
        assert!(z.touch.is_none());
        assert!(z.use_.is_none());
        assert!(z.pain.is_none());
        assert!(z.die.is_none());
        core::mem::forget(z);
    }
}
