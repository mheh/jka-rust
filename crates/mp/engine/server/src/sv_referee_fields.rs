//! Field-name attribution for the engine referee's divergence reports.
//!
//! NEW engine tooling (not a Raven port), shared vocabulary with the in-repo
//! mock referee (`crates/jampgame/tests/referee.rs`): exhaustive
//! `(offset, name)` tables over the snapshot types, built with
//! `core::mem::offset_of!` so they are checked against the real `#[repr(C)]`
//! layouts at compile time. A divergent byte offset maps to `"field+N"`, or
//! `"parent.sub+N"` when it lands inside a nested struct (`fd`, `pos`/`apos`).

use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::common::mp::qcommon::player_state::{forcedata_t, playerState_t};
use mp_qshared::shared::trajectory::trajectory_t;

/// `&[(offset, name)]` in declaration (ascending offset) order.
macro_rules! field_offsets {
    ($ty:ty; $($f:ident),+ $(,)?) => {
        &[ $( (core::mem::offset_of!($ty, $f), stringify!($f)) ),+ ]
    };
}

pub struct FieldTable {
    kind: &'static str,
    fields: &'static [(usize, &'static str)],
    /// Total `size_of` — pins the table's owning type at compile time.
    #[allow(dead_code)]
    size: usize,
}

pub const PS: FieldTable = FieldTable {
    kind: "playerState_t",
    size: core::mem::size_of::<playerState_t>(),
    fields: field_offsets!(playerState_t;
        commandTime, pm_type, bobCycle, pm_flags, pm_time, origin, velocity, moveDir,
        weaponTime, weaponChargeTime, weaponChargeSubtractTime, gravity, speed, basespeed,
        delta_angles, slopeRecalcTime, useTime, groundEntityNum, legsTimer, legsAnim, torsoTimer,
        torsoAnim, legsFlip, torsoFlip, movementDir, eFlags, eFlags2, eventSequence, events,
        eventParms, externalEvent, externalEventParm, externalEventTime, clientNum, weapon,
        weaponstate, viewangles, viewheight, damageEvent, damageYaw, damagePitch, damageCount,
        damageType, painTime, painDirection, yawAngle, yawing, pitchAngle, pitching, stats,
        persistant, powerups, ammo, generic1, loopSound, jumppad_ent, ping, pmove_framecount,
        jumppad_frame, entityEventSequence, lastOnGround, saberInFlight, saberMove, saberBlocking,
        saberBlocked, saberLockTime, saberLockEnemy, saberLockFrame, saberLockHits,
        saberLockHitCheckTime, saberLockHitIncrementTime, saberLockAdvance, saberEntityNum,
        saberEntityDist, saberEntityState, saberThrowDelay, saberCanThrow, saberDidThrowTime,
        saberDamageDebounceTime, saberHitWallSoundDebounceTime, saberEventFlags, rocketLockIndex,
        rocketLastValidTime, rocketLockTime, rocketTargetTime, emplacedIndex, emplacedTime,
        isJediMaster, forceRestricted, trueJedi, trueNonJedi, saberIndex, genericEnemyIndex,
        droneFireTime, droneExistTime, activeForcePass, hasDetPackPlanted, holocronsCarried,
        holocronCantTouch, holocronCantTouchTime, holocronBits, electrifyTime, saberAttackSequence,
        saberIdleWound, saberAttackWound, saberBlockTime, otherKiller, otherKillerTime,
        otherKillerDebounceTime, fd, forceJumpFlip, forceHandExtend, forceHandExtendTime,
        forceRageDrainTime, forceDodgeAnim, quickerGetup, groundTime, footstepTime, otherSoundTime,
        otherSoundLen, forceGripMoveInterval, forceGripChangeMovetype, forceKickFlip, duelIndex,
        duelTime, duelInProgress, saberAttackChainCount, saberHolstered, forceAllowDeactivateTime,
        zoomMode, zoomTime, zoomLocked, zoomFov, zoomLockTime, fallingToDeath, useDelay, inAirAnim,
        lastHitLoc, heldByClient, ragAttach, iModelScale, brokenLimbs, hasLookTarget, lookTarget,
        customRGBA, standheight, crouchheight, m_iVehicleNum, vehOrientation, vehBoarding,
        vehSurfaces, vehTurnaroundIndex, vehTurnaroundTime, vehWeaponsLinked, hyperSpaceTime,
        hyperSpaceAngles, hackingTime, hackingBaseTime, jetpackFuel, cloakFuel, userInt1, userInt2,
        userInt3, userFloat1, userFloat2, userFloat3, userVec1, userVec2,
    ),
};

