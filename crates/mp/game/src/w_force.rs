//! Port of `oracle/oracle/codemp/game/w_force.c` (jampgame force-power logic).
//!
//! Generated from `tools/closure-prototype/fnskel.py`; bodies filled per the
//! jampgame mega-pass (settled fork rulings,
//! `docs/handoffs/jampgame-fork-discovery.md`).
//!
//! SPINE (fork rulings 1/4 + `docs/architecture/engine-seam.md`): logic fns that
//! reach `level`/cvars/`g_entities`/traps thread the `GameContext<'_>` receiver
//! (`.world: *mut GameWorld`, `.engine`) — the only ported-logic precedent
//! (`g_init_game`). Globals are `GameWorld` fields (fork 1): `level` →
//! `(*ctx.world).level`, cvars → `(*ctx.world).cvars`, `g_entities[i]` →
//! `(*ctx.world).entities[i]`. Traps go through `trap::X(ctx.engine, …)`.
//! Cross-file callees are invoked with the packet's resolved raw-pointer
//! signatures verbatim (their own porters thread the spine).
//!
//! Raw `gentity_t*`/`gclient_t*`/`playerState_t*` chains are transcribed as
//! `unsafe` raw-pointer field access mirroring the C exactly (the fnskel
//! skeletons operate in raw-pointer space; `GameContext.world` is itself a raw
//! pointer). EntityId reshaping (fork 4) lands in the later integration pass.
//!
//! NOTE (integration-deferred): the packet does not enumerate the Raven
//! constant spellings (`EV_*`, `FP_*`, `FORCE_LEVEL_*`, `PDSOUND_*`, `CHAN_*`,
//! …) nor their owning enums; they are transcribed by their faithful Raven
//! names (the port preserves them) and their exact enum-qualification / module
//! path is resolved at integration (the mega-pass tree is not compiled per
//! porter — "Do NOT run cargo"). `forcePowerNeeded` is the bg-shared const
//! table (fork 5: const tables stay const), referenced by its Raven name.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/oracle/codemp/game/q_shared.h`
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;
use crate::NPC_AI_Jedi::Jedi_Decloak;
use crate::ai_main::{InFieldOfVision, OrgVisible};
use crate::bg_misc::{BG_CanUseFPNow, BG_HasYsalamiri};
use crate::bg_panimate::{BG_InReboundHold, BG_InReboundJump, BG_FullBodyTauntAnim, BG_SaberInSpecial};
use crate::bg_pmove::BG_InKnockDown;
use crate::bg_saber::BG_ForcePowerDrain;
use crate::g_combat::G_Damage;
use crate::g_team::OnSameTeam;
use crate::g_utils::{
    G_EntitySound, G_MuteSound, G_PlayEffect, G_SetAnim, G_Sound, G_SoundAtLoc, G_SoundIndex,
    G_TempEntity, GlobalUse,
};
use crate::g_weapon::WP_FireGenericBlasterMissile;
use crate::NPC_senses::InFront;
use crate::q_math::{AngleSubtract, AngleVectors, DirToByte, Q_irand, VectorLength, VectorNormalize, vectoangles};
use crate::w_saber::HasSetSaberOnly;
use crate::trap;
use crate::world::GameContext;

// vec3 origin (`{0,0,0}`), the all-zero trace mins/maxs sentinel.
use crate::q_math::vec3_origin;

// Const/enum families transcribed by faithful Raven name (file header note).
use crate::entity::hit_location::*;
use crate::level::damage_flags::*;
use mp_abi::game::syscalls::G_CVAR_UPDATE::GCvarUpdateArgs;
use mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::public::anim_number::animNumber_t::*;
use mp_bg::public::jump_velocity::JUMP_VELOCITY;
use mp_bg::public::effect_types::effectTypes_t::*;
use mp_bg::public::weaponstate::weaponstate_t::*;
use mp_qshared::common::mp::qcommon::usercmd_button::*;

/// Raven `M_PI` (`<math.h>`), used by the seeker-drone orbit math.
const M_PI: f64 = std::f64::consts::PI;

/// Raven `PITCH`/`YAW`/`ROLL` — Euler-angle component indices.
/// Source: `oracle/oracle/codemp/game/q_shared.h`
const PITCH: usize = 0;
const YAW: usize = 1;
const ROLL: usize = 2;

// w_force.c-local `#define`s referenced below by their faithful Raven names and
// resolved at integration (same convention as the module-doc note above; their
// numeric values live in the un-ported `oracle/oracle/codemp/game/w_force.c`
// header block and `g_local.h`, not in this packet):
//TODO: Port FORCE_LIGHTNING_RADIUS   // Source: oracle/oracle/codemp/game/w_force.c
//TODO: Port MAX_DRAIN_DISTANCE       // Source: oracle/oracle/codemp/game/w_force.c
//TODO: Port MAX_TRICK_DISTANCE       // Source: oracle/oracle/codemp/game/w_force.c
//TODO: Port MASK_SHOT                // Source: oracle/oracle/codemp/game/q_shared.h
//TODO: Port MASK_PLAYERSOLID         // Source: oracle/oracle/codemp/game/q_shared.h
//TODO: Port GRIP_DRAIN_AMOUNT        // Source: oracle/oracle/codemp/game/g_local.h
//TODO: Port SVF_BOT                   // Source: oracle/oracle/codemp/game/q_shared.h
//TODO: Port FJ_FORWARD               // Source: oracle/oracle/codemp/game/w_force.c
//TODO: Port FJ_BACKWARD              // Source: oracle/oracle/codemp/game/w_force.c
//TODO: Port FJ_RIGHT                 // Source: oracle/oracle/codemp/game/w_force.c
//TODO: Port FJ_LEFT                  // Source: oracle/oracle/codemp/game/w_force.c
//TODO: Port FJ_UP                    // Source: oracle/oracle/codemp/game/w_force.c

/// Raven `PMF_FOLLOW`/`PMF_STUCK_TO_WALL` (`playerState_t::pm_flags` bits).
/// Source: `oracle/oracle/codemp/game/bg_public.h:415,417`
const PMF_FOLLOW: c_int = 4096;
const PMF_STUCK_TO_WALL: c_int = 16384;

/// Raven `SFL_TWO_HANDED` (`weaponData_t::weaponflags` bit) — uses both hands.
/// Source: `oracle/oracle/codemp/game/q_shared.h:691`
const SFL_TWO_HANDED: c_int = 1 << 4;

use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;
use mp_bg::public::entity_event::entity_event_t::{EV_FORCE_DRAINED, EV_PREDEFSOUND, EV_TEAM_POWER};

/// Raven `mindTrickTime` per force-mastery level (ms).
///
/// Source: `oracle/oracle/codemp/game/w_force.c:139-145`
pub const mindTrickTime: [c_int; 4] = [0 /*none*/, 5000, 10000, 15000];

/// Raven `G_PreDefSound` — spawn a predefined-sound temp entity at `org`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:40-49`
pub fn G_PreDefSound(ctx: GameContext<'_>, org: vec3_t, pdSound: c_int) -> *mut gentity_t {
    unsafe {
        let te = G_TempEntity(ctx, org, EV_PREDEFSOUND as c_int);
        (*te).s.eventParm = pdSound;
        (*te).s.origin = org; // VectorCopy(org, te->s.origin)
        te
    }
}

/// Raven `WP_InitForcePowers`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:147-572`
// PORT-ESCALATION(unported-global): reads the file-scope
// `forcePowerNeeded/bgSiegeClasses` table(s) — genuinely unported runtime data
// (fork-discovery ruling 1: globals -> GameWorld fields), not just a
// missing `use`.
pub fn WP_InitForcePowers(ctx: GameContext<'_>, ent: *mut gentity_t) {
    todo!("Port WP_InitForcePowers — parked: unported-global (forcePowerNeeded/bgSiegeClasses)")
}

/// Raven `WP_SpawnInitForcePowers` — reset per-spawn force state.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:574-691`
// PORT-ESCALATION(unported-global): reads the file-scope
// `forcePowerNeeded/bgSiegeClasses` table(s) — genuinely unported runtime data
// (fork-discovery ruling 1: globals -> GameWorld fields), not just a
// missing `use`.
pub fn WP_SpawnInitForcePowers(ctx: GameContext<'_>, ent: *mut gentity_t) {
    todo!("Port WP_SpawnInitForcePowers — parked: unported-global (forcePowerNeeded/bgSiegeClasses)")
}

/// Raven `ForcePowerUsableOn` — can `attacker` use `forcePower` on `other`?
///
/// Source: `oracle/oracle/codemp/game/w_force.c:697-772`
pub fn ForcePowerUsableOn(
    ctx: GameContext<'_>,
    attacker: *mut gentity_t,
    other: *mut gentity_t,
    forcePower: forcePowers_t,
) -> c_int {
    unsafe {
        let gametype = (*ctx.world).cvars.g_gametype.integer;
        let level_time = (*ctx.world).level.time;

        if !other.is_null()
            && !(*other).client.is_null()
            && BG_HasYsalamiri(gametype, &mut (*((*other).client as *mut gclient_t)).ps) != 0
        {
            return 0;
        }

        if !attacker.is_null()
            && !(*attacker).client.is_null()
            && BG_CanUseFPNow(gametype, &mut (*((*attacker).client as *mut gclient_t)).ps, level_time, forcePower) == 0
        {
            return 0;
        }

        //Dueling fighters cannot use force powers on others, with the exception of force push when locked with each other
        if !attacker.is_null()
            && !(*attacker).client.is_null()
            && (*((*attacker).client as *mut gclient_t)).ps.duelInProgress != 0
        {
            return 0;
        }

        if !other.is_null()
            && !(*other).client.is_null()
            && (*((*other).client as *mut gclient_t)).ps.duelInProgress != 0
        {
            return 0;
        }

        if forcePower == FP_GRIP {
            if !other.is_null()
                && !(*other).client.is_null()
                && (*((*other).client as *mut gclient_t)).ps.fd.forcePowersActive & (1 << FP_ABSORB) != 0
            {
                //don't allow gripping to begin with if they are absorbing
                //play sound indicating that attack was absorbed
                if (*((*other).client as *mut gclient_t)).forcePowerSoundDebounce < level_time {
                    let abSound =
                        G_PreDefSound(ctx, (*((*other).client as *mut gclient_t)).ps.origin, PDSOUND_ABSORBHIT as c_int);
                    (*abSound).s.trickedentindex = (*other).s.number;
                    (*((*other).client as *mut gclient_t)).forcePowerSoundDebounce = level_time + 400;
                }
                return 0;
            } else if !other.is_null()
                && !(*other).client.is_null()
                && (*((*other).client as *mut gclient_t)).ps.weapon == WP_SABER
                && BG_SaberInSpecial((*((*other).client as *mut gclient_t)).ps.saberMove) != 0
            {
                //don't grip person while they are in a special or some really bad things can happen.
                return 0;
            }
        }

        if !other.is_null()
            && !(*other).client.is_null()
            && (forcePower == FP_PUSH || forcePower == FP_PULL)
        {
            if BG_InKnockDown((*((*other).client as *mut gclient_t)).ps.legsAnim) != 0 {
                return 0;
            }
        }

        if !other.is_null()
            && !(*other).client.is_null()
            && (*other).s.eType == ET_NPC as c_int
            && (*other).s.NPC_class == CLASS_VEHICLE as c_int
        {
            //can't use the force on vehicles.. except lightning
            if forcePower == FP_LIGHTNING {
                return 1;
            } else {
                return 0;
            }
        }

        if !other.is_null()
            && !(*other).client.is_null()
            && (*other).s.eType == ET_NPC as c_int
            && gametype == GT_SIEGE
        {
            //can't use powers at all on npc's normally in siege...
            return 0;
        }

        1
    }
}

/// Raven `WP_ForcePowerAvailable` — is there enough force pool for `forcePower`?
///
/// Source: `oracle/oracle/codemp/game/w_force.c:774-801`
// PORT-ESCALATION(unported-global): reads the file-scope
// `forcePowerNeeded/bgSiegeClasses` table(s) — genuinely unported runtime data
// (fork-discovery ruling 1: globals -> GameWorld fields), not just a
// missing `use`.
pub fn WP_ForcePowerAvailable(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    forcePower: forcePowers_t,
    overrideAmt: c_int,
) -> qboolean {
    todo!("Port WP_ForcePowerAvailable — parked: unported-global (forcePowerNeeded/bgSiegeClasses)")
}

/// Raven `WP_ForcePowerInUse`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:803-811`
pub fn WP_ForcePowerInUse(self_: *mut gentity_t, forcePower: forcePowers_t) -> qboolean {
    unsafe {
        if (*((*self_).client as *mut gclient_t)).ps.fd.forcePowersActive & (1 << forcePower) != 0 {
            //already using this power
            return qtrue;
        }
        qfalse
    }
}

