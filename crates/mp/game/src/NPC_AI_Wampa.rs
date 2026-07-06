// PORT-COMPLETE: NPC_AI_Wampa.c 1/10
//! Port of `oracle/oracle/codemp/game/NPC_AI_Wampa.c` (jampgame mega-pass).
//!
//! SPINE (fork rulings 1/4): NPC AI think-loop helper functions. Most functions
//! in this file read the implicit NPC/NPCInfo/ucmd bot-AI actor globals that
//! Raven's `ai_main.c` think-loop sets per NPC frame. The faithful skeleton
//! signatures carry no channel to reach these implicit globals (no `GameWorld`/
//! `GameContext` field for "current NPC" and no entity parameter in most cases).
//! This matches the `ai-context` precedent in `NPC_utils.rs`, `NPC_combat.rs`,
//! `NPC_AI_Jedi.rs` — parked pending resolution of how NPC-frame state is
//! threaded to these helpers (topic: `ai-context-threading`).
//!
//! PARKED (see PORT-NOTE markers): 10 functions. Only `NPC_Wampa_Precache`
//! is ported (accesses no implicit globals, only calls G_SoundIndex with a
//! string literal).
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_utils::G_SoundIndex;
use crate::prelude::*;
use crate::trap;
use mp_qshared::common::mp::qcommon::usercmd_button::BUTTON_WALKING;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/oracle/codemp/game/q_shared.h`
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

// These define the working combat range for these suckers
// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:5-9`
const MIN_DISTANCE: c_int = 48;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;
const MAX_DISTANCE: c_int = 1024;
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;

const LSTATE_CLEAR: c_int = 0;
const LSTATE_WAITING: c_int = 1;

// `DistanceSquared` is the canonical `crate::q_math::DistanceSquared`, reached
// via the prelude glob (no per-file copy).

/// Raven `Wampa_SetBolts`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:16-36`
pub fn Wampa_SetBolts(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        if !self_.is_null() && !(*self_).client.is_null() {
            let ri = &mut (*((*self_).client as *mut gclient_t)).renderInfo;
            ri.headBolt = trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                    (*self_).ghoul2,
                    0,
                    c"*head_eyes".to_owned(),
                ),
            );
            ri.torsoBolt = trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                    (*self_).ghoul2,
                    0,
                    c"lower_spine".to_owned(),
                ),
            );
            ri.crotchBolt = trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                    (*self_).ghoul2,
                    0,
                    c"rear_bone".to_owned(),
                ),
            );
            ri.handLBolt = trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                    (*self_).ghoul2,
                    0,
                    c"*l_hand".to_owned(),
                ),
            );
            ri.handRBolt = trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                    (*self_).ghoul2,
                    0,
                    c"*r_hand".to_owned(),
                ),
            );
            ri.footLBolt = trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                    (*self_).ghoul2,
                    0,
                    c"*l_leg_foot".to_owned(),
                ),
            );
            ri.footRBolt = trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                    (*self_).ghoul2,
                    0,
                    c"*r_leg_foot".to_owned(),
                ),
            );
        }
    }
}

/// Raven `NPC_Wampa_Precache`.
///
/// Precaches the swipe-hit sound. All growl/snort variants are commented out
/// in the oracle source (oracle/oracle/codemp/game/NPC_AI_Wampa.c:45-55).
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:43-58`
pub fn NPC_Wampa_Precache(ctx: GameContext<'_>) {
    // Only the swipe sound is live; growl/snort loops are commented out
    G_SoundIndex(b"sound/chars/rancor/swipehit.wav\0".as_ptr() as *const c_char);
}

/// Raven `Wampa_Idle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:66-76`
pub fn Wampa_Idle(ctx: GameContext<'_>) {
    unsafe {
        let npc_info = (*ctx.world).globals.NPCInfo;
        if !npc_info.is_null() {
            (*npc_info).localState = LSTATE_CLEAR;
        }

        //If we have somewhere to go, then do that
        if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
            (*ctx.world).globals.ucmd.buttons &= !BUTTON_WALKING;
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
        }
    }
}

/// Raven `Wampa_CheckRoar`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:78-88`
pub fn Wampa_CheckRoar(ctx: GameContext<'_>, self_: *mut gentity_t) -> qboolean {
    unsafe {
        let level_time = (*ctx.world).level.time as f32;
        if (*self_).wait < level_time {
            (*self_).wait = level_time + (*ctx.world).bg_state.rng.Q_irand(5000, 20000) as f32;
            crate::npc_c::NPC_SetAnim(
                ctx,
                self_,
                SETANIM_BOTH,
                (*ctx.world).bg_state.rng.Q_irand(
                    crate::prelude::BOTH_GESTURE1 as c_int,
                    crate::prelude::BOTH_GESTURE2 as c_int,
                ),
                (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD),
            );
            crate::g_timer::TIMER_Set(
                ctx,
                self_,
                c"rageTime".as_ptr(),
                (*((*self_).client as *mut gclient_t)).ps.legsTimer,
            );
            return qtrue;
        }
        qfalse
    }
}

