// PORT-COMPLETE: NPC_AI_Remote.c 1/10
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_Remote.c`.
//!
//! One function ported; ten parked due to ambient-state infrastructure.
//! All functions except `NPC_Remote_Precache` rely on file-scope globals set
//! up by `SetNPCGlobals()` (NPC, NPCInfo, ucmd) or read other ambient state
//! (level, g_spskill). The faithful signatures carry no context parameter
//! (`&Engine`, `&mut GameWorld`), and porting-rules §B3 forbids inventing
//! `static mut` globals. How these threadless faithful signatures access the
//! ambient state is an unsettled architectural question.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

/// Raven `NPC_Remote_Precache`.
///
/// Caches sound and effect resources for Remote NPCs at map load time.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:17-22`
pub fn NPC_Remote_Precache(ctx: GameContext<'_>) {
    crate::g_utils::G_SoundIndex(c"sound/chars/remote/misc/fire.wav".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/remote/misc/hiss.wav".as_ptr());
    crate::g_utils::G_EffectIndex(c"env/small_explode".as_ptr());
}

/// Raven `NPC_Remote_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:29-37`
pub fn NPC_Remote_Pain(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    crate::npc_c::SaveNPCGlobals(ctx);
    crate::npc_c::SetNPCGlobals(ctx, self_);
    Remote_Strafe(ctx);
    crate::npc_c::RestoreNPCGlobals(ctx);
    crate::NPC_reactions::NPC_Pain(ctx, self_, attacker, damage);
}

/// Raven `Remote_MaintainHeight`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:44-128`
pub fn Remote_MaintainHeight(ctx: GameContext<'_>) {
    let mut dif: f32;
    let npc = unsafe { (*ctx.world).globals.NPC };
    let npc_info = unsafe { (*ctx.world).globals.NPCInfo };

    // Update our angles regardless
    crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);

    if unsafe { (*(*npc).client).ps.velocity[2] } != 0.0 {
        unsafe { (*(*npc).client).ps.velocity[2] *= VELOCITY_DECAY };

        if unsafe { (*(*npc).client).ps.velocity[2].abs() } < 2.0 {
            unsafe { (*(*npc).client).ps.velocity[2] = 0.0 };
        }
    }

    // If we have an enemy, we should try to hover at or a little below enemy eye level
    if unsafe { (*npc).enemy }.is_some() {
        if crate::g_timer::TIMER_Done(ctx, npc, c"heightChange".as_ptr()) != 0 {
            crate::g_timer::TIMER_Set(
                ctx,
                npc,
                c"heightChange".as_ptr(),
                (*ctx.world).bg_state.rng.Q_irand(1000, 3000),
            );

            // Find the height difference
            let enemy_ent = unsafe {
                let enemy_id = (*npc).enemy.unwrap();
                &mut (*ctx.world).entities[enemy_id.0 as usize] as *mut gentity_t
            };
            dif = unsafe {
                ((*enemy_ent).r.currentOrigin[2]
                    + (*ctx.world).bg_state.rng.Q_irand(0, (*enemy_ent).r.maxs[2] as c_int + 8)
                        as f32)
                    - (*npc).r.currentOrigin[2]
            };

            // cap to prevent dramatic height shifts
            if dif.abs() > 2.0 {
                if dif.abs() > 24.0 {
                    dif = if dif < 0.0 { -24.0 } else { 24.0 };
                }
                dif *= 10.0;
                unsafe { (*(*npc).client).ps.velocity[2] = ((*(*npc).client).ps.velocity[2] + dif) / 2.0 };
                crate::g_utils::G_Sound(
                    ctx,
                    npc,
                    CHAN_AUTO,
                    crate::g_utils::G_SoundIndex(c"sound/chars/remote/misc/hiss.wav".as_ptr()),
                );
            }
        }
    } else {
        let mut goal: *mut gentity_t = core::ptr::null_mut();

        if unsafe { (*npc_info).goalEntity }.is_some() {
            goal = unsafe {
                let goal_id = (*npc_info).goalEntity.unwrap();
                &mut (*ctx.world).entities[goal_id.0 as usize] as *mut gentity_t
            };
        } else if unsafe { (*npc_info).lastGoalEntity }.is_some() {
            goal = unsafe {
                let goal_id = (*npc_info).lastGoalEntity.unwrap();
                &mut (*ctx.world).entities[goal_id.0 as usize] as *mut gentity_t
            };
        }

        if !goal.is_null() {
            dif = unsafe { (*goal).r.currentOrigin[2] - (*npc).r.currentOrigin[2] };

            if dif.abs() > 24.0 {
                dif = if dif < 0.0 { -24.0 } else { 24.0 };
                unsafe { (*(*npc).client).ps.velocity[2] = ((*(*npc).client).ps.velocity[2] + dif) / 2.0 };
            }
        }
    }

    // Apply friction
    if unsafe { (*(*npc).client).ps.velocity[0] } != 0.0 {
        unsafe { (*(*npc).client).ps.velocity[0] *= VELOCITY_DECAY };

        if unsafe { (*(*npc).client).ps.velocity[0].abs() } < 1.0 {
            unsafe { (*(*npc).client).ps.velocity[0] = 0.0 };
        }
    }

    if unsafe { (*(*npc).client).ps.velocity[1] } != 0.0 {
        unsafe { (*(*npc).client).ps.velocity[1] *= VELOCITY_DECAY };

        if unsafe { (*(*npc).client).ps.velocity[1].abs() } < 1.0 {
            unsafe { (*(*npc).client).ps.velocity[1] = 0.0 };
        }
    }
}