/// Raven `WP_ForcePowerUsable` — full gate on activating `forcePower`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:813-938`
pub fn WP_ForcePowerUsable(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    forcePower: forcePowers_t,
) -> qboolean {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let gametype = (*ctx.world).cvars.g_gametype.integer;
        let level_time = (*ctx.world).level.time;

        if BG_HasYsalamiri(gametype, &mut (*cl).ps) != 0 {
            return qfalse;
        }

        if (*self_).health <= 0
            || (*cl).ps.stats[STAT_HEALTH as usize] <= 0
            || (*cl).ps.eFlags & EF_DEAD != 0
        {
            return qfalse;
        }

        if (*cl).ps.pm_flags & PMF_FOLLOW != 0 {
            //specs can't use powers through people
            return qfalse;
        }
        if (*cl).sess.sessionTeam == TEAM_SPECTATOR {
            return qfalse;
        }
        if (*cl).tempSpectate >= level_time {
            return qfalse;
        }

        if BG_CanUseFPNow(gametype, &mut (*cl).ps, level_time, forcePower) == 0 {
            return qfalse;
        }

        if (*cl).ps.fd.forcePowersKnown & (1 << forcePower) == 0 {
            //don't know this power
            return qfalse;
        }

        if (*cl).ps.fd.forcePowersActive & (1 << forcePower) != 0 {
            //already using this power
            if forcePower != FP_LEVITATION {
                return qfalse;
            }
        }

        if forcePower == FP_LEVITATION && (*cl).fjDidJump != 0 {
            return qfalse;
        }

        if (*cl).ps.fd.forcePowerLevel[forcePower as usize] == 0 {
            return qfalse;
        }

        if (*ctx.world).cvars.g_debugMelee.integer != 0 {
            if (*cl).ps.pm_flags & PMF_STUCK_TO_WALL != 0 {
                //no offensive force powers when stuck to wall
                match forcePower {
                    FP_GRIP | FP_LIGHTNING | FP_DRAIN | FP_SABER_OFFENSE | FP_SABER_DEFENSE
                    | FP_SABERTHROW => return qfalse,
                    _ => {}
                }
            }
        }

        if (*cl).ps.saberHolstered == 0 {
            if (*cl).saber[0].saberFlags & SFL_TWO_HANDED != 0 {
                if (*ctx.world).cvars.g_saberRestrictForce.integer != 0 {
                    match forcePower {
                        FP_PUSH | FP_PULL | FP_TELEPATHY | FP_GRIP | FP_LIGHTNING | FP_DRAIN => {
                            return qfalse
                        }
                        _ => {}
                    }
                }
            }

            if (*cl).saber[0].saberFlags & SFL_TWO_HANDED != 0
                || ((*cl).saber[0].model[0] != 0)
            {
                //this saber requires the use of two hands OR our other hand is using an active saber too
                if (*cl).saber[0].forceRestrictions & (1 << forcePower) != 0 {
                    //this power is verboten when using this saber
                    return qfalse;
                }
            }

            if (*cl).saber[0].model[0] != 0 {
                //both sabers on
                if (*ctx.world).cvars.g_saberRestrictForce.integer != 0 {
                    match forcePower {
                        FP_PUSH | FP_PULL | FP_TELEPATHY | FP_GRIP | FP_LIGHTNING | FP_DRAIN => {
                            return qfalse
                        }
                        _ => {}
                    }
                }
                if (*cl).saber[1].forceRestrictions & (1 << forcePower) != 0 {
                    //this power is verboten when using this saber
                    return qfalse;
                }
            }
        }
        WP_ForcePowerAvailable(ctx, self_, forcePower, 0) // OVERRIDEFIXME
    }
}

/// Raven `WP_AbsorbConversion` — absorb an incoming force attack, return the
/// remaining (post-absorb) power level, or `-1` when not absorbed.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:940-997`
pub fn WP_AbsorbConversion(
    ctx: GameContext<'_>,
    attacked: *mut gentity_t,
    atdAbsLevel: c_int,
    attacker: *mut gentity_t,
    atPower: c_int,
    atPowerLevel: c_int,
    atForceSpent: c_int,
) -> c_int {
    unsafe {
        let mut getLevel;
        let mut addTot;

        if atPower != FP_LIGHTNING
            && atPower != FP_DRAIN
            && atPower != FP_GRIP
            && atPower != FP_PUSH
            && atPower != FP_PULL
        {
            //Only these powers can be absorbed
            return -1;
        }

        if atdAbsLevel == 0 {
            //looks like attacker doesn't have any absorb power
            return -1;
        }

        let atcl = (*attacked).client as *mut gclient_t;
        if (*atcl).ps.fd.forcePowersActive & (1 << FP_ABSORB) == 0 {
            //absorb is not active
            return -1;
        }

        //Subtract absorb power level from the offensive force power
        getLevel = atPowerLevel;
        getLevel -= atdAbsLevel;

        if getLevel < 0 {
            getLevel = 0;
        }

        //let the attacker absorb an amount of force used in this attack based on his level of absorb
        addTot = (atForceSpent / 3) * (*atcl).ps.fd.forcePowerLevel[FP_ABSORB as usize];

        if addTot < 1 && atForceSpent >= 1 {
            addTot = 1;
        }
        (*atcl).ps.fd.forcePower += addTot;
        if (*atcl).ps.fd.forcePower > 100 {
            (*atcl).ps.fd.forcePower = 100;
        }

        //play sound indicating that attack was absorbed
        let level_time = (*ctx.world).level.time;
        if (*atcl).forcePowerSoundDebounce < level_time {
            let abSound = G_PreDefSound(ctx, (*atcl).ps.origin, PDSOUND_ABSORBHIT as c_int);
            (*abSound).s.trickedentindex = (*attacked).s.number;

            (*atcl).forcePowerSoundDebounce = level_time + 400;
        }

        getLevel
    }
}

/// Raven `WP_ForcePowerRegenerate` — regen the force pool on a regular interval.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:999-1019`
pub fn WP_ForcePowerRegenerate(self_: *mut gentity_t, overrideAmt: c_int) {
    unsafe {
        if (*self_).client.is_null() {
            return;
        }
        let cl = (*self_).client as *mut gclient_t;

        if overrideAmt != 0 {
            //custom regen amount
            (*cl).ps.fd.forcePower += overrideAmt;
        } else {
            //otherwise, just 1
            (*cl).ps.fd.forcePower += 1;
        }

        if (*cl).ps.fd.forcePower > (*cl).ps.fd.forcePowerMax {
            //cap it off at the max (default 100)
            (*cl).ps.fd.forcePower = (*cl).ps.fd.forcePowerMax;
        }
    }
}

/// Raven `WP_ForcePowerStart` — activate the given force power.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:1021-1234`
pub fn WP_ForcePowerStart(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    forcePower: forcePowers_t,
    mut overrideAmt: c_int,
) {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;
        let mut duration: c_int = 0;
        let mut hearable = qfalse;
        let mut hearDist: f32 = 0.0;

        if WP_ForcePowerAvailable(ctx, self_, forcePower, overrideAmt) == 0 {
            return;
        }

        if BG_FullBodyTauntAnim((*cl).ps.legsAnim) != 0 {
            //stop taunt
            (*cl).ps.legsTimer = 0;
        }
        if BG_FullBodyTauntAnim((*cl).ps.torsoAnim) != 0 {
            //stop taunt
            (*cl).ps.torsoTimer = 0;
        }
        //hearable and hearDist are merely for the benefit of bots, and not related to if a sound is actually played.
        //If duration is set, the force power will assume to be timer-based.
        match forcePower {
            FP_HEAL => {
                hearable = qtrue;
                hearDist = 256.0;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
            }
            FP_LEVITATION => {
                hearable = qtrue;
                hearDist = 256.0;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
            }
            FP_SPEED => {
                hearable = qtrue;
                hearDist = 256.0;
                if (*cl).ps.fd.forcePowerLevel[FP_SPEED as usize] == FORCE_LEVEL_1 {
                    duration = 10000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_SPEED as usize] == FORCE_LEVEL_2 {
                    duration = 15000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_SPEED as usize] == FORCE_LEVEL_3 {
                    duration = 20000;
                } else {
                    //shouldn't get here
                    // break;
                }
                if duration != 0 || (*cl).ps.fd.forcePowerLevel[FP_SPEED as usize] >= FORCE_LEVEL_1 {
                    if overrideAmt != 0 {
                        duration = overrideAmt;
                    }
                    (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
                }
            }
            FP_PUSH => {
                hearable = qtrue;
                hearDist = 256.0;
            }
            FP_PULL => {
                hearable = qtrue;
                hearDist = 256.0;
            }
            FP_TELEPATHY => {
                hearable = qtrue;
                hearDist = 256.0;
                if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_1 {
                    duration = 20000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_2 {
                    duration = 25000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_3 {
                    duration = 30000;
                }
                if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] >= FORCE_LEVEL_1 {
                    (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
                }
            }
            FP_GRIP => {
                hearable = qtrue;
                hearDist = 256.0;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
                (*cl).ps.powerups[PW_DISINT_4 as usize] = level_time + 60000;
            }
            FP_LIGHTNING => {
                hearable = qtrue;
                hearDist = 512.0;
                duration = overrideAmt;
                overrideAmt = 0;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
                (*cl).ps.activeForcePass = (*cl).ps.fd.forcePowerLevel[FP_LIGHTNING as usize];
            }
            FP_RAGE => {
                hearable = qtrue;
                hearDist = 256.0;
                if (*cl).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_1 {
                    duration = 8000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_2 {
                    duration = 14000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_3 {
                    duration = 20000;
                }
                if (*cl).ps.fd.forcePowerLevel[FP_RAGE as usize] >= FORCE_LEVEL_1 {
                    (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
                }
            }
            FP_PROTECT => {
                hearable = qtrue;
                hearDist = 256.0;
                duration = 20000;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
            }
            FP_ABSORB => {
                hearable = qtrue;
                hearDist = 256.0;
                duration = 20000;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
            }
            FP_TEAM_HEAL => {
                hearable = qtrue;
                hearDist = 256.0;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
            }
            FP_TEAM_FORCE => {
                hearable = qtrue;
                hearDist = 256.0;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
            }
            FP_DRAIN => {
                hearable = qtrue;
                hearDist = 256.0;
                duration = overrideAmt;
                overrideAmt = 0;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
            }
            FP_SEE => {
                hearable = qtrue;
                hearDist = 256.0;
                if (*cl).ps.fd.forcePowerLevel[FP_SEE as usize] == FORCE_LEVEL_1 {
                    duration = 10000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_SEE as usize] == FORCE_LEVEL_2 {
                    duration = 20000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_SEE as usize] == FORCE_LEVEL_3 {
                    duration = 30000;
                }
                if (*cl).ps.fd.forcePowerLevel[FP_SEE as usize] >= FORCE_LEVEL_1 {
                    (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
                }
            }
            FP_SABER_OFFENSE => {}
            FP_SABER_DEFENSE => {}
            FP_SABERTHROW => {}
            _ => {}
        }

        if duration != 0 {
            (*cl).ps.fd.forcePowerDuration[forcePower as usize] = level_time + duration;
        } else {
            (*cl).ps.fd.forcePowerDuration[forcePower as usize] = 0;
        }

        if hearable != 0 {
            (*cl).ps.otherSoundLen = hearDist;
            (*cl).ps.otherSoundTime = level_time + 100;
        }

        (*cl).ps.fd.forcePowerDebounce[forcePower as usize] = 0;

        if forcePower == FP_SPEED && overrideAmt != 0 {
            BG_ForcePowerDrain(&mut (*cl).ps, forcePower, (overrideAmt as f32 * 0.025) as c_int);
        } else if forcePower != FP_GRIP && forcePower != FP_DRAIN {
            //grip and drain drain as damage is done
            BG_ForcePowerDrain(&mut (*cl).ps, forcePower, overrideAmt);
        }
    }
}

/// Raven `ForceHeal`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:1236-1292`
pub fn ForceHeal(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;

        if (*self_).health <= 0 {
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_HEAL) == 0 {
            return;
        }

        if (*self_).health >= (*cl).ps.stats[STAT_MAX_HEALTH as usize] {
            return;
        }

        if (*cl).ps.fd.forcePowerLevel[FP_HEAL as usize] == FORCE_LEVEL_3 {
            (*self_).health += 25; //This was 50, but that angered the Balance God.
            if (*self_).health > (*cl).ps.stats[STAT_MAX_HEALTH as usize] {
                (*self_).health = (*cl).ps.stats[STAT_MAX_HEALTH as usize];
            }
            BG_ForcePowerDrain(&mut (*cl).ps, FP_HEAL, 0);
        } else if (*cl).ps.fd.forcePowerLevel[FP_HEAL as usize] == FORCE_LEVEL_2 {
            (*self_).health += 10;
            if (*self_).health > (*cl).ps.stats[STAT_MAX_HEALTH as usize] {
                (*self_).health = (*cl).ps.stats[STAT_MAX_HEALTH as usize];
            }
            BG_ForcePowerDrain(&mut (*cl).ps, FP_HEAL, 0);
        } else {
            (*self_).health += 5;
            if (*self_).health > (*cl).ps.stats[STAT_MAX_HEALTH as usize] {
                (*self_).health = (*cl).ps.stats[STAT_MAX_HEALTH as usize];
            }
            BG_ForcePowerDrain(&mut (*cl).ps, FP_HEAL, 0);
        }
        //NOTE: Decided to make all levels instant.

        let snd = std::ffi::CString::new("sound/weapons/force/heal.wav").unwrap();
        G_Sound(ctx, self_, CHAN_ITEM, G_SoundIndex(snd.as_ptr()));
    }
}

/// Raven `WP_AddToClientBitflags` — pack `entNum` into a temp-ent's tricked-index
/// bitfields.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:1294-1317`
pub fn WP_AddToClientBitflags(ent: *mut gentity_t, entNum: c_int) {
    unsafe {
        if ent.is_null() {
            return;
        }

        if entNum > 47 {
            (*ent).s.trickedentindex4 |= 1 << (entNum - 48);
        } else if entNum > 31 {
            (*ent).s.trickedentindex3 |= 1 << (entNum - 32);
        } else if entNum > 15 {
            (*ent).s.trickedentindex2 |= 1 << (entNum - 16);
        } else {
            (*ent).s.trickedentindex |= 1 << entNum;
        }
    }
}

/// Raven `ForceTeamHeal`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:1319-1422`
// PORT-ESCALATION(unported-global): reads the file-scope
// `forcePowerNeeded/bgSiegeClasses` table(s) — genuinely unported runtime data
// (fork-discovery ruling 1: globals -> GameWorld fields), not just a
// missing `use`.
pub fn ForceTeamHeal(ctx: GameContext<'_>, self_: *mut gentity_t) {
    todo!("Port ForceTeamHeal — parked: unported-global (forcePowerNeeded/bgSiegeClasses)")
}

/// Raven `ForceTeamForceReplenish`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:1424-1521`
// PORT-ESCALATION(unported-global): reads the file-scope
// `forcePowerNeeded/bgSiegeClasses` table(s) — genuinely unported runtime data
// (fork-discovery ruling 1: globals -> GameWorld fields), not just a
// missing `use`.
pub fn ForceTeamForceReplenish(ctx: GameContext<'_>, self_: *mut gentity_t) {
    todo!("Port ForceTeamForceReplenish — parked: unported-global (forcePowerNeeded/bgSiegeClasses)")
}

/// Raven `ForceGrip`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:1523-1594`
// PORT-ESCALATION(vehicle-vtable): the grip-hit branch calls
// `vehEnt->m_pVehicle->m_pVehicleInfo->Eject(...)` — the fork-7 vehicle vtable,
// resolved as mp_bg enum-over-type dispatch, which is not in this packet's
// resolved call surface. Faithful port of that sub-effect needs the bg Eject
// dispatch entrypoint; skipping it would be speculative behavior (porting-rules
// SA), so the whole fn is parked.
pub fn ForceGrip(ctx: GameContext<'_>, self_: *mut gentity_t) {
    todo!("Port ForceGrip — oracle/oracle/codemp/game/w_force.c:1523")
}

/// Raven `ForceSpeed`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:1596-1629`
pub fn ForceSpeed(ctx: GameContext<'_>, self_: *mut gentity_t, forceDuration: c_int) {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;

        if (*self_).health <= 0 {
            return;
        }

        if (*cl).ps.forceAllowDeactivateTime < level_time
            && (*cl).ps.fd.forcePowersActive & (1 << FP_SPEED) != 0
        {
            WP_ForcePowerStop(ctx, self_, FP_SPEED);
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_SPEED) == 0 {
            return;
        }

        if (*cl).holdingObjectiveItem >= MAX_CLIENTS as c_int
            && (*cl).holdingObjectiveItem < ENTITYNUM_WORLD
        {
            //holding Siege item
            if (*ctx.world).entities[(*cl).holdingObjectiveItem as usize].genericValue15 != 0 {
                //disables force powers
                return;
            }
        }

        (*cl).ps.forceAllowDeactivateTime = level_time + 1500;

        WP_ForcePowerStart(ctx, self_, FP_SPEED, forceDuration);
        let snd = std::ffi::CString::new("sound/weapons/force/speed.wav").unwrap();
        G_Sound(ctx, self_, CHAN_BODY, G_SoundIndex(snd.as_ptr()));
        G_Sound(ctx, self_, TRACK_CHANNEL_2 as c_int, (*ctx.world).speedLoopSound);
    }
}

/// Raven `ForceSeeing`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:1631-1656`
pub fn ForceSeeing(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;

        if (*self_).health <= 0 {
            return;
        }

        if (*cl).ps.forceAllowDeactivateTime < level_time
            && (*cl).ps.fd.forcePowersActive & (1 << FP_SEE) != 0
        {
            WP_ForcePowerStop(ctx, self_, FP_SEE);
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_SEE) == 0 {
            return;
        }

        (*cl).ps.forceAllowDeactivateTime = level_time + 1500;

        WP_ForcePowerStart(ctx, self_, FP_SEE, 0);

        let snd = std::ffi::CString::new("sound/weapons/force/see.wav").unwrap();
        G_Sound(ctx, self_, CHAN_AUTO, G_SoundIndex(snd.as_ptr()));
        G_Sound(ctx, self_, TRACK_CHANNEL_5 as c_int, (*ctx.world).seeLoopSound);
    }
}

/// Raven `ForceProtect`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:1658-1692`
pub fn ForceProtect(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;

        if (*self_).health <= 0 {
            return;
        }

        if (*cl).ps.forceAllowDeactivateTime < level_time
            && (*cl).ps.fd.forcePowersActive & (1 << FP_PROTECT) != 0
        {
            WP_ForcePowerStop(ctx, self_, FP_PROTECT);
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_PROTECT) == 0 {
            return;
        }

        // Make sure to turn off Force Rage and Force Absorb.
        if (*cl).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0 {
            WP_ForcePowerStop(ctx, self_, FP_RAGE);
        }
        if (*cl).ps.fd.forcePowersActive & (1 << FP_ABSORB) != 0 {
            WP_ForcePowerStop(ctx, self_, FP_ABSORB);
        }

        (*cl).ps.forceAllowDeactivateTime = level_time + 1500;

        WP_ForcePowerStart(ctx, self_, FP_PROTECT, 0);
        G_PreDefSound(ctx, (*cl).ps.origin, PDSOUND_PROTECT as c_int);
        G_Sound(ctx, self_, TRACK_CHANNEL_3 as c_int, (*ctx.world).protectLoopSound);
    }
}

/// Raven `ForceAbsorb`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:1694-1728`
pub fn ForceAbsorb(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;

        if (*self_).health <= 0 {
            return;
        }

        if (*cl).ps.forceAllowDeactivateTime < level_time
            && (*cl).ps.fd.forcePowersActive & (1 << FP_ABSORB) != 0
        {
            WP_ForcePowerStop(ctx, self_, FP_ABSORB);
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_ABSORB) == 0 {
            return;
        }

        // Make sure to turn off Force Rage and Force Protection.
        if (*cl).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0 {
            WP_ForcePowerStop(ctx, self_, FP_RAGE);
        }
        if (*cl).ps.fd.forcePowersActive & (1 << FP_PROTECT) != 0 {
            WP_ForcePowerStop(ctx, self_, FP_PROTECT);
        }

        (*cl).ps.forceAllowDeactivateTime = level_time + 1500;

        WP_ForcePowerStart(ctx, self_, FP_ABSORB, 0);
        G_PreDefSound(ctx, (*cl).ps.origin, PDSOUND_ABSORB as c_int);
        G_Sound(ctx, self_, TRACK_CHANNEL_3 as c_int, (*ctx.world).absorbLoopSound);
    }
}

/// Raven `ForceRage`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:1730-1775`
pub fn ForceRage(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;

        if (*self_).health <= 0 {
            return;
        }

        if (*cl).ps.forceAllowDeactivateTime < level_time
            && (*cl).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0
        {
            WP_ForcePowerStop(ctx, self_, FP_RAGE);
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_RAGE) == 0 {
            return;
        }

        if (*cl).ps.fd.forceRageRecoveryTime >= level_time {
            return;
        }

        if (*self_).health < 10 {
            return;
        }

        // Make sure to turn off Force Protection and Force Absorb.
        if (*cl).ps.fd.forcePowersActive & (1 << FP_PROTECT) != 0 {
            WP_ForcePowerStop(ctx, self_, FP_PROTECT);
        }
        if (*cl).ps.fd.forcePowersActive & (1 << FP_ABSORB) != 0 {
            WP_ForcePowerStop(ctx, self_, FP_ABSORB);
        }

        (*cl).ps.forceAllowDeactivateTime = level_time + 1500;

        WP_ForcePowerStart(ctx, self_, FP_RAGE, 0);

        let snd = std::ffi::CString::new("sound/weapons/force/rage.wav").unwrap();
        G_Sound(ctx, self_, TRACK_CHANNEL_4 as c_int, G_SoundIndex(snd.as_ptr()));
        G_Sound(ctx, self_, TRACK_CHANNEL_3 as c_int, (*ctx.world).rageLoopSound);
    }
}

