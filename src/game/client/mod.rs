//! The server-side player client: `gclient_s` / `gclient_t`, session, persistant (Raven `g_local.h`).
//!
//! `gclient_s` (`gclient_t`) is engine-visible via its leading `playerState_t ps`;
//! the rest is private to the game. Pointer-bearing => arch-dependent layout (the
//! `#[cfg(target_pointer_width = "64")]` asserts pin the host-64-bit offsets).
//!
//! Migration target: `crate::modules::mp::game::client`.
//! Source: `oracle/oracle/codemp/game/g_local.h:366`

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// TODO: NOT-PORTED
use crate::game::npc::lookMode_t;
// TODO: NOT-PORTED
use crate::game::teams::{class_t, npcteam_t};
use crate::common::mp::qcommon::usercmd_t;
// TODO: NOT-PORTED
use crate::shared::{playerState_t, qboolean, saberInfo_t, vec3_t, MAX_QPATH, MAX_SABERS};
use core::ffi::{c_char, c_int, c_uint, c_void};

use super::entity::gentity_s;

/// `clientConnected_t`.
///
/// Raven: anonymous enum + `typedef int`.
/// Source: `oracle/oracle/codemp/game/g_local.h:366`
pub type clientConnected_t = c_int;
pub const CON_DISCONNECTED: clientConnected_t = 0;
pub const CON_CONNECTING: clientConnected_t = 1;
pub const CON_CONNECTED: clientConnected_t = 2;

/// `spectatorState_t`.
///
/// Source: `oracle/oracle/codemp/game/g_local.h:373`
pub type spectatorState_t = c_int;
pub const SPECTATOR_NOT: spectatorState_t = 0;
pub const SPECTATOR_FREE: spectatorState_t = 1;
pub const SPECTATOR_FOLLOW: spectatorState_t = 2;
pub const SPECTATOR_SCOREBOARD: spectatorState_t = 3;

/// `playerTeamStateState_t`.
///
/// Source: `oracle/oracle/codemp/game/g_local.h:380`
pub type playerTeamStateState_t = c_int;
pub const TEAM_BEGIN: playerTeamStateState_t = 0; // Beginning a team game, spawn at base
pub const TEAM_ACTIVE: playerTeamStateState_t = 1; // Now actively playing

/// `playerTeamState_t`.
///
/// Raven: status in teamplay games.
/// Source: `oracle/oracle/codemp/game/g_local.h:385`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct playerTeamState_t {
    pub state: playerTeamStateState_t,

    pub location: c_int,

    pub captures: c_int,
    pub basedefense: c_int,
    pub carrierdefense: c_int,
    pub flagrecovery: c_int,
    pub fragcarrier: c_int,
    pub assists: c_int,

    pub lasthurtcarrier: f32,
    pub lastreturnedflag: f32,
    pub flagsince: f32,
    pub lastfraggedcarrier: f32,
}
const _: () = assert!(core::mem::size_of::<playerTeamState_t>() == 48);

// the auto following clients don't follow a specific client
// number, but instead follow the first two active players
pub const FOLLOW_ACTIVE1: c_int = -1;
pub const FOLLOW_ACTIVE2: c_int = -2;

/// `clientSession_t`.
///
/// Raven: client data that stays across multiple levels or tournament restarts.
/// Raven: this is achieved by writing all the data to cvar strings at game shutdown
/// Raven: time and reading them back at connection time. Anything added here
/// Raven: MUST be dealt with in G_InitSessionData() / G_ReadSessionData() / G_WriteSessionData().
/// Source: `oracle/oracle/codemp/game/g_local.h:408`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct clientSession_t {
    pub sessionTeam: c_int,   // team_t
    pub spectatorTime: c_int, // for determining next-in-line to play
    pub spectatorState: spectatorState_t,
    pub spectatorClient: c_int, // for chasecam and follow mode
    pub wins: c_int,            // tournament stats
    pub losses: c_int,
    pub selectedFP: c_int, // check against this, if doesn't match value in playerstate then update userinfo
    pub saberLevel: c_int, // similar to above method, but for current saber attack level
    pub setForce: qboolean, // set to true once player is given the chance to set force powers
    pub updateUITime: c_int, // only update userinfo for FP/SL if < level.time
    pub teamLeader: qboolean, // true when this client is a team leader
    pub siegeClass: [c_char; 64],
    pub saberType: [c_char; 64],
    pub saber2Type: [c_char; 64],
    pub duelTeam: c_int,
    pub siegeDesiredTeam: c_int,
    pub killCount: c_int,
    pub TKCount: c_int,
    pub IPstring: [c_char; 32], // yeah, I know, could be 16, but, just in case...
}
const _: () = assert!(core::mem::size_of::<clientSession_t>() == 284);

// playerstate mGameFlags
pub const PSG_VOTED: c_int = 1 << 0; // already cast a vote
pub const PSG_TEAMVOTED: c_int = 1 << 1; // already cast a team vote

