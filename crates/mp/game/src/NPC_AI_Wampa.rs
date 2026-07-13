// PORT-COMPLETE: NPC_AI_Wampa.c 1/10
//! Port of `oracle/codemp/game/NPC_AI_Wampa.c` (jampgame mega-pass).
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

// EntityId seam helper: resolve `Option<EntityId>` back to the raw pointer the
// verbatim body still expects (`None` -> null), per the `NPC_AI_Stormtrooper.rs`
// precedent.
#[inline]
unsafe fn ent_resolve_opt(ctx: &mut GameContext, id: Option<EntityId>) -> *mut gentity_t {
    match id {
        Some(i) => unsafe { &mut (*ctx.world_raw()).g_entities[i.index()] as *mut gentity_t },
        None => core::ptr::null_mut(),
    }
}

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

// These define the working combat range for these suckers
// Source: `oracle/codemp/game/NPC_AI_Wampa.c:5-9`
const MIN_DISTANCE: c_int = 48;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;
const MAX_DISTANCE: c_int = 1024;
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;

// Source: `oracle/codemp/game/NPC_AI_Wampa.c:11-12`
const LSTATE_CLEAR: c_int = 0;
const LSTATE_WAITING: c_int = 1;

// `DistanceSquared` is the canonical `crate::q_math::DistanceSquared`, reached
// via the prelude glob (no per-file copy).

/// Raven `Wampa_SetBolts`.
///
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:16-36`
pub fn Wampa_SetBolts(ctx: &mut GameContext, self_: Option<EntityId>) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = unsafe { ent_resolve_opt(ctx, self_) };
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
/// in the oracle source (oracle/codemp/game/NPC_AI_Wampa.c:45-55).
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:43-58`
pub fn NPC_Wampa_Precache(ctx: &mut GameContext) {
    // Only the swipe sound is live; growl/snort loops are commented out
    G_SoundIndex(b"sound/chars/rancor/swipehit.wav\0".as_ptr() as *const c_char);
}

