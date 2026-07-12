//! SP `playerState_t` copied from Raven `code/game/q_shared.h`.
//!
//! Source: `oracle/code/game/q_shared.h:2066-2361`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use crate::shared::{qboolean, vec3_t, waterHeightLevel_t, NUM_FORCE_POWERS};

use super::saber::{saberInfo_t, MAX_SABERS};

/// Raven SP `MAX_STATS`.
///
/// Type definition source: `oracle/code/game/q_shared.h:1582`
pub const MAX_STATS: usize = 16;
/// Raven SP `MAX_PERSISTANT`.
///
/// Type definition source: `oracle/code/game/q_shared.h:1586`
pub const MAX_PERSISTANT: usize = 16;
/// Raven SP `MAX_POWERUPS`.
///
/// Type definition source: `oracle/code/game/q_shared.h:1589`
pub const MAX_POWERUPS: usize = 16;
/// Raven SP `MAX_AMMO`.
///
/// Type definition source: `oracle/code/game/q_shared.h:1591`
pub const MAX_AMMO: usize = 10;
/// Raven SP `MAX_INVENTORY`.
///
/// Raven: See INV_MAX.
/// Type definition source: `oracle/code/game/q_shared.h:1592`
pub const MAX_INVENTORY: usize = 15;
/// Raven SP `MAX_SECURITY_KEYS`.
///
/// Type definition source: `oracle/code/game/q_shared.h:1593`
pub const MAX_SECURITY_KEYS: usize = 5;
/// Raven SP `MAX_SECURITY_KEY_MESSSAGE`.
///
/// Type definition source: `oracle/code/game/q_shared.h:1594`
pub const MAX_SECURITY_KEY_MESSSAGE: usize = 24;
/// Raven SP `MAX_PS_EVENTS`.
///
/// Raven: this must be a power of 2 unless you change some &'s to %'s -ste
/// Type definition source: `oracle/code/game/q_shared.h:1596`
pub const MAX_PS_EVENTS: usize = 2;

