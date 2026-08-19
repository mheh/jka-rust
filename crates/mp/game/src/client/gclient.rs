//! MP `gclient_s` / `gclient_t`.
//!
//! Type definition source: `oracle/codemp/game/g_local.h:536-748`

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_uint, c_void};

use crate::entity::gentity_t;
use mp_qshared::common::mp::entity_id::EntityId;
use mp_qshared::common::mp::qcommon::{playerState_t, saberInfo_t, MAX_SABERS};
use mp_qshared::shared::{qboolean, vec3_t};

use crate::teams::{class_t, npcteam_t};

use super::client_persistant::clientPersistant_t;
use super::client_session::clientSession_t;
use super::render_info::renderInfo_t;

/// Raven `gclient_s`. The engine clears this struct on each `ClientSpawn()`, except for `pers` and `sess`.
///
/// `ps` must be first, because the server expects it. The rest of the struct is private to the game.
/// The struct holds pointers, so its layout depends on the target architecture. The asserts below pin the host 64-bit layout.
/// Type definition source: `oracle/codemp/game/g_local.h:536-748`
///
/// This type is not `Copy`, because `pers.netname` is an owned `String`.
/// The struct keeps `#[repr(C)]` and `ps` at offset 0,
/// since the engine reads the `playerState` prefix at a runtime-queried stride (`trap_LocateGameData`).
/// Every `offset_of` assert at or after `pers` is dropped, because `pers`'s byte size shrank when `netname` stopped being a fixed array.
#[repr(C)]
#[derive(Clone)]
pub struct gclient_s {
    // ps MUST be the first element, because the server expects it
    pub ps: playerState_t,

    // the rest of the structure is private to game
    pub pers: clientPersistant_t,
    pub sess: clientSession_t,

    pub saber: [saberInfo_t; MAX_SABERS],
    pub weaponGhoul2: [*mut c_void; MAX_SABERS],

    pub tossableItemDebounce: c_int,

    pub bodyGrabTime: c_int,
    pub bodyGrabIndex: c_int,

    pub pushEffectTime: c_int,

    pub invulnerableTimer: c_int,

    pub saberCycleQueue: c_int,

    pub legsAnimExecute: c_int,
    pub torsoAnimExecute: c_int,
    pub legsLastFlip: qboolean,
    pub torsoLastFlip: qboolean,

    pub readyToExit: qboolean, // wishes to leave the intermission

    pub noclip: qboolean,

    pub lastCmdTime: c_int, // level.time of last usercmd_t, for EF_CONNECTION
    pub buttons: c_int,
    pub oldbuttons: c_int,
    pub latched_buttons: c_int,

    pub oldOrigin: vec3_t,

    // sum up damage over an entire frame, so
    // shotgun blasts give a single big kick
    pub damage_armor: c_int,        // damage absorbed by armor
    pub damage_blood: c_int,        // damage taken out of health
    pub damage_knockback: c_int,    // impact damage
    pub damage_from: vec3_t,        // origin for vector calculation
    pub damage_fromWorld: qboolean, // if true, don't use the damage_from vector

    pub damageBoxHandle_Head: c_int, //entity number of head damage box
    pub damageBoxHandle_RLeg: c_int, //entity number of right leg damage box
    pub damageBoxHandle_LLeg: c_int, //entity number of left leg damage box

    pub accurateCount: c_int, // for "impressive" reward sound

    pub accuracy_shots: c_int, // total number of shots
    pub accuracy_hits: c_int,  // total number of hits

    pub lastkilled_client: c_int, // last client that this client killed
    pub lasthurt_client: c_int,   // last client that damaged this client
    pub lasthurt_mod: c_int,      // type of damage the client did

    // timers
    pub respawnTime: c_int,          // can respawn when time > this
    pub inactivityTime: c_int,       // kick players when time > this
    pub inactivityWarning: qboolean, // qtrue if the five seoond warning has been given
    pub rewardTime: c_int,           // clear the EF_AWARD_IMPRESSIVE, etc when time > this