/// Raven `ForceLightning`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:1777-1810`
pub fn ForceLightning(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;

        if (*self_).health <= 0 {
            return;
        }
        if (*cl).ps.fd.forcePower < 25 || WP_ForcePowerUsable(ctx, self_, FP_LIGHTNING) == 0 {
            return;
        }
        if (*cl).ps.fd.forcePowerDebounce[FP_LIGHTNING as usize] > level_time {
            //stops it while using it and also after using it, up to 3 second delay
            return;
        }

        if (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int {
            return;
        }

        if (*cl).ps.weaponTime > 0 {
            return;
        }

        //Shoot lightning from hand
        //using grip anim now, to extend the burst time
        (*cl).ps.forceHandExtend = HANDEXTEND_FORCE_HOLD as c_int;
        (*cl).ps.forceHandExtendTime = level_time + 20000;

        let snd = std::ffi::CString::new("sound/weapons/force/lightning").unwrap();
        G_Sound(ctx, self_, CHAN_BODY, G_SoundIndex(snd.as_ptr()));

        WP_ForcePowerStart(ctx, self_, FP_LIGHTNING, 500);
    }
}

/// Raven `ForceLightningDamage` — apply a lightning tick to `traceEnt`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:1812-1900`
pub fn ForceLightningDamage(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    traceEnt: *mut gentity_t,
    dir: vec3_t,
    impactPoint: vec3_t,
) {
    unsafe {
        let scl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;

        (*scl).dangerTime = level_time;
        (*scl).ps.eFlags &= !EF_INVULNERABLE;
        (*scl).invulnerableTimer = 0;

        if !traceEnt.is_null() && (*traceEnt).takedamage != 0 {
            if (*traceEnt).client.is_null() && (*traceEnt).s.eType == ET_NPC as c_int {
                //g2animent
                if (*traceEnt).s.genericenemyindex < level_time {
                    (*traceEnt).s.genericenemyindex = level_time + 2000;
                }
            }
            if !(*traceEnt).client.is_null() {
                //an enemy or object
                let tcl = (*traceEnt).client as *mut gclient_t;
                if (*tcl).noLightningTime >= level_time {
                    //give them power and don't hurt them.
                    (*tcl).ps.fd.forcePower += 1;
                    if (*tcl).ps.fd.forcePower > 100 {
                        (*tcl).ps.fd.forcePower = 100;
                    }
                    return;
                }
                if ForcePowerUsableOn(ctx, self_, traceEnt, FP_LIGHTNING) != 0 {
                    let mut dmg = Q_irand(1, 2); //Q_irand( 1, 3 );

                    let mut modPowerLevel = -1;

                    if !(*traceEnt).client.is_null() {
                        modPowerLevel = WP_AbsorbConversion(
                            ctx,
                            traceEnt,
                            (*tcl).ps.fd.forcePowerLevel[FP_ABSORB as usize],
                            self_,
                            FP_LIGHTNING,
                            (*scl).ps.fd.forcePowerLevel[FP_LIGHTNING as usize],
                            1,
                        );
                    }

                    if modPowerLevel != -1 {
                        if modPowerLevel == 0 {
                            dmg = 0;
                            (*tcl).noLightningTime = level_time + 400;
                        } else if modPowerLevel == 1 {
                            dmg = 1;
                            (*tcl).noLightningTime = level_time + 300;
                        } else if modPowerLevel == 2 {
                            dmg = 1;
                            (*tcl).noLightningTime = level_time + 100;
                        }
                    }

                    if (*scl).ps.weapon == WP_MELEE
                        && (*scl).ps.fd.forcePowerLevel[FP_LIGHTNING as usize] > FORCE_LEVEL_2
                    {
                        //2-handed lightning
                        //jackin' 'em up, Palpatine-style
                        dmg *= 2;
                    }

                    if dmg != 0 {
                        //rww - Shields can now absorb lightning too.
                        G_Damage(traceEnt, self_, self_, dir, impactPoint, dmg, 0, MOD_FORCE_DARK as c_int);
                    }
                    if !(*traceEnt).client.is_null() {
                        if Q_irand(0, 2) == 0 {
                            let snd = std::ffi::CString::new(format!(
                                "sound/weapons/force/lightninghit{}",
                                Q_irand(1, 3)
                            ))
                            .unwrap();
                            G_Sound(ctx, traceEnt, CHAN_BODY, G_SoundIndex(snd.as_ptr()));
                        }

                        if (*tcl).ps.electrifyTime < (level_time + 400) {
                            //only update every 400ms to reduce bandwidth usage (as it is passing a 32-bit time value)
                            (*tcl).ps.electrifyTime = level_time + 800;
                        }
                        if (*tcl).ps.powerups[PW_CLOAKED as usize] != 0 {
                            //disable cloak temporarily
                            Jedi_Decloak(ctx, traceEnt);
                            (*tcl).cloakToggleTime = level_time + Q_irand(3000, 10000);
                        }
                    }
                }
            }
        }
    }
}

/// Raven `ForceShootLightning`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:1902-2020`
pub fn ForceShootLightning(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        let scl = (*self_).client as *mut gclient_t;

        if (*self_).health <= 0 {
            return;
        }
        let mut forward: vec3_t = [0.0; 3];
        AngleVectors((*scl).ps.viewangles, Some(&mut forward), None, None);
        VectorNormalize(&mut forward);

        let mut tr: trace_t = core::mem::zeroed();

        if (*scl).ps.fd.forcePowerLevel[FP_LIGHTNING as usize] > FORCE_LEVEL_2 {
            //arc
            let radius: f32 = FORCE_LIGHTNING_RADIUS as f32;
            let center: vec3_t = (*scl).ps.origin;
            let mut mins: vec3_t = [0.0; 3];
            let mut maxs: vec3_t = [0.0; 3];
            for i in 0..3 {
                mins[i] = center[i] - radius;
                maxs[i] = center[i] + radius;
            }
            let mut iEntityList = [0i32; MAX_GENTITIES as usize];
            let numListedEntities = trap::EntitiesInBox(
                ctx.engine,
                GEntitiesInBoxArgs::new(
                    &mins as *const vec3_t,
                    &maxs as *const vec3_t,
                    iEntityList.as_mut_ptr(),
                    MAX_GENTITIES as c_int,
                ),
            );

            for e in 0..numListedEntities {
                let traceEnt = &mut (*ctx.world).entities[iEntityList[e as usize] as usize]
                    as *mut gentity_t;

                if traceEnt == self_ {
                    continue;
                }
                if (*traceEnt).r.ownerNum == (*self_).s.number
                    && (*traceEnt).s.weapon != WP_THERMAL
                //can push your own thermals
                {
                    continue;
                }
                if (*traceEnt).inuse == 0 {
                    continue;
                }
                if (*traceEnt).takedamage == 0 {
                    continue;
                }
                if (*traceEnt).health <= 0 {
                    //no torturing corpses
                    continue;
                }
                if (*ctx.world).cvars.g_friendlyFire.integer == 0 && OnSameTeam(ctx, self_, traceEnt) != 0
                {
                    continue;
                }

                // find the distance from the edge of the bounding box
                let mut v: vec3_t = [0.0; 3];
                for i in 0..3 {
                    if center[i] < (*traceEnt).r.absmin[i] {
                        v[i] = (*traceEnt).r.absmin[i] - center[i];
                    } else if center[i] > (*traceEnt).r.absmax[i] {
                        v[i] = center[i] - (*traceEnt).r.absmax[i];
                    } else {
                        v[i] = 0.0;
                    }
                }

                let size: vec3_t = [
                    (*traceEnt).r.absmax[0] - (*traceEnt).r.absmin[0],
                    (*traceEnt).r.absmax[1] - (*traceEnt).r.absmin[1],
                    (*traceEnt).r.absmax[2] - (*traceEnt).r.absmin[2],
                ];
                let ent_org: vec3_t = [
                    (*traceEnt).r.absmin[0] + 0.5 * size[0],
                    (*traceEnt).r.absmin[1] + 0.5 * size[1],
                    (*traceEnt).r.absmin[2] + 0.5 * size[2],
                ];

                //see if they're in front of me / within the forward cone
                let mut dir: vec3_t = [
                    ent_org[0] - center[0],
                    ent_org[1] - center[1],
                    ent_org[2] - center[2],
                ];
                VectorNormalize(&mut dir);
                let dot = dir[0] * forward[0] + dir[1] * forward[1] + dir[2] * forward[2];
                if dot < 0.5 {
                    continue;
                }

                //must be close enough
                let dist = VectorLength(v);
                if dist >= radius {
                    continue;
                }

                //in PVS?
                if (*traceEnt).r.bmodel == 0
                    && trap::InPVS(
                        ctx.engine,
                        GInPvsArgs::new(&ent_org as *const vec3_t, &(*scl).ps.origin as *const vec3_t),
                    ) == 0
                {
                    //must be in PVS
                    continue;
                }

                //Now check and see if we can actually hit it
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &(*scl).ps.origin as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &ent_org as *const vec3_t,
                        (*self_).s.number,
                        MASK_SHOT,
                    ),
                );
                if tr.fraction < 1.0 && tr.entityNum != (*traceEnt).s.number {
                    //must have clear LOS
                    continue;
                }

                // ok, we are within the radius, add us to the incoming list
                ForceLightningDamage(ctx, self_, traceEnt, dir, ent_org);
            }
        } else {
            //trace-line
            let end: vec3_t = [
                (*scl).ps.origin[0] + 2048.0 * forward[0],
                (*scl).ps.origin[1] + 2048.0 * forward[1],
                (*scl).ps.origin[2] + 2048.0 * forward[2],
            ];

            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &(*scl).ps.origin as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &end as *const vec3_t,
                    (*self_).s.number,
                    MASK_SHOT,
                ),
            );
            if tr.entityNum == ENTITYNUM_NONE || tr.fraction == 1.0 || tr.allsolid != 0 || tr.startsolid != 0
            {
                return;
            }

            let traceEnt = &mut (*ctx.world).entities[tr.entityNum as usize] as *mut gentity_t;
            ForceLightningDamage(ctx, self_, traceEnt, forward, tr.endpos);
        }
    }
}