/// Raven `Wampa_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:94-119`
pub fn Wampa_Patrol(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        if !npc_info.is_null() {
            (*npc_info).localState = LSTATE_CLEAR;
        }

        //If we have somewhere to go, then do that
        if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
            (*ctx.world).globals.ucmd.buttons |= BUTTON_WALKING;
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
        } else {
            if crate::g_timer::TIMER_Done(ctx, npc, c"patrolTime".as_ptr()) != 0 {
                crate::g_timer::TIMER_Set(
                    ctx,
                    npc,
                    c"patrolTime".as_ptr(),
                    ((*ctx.world).bg_state.rng.crandom() * 5000.0 + 5000.0) as c_int,
                );
            }
        }

        if crate::NPC_utils::NPC_CheckEnemyExt(ctx, qtrue) == qfalse {
            Wampa_Idle(ctx);
            return;
        }
        Wampa_CheckRoar(ctx, npc);
        crate::g_timer::TIMER_Set(
            ctx,
            npc,
            c"lookForNewEnemy".as_ptr(),
            (*ctx.world).bg_state.rng.Q_irand(5000, 15000),
        );
    }
}

/// Raven `Wampa_Move`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:126-169`
pub fn Wampa_Move(ctx: GameContext<'_>, visible: qboolean) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        if !npc_info.is_null() && (*npc_info).localState != LSTATE_WAITING {
            (*npc_info).goalEntity = (*npc).enemy;

            if !(*npc).enemy.is_none() {
                // pick correct movement speed and anim
                // run by default
                (*ctx.world).globals.ucmd.buttons &= !BUTTON_WALKING;
                if crate::g_timer::TIMER_Done(ctx, npc, c"runfar".as_ptr()) == 0
                    || crate::g_timer::TIMER_Done(ctx, npc, c"runclose".as_ptr()) == 0
                {
                    // keep running with this anim & speed for a bit
                } else if crate::g_timer::TIMER_Done(ctx, npc, c"walk".as_ptr()) == 0 {
                    // keep walking for a bit
                    (*ctx.world).globals.ucmd.buttons |= BUTTON_WALKING;
                } else if visible != 0
                    && (*ctx.world).globals.enemyDist > 384.0
                    && (*npc_info).stats.runSpeed == 180
                {
                    // fast run, all fours
                    (*npc_info).stats.runSpeed = 300;
                    crate::g_timer::TIMER_Set(
                        ctx,
                        npc,
                        c"runfar".as_ptr(),
                        (*ctx.world).bg_state.rng.Q_irand(2000, 4000),
                    );
                } else if (*ctx.world).globals.enemyDist > 256.0
                    && (*npc_info).stats.runSpeed == 300
                {
                    // slow run, upright
                    (*npc_info).stats.runSpeed = 180;
                    crate::g_timer::TIMER_Set(
                        ctx,
                        npc,
                        c"runclose".as_ptr(),
                        (*ctx.world).bg_state.rng.Q_irand(3000, 5000),
                    );
                } else if (*ctx.world).globals.enemyDist < 128.0 {
                    // walk
                    (*npc_info).stats.runSpeed = 180;
                    (*ctx.world).globals.ucmd.buttons |= BUTTON_WALKING;
                    crate::g_timer::TIMER_Set(
                        ctx,
                        npc,
                        c"walk".as_ptr(),
                        (*ctx.world).bg_state.rng.Q_irand(4000, 6000),
                    );
                }
            }

            if (*npc_info).stats.runSpeed == 300 {
                // need to use the alternate run - hunched over on all fours
                (*((*npc).client as *mut gclient_t)).ps.eFlags2 |=
                    mp_bg::public::entity_effects::EF2_USE_ALT_ANIM;
            }
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
            (*npc_info).goalRadius = MAX_DISTANCE; // just get us within combat range
        }
    }
}