pub const MAX_NETNAME: usize = 36;
pub const MAX_VOTE_COUNT: c_int = 3;

/// `clientPersistant_t`.
///
/// Raven: client data that stays across multiple respawns, but is cleared on
/// Raven: each level change or team change at ClientBegin().
/// Source: `oracle/oracle/codemp/game/g_local.h:441`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct clientPersistant_t {
    pub connected: clientConnected_t,
    pub cmd: usercmd_t,              // we would lose angles if not persistant
    pub localClient: qboolean,       // true if "ip" info key is "localhost"
    pub initialSpawn: qboolean,      // the first spawn should be at a cool location
    pub predictItemPickup: qboolean, // based on cg_predictItems userinfo
    pub pmoveFixed: qboolean,        //
    pub netname: [c_char; MAX_NETNAME],
    pub netnameTime: c_int,           // Last time the name was changed
    pub maxHealth: c_int,             // for handicapping
    pub enterTime: c_int,             // level.time the client entered the game
    pub teamState: playerTeamState_t, // status in teamplay games
    pub voteCount: c_int,             // to prevent people from constantly calling votes
    pub teamVoteCount: c_int,         // to prevent people from constantly calling votes
    pub teamInfo: qboolean,           // send team overlay updates?
}
const _: () = assert!(core::mem::size_of::<clientPersistant_t>() == 156);
const _: () = assert!(core::mem::offset_of!(clientPersistant_t, netname) == 48);
const _: () = assert!(core::mem::offset_of!(clientPersistant_t, teamState) == 96);

/// `renderInfo_t`.
///
/// Raven: per-client model-rendering state: model-part yaw/pitch ranges, muzzle
/// Raven: points, tag points, look target, bolt indices, and `lastG2`.
/// Source: `oracle/oracle/codemp/game/g_local.h:460`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct renderInfo_t {
    //In whole degrees, How far to let the different model parts yaw and pitch
    pub headYawRangeLeft: c_int,
    pub headYawRangeRight: c_int,
    pub headPitchRangeUp: c_int,
    pub headPitchRangeDown: c_int,

    pub torsoYawRangeLeft: c_int,
    pub torsoYawRangeRight: c_int,
    pub torsoPitchRangeUp: c_int,
    pub torsoPitchRangeDown: c_int,

    pub legsFrame: c_int,
    pub torsoFrame: c_int,

    pub legsFpsMod: f32,
    pub torsoFpsMod: f32,

    //Fields to apply to entire model set, individual model's equivalents will modify this value
    pub customRGB: vec3_t,  //Red Green Blue, 0 = don't apply
    pub customAlpha: c_int, //Alpha to apply, 0 = none?

    //RF?
    pub renderFlags: c_int,

    //
    pub muzzlePoint: vec3_t,
    pub muzzleDir: vec3_t,
    pub muzzlePointOld: vec3_t,
    pub muzzleDirOld: vec3_t,
    //vec3_t		muzzlePointNext;	// Muzzle point one server frame in the future!
    //vec3_t		muzzleDirNext;
    pub mPCalcTime: c_int, //Last time muzzle point was calced

    //
    pub lockYaw: f32, //

    //
    pub headPoint: vec3_t,   //Where your tag_head is
    pub headAngles: vec3_t,  //where the tag_head in the torso is pointing
    pub handRPoint: vec3_t,  //where your right hand is
    pub handLPoint: vec3_t,  //where your left hand is
    pub crotchPoint: vec3_t, //Where your crotch is
    pub footRPoint: vec3_t,  //where your right hand is
    pub footLPoint: vec3_t,  //where your left hand is
    pub torsoPoint: vec3_t,  //Where your chest is
    pub torsoAngles: vec3_t, //Where the chest is pointing
    pub eyePoint: vec3_t,    //Where your eyes are
    pub eyeAngles: vec3_t,   //Where your eyes face
    pub lookTarget: c_int,   //Which ent to look at with lookAngles
    pub lookMode: lookMode_t,
    pub lookTargetClearTime: c_int,  //Time to clear the lookTarget
    pub lastVoiceVolume: c_int,      //Last frame's voice volume
    pub lastHeadAngles: vec3_t,      //Last headAngles, NOT actual facing of head model
    pub headBobAngles: vec3_t,       //headAngle offsets
    pub targetHeadBobAngles: vec3_t, //head bob angles will try to get to targetHeadBobAngles
    pub lookingDebounceTime: c_int,  //When we can stop using head looking angle behavior
    pub legsYaw: f32,                //yaw angle your legs are actually rendering at

    //for tracking legitimate bolt indecies
    pub lastG2: *mut c_void, //if it doesn't match ent->ghoul2, the bolts are considered invalid.
    pub headBolt: c_int,
    pub handRBolt: c_int,
    pub handLBolt: c_int,
    pub torsoBolt: c_int,
    pub crotchBolt: c_int,
    pub footRBolt: c_int,
    pub footLBolt: c_int,
    pub motionBolt: c_int,

    pub boltValidityTime: c_int,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<renderInfo_t>() == 368);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(renderInfo_t, lookMode) == 260);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(renderInfo_t, lastG2) == 320);