/// Raven `ForceDrain`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2022-2056`
pub fn ForceDrain(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;

        if (*self_).health <= 0 {
            return;
        }

        if (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int {
            return;
        }

        if (*cl).ps.weaponTime > 0 {
            return;
        }

        if (*cl).ps.fd.forcePower < 25 || WP_ForcePowerUsable(ctx, self_, FP_DRAIN) == 0 {
            return;
        }
        if (*cl).ps.fd.forcePowerDebounce[FP_DRAIN as usize] > level_time {
            //stops it while using it and also after using it, up to 3 second delay
            return;
        }

        (*cl).ps.forceHandExtend = HANDEXTEND_FORCE_HOLD as c_int;
        (*cl).ps.forceHandExtendTime = level_time + 20000;

        let snd = std::ffi::CString::new("sound/weapons/force/drain.wav").unwrap();
        G_Sound(ctx, self_, CHAN_BODY, G_SoundIndex(snd.as_ptr()));

        WP_ForcePowerStart(ctx, self_, FP_DRAIN, 500);
    }
}

/// Raven `ForceDrainDamage`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2058-2182`
pub fn ForceDrainDamage(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    traceEnt: *mut gentity_t,
    dir: vec3_t,
    impactPoint: vec3_t,
) {
    unsafe {
        let scl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;

        (*scl).dangerTime = level_time;
        (*scl).ps.eFlags &= !EF_INVULNERABLE;
        (*scl).invulnerableTimer = 0;

        if !traceEnt.is_null() && (*traceEnt).takedamage != 0 {
            let tcl = (*traceEnt).client as *mut gclient_t;
            if !(*traceEnt).client.is_null()
                && (OnSameTeam(ctx, self_, traceEnt) == 0
                    || (*ctx.world).cvars.g_friendlyFire.integer != 0)
                && (*scl).ps.fd.forceDrainTime < level_time
                && (*tcl).ps.fd.forcePower != 0
            {
                //an enemy or object
                if (*traceEnt).client.is_null() && (*traceEnt).s.eType == ET_NPC as c_int {
                    //g2animent
                    if (*traceEnt).s.genericenemyindex < level_time {
                        (*traceEnt).s.genericenemyindex = level_time + 2000;
                    }
                }
                if ForcePowerUsableOn(ctx, self_, traceEnt, FP_DRAIN) != 0 {
                    let mut modPowerLevel = -1;
                    let mut dmg = 0; //Q_irand( 1, 3 );
                    if (*scl).ps.fd.forcePowerLevel[FP_DRAIN as usize] == FORCE_LEVEL_1 {
                        dmg = 2; //because it's one-shot
                    } else if (*scl).ps.fd.forcePowerLevel[FP_DRAIN as usize] == FORCE_LEVEL_2 {
                        dmg = 3;
                    } else if (*scl).ps.fd.forcePowerLevel[FP_DRAIN as usize] == FORCE_LEVEL_3 {
                        dmg = 4;
                    }

                    if !(*traceEnt).client.is_null() {
                        modPowerLevel = WP_AbsorbConversion(
                            ctx,
                            traceEnt,
                            (*tcl).ps.fd.forcePowerLevel[FP_ABSORB as usize],
                            self_,
                            FP_DRAIN,
                            (*scl).ps.fd.forcePowerLevel[FP_DRAIN as usize],
                            1,
                        );
                    }

                    if modPowerLevel != -1 {
                        if modPowerLevel == 0 {
                            dmg = 0;
                        } else if modPowerLevel == 1 {
                            dmg = 1;
                        } else if modPowerLevel == 2 {
                            dmg = 2;
                        }
                    }
                    //G_Damage( traceEnt, self, self, dir, impactPoint, dmg, 0, MOD_FORCE_DARK );

                    if dmg != 0 {
                        (*tcl).ps.fd.forcePower -= dmg;
                    }
                    if (*tcl).ps.fd.forcePower < 0 {
                        (*tcl).ps.fd.forcePower = 0;
                    }

                    if (*scl).ps.stats[STAT_HEALTH as usize] < (*scl).ps.stats[STAT_MAX_HEALTH as usize]
                        && (*self_).health > 0
                        && (*scl).ps.stats[STAT_HEALTH as usize] > 0
                    {
                        (*self_).health += dmg;
                        if (*self_).health > (*scl).ps.stats[STAT_MAX_HEALTH as usize] {
                            (*self_).health = (*scl).ps.stats[STAT_MAX_HEALTH as usize];
                        }
                        (*scl).ps.stats[STAT_HEALTH as usize] = (*self_).health;
                    }

                    //don't let the client being drained get force power back right away
                    (*tcl).ps.fd.forcePowerRegenDebounceTime = level_time + 800;

                    if (*tcl).forcePowerSoundDebounce < level_time {
                        let tent = G_TempEntity(ctx, impactPoint, EV_FORCE_DRAINED as c_int);
                        (*tent).s.eventParm = DirToByte(dir);
                        (*tent).s.owner = (*traceEnt).s.number;

                        (*tcl).forcePowerSoundDebounce = level_time + 400;
                    }
                }
            }
        }
    }
}

/// Raven `ForceShootDrain`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2184-2315`
pub fn ForceShootDrain(ctx: GameContext<'_>, self_: *mut gentity_t) -> c_int {
    unsafe {
        let scl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;
        let mut gotOneOrMore = 0;

        if (*self_).health <= 0 {
            return 0;
        }
        let mut forward: vec3_t = [0.0; 3];
        AngleVectors((*scl).ps.viewangles, Some(&mut forward), None, None);
        VectorNormalize(&mut forward);

        let mut tr: trace_t = core::mem::zeroed();

        if (*scl).ps.fd.forcePowerLevel[FP_DRAIN as usize] > FORCE_LEVEL_2 {
            //arc
            let radius: f32 = MAX_DRAIN_DISTANCE as f32;
            let center: vec3_t = (*scl).ps.origin;
            let mut mins: vec3_t = [0.0; 3];
            let mut maxs: vec3_t = [0.0; 3];
            for i in 0..3 {
                mins[i] = center[i] - radius;
                maxs[i] = center[i] + radius;
            }
            let mut iEntityList = [0i32; MAX_GENTITIES as usize];
            let numListedEntities = trap::EntitiesInBox(
                ctx.engine,
                GEntitiesInBoxArgs::new(
                    &mins as *const vec3_t,
                    &maxs as *const vec3_t,
                    iEntityList.as_mut_ptr(),
                    MAX_GENTITIES as c_int,
                ),
            );

            for e in 0..numListedEntities {
                let traceEnt = &mut (*ctx.world).entities[iEntityList[e as usize] as usize]
                    as *mut gentity_t;

                if traceEnt == self_ {
                    continue;
                }
                if (*traceEnt).inuse == 0 {
                    continue;
                }
                if (*traceEnt).takedamage == 0 {
                    continue;
                }
                if (*traceEnt).health <= 0 {
                    //no torturing corpses
                    continue;
                }
                if (*traceEnt).client.is_null() {
                    continue;
                }
                let tcl = (*traceEnt).client as *mut gclient_t;
                if (*tcl).ps.fd.forcePower == 0 {
                    continue;
                }
                if OnSameTeam(ctx, self_, traceEnt) != 0
                    && (*ctx.world).cvars.g_friendlyFire.integer == 0
                {
                    continue;
                }

                // find the distance from the edge of the bounding box
                let mut v: vec3_t = [0.0; 3];
                for i in 0..3 {
                    if center[i] < (*traceEnt).r.absmin[i] {
                        v[i] = (*traceEnt).r.absmin[i] - center[i];
                    } else if center[i] > (*traceEnt).r.absmax[i] {
                        v[i] = center[i] - (*traceEnt).r.absmax[i];
                    } else {
                        v[i] = 0.0;
                    }
                }

                let size: vec3_t = [
                    (*traceEnt).r.absmax[0] - (*traceEnt).r.absmin[0],
                    (*traceEnt).r.absmax[1] - (*traceEnt).r.absmin[1],
                    (*traceEnt).r.absmax[2] - (*traceEnt).r.absmin[2],
                ];
                let ent_org: vec3_t = [
                    (*traceEnt).r.absmin[0] + 0.5 * size[0],
                    (*traceEnt).r.absmin[1] + 0.5 * size[1],
                    (*traceEnt).r.absmin[2] + 0.5 * size[2],
                ];

                let mut dir: vec3_t = [
                    ent_org[0] - center[0],
                    ent_org[1] - center[1],
                    ent_org[2] - center[2],
                ];
                VectorNormalize(&mut dir);
                let dot = dir[0] * forward[0] + dir[1] * forward[1] + dir[2] * forward[2];
                if dot < 0.5 {
                    continue;
                }

                let dist = VectorLength(v);
                if dist >= radius {
                    continue;
                }

                if (*traceEnt).r.bmodel == 0
                    && trap::InPVS(
                        ctx.engine,
                        GInPvsArgs::new(&ent_org as *const vec3_t, &(*scl).ps.origin as *const vec3_t),
                    ) == 0
                {
                    continue;
                }

                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &(*scl).ps.origin as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &ent_org as *const vec3_t,
                        (*self_).s.number,
                        MASK_SHOT,
                    ),
                );
                if tr.fraction < 1.0 && tr.entityNum != (*traceEnt).s.number {
                    continue;
                }

                ForceDrainDamage(ctx, self_, traceEnt, dir, ent_org);
                gotOneOrMore = 1;
            }
        } else {
            //trace-line
            let end: vec3_t = [
                (*scl).ps.origin[0] + 2048.0 * forward[0],
                (*scl).ps.origin[1] + 2048.0 * forward[1],
                (*scl).ps.origin[2] + 2048.0 * forward[2],
            ];

            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &(*scl).ps.origin as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &end as *const vec3_t,
                    (*self_).s.number,
                    MASK_SHOT,
                ),
            );
            if tr.entityNum == ENTITYNUM_NONE
                || tr.fraction == 1.0
                || tr.allsolid != 0
                || tr.startsolid != 0
                || (*ctx.world).entities[tr.entityNum as usize].client.is_null()
                || (*ctx.world).entities[tr.entityNum as usize].inuse == 0
            {
                return 0;
            }

            let traceEnt = &mut (*ctx.world).entities[tr.entityNum as usize] as *mut gentity_t;
            ForceDrainDamage(ctx, self_, traceEnt, forward, tr.endpos);
            gotOneOrMore = 1;
        }

        (*scl).ps.activeForcePass =
            (*scl).ps.fd.forcePowerLevel[FP_DRAIN as usize] + FORCE_LEVEL_3;

        //used to be 1, but this did, too, anger the God of Balance.
        BG_ForcePowerDrain(&mut (*scl).ps, FP_DRAIN, 5);

        (*scl).ps.fd.forcePowerRegenDebounceTime = level_time + 500;

        gotOneOrMore
    }
}

/// Raven `ForceJumpCharge`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2317-2375`
// PORT-ESCALATION(unported-global-table): reads the file-scope `forceJumpStrength`
// and `forcePowerNeeded` tables — genuinely un-ported runtime data (fork-5 const
// tables); their values are not in this packet and cannot be invented (no oracle
// read), so the fn stays parked like its pass-1 siblings.
pub fn ForceJumpCharge(ctx: GameContext<'_>, self_: *mut gentity_t, ucmd: *mut usercmd_t) {
    todo!("Port ForceJumpCharge — parked: unported-global (forceJumpStrength/forcePowerNeeded)")
}