/// Raven `Remote_Strafe`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:139-168`
pub fn Remote_Strafe(ctx: GameContext<'_>) {
    use crate::trap;

    let npc = unsafe { (*ctx.world).globals.NPC };
    let npc_info = unsafe { (*ctx.world).globals.NPCInfo };

    let mut dir: c_int;
    let mut end: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut tr: trace_t = Default::default();

    crate::q_math::AngleVectors(
        unsafe { (*(*npc).client).renderInfo.eyeAngles },
        None,
        Some(&mut right),
        None,
    );

    // Pick a random strafe direction, then check to see if doing a strafe would be reasonable valid
    dir = if ((*ctx.world).bg_state.rng.rand() & 1) != 0 { -1 } else { 1 };
    crate::q_math::_VectorMA(
        unsafe { (*npc).r.currentOrigin },
        REMOTE_STRAFE_DIS * dir as f32,
        right,
        &mut end,
    );

    trap::Trace(
        ctx.engine,
        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
            &mut tr as *mut trace_t,
            &unsafe { (*npc).r.currentOrigin } as *const vec3_t,
            core::ptr::null(),
            core::ptr::null(),
            &end as *const vec3_t,
            unsafe { (*npc).s.number },
            MASK_SOLID,
        ),
    );

    // Close enough
    if tr.fraction > 0.9f32 {
        unsafe {
            let client_ref = &mut *(*npc).client;
            crate::q_math::_VectorMA(
                client_ref.ps.velocity,
                REMOTE_STRAFE_VEL * dir as f32,
                right,
                &mut client_ref.ps.velocity,
            );

            crate::g_utils::G_Sound(
                ctx,
                npc,
                CHAN_AUTO,
                crate::g_utils::G_SoundIndex(c"sound/chars/remote/misc/hiss.wav".as_ptr()),
            );

            // Add a slight upward push
            client_ref.ps.velocity[2] += REMOTE_UPWARD_PUSH;

            // Set the strafe start time so we can do a controlled roll
            (*npc_info).standTime =
                (*ctx.world).level.time + 3000 + (*ctx.world).bg_state.rng.random() * 500.0;
        }
    }
}