/// Raven `Wampa_Idle`.
///
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:66-76`
pub fn Wampa_Idle(ctx: &mut GameContext) {
    unsafe {
        let npc_info = (*ctx.world_raw()).globals.NPCInfo;
        if !npc_info.is_null() {
            (*npc_info).localState = LSTATE_CLEAR;
        }

        //If we have somewhere to go, then do that
        if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
            (*ctx.world_raw()).globals.ucmd.buttons &= !BUTTON_WALKING;
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
        }
    }
}

/// Raven `Wampa_CheckRoar`.
///
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:78-88`
pub fn Wampa_CheckRoar(ctx: &mut GameContext, self_: EntityId) -> qboolean {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    unsafe {
        let level_time = (*ctx.world_raw()).level.time as f32;
        if (*self_).wait < level_time {
            (*self_).wait =
                level_time + (*ctx.world_raw()).bg_state.rng.Q_irand(5000, 20000) as f32;
            let __h397 = ctx.entity_id_of(self_).unwrap();
            let __h398 = (*ctx.world_raw()).bg_state.rng.Q_irand(
                crate::prelude::BOTH_GESTURE1 as c_int,
                crate::prelude::BOTH_GESTURE2 as c_int,
            );
            crate::npc_c::NPC_SetAnim(
                ctx,
                __h397,
                SETANIM_BOTH,
                __h398,
                (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD),
            );
            crate::g_timer::TIMER_Set(
                ctx,
                ctx.entity_id_of(self_),
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
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:94-119`
pub fn Wampa_Patrol(ctx: &mut GameContext) {
    unsafe {
        let npc = (*ctx.world_raw()).globals.NPC;
        let npc_info = (*ctx.world_raw()).globals.NPCInfo;

        if !npc_info.is_null() {
            (*npc_info).localState = LSTATE_CLEAR;
        }

        //If we have somewhere to go, then do that
        if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
            (*ctx.world_raw()).globals.ucmd.buttons |= BUTTON_WALKING;
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
        } else {
            if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"patrolTime".as_ptr()) != 0 {
                let __h399 = ctx.entity_id_of(npc);
                let __h400 = ((*ctx.world_raw()).bg_state.rng.crandom() * 5000.0 + 5000.0) as c_int;
                crate::g_timer::TIMER_Set(ctx, __h399, c"patrolTime".as_ptr(), __h400);
            }
        }

        if crate::NPC_utils::NPC_CheckEnemyExt(ctx, qtrue) == qfalse {
            Wampa_Idle(ctx);
            return;
        }
        Wampa_CheckRoar(ctx, ctx.entity_id_of(npc).unwrap());
        let __h401 = ctx.entity_id_of(npc);
        let __h402 = (*ctx.world_raw()).bg_state.rng.Q_irand(5000, 15000);
        crate::g_timer::TIMER_Set(ctx, __h401, c"lookForNewEnemy".as_ptr(), __h402);
    }
}

/// Raven `Wampa_Move`.
///
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:126-169`
pub fn Wampa_Move(ctx: &mut GameContext, visible: qboolean) {
    unsafe {
        let npc = (*ctx.world_raw()).globals.NPC;
        let npc_info = (*ctx.world_raw()).globals.NPCInfo;

        if !npc_info.is_null() && (*npc_info).localState != LSTATE_WAITING {
            (*npc_info).goalEntity = (*npc).enemy;

            if !(*npc).enemy.is_none() {
                // pick correct movement speed and anim
                // run by default
                (*ctx.world_raw()).globals.ucmd.buttons &= !BUTTON_WALKING;
                if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"runfar".as_ptr()) == 0
                    || crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"runclose".as_ptr())
                        == 0
                {
                    // keep running with this anim & speed for a bit
                } else if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"walk".as_ptr())
                    == 0
                {
                    // keep walking for a bit
                    (*ctx.world_raw()).globals.ucmd.buttons |= BUTTON_WALKING;
                } else if visible != 0
                    && (*ctx.world_raw()).globals.enemyDist > 384.0
                    && (*npc_info).stats.runSpeed == 180
                {
                    // fast run, all fours
                    (*npc_info).stats.runSpeed = 300;
                    let __h403 = ctx.entity_id_of(npc);
                    let __h404 = (*ctx.world_raw()).bg_state.rng.Q_irand(2000, 4000);
                    crate::g_timer::TIMER_Set(ctx, __h403, c"runfar".as_ptr(), __h404);
                } else if (*ctx.world_raw()).globals.enemyDist > 256.0
                    && (*npc_info).stats.runSpeed == 300
                {
                    // slow run, upright
                    (*npc_info).stats.runSpeed = 180;
                    let __h405 = ctx.entity_id_of(npc);
                    let __h406 = (*ctx.world_raw()).bg_state.rng.Q_irand(3000, 5000);
                    crate::g_timer::TIMER_Set(ctx, __h405, c"runclose".as_ptr(), __h406);
                } else if (*ctx.world_raw()).globals.enemyDist < 128.0 {
                    // walk
                    (*npc_info).stats.runSpeed = 180;
                    (*ctx.world_raw()).globals.ucmd.buttons |= BUTTON_WALKING;
                    let __h407 = ctx.entity_id_of(npc);
                    let __h408 = (*ctx.world_raw()).bg_state.rng.Q_irand(4000, 6000);
                    crate::g_timer::TIMER_Set(ctx, __h407, c"walk".as_ptr(), __h408);
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
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:177-264`
pub fn Wampa_Slash(ctx: &mut GameContext, boltIndex: c_int, backhand: qboolean) {
    unsafe {
        let mut radiusEntNums: [c_int; 128] = [0; 128];
        let radius = 88.0f32;
        let radiusSquared = radius * radius;
        let mut boltOrg: [f32; 3] = [0.0; 3];
        // damage is rolled once, before the loop, and applied to every entity hit.
        let damage = if backhand != 0 {
            (*ctx.world_raw()).bg_state.rng.Q_irand(10, 15)
        } else {
            (*ctx.world_raw()).bg_state.rng.Q_irand(20, 30)
        };

        let numEnts = crate::NPC_utils::NPC_GetEntsNearBolt(
            ctx,
            radiusEntNums.as_mut_ptr(),
            radius,
            boltIndex,
            &mut boltOrg,
        );

        for i in 0..(numEnts as usize) {
            let radiusEnt = (*ctx.world_raw())
                .g_entities
                .get_unchecked_mut(radiusEntNums[i] as usize)
                as *mut gentity_t;
            if (*radiusEnt).inuse == 0 {
                continue;
            }

            let npc = (*ctx.world_raw()).globals.NPC;
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
                    ctx.entity_id_of(radiusEnt),
                    ctx.entity_id_of(npc),
                    ctx.entity_id_of(npc),
                    Some(&mut origin),
                    (*radiusEnt).r.currentOrigin,
                    damage,
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
                        (*ctx.world_raw()).bg_state.rng.flrand(25.0, 50.0);
                    angs[crate::prelude::PITCH as usize] =
                        (*ctx.world_raw()).bg_state.rng.flrand(-25.0, -15.0);
                    crate::q_math::AngleVectors(angs, Some(&mut pushDir), None, None);
                    if (*((*radiusEnt).client as *mut gclient_t)).NPC_class
                        != crate::prelude::CLASS_WAMPA
                        && (*((*radiusEnt).client as *mut gclient_t)).NPC_class
                            != crate::prelude::CLASS_RANCOR
                        && (*((*radiusEnt).client as *mut gclient_t)).NPC_class
                            != crate::prelude::CLASS_ATST
                    {
                        crate::g_utils::G_Throw(
                            ctx,
                            ctx.entity_id_of(radiusEnt).unwrap(),
                            pushDir,
                            65.0,
                        );
                        if crate::bg_pmove::BG_KnockDownable(
                            &mut (*((*radiusEnt).client as *mut gclient_t)).ps as *mut _,
                        ) != 0
                            && (*radiusEnt).health > 0
                            && (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) != 0
                        {
                            // do pain on enemy
                            (*((*radiusEnt).client as *mut gclient_t))
                                .ps
                                .forceHandExtend = crate::prelude::HANDEXTEND_KNOCKDOWN as c_int;
                            (*((*radiusEnt).client as *mut gclient_t)).ps.forceDodgeAnim = 0;
                            (*((*radiusEnt).client as *mut gclient_t))
                                .ps
                                .forceHandExtendTime = (*ctx.world_raw()).level.time + 1100;
                            (*((*radiusEnt).client as *mut gclient_t)).ps.quickerGetup = qfalse;
                        }
                    }
                } else if (*radiusEnt).health <= 0 && !(*radiusEnt).client.is_null() {
                    // killed them, chance of dismembering
                    if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) == 0 {
                        // bite something off
                        let hitLoc = (*ctx.world_raw()).bg_state.rng.Q_irand(
                            crate::prelude::G2_MODELPART_HEAD as c_int,
                            crate::prelude::G2_MODELPART_RLEG as c_int,
                        );
                        if hitLoc == crate::prelude::G2_MODELPART_HEAD as c_int {
                            crate::npc_c::NPC_SetAnim(
                                ctx,
                                ctx.entity_id_of(radiusEnt).unwrap(),
                                SETANIM_BOTH,
                                crate::prelude::BOTH_DEATH17 as c_int,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        } else if hitLoc == crate::prelude::G2_MODELPART_WAIST as c_int {
                            crate::npc_c::NPC_SetAnim(
                                ctx,
                                ctx.entity_id_of(radiusEnt).unwrap(),
                                SETANIM_BOTH,
                                crate::prelude::BOTH_DEATHBACKWARD2 as c_int,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        }
                        crate::g_combat::G_Dismember(
                            ctx,
                            ctx.entity_id_of(radiusEnt).unwrap(),
                            ctx.entity_id_of(npc),
                            (*radiusEnt).r.currentOrigin,
                            hitLoc,
                            90.0,
                            0.0,
                            (*((*radiusEnt).client as *mut gclient_t)).ps.torsoAnim,
                            qtrue,
                        );
                    }
                } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 3) == 0
                    && (*radiusEnt).health > 0
                {
                    // one out of every 4 normal hits does a knockdown, too
                    let mut pushDir: [f32; 3] = [0.0; 3];
                    let mut angs: [f32; 3] = [0.0; 3];
                    crate::q_math::_VectorCopy(
                        (*((*npc).client as *mut gclient_t)).ps.viewangles,
                        &mut angs,
                    );
                    angs[crate::prelude::YAW as usize] +=
                        (*ctx.world_raw()).bg_state.rng.flrand(25.0, 50.0);
                    angs[crate::prelude::PITCH as usize] =
                        (*ctx.world_raw()).bg_state.rng.flrand(-25.0, -15.0);
                    crate::q_math::AngleVectors(angs, Some(&mut pushDir), None, None);
                    crate::g_combat::G_Knockdown(ctx, ctx.entity_id_of(radiusEnt));
                }
                crate::g_utils::G_Sound(
                    ctx,
                    ctx.entity_id_of(radiusEnt),
                    crate::prelude::CHAN_WEAPON,
                    crate::g_utils::G_SoundIndex(c"sound/chars/rancor/swipehit.wav".as_ptr()),
                );
            }
        }
    }
}

/// Raven `Wampa_Attack`.
///
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:267-341`
pub fn Wampa_Attack(ctx: &mut GameContext, distance: f32, doCharge: qboolean) {
    unsafe {
        let npc = (*ctx.world_raw()).globals.NPC;
        if crate::g_timer::TIMER_Exists(ctx, ctx.entity_id_of(npc), c"attacking".as_ptr()) == 0 {
            let __h409 = ctx.entity_id_of(npc);
            let __h410 = (*((*npc).client as *mut gclient_t)).ps.legsTimer as c_int
                + ((*ctx.world_raw()).bg_state.rng.random() * 200.0) as c_int;
            if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 2) != 0 && doCharge == 0 {
                // double slash
                crate::npc_c::NPC_SetAnim(
                    ctx,
                    ctx.entity_id_of(npc).unwrap(),
                    SETANIM_BOTH,
                    crate::prelude::BOTH_ATTACK1 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                crate::g_timer::TIMER_Set(ctx, ctx.entity_id_of(npc), c"attack_dmg".as_ptr(), 750);
            } else if doCharge != 0
                || (distance > 270.0
                    && distance < 430.0
                    && (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) == 0)
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
                    ctx.entity_id_of(npc).unwrap(),
                    SETANIM_BOTH,
                    crate::prelude::BOTH_ATTACK2 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                crate::g_timer::TIMER_Set(ctx, ctx.entity_id_of(npc), c"attack_dmg".as_ptr(), 500);
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
                    ctx.entity_id_of(npc).unwrap(),
                    SETANIM_BOTH,
                    crate::prelude::BOTH_ATTACK3 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                crate::g_timer::TIMER_Set(ctx, ctx.entity_id_of(npc), c"attack_dmg".as_ptr(), 250);
            }

            crate::g_timer::TIMER_Set(ctx, __h409, c"attacking".as_ptr(), __h410);
            // allow us to re-evaluate our running speed/anim
            crate::g_timer::TIMER_Set(ctx, ctx.entity_id_of(npc), c"runfar".as_ptr(), -1);
            crate::g_timer::TIMER_Set(ctx, ctx.entity_id_of(npc), c"runclose".as_ptr(), -1);
            crate::g_timer::TIMER_Set(ctx, ctx.entity_id_of(npc), c"walk".as_ptr(), -1);
        }

        // Need to do delayed damage since the attack animations encapsulate multiple mini-attacks

        if crate::g_timer::TIMER_Done2(ctx, ctx.entity_id_of(npc), c"attack_dmg".as_ptr(), qtrue)
            != 0
        {
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
                    crate::g_timer::TIMER_Set(
                        ctx,
                        ctx.entity_id_of(npc),
                        c"attack_dmg2".as_ptr(),
                        100,
                    );
                }
                _ if (*((*npc).client as *mut gclient_t)).ps.legsAnim
                    == crate::prelude::BOTH_ATTACK2 as c_int =>
                {
                    Wampa_Slash(
                        ctx,
                        (*((*npc).client as *mut gclient_t)).renderInfo.handRBolt,
                        qfalse,
                    );
                    crate::g_timer::TIMER_Set(
                        ctx,
                        ctx.entity_id_of(npc),
                        c"attack_dmg2".as_ptr(),
                        100,
                    );
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
        } else if crate::g_timer::TIMER_Done2(
            ctx,
            ctx.entity_id_of(npc),
            c"attack_dmg2".as_ptr(),
            qtrue,
        ) != 0
        {
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
        crate::g_timer::TIMER_Done2(ctx, ctx.entity_id_of(npc), c"attacking".as_ptr(), qtrue);

        if (*((*npc).client as *mut gclient_t)).ps.legsAnim == crate::prelude::BOTH_ATTACK1 as c_int
            && distance > ((*npc).r.maxs[0] as f32 + MIN_DISTANCE as f32)
        {
            // okay to keep moving
            (*ctx.world_raw()).globals.ucmd.buttons |= BUTTON_WALKING;
            Wampa_Move(ctx, 1);
        }
    }
}

/// Raven `Wampa_Combat`.
///
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:344-425`
pub fn Wampa_Combat(ctx: &mut GameContext) {
    unsafe {
        let npc = (*ctx.world_raw()).globals.NPC;
        let npc_info = (*ctx.world_raw()).globals.NPCInfo;
        // Raven dereferences `NPC->enemy` unguarded here; this function is only
        // called while actively engaged, so the enemy is assumed live.
        let enemy_ent =
            &mut (*ctx.world_raw()).g_entities[(*npc).enemy.unwrap().index()] as *mut gentity_t;

        // If we cannot see our target or we have somewhere to go, then do that
        if crate::NPC_utils::NPC_ClearLOS(ctx, (*npc).r.currentOrigin, (*enemy_ent).r.currentOrigin)
            == 0
        {
            if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 10) == 0 {
                if Wampa_CheckRoar(ctx, ctx.entity_id_of(npc).unwrap()) != 0 {
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
            (*ctx.world_raw()).globals.enemyDist = distance;
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
                    if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 9) == 0 {
                        // 10% chance of doing charge anim
                        // go for the charge
                        doCharge = qtrue;
                        advance = qfalse;
                    }
                }
            }

            if (advance != 0 || (*npc_info).localState == LSTATE_WAITING)
                && crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"attacking".as_ptr())
                    != 0
            {
                // waiting monsters can't attack
                if crate::g_timer::TIMER_Done2(
                    ctx,
                    ctx.entity_id_of(npc),
                    c"takingPain".as_ptr(),
                    qtrue,
                ) != 0
                {
                    (*npc_info).localState = LSTATE_CLEAR;
                } else {
                    Wampa_Move(ctx, 1);
                }
            } else {
                if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 20) == 0 {
                    // FIXME: only do this if we just damaged them or vice-versa?
                    if Wampa_CheckRoar(ctx, ctx.entity_id_of(npc).unwrap()) != 0 {
                        return;
                    }
                }
                if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) == 0 {
                    // FIXME: base on skill
                    Wampa_Attack(ctx, distance, doCharge);
                }
            }
        }
    }
}

/// Raven `NPC_Wampa_Pain`.
///
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:433-499`
pub fn NPC_Wampa_Pain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    // STAGE-1: EntityId params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let attacker: *mut gentity_t = unsafe { ent_resolve_opt(ctx, attacker) };
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
            && ent_id_opt((*ctx.world_raw()).g_entities.as_ptr(), attacker) != (*self_).enemy
            && ((*attacker).flags & crate::prelude::FL_NOTARGET) == 0
        {
            // Resolved once; only dereferenced downstream after the `is_none()`
            // short-circuit guards it (mirrors Raven's unguarded `self->enemy->x`).
            let enemy_ptr = match (*self_).enemy {
                Some(id) => &mut (*ctx.world_raw()).g_entities[id.index()] as *mut gentity_t,
                None => core::ptr::null_mut(),
            };
            if ((*attacker).s.number == 0 && (*ctx.world_raw()).bg_state.rng.Q_irand(0, 3) == 0)
                || (*self_).enemy.is_none()
                || (*enemy_ptr).health == 0
                || (!(*self_).enemy.is_none()
                    && !(*enemy_ptr).client.is_null()
                    && (*((*enemy_ptr).client as *mut gclient_t)).NPC_class
                        == crate::prelude::CLASS_WAMPA)
                || ((*ctx.world_raw()).bg_state.rng.Q_irand(0, 4) == 0
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
                crate::NPC_combat::G_SetEnemy(
                    ctx,
                    ctx.entity_id_of(self_).unwrap(),
                    ctx.entity_id_of(attacker),
                );
                let __h411 = ctx.entity_id_of(self_);
                let __h412 = (*ctx.world_raw()).bg_state.rng.Q_irand(5000, 15000);
                crate::g_timer::TIMER_Set(ctx, __h411, c"lookForNewEnemy".as_ptr(), __h412);
                if hitByWampa != 0 {
                    let __h413 = ctx.entity_id_of(self_);
                    let __h414 = (*ctx.world_raw()).bg_state.rng.Q_irand(2000, 5000);
                    // stay mad at this Wampa for 2-5 secs before looking for attacker enemies
                    crate::g_timer::TIMER_Set(ctx, __h413, c"wampaInfight".as_ptr(), __h414);
                }
            }
        }
        if (hitByWampa != 0 || (*ctx.world_raw()).bg_state.rng.Q_irand(0, 100) < damage) // hit by wampa, hit while holding live victim, or took a lot of damage
            && (*((*self_).client as *mut gclient_t)).ps.legsAnim != (crate::prelude::BOTH_GESTURE1) as i32
            && (*((*self_).client as *mut gclient_t)).ps.legsAnim != (crate::prelude::BOTH_GESTURE2) as i32
            && crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(self_), c"takingPain".as_ptr()) != 0
        {
            if Wampa_CheckRoar(ctx, ctx.entity_id_of(self_).unwrap()) == 0 {
                if (*((*self_).client as *mut gclient_t)).ps.legsAnim
                    != (crate::prelude::BOTH_ATTACK1) as i32
                    && (*((*self_).client as *mut gclient_t)).ps.legsAnim
                        != (crate::prelude::BOTH_ATTACK2) as i32
                    && (*((*self_).client as *mut gclient_t)).ps.legsAnim
                        != (crate::prelude::BOTH_ATTACK3) as i32
                {
                    // cant interrupt one of the big attack anims
                    if (*self_).health > 100 || hitByWampa != 0 {
                        crate::g_timer::TIMER_Remove(
                            ctx,
                            ctx.entity_id_of(self_),
                            c"attacking".as_ptr(),
                        );

                        crate::q_math::_VectorCopy(
                            (*((*self_).NPC as *mut gNPC_t)).lastPathAngles,
                            &mut (*self_).s.angles,
                        );

                        let __h415 = ctx.entity_id_of(self_);
                        let __h416 = (*((*self_).client as *mut gclient_t)).ps.legsTimer
                            + (*ctx.world_raw()).bg_state.rng.Q_irand(0, 500);
                        if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) == 0 {
                            crate::npc_c::NPC_SetAnim(
                                ctx,
                                ctx.entity_id_of(self_).unwrap(),
                                SETANIM_BOTH,
                                crate::prelude::BOTH_PAIN2 as c_int,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        } else {
                            crate::npc_c::NPC_SetAnim(
                                ctx,
                                ctx.entity_id_of(self_).unwrap(),
                                SETANIM_BOTH,
                                crate::prelude::BOTH_PAIN1 as c_int,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        }
                        crate::g_timer::TIMER_Set(ctx, __h415, c"takingPain".as_ptr(), __h416);
                        // allow us to re-evaluate our running speed/anim
                        crate::g_timer::TIMER_Set(
                            ctx,
                            ctx.entity_id_of(self_),
                            c"runfar".as_ptr(),
                            -1,
                        );
                        crate::g_timer::TIMER_Set(
                            ctx,
                            ctx.entity_id_of(self_),
                            c"runclose".as_ptr(),
                            -1,
                        );
                        crate::g_timer::TIMER_Set(
                            ctx,
                            ctx.entity_id_of(self_),
                            c"walk".as_ptr(),
                            -1,
                        );

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
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:506-654`
pub fn NPC_BSWampa_Default(ctx: &mut GameContext) {
    unsafe {
        let npc = (*ctx.world_raw()).globals.NPC;
        let npc_info = (*ctx.world_raw()).globals.NPCInfo;

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
        if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"rageTime".as_ptr()) == 0 {
            // do nothing but roar first time we see an enemy
            crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);
            return;
        }
        if !(*npc).enemy.is_none() {
            // Guaranteed `Some` inside this block by the guard above (mirrors
            // Raven's unguarded `NPC->enemy->x` once `NPC->enemy` is known set).
            let enemy_ptr =
                &mut (*ctx.world_raw()).g_entities[(*npc).enemy.unwrap().index()] as *mut gentity_t;
            if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"attacking".as_ptr()) == 0 {
                // in middle of attack
                // face enemy
                crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);
                // continue attack logic
                (*ctx.world_raw()).globals.enemyDist =
                    crate::q_math::Distance((*npc).r.currentOrigin, (*enemy_ptr).r.currentOrigin);
                let __h417 = (*ctx.world_raw()).globals.enemyDist;
                Wampa_Attack(ctx, __h417, qfalse);
                return;
            } else {
                if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"angrynoise".as_ptr())
                    != 0
                {
                    let angrynoise_snd = crate::cstr_util::cstr(&format!(
                        "sound/chars/wampa/misc/anger{}.wav",
                        (*ctx.world_raw()).bg_state.rng.Q_irand(1, 2)
                    ));
                    crate::g_utils::G_Sound(
                        ctx,
                        ctx.entity_id_of(npc),
                        crate::prelude::CHAN_VOICE,
                        crate::g_utils::G_SoundIndex(angrynoise_snd.as_ptr()),
                    );

                    let __h418 = ctx.entity_id_of(npc);
                    let __h419 = (*ctx.world_raw()).bg_state.rng.Q_irand(5000, 10000);
                    crate::g_timer::TIMER_Set(ctx, __h418, c"angrynoise".as_ptr(), __h419);
                }
                // else, if he's in our hand, we eat, else if he's on the ground, we keep attacking his dead body for a while
                if !(*npc).enemy.is_none()
                    && !(*enemy_ptr).client.is_null()
                    && (*((*enemy_ptr).client as *mut gclient_t)).NPC_class
                        == crate::prelude::CLASS_WAMPA
                {
                    // got mad at another Wampa, look for a valid enemy
                    if crate::g_timer::TIMER_Done(
                        ctx,
                        ctx.entity_id_of(npc),
                        c"wampaInfight".as_ptr(),
                    ) != 0
                    {
                        crate::NPC_utils::NPC_CheckEnemyExt(ctx, qtrue);
                    }
                } else {
                    let enemy_for_valid = match (*npc).enemy {
                        Some(id) => {
                            &mut (*ctx.world_raw()).g_entities[id.index()] as *mut gentity_t
                        }
                        None => core::ptr::null_mut(),
                    };
                    if crate::NPC_combat::ValidEnemy(ctx, ctx.entity_id_of(enemy_for_valid))
                        == qfalse
                    {
                        crate::g_timer::TIMER_Remove(
                            ctx,
                            ctx.entity_id_of(npc),
                            c"lookForNewEnemy".as_ptr(),
                        ); // make them look again right now
                        if (*enemy_ptr).inuse == 0
                            || (*ctx.world_raw()).level.time - (*enemy_ptr).s.time
                                > (*ctx.world_raw()).bg_state.rng.Q_irand(10000, 15000)
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
                    if crate::g_timer::TIMER_Done(
                        ctx,
                        ctx.entity_id_of(npc),
                        c"lookForNewEnemy".as_ptr(),
                    ) != 0
                    {
                        let newEnemy;
                        let sav_enemy = (*npc).enemy; // FIXME: what about NPC->lastEnemy?
                        (*npc).enemy = None;
                        let __h420 = if (*npc_info).confusionTime < (*ctx.world_raw()).level.time {
                            qtrue
                        } else {
                            qfalse
                        };
                        newEnemy = crate::NPC_combat::NPC_CheckEnemy(ctx, __h420, qfalse, qfalse);
                        (*npc).enemy = sav_enemy;
                        if !newEnemy.is_null()
                            && ent_id_opt((*ctx.world_raw()).g_entities.as_ptr(), newEnemy)
                                != sav_enemy
                        {
                            // picked up a new enemy!
                            (*npc).lastEnemy = (*npc).enemy;
                            crate::NPC_combat::G_SetEnemy(
                                ctx,
                                ctx.entity_id_of(npc).unwrap(),
                                ctx.entity_id_of(newEnemy),
                            );
                            let __h421 = ctx.entity_id_of(npc);
                            let __h422 = (*ctx.world_raw()).bg_state.rng.Q_irand(5000, 15000);
                            // hold this one for at least 5-15 seconds
                            crate::g_timer::TIMER_Set(
                                ctx,
                                __h421,
                                c"lookForNewEnemy".as_ptr(),
                                __h422,
                            );
                        } else {
                            let __h423 = ctx.entity_id_of(npc);
                            let __h424 = (*ctx.world_raw()).bg_state.rng.Q_irand(2000, 5000);
                            // look again in 2-5 secs
                            crate::g_timer::TIMER_Set(
                                ctx,
                                __h423,
                                c"lookForNewEnemy".as_ptr(),
                                __h424,
                            );
                        }
                    }
                }
                Wampa_Combat(ctx);
                return;
            }
        } else {
            if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"idlenoise".as_ptr()) != 0 {
                crate::g_utils::G_Sound(
                    ctx,
                    ctx.entity_id_of(npc),
                    crate::prelude::CHAN_AUTO,
                    crate::g_utils::G_SoundIndex(c"sound/chars/wampa/misc/anger3.wav".as_ptr()),
                );

                let __h425 = ctx.entity_id_of(npc);
                let __h426 = (*ctx.world_raw()).bg_state.rng.Q_irand(2000, 4000);
                crate::g_timer::TIMER_Set(ctx, __h425, c"idlenoise".as_ptr(), __h426);
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
                (*ctx.world_raw()).globals.ucmd.buttons |= BUTTON_WALKING;
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
                (*ctx.world_raw()).globals.ucmd.buttons |= BUTTON_WALKING;
                crate::NPC_behavior::NPC_BSWander(ctx);
                if ((*npc_info).scriptFlags & crate::prelude::SCF_LOOK_FOR_ENEMIES) != 0 {
                    if crate::NPC_utils::NPC_CheckEnemyExt(ctx, qtrue) == qfalse {
                        Wampa_Idle(ctx);
                    } else {
                        Wampa_CheckRoar(ctx, ctx.entity_id_of(npc).unwrap());
                        let __h427 = ctx.entity_id_of(npc);
                        let __h428 = (*ctx.world_raw()).bg_state.rng.Q_irand(5000, 15000);
                        crate::g_timer::TIMER_Set(ctx, __h427, c"lookForNewEnemy".as_ptr(), __h428);
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