/// Raven `WP_GetVelocityForForceJump`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2377-2460`
// `jumpVel` is a written-through out-param (`VectorMA(... jumpVel)`); fork-9
// reshapes the by-value `vec3_t` to `&mut vec3_t`.
pub fn WP_GetVelocityForForceJump(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    jumpVel: &mut vec3_t,
    ucmd: *mut usercmd_t,
) -> c_int {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;

        let mut pushFwd: f32 = 0.0;
        let mut pushRt: f32 = 0.0;
        let mut view: vec3_t = (*cl).ps.viewangles;
        view[0] = 0.0;
        let mut forward: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        AngleVectors(view, Some(&mut forward), Some(&mut right), None);

        if (*ucmd).forwardmove != 0 && (*ucmd).rightmove != 0 {
            if (*ucmd).forwardmove > 0 {
                pushFwd = 50.0;
            } else {
                pushFwd = -50.0;
            }
            if (*ucmd).rightmove > 0 {
                pushRt = 50.0;
            } else {
                pushRt = -50.0;
            }
        } else if (*ucmd).forwardmove != 0 || (*ucmd).rightmove != 0 {
            if (*ucmd).forwardmove > 0 {
                pushFwd = 100.0;
            } else if (*ucmd).forwardmove < 0 {
                pushFwd = -100.0;
            } else if (*ucmd).rightmove > 0 {
                pushRt = 100.0;
            } else if (*ucmd).rightmove < 0 {
                pushRt = -100.0;
            }
        }

        G_MuteSound(
            ctx,
            (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 as c_int - 50) as usize],
            CHAN_VOICE,
        );

        G_PreDefSound(ctx, (*cl).ps.origin, PDSOUND_FORCEJUMP as c_int);

        if (*cl).ps.fd.forceJumpCharge < JUMP_VELOCITY + 40.0 {
            //give him at least a tiny boost from just a tap
            (*cl).ps.fd.forceJumpCharge = JUMP_VELOCITY + 400.0;
        }

        if (*cl).ps.velocity[2] < -30.0 {
            //so that we can get a good boost when force jumping in a fall
            (*cl).ps.velocity[2] = -30.0;
        }

        for i in 0..3 {
            jumpVel[i] = (*cl).ps.velocity[i] + pushFwd * forward[i];
        }
        for i in 0..3 {
            jumpVel[i] = (*cl).ps.velocity[i] + pushRt * right[i];
        }
        jumpVel[2] += (*cl).ps.fd.forceJumpCharge;
        if pushFwd > 0.0 && (*cl).ps.fd.forceJumpCharge > 200.0 {
            FJ_FORWARD
        } else if pushFwd < 0.0 && (*cl).ps.fd.forceJumpCharge > 200.0 {
            FJ_BACKWARD
        } else if pushRt > 0.0 && (*cl).ps.fd.forceJumpCharge > 200.0 {
            FJ_RIGHT
        } else if pushRt < 0.0 && (*cl).ps.fd.forceJumpCharge > 200.0 {
            FJ_LEFT
        } else {
            FJ_UP
        }
    }
}

/// Raven `ForceJump`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2462-2500`
// PORT-ESCALATION(unported-global-table): reads `forceJumpStrength` and
// `forcePowerNeeded` (fork-5 const tables not yet ported; values absent from
// packet), so faithful port is blocked — parked.
pub fn ForceJump(ctx: GameContext<'_>, self_: *mut gentity_t, ucmd: *mut usercmd_t) {
    todo!("Port ForceJump — parked: unported-global (forceJumpStrength/forcePowerNeeded)")
}

/// Raven `WP_AddAsMindtricked` — pack `entNum` into a forcedata mindtrick mask.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2502-2525`
pub fn WP_AddAsMindtricked(fd: *mut forcedata_t, entNum: c_int) {
    unsafe {
        if fd.is_null() {
            return;
        }

        if entNum > 47 {
            (*fd).forceMindtrickTargetIndex4 |= 1 << (entNum - 48);
        } else if entNum > 31 {
            (*fd).forceMindtrickTargetIndex3 |= 1 << (entNum - 32);
        } else if entNum > 15 {
            (*fd).forceMindtrickTargetIndex2 |= 1 << (entNum - 16);
        } else {
            (*fd).forceMindtrickTargetIndex |= 1 << entNum;
        }
    }
}

/// Raven `ForceTelepathyCheckDirectNPCTarget`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2527-2721`
// PORT-ESCALATION(unported-npc-subsystem): the body reaches deep into the
// un-ported single-player-derived NPC subsystem — `traceEnt->NPC->scriptFlags`,
// `->charmedTime`/`->confusionTime`, `client->NPC_class`/`playerTeam`/`enemyTeam`/
// `leader`, `renderInfo.eyeAngles`/`eyePoint`, `s.teamowner`, `genericValue1..3`
// — none of which are in this packet's resolved field/call surface. Faithful
// transcription would require inventing NPC struct shape (porting-rules §A2), so
// the fn is parked.
pub fn ForceTelepathyCheckDirectNPCTarget(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    tr: *mut trace_t,
    tookPower: *mut qboolean,
) -> qboolean {
    todo!("Port ForceTelepathyCheckDirectNPCTarget — parked: unported-npc-subsystem")
}

/// Raven `ForceTelepathy`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2723-2893`
pub fn ForceTelepathy(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;

        let mut tr: trace_t = core::mem::zeroed();
        let mut visionArc: f32 = 0.0;
        let mut radius: f32 = MAX_TRICK_DISTANCE as f32;
        let mut tookPower: qboolean = qfalse;

        if (*self_).health <= 0 {
            return;
        }

        if (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int {
            return;
        }

        if (*cl).ps.weaponTime > 0 {
            return;
        }

        if (*cl).ps.powerups[PW_REDFLAG as usize] != 0 || (*cl).ps.powerups[PW_BLUEFLAG as usize] != 0
        {
            //can't mindtrick while carrying the flag
            return;
        }

        if (*cl).ps.forceAllowDeactivateTime < level_time
            && (*cl).ps.fd.forcePowersActive & (1 << FP_TELEPATHY) != 0
        {
            WP_ForcePowerStop(ctx, self_, FP_TELEPATHY);
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_TELEPATHY) == 0 {
            return;
        }

        if ForceTelepathyCheckDirectNPCTarget(ctx, self_, &mut tr, &mut tookPower) != 0 {
            //hit an NPC directly
            (*cl).ps.forceAllowDeactivateTime = level_time + 1500;
            let snd = std::ffi::CString::new("sound/weapons/force/distract.wav").unwrap();
            G_Sound(ctx, self_, CHAN_AUTO, G_SoundIndex(snd.as_ptr()));
            (*cl).ps.forceHandExtend = HANDEXTEND_FORCEPUSH as c_int;
            (*cl).ps.forceHandExtendTime = level_time + 1000;
            return;
        }

        if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_2 {
            visionArc = 180.0;
        } else if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_3 {
            visionArc = 360.0;
            radius = MAX_TRICK_DISTANCE as f32 * 2.0;
        }

        let fwdangles: vec3_t = (*cl).ps.viewangles;
        let mut forward: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        AngleVectors(fwdangles, Some(&mut forward), Some(&mut right), None);
        let center: vec3_t = (*cl).ps.origin;

        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        for i in 0..3 {
            mins[i] = center[i] - radius;
            maxs[i] = center[i] + radius;
        }

        if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_1 {
            let ent = &mut (*ctx.world).entities[tr.entityNum as usize] as *mut gentity_t;
            if tr.fraction != 1.0
                && tr.entityNum != ENTITYNUM_NONE
                && (*ent).inuse != 0
                && !(*ent).client.is_null()
                && (*((*ent).client as *mut gclient_t)).pers.connected != 0
                && (*((*ent).client as *mut gclient_t)).sess.sessionTeam != TEAM_SPECTATOR
            {
                WP_AddAsMindtricked(&mut (*cl).ps.fd, tr.entityNum);
                if tookPower == 0 {
                    WP_ForcePowerStart(ctx, self_, FP_TELEPATHY, 0);
                }

                let snd = std::ffi::CString::new("sound/weapons/force/distract.wav").unwrap();
                G_Sound(ctx, self_, CHAN_AUTO, G_SoundIndex(snd.as_ptr()));

                (*cl).ps.forceHandExtend = HANDEXTEND_FORCEPUSH as c_int;
                (*cl).ps.forceHandExtendTime = level_time + 1000;
            }
        } else {
            //level 2 & 3
            let mut entityList = [0i32; MAX_GENTITIES as usize];
            let mut gotatleastone: qboolean = qfalse;

            let numListedEntities = trap::EntitiesInBox(
                ctx.engine,
                GEntitiesInBoxArgs::new(
                    &mins as *const vec3_t,
                    &maxs as *const vec3_t,
                    entityList.as_mut_ptr(),
                    MAX_GENTITIES as c_int,
                ),
            );

            for e in 0..numListedEntities {
                let mut ent = &mut (*ctx.world).entities[entityList[e as usize] as usize]
                    as *mut gentity_t;

                {
                    let mut thispush_org: vec3_t;
                    if !(*ent).client.is_null() {
                        thispush_org = (*((*ent).client as *mut gclient_t)).ps.origin;
                    } else {
                        thispush_org = (*ent).s.pos.trBase;
                    }
                    let mut tto: vec3_t = (*cl).ps.origin;
                    tto[2] += (*cl).ps.viewheight as f32;
                    let mut a: vec3_t = [
                        thispush_org[0] - tto[0],
                        thispush_org[1] - tto[1],
                        thispush_org[2] - tto[2],
                    ];
                    let a_in = a;
                    vectoangles(a_in, &mut a);

                    if (*ent).client.is_null() {
                        entityList[e as usize] = ENTITYNUM_NONE;
                    } else if InFieldOfVision((*cl).ps.viewangles, visionArc, a) == 0 {
                        //only bother with arc rules if the victim is a client
                        entityList[e as usize] = ENTITYNUM_NONE;
                    } else if ForcePowerUsableOn(ctx, self_, ent, FP_TELEPATHY) == 0 {
                        entityList[e as usize] = ENTITYNUM_NONE;
                    } else if OnSameTeam(ctx, self_, ent) != 0 {
                        entityList[e as usize] = ENTITYNUM_NONE;
                    }
                }
                ent = &mut (*ctx.world).entities[entityList[e as usize] as usize] as *mut gentity_t;
                if ent != self_ && !(*ent).client.is_null() {
                    gotatleastone = qtrue;
                    WP_AddAsMindtricked(&mut (*cl).ps.fd, (*ent).s.number);
                }
            }

            if gotatleastone != 0 {
                (*cl).ps.forceAllowDeactivateTime = level_time + 1500;

                if tookPower == 0 {
                    WP_ForcePowerStart(ctx, self_, FP_TELEPATHY, 0);
                }

                let snd = std::ffi::CString::new("sound/weapons/force/distract.wav").unwrap();
                G_Sound(ctx, self_, CHAN_AUTO, G_SoundIndex(snd.as_ptr()));

                (*cl).ps.forceHandExtend = HANDEXTEND_FORCEPUSH as c_int;
                (*cl).ps.forceHandExtendTime = level_time + 1000;
            }
        }
    }
}

/// Raven `GEntity_UseFunc`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2895-2898`
pub fn GEntity_UseFunc(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    GlobalUse(self_, other, activator);
}

/// Raven `CanCounterThrow`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2900-2968`
pub fn CanCounterThrow(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    thrower: *mut gentity_t,
    pull: qboolean,
) -> qboolean {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;
        let powerUse: forcePowers_t;

        if (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int {
            return 0;
        }

        if (*cl).ps.weaponTime > 0 {
            return 0;
        }

        if (*self_).health <= 0 {
            return 0;
        }

        if (*cl).ps.powerups[PW_DISINT_4 as usize] > level_time {
            return 0;
        }

        if (*cl).ps.weaponstate == WEAPON_CHARGING as c_int
            || (*cl).ps.weaponstate == WEAPON_CHARGING_ALT as c_int
        {
            //don't autodefend when charging a weapon
            return 0;
        }

        if (*ctx.world).cvars.g_gametype.integer == GT_SIEGE
            && pull != 0
            && !thrower.is_null()
            && !(*thrower).client.is_null()
        {
            //in siege, pull will affect people if they are not facing you, so they can't run away so much
            let tcl = (*thrower).client as *mut gclient_t;
            let mut d: vec3_t = [
                (*tcl).ps.origin[0] - (*cl).ps.origin[0],
                (*tcl).ps.origin[1] - (*cl).ps.origin[1],
                (*tcl).ps.origin[2] - (*cl).ps.origin[2],
            ];
            let d_in = d;
            vectoangles(d_in, &mut d);

            let a = AngleSubtract(d[YAW], (*cl).ps.viewangles[YAW]);

            if a > 60.0 || a < -60.0 {
                //if facing more than 60 degrees away they cannot defend
                return 0;
            }
        }

        if pull != 0 {
            powerUse = FP_PULL;
        } else {
            powerUse = FP_PUSH;
        }

        if WP_ForcePowerUsable(ctx, self_, powerUse) == 0 {
            return 0;
        }

        if (*cl).ps.groundEntityNum == ENTITYNUM_NONE {
            //you cannot counter a push/pull if you're in the air
            return 0;
        }

        1
    }
}

/// Raven `G_InGetUpAnim`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2970-3023`
pub fn G_InGetUpAnim(ps: *mut playerState_t) -> qboolean {
    unsafe {
        let legs = (*ps).legsAnim;
        if legs == BOTH_GETUP1 as c_int
            || legs == BOTH_GETUP2 as c_int
            || legs == BOTH_GETUP3 as c_int
            || legs == BOTH_GETUP4 as c_int
            || legs == BOTH_GETUP5 as c_int
            || legs == BOTH_FORCE_GETUP_F1 as c_int
            || legs == BOTH_FORCE_GETUP_F2 as c_int
            || legs == BOTH_FORCE_GETUP_B1 as c_int
            || legs == BOTH_FORCE_GETUP_B2 as c_int
            || legs == BOTH_FORCE_GETUP_B3 as c_int
            || legs == BOTH_FORCE_GETUP_B4 as c_int
            || legs == BOTH_FORCE_GETUP_B5 as c_int
            || legs == BOTH_GETUP_BROLL_B as c_int
            || legs == BOTH_GETUP_BROLL_F as c_int
            || legs == BOTH_GETUP_BROLL_L as c_int
            || legs == BOTH_GETUP_BROLL_R as c_int
            || legs == BOTH_GETUP_FROLL_B as c_int
            || legs == BOTH_GETUP_FROLL_F as c_int
            || legs == BOTH_GETUP_FROLL_L as c_int
            || legs == BOTH_GETUP_FROLL_R as c_int
        {
            return qtrue;
        }

        let torso = (*ps).torsoAnim;
        if torso == BOTH_GETUP1 as c_int
            || torso == BOTH_GETUP2 as c_int
            || torso == BOTH_GETUP3 as c_int
            || torso == BOTH_GETUP4 as c_int
            || torso == BOTH_GETUP5 as c_int
            || torso == BOTH_FORCE_GETUP_F1 as c_int
            || torso == BOTH_FORCE_GETUP_F2 as c_int
            || torso == BOTH_FORCE_GETUP_B1 as c_int
            || torso == BOTH_FORCE_GETUP_B2 as c_int
            || torso == BOTH_FORCE_GETUP_B3 as c_int
            || torso == BOTH_FORCE_GETUP_B4 as c_int
            || torso == BOTH_FORCE_GETUP_B5 as c_int
            || torso == BOTH_GETUP_BROLL_B as c_int
            || torso == BOTH_GETUP_BROLL_F as c_int
            || torso == BOTH_GETUP_BROLL_L as c_int
            || torso == BOTH_GETUP_BROLL_R as c_int
            || torso == BOTH_GETUP_FROLL_B as c_int
            || torso == BOTH_GETUP_FROLL_F as c_int
            || torso == BOTH_GETUP_FROLL_L as c_int
            || torso == BOTH_GETUP_FROLL_R as c_int
        {
            return qtrue;
        }

        qfalse
    }
}

