// PORT-COMPLETE: NPC_AI_Wampa.c
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
//! Safe-state migration **Campaign 2c** (deref regime): every entity reach is a
//! checked `ctx.world.entity(id)`/`entity_mut(id)` borrow taken at the point of
//! use; the ambient `NPC` actor is a raw pointer value only long enough to
//! recover its `EntityId`. The `gNPC_t` (`NPCInfo`/`self->NPC`) struct has no
//! accessor, so its derefs stay raw in tight `unsafe` blocks (`// FLAG:` sites),
//! as do the BG_Alloc'd pool-client (`gclient_t`) derefs, read via the safe
//! entity borrow (trap 2b/2c). This file is referee-blind — parity rests on the
//! compile + golden suite.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_utils::G_SoundIndex;
use crate::prelude::*;
use crate::trap;
use mp_qshared::common::mp::qcommon::usercmd_button::BUTTON_WALKING;

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
    let Some(self_id) = self_ else {
        return;
    };
    // FLAG: Wampa pool client, deref raw via the safe entity borrow (trap 2b).
    let client = ctx.world.entity(self_id).client;
    if client.is_null() {
        return;
    }
    let ghoul2 = ctx.world.entity(self_id).ghoul2;
    unsafe {
        let ri = &mut (*client).renderInfo;
        ri.headBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*head_eyes");
        ri.torsoBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "lower_spine");
        ri.crotchBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "rear_bone");
        ri.handLBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*l_hand");
        ri.handRBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*r_hand");
        ri.footLBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*l_leg_foot");
        ri.footRBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*r_leg_foot");
    }
}

/// Raven `NPC_Wampa_Precache`.
///
/// Precaches the swipe-hit sound. All growl/snort variants are commented out
/// in the oracle source (oracle/codemp/game/NPC_AI_Wampa.c:45-55).
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:43-58`
pub fn NPC_Wampa_Precache(ctx: &mut GameContext) {
    // Only the swipe sound is live; growl/snort loops are commented out
    G_SoundIndex(ctx, "sound/chars/rancor/swipehit.wav");
}

/// Raven `Wampa_Idle`.
///
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:66-76`
pub fn Wampa_Idle(ctx: &mut GameContext) {
    // FLAG: gNPC_t (NPCInfo) has no accessor; deref stays raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;
    if !npc_info.is_null() {
        unsafe {
            (*npc_info).localState = LSTATE_CLEAR;
        }
    }

    //If we have somewhere to go, then do that
    if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
        ctx.world.globals.ucmd.buttons &= !BUTTON_WALKING;
        crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
    }
}

