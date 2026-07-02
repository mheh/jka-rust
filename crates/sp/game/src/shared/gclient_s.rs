#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_schar};

use sp_qshared::common::sp::gentity_t;
use sp_qshared::common::sp::qcommon::{playerState_t, usercmd_t};
use sp_qshared::shared::{qboolean, vec3_t};

use crate::teams::class::class_t;
use crate::teams::team::team_t;

use super::client_info_t::clientInfo_t;
use super::client_persistant_t::clientPersistant_t;
use super::client_session_t::clientSession_t;
use super::movetype_t::movetype_t;
use super::render_info_s::renderInfo_t;

/// Raven `gclient_t`.
///
/// Raven: `ps` MUST be the first element, because the server expects it; the rest
/// is private to game.
/// Type definition source: `oracle/oracle/code/game/g_shared.h:387-488`
#[repr(C)]
pub struct gclient_t {
    // ps MUST be the first element, because the server expects it
    /// communicated by server to clients
    pub ps: playerState_t,

    // private to game
    pub pers: clientPersistant_t,
    pub sess: clientSession_t,

    /// level.time of last usercmd_t, for EF_CONNECTION
    pub lastCmdTime: c_int,

    /// most recent usercmd
    pub usercmd: usercmd_t,

    pub buttons: c_int,
    pub oldbuttons: c_int,
    pub latched_buttons: c_int,

    // sum up damage over an entire frame, so
    // shotgun blasts give a single big kick
    /// damage absorbed by armor
    pub damage_armor: c_int,
    /// damage taken out of health
    pub damage_blood: c_int,
    /// origin for vector calculation
    pub damage_from: vec3_t,
    /// if true, don't use the damage_from vector
    pub damage_fromWorld: bool,
    pub noclip: bool,
    //icarus forced moving.  is this still used?
    pub forced_forwardmove: c_schar,
    pub forced_rightmove: c_schar,

    // timers
    /// can respawn when time > this, force after g_forcerespwan
    pub respawnTime: c_int,
    /// for playing idleAnims
    pub idleTime: c_int,

    pub airOutTime: c_int,

    // timeResidual is used to handle events that happen every second
    // like health / armor countdowns and regeneration
    pub timeResidual: c_int,

    // Facial Expression Timers
    /// time before next blink. If a minus value, we are in blink mode
    pub facial_blink: f32,
    /// time before next alert, frown or smile. If a minus value, we are in anim mode
    pub facial_timer: f32,
    /// anim to show in anim mode
    pub facial_anim: c_int,

    // Client info - updated when ClientInfoChanged is called, instead of using configstrings
    pub clientInfo: clientInfo_t,
    pub moveType: movetype_t,
    pub jetPackTime: c_int,
    /// msec to delay calling G_FireWeapon after EV_FIREWEAPON event is called
    pub fireDelay: c_int,

    /// The time at which a breath should be triggered. -Aurelio
    pub breathPuffTime: c_int,

    // Used to be in gentity_t, now here.. mostly formation stuff
    pub playerTeam: team_t,
    pub enemyTeam: team_t,
    pub leader: *mut gentity_t,
    pub NPC_class: class_t,

    // FIXME: could combine these
    /// How close ents have to be to pick you up as an enemy
    pub hiddenDist: f32,
    /// Normalized direction in which NPCs can't see you (you are hidden)
    pub hiddenDir: vec3_t,

    pub renderInfo: renderInfo_t,

    // dismember tracker
    pub dismembered: bool,
    /// probability of the legs being dismembered (located in NPC.cfg, 0 = never, 100 = always)
    pub dismemberProbLegs: c_char,
    /// probability of the head being dismembered (located in NPC.cfg, 0 = never, 100 = always)
    pub dismemberProbHead: c_char,
    /// probability of the arms being dismembered (located in NPC.cfg, 0 = never, 100 = always)
    pub dismemberProbArms: c_char,
    /// probability of the hands being dismembered (located in NPC.cfg, 0 = never, 100 = always)
    pub dismemberProbHands: c_char,
    /// probability of the waist being dismembered (located in NPC.cfg, 0 = never, 100 = always)
    pub dismemberProbWaist: c_char,

    pub standheight: c_int,
    pub crouchheight: c_int,
    /// Amount of poison damage to be given
    pub poisonDamage: c_int,
    /// When to apply poison damage
    pub poisonTime: c_int,
    /// debouncer for slope-foot-height-diff calcing
    pub slopeRecalcTime: c_int,

    pub pushVec: vec3_t,
    pub pushVecTime: c_int,

    /// don't do ragdoll stuff if > level.time
    pub noRagTime: c_int,
    pub isRagging: qboolean,
    /// dragging body or doing something else to override one or more ragdoll effector's/pcj's
    pub overridingBones: c_int,

    /// keeping track of positions between rags while dragging corpses
    pub ragLastOrigin: vec3_t,
    pub ragLastOriginTime: c_int,

    // push refraction effect vars
    pub pushEffectFadeTime: c_int,
    pub pushEffectOrigin: vec3_t,

    // Rocket locking vars for non-player clients (only Vehicles use these right now...)
    pub rocketLockIndex: c_int,
    pub rocketLastValidTime: f32,
    pub rocketLockTime: f32,
    pub rocketTargetTime: f32,

    // for trigger_space brushes
    pub inSpaceSuffocation: c_int,
    pub inSpaceIndex: c_int,
}