/// Raven `G_LetGoOfWall`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:3025-3042`
pub fn G_LetGoOfWall(ctx: GameContext<'_>, ent: *mut gentity_t) {
    unsafe {
        if ent.is_null() || (*ent).client.is_null() {
            return;
        }
        let cl = (*ent).client as *mut gclient_t;
        (*cl).ps.pm_flags &= !PMF_STUCK_TO_WALL;
        if BG_InReboundJump((*cl).ps.legsAnim) != 0 || BG_InReboundHold((*cl).ps.legsAnim) != 0 {
            (*cl).ps.legsTimer = 0;
        }
        if BG_InReboundJump((*cl).ps.torsoAnim) != 0 || BG_InReboundHold((*cl).ps.torsoAnim) != 0 {
            (*cl).ps.torsoTimer = 0;
        }
    }
}

/// Raven `ForceThrow`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:3054-3820`
// PORT-ESCALATION(unported-global-and-vehicle-vtable): reads the un-ported
// `forcePowerNeeded` table (fork-5), calls the fork-7 vehicle vtable
// (`vehEnt->m_pVehicle->m_pVehicleInfo->Eject`, not in the resolved call surface),
// and uses `VectorCompare` (marked unresolved in the packet). Multiple genuinely
// un-ported deps — parked.
pub fn ForceThrow(ctx: GameContext<'_>, self_: *mut gentity_t, pull: qboolean) {
    todo!("Port ForceThrow — parked: unported-global (forcePowerNeeded) + vehicle-vtable (Eject) + VectorCompare")
}

/// Raven `WP_ForcePowerStop`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:3822-3946`
pub fn WP_ForcePowerStop(ctx: GameContext<'_>, self_: *mut gentity_t, forcePower: forcePowers_t) {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;
        let wasActive = (*cl).ps.fd.forcePowersActive;

        (*cl).ps.fd.forcePowersActive &= !(1 << forcePower);

        match forcePower {
            FP_HEAL => {
                (*cl).ps.fd.forceHealAmount = 0;
                (*cl).ps.fd.forceHealTime = 0;
            }
            FP_LEVITATION => {}
            FP_SPEED => {
                if wasActive & (1 << FP_SPEED) != 0 {
                    G_MuteSound(
                        ctx,
                        (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_2 as c_int - 50) as usize],
                        CHAN_VOICE,
                    );
                }
            }
            FP_PUSH => {}
            FP_PULL => {}
            FP_TELEPATHY => {
                if wasActive & (1 << FP_TELEPATHY) != 0 {
                    let snd =
                        std::ffi::CString::new("sound/weapons/force/distractstop.wav").unwrap();
                    G_Sound(ctx, self_, CHAN_AUTO, G_SoundIndex(snd.as_ptr()));
                }
                (*cl).ps.fd.forceMindtrickTargetIndex = 0;
                (*cl).ps.fd.forceMindtrickTargetIndex2 = 0;
                (*cl).ps.fd.forceMindtrickTargetIndex3 = 0;
                (*cl).ps.fd.forceMindtrickTargetIndex4 = 0;
            }
            FP_SEE => {
                if wasActive & (1 << FP_SEE) != 0 {
                    G_MuteSound(
                        ctx,
                        (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_5 as c_int - 50) as usize],
                        CHAN_VOICE,
                    );
                }
            }
            FP_GRIP => {
                (*cl).ps.fd.forceGripUseTime = level_time + 3000;
                let gripIdx = (*cl).ps.fd.forceGripEntityNum as usize;
                let gripEnt = &mut (*ctx.world).entities[gripIdx] as *mut gentity_t;
                if (*cl).ps.fd.forcePowerLevel[FP_GRIP as usize] > FORCE_LEVEL_1
                    && !(*gripEnt).client.is_null()
                    && (*gripEnt).health > 0
                    && (*gripEnt).inuse != 0
                    && (level_time
                        - (*((*gripEnt).client as *mut gclient_t)).ps.fd.forceGripStarted)
                        > 500
                {
                    //if we had our throat crushed in for more than half a second, gasp for air when we're let go
                    if wasActive & (1 << FP_GRIP) != 0 {
                        let snd = std::ffi::CString::new("*gasp.wav").unwrap();
                        G_EntitySound(ctx, gripEnt, CHAN_VOICE, G_SoundIndex(snd.as_ptr()));
                    }
                }

                if !(*gripEnt).client.is_null() && (*gripEnt).inuse != 0 {
                    (*((*gripEnt).client as *mut gclient_t)).ps.forceGripChangeMovetype =
                        PM_NORMAL as c_int;
                }

                if (*cl).ps.forceHandExtend == HANDEXTEND_FORCE_HOLD as c_int {
                    (*cl).ps.forceHandExtendTime = 0;
                }

                (*cl).ps.fd.forceGripEntityNum = ENTITYNUM_NONE;

                (*cl).ps.powerups[PW_DISINT_4 as usize] = 0;
            }
            FP_LIGHTNING => {
                if (*cl).ps.fd.forcePowerLevel[FP_LIGHTNING as usize] < FORCE_LEVEL_2 {
                    //don't do it again for 3 seconds, minimum...
                    (*cl).ps.fd.forcePowerDebounce[FP_LIGHTNING as usize] = level_time + 3000;
                } else {
                    (*cl).ps.fd.forcePowerDebounce[FP_LIGHTNING as usize] = level_time + 1500;
                }
                if (*cl).ps.forceHandExtend == HANDEXTEND_FORCE_HOLD as c_int {
                    (*cl).ps.forceHandExtendTime = 0; //reset hand position
                }

                (*cl).ps.activeForcePass = 0;
            }
            FP_RAGE => {
                (*cl).ps.fd.forceRageRecoveryTime = level_time + 10000;
                if wasActive & (1 << FP_RAGE) != 0 {
                    G_MuteSound(
                        ctx,
                        (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_3 as c_int - 50) as usize],
                        CHAN_VOICE,
                    );
                }
            }
            FP_ABSORB => {
                if wasActive & (1 << FP_ABSORB) != 0 {
                    G_MuteSound(
                        ctx,
                        (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_3 as c_int - 50) as usize],
                        CHAN_VOICE,
                    );
                }
            }
            FP_PROTECT => {
                if wasActive & (1 << FP_PROTECT) != 0 {
                    G_MuteSound(
                        ctx,
                        (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_3 as c_int - 50) as usize],
                        CHAN_VOICE,
                    );
                }
            }
            FP_DRAIN => {
                if (*cl).ps.fd.forcePowerLevel[FP_DRAIN as usize] < FORCE_LEVEL_2 {
                    //don't do it again for 3 seconds, minimum...
                    (*cl).ps.fd.forcePowerDebounce[FP_DRAIN as usize] = level_time + 3000;
                } else {
                    (*cl).ps.fd.forcePowerDebounce[FP_DRAIN as usize] = level_time + 1500;
                }

                if (*cl).ps.forceHandExtend == HANDEXTEND_FORCE_HOLD as c_int {
                    (*cl).ps.forceHandExtendTime = 0; //reset hand position
                }

                (*cl).ps.activeForcePass = 0;
            }
            _ => {}
        }
    }
}

/// Raven `DoGripAction`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:3948-4162`
// PORT-ESCALATION(unported-global-table): reads `forcePowerNeeded[level][power]`
// (fork-5 const table not yet ported; values absent from packet). Parked like
// the other `forcePowerNeeded` consumers.
pub fn DoGripAction(ctx: GameContext<'_>, self_: *mut gentity_t, forcePower: forcePowers_t) {
    todo!("Port DoGripAction — parked: unported-global (forcePowerNeeded)")
}

/// Raven `G_IsMindTricked` — is `client` in one of `fd`'s mindtrick masks?
///
/// Source: `oracle/oracle/codemp/game/w_force.c:4164-4206`
pub fn G_IsMindTricked(fd: *mut forcedata_t, client: c_int) -> qboolean {
    unsafe {
        let checkIn;
        let mut sub = 0;

        if fd.is_null() {
            return qfalse;
        }

        let trickIndex1 = (*fd).forceMindtrickTargetIndex;
        let trickIndex2 = (*fd).forceMindtrickTargetIndex2;
        let trickIndex3 = (*fd).forceMindtrickTargetIndex3;
        let trickIndex4 = (*fd).forceMindtrickTargetIndex4;

        if client > 47 {
            checkIn = trickIndex4;
            sub = 48;
        } else if client > 31 {
            checkIn = trickIndex3;
            sub = 32;
        } else if client > 15 {
            checkIn = trickIndex2;
            sub = 16;
        } else {
            checkIn = trickIndex1;
        }

        if checkIn & (1 << (client - sub)) != 0 {
            return qtrue;
        }

        qfalse
    }
}

/// Raven `RemoveTrickedEnt` — clear `client` from `fd`'s mindtrick masks.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:4208-4231`
fn RemoveTrickedEnt(fd: *mut forcedata_t, client: c_int) {
    unsafe {
        if fd.is_null() {
            return;
        }

        if client > 47 {
            (*fd).forceMindtrickTargetIndex4 &= !(1 << (client - 48));
        } else if client > 31 {
            (*fd).forceMindtrickTargetIndex3 &= !(1 << (client - 32));
        } else if client > 15 {
            (*fd).forceMindtrickTargetIndex2 &= !(1 << (client - 16));
        } else {
            (*fd).forceMindtrickTargetIndex &= !(1 << client);
        }
    }
}

/// Raven `WP_UpdateMindtrickEnts`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:4236-4280`
fn WP_UpdateMindtrickEnts(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;
        let g_time_since = (*ctx.world).globals.g_TimeSinceLastFrame;
        let gametype = (*ctx.world).cvars.g_gametype.integer;

        let mut i: c_int = 0;
        while i < MAX_CLIENTS as c_int {
            if G_IsMindTricked(&mut (*cl).ps.fd, i) != 0 {
                let ent = &mut (*ctx.world).entities[i as usize] as *mut gentity_t;

                if (*ent).client.is_null()
                    || (*ent).inuse == 0
                    || (*ent).health < 1
                    || ((*((*ent).client as *mut gclient_t)).ps.fd.forcePowersActive
                        & (1 << FP_SEE))
                        != 0
                {
                    RemoveTrickedEnt(&mut (*cl).ps.fd, i);
                } else if (level_time - (*cl).dangerTime) < g_time_since * 4 {
                    //Untrick this entity if the tricker (self) fires while in his fov
                    let ecl = (*ent).client as *mut gclient_t;
                    if trap::InPVS(
                        ctx.engine,
                        GInPvsArgs::new(
                            &(*ecl).ps.origin as *const vec3_t,
                            &(*cl).ps.origin as *const vec3_t,
                        ),
                    ) != 0
                        && OrgVisible(ctx, (*ecl).ps.origin, (*cl).ps.origin, (*ent).s.number) != 0
                    {
                        RemoveTrickedEnt(&mut (*cl).ps.fd, i);
                    }
                } else if BG_HasYsalamiri(gametype, &mut (*((*ent).client as *mut gclient_t)).ps) != 0
                {
                    RemoveTrickedEnt(&mut (*cl).ps.fd, i);
                }
            }

            i += 1;
        }

        if (*cl).ps.fd.forceMindtrickTargetIndex == 0
            && (*cl).ps.fd.forceMindtrickTargetIndex2 == 0
            && (*cl).ps.fd.forceMindtrickTargetIndex3 == 0
            && (*cl).ps.fd.forceMindtrickTargetIndex4 == 0
        {
            //everyone who we had tricked is no longer tricked, so stop the power
            WP_ForcePowerStop(ctx, self_, FP_TELEPATHY);
        } else if (*cl).ps.powerups[PW_REDFLAG as usize] != 0
            || (*cl).ps.powerups[PW_BLUEFLAG as usize] != 0
        {
            WP_ForcePowerStop(ctx, self_, FP_TELEPATHY);
        }
    }
}