    pub airOutTime: c_int,

    pub lastKillTime: c_int, // for multiple kill rewards

    pub fireHeld: qboolean,     // used for hook
    pub hook: Option<EntityId>, // grapple hook if out

    pub switchTeamTime: c_int, // time the player switched teams

    pub switchDuelTeamTime: c_int, // time the player switched duel teams

    pub switchClassTime: c_int, // class changed debounce timer

    // timeResidual is used to handle events that happen every second
    // like health / armor countdowns and regeneration
    pub timeResidual: c_int,

    pub areabits: *mut c_char,

    pub g2LastSurfaceHit: c_int, //index of surface hit during the most recent ghoul2 collision
    pub g2LastSurfaceTime: c_int, //time when the surface index was set

    pub corrTime: c_int,

    pub lastHeadAngles: vec3_t,
    pub lookTime: c_int,

    pub brokenLimbs: c_int,

    pub noCorpse: qboolean, //don't leave a corpse on respawn this time.

    pub jetPackTime: c_int,

    pub jetPackOn: qboolean,
    pub jetPackToggleTime: c_int,
    pub jetPackDebRecharge: c_int,
    pub jetPackDebReduce: c_int,

    pub cloakToggleTime: c_int,
    pub cloakDebRecharge: c_int,
    pub cloakDebReduce: c_int,

    pub saberStoredIndex: c_int, //stores saberEntityNum for when it's set to 0

    pub saberKnockedTime: c_int, //if saber gets knocked away, can't pull it back until this value is < level.time

    pub olderSaberBase: vec3_t, //Set before lastSaberBase_Always, to whatever lastSaberBase_Always was previously
    pub olderIsValid: qboolean,

    pub lastSaberDir_Always: vec3_t, //every getboltmatrix, set to saber dir
    pub lastSaberBase_Always: vec3_t, //every getboltmatrix, set to saber base
    pub lastSaberStorageTime: c_int, //server time that the above two values were updated

    pub hasCurrentPosition: qboolean, //are lastSaberTip and lastSaberBase valid?

    pub dangerTime: c_int, // level.time when last attack occured

    pub idleTime: c_int, //keep track of when to play an idle anim on the client.

    pub idleHealth: c_int,      // stop idling if health decreases
    pub idleViewAngles: vec3_t, // stop idling if viewangles change

    pub forcePowerSoundDebounce: c_int, //if > level.time, don't do certain sound events again (drain sound, absorb sound, etc)

    // This field is an owned `String`. It is game-private (past `ps` and `pers`), so no layout contract binds it.
    // The write bound on its byte width stays `MAX_QPATH - 1`.
    pub modelname: String,

    pub fjDidJump: qboolean,

    pub ikStatus: qboolean,

    pub throwingIndex: c_int,
    pub beingThrown: c_int,
    pub doingThrow: c_int,

    pub hiddenDist: f32,   //How close ents have to be to pick you up as an enemy
    pub hiddenDir: vec3_t, //Normalized direction in which NPCs can't see you

    pub renderInfo: renderInfo_t,

    //mostly NPC stuff:
    pub playerTeam: npcteam_t,
    pub enemyTeam: npcteam_t,
    pub squadname: *mut c_char,
    pub team_leader: Option<EntityId>,
    pub leader: Option<EntityId>,
    pub follower: Option<EntityId>,
    pub numFollowers: c_int,
    pub formationGoal: *mut gentity_t,
    pub nextFormGoal: c_int,
    pub NPC_class: class_t,

    pub pushVec: vec3_t,
    pub pushVecTime: c_int,

    pub siegeClass: c_int,
    pub holdingObjectiveItem: c_int,

    //time values for when being healed/supplied by supplier class
    pub isMedHealed: c_int,
    pub isMedSupplied: c_int,

    //seperate debounce time for refilling someone's ammo as a supplier
    pub medSupplyDebounce: c_int,