pub const FD: FieldTable = FieldTable {
    kind: "forcedata_t",
    size: core::mem::size_of::<forcedata_t>(),
    fields: field_offsets!(forcedata_t;
        forcePowerDebounce, forcePowersKnown, forcePowersActive, forcePowerSelected,
        forceButtonNeedRelease, forcePowerDuration, forcePower, forcePowerMax,
        forcePowerRegenDebounceTime, forcePowerLevel, forcePowerBaseLevel, forceUsingAdded,
        forceJumpZStart, forceJumpCharge, forceJumpSound, forceJumpAddTime, forceGripEntityNum,
        forceGripDamageDebounceTime, forceGripBeingGripped, forceGripCripple, forceGripUseTime,
        forceGripSoundTime, forceGripStarted, forceHealTime, forceHealAmount,
        forceMindtrickTargetIndex, forceMindtrickTargetIndex2, forceMindtrickTargetIndex3,
        forceMindtrickTargetIndex4, forceRageRecoveryTime, forceDrainEntNum, forceDrainTime,
        forceDoInit, forceSide, forceRank, forceDeactivateAll, killSoundEntIndex, sentryDeployed,
        saberAnimLevelBase, saberAnimLevel, saberDrawAnimLevel, suicides, privateDuelTime,
    ),
};

pub const ES: FieldTable = FieldTable {
    kind: "entityState_t",
    size: core::mem::size_of::<entityState_t>(),
    fields: field_offsets!(entityState_t;
        number, eType, eFlags, eFlags2, pos, apos, time, time2, origin, origin2, angles, angles2,
        bolt1, bolt2, trickedentindex, trickedentindex2, trickedentindex3, trickedentindex4, speed,
        fireflag, genericenemyindex, activeForcePass, emplacedOwner, otherEntityNum, otherEntityNum2,
        groundEntityNum, constantLight, loopSound, loopIsSoundset, soundSetIndex, modelGhoul2,
        g2radius, modelindex, modelindex2, clientNum, frame, saberInFlight, saberEntityNum,
        saberMove, forcePowersActive, saberHolstered, isJediMaster, isPortalEnt, solid, event,
        eventParm, owner, teamowner, shouldtarget, powerups, weapon, legsAnim, torsoAnim, legsFlip,
        torsoFlip, forceFrame, generic1, heldByClient, ragAttach, iModelScale, brokenLimbs,
        boltToPlayer, hasLookTarget, lookTarget, customRGBA, health, maxhealth, npcSaber1, npcSaber2,
        csSounds_Std, csSounds_Combat, csSounds_Extra, csSounds_Jedi, surfacesOn, surfacesOff,
        boneIndex1, boneIndex2, boneIndex3, boneIndex4, boneOrient, boneAngles1, boneAngles2,
        boneAngles3, boneAngles4, NPC_class, m_iVehicleNum, userInt1, userInt2, userInt3, userFloat1,
        userFloat2, userFloat3, userVec1, userVec2,
    ),
};

pub const TR: FieldTable = FieldTable {
    kind: "trajectory_t",
    size: core::mem::size_of::<trajectory_t>(),
    fields: field_offsets!(trajectory_t; trType, trTime, trDuration, trBase, trDelta),
};

/// The field whose byte range contains `off` (last field with offset <= off).
fn field_at(t: &FieldTable, off: usize) -> (usize, &'static str) {
    let mut best = t.fields[0];
    for &e in t.fields {
        if e.0 <= off {
            best = e;
        } else {
            break;
        }
    }
    best
}

/// `"field+N"` (or `"parent.sub+N"` when the byte lands inside a nested struct).
pub fn describe(t: &FieldTable, off: usize) -> String {
    let (start, name) = field_at(t, off);
    let inner = off - start;
    let nested = match (t.kind, name) {
        ("playerState_t", "fd") => Some(&FD),
        ("entityState_t", "pos") | ("entityState_t", "apos") => Some(&TR),
        _ => None,
    };
    if let Some(sub) = nested {
        let (ss, sn) = field_at(sub, inner);
        format!("{name}.{sn}+{}", inner - ss)
    } else {
        format!("{name}+{inner}")
    }
}