/// Raven `WP_ForcePowerRun`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:4282-4506`
fn WP_ForcePowerRun(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    forcePower: forcePowers_t,
    cmd: *mut usercmd_t,
) {
    // Raven declares `extern usercmd_t ucmd;` here but never references it.
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;

        match forcePower {
            FP_HEAL => {
                if (*cl).ps.fd.forcePowerLevel[FP_HEAL as usize] == FORCE_LEVEL_1
                    && ((*cl).ps.velocity[0] != 0.0
                        || (*cl).ps.velocity[1] != 0.0
                        || (*cl).ps.velocity[2] != 0.0)
                {
                    WP_ForcePowerStop(ctx, self_, forcePower);
                    return;
                }

                if (*self_).health < 1 || (*cl).ps.stats[STAT_HEALTH as usize] < 1 {
                    WP_ForcePowerStop(ctx, self_, forcePower);
                    return;
                }

                if (*cl).ps.fd.forceHealTime > level_time {
                    return;
                }
                if (*self_).health > (*cl).ps.stats[STAT_MAX_HEALTH as usize] {
                    //we might start out over max_health and we don't want force heal taking us down
                    WP_ForcePowerStop(ctx, self_, forcePower);
                    return;
                }
                (*cl).ps.fd.forceHealTime = level_time + 1000;
                (*self_).health += 1;
                (*cl).ps.fd.forceHealAmount += 1;

                if (*self_).health > (*cl).ps.stats[STAT_MAX_HEALTH as usize] {
                    (*self_).health = (*cl).ps.stats[STAT_MAX_HEALTH as usize];
                    WP_ForcePowerStop(ctx, self_, forcePower);
                }

                if ((*cl).ps.fd.forcePowerLevel[FP_HEAL as usize] == FORCE_LEVEL_1
                    && (*cl).ps.fd.forceHealAmount >= 25)
                    || ((*cl).ps.fd.forcePowerLevel[FP_HEAL as usize] == FORCE_LEVEL_2
                        && (*cl).ps.fd.forceHealAmount >= 33)
                {
                    WP_ForcePowerStop(ctx, self_, forcePower);
                }
            }
            FP_SPEED => {
                //This is handled in PM_WalkMove and PM_StepSlideMove
                if (*cl).holdingObjectiveItem >= MAX_CLIENTS as c_int
                    && (*cl).holdingObjectiveItem < ENTITYNUM_WORLD
                {
                    if (*ctx.world).entities[(*cl).holdingObjectiveItem as usize].genericValue15 != 0
                    {
                        //disables force powers
                        WP_ForcePowerStop(ctx, self_, forcePower);
                    }
                }
            }
            FP_GRIP => {
                if (*cl).ps.forceHandExtend != HANDEXTEND_FORCE_HOLD as c_int {
                    WP_ForcePowerStop(ctx, self_, FP_GRIP);
                    return;
                }

                if (*cl).ps.fd.forcePowerDebounce[FP_PULL as usize] < level_time {
                    //Using the debounce value reserved for pull for this because pull doesn't need it.
                    BG_ForcePowerDrain(&mut (*cl).ps, forcePower, 1);
                    (*cl).ps.fd.forcePowerDebounce[FP_PULL as usize] = level_time + 100;
                }

                if (*cl).ps.fd.forcePower < 1 {
                    WP_ForcePowerStop(ctx, self_, FP_GRIP);
                    return;
                }

                DoGripAction(ctx, self_, forcePower);
            }
            FP_LEVITATION => {
                if (*cl).ps.groundEntityNum != ENTITYNUM_NONE
                    && (*cl).ps.fd.forceJumpZStart == 0
                {
                    //done with jump
                    WP_ForcePowerStop(ctx, self_, forcePower);
                }
            }
            FP_RAGE => {
                if (*self_).health < 1 {
                    WP_ForcePowerStop(ctx, self_, forcePower);
                    return;
                }
                if (*cl).ps.forceRageDrainTime < level_time {
                    let mut addTime = 400;

                    (*self_).health -= 2;

                    if (*cl).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_1 {
                        addTime = 150;
                    } else if (*cl).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_2 {
                        addTime = 300;
                    } else if (*cl).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_3 {
                        addTime = 450;
                    }
                    (*cl).ps.forceRageDrainTime = level_time + addTime;
                }

                if (*self_).health < 1 {
                    (*self_).health = 1;
                    WP_ForcePowerStop(ctx, self_, forcePower);
                }

                (*cl).ps.stats[STAT_HEALTH as usize] = (*self_).health;
            }
            FP_DRAIN => {
                if (*cl).ps.forceHandExtend != HANDEXTEND_FORCE_HOLD as c_int {
                    WP_ForcePowerStop(ctx, self_, forcePower);
                    return;
                }

                if (*cl).ps.fd.forcePowerLevel[FP_DRAIN as usize] > FORCE_LEVEL_1 {
                    //higher than level 1
                    if ((*cmd).buttons & BUTTON_FORCE_DRAIN) != 0
                        || (((*cmd).buttons & BUTTON_FORCEPOWER) != 0
                            && (*cl).ps.fd.forcePowerSelected == FP_DRAIN)
                    {
                        //holding it keeps it going
                        (*cl).ps.fd.forcePowerDuration[FP_DRAIN as usize] = level_time + 500;
                    }
                }
                // OVERRIDEFIXME
                if WP_ForcePowerAvailable(ctx, self_, forcePower, 0) == 0
                    || (*cl).ps.fd.forcePowerDuration[FP_DRAIN as usize] < level_time
                    || (*cl).ps.fd.forcePower < 25
                {
                    WP_ForcePowerStop(ctx, self_, forcePower);
                } else {
                    ForceShootDrain(ctx, self_);
                }
            }
            FP_LIGHTNING => {
                if (*cl).ps.forceHandExtend != HANDEXTEND_FORCE_HOLD as c_int {
                    //once hand starts to go in in animation, lightning should stop
                    WP_ForcePowerStop(ctx, self_, forcePower);
                    return;
                }

                if (*cl).ps.fd.forcePowerLevel[FP_LIGHTNING as usize] > FORCE_LEVEL_1 {
                    //higher than level 1
                    if ((*cmd).buttons & BUTTON_FORCE_LIGHTNING) != 0
                        || (((*cmd).buttons & BUTTON_FORCEPOWER) != 0
                            && (*cl).ps.fd.forcePowerSelected == FP_LIGHTNING)
                    {
                        //holding it keeps it going
                        (*cl).ps.fd.forcePowerDuration[FP_LIGHTNING as usize] = level_time + 500;
                    }
                }
                // OVERRIDEFIXME
                if WP_ForcePowerAvailable(ctx, self_, forcePower, 0) == 0
                    || (*cl).ps.fd.forcePowerDuration[FP_LIGHTNING as usize] < level_time
                    || (*cl).ps.fd.forcePower < 25
                {
                    WP_ForcePowerStop(ctx, self_, forcePower);
                } else {
                    ForceShootLightning(ctx, self_);
                    BG_ForcePowerDrain(&mut (*cl).ps, forcePower, 0);
                }
            }
            FP_TELEPATHY => {
                if (*cl).holdingObjectiveItem >= MAX_CLIENTS as c_int
                    && (*cl).holdingObjectiveItem < ENTITYNUM_WORLD
                    && (*ctx.world).entities[(*cl).holdingObjectiveItem as usize].genericValue15
                        != 0
                {
                    //if force hindered can't mindtrick whilst carrying a siege item
                    WP_ForcePowerStop(ctx, self_, FP_TELEPATHY);
                } else {
                    WP_UpdateMindtrickEnts(ctx, self_);
                }
            }
            FP_SABER_OFFENSE => {}
            FP_SABER_DEFENSE => {}
            FP_SABERTHROW => {}
            FP_PROTECT => {
                if (*cl).ps.fd.forcePowerDebounce[forcePower as usize] < level_time {
                    BG_ForcePowerDrain(&mut (*cl).ps, forcePower, 1);
                    if (*cl).ps.fd.forcePower < 1 {
                        WP_ForcePowerStop(ctx, self_, forcePower);
                    }

                    (*cl).ps.fd.forcePowerDebounce[forcePower as usize] = level_time + 300;
                }
            }
            FP_ABSORB => {
                if (*cl).ps.fd.forcePowerDebounce[forcePower as usize] < level_time {
                    BG_ForcePowerDrain(&mut (*cl).ps, forcePower, 1);
                    if (*cl).ps.fd.forcePower < 1 {
                        WP_ForcePowerStop(ctx, self_, forcePower);
                    }

                    (*cl).ps.fd.forcePowerDebounce[forcePower as usize] = level_time + 600;
                }
            }
            _ => {}
        }
    }
}

