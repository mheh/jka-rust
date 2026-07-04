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
use crate::bg_misc::{BG_CanUseFPNow, BG_HasYsalamiri};
use crate::bg_panimate::{BG_FullBodyTauntAnim, BG_SaberInSpecial};
use crate::bg_pmove::BG_InKnockDown;
use crate::bg_saber::BG_ForcePowerDrain;
use crate::g_combat::G_Damage;
use crate::g_team::OnSameTeam;
use crate::g_utils::{G_Sound, G_SoundIndex, G_TempEntity};
use crate::q_math::{Q_irand, VectorLength};
use crate::w_saber::HasSetSaberOnly;
use crate::trap;
use crate::world::GameContext;

/// Raven `PMF_FOLLOW`/`PMF_STUCK_TO_WALL` (`playerState_t::pm_flags` bits).
/// Source: `oracle/oracle/codemp/game/bg_public.h:415,417`
const PMF_FOLLOW: c_int = 4096;
const PMF_STUCK_TO_WALL: c_int = 16384;

/// Raven `SFL_TWO_HANDED` (`weaponData_t::weaponflags` bit) — uses both hands.
/// Source: `oracle/oracle/codemp/game/q_shared.h:691`
const SFL_TWO_HANDED: c_int = 1 << 4;

use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;
use mp_bg::public::entity_event::entity_event_t::{EV_PREDEFSOUND, EV_TEAM_POWER};

/// Raven `mindTrickTime` per force-mastery level (ms).
///
/// Source: `oracle/oracle/codemp/game/w_force.c:139-145`
const mindTrickTime: [c_int; 4] = [0 /*none*/, 5000, 10000, 15000];

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
    todo!("Port ForceShootLightning — oracle/oracle/codemp/game/w_force.c:1902")
}

/// Raven `ForceDrain`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2022-2056`
pub fn ForceDrain(ctx: GameContext<'_>, self_: *mut gentity_t) {
    todo!("Port ForceDrain — oracle/oracle/codemp/game/w_force.c:2022")
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
    todo!("Port ForceDrainDamage — oracle/oracle/codemp/game/w_force.c:2058")
}

/// Raven `ForceShootDrain`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2184-2315`
pub fn ForceShootDrain(ctx: GameContext<'_>, self_: *mut gentity_t) -> c_int {
    todo!("Port ForceShootDrain — oracle/oracle/codemp/game/w_force.c:2184")
}

/// Raven `ForceJumpCharge`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2317-2375`
pub fn ForceJumpCharge(ctx: GameContext<'_>, self_: *mut gentity_t, ucmd: *mut usercmd_t) {
    todo!("Port ForceJumpCharge — oracle/oracle/codemp/game/w_force.c:2317")
}

/// Raven `WP_GetVelocityForForceJump`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2377-2460`
pub fn WP_GetVelocityForForceJump(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    jumpVel: vec3_t,
    ucmd: *mut usercmd_t,
) -> c_int {
    todo!("Port WP_GetVelocityForForceJump — oracle/oracle/codemp/game/w_force.c:2377")
}

/// Raven `ForceJump`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2462-2500`
pub fn ForceJump(ctx: GameContext<'_>, self_: *mut gentity_t, ucmd: *mut usercmd_t) {
    todo!("Port ForceJump — oracle/oracle/codemp/game/w_force.c:2462")
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
pub fn ForceTelepathyCheckDirectNPCTarget(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    tr: *mut trace_t,
    tookPower: *mut qboolean,
) -> qboolean {
    todo!("Port ForceTelepathyCheckDirectNPCTarget — oracle/oracle/codemp/game/w_force.c:2527")
}

/// Raven `ForceTelepathy`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2723-2893`
pub fn ForceTelepathy(ctx: GameContext<'_>, self_: *mut gentity_t) {
    todo!("Port ForceTelepathy — oracle/oracle/codemp/game/w_force.c:2723")
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
    todo!("Port GEntity_UseFunc — oracle/oracle/codemp/game/w_force.c:2895")
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
    todo!("Port CanCounterThrow — oracle/oracle/codemp/game/w_force.c:2900")
}

/// Raven `G_InGetUpAnim`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:2970-3023`
pub fn G_InGetUpAnim(ps: *mut playerState_t) -> qboolean {
    todo!("Port G_InGetUpAnim — oracle/oracle/codemp/game/w_force.c:2970")
}

/// Raven `G_LetGoOfWall`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:3025-3042`
pub fn G_LetGoOfWall(ctx: GameContext<'_>, ent: *mut gentity_t) {
    todo!("Port G_LetGoOfWall — oracle/oracle/codemp/game/w_force.c:3025")
}

/// Raven `ForceThrow`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:3054-3820`
pub fn ForceThrow(ctx: GameContext<'_>, self_: *mut gentity_t, pull: qboolean) {
    todo!("Port ForceThrow — oracle/oracle/codemp/game/w_force.c:3054")
}

/// Raven `WP_ForcePowerStop`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:3822-3946`
pub fn WP_ForcePowerStop(ctx: GameContext<'_>, self_: *mut gentity_t, forcePower: forcePowers_t) {
    todo!("Port WP_ForcePowerStop — oracle/oracle/codemp/game/w_force.c:3822")
}

/// Raven `DoGripAction`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:3948-4162`
pub fn DoGripAction(ctx: GameContext<'_>, self_: *mut gentity_t, forcePower: forcePowers_t) {
    todo!("Port DoGripAction — oracle/oracle/codemp/game/w_force.c:3948")
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
    todo!("Port WP_UpdateMindtrickEnts — oracle/oracle/codemp/game/w_force.c:4236")
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
    todo!("Port WP_ForcePowerRun — oracle/oracle/codemp/game/w_force.c:4282")
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
    todo!("Port WP_DoSpecificPower — oracle/oracle/codemp/game/w_force.c:4508")
}

/// Raven `FindGenericEnemyIndex`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:4673-4709`
pub fn FindGenericEnemyIndex(ctx: GameContext<'_>, self_: *mut gentity_t) {
    todo!("Port FindGenericEnemyIndex — oracle/oracle/codemp/game/w_force.c:4673")
}

/// Raven `SeekerDroneUpdate`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:4711-4868`
pub fn SeekerDroneUpdate(ctx: GameContext<'_>, self_: *mut gentity_t) {
    todo!("Port SeekerDroneUpdate — oracle/oracle/codemp/game/w_force.c:4711")
}

/// Raven `HolocronUpdate`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:4870-4956`
pub fn HolocronUpdate(ctx: GameContext<'_>, self_: *mut gentity_t) {
    todo!("Port HolocronUpdate — oracle/oracle/codemp/game/w_force.c:4870")
}

/// Raven `JediMasterUpdate`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:4958-5011`
pub fn JediMasterUpdate(ctx: GameContext<'_>, self_: *mut gentity_t) {
    todo!("Port JediMasterUpdate — oracle/oracle/codemp/game/w_force.c:4958")
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
    todo!("Port G_SpecialRollGetup — oracle/oracle/codemp/game/w_force.c:5037")
}

/// Raven `WP_ForcePowersUpdate`.
///
/// Source: `oracle/oracle/codemp/game/w_force.c:5094-5671`
pub fn WP_ForcePowersUpdate(ctx: GameContext<'_>, self_: *mut gentity_t, ucmd: *mut usercmd_t) {
    todo!("Port WP_ForcePowersUpdate — oracle/oracle/codemp/game/w_force.c:5094")
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
    todo!("Port Jedi_DodgeEvasion — oracle/oracle/codemp/game/w_force.c:5673")
}