/// Raven `Wampa_CheckRoar`.
///
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:78-88`
pub fn Wampa_CheckRoar(ctx: &mut GameContext, self_: EntityId) -> qboolean {
    let level_time = ctx.world.level.time as f32;
    if ctx.world.entity(self_).wait < level_time {
        let roar = ctx.world.bg_state.rng.Q_irand(5000, 20000) as f32;
        ctx.world.entity_mut(self_).wait = level_time + roar;
        let anim = ctx.world.bg_state.rng.Q_irand(
            crate::prelude::BOTH_GESTURE1 as c_int,
            crate::prelude::BOTH_GESTURE2 as c_int,
        );
        crate::npc_c::NPC_SetAnim(
            ctx,
            self_,
            SETANIM_BOTH,
            anim,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        // FLAG: Wampa pool client, deref raw via the safe entity borrow (trap 2b).
        let client = ctx.world.entity(self_).client;
        let legs_timer = unsafe { (*client).ps.legsTimer };
        crate::g_timer::TIMER_Set(ctx, Some(self_), c"rageTime".as_ptr(), legs_timer);
        return qtrue;
    }
    qfalse
}

/// Raven `Wampa_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:94-119`
pub fn Wampa_Patrol(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; deref stays raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;

    if !npc_info.is_null() {
        unsafe {
            (*npc_info).localState = LSTATE_CLEAR;
        }
    }

    //If we have somewhere to go, then do that
    if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
        ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
        crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
    } else {
        if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"patrolTime".as_ptr()) != 0 {
            let patrol_time = (ctx.world.bg_state.rng.crandom() * 5000.0 + 5000.0) as c_int;
            crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"patrolTime".as_ptr(), patrol_time);
        }
    }

    if crate::NPC_utils::NPC_CheckEnemyExt(ctx, qtrue) == qfalse {
        Wampa_Idle(ctx);
        return;
    }
    Wampa_CheckRoar(ctx, npc_id);
    let look_for_new_enemy = ctx.world.bg_state.rng.Q_irand(5000, 15000);
    crate::g_timer::TIMER_Set(
        ctx,
        Some(npc_id),
        c"lookForNewEnemy".as_ptr(),
        look_for_new_enemy,
    );
}

/// Raven `Wampa_Move`.
///
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:126-169`
pub fn Wampa_Move(ctx: &mut GameContext, visible: qboolean) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;

    unsafe {
        if !npc_info.is_null() && (*npc_info).localState != LSTATE_WAITING {
            (*npc_info).goalEntity = ctx.world.entity(npc_id).enemy;

            if ctx.world.entity(npc_id).enemy.is_some() {
                // pick correct movement speed and anim
                // run by default
                ctx.world.globals.ucmd.buttons &= !BUTTON_WALKING;
                if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"runfar".as_ptr()) == 0
                    || crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"runclose".as_ptr()) == 0
                {
                    // keep running with this anim & speed for a bit
                } else if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"walk".as_ptr()) == 0 {
                    // keep walking for a bit
                    ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
                } else if visible != 0
                    && ctx.world.globals.enemyDist > 384.0
                    && (*npc_info).stats.runSpeed == 180
                {
                    // fast run, all fours
                    (*npc_info).stats.runSpeed = 300;
                    let runfar = ctx.world.bg_state.rng.Q_irand(2000, 4000);
                    crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"runfar".as_ptr(), runfar);
                } else if ctx.world.globals.enemyDist > 256.0 && (*npc_info).stats.runSpeed == 300 {
                    // slow run, upright
                    (*npc_info).stats.runSpeed = 180;
                    let runclose = ctx.world.bg_state.rng.Q_irand(3000, 5000);
                    crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"runclose".as_ptr(), runclose);
                } else if ctx.world.globals.enemyDist < 128.0 {
                    // walk
                    (*npc_info).stats.runSpeed = 180;
                    ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
                    let walk = ctx.world.bg_state.rng.Q_irand(4000, 6000);
                    crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"walk".as_ptr(), walk);
                }
            }

            if (*npc_info).stats.runSpeed == 300 {
                // need to use the alternate run - hunched over on all fours
                // FLAG: Wampa pool client, deref raw via the safe entity borrow (trap 2b).
                let client = ctx.world.entity(npc_id).client;
                (*client).ps.eFlags2 |= mp_bg::public::entity_effects::EF2_USE_ALT_ANIM;
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
    let mut radiusEntNums: [c_int; 128] = [0; 128];
    let radius = 88.0f32;
    let radiusSquared = radius * radius;
    let mut boltOrg: [f32; 3] = [0.0; 3];
    // damage is rolled once, before the loop, and applied to every entity hit.
    let damage = if backhand != 0 {
        ctx.world.bg_state.rng.Q_irand(10, 15)
    } else {
        ctx.world.bg_state.rng.Q_irand(20, 30)
    };

    let numEnts = crate::NPC_utils::NPC_GetEntsNearBolt(
        ctx,
        radiusEntNums.as_mut_ptr(),
        radius,
        boltIndex,
        &mut boltOrg,
    );

    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    for i in 0..(numEnts as usize) {
        let radius_id = EntityId(radiusEntNums[i] as u32);
        if ctx.world.entity(radius_id).inuse == 0 {
            continue;
        }

        if radius_id == npc_id {
            // Skip the wampa ent
            continue;
        }

        // FLAG: arbitrary hit ent — client may be real or NPC pool; read the
        // pointer via the safe entity borrow, deref raw (trap 2c).
        let radius_client = ctx.world.entity(radius_id).client;
        if radius_client.is_null() {
            // must be a client
            continue;
        }

        let radius_origin = ctx.world.entity(radius_id).r.currentOrigin;
        if DistanceSquared(radius_origin, boltOrg) <= radiusSquared {
            // smack
            // Raven passes the global `vec3_origin` as `dir`; G_Damage
            // normalizes `dir` in place (a no-op on the zero vector), so a
            // fresh local copy is behaviorally identical.
            let mut origin = vec3_origin;
            crate::g_combat::G_Damage(
                ctx,
                Some(radius_id),
                Some(npc_id),
                Some(npc_id),
                Some(&mut origin),
                radius_origin,
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
                // FLAG: Wampa pool client, deref raw via the safe entity borrow (trap 2b).
                let npc_client = ctx.world.entity(npc_id).client;
                let viewangles = unsafe { (*npc_client).ps.viewangles };
                crate::q_math::_VectorCopy(viewangles, &mut angs);
                angs[crate::prelude::YAW as usize] += ctx.world.bg_state.rng.flrand(25.0, 50.0);
                angs[crate::prelude::PITCH as usize] = ctx.world.bg_state.rng.flrand(-25.0, -15.0);
                crate::q_math::AngleVectors(angs, Some(&mut pushDir), None, None);
                // FLAG: arbitrary hit ent client, deref raw via the safe borrow (trap 2c).
                let npc_class = unsafe { (*radius_client).NPC_class };
                if npc_class != crate::prelude::CLASS_WAMPA
                    && npc_class != crate::prelude::CLASS_RANCOR
                    && npc_class != crate::prelude::CLASS_ATST
                {
                    crate::g_utils::G_Throw(ctx, radius_id, pushDir, 65.0);
                    let knockdownable = unsafe {
                        mp_bg::bg_pmove::BG_KnockDownable(&mut (*radius_client).ps as *mut _)
                    };
                    if knockdownable != 0
                        && ctx.world.entity(radius_id).health > 0
                        && ctx.world.bg_state.rng.Q_irand(0, 1) != 0
                    {
                        // do pain on enemy
                        let level_time = ctx.world.level.time;
                        unsafe {
                            (*radius_client).ps.forceHandExtend =
                                crate::prelude::HANDEXTEND_KNOCKDOWN as c_int;
                            (*radius_client).ps.forceDodgeAnim = 0;
                            (*radius_client).ps.forceHandExtendTime = level_time + 1100;
                            (*radius_client).ps.quickerGetup = qfalse;
                        }
                    }
                }
            } else if ctx.world.entity(radius_id).health <= 0 && !radius_client.is_null() {
                // killed them, chance of dismembering
                if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
                    // bite something off
                    let hitLoc = ctx.world.bg_state.rng.Q_irand(
                        crate::prelude::G2_MODELPART_HEAD as c_int,
                        crate::prelude::G2_MODELPART_RLEG as c_int,
                    );
                    if hitLoc == crate::prelude::G2_MODELPART_HEAD as c_int {
                        crate::npc_c::NPC_SetAnim(
                            ctx,
                            radius_id,
                            SETANIM_BOTH,
                            crate::prelude::BOTH_DEATH17 as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                    } else if hitLoc == crate::prelude::G2_MODELPART_WAIST as c_int {
                        crate::npc_c::NPC_SetAnim(
                            ctx,
                            radius_id,
                            SETANIM_BOTH,
                            crate::prelude::BOTH_DEATHBACKWARD2 as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                    }
                    let dismember_origin = ctx.world.entity(radius_id).r.currentOrigin;
                    // FLAG: arbitrary hit ent client, deref raw via the safe borrow (trap 2c).
                    let torso_anim = unsafe { (*radius_client).ps.torsoAnim };
                    crate::g_combat::G_Dismember(
                        ctx,
                        radius_id,
                        Some(npc_id),
                        dismember_origin,
                        hitLoc,
                        90.0,
                        0.0,
                        torso_anim,
                        qtrue,
                    );
                }
            } else if ctx.world.bg_state.rng.Q_irand(0, 3) == 0
                && ctx.world.entity(radius_id).health > 0
            {
                // one out of every 4 normal hits does a knockdown, too
                let mut pushDir: [f32; 3] = [0.0; 3];
                let mut angs: [f32; 3] = [0.0; 3];
                // FLAG: Wampa pool client, deref raw via the safe entity borrow (trap 2b).
                let npc_client = ctx.world.entity(npc_id).client;
                let viewangles = unsafe { (*npc_client).ps.viewangles };
                crate::q_math::_VectorCopy(viewangles, &mut angs);
                angs[crate::prelude::YAW as usize] += ctx.world.bg_state.rng.flrand(25.0, 50.0);
                angs[crate::prelude::PITCH as usize] = ctx.world.bg_state.rng.flrand(-25.0, -15.0);
                crate::q_math::AngleVectors(angs, Some(&mut pushDir), None, None);
                crate::g_combat::G_Knockdown(ctx, Some(radius_id));
            }
            let sound = G_SoundIndex(ctx, "sound/chars/rancor/swipehit.wav");
            crate::g_utils::G_Sound(
                ctx,
                Some(radius_id),
                crate::prelude::CHAN_WEAPON,
                sound,
            );
        }
    }
}

/// Raven `Wampa_Attack`.
///
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:267-341`
pub fn Wampa_Attack(ctx: &mut GameContext, distance: f32, doCharge: qboolean) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: Wampa pool client, deref raw via the safe entity borrow (trap 2b).
    let client = ctx.world.entity(npc_id).client;
    unsafe {
        if crate::g_timer::TIMER_Exists(ctx, Some(npc_id), c"attacking".as_ptr()) == 0 {
            let attacking = (*client).ps.legsTimer as c_int
                + (ctx.world.bg_state.rng.random() * 200.0) as c_int;
            if ctx.world.bg_state.rng.Q_irand(0, 2) != 0 && doCharge == 0 {
                // double slash
                crate::npc_c::NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_BOTH,
                    crate::prelude::BOTH_ATTACK1 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attack_dmg".as_ptr(), 750);
            } else if doCharge != 0
                || (distance > 270.0
                    && distance < 430.0
                    && ctx.world.bg_state.rng.Q_irand(0, 1) == 0)
            {
                // leap
                let mut fwd: [f32; 3] = [0.0; 3];
                let mut yawAng: [f32; 3] = [0.0; 3];
                let viewangles_yaw = (*client).ps.viewangles[crate::prelude::YAW as usize];
                crate::q_math::VectorSet(&mut yawAng, 0.0, viewangles_yaw, 0.0);
                crate::npc_c::NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_BOTH,
                    crate::prelude::BOTH_ATTACK2 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attack_dmg".as_ptr(), 500);
                crate::q_math::AngleVectors(yawAng, Some(&mut fwd), None, None);
                crate::q_math::_VectorScale(fwd, distance * 1.5, &mut (*client).ps.velocity);
                (*client).ps.velocity[2] = 150.0;
                (*client).ps.groundEntityNum = crate::prelude::ENTITYNUM_NONE;
            } else {
                // backhand
                crate::npc_c::NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_BOTH,
                    crate::prelude::BOTH_ATTACK3 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attack_dmg".as_ptr(), 250);
            }

            crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attacking".as_ptr(), attacking);
            // allow us to re-evaluate our running speed/anim
            crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"runfar".as_ptr(), -1);
            crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"runclose".as_ptr(), -1);
            crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"walk".as_ptr(), -1);
        }

        // Need to do delayed damage since the attack animations encapsulate multiple mini-attacks

        if crate::g_timer::TIMER_Done2(ctx, Some(npc_id), c"attack_dmg".as_ptr(), qtrue) != 0 {
            match (*client).ps.legsAnim {
                _ if (*client).ps.legsAnim == crate::prelude::BOTH_ATTACK1 as c_int => {
                    let bolt = (*client).renderInfo.handRBolt;
                    Wampa_Slash(ctx, bolt, qfalse);
                    // do second hit
                    crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attack_dmg2".as_ptr(), 100);
                }
                _ if (*client).ps.legsAnim == crate::prelude::BOTH_ATTACK2 as c_int => {
                    let bolt = (*client).renderInfo.handRBolt;
                    Wampa_Slash(ctx, bolt, qfalse);
                    crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attack_dmg2".as_ptr(), 100);
                }
                _ if (*client).ps.legsAnim == crate::prelude::BOTH_ATTACK3 as c_int => {
                    let bolt = (*client).renderInfo.handLBolt;
                    Wampa_Slash(ctx, bolt, qtrue);
                }
                _ => {}
            }
        } else if crate::g_timer::TIMER_Done2(ctx, Some(npc_id), c"attack_dmg2".as_ptr(), qtrue)
            != 0
        {
            match (*client).ps.legsAnim {
                _ if (*client).ps.legsAnim == crate::prelude::BOTH_ATTACK1 as c_int => {
                    let bolt = (*client).renderInfo.handLBolt;
                    Wampa_Slash(ctx, bolt, qfalse);
                }
                _ if (*client).ps.legsAnim == crate::prelude::BOTH_ATTACK2 as c_int => {
                    let bolt = (*client).renderInfo.handLBolt;
                    Wampa_Slash(ctx, bolt, qfalse);
                }
                _ => {}
            }
        }

        // Just using this to remove the attacking flag at the right time
        crate::g_timer::TIMER_Done2(ctx, Some(npc_id), c"attacking".as_ptr(), qtrue);

        if (*client).ps.legsAnim == crate::prelude::BOTH_ATTACK1 as c_int
            && distance > (ctx.world.entity(npc_id).r.maxs[0] as f32 + MIN_DISTANCE as f32)
        {
            // okay to keep moving
            ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
            Wampa_Move(ctx, 1);
        }
    }
}

/// Raven `Wampa_Combat`.
///
/// Source: `oracle/codemp/game/NPC_AI_Wampa.c:344-425`
pub fn Wampa_Combat(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;
    // Raven dereferences `NPC->enemy` unguarded here; this function is only
    // called while actively engaged, so the enemy is assumed live.
    let enemy_id = ctx.world.entity(npc_id).enemy.unwrap();

    unsafe {
        let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
        let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
        // If we cannot see our target or we have somewhere to go, then do that
        if crate::NPC_utils::NPC_ClearLOS(ctx, npc_origin, enemy_origin) == 0 {
            if ctx.world.bg_state.rng.Q_irand(0, 10) == 0 {
                if Wampa_CheckRoar(ctx, npc_id) != 0 {
                    return;
                }
            }
            (*npc_info).combatMove = qtrue;
            (*npc_info).goalEntity = ctx.world.entity(npc_id).enemy;
            (*npc_info).goalRadius = MAX_DISTANCE; // just get us within combat range

            Wampa_Move(ctx, 0);
            return;
        } else if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
            (*npc_info).combatMove = qtrue;
            (*npc_info).goalEntity = ctx.world.entity(npc_id).enemy;
            (*npc_info).goalRadius = MAX_DISTANCE; // just get us within combat range

            Wampa_Move(ctx, 1);
            return;
        } else {
            let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
            let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
            let distance = crate::q_math::Distance(npc_origin, enemy_origin);
            ctx.world.globals.enemyDist = distance;
            let mut advance =
                if distance > (ctx.world.entity(npc_id).r.maxs[0] as f32 + MIN_DISTANCE as f32) {
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
                let npc_yaw =
                    ctx.world.entity(npc_id).r.currentAngles[crate::prelude::YAW as usize];
                crate::q_math::VectorSet(&mut yawOnlyAngles, 0.0, npc_yaw, 0.0);
                let enemy_health = ctx.world.entity(enemy_id).health;
                let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
                let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
                if enemy_health > 0 // enemy still alive
                    && (distance - 350.0).abs() <= 80.0 // enemy anywhere from 270 to 430 away
                    && crate::NPC_senses::InFOV3(enemy_origin, npc_origin, yawOnlyAngles, 20, 20) != 0
                {
                    // enemy generally in front
                    if ctx.world.bg_state.rng.Q_irand(0, 9) == 0 {
                        // 10% chance of doing charge anim
                        // go for the charge
                        doCharge = qtrue;
                        advance = qfalse;
                    }
                }
            }

            if (advance != 0 || (*npc_info).localState == LSTATE_WAITING)
                && crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"attacking".as_ptr()) != 0
            {
                // waiting monsters can't attack
                if crate::g_timer::TIMER_Done2(ctx, Some(npc_id), c"takingPain".as_ptr(), qtrue)
                    != 0
                {
                    (*npc_info).localState = LSTATE_CLEAR;
                } else {
                    Wampa_Move(ctx, 1);
                }
            } else {
                if ctx.world.bg_state.rng.Q_irand(0, 20) == 0 {
                    // FIXME: only do this if we just damaged them or vice-versa?
                    if Wampa_CheckRoar(ctx, npc_id) != 0 {
                        return;
                    }
                }
                if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
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
    // FLAG: Wampa pool client (self), deref raw via the safe entity borrow (trap 2b).
    let self_client = ctx.world.entity(self_).client;
    // FLAG: gNPC_t (self->NPC) has no accessor; derefs stay raw (recipe 2c).
    let self_npc = ctx.world.entity(self_).NPC;

    unsafe {
        let mut hitByWampa = qfalse;
        if let Some(attacker_id) = attacker {
            // FLAG: arbitrary attacker — client may be real or NPC pool; read the
            // pointer via the safe entity borrow, deref raw (trap 2c).
            let attacker_client = ctx.world.entity(attacker_id).client;
            if !attacker_client.is_null()
                && (*attacker_client).NPC_class == crate::prelude::CLASS_WAMPA
            {
                hitByWampa = qtrue;
            }
        }
        if let Some(attacker_id) = attacker {
            if ctx.world.entity(attacker_id).inuse != 0
                && Some(attacker_id) != ctx.world.entity(self_).enemy
                && (ctx.world.entity(attacker_id).flags & crate::prelude::FL_NOTARGET) == 0
            {
                let attacker_number = ctx.world.entity(attacker_id).s.number;
                let self_enemy = ctx.world.entity(self_).enemy;
                let self_origin = ctx.world.entity(self_).r.currentOrigin;
                let attacker_origin = ctx.world.entity(attacker_id).r.currentOrigin;
                let enemy_health = self_enemy.map(|id| ctx.world.entity(id).health);
                let enemy_origin = self_enemy.map(|id| ctx.world.entity(id).r.currentOrigin);
                // FLAG: enemy client may be real or NPC pool; read ptr via safe
                // borrow, deref raw (trap 2c).
                let enemy_is_wampa = match self_enemy {
                    Some(id) => {
                        let c = ctx.world.entity(id).client;
                        !c.is_null() && (*c).NPC_class == crate::prelude::CLASS_WAMPA
                    }
                    None => false,
                };
                if (attacker_number == 0 && ctx.world.bg_state.rng.Q_irand(0, 3) == 0)
                    || self_enemy.is_none()
                    || enemy_health == Some(0)
                    || enemy_is_wampa
                    || (ctx.world.bg_state.rng.Q_irand(0, 4) == 0
                        && crate::q_math::DistanceSquared(attacker_origin, self_origin)
                            < crate::q_math::DistanceSquared(enemy_origin.unwrap(), self_origin))
                {
                    // if my enemy is dead (or attacked by player) and I'm not still holding/eating someone, turn on the attacker
                    // FIXME: if can't nav to my enemy, take this guy if I can nav to him
                    crate::NPC_combat::G_SetEnemy(ctx, self_, Some(attacker_id));
                    let look_for_new_enemy = ctx.world.bg_state.rng.Q_irand(5000, 15000);
                    crate::g_timer::TIMER_Set(
                        ctx,
                        Some(self_),
                        c"lookForNewEnemy".as_ptr(),
                        look_for_new_enemy,
                    );
                    if hitByWampa != 0 {
                        let wampa_infight = ctx.world.bg_state.rng.Q_irand(2000, 5000);
                        // stay mad at this Wampa for 2-5 secs before looking for attacker enemies
                        crate::g_timer::TIMER_Set(
                            ctx,
                            Some(self_),
                            c"wampaInfight".as_ptr(),
                            wampa_infight,
                        );
                    }
                }
            }
        }
        if (hitByWampa != 0 || ctx.world.bg_state.rng.Q_irand(0, 100) < damage) // hit by wampa, hit while holding live victim, or took a lot of damage
            && (*self_client).ps.legsAnim != (crate::prelude::BOTH_GESTURE1) as i32
            && (*self_client).ps.legsAnim != (crate::prelude::BOTH_GESTURE2) as i32
            && crate::g_timer::TIMER_Done(ctx, Some(self_), c"takingPain".as_ptr()) != 0
        {
            if Wampa_CheckRoar(ctx, self_) == 0 {
                if (*self_client).ps.legsAnim != (crate::prelude::BOTH_ATTACK1) as i32
                    && (*self_client).ps.legsAnim != (crate::prelude::BOTH_ATTACK2) as i32
                    && (*self_client).ps.legsAnim != (crate::prelude::BOTH_ATTACK3) as i32
                {
                    // cant interrupt one of the big attack anims
                    if ctx.world.entity(self_).health > 100 || hitByWampa != 0 {
                        crate::g_timer::TIMER_Remove(ctx, Some(self_), c"attacking".as_ptr());

                        let last_path_angles = (*self_npc).lastPathAngles;
                        crate::q_math::_VectorCopy(
                            last_path_angles,
                            &mut ctx.world.entity_mut(self_).s.angles,
                        );

                        if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
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
                        // Oracle reads legsTimer + draws Q_irand(0,500) LAST —
                        // after the anim-pick draw and after NPC_SetAnim.
                        // Source: oracle/codemp/game/NPC_AI_Wampa.c:485
                        let taking_pain =
                            (*self_client).ps.legsTimer + ctx.world.bg_state.rng.Q_irand(0, 500);
                        crate::g_timer::TIMER_Set(
                            ctx,
                            Some(self_),
                            c"takingPain".as_ptr(),
                            taking_pain,
                        );
                        // allow us to re-evaluate our running speed/anim
                        crate::g_timer::TIMER_Set(ctx, Some(self_), c"runfar".as_ptr(), -1);
                        crate::g_timer::TIMER_Set(ctx, Some(self_), c"runclose".as_ptr(), -1);
                        crate::g_timer::TIMER_Set(ctx, Some(self_), c"walk".as_ptr(), -1);

                        if !self_npc.is_null() {
                            (*self_npc).localState = LSTATE_WAITING;
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
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;
    // FLAG: Wampa pool client, deref raw via the safe entity borrow (trap 2b).
    let client = ctx.world.entity(npc_id).client;

    unsafe {
        (*client).ps.eFlags2 &= !mp_bg::public::entity_effects::EF2_USE_ALT_ANIM;
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
        if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"rageTime".as_ptr()) == 0 {
            // do nothing but roar first time we see an enemy
            crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);
            return;
        }
        if ctx.world.entity(npc_id).enemy.is_some() {
            // Guaranteed `Some` inside this block by the guard above (mirrors
            // Raven's unguarded `NPC->enemy->x` once `NPC->enemy` is known set).
            let enemy_id = ctx.world.entity(npc_id).enemy.unwrap();
            if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"attacking".as_ptr()) == 0 {
                // in middle of attack
                // face enemy
                crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);
                // continue attack logic
                let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
                let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
                ctx.world.globals.enemyDist = crate::q_math::Distance(npc_origin, enemy_origin);
                let enemy_dist = ctx.world.globals.enemyDist;
                Wampa_Attack(ctx, enemy_dist, qfalse);
                return;
            } else {
                if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"angrynoise".as_ptr()) != 0 {
                    let angrynoise_snd = format!(
                        "sound/chars/wampa/misc/anger{}.wav",
                        ctx.world.bg_state.rng.Q_irand(1, 2)
                    );
                    let sound = G_SoundIndex(ctx, &angrynoise_snd);
                    crate::g_utils::G_Sound(
                        ctx,
                        Some(npc_id),
                        crate::prelude::CHAN_VOICE,
                        sound,
                    );

                    let angrynoise = ctx.world.bg_state.rng.Q_irand(5000, 10000);
                    crate::g_timer::TIMER_Set(
                        ctx,
                        Some(npc_id),
                        c"angrynoise".as_ptr(),
                        angrynoise,
                    );
                }
                // else, if he's in our hand, we eat, else if he's on the ground, we keep attacking his dead body for a while
                // FLAG: enemy client may be real or NPC pool; read ptr via safe
                // borrow, deref raw (trap 2c).
                let enemy_is_wampa = {
                    let c = ctx.world.entity(enemy_id).client;
                    !c.is_null() && (*c).NPC_class == crate::prelude::CLASS_WAMPA
                };
                if enemy_is_wampa {
                    // got mad at another Wampa, look for a valid enemy
                    if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"wampaInfight".as_ptr()) != 0
                    {
                        crate::NPC_utils::NPC_CheckEnemyExt(ctx, qtrue);
                    }
                } else {
                    let enemy_now = ctx.world.entity(npc_id).enemy;
                    if crate::NPC_combat::ValidEnemy(ctx, enemy_now) == qfalse {
                        crate::g_timer::TIMER_Remove(
                            ctx,
                            Some(npc_id),
                            c"lookForNewEnemy".as_ptr(),
                        ); // make them look again right now
                        if ctx.world.entity(enemy_id).inuse == 0
                            || ctx.world.level.time - ctx.world.entity(enemy_id).s.time
                                > ctx.world.bg_state.rng.Q_irand(10000, 15000)
                        {
                            // it's been a while since the enemy died, or enemy is completely gone, get bored with him
                            ctx.world.entity_mut(npc_id).enemy = None;
                            Wampa_Patrol(ctx);
                            crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
                            // just lost my enemy
                            if (ctx.world.entity(npc_id).spawnflags & 2) != 0 {
                                // search around me if I don't have an enemy
                                let waypoint = ctx.world.entity(npc_id).waypoint;
                                crate::NPC_behavior::NPC_BSSearchStart(
                                    ctx,
                                    waypoint,
                                    crate::prelude::BS_SEARCH,
                                );
                                (*npc_info).tempBehavior = crate::prelude::BS_DEFAULT;
                            } else if (ctx.world.entity(npc_id).spawnflags & 1) != 0 {
                                // wander if I don't have an enemy
                                let waypoint = ctx.world.entity(npc_id).waypoint;
                                crate::NPC_behavior::NPC_BSSearchStart(
                                    ctx,
                                    waypoint,
                                    crate::prelude::BS_WANDER,
                                );
                                (*npc_info).tempBehavior = crate::prelude::BS_DEFAULT;
                            }
                            return;
                        }
                    }
                    if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"lookForNewEnemy".as_ptr())
                        != 0
                    {
                        let newEnemy;
                        let sav_enemy = ctx.world.entity(npc_id).enemy; // FIXME: what about NPC->lastEnemy?
                        ctx.world.entity_mut(npc_id).enemy = None;
                        let check_all = if (*npc_info).confusionTime < ctx.world.level.time {
                            qtrue
                        } else {
                            qfalse
                        };
                        newEnemy =
                            crate::NPC_combat::NPC_CheckEnemy(ctx, check_all, qfalse, qfalse);
                        ctx.world.entity_mut(npc_id).enemy = sav_enemy;
                        if !newEnemy.is_null()
                            && ent_id_opt(ctx.world.g_entities.as_ptr(), newEnemy) != sav_enemy
                        {
                            // picked up a new enemy!
                            let cur_enemy = ctx.world.entity(npc_id).enemy;
                            ctx.world.entity_mut(npc_id).lastEnemy = cur_enemy;
                            crate::NPC_combat::G_SetEnemy(ctx, npc_id, ctx.entity_id_of(newEnemy));
                            let look_for_new_enemy = ctx.world.bg_state.rng.Q_irand(5000, 15000);
                            // hold this one for at least 5-15 seconds
                            crate::g_timer::TIMER_Set(
                                ctx,
                                Some(npc_id),
                                c"lookForNewEnemy".as_ptr(),
                                look_for_new_enemy,
                            );
                        } else {
                            let look_for_new_enemy = ctx.world.bg_state.rng.Q_irand(2000, 5000);
                            // look again in 2-5 secs
                            crate::g_timer::TIMER_Set(
                                ctx,
                                Some(npc_id),
                                c"lookForNewEnemy".as_ptr(),
                                look_for_new_enemy,
                            );
                        }
                    }
                }
                Wampa_Combat(ctx);
                return;
            }
        } else {
            if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"idlenoise".as_ptr()) != 0 {
                let sound = G_SoundIndex(ctx, "sound/chars/wampa/misc/anger3.wav");
                crate::g_utils::G_Sound(
                    ctx,
                    Some(npc_id),
                    crate::prelude::CHAN_AUTO,
                    sound,
                );

                let idlenoise = ctx.world.bg_state.rng.Q_irand(2000, 4000);
                crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"idlenoise".as_ptr(), idlenoise);
            }
            if (ctx.world.entity(npc_id).spawnflags & 2) != 0 {
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
                ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
                crate::NPC_behavior::NPC_BSSearch(ctx); // this automatically looks for enemies
            } else if (ctx.world.entity(npc_id).spawnflags & 1) != 0 {
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
                ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
                crate::NPC_behavior::NPC_BSWander(ctx);
                if ((*npc_info).scriptFlags & crate::prelude::SCF_LOOK_FOR_ENEMIES) != 0 {
                    if crate::NPC_utils::NPC_CheckEnemyExt(ctx, qtrue) == qfalse {
                        Wampa_Idle(ctx);
                    } else {
                        Wampa_CheckRoar(ctx, npc_id);
                        let look_for_new_enemy = ctx.world.bg_state.rng.Q_irand(5000, 15000);
                        crate::g_timer::TIMER_Set(
                            ctx,
                            Some(npc_id),
                            c"lookForNewEnemy".as_ptr(),
                            look_for_new_enemy,
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