/// Raven SP `playerState_t` — full client/server prediction state for a player.
///
/// Raven: playerState_t is the information needed by both the client and server
/// to predict player motion and actions; nothing outside of pmove should modify
/// these, or some degree of prediction error will occur. It is a full superset
/// of entityState_t as it is used by players, so if a playerState_t is
/// transmitted, the entityState_t can be fully derived from it.
/// !!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!
///
/// C++ helper methods interspersed among the fields in Raven's source (e.g.
/// `SaberStaff`, `SaberActivate`, `SaberDisarmBonus`) do not affect layout and
/// are not ported; only the data fields are.
/// Type definition source: `oracle/code/game/q_shared.h:2077-2361`
// Note: no `PartialEq` derive — `saberInfo_t` (embedded via `saber`) holds raw
// pointers and does not itself derive `PartialEq`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct playerState_t {
    /// cmd->serverTime of last executed command.
    pub commandTime: c_int,
    pub pm_type: c_int,
    /// For view bobbing and footstep generation.
    pub bobCycle: c_int,
    /// Ducked, jump_held, etc.
    pub pm_flags: c_int,
    pub pm_time: c_int,

    pub origin: vec3_t,
    pub velocity: vec3_t,
    pub weaponTime: c_int,
    pub weaponChargeTime: c_int,
    /// For the phaser.
    pub rechargeTime: c_int,
    pub gravity: c_int,
    pub leanofs: c_int,
    pub friction: c_int,
    pub speed: c_int,
    /// Add to command angles to get view direction, changed by spawns,
    /// rotating objects, and teleporters.
    pub delta_angles: [c_int; 3],

    /// ENTITYNUM_NONE = in air.
    pub groundEntityNum: c_int,
    pub legsAnim: c_int,
    /// Don't change low priority animations on legs until this runs out.
    pub legsAnimTimer: c_int,
    pub torsoAnim: c_int,
    /// Don't change low priority animations on torso until this runs out.
    pub torsoAnimTimer: c_int,
    /// A number 0 to 7 that represents the relative angle of movement to the
    /// view angle (axial and diagonals); when at rest, the value will remain
    /// unchanged. Used to twist the legs during strafing.
    pub movementDir: c_int,

    /// Copied to entityState_t->eFlags.
    pub eFlags: c_int,

    /// Pmove generated events.
    pub eventSequence: c_int,
    pub events: [c_int; MAX_PS_EVENTS],
    pub eventParms: [c_int; MAX_PS_EVENTS],

    /// Events set on player from another source.
    pub externalEvent: c_int,
    pub externalEventParm: c_int,
    pub externalEventTime: c_int,

    /// Ranges from 0 to MAX_CLIENTS-1.
    pub clientNum: c_int,
    /// Copied to entityState_t->weapon.
    pub weapon: c_int,
    pub weaponstate: c_int,

    pub batteryCharge: c_int,

    /// For fixed views.
    pub viewangles: vec3_t,
    /// Actual legs forward facing.
    pub legsYaw: f32,
    pub viewheight: c_int,

    // damage feedback
    /// When it changes, latch the other parms.
    pub damageEvent: c_int,
    pub damageYaw: c_int,
    pub damagePitch: c_int,
    pub damageCount: c_int,

    pub stats: [c_int; MAX_STATS],
    /// Stats that aren't cleared on death.
    pub persistant: [c_int; MAX_PERSISTANT],
    /// level.time that the powerup runs out.
    pub powerups: [c_int; MAX_POWERUPS],
    pub ammo: [c_int; MAX_AMMO],
    /// Count of each inventory item.
    pub inventory: [c_int; MAX_INVENTORY],
    /// Security key types.
    pub security_key_message: [[u8; MAX_SECURITY_KEY_MESSSAGE]; MAX_SECURITY_KEYS],

    pub serverViewOrg: vec3_t,

    pub saberInFlight: qboolean,

    /// For overriding player movement controls and vieworg.
    pub viewEntity: c_int,
    /// Prediction needs to know this.
    pub forcePowersActive: c_int,

    /// Not sent.
    pub useTime: c_int,
    /// Last time you shot your weapon.
    pub lastShotTime: c_int,
    /// Server to game info for scoreboard.
    pub ping: c_int,
    /// Last time you were on the ground.
    pub lastOnGround: c_int,
    /// Last time you were on the ground.
    pub lastStationary: c_int,
    pub weaponShotCount: c_int,

    // FIXME: maybe allocate all these structures (saber, force powers, vehicles)
    // or descend them as classes - so not every client has all this info
    pub saber: [saberInfo_t; MAX_SABERS],
    pub dualSabers: qboolean,

    pub saberMove: i16,
    pub saberMoveNext: i16,
    pub saberBounceMove: i16,
    pub saberBlocking: i16,
    pub saberBlocked: i16,
    pub leanStopDebounceTime: i16,

    pub saberEntityNum: c_int,
    pub saberEntityDist: f32,
    pub saberThrowTime: c_int,
    pub saberEntityState: c_int,
    pub saberDamageDebounceTime: c_int,
    pub saberHitWallSoundDebounceTime: c_int,
    pub saberEventFlags: c_int,
    pub saberBlockingTime: c_int,
    pub saberAnimLevel: c_int,
    pub saberAttackChainCount: c_int,
    pub saberLockTime: c_int,
    pub saberLockEnemy: c_int,
    pub saberStylesKnown: c_int,

    pub forcePowersKnown: c_int,
    /// For effects that have a duration.
    pub forcePowerDuration: [c_int; NUM_FORCE_POWERS as usize],
    /// For effects that must have an interval.
    pub forcePowerDebounce: [c_int; NUM_FORCE_POWERS as usize],
    pub forcePower: c_int,
    pub forcePowerMax: c_int,
    pub forcePowerRegenDebounceTime: c_int,
    /// Default is 100ms.
    pub forcePowerRegenRate: c_int,
    /// Default is 1.
    pub forcePowerRegenAmount: c_int,
    /// So we know the max forceJump power you have.
    pub forcePowerLevel: [c_int; NUM_FORCE_POWERS as usize],
    /// So when you land, you don't get hurt as much.
    pub forceJumpZStart: f32,
    /// You're current forceJump charge-up level, increases the longer you hold
    /// the force jump button down.
    pub forceJumpCharge: f32,
    /// What entity I'm gripping.
    pub forceGripEntityNum: c_int,
    /// Where the gripped ent should be lifted to.
    pub forceGripOrg: vec3_t,
    /// What entity I'm draining.
    pub forceDrainEntityNum: c_int,
    /// Where the drained ent should be lifted to.
    pub forceDrainOrg: vec3_t,
    /// How many points of force heal have been applied so far.
    pub forceHealCount: c_int,

    // new Jedi Academy force powers
    pub forceAllowDeactivateTime: c_int,
    pub forceRageDrainTime: c_int,
    pub forceRageRecoveryTime: c_int,
    pub forceDrainEntNum: c_int,
    pub forceDrainTime: f32,
    /// Client is being forced to use these powers (FIXME: and only these?).
    pub forcePowersForced: c_int,
    pub pullAttackEntNum: c_int,
    pub pullAttackTime: c_int,
    pub lastKickedEntNum: c_int,

    /// Replaced BUTTON_GESTURE.
    pub taunting: c_int,

    /// So when you land, you don't get hurt as much.
    pub jumpZStart: f32,
    pub moveDir: vec3_t,

    /// Exactly what the z org of the water is (will be +4 above if under
    /// water, -4 below if not in water).
    pub waterheight: f32,
    /// How high it really is.
    pub waterHeightLevel: waterHeightLevel_t,

    // testing IK grabbing
    /// For IK.
    pub ikStatus: qboolean,
    /// For IK, who I'm grabbing, if anyone.
    pub heldClient: c_int,
    /// For IK, someone is grabbing me.
    pub heldByClient: c_int,
    /// For IK, what bolt I'm attached to on the holder someone is grabbing me by.
    pub heldByBolt: c_int,
    /// For IK, what bone I'm being held by.
    pub heldByBone: c_int,

    // vehicle turn-around stuff... FIXME: only vehicles need this in SP...
    pub vehTurnaroundIndex: c_int,
    pub vehTurnaroundTime: c_int,

    // NOTE: not really used in SP, just for Fighter Vehicle damage stuff
    pub brokenLimbs: c_int,
    pub electrifyTime: c_int,
}