    //used in conjunction with ps.hackingTime
    pub isHacking: c_int,
    pub hackingAngles: vec3_t,

    //debounce time for sending extended siege data to certain classes
    pub siegeEDataSend: c_int,

    pub ewebIndex: c_int,  //index of e-web gun if spawned
    pub ewebTime: c_int,   //e-web use debounce
    pub ewebHealth: c_int, //health of e-web

    pub inSpaceIndex: c_int,       //ent index of space trigger if inside one
    pub inSpaceSuffocation: c_int, //suffocation timer

    pub tempSpectate: c_int, //time to force spectator mode

    //keep track of last person kicked and the time so we don't hit multiple times per kick
    pub jediKickIndex: c_int,
    pub jediKickTime: c_int,

    //special moves (designed for kyle boss npc, but useable by players in mp)
    pub grappleIndex: c_int,
    pub grappleState: c_int,

    pub solidHack: c_int,

    pub noLightningTime: c_int,

    pub mGameFlags: c_uint,

    //fallen duelist
    pub iAmALoser: qboolean,

    pub lastGenCmd: c_int,
    pub lastGenCmdTime: c_int,

    //can't put these in playerstate, crashes game (need to change exe?)
    pub otherKillerMOD: c_int,
    pub otherKillerVehWeapon: c_int,
    pub otherKillerWeaponType: c_int,
}

/// Raven `gclient_t` is `typedef struct gclient_s gclient_t`.
///
/// Type definition source: `oracle/codemp/game/g_local.h:17`
pub type gclient_t = gclient_s;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gclient_t, ps) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gclient_t, pers) == 1552);
// `ps` (offset 0) and `pers` (offset 1552, which equals `size_of::<playerState_t>()`) are unaffected by `pers`'s internal
// `String` field.
// The `size_of` assert and every `offset_of` assert at or after `pers` are dropped, because `pers`'s byte size shrank when
// `netname` became a `String`.
// This struct is game-internal, not ABI-fixed beyond its prefix. The engine learns the full stride at runtime through
// `trap_LocateGameData`.

impl Default for gclient_t {
    /// Raven zero-fills `gclient_t` wholesale: `memset(client, 0, ...)` in `ClientConnect`/`ClientSpawn`, and
    /// `memset(g_clients, 0, ...)` in `G_InitGame`.
    /// Every field is all-zero-valid, except the owned `String`s (`pers.netname`,
    /// `sess.{siegeClass,saberType,saber2Type,IPstring}`, `modelname`), whose zeroed bytes are invalid values.
    /// This impl writes the zeroed image into heap storage, then installs a valid empty `String` into each of those slots
    /// before the value is ever read.
    /// The result matches Raven's zero state: every scalar is 0, and every name is empty.
    fn default() -> Self {
        let mut u = core::mem::MaybeUninit::<gclient_t>::uninit();
        let p = u.as_mut_ptr();
        // SAFETY: `p` is freshly-allocated, correctly-aligned storage for one `gclient_t`.
        // `write_bytes` zeroes every field. Every field is all-zero-valid, except the owned `String`s.
        // Each `ptr::write` then overwrites one non-zero-valid `String` slot with a valid empty `String`, and its invalid
        // zeroed bytes are never dropped.
        // `assume_init` then observes a fully-valid value.
        unsafe {
            core::ptr::write_bytes(p, 0, 1);
            core::ptr::write(core::ptr::addr_of_mut!((*p).pers.netname), String::new());
            core::ptr::write(core::ptr::addr_of_mut!((*p).sess.siegeClass), String::new());
            core::ptr::write(core::ptr::addr_of_mut!((*p).sess.saberType), String::new());
            core::ptr::write(core::ptr::addr_of_mut!((*p).sess.saber2Type), String::new());
            core::ptr::write(core::ptr::addr_of_mut!((*p).sess.IPstring), String::new());
            core::ptr::write(core::ptr::addr_of_mut!((*p).modelname), String::new());
            u.assume_init()
        }
    }
}