/// Raven `Remote_Hunt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:178-221`
pub fn Remote_Hunt(
    ctx: GameContext<'_>,
    visible: qboolean,
    advance: qboolean,
    retreat: qboolean,
) {
    let npc = unsafe { (*ctx.world).globals.NPC };
    let npc_info = unsafe { (*ctx.world).globals.NPCInfo };

    let mut distance: f32;
    let mut speed: f32;
    let mut forward: vec3_t = [0.0; 3];

    // If we're not supposed to stand still, pursue the player
    if unsafe { (*npc_info).standTime < (*ctx.world).level.time } {
        // Only strafe when we can see the player
        if visible != 0 {
            Remote_Strafe(ctx);
            return;
        }
    }

    // If we don't want to advance, stop here
    if advance == 0 && visible != 0 {
        return;
    }

    // Only try and navigate if the player is visible
    if visible == 0 {
        // Move towards our goal
        unsafe {
            (*npc_info).goalEntity = (*npc).enemy;
            (*npc_info).goalRadius = 12;
        }

        // Get our direction from the navigator if we can't see our target
        // PORT-NOTE(npc_getmovedirection-signature): packet signature uses `mut out: vec3_t` but call needs `&mut vec3_t`
        if crate::NPC_move::NPC_GetMoveDirection(ctx, &mut forward, &mut distance as *mut f32) == 0 {
            return;
        }
    } else {
        unsafe {
            if let Some(enemy_id) = (*npc).enemy {
                let enemy_ent = &mut (*ctx.world).entities[enemy_id.0 as usize] as *mut gentity_t;
                crate::q_math::_VectorSubtract(
                    (*enemy_ent).r.currentOrigin,
                    (*npc).r.currentOrigin,
                    &mut forward,
                );
                distance = crate::q_math::VectorNormalize(&mut forward);
            }
        }
    }

    speed = REMOTE_FORWARD_BASE_SPEED + REMOTE_FORWARD_MULTIPLIER * unsafe { (*ctx.world).cvars.g_spskill.integer } as f32;
    if retreat != 0 {
        speed *= -1.0;
    }

    unsafe {
        let client_ref = &mut *(*npc).client;
        crate::q_math::_VectorMA(client_ref.ps.velocity, speed, forward, &mut client_ref.ps.velocity);
    }
}