const _: () = assert!(core::mem::size_of::<playerState_t>() == 4992);
const _: () = assert!(core::mem::offset_of!(playerState_t, commandTime) == 0);
const _: () = assert!(core::mem::offset_of!(playerState_t, pm_type) == 4);
const _: () = assert!(core::mem::offset_of!(playerState_t, bobCycle) == 8);
const _: () = assert!(core::mem::offset_of!(playerState_t, pm_flags) == 12);
const _: () = assert!(core::mem::offset_of!(playerState_t, pm_time) == 16);
const _: () = assert!(core::mem::offset_of!(playerState_t, origin) == 20);
const _: () = assert!(core::mem::offset_of!(playerState_t, velocity) == 32);
const _: () = assert!(core::mem::offset_of!(playerState_t, weaponTime) == 44);
const _: () = assert!(core::mem::offset_of!(playerState_t, weaponChargeTime) == 48);
const _: () = assert!(core::mem::offset_of!(playerState_t, rechargeTime) == 52);
const _: () = assert!(core::mem::offset_of!(playerState_t, gravity) == 56);
const _: () = assert!(core::mem::offset_of!(playerState_t, leanofs) == 60);
const _: () = assert!(core::mem::offset_of!(playerState_t, friction) == 64);
const _: () = assert!(core::mem::offset_of!(playerState_t, speed) == 68);
const _: () = assert!(core::mem::offset_of!(playerState_t, delta_angles) == 72);
const _: () = assert!(core::mem::offset_of!(playerState_t, groundEntityNum) == 84);
const _: () = assert!(core::mem::offset_of!(playerState_t, legsAnim) == 88);
const _: () = assert!(core::mem::offset_of!(playerState_t, legsAnimTimer) == 92);
const _: () = assert!(core::mem::offset_of!(playerState_t, torsoAnim) == 96);
const _: () = assert!(core::mem::offset_of!(playerState_t, torsoAnimTimer) == 100);
const _: () = assert!(core::mem::offset_of!(playerState_t, movementDir) == 104);
const _: () = assert!(core::mem::offset_of!(playerState_t, eFlags) == 108);
const _: () = assert!(core::mem::offset_of!(playerState_t, eventSequence) == 112);
const _: () = assert!(core::mem::offset_of!(playerState_t, events) == 116);
const _: () = assert!(core::mem::offset_of!(playerState_t, eventParms) == 124);
const _: () = assert!(core::mem::offset_of!(playerState_t, externalEvent) == 132);
const _: () = assert!(core::mem::offset_of!(playerState_t, externalEventParm) == 136);
const _: () = assert!(core::mem::offset_of!(playerState_t, externalEventTime) == 140);
const _: () = assert!(core::mem::offset_of!(playerState_t, clientNum) == 144);
const _: () = assert!(core::mem::offset_of!(playerState_t, weapon) == 148);
const _: () = assert!(core::mem::offset_of!(playerState_t, weaponstate) == 152);
const _: () = assert!(core::mem::offset_of!(playerState_t, batteryCharge) == 156);
const _: () = assert!(core::mem::offset_of!(playerState_t, viewangles) == 160);
const _: () = assert!(core::mem::offset_of!(playerState_t, legsYaw) == 172);
const _: () = assert!(core::mem::offset_of!(playerState_t, viewheight) == 176);
const _: () = assert!(core::mem::offset_of!(playerState_t, damageEvent) == 180);
const _: () = assert!(core::mem::offset_of!(playerState_t, damageYaw) == 184);
const _: () = assert!(core::mem::offset_of!(playerState_t, damagePitch) == 188);
const _: () = assert!(core::mem::offset_of!(playerState_t, damageCount) == 192);
const _: () = assert!(core::mem::offset_of!(playerState_t, stats) == 196);
const _: () = assert!(core::mem::offset_of!(playerState_t, persistant) == 260);
const _: () = assert!(core::mem::offset_of!(playerState_t, powerups) == 324);
const _: () = assert!(core::mem::offset_of!(playerState_t, ammo) == 388);
const _: () = assert!(core::mem::offset_of!(playerState_t, inventory) == 428);
const _: () = assert!(core::mem::offset_of!(playerState_t, security_key_message) == 488);
const _: () = assert!(core::mem::offset_of!(playerState_t, serverViewOrg) == 608);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberInFlight) == 620);
const _: () = assert!(core::mem::offset_of!(playerState_t, viewEntity) == 624);
const _: () = assert!(core::mem::offset_of!(playerState_t, forcePowersActive) == 628);
const _: () = assert!(core::mem::offset_of!(playerState_t, useTime) == 632);
const _: () = assert!(core::mem::offset_of!(playerState_t, lastShotTime) == 636);
const _: () = assert!(core::mem::offset_of!(playerState_t, ping) == 640);
const _: () = assert!(core::mem::offset_of!(playerState_t, lastOnGround) == 644);
const _: () = assert!(core::mem::offset_of!(playerState_t, lastStationary) == 648);
const _: () = assert!(core::mem::offset_of!(playerState_t, weaponShotCount) == 652);
const _: () = assert!(core::mem::offset_of!(playerState_t, saber) == 656);
const _: () = assert!(core::mem::offset_of!(playerState_t, dualSabers) == 4560);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberMove) == 4564);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberMoveNext) == 4566);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberBounceMove) == 4568);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberBlocking) == 4570);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberBlocked) == 4572);
const _: () = assert!(core::mem::offset_of!(playerState_t, leanStopDebounceTime) == 4574);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberEntityNum) == 4576);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberEntityDist) == 4580);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberThrowTime) == 4584);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberEntityState) == 4588);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberDamageDebounceTime) == 4592);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberHitWallSoundDebounceTime) == 4596);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberEventFlags) == 4600);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberBlockingTime) == 4604);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberAnimLevel) == 4608);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberAttackChainCount) == 4612);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberLockTime) == 4616);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberLockEnemy) == 4620);
const _: () = assert!(core::mem::offset_of!(playerState_t, saberStylesKnown) == 4624);
const _: () = assert!(core::mem::offset_of!(playerState_t, forcePowersKnown) == 4628);
const _: () = assert!(core::mem::offset_of!(playerState_t, forcePowerDuration) == 4632);
const _: () = assert!(core::mem::offset_of!(playerState_t, forcePowerDebounce) == 4696);
const _: () = assert!(core::mem::offset_of!(playerState_t, forcePower) == 4760);
const _: () = assert!(core::mem::offset_of!(playerState_t, forcePowerMax) == 4764);
const _: () = assert!(core::mem::offset_of!(playerState_t, forcePowerRegenDebounceTime) == 4768);
const _: () = assert!(core::mem::offset_of!(playerState_t, forcePowerRegenRate) == 4772);
const _: () = assert!(core::mem::offset_of!(playerState_t, forcePowerRegenAmount) == 4776);
const _: () = assert!(core::mem::offset_of!(playerState_t, forcePowerLevel) == 4780);
const _: () = assert!(core::mem::offset_of!(playerState_t, forceJumpZStart) == 4844);
const _: () = assert!(core::mem::offset_of!(playerState_t, forceJumpCharge) == 4848);
const _: () = assert!(core::mem::offset_of!(playerState_t, forceGripEntityNum) == 4852);
const _: () = assert!(core::mem::offset_of!(playerState_t, forceGripOrg) == 4856);
const _: () = assert!(core::mem::offset_of!(playerState_t, forceDrainEntityNum) == 4868);
const _: () = assert!(core::mem::offset_of!(playerState_t, forceDrainOrg) == 4872);
const _: () = assert!(core::mem::offset_of!(playerState_t, forceHealCount) == 4884);
const _: () = assert!(core::mem::offset_of!(playerState_t, forceAllowDeactivateTime) == 4888);
const _: () = assert!(core::mem::offset_of!(playerState_t, forceRageDrainTime) == 4892);
const _: () = assert!(core::mem::offset_of!(playerState_t, forceRageRecoveryTime) == 4896);
const _: () = assert!(core::mem::offset_of!(playerState_t, forceDrainEntNum) == 4900);
const _: () = assert!(core::mem::offset_of!(playerState_t, forceDrainTime) == 4904);
const _: () = assert!(core::mem::offset_of!(playerState_t, forcePowersForced) == 4908);
const _: () = assert!(core::mem::offset_of!(playerState_t, pullAttackEntNum) == 4912);
const _: () = assert!(core::mem::offset_of!(playerState_t, pullAttackTime) == 4916);
const _: () = assert!(core::mem::offset_of!(playerState_t, lastKickedEntNum) == 4920);
const _: () = assert!(core::mem::offset_of!(playerState_t, taunting) == 4924);
const _: () = assert!(core::mem::offset_of!(playerState_t, jumpZStart) == 4928);
const _: () = assert!(core::mem::offset_of!(playerState_t, moveDir) == 4932);
const _: () = assert!(core::mem::offset_of!(playerState_t, waterheight) == 4944);
const _: () = assert!(core::mem::offset_of!(playerState_t, waterHeightLevel) == 4948);
const _: () = assert!(core::mem::offset_of!(playerState_t, ikStatus) == 4952);
const _: () = assert!(core::mem::offset_of!(playerState_t, heldClient) == 4956);
const _: () = assert!(core::mem::offset_of!(playerState_t, heldByClient) == 4960);
const _: () = assert!(core::mem::offset_of!(playerState_t, heldByBolt) == 4964);
const _: () = assert!(core::mem::offset_of!(playerState_t, heldByBone) == 4968);
const _: () = assert!(core::mem::offset_of!(playerState_t, vehTurnaroundIndex) == 4972);
const _: () = assert!(core::mem::offset_of!(playerState_t, vehTurnaroundTime) == 4976);
const _: () = assert!(core::mem::offset_of!(playerState_t, brokenLimbs) == 4980);
const _: () = assert!(core::mem::offset_of!(playerState_t, electrifyTime) == 4984);