const _: () = assert!(core::mem::size_of::<gclient_t>() == 7384);
const _: () = assert!(core::mem::offset_of!(gclient_t, ps) == 0);
const _: () = assert!(core::mem::offset_of!(gclient_t, pers) == 4992);
const _: () = assert!(core::mem::offset_of!(gclient_t, sess) == 5120);
const _: () = assert!(core::mem::offset_of!(gclient_t, lastCmdTime) == 6156);
const _: () = assert!(core::mem::offset_of!(gclient_t, usercmd) == 6160);
const _: () = assert!(core::mem::offset_of!(gclient_t, buttons) == 6188);
const _: () = assert!(core::mem::offset_of!(gclient_t, oldbuttons) == 6192);
const _: () = assert!(core::mem::offset_of!(gclient_t, latched_buttons) == 6196);
const _: () = assert!(core::mem::offset_of!(gclient_t, damage_armor) == 6200);
const _: () = assert!(core::mem::offset_of!(gclient_t, damage_blood) == 6204);
const _: () = assert!(core::mem::offset_of!(gclient_t, damage_from) == 6208);
const _: () = assert!(core::mem::offset_of!(gclient_t, damage_fromWorld) == 6220);
const _: () = assert!(core::mem::offset_of!(gclient_t, noclip) == 6221);
const _: () = assert!(core::mem::offset_of!(gclient_t, forced_forwardmove) == 6222);
const _: () = assert!(core::mem::offset_of!(gclient_t, forced_rightmove) == 6223);
const _: () = assert!(core::mem::offset_of!(gclient_t, respawnTime) == 6224);
const _: () = assert!(core::mem::offset_of!(gclient_t, idleTime) == 6228);
const _: () = assert!(core::mem::offset_of!(gclient_t, airOutTime) == 6232);
const _: () = assert!(core::mem::offset_of!(gclient_t, timeResidual) == 6236);
const _: () = assert!(core::mem::offset_of!(gclient_t, facial_blink) == 6240);
const _: () = assert!(core::mem::offset_of!(gclient_t, facial_timer) == 6244);
const _: () = assert!(core::mem::offset_of!(gclient_t, facial_anim) == 6248);
const _: () = assert!(core::mem::offset_of!(gclient_t, clientInfo) == 6256);
const _: () = assert!(core::mem::offset_of!(gclient_t, moveType) == 6752);
const _: () = assert!(core::mem::offset_of!(gclient_t, jetPackTime) == 6756);
const _: () = assert!(core::mem::offset_of!(gclient_t, fireDelay) == 6760);
const _: () = assert!(core::mem::offset_of!(gclient_t, breathPuffTime) == 6764);
const _: () = assert!(core::mem::offset_of!(gclient_t, playerTeam) == 6768);
const _: () = assert!(core::mem::offset_of!(gclient_t, enemyTeam) == 6772);
const _: () = assert!(core::mem::offset_of!(gclient_t, leader) == 6776);
const _: () = assert!(core::mem::offset_of!(gclient_t, NPC_class) == 6784);
const _: () = assert!(core::mem::offset_of!(gclient_t, hiddenDist) == 6788);
const _: () = assert!(core::mem::offset_of!(gclient_t, hiddenDir) == 6792);
const _: () = assert!(core::mem::offset_of!(gclient_t, renderInfo) == 6804);
const _: () = assert!(core::mem::offset_of!(gclient_t, dismembered) == 7272);
const _: () = assert!(core::mem::offset_of!(gclient_t, dismemberProbLegs) == 7273);
const _: () = assert!(core::mem::offset_of!(gclient_t, dismemberProbHead) == 7274);
const _: () = assert!(core::mem::offset_of!(gclient_t, dismemberProbArms) == 7275);
const _: () = assert!(core::mem::offset_of!(gclient_t, dismemberProbHands) == 7276);
const _: () = assert!(core::mem::offset_of!(gclient_t, dismemberProbWaist) == 7277);
const _: () = assert!(core::mem::offset_of!(gclient_t, standheight) == 7280);
const _: () = assert!(core::mem::offset_of!(gclient_t, crouchheight) == 7284);
const _: () = assert!(core::mem::offset_of!(gclient_t, poisonDamage) == 7288);
const _: () = assert!(core::mem::offset_of!(gclient_t, poisonTime) == 7292);
const _: () = assert!(core::mem::offset_of!(gclient_t, slopeRecalcTime) == 7296);
const _: () = assert!(core::mem::offset_of!(gclient_t, pushVec) == 7300);
const _: () = assert!(core::mem::offset_of!(gclient_t, pushVecTime) == 7312);
const _: () = assert!(core::mem::offset_of!(gclient_t, noRagTime) == 7316);
const _: () = assert!(core::mem::offset_of!(gclient_t, isRagging) == 7320);
const _: () = assert!(core::mem::offset_of!(gclient_t, overridingBones) == 7324);
const _: () = assert!(core::mem::offset_of!(gclient_t, ragLastOrigin) == 7328);
const _: () = assert!(core::mem::offset_of!(gclient_t, ragLastOriginTime) == 7340);
const _: () = assert!(core::mem::offset_of!(gclient_t, pushEffectFadeTime) == 7344);
const _: () = assert!(core::mem::offset_of!(gclient_t, pushEffectOrigin) == 7348);
const _: () = assert!(core::mem::offset_of!(gclient_t, rocketLockIndex) == 7360);
const _: () = assert!(core::mem::offset_of!(gclient_t, rocketLastValidTime) == 7364);
const _: () = assert!(core::mem::offset_of!(gclient_t, rocketLockTime) == 7368);
const _: () = assert!(core::mem::offset_of!(gclient_t, rocketTargetTime) == 7372);
const _: () = assert!(core::mem::offset_of!(gclient_t, inSpaceSuffocation) == 7376);
const _: () = assert!(core::mem::offset_of!(gclient_t, inSpaceIndex) == 7380);