/// Raven `WP_DoSpecificPower`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:4508-4671`
pub fn WP_DoSpecificPower(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    ucmd: *mut usercmd_t,
    forcepower: forcePowers_t,
) -> c_int {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;

        let mut powerSucceeded = 1;

        // OVERRIDEFIXME
        if WP_ForcePowerAvailable(ctx, self_, forcepower, 0) == 0 {
            return 0;
        }

        match forcepower {
            FP_HEAL => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceHeal(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_LEVITATION => {
                //if leave the ground by some other means, cancel the force jump
                if (*cl).ps.groundEntityNum == ENTITYNUM_NONE {
                    (*cl).ps.fd.forceJumpCharge = 0.0;
                    G_MuteSound(
                        ctx,
                        (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 as c_int - 50) as usize],
                        CHAN_VOICE,
                    );
                } else {
                    //still on ground, so jump
                    ForceJump(ctx, self_, ucmd);
                }
            }
            FP_SPEED => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceSpeed(ctx, self_, 0);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_GRIP => {
                if (*cl).ps.fd.forceGripEntityNum == ENTITYNUM_NONE {
                    ForceGrip(ctx, self_);
                }

                if (*cl).ps.fd.forceGripEntityNum != ENTITYNUM_NONE {
                    if (*cl).ps.fd.forcePowersActive & (1 << FP_GRIP) == 0 {
                        WP_ForcePowerStart(ctx, self_, FP_GRIP, 0);
                        BG_ForcePowerDrain(&mut (*cl).ps, FP_GRIP, GRIP_DRAIN_AMOUNT);
                    }
                } else {
                    powerSucceeded = 0;
                }
            }
            FP_LIGHTNING => {
                ForceLightning(ctx, self_);
            }
            FP_PUSH => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if !((*cl).ps.fd.forceButtonNeedRelease != 0
                    && ((*self_).r.svFlags & SVF_BOT) == 0)
                {
                    ForceThrow(ctx, self_, qfalse);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_PULL => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceThrow(ctx, self_, qtrue);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_TELEPATHY => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceTelepathy(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_RAGE => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceRage(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_PROTECT => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceProtect(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_ABSORB => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceAbsorb(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_TEAM_HEAL => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceTeamHeal(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_TEAM_FORCE => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceTeamForceReplenish(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_DRAIN => {
                ForceDrain(ctx, self_);
            }
            FP_SEE => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceSeeing(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_SABER_OFFENSE => {}
            FP_SABER_DEFENSE => {}
            FP_SABERTHROW => {}
            _ => {}
        }

        powerSucceeded
    }
}

/// Raven `FindGenericEnemyIndex`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:4673-4709`
pub fn FindGenericEnemyIndex(ctx: GameContext<'_>, self_: *mut gentity_t) {
    //Find another client that would be considered a threat.
    unsafe {
        let scl = (*self_).client as *mut gclient_t;
        let mut besten: *mut gentity_t = core::ptr::null_mut();
        let mut blen: f32 = 99999999.0;

        let mut i: c_int = 0;
        while i < MAX_CLIENTS as c_int {
            let ent = &mut (*ctx.world).entities[i as usize] as *mut gentity_t;

            if !(*ent).client.is_null()
                && (*ent).s.number != (*self_).s.number
                && (*ent).health > 0
                && OnSameTeam(ctx, self_, ent) == 0
                && (*((*ent).client as *mut gclient_t)).ps.pm_type != PM_INTERMISSION as c_int
                && (*((*ent).client as *mut gclient_t)).ps.pm_type != PM_SPECTATOR as c_int
            {
                let ecl = (*ent).client as *mut gclient_t;
                let a: vec3_t = [
                    (*ecl).ps.origin[0] - (*scl).ps.origin[0],
                    (*ecl).ps.origin[1] - (*scl).ps.origin[1],
                    (*ecl).ps.origin[2] - (*scl).ps.origin[2],
                ];
                let tlen = VectorLength(a);

                if tlen < blen
                    && InFront((*ecl).ps.origin, (*scl).ps.origin, (*scl).ps.viewangles, 0.8) != 0
                    && OrgVisible(ctx, (*scl).ps.origin, (*ecl).ps.origin, (*self_).s.number) != 0
                {
                    blen = tlen;
                    besten = ent;
                }
            }

            i += 1;
        }

        if besten.is_null() {
            return;
        }

        (*scl).ps.genericEnemyIndex = (*besten).s.number;
    }
}

/// Raven `SeekerDroneUpdate`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:4711-4868`
pub fn SeekerDroneUpdate(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;

        if (*cl).ps.eFlags & EF_SEEKERDRONE == 0 {
            (*cl).ps.genericEnemyIndex = -1;
            return;
        }

        if (*self_).health < 1 {
            let mut elevated: vec3_t = (*cl).ps.origin;
            elevated[2] += 40.0;

            let angle = ((level_time / 12) & 255) as f64 * (M_PI * 2.0) / 255.0;
            let dir: vec3_t = [
                (angle.cos() * 20.0) as f32,
                (angle.sin() * 20.0) as f32,
                (angle.cos() * 5.0) as f32,
            ];
            let org: vec3_t = [
                elevated[0] + dir[0],
                elevated[1] + dir[1],
                elevated[2] + dir[2],
            ];

            let mut a: vec3_t = [0.0; 3];
            a[ROLL] = 0.0;
            a[YAW] = 0.0;
            a[PITCH] = 1.0;

            G_PlayEffect(EFFECT_SPARK_EXPLOSION as c_int, org, a);

            (*cl).ps.eFlags -= EF_SEEKERDRONE;
            (*cl).ps.genericEnemyIndex = -1;

            return;
        }

        if (*cl).ps.droneExistTime >= level_time && (*cl).ps.droneExistTime < (level_time + 5000) {
            (*cl).ps.genericEnemyIndex = 1024 + (*cl).ps.droneExistTime;
            if (*cl).ps.droneFireTime < level_time {
                let snd = std::ffi::CString::new("sound/weapons/laser_trap/warning.wav").unwrap();
                G_Sound(ctx, self_, CHAN_BODY, G_SoundIndex(snd.as_ptr()));
                (*cl).ps.droneFireTime = level_time + 100;
            }
            return;
        } else if (*cl).ps.droneExistTime < level_time {
            let mut elevated: vec3_t = (*cl).ps.origin;
            elevated[2] += 40.0;

            let mut prefig = ((*cl).ps.droneExistTime - level_time) / 80;

            if prefig > 55 {
                prefig = 55;
            } else if prefig < 1 {
                prefig = 1;
            }

            elevated[2] -= (55 - prefig) as f32;

            let angle = ((level_time / 12) & 255) as f64 * (M_PI * 2.0) / 255.0;
            let dir: vec3_t = [
                (angle.cos() * 20.0) as f32,
                (angle.sin() * 20.0) as f32,
                (angle.cos() * 5.0) as f32,
            ];
            let org: vec3_t = [
                elevated[0] + dir[0],
                elevated[1] + dir[1],
                elevated[2] + dir[2],
            ];

            let mut a: vec3_t = [0.0; 3];
            a[ROLL] = 0.0;
            a[YAW] = 0.0;
            a[PITCH] = 1.0;

            G_PlayEffect(EFFECT_SPARK_EXPLOSION as c_int, org, a);

            (*cl).ps.eFlags -= EF_SEEKERDRONE;
            (*cl).ps.genericEnemyIndex = -1;

            return;
        }

        if (*cl).ps.genericEnemyIndex == -1 {
            (*cl).ps.genericEnemyIndex = ENTITYNUM_NONE;
        }

        if (*cl).ps.genericEnemyIndex != ENTITYNUM_NONE && (*cl).ps.genericEnemyIndex != -1 {
            let en = &mut (*ctx.world).entities[(*cl).ps.genericEnemyIndex as usize] as *mut gentity_t;

            if (*en).client.is_null() {
                (*cl).ps.genericEnemyIndex = ENTITYNUM_NONE;
            } else if (*en).s.number == (*self_).s.number {
                (*cl).ps.genericEnemyIndex = ENTITYNUM_NONE;
            } else if (*en).health < 1 {
                (*cl).ps.genericEnemyIndex = ENTITYNUM_NONE;
            } else if OnSameTeam(ctx, self_, en) != 0 {
                (*cl).ps.genericEnemyIndex = ENTITYNUM_NONE;
            } else {
                let ecl = (*en).client as *mut gclient_t;
                if InFront((*ecl).ps.origin, (*cl).ps.origin, (*cl).ps.viewangles, 0.8) == 0 {
                    (*cl).ps.genericEnemyIndex = ENTITYNUM_NONE;
                } else if OrgVisible(ctx, (*cl).ps.origin, (*ecl).ps.origin, (*self_).s.number) == 0 {
                    (*cl).ps.genericEnemyIndex = ENTITYNUM_NONE;
                }
            }
        }

        if (*cl).ps.genericEnemyIndex == ENTITYNUM_NONE || (*cl).ps.genericEnemyIndex == -1 {
            FindGenericEnemyIndex(ctx, self_);
        }

        if (*cl).ps.genericEnemyIndex != ENTITYNUM_NONE && (*cl).ps.genericEnemyIndex != -1 {
            let en = &mut (*ctx.world).entities[(*cl).ps.genericEnemyIndex as usize] as *mut gentity_t;

            let mut elevated: vec3_t = (*cl).ps.origin;
            elevated[2] += 40.0;

            let angle = ((level_time / 12) & 255) as f64 * (M_PI * 2.0) / 255.0;
            let dir: vec3_t = [
                (angle.cos() * 20.0) as f32,
                (angle.sin() * 20.0) as f32,
                (angle.cos() * 5.0) as f32,
            ];
            let org: vec3_t = [
                elevated[0] + dir[0],
                elevated[1] + dir[1],
                elevated[2] + dir[2],
            ];

            //org is now where the thing should be client-side because it uses the same time-based offset
            if (*cl).ps.droneFireTime < level_time {
                let ecl = (*en).client as *mut gclient_t;
                let mut tr: trace_t = core::mem::zeroed();
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &org as *const vec3_t,
                        core::ptr::null(),
                        core::ptr::null(),
                        &(*ecl).ps.origin as *const vec3_t,
                        -1,
                        MASK_SOLID,
                    ),
                );

                if tr.fraction == 1.0 && tr.startsolid == 0 && tr.allsolid == 0 {
                    let mut endir: vec3_t = [
                        (*ecl).ps.origin[0] - org[0],
                        (*ecl).ps.origin[1] - org[1],
                        (*ecl).ps.origin[2] - org[2],
                    ];
                    VectorNormalize(&mut endir);

                    WP_FireGenericBlasterMissile(
                        ctx, self_, org, endir, 0, 15, 2000, MOD_BLASTER as c_int,
                    );
                    let snd = std::ffi::CString::new("sound/weapons/bryar/fire.wav").unwrap();
                    G_SoundAtLoc(ctx, org, CHAN_WEAPON, G_SoundIndex(snd.as_ptr()));

                    (*cl).ps.droneFireTime = level_time + Q_irand(400, 700);
                }
            }
        }
    }
}

/// Raven `HolocronUpdate`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:4870-4956`
pub fn HolocronUpdate(ctx: GameContext<'_>, self_: *mut gentity_t) {
    //keep holocron status updated in holocron mode
    unsafe {
        let cl = (*self_).client as *mut gclient_t;

        let mut noHRank = 0;

        if noHRank < FORCE_LEVEL_0 {
            noHRank = FORCE_LEVEL_0;
        }
        if noHRank > FORCE_LEVEL_3 {
            noHRank = FORCE_LEVEL_3;
        }

        trap::Cvar_Update(
            ctx.engine,
            GCvarUpdateArgs::new(&mut (*ctx.world).cvars.g_MaxHolocronCarry as *mut vmCvar_t),
        );

        let mut i = 0;
        while i < NUM_FORCE_POWERS {
            if (*cl).ps.holocronsCarried[i as usize] != 0 {
                //carrying it, make sure we have the power
                (*cl).ps.holocronBits |= 1 << i;
                (*cl).ps.fd.forcePowersKnown |= 1 << i;
                (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_3;
            } else {
                //otherwise, make sure the power is cleared from us
                (*cl).ps.fd.forcePowerLevel[i as usize] = 0;
                if (*cl).ps.holocronBits & (1 << i) != 0 {
                    (*cl).ps.holocronBits -= 1 << i;
                }

                if (*cl).ps.fd.forcePowersKnown & (1 << i) != 0
                    && i != FP_LEVITATION
                    && i != FP_SABER_OFFENSE
                {
                    (*cl).ps.fd.forcePowersKnown -= 1 << i;
                }

                if (*cl).ps.fd.forcePowersActive & (1 << i) != 0
                    && i != FP_LEVITATION
                    && i != FP_SABER_OFFENSE
                {
                    WP_ForcePowerStop(ctx, self_, i);
                }

                if i == FP_LEVITATION {
                    if noHRank >= FORCE_LEVEL_1 {
                        (*cl).ps.fd.forcePowerLevel[i as usize] = noHRank;
                    } else {
                        (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_1;
                    }
                } else if i == FP_SABER_OFFENSE {
                    (*cl).ps.fd.forcePowersKnown |= 1 << i;

                    if noHRank >= FORCE_LEVEL_1 {
                        (*cl).ps.fd.forcePowerLevel[i as usize] = noHRank;
                    } else {
                        (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_1;
                    }
                } else {
                    (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_0;
                }
            }

            i += 1;
        }

        if HasSetSaberOnly(ctx) != 0 {
            //if saberonly, we get these powers no matter what (still need the holocrons for level 3)
            if (*cl).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] < FORCE_LEVEL_1 {
                (*cl).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] = FORCE_LEVEL_1;
            }
            if (*cl).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] < FORCE_LEVEL_1 {
                (*cl).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] = FORCE_LEVEL_1;
            }
        }
    }
}

/// Raven `JediMasterUpdate`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:4958-5011`
pub fn JediMasterUpdate(ctx: GameContext<'_>, self_: *mut gentity_t) {
    //keep jedi master status updated for JM gametype
    unsafe {
        let cl = (*self_).client as *mut gclient_t;

        trap::Cvar_Update(
            ctx.engine,
            GCvarUpdateArgs::new(&mut (*ctx.world).cvars.g_MaxHolocronCarry as *mut vmCvar_t),
        );

        let mut i = 0;
        while i < NUM_FORCE_POWERS {
            if (*cl).ps.isJediMaster != 0 {
                (*cl).ps.fd.forcePowersKnown |= 1 << i;
                (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_3;

                if i == FP_TEAM_HEAL || i == FP_TEAM_FORCE || i == FP_DRAIN || i == FP_ABSORB {
                    //team powers are useless in JM, absorb is too, drain relatively useless
                    (*cl).ps.fd.forcePowersKnown &= !(1 << i);
                    (*cl).ps.fd.forcePowerLevel[i as usize] = 0;
                }

                if i == FP_TELEPATHY {
                    //level 3 mindtrick lets the JM hide too much
                    (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_2;
                }
            } else {
                if (*cl).ps.fd.forcePowersKnown & (1 << i) != 0 && i != FP_LEVITATION {
                    (*cl).ps.fd.forcePowersKnown -= 1 << i;
                }

                if (*cl).ps.fd.forcePowersActive & (1 << i) != 0 && i != FP_LEVITATION {
                    WP_ForcePowerStop(ctx, self_, i);
                }

                if i == FP_LEVITATION {
                    (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_1;
                } else {
                    (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_0;
                }
            }

            i += 1;
        }
    }
}

/// Raven `WP_HasForcePowers` — does `ps` know any non-trivial force power?
///
/// Source: `oracle/oracle/codemp/game/w_force.c:5013-5034`
pub fn WP_HasForcePowers(ps: *const playerState_t) -> qboolean {
    unsafe {
        if !ps.is_null() {
            let mut i = 0;
            while i < NUM_FORCE_POWERS {
                if i == FP_LEVITATION {
                    if (*ps).fd.forcePowerLevel[i as usize] > FORCE_LEVEL_1 {
                        return qtrue;
                    }
                } else if (*ps).fd.forcePowerLevel[i as usize] > FORCE_LEVEL_0 {
                    return qtrue;
                }
                i += 1;
            }
        }
        qfalse
    }
}

/// Raven `G_SpecialRollGetup`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:5037-5092`
pub fn G_SpecialRollGetup(ctx: GameContext<'_>, self_: *mut gentity_t) -> qboolean {
    unsafe {
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;
        let mut rolled: qboolean = qfalse;
        let cmd = &mut (*cl).pers.cmd as *mut usercmd_t;

        if (*cl).pers.cmd.rightmove > 0 && (*cl).pers.cmd.forwardmove == 0 {
            G_SetAnim(
                self_,
                cmd,
                SETANIM_BOTH,
                BOTH_GETUP_BROLL_R as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                0,
            );
            rolled = qtrue;
        } else if (*cl).pers.cmd.rightmove < 0 && (*cl).pers.cmd.forwardmove == 0 {
            G_SetAnim(
                self_,
                cmd,
                SETANIM_BOTH,
                BOTH_GETUP_BROLL_L as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                0,
            );
            rolled = qtrue;
        } else if (*cl).pers.cmd.rightmove == 0 && (*cl).pers.cmd.forwardmove > 0 {
            G_SetAnim(
                self_,
                cmd,
                SETANIM_BOTH,
                BOTH_GETUP_BROLL_F as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                0,
            );
            rolled = qtrue;
        } else if (*cl).pers.cmd.rightmove == 0 && (*cl).pers.cmd.forwardmove < 0 {
            G_SetAnim(
                self_,
                cmd,
                SETANIM_BOTH,
                BOTH_GETUP_BROLL_B as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                0,
            );
            rolled = qtrue;
        } else if (*cl).pers.cmd.upmove != 0 {
            G_PreDefSound(ctx, (*cl).ps.origin, PDSOUND_FORCEJUMP as c_int);
            (*cl).ps.forceDodgeAnim = 2;
            (*cl).ps.forceHandExtendTime = level_time + 500;
        }

        if rolled != 0 {
            let snd = std::ffi::CString::new("*jump1.wav").unwrap();
            G_EntitySound(ctx, self_, CHAN_VOICE, G_SoundIndex(snd.as_ptr()));
        }

        rolled
    }
}

/// Raven `WP_ForcePowersUpdate`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:5094-5671`
// PORT-ESCALATION(unported-global-table): the siege force-regen branch reads
// `bgSiegeClasses[...].classflags` (fork-5 saga class data, not yet ported;
// values absent from packet) and `forcePowerDarkLight` (currently a private
// `const` in `bg_misc.rs`, not exported). Faithful port of those two branches is
// blocked, so the whole fn is parked with its pass-1 siblings.
pub fn WP_ForcePowersUpdate(ctx: GameContext<'_>, self_: *mut gentity_t, ucmd: *mut usercmd_t) {
    todo!("Port WP_ForcePowersUpdate — parked: unported-global (bgSiegeClasses/forcePowerDarkLight)")
}

/// Raven `Jedi_DodgeEvasion`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:5673-5801`
pub fn Jedi_DodgeEvasion(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    shooter: *mut gentity_t,
    tr: *mut trace_t,
    hitLoc: c_int,
) -> qboolean {
    unsafe {
        let mut dodgeAnim: c_int = -1;

        if self_.is_null() || (*self_).client.is_null() || (*self_).health <= 0 {
            return qfalse;
        }
        let cl = (*self_).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;
        let g_forceDodge = (*ctx.world).cvars.g_forceDodge.integer;

        if g_forceDodge == 0 {
            return qfalse;
        }

        if g_forceDodge != 2 {
            if (*cl).ps.fd.forcePowersActive & (1 << FP_SEE) == 0 {
                return qfalse;
            }
        }

        if (*cl).ps.groundEntityNum == ENTITYNUM_NONE {
            //can't dodge in mid-air
            return qfalse;
        }

        if (*cl).ps.weaponTime > 0 || (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int {
            //in some effect that stops me from moving on my own
            return qfalse;
        }

        if g_forceDodge == 2 {
            if (*cl).ps.fd.forcePowersActive != 0 {
                //for now just don't let us dodge if we're using a force power at all
                return qfalse;
            }
        }

        if g_forceDodge == 2 {
            if WP_ForcePowerUsable(ctx, self_, FP_SPEED) == 0 {
                //make sure we have it and have enough force power
                return qfalse;
            }
        }

        if g_forceDodge == 2 {
            if Q_irand(1, 7) > (*cl).ps.fd.forcePowerLevel[FP_SPEED as usize] {
                //more likely to fail on lower force speed level
                return qfalse;
            }
        } else {
            //We now dodge all the time, but only on level 3
            if (*cl).ps.fd.forcePowerLevel[FP_SEE as usize] < FORCE_LEVEL_3 {
                //more likely to fail on lower force sight level
                return qfalse;
            }
        }

        match hitLoc {
            HL_NONE => return qfalse,
            HL_FOOT_RT | HL_FOOT_LT | HL_LEG_RT | HL_LEG_LT => return qfalse,
            HL_BACK_RT => dodgeAnim = BOTH_DODGE_FL as c_int,
            HL_CHEST_RT => dodgeAnim = BOTH_DODGE_FR as c_int,
            HL_BACK_LT => dodgeAnim = BOTH_DODGE_FR as c_int,
            HL_CHEST_LT => dodgeAnim = BOTH_DODGE_FR as c_int,
            HL_BACK | HL_CHEST | HL_WAIST => dodgeAnim = BOTH_DODGE_FL as c_int,
            HL_ARM_RT | HL_HAND_RT => dodgeAnim = BOTH_DODGE_L as c_int,
            HL_ARM_LT | HL_HAND_LT => dodgeAnim = BOTH_DODGE_R as c_int,
            HL_HEAD => dodgeAnim = BOTH_DODGE_FL as c_int,
            _ => return qfalse,
        }

        if dodgeAnim != -1 {
            //Our own happy way of forcing an anim:
            (*cl).ps.forceHandExtend = HANDEXTEND_DODGE as c_int;
            (*cl).ps.forceDodgeAnim = dodgeAnim;
            (*cl).ps.forceHandExtendTime = level_time + 300;

            (*cl).ps.powerups[PW_SPEEDBURST as usize] = level_time + 100;

            if g_forceDodge == 2 {
                ForceSpeed(ctx, self_, 500);
            } else {
                let snd = std::ffi::CString::new("sound/weapons/force/speed.wav").unwrap();
                G_Sound(ctx, self_, CHAN_BODY, G_SoundIndex(snd.as_ptr()));
            }
            return qtrue;
        }
        qfalse
    }
}