/// `gclient_s` / `gclient_t`.
///
/// Raven: this structure is cleared on each ClientSpawn(), except for
/// Raven: `client->pers` and `client->sess`.
/// Raven: `ps` MUST be the first element, because the server expects it.
/// Raven: the rest of the structure is private to game.
/// Source: `oracle/oracle/codemp/game/g_local.h:534`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gclient_s {
    // ps MUST be the first element, because the server expects it
    pub ps: playerState_t, // communicated by server to clients

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
    // we can't just use pers.lastCommand.time, because
    // of the g_sycronousclients case
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

    //
    pub lastkilled_client: c_int, // last client that this client killed
    pub lasthurt_client: c_int,   // last client that damaged this client
    pub lasthurt_mod: c_int,      // type of damage the client did

    // timers
    pub respawnTime: c_int, // can respawn when time > this, force after g_forcerespwan
    pub inactivityTime: c_int, // kick players when time > this
    pub inactivityWarning: qboolean, // qtrue if the five seoond warning has been given
    pub rewardTime: c_int,  // clear the EF_AWARD_IMPRESSIVE, etc when time > this

    pub airOutTime: c_int,

    pub lastKillTime: c_int, // for multiple kill rewards

    pub fireHeld: qboolean,   // used for hook
    pub hook: *mut gentity_s, // grapple hook if out

    pub switchTeamTime: c_int, // time the player switched teams

    pub switchDuelTeamTime: c_int, // time the player switched duel teams

    pub switchClassTime: c_int, // class changed debounce timer

    // timeResidual is used to handle events that happen every second
    // like health / armor countdowns and regeneration
    pub timeResidual: c_int,

    pub areabits: *mut c_char,

    pub g2LastSurfaceHit: c_int, //index of surface hit during the most recent ghoul2 collision performed on this client.
    pub g2LastSurfaceTime: c_int, //time when the surface index was set (to make sure it's up to date)

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

    pub saberStoredIndex: c_int, //stores saberEntityNum from playerstate for when it's set to 0 (indicating saber was knocked out of the air)

    pub saberKnockedTime: c_int, //if saber gets knocked away, can't pull it back until this value is < level.time

    pub olderSaberBase: vec3_t, //Set before lastSaberBase_Always, to whatever lastSaberBase_Always was previously
    pub olderIsValid: qboolean, //is it valid?

    pub lastSaberDir_Always: vec3_t, //every getboltmatrix, set to saber dir
    pub lastSaberBase_Always: vec3_t, //every getboltmatrix, set to saber base
    pub lastSaberStorageTime: c_int, //server time that the above two values were updated (for making sure they aren't out of date)

    pub hasCurrentPosition: qboolean, //are lastSaberTip and lastSaberBase valid?

    pub dangerTime: c_int, // level.time when last attack occured

    pub idleTime: c_int, //keep track of when to play an idle anim on the client.

    pub idleHealth: c_int,      //stop idling if health decreases
    pub idleViewAngles: vec3_t, //stop idling if viewangles change

    pub forcePowerSoundDebounce: c_int, //if > level.time, don't do certain sound events again (drain sound, absorb sound, etc)

    pub modelname: [c_char; MAX_QPATH],

    pub fjDidJump: qboolean,

    pub ikStatus: qboolean,

    pub throwingIndex: c_int,
    pub beingThrown: c_int,
    pub doingThrow: c_int,

    pub hiddenDist: f32,   //How close ents have to be to pick you up as an enemy
    pub hiddenDir: vec3_t, //Normalized direction in which NPCs can't see you (you are hidden)

    pub renderInfo: renderInfo_t,

    //mostly NPC stuff:
    pub playerTeam: npcteam_t,
    pub enemyTeam: npcteam_t,
    pub squadname: *mut c_char,
    pub team_leader: *mut gentity_s,
    pub leader: *mut gentity_s,
    pub follower: *mut gentity_s,
    pub numFollowers: c_int,
    pub formationGoal: *mut gentity_s,
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
    pub ewebHealth: c_int, //health of e-web (to keep track between deployments)

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

/// `gclient_t`.
///
/// Raven: `typedef struct gclient_s gclient_t`.
/// Source: `oracle/oracle/codemp/game/g_local.h:17`
pub type gclient_t = gclient_s;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<gclient_t>() == 7344);
const _: () = assert!(core::mem::offset_of!(gclient_t, ps) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gclient_t, pers) == 1552);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gclient_t, sess) == 1708);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gclient_t, saber) == 1992);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gclient_t, renderInfo) == 6776);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gclient_t, NPC_class) == 7204);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gclient_t, lastGenCmdTime) == 7324);