/// Raven `Wampa_Slash`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:177-264`
pub fn Wampa_Slash(ctx: GameContext<'_>, boltIndex: c_int, backhand: qboolean) {
    unsafe {
        let mut radiusEntNums: [c_int; 128] = [0; 128];
        let radius = 88.0f32;
        let radiusSquared = radius * radius;
        let mut boltOrg: [f32; 3] = [0.0; 3];

        let numEnts = crate::NPC_utils::NPC_GetEntsNearBolt(
            ctx,
            radiusEntNums.as_mut_ptr(),
            radius,
            boltIndex,
            &mut boltOrg,
        );

        for i in 0..(numEnts as usize) {
            let radiusEnt = (*ctx.world)
                .g_entities
                .get_unchecked_mut(radiusEntNums[i] as usize)
                as *mut gentity_t;
            if (*radiusEnt).inuse == 0 {
                continue;
            }

            let npc = (*ctx.world).globals.NPC;
            if radiusEnt == npc {
                // Skip the wampa ent
                continue;
            }

            if (*radiusEnt).client.is_null() {
                // must be a client
                continue;
            }

            if DistanceSquared((*radiusEnt).r.currentOrigin, boltOrg) <= radiusSquared {
                // smack
                // Raven passes the global `vec3_origin` as `dir`; G_Damage
                // normalizes `dir` in place (a no-op on the zero vector), so a
                // fresh local copy is behaviorally identical.
                let mut origin = vec3_origin;
                crate::g_combat::G_Damage(
                    ctx,
                    radiusEnt,
                    npc,
                    npc,
                    Some(&mut origin),
                    (*radiusEnt).r.currentOrigin,
                    if backhand != 0 {
                        (*ctx.world).bg_state.rng.Q_irand(10, 15)
                    } else {
                        (*ctx.world).bg_state.rng.Q_irand(20, 30)
                    },
                    if backhand != 0 {
                        crate::prelude::DAMAGE_NO_ARMOR
                    } else {
                        crate::prelude::DAMAGE_NO_ARMOR | crate::prelude::DAMAGE_NO_KNOCKBACK
                    },
                    crate::prelude::MOD_MELEE as c_int,
                );
                if backhand != 0 {
                    // actually push the enemy
                    let mut pushDir: [f32; 3] = [0.0; 3];
                    let mut angs: [f32; 3] = [0.0; 3];
                    crate::q_math::_VectorCopy(
                        (*((*npc).client as *mut gclient_t)).ps.viewangles,
                        &mut angs,
                    );
                    angs[crate::prelude::YAW as usize] +=
                        (*ctx.world).bg_state.rng.flrand(25.0, 50.0);
                    angs[crate::prelude::PITCH as usize] =
                        (*ctx.world).bg_state.rng.flrand(-25.0, -15.0);
                    crate::q_math::AngleVectors(angs, Some(&mut pushDir), None, None);
                    if (*((*radiusEnt).client as *mut gclient_t)).NPC_class
                        != crate::prelude::CLASS_WAMPA
                        && (*((*radiusEnt).client as *mut gclient_t)).NPC_class
                            != crate::prelude::CLASS_RANCOR
                        && (*((*radiusEnt).client as *mut gclient_t)).NPC_class
                            != crate::prelude::CLASS_ATST
                    {
                        crate::g_utils::G_Throw(ctx, radiusEnt, pushDir, 65.0);
                        if crate::bg_pmove::BG_KnockDownable(
                            &mut (*((*radiusEnt).client as *mut gclient_t)).ps as *mut _,
                        ) != 0
                            && (*radiusEnt).health > 0
                            && (*ctx.world).bg_state.rng.Q_irand(0, 1) != 0
                        {
                            // do pain on enemy
                            (*((*radiusEnt).client as *mut gclient_t))
                                .ps
                                .forceHandExtend = crate::prelude::HANDEXTEND_KNOCKDOWN as c_int;
                            (*((*radiusEnt).client as *mut gclient_t)).ps.forceDodgeAnim = 0;
                            (*((*radiusEnt).client as *mut gclient_t))
                                .ps
                                .forceHandExtendTime = (*ctx.world).level.time + 1100;
                            (*((*radiusEnt).client as *mut gclient_t)).ps.quickerGetup = qfalse;
                        }
                    }
                } else if (*radiusEnt).health <= 0 && !(*radiusEnt).client.is_null() {
                    // killed them, chance of dismembering
                    if (*ctx.world).bg_state.rng.Q_irand(0, 1) == 0 {
                        // bite something off
                        let hitLoc = (*ctx.world).bg_state.rng.Q_irand(
                            crate::prelude::G2_MODELPART_HEAD as c_int,
                            crate::prelude::G2_MODELPART_RLEG as c_int,
                        );
                        if hitLoc == crate::prelude::G2_MODELPART_HEAD as c_int {
                            crate::npc_c::NPC_SetAnim(
                                ctx,
                                radiusEnt,
                                SETANIM_BOTH,
                                crate::prelude::BOTH_DEATH17 as c_int,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        } else if hitLoc == crate::prelude::G2_MODELPART_WAIST as c_int {
                            crate::npc_c::NPC_SetAnim(
                                ctx,
                                radiusEnt,
                                SETANIM_BOTH,
                                crate::prelude::BOTH_DEATHBACKWARD2 as c_int,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        }
                        crate::g_combat::G_Dismember(
                            ctx,
                            radiusEnt,
                            npc,
                            (*radiusEnt).r.currentOrigin,
                            hitLoc,
                            90.0,
                            0.0,
                            (*((*radiusEnt).client as *mut gclient_t)).ps.torsoAnim,
                            qtrue,
                        );
                    }
                } else if (*ctx.world).bg_state.rng.Q_irand(0, 3) == 0 && (*radiusEnt).health > 0 {
                    // one out of every 4 normal hits does a knockdown, too
                    let mut pushDir: [f32; 3] = [0.0; 3];
                    let mut angs: [f32; 3] = [0.0; 3];
                    crate::q_math::_VectorCopy(
                        (*((*npc).client as *mut gclient_t)).ps.viewangles,
                        &mut angs,
                    );
                    angs[crate::prelude::YAW as usize] +=
                        (*ctx.world).bg_state.rng.flrand(25.0, 50.0);
                    angs[crate::prelude::PITCH as usize] =
                        (*ctx.world).bg_state.rng.flrand(-25.0, -15.0);
                    crate::q_math::AngleVectors(angs, Some(&mut pushDir), None, None);
                    crate::g_combat::G_Knockdown(ctx, radiusEnt);
                }
                crate::g_utils::G_Sound(
                    ctx,
                    radiusEnt,
                    crate::prelude::CHAN_WEAPON,
                    crate::g_utils::G_SoundIndex(c"sound/chars/rancor/swipehit.wav".as_ptr()),
                );
            }
        }
    }
}

/// Raven `Wampa_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:267-341`
pub fn Wampa_Attack(ctx: GameContext<'_>, distance: f32, doCharge: qboolean) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        if crate::g_timer::TIMER_Exists(ctx, npc, c"attacking".as_ptr()) == 0 {
            if (*ctx.world).bg_state.rng.Q_irand(0, 2) != 0 && doCharge == 0 {
                // double slash
                crate::npc_c::NPC_SetAnim(
                    ctx,
                    npc,
                    SETANIM_BOTH,
                    crate::prelude::BOTH_ATTACK1 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                crate::g_timer::TIMER_Set(ctx, npc, c"attack_dmg".as_ptr(), 750);
            } else if doCharge != 0
                || (distance > 270.0
                    && distance < 430.0
                    && (*ctx.world).bg_state.rng.Q_irand(0, 1) == 0)
            {
                // leap
                let mut fwd: [f32; 3] = [0.0; 3];
                let mut yawAng: [f32; 3] = [0.0; 3];
                crate::q_math::VectorSet(
                    &mut yawAng,
                    0.0,
                    (*((*npc).client as *mut gclient_t)).ps.viewangles
                        [crate::prelude::YAW as usize],
                    0.0,
                );
                crate::npc_c::NPC_SetAnim(
                    ctx,
                    npc,
                    SETANIM_BOTH,
                    crate::prelude::BOTH_ATTACK2 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                crate::g_timer::TIMER_Set(ctx, npc, c"attack_dmg".as_ptr(), 500);
                crate::q_math::AngleVectors(yawAng, Some(&mut fwd), None, None);
                crate::q_math::_VectorScale(
                    fwd,
                    distance * 1.5,
                    &mut (*((*npc).client as *mut gclient_t)).ps.velocity,
                );
                (*((*npc).client as *mut gclient_t)).ps.velocity[2] = 150.0;
                (*((*npc).client as *mut gclient_t)).ps.groundEntityNum =
                    crate::prelude::ENTITYNUM_NONE;
            } else {
                // backhand
                crate::npc_c::NPC_SetAnim(
                    ctx,
                    npc,
                    SETANIM_BOTH,
                    crate::prelude::BOTH_ATTACK3 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                crate::g_timer::TIMER_Set(ctx, npc, c"attack_dmg".as_ptr(), 250);
            }

            crate::g_timer::TIMER_Set(
                ctx,
                npc,
                c"attacking".as_ptr(),
                (*((*npc).client as *mut gclient_t)).ps.legsTimer as c_int
                    + ((*ctx.world).bg_state.rng.random() * 200.0) as c_int,
            );
            // allow us to re-evaluate our running speed/anim
            crate::g_timer::TIMER_Set(ctx, npc, c"runfar".as_ptr(), -1);
            crate::g_timer::TIMER_Set(ctx, npc, c"runclose".as_ptr(), -1);
            crate::g_timer::TIMER_Set(ctx, npc, c"walk".as_ptr(), -1);
        }

        // Need to do delayed damage since the attack animations encapsulate multiple mini-attacks

        if crate::g_timer::TIMER_Done2(ctx, npc, c"attack_dmg".as_ptr(), qtrue) != 0 {
            match (*((*npc).client as *mut gclient_t)).ps.legsAnim {
                _ if (*((*npc).client as *mut gclient_t)).ps.legsAnim
                    == crate::prelude::BOTH_ATTACK1 as c_int =>
                {
                    Wampa_Slash(
                        ctx,
                        (*((*npc).client as *mut gclient_t)).renderInfo.handRBolt,
                        qfalse,
                    );
                    // do second hit
                    crate::g_timer::TIMER_Set(ctx, npc, c"attack_dmg2".as_ptr(), 100);
                }
                _ if (*((*npc).client as *mut gclient_t)).ps.legsAnim
                    == crate::prelude::BOTH_ATTACK2 as c_int =>
                {
                    Wampa_Slash(
                        ctx,
                        (*((*npc).client as *mut gclient_t)).renderInfo.handRBolt,
                        qfalse,
                    );
                    crate::g_timer::TIMER_Set(ctx, npc, c"attack_dmg2".as_ptr(), 100);
                }
                _ if (*((*npc).client as *mut gclient_t)).ps.legsAnim
                    == crate::prelude::BOTH_ATTACK3 as c_int =>
                {
                    Wampa_Slash(
                        ctx,
                        (*((*npc).client as *mut gclient_t)).renderInfo.handLBolt,
                        qtrue,
                    );
                }
                _ => {}
            }
        } else if crate::g_timer::TIMER_Done2(ctx, npc, c"attack_dmg2".as_ptr(), qtrue) != 0 {
            match (*((*npc).client as *mut gclient_t)).ps.legsAnim {
                _ if (*((*npc).client as *mut gclient_t)).ps.legsAnim
                    == crate::prelude::BOTH_ATTACK1 as c_int =>
                {
                    Wampa_Slash(
                        ctx,
                        (*((*npc).client as *mut gclient_t)).renderInfo.handLBolt,
                        qfalse,
                    );
                }
                _ if (*((*npc).client as *mut gclient_t)).ps.legsAnim
                    == crate::prelude::BOTH_ATTACK2 as c_int =>
                {
                    Wampa_Slash(
                        ctx,
                        (*((*npc).client as *mut gclient_t)).renderInfo.handLBolt,
                        qfalse,
                    );
                }
                _ => {}
            }
        }

        // Just using this to remove the attacking flag at the right time
        crate::g_timer::TIMER_Done2(ctx, npc, c"attacking".as_ptr(), qtrue);

        if (*((*npc).client as *mut gclient_t)).ps.legsAnim == crate::prelude::BOTH_ATTACK1 as c_int
            && distance > ((*npc).r.maxs[0] as f32 + MIN_DISTANCE as f32)
        {
            // okay to keep moving
            (*ctx.world).globals.ucmd.buttons |= BUTTON_WALKING;
            Wampa_Move(ctx, 1);
        }
    }
}

/// Raven `Wampa_Combat`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:344-425`
pub fn Wampa_Combat(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;
        // Raven dereferences `NPC->enemy` unguarded here; this function is only
        // called while actively engaged, so the enemy is assumed live.
        let enemy_ent =
            &mut (*ctx.world).g_entities[(*npc).enemy.unwrap().index()] as *mut gentity_t;

        // If we cannot see our target or we have somewhere to go, then do that
        if crate::NPC_utils::NPC_ClearLOS(ctx, (*npc).r.currentOrigin, (*enemy_ent).r.currentOrigin)
            == 0
        {
            if (*ctx.world).bg_state.rng.Q_irand(0, 10) == 0 {
                if Wampa_CheckRoar(ctx, npc) != 0 {
                    return;
                }
            }
            (*npc_info).combatMove = qtrue;
            (*npc_info).goalEntity = (*npc).enemy;
            (*npc_info).goalRadius = MAX_DISTANCE; // just get us within combat range

            Wampa_Move(ctx, 0);
            return;
        } else if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
            (*npc_info).combatMove = qtrue;
            (*npc_info).goalEntity = (*npc).enemy;
            (*npc_info).goalRadius = MAX_DISTANCE; // just get us within combat range

            Wampa_Move(ctx, 1);
            return;
        } else {
            let distance =
                crate::q_math::Distance((*npc).r.currentOrigin, (*enemy_ent).r.currentOrigin);
            (*ctx.world).globals.enemyDist = distance;
            let mut advance = if distance > ((*npc).r.maxs[0] as f32 + MIN_DISTANCE as f32) {
                qtrue
            } else {
                qfalse
            };
            let mut doCharge = qfalse;

            // Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
            // FIXME: always seems to face off to the left or right?!!!!
            crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);

            if advance != 0 {
                // have to get closer
                let mut yawOnlyAngles: [f32; 3] = [0.0; 3];
                crate::q_math::VectorSet(
                    &mut yawOnlyAngles,
                    0.0,
                    (*npc).r.currentAngles[crate::prelude::YAW as usize],
                    0.0,
                );
                if (*enemy_ent).health > 0 // enemy still alive
                    && (distance - 350.0).abs() <= 80.0 // enemy anywhere from 270 to 430 away
                    && crate::NPC_senses::InFOV3((*enemy_ent).r.currentOrigin, (*npc).r.currentOrigin, yawOnlyAngles, 20, 20) != 0
                {
                    // enemy generally in front
                    if (*ctx.world).bg_state.rng.Q_irand(0, 9) == 0 {
                        // 10% chance of doing charge anim
                        // go for the charge
                        doCharge = qtrue;
                        advance = qfalse;
                    }
                }
            }

            if (advance != 0 || (*npc_info).localState == LSTATE_WAITING)
                && crate::g_timer::TIMER_Done(ctx, npc, c"attacking".as_ptr()) != 0
            {
                // waiting monsters can't attack
                if crate::g_timer::TIMER_Done2(ctx, npc, c"takingPain".as_ptr(), qtrue) != 0 {
                    (*npc_info).localState = LSTATE_CLEAR;
                } else {
                    Wampa_Move(ctx, 1);
                }
            } else {
                if (*ctx.world).bg_state.rng.Q_irand(0, 20) == 0 {
                    // FIXME: only do this if we just damaged them or vice-versa?
                    if Wampa_CheckRoar(ctx, npc) != 0 {
                        return;
                    }
                }
                if (*ctx.world).bg_state.rng.Q_irand(0, 1) == 0 {
                    // FIXME: base on skill
                    Wampa_Attack(ctx, distance, doCharge);
                }
            }
        }
    }
}