/// Raven `Remote_Fire`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:229-257`
pub fn Remote_Fire(ctx: GameContext<'_>) {
    let npc = unsafe { (*ctx.world).globals.NPC };

    let mut delta1: vec3_t = [0.0; 3];
    let mut enemy_org1: vec3_t = [0.0; 3];
    let mut muzzle1: vec3_t = [0.0; 3];
    let mut angleToEnemy1: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut vright: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];

    // PORT-NOTE(static-vec3-locals): Raven used static vec3_t arrays for caching; using local vars here
    unsafe {
        if let Some(enemy_id) = (*npc).enemy {
            let enemy_ent = &mut (*ctx.world).entities[enemy_id.0 as usize] as *mut gentity_t;
            crate::NPC_utils::CalcEntitySpot(ctx, enemy_ent, SPOT_HEAD, &mut enemy_org1);
        }
    }

    crate::q_math::_VectorCopy(unsafe { (*npc).r.currentOrigin }, &mut muzzle1);

    crate::q_math::_VectorSubtract(enemy_org1, muzzle1, &mut delta1);

    crate::q_math::vectoangles(delta1, &mut angleToEnemy1);
    crate::q_math::AngleVectors(angleToEnemy1, Some(&mut forward), Some(&mut vright), Some(&mut up));

    let missile = crate::g_missile::CreateMissile(
        ctx,
        unsafe { (*npc).r.currentOrigin },
        forward,
        1000.0,
        10000,
        npc,
        qfalse,
    );

    crate::g_utils::G_PlayEffectID(
        crate::g_utils::G_EffectIndex(c"bryar/muzzle_flash".as_ptr()),
        unsafe { (*npc).r.currentOrigin },
        forward,
    );

    unsafe {
        (*missile).classname = c"briar".as_ptr();
        (*missile).s.weapon = WP_BRYAR_PISTOL;

        (*missile).damage = 10;
        (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
        (*missile).methodOfDeath = MOD_BRYAR_PISTOL;
        (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
    }
}

/// Raven `Remote_Ranged`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:264-277`
pub fn Remote_Ranged(
    ctx: GameContext<'_>,
    visible: qboolean,
    advance: qboolean,
    retreat: qboolean,
) {
    let npc = unsafe { (*ctx.world).globals.NPC };
    let npc_info = unsafe { (*ctx.world).globals.NPCInfo };

    if crate::g_timer::TIMER_Done(ctx, npc, c"attackDelay".as_ptr()) != 0 {
        crate::g_timer::TIMER_Set(
            ctx,
            npc,
            c"attackDelay".as_ptr(),
            (*ctx.world).bg_state.rng.Q_irand(500, 3000),
        );
        Remote_Fire(ctx);
    }

    if unsafe { ((*npc_info).scriptFlags & SCF_CHASE_ENEMIES) } != 0 {
        Remote_Hunt(ctx, visible, advance, retreat);
    }
}

/// Raven `Remote_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:290-332`
pub fn Remote_Attack(ctx: GameContext<'_>) {
    let npc = unsafe { (*ctx.world).globals.NPC };
    let npc_info = unsafe { (*ctx.world).globals.NPCInfo };

    let mut distance: f32;
    let mut visible: qboolean;
    let mut idealDist: f32;
    let mut advance: qboolean;
    let mut retreat: qboolean;

    if crate::g_timer::TIMER_Done(ctx, npc, c"spin".as_ptr()) != 0 {
        crate::g_timer::TIMER_Set(ctx, npc, c"spin".as_ptr(), (*ctx.world).bg_state.rng.Q_irand(250, 1500));
        unsafe {
            (*npc_info).desiredYaw += (*ctx.world).bg_state.rng.Q_irand(-200, 200) as f32;
        }
    }

    // Always keep a good height off the ground
    Remote_MaintainHeight(ctx);

    // If we don't have an enemy, just idle
    if crate::NPC_utils::NPC_CheckEnemyExt(ctx, qfalse) == qfalse {
        Remote_Idle(ctx);
        return;
    }

    // Rate our distance to the target, and our visibility
    unsafe {
        if let Some(enemy_id) = (*npc).enemy {
            let enemy_ent = &mut (*ctx.world).entities[enemy_id.0 as usize] as *mut gentity_t;
            distance = crate::q_math::DistanceHorizontalSquared((*npc).r.currentOrigin, (*enemy_ent).r.currentOrigin) as c_int as f32;
            visible = if crate::NPC_utils::NPC_ClearLOS4(ctx, enemy_ent) != 0 { qtrue } else { qfalse };
        } else {
            distance = 0.0;
            visible = qfalse;
        }
    }

    idealDist = MIN_DISTANCE_SQR + (MIN_DISTANCE_SQR * (*ctx.world).bg_state.rng.flrand(0.0, 1.0));
    advance = if distance > idealDist * 1.25 { qtrue } else { qfalse };
    retreat = if distance < idealDist * 0.75 { qtrue } else { qfalse };

    // If we cannot see our target, move to see it
    if visible == qfalse {
        if unsafe { ((*npc_info).scriptFlags & SCF_CHASE_ENEMIES) } != 0 {
            Remote_Hunt(ctx, visible, advance, retreat);
            return;
        }
    }

    Remote_Ranged(ctx, visible, advance, retreat);
}

/// Raven `Remote_Idle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:339-344`
pub fn Remote_Idle(ctx: GameContext<'_>) {
    Remote_MaintainHeight(ctx);
    crate::NPC_AI_Default::NPC_BSIdle(ctx);
}

/// Raven `Remote_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:351-367`
pub fn Remote_Patrol(ctx: GameContext<'_>) {
    let npc = unsafe { (*ctx.world).globals.NPC };

    Remote_MaintainHeight(ctx);

    // If we have somewhere to go, then do that
    if unsafe { (*npc).enemy }.is_none() {
        let goal = crate::NPC_goal::UpdateGoal(ctx);
        if !goal.is_null() {
            // start loop sound once we move
            unsafe {
                (*ctx.world).globals.ucmd.buttons |= BUTTON_WALKING;
            }
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
        }
    }

    crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
}

/// Raven `NPC_BSRemote_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:375-389`
pub fn NPC_BSRemote_Default(ctx: GameContext<'_>) {
    let npc = unsafe { (*ctx.world).globals.NPC };
    let npc_info = unsafe { (*ctx.world).globals.NPCInfo };

    if unsafe { (*npc).enemy }.is_some() {
        Remote_Attack(ctx);
    } else if unsafe { ((*npc_info).scriptFlags & SCF_LOOK_FOR_ENEMIES) } != 0 {
        Remote_Patrol(ctx);
    } else {
        Remote_Idle(ctx);
    }
}