/// Raven `NPC_Wampa_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:433-499`
pub fn NPC_Wampa_Pain(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    unsafe {
        let mut hitByWampa = qfalse;
        if !attacker.is_null()
            && !(*attacker).client.is_null()
            && (*((*attacker).client as *mut gclient_t)).NPC_class == crate::prelude::CLASS_WAMPA
        {
            hitByWampa = qtrue;
        }
        if !attacker.is_null()
            && (*attacker).inuse != 0
            && ent_id_opt((*ctx.world).g_entities.as_ptr(), attacker) != (*self_).enemy
            && ((*attacker).flags & crate::prelude::FL_NOTARGET) == 0
        {
            // Resolved once; only dereferenced downstream after the `is_none()`
            // short-circuit guards it (mirrors Raven's unguarded `self->enemy->x`).
            let enemy_ptr = match (*self_).enemy {
                Some(id) => &mut (*ctx.world).g_entities[id.index()] as *mut gentity_t,
                None => core::ptr::null_mut(),
            };
            if ((*attacker).s.number == 0 && (*ctx.world).bg_state.rng.Q_irand(0, 3) == 0)
                || (*self_).enemy.is_none()
                || (*enemy_ptr).health == 0
                || (!(*self_).enemy.is_none()
                    && !(*enemy_ptr).client.is_null()
                    && (*((*enemy_ptr).client as *mut gclient_t)).NPC_class
                        == crate::prelude::CLASS_WAMPA)
                || ((*ctx.world).bg_state.rng.Q_irand(0, 4) == 0
                    && crate::q_math::DistanceSquared(
                        (*attacker).r.currentOrigin,
                        (*self_).r.currentOrigin,
                    ) < crate::q_math::DistanceSquared(
                        (*enemy_ptr).r.currentOrigin,
                        (*self_).r.currentOrigin,
                    ))
            {
                // if my enemy is dead (or attacked by player) and I'm not still holding/eating someone, turn on the attacker
                // FIXME: if can't nav to my enemy, take this guy if I can nav to him
                crate::NPC_combat::G_SetEnemy(ctx, self_, attacker);
                crate::g_timer::TIMER_Set(
                    ctx,
                    self_,
                    c"lookForNewEnemy".as_ptr(),
                    (*ctx.world).bg_state.rng.Q_irand(5000, 15000),
                );
                if hitByWampa != 0 {
                    // stay mad at this Wampa for 2-5 secs before looking for attacker enemies
                    crate::g_timer::TIMER_Set(
                        ctx,
                        self_,
                        c"wampaInfight".as_ptr(),
                        (*ctx.world).bg_state.rng.Q_irand(2000, 5000),
                    );
                }
            }
        }
        if (hitByWampa != 0 || (*ctx.world).bg_state.rng.Q_irand(0, 100) < damage) // hit by wampa, hit while holding live victim, or took a lot of damage
            && (*((*self_).client as *mut gclient_t)).ps.legsAnim != (crate::prelude::BOTH_GESTURE1) as i32
            && (*((*self_).client as *mut gclient_t)).ps.legsAnim != (crate::prelude::BOTH_GESTURE2) as i32
            && crate::g_timer::TIMER_Done(ctx, self_, c"takingPain".as_ptr()) != 0
        {
            if Wampa_CheckRoar(ctx, self_) == 0 {
                if (*((*self_).client as *mut gclient_t)).ps.legsAnim
                    != (crate::prelude::BOTH_ATTACK1) as i32
                    && (*((*self_).client as *mut gclient_t)).ps.legsAnim
                        != (crate::prelude::BOTH_ATTACK2) as i32
                    && (*((*self_).client as *mut gclient_t)).ps.legsAnim
                        != (crate::prelude::BOTH_ATTACK3) as i32
                {
                    // cant interrupt one of the big attack anims
                    if (*self_).health > 100 || hitByWampa != 0 {
                        crate::g_timer::TIMER_Remove(ctx, self_, c"attacking".as_ptr());

                        crate::q_math::_VectorCopy(
                            (*((*self_).NPC as *mut gNPC_t)).lastPathAngles,
                            &mut (*self_).s.angles,
                        );

                        if (*ctx.world).bg_state.rng.Q_irand(0, 1) == 0 {
                            crate::npc_c::NPC_SetAnim(
                                ctx,
                                self_,
                                SETANIM_BOTH,
                                crate::prelude::BOTH_PAIN2 as c_int,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        } else {
                            crate::npc_c::NPC_SetAnim(
                                ctx,
                                self_,
                                SETANIM_BOTH,
                                crate::prelude::BOTH_PAIN1 as c_int,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        }
                        crate::g_timer::TIMER_Set(
                            ctx,
                            self_,
                            c"takingPain".as_ptr(),
                            (*((*self_).client as *mut gclient_t)).ps.legsTimer
                                + (*ctx.world).bg_state.rng.Q_irand(0, 500),
                        );
                        // allow us to re-evaluate our running speed/anim
                        crate::g_timer::TIMER_Set(ctx, self_, c"runfar".as_ptr(), -1);
                        crate::g_timer::TIMER_Set(ctx, self_, c"runclose".as_ptr(), -1);
                        crate::g_timer::TIMER_Set(ctx, self_, c"walk".as_ptr(), -1);

                        if !(*self_).NPC.is_null() {
                            (*((*self_).NPC as *mut crate::npc::g_npc_t::gNPC_t)).localState =
                                LSTATE_WAITING;
                        }
                    }
                }
            }
        }
    }
}

/// Raven `NPC_BSWampa_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Wampa.c:506-654`
pub fn NPC_BSWampa_Default(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        (*((*npc).client as *mut gclient_t)).ps.eFlags2 &=
            !mp_bg::public::entity_effects::EF2_USE_ALT_ANIM;
        // NORMAL ANIMS
        // stand1 = normal stand
        // walk1 = normal, non-angry walk
        // walk2 = injured
        // run1 = far away run
        // run2 = close run
        // VICTIM ANIMS
        // grabswipe = melee1 - sweep out and grab
        // stand2 attack = attack4 - while holding victim, swipe at him
        // walk3_drag = walk5 - walk with drag
        // stand2 = hold victim
        // stand2to1 = drop victim
        if crate::g_timer::TIMER_Done(ctx, npc, c"rageTime".as_ptr()) == 0 {
            // do nothing but roar first time we see an enemy
            crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);
            return;
        }
        if !(*npc).enemy.is_none() {
            // Guaranteed `Some` inside this block by the guard above (mirrors
            // Raven's unguarded `NPC->enemy->x` once `NPC->enemy` is known set).
            let enemy_ptr =
                &mut (*ctx.world).g_entities[(*npc).enemy.unwrap().index()] as *mut gentity_t;
            if crate::g_timer::TIMER_Done(ctx, npc, c"attacking".as_ptr()) == 0 {
                // in middle of attack
                // face enemy
                crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);
                // continue attack logic
                (*ctx.world).globals.enemyDist =
                    crate::q_math::Distance((*npc).r.currentOrigin, (*enemy_ptr).r.currentOrigin);
                Wampa_Attack(ctx, (*ctx.world).globals.enemyDist, qfalse);
                return;
            } else {
                if crate::g_timer::TIMER_Done(ctx, npc, c"angrynoise".as_ptr()) != 0 {
                    let angrynoise_snd = crate::cstr_util::cstr(&format!(
                        "sound/chars/wampa/misc/anger{}.wav",
                        (*ctx.world).bg_state.rng.Q_irand(1, 2)
                    ));
                    crate::g_utils::G_Sound(
                        ctx,
                        npc,
                        crate::prelude::CHAN_VOICE,
                        crate::g_utils::G_SoundIndex(angrynoise_snd.as_ptr()),
                    );

                    crate::g_timer::TIMER_Set(
                        ctx,
                        npc,
                        c"angrynoise".as_ptr(),
                        (*ctx.world).bg_state.rng.Q_irand(5000, 10000),
                    );
                }
                // else, if he's in our hand, we eat, else if he's on the ground, we keep attacking his dead body for a while
                if !(*npc).enemy.is_none()
                    && !(*enemy_ptr).client.is_null()
                    && (*((*enemy_ptr).client as *mut gclient_t)).NPC_class
                        == crate::prelude::CLASS_WAMPA
                {
                    // got mad at another Wampa, look for a valid enemy
                    if crate::g_timer::TIMER_Done(ctx, npc, c"wampaInfight".as_ptr()) != 0 {
                        crate::NPC_utils::NPC_CheckEnemyExt(ctx, qtrue);
                    }
                } else {
                    let enemy_for_valid = match (*npc).enemy {
                        Some(id) => &mut (*ctx.world).g_entities[id.index()] as *mut gentity_t,
                        None => core::ptr::null_mut(),
                    };
                    if crate::NPC_combat::ValidEnemy(ctx, enemy_for_valid) == qfalse {
                        crate::g_timer::TIMER_Remove(ctx, npc, c"lookForNewEnemy".as_ptr()); // make them look again right now
                        if (*enemy_ptr).inuse == 0
                            || (*ctx.world).level.time - (*enemy_ptr).s.time
                                > (*ctx.world).bg_state.rng.Q_irand(10000, 15000)
                        {
                            // it's been a while since the enemy died, or enemy is completely gone, get bored with him
                            (*npc).enemy = None;
                            Wampa_Patrol(ctx);
                            crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
                            // just lost my enemy
                            if ((*npc).spawnflags & 2) != 0 {
                                // search around me if I don't have an enemy
                                crate::NPC_behavior::NPC_BSSearchStart(
                                    ctx,
                                    (*npc).waypoint,
                                    crate::prelude::BS_SEARCH,
                                );
                                (*npc_info).tempBehavior = crate::prelude::BS_DEFAULT;
                            } else if ((*npc).spawnflags & 1) != 0 {
                                // wander if I don't have an enemy
                                crate::NPC_behavior::NPC_BSSearchStart(
                                    ctx,
                                    (*npc).waypoint,
                                    crate::prelude::BS_WANDER,
                                );
                                (*npc_info).tempBehavior = crate::prelude::BS_DEFAULT;
                            }
                            return;
                        }
                    }
                    if crate::g_timer::TIMER_Done(ctx, npc, c"lookForNewEnemy".as_ptr()) != 0 {
                        let newEnemy;
                        let sav_enemy = (*npc).enemy; // FIXME: what about NPC->lastEnemy?
                        (*npc).enemy = None;
                        newEnemy = crate::NPC_combat::NPC_CheckEnemy(
                            ctx,
                            if (*npc_info).confusionTime < (*ctx.world).level.time {
                                qtrue
                            } else {
                                qfalse
                            },
                            qfalse,
                            qfalse,
                        );
                        (*npc).enemy = sav_enemy;
                        if !newEnemy.is_null()
                            && ent_id_opt((*ctx.world).g_entities.as_ptr(), newEnemy) != sav_enemy
                        {
                            // picked up a new enemy!
                            (*npc).lastEnemy = (*npc).enemy;
                            crate::NPC_combat::G_SetEnemy(ctx, npc, newEnemy);
                            // hold this one for at least 5-15 seconds
                            crate::g_timer::TIMER_Set(
                                ctx,
                                npc,
                                c"lookForNewEnemy".as_ptr(),
                                (*ctx.world).bg_state.rng.Q_irand(5000, 15000),
                            );
                        } else {
                            // look again in 2-5 secs
                            crate::g_timer::TIMER_Set(
                                ctx,
                                npc,
                                c"lookForNewEnemy".as_ptr(),
                                (*ctx.world).bg_state.rng.Q_irand(2000, 5000),
                            );
                        }
                    }
                }
                Wampa_Combat(ctx);
                return;
            }
        } else {
            if crate::g_timer::TIMER_Done(ctx, npc, c"idlenoise".as_ptr()) != 0 {
                crate::g_utils::G_Sound(
                    ctx,
                    npc,
                    crate::prelude::CHAN_AUTO,
                    crate::g_utils::G_SoundIndex(c"sound/chars/wampa/misc/anger3.wav".as_ptr()),
                );

                crate::g_timer::TIMER_Set(
                    ctx,
                    npc,
                    c"idlenoise".as_ptr(),
                    (*ctx.world).bg_state.rng.Q_irand(2000, 4000),
                );
            }
            if ((*npc).spawnflags & 2) != 0 {
                // search around me if I don't have an enemy
                if (*npc_info).homeWp == crate::prelude::WAYPOINT_NONE {
                    // no homewap, initialize the search behavior
                    crate::NPC_behavior::NPC_BSSearchStart(
                        ctx,
                        crate::prelude::WAYPOINT_NONE,
                        crate::prelude::BS_SEARCH,
                    );
                    (*npc_info).tempBehavior = crate::prelude::BS_DEFAULT;
                }
                (*ctx.world).globals.ucmd.buttons |= BUTTON_WALKING;
                crate::NPC_behavior::NPC_BSSearch(ctx); // this automatically looks for enemies
            } else if ((*npc).spawnflags & 1) != 0 {
                // wander if I don't have an enemy
                if (*npc_info).homeWp == crate::prelude::WAYPOINT_NONE {
                    // no homewap, initialize the wander behavior
                    crate::NPC_behavior::NPC_BSSearchStart(
                        ctx,
                        crate::prelude::WAYPOINT_NONE,
                        crate::prelude::BS_WANDER,
                    );
                    (*npc_info).tempBehavior = crate::prelude::BS_DEFAULT;
                }
                (*ctx.world).globals.ucmd.buttons |= BUTTON_WALKING;
                crate::NPC_behavior::NPC_BSWander(ctx);
                if ((*npc_info).scriptFlags & crate::prelude::SCF_LOOK_FOR_ENEMIES) != 0 {
                    if crate::NPC_utils::NPC_CheckEnemyExt(ctx, qtrue) == qfalse {
                        Wampa_Idle(ctx);
                    } else {
                        Wampa_CheckRoar(ctx, npc);
                        crate::g_timer::TIMER_Set(
                            ctx,
                            npc,
                            c"lookForNewEnemy".as_ptr(),
                            (*ctx.world).bg_state.rng.Q_irand(5000, 15000),
                        );
                    }
                }
            } else {
                if ((*npc_info).scriptFlags & crate::prelude::SCF_LOOK_FOR_ENEMIES) != 0 {
                    Wampa_Patrol(ctx);
                } else {
                    Wampa_Idle(ctx);
                }
            }
        }

        crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}
