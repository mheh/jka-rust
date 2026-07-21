// PORT-COMPLETE: NPC_AI_Remote.c
//! FAITHFUL port of `oracle/codemp/game/NPC_AI_Remote.c`.
//!
//! One function ported; ten parked due to ambient-state infrastructure.
//! All functions except `NPC_Remote_Precache` rely on file-scope globals set
//! up by `SetNPCGlobals()` (NPC, NPCInfo, ucmd) or read other ambient state
//! (level, g_spskill). The faithful signatures carry no context parameter
//! (`&Engine`, `&mut GameWorld`), and porting-rules §B3 forbids inventing
//! `static mut` globals. How these threadless faithful signatures access the
//! ambient state is an unsettled architectural question.
//!
//! safe-state 2c: the entity half is converted to accessor borrows
//! (`ctx.world.entity(npc_id)`); the gNPC_t (`NPCInfo`) half and the NPC's
//! BG_Alloc'd pool client (`gClPtrs`, CLIENT-POINTER TRAP) stay raw in tight
//! unsafe blocks, exactly as Raven derefs them.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_missile::CreateMissile;
use crate::prelude::*;
use crate::g_utils::G_EffectIndex;
use crate::g_utils::G_SoundIndex;
use crate::trap;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;

// Raven's file-scope combat/movement tuning defines.
// Source: `oracle/codemp/game/NPC_AI_Remote.c:6,130-132,170-171,282-283`
const VELOCITY_DECAY: f32 = 0.85;
const REMOTE_STRAFE_VEL: f32 = 256.0;
const REMOTE_STRAFE_DIS: f32 = 200.0;
const REMOTE_UPWARD_PUSH: f32 = 32.0;
const REMOTE_FORWARD_BASE_SPEED: f32 = 10.0;
const REMOTE_FORWARD_MULTIPLIER: f32 = 5.0;
const MIN_DISTANCE: f32 = 80.0;
const MIN_DISTANCE_SQR: f32 = MIN_DISTANCE * MIN_DISTANCE;

/// Raven `NPC_Remote_Precache`.
///
/// Caches sound and effect resources for Remote NPCs at map load time.
///
/// Source: `oracle/codemp/game/NPC_AI_Remote.c:17-22`
pub fn NPC_Remote_Precache(ctx: &mut GameContext) {
    G_SoundIndex("sound/chars/remote/misc/fire.wav");
    G_SoundIndex("sound/chars/remote/misc/hiss.wav");
    G_EffectIndex("env/small_explode");
}

/// Raven `NPC_Remote_Pain`.
///
/// Source: `oracle/codemp/game/NPC_AI_Remote.c:29-37`
pub fn NPC_Remote_Pain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
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
/// Source: `oracle/codemp/game/NPC_AI_Remote.c:44-128`
pub fn Remote_MaintainHeight(ctx: &mut GameContext) {
    let mut dif: f32;
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // CLIENT-POINTER TRAP (2c/2b): the NPC carries a BG_Alloc'd pool client
    // (`gClPtrs`), not a `level.clients` slot — read the pointer through the
    // entity accessor, then deref it raw exactly as Raven does.
    let client = ctx.world.entity(npc_id).client;

    // Update our angles regardless
    crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);

    if unsafe { (*client).ps.velocity[2] } != 0.0 {
        unsafe { (*client).ps.velocity[2] *= VELOCITY_DECAY };

        if unsafe { (*client).ps.velocity[2].abs() } < 2.0 {
            unsafe { (*client).ps.velocity[2] = 0.0 };
        }
    }

    // If we have an enemy, we should try to hover at or a little below enemy eye level
    if ctx.world.entity(npc_id).enemy.is_some() {
        let npc_id_opt = ctx.entity_id_of(npc);
        if crate::g_timer::TIMER_Done(ctx, npc_id_opt, c"heightChange".as_ptr()) != 0 {
            let npc_id_opt = ctx.entity_id_of(npc);
            let delay = ctx.world.bg_state.rng.Q_irand(1000, 3000);
            crate::g_timer::TIMER_Set(ctx, npc_id_opt, c"heightChange".as_ptr(), delay);

            // Find the height difference
            let enemy_id = ctx.world.entity(npc_id).enemy.unwrap();
            let enemy_org2 = ctx.world.entity(enemy_id).r.currentOrigin[2];
            let enemy_maxs2 = ctx.world.entity(enemy_id).r.maxs[2];
            let npc_org2 = ctx.world.entity(npc_id).r.currentOrigin[2];
            let rnd = ctx.world.bg_state.rng.Q_irand(0, enemy_maxs2 as c_int + 8) as f32;
            dif = (enemy_org2 + rnd) - npc_org2;

            // cap to prevent dramatic height shifts
            if dif.abs() > 2.0 {
                if dif.abs() > 24.0 {
                    dif = if dif < 0.0 { -24.0 } else { 24.0 };
                }
                dif *= 10.0;
                unsafe {
                    (*client).ps.velocity[2] = ((*client).ps.velocity[2] + dif) / 2.0;
                }
                crate::g_utils::G_Sound(
                    ctx,
                    ctx.entity_id_of(npc),
                    CHAN_AUTO,
                    G_SoundIndex("sound/chars/remote/misc/hiss.wav"),
                );
            }
        }
    } else {
        // `goalEntity`/`lastGoalEntity` live on gNPC_t (NPCInfo) — deref raw (2c).
        let goal_id: Option<EntityId> = unsafe {
            if (*npc_info).goalEntity.is_some() {
                (*npc_info).goalEntity
            } else if (*npc_info).lastGoalEntity.is_some() {
                (*npc_info).lastGoalEntity
            } else {
                None
            }
        };

        if let Some(goal_id) = goal_id {
            let goal_org2 = ctx.world.entity(goal_id).r.currentOrigin[2];
            let npc_org2 = ctx.world.entity(npc_id).r.currentOrigin[2];
            dif = goal_org2 - npc_org2;

            if dif.abs() > 24.0 {
                dif = if dif < 0.0 { -24.0 } else { 24.0 };
                unsafe {
                    (*client).ps.velocity[2] = ((*client).ps.velocity[2] + dif) / 2.0;
                }
            }
        }
    }

    // Apply friction
    if unsafe { (*client).ps.velocity[0] } != 0.0 {
        unsafe { (*client).ps.velocity[0] *= VELOCITY_DECAY };

        if unsafe { (*client).ps.velocity[0].abs() } < 1.0 {
            unsafe { (*client).ps.velocity[0] = 0.0 };
        }
    }

    if unsafe { (*client).ps.velocity[1] } != 0.0 {
        unsafe { (*client).ps.velocity[1] *= VELOCITY_DECAY };

        if unsafe { (*client).ps.velocity[1].abs() } < 1.0 {
            unsafe { (*client).ps.velocity[1] = 0.0 };
        }
    }
}

/// Raven `Remote_Strafe`.
///
/// Source: `oracle/codemp/game/NPC_AI_Remote.c:139-168`
pub fn Remote_Strafe(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // CLIENT-POINTER TRAP (2c/2b): NPC pool client — ptr via accessor, deref raw.
    let client = ctx.world.entity(npc_id).client;

    let mut dir: c_int;
    let mut end: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut tr: trace_t = unsafe { core::mem::zeroed() };

    crate::q_math::AngleVectors(
        unsafe { (*client).renderInfo.eyeAngles },
        None,
        Some(&mut right),
        None,
    );

    // Pick a random strafe direction, then check to see if doing a strafe would be reasonable valid
    dir = if ctx.world.bg_state.rng.rand() & 1 != 0 {
        -1
    } else {
        1
    };
    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
    crate::q_math::_VectorMA(npc_origin, REMOTE_STRAFE_DIS * dir as f32, right, &mut end);

    let npc_number = ctx.world.entity(npc_id).s.number;
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &npc_origin as *const vec3_t,
            core::ptr::null(),
            core::ptr::null(),
            &end as *const vec3_t,
            npc_number,
            MASK_SOLID,
        ),
    );

    // Close enough
    if tr.fraction > 0.9f32 {
        unsafe {
            let client_ref = &mut *client;
            crate::q_math::_VectorMA(
                client_ref.ps.velocity,
                REMOTE_STRAFE_VEL * dir as f32,
                right,
                &mut client_ref.ps.velocity,
            );
        }

        crate::g_utils::G_Sound(
            ctx,
            ctx.entity_id_of(npc),
            CHAN_AUTO,
            G_SoundIndex("sound/chars/remote/misc/hiss.wav"),
        );

        unsafe {
            // Add a slight upward push
            (*client).ps.velocity[2] += REMOTE_UPWARD_PUSH;

            // Set the strafe start time so we can do a controlled roll (NPCInfo, raw 2c)
            (*npc_info).standTime =
                ctx.world.level.time + 3000 + (ctx.world.bg_state.rng.random() * 500.0) as c_int;
        }
    }
}

/// Raven `Remote_Hunt`.
///
/// Source: `oracle/codemp/game/NPC_AI_Remote.c:178-221`
pub fn Remote_Hunt(ctx: &mut GameContext, visible: qboolean, advance: qboolean, retreat: qboolean) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // CLIENT-POINTER TRAP (2c/2b): NPC pool client — ptr via accessor, deref raw.
    let client = ctx.world.entity(npc_id).client;

    let mut distance: f32 = 0.0;
    let mut speed: f32;
    let mut forward: vec3_t = [0.0; 3];

    // If we're not supposed to stand still, pursue the player (standTime, NPCInfo raw 2c)
    if unsafe { (*npc_info).standTime } < ctx.world.level.time {
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
        let enemy = ctx.world.entity(npc_id).enemy;
        unsafe {
            (*npc_info).goalEntity = enemy;
            (*npc_info).goalRadius = 12;
        }

        // Get our direction from the navigator if we can't see our target
        if crate::NPC_move::NPC_GetMoveDirection(ctx, &mut forward, &mut distance as *mut f32) == 0
        {
            return;
        }
    } else {
        // §19: oracle derefs `NPC->enemy->r.currentOrigin` unconditionally.
        // Source: oracle/codemp/game/NPC_AI_Remote.c:211
        if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
            let enemy_org = ctx.world.entity(enemy_id).r.currentOrigin;
            let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
            crate::q_math::_VectorSubtract(enemy_org, npc_origin, &mut forward);
            distance = crate::q_math::VectorNormalize(&mut forward);
        }
    }

    speed = REMOTE_FORWARD_BASE_SPEED
        + REMOTE_FORWARD_MULTIPLIER * ctx.world.cvars.g_spskill.integer as f32;
    if retreat != 0 {
        speed *= -1.0;
    }

    unsafe {
        let client_ref = &mut *client;
        crate::q_math::_VectorMA(
            client_ref.ps.velocity,
            speed,
            forward,
            &mut client_ref.ps.velocity,
        );
    }
}

/// Raven `Remote_Fire`.
///
/// Source: `oracle/codemp/game/NPC_AI_Remote.c:229-257`
pub fn Remote_Fire(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;

    let mut delta1: vec3_t = [0.0; 3];
    let mut enemy_org1: vec3_t = [0.0; 3];
    let mut muzzle1: vec3_t = [0.0; 3];
    let mut angleToEnemy1: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut vright: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];

    // Oracle's `static vec3_t forward/vright/up`/`muzzle` carry no cross-call
    // state (rewritten every call / unused), so plain locals are byte-faithful
    // (§B3); `enemy_id: Option` mirrors CalcEntitySpot's null-ent early return.
    let enemy_id = ctx.world.entity(npc_id).enemy;
    crate::NPC_utils::CalcEntitySpot(ctx, enemy_id, SPOT_HEAD, &mut enemy_org1);

    crate::q_math::_VectorCopy(npc_origin, &mut muzzle1);

    crate::q_math::_VectorSubtract(enemy_org1, muzzle1, &mut delta1);

    crate::q_math::vectoangles(delta1, &mut angleToEnemy1);
    crate::q_math::AngleVectors(
        angleToEnemy1,
        Some(&mut forward),
        Some(&mut vright),
        Some(&mut up),
    );

    let missile_id = CreateMissile(
        ctx,
        npc_origin,
        forward,
        1000.0,
        10000,
        ctx.entity_id_of(npc).unwrap(),
        false,
    );

    crate::g_utils::G_PlayEffectID(
        G_EffectIndex("bryar/muzzle_flash"),
        npc_origin,
        forward,
    );

    let m = ctx.world.entity_mut(missile_id);
    m.classname = c"briar".as_ptr() as *mut c_char;
    m.s.weapon = WP_BRYAR_PISTOL as c_int;

    m.damage = 10;
    m.dflags = DAMAGE_DEATH_KNOCKBACK;
    m.methodOfDeath = MOD_BRYAR_PISTOL as c_int;
    m.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
}

/// Raven `Remote_Ranged`.
///
/// Source: `oracle/codemp/game/NPC_AI_Remote.c:264-277`
pub fn Remote_Ranged(
    ctx: &mut GameContext,
    visible: qboolean,
    advance: qboolean,
    retreat: qboolean,
) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;

    let npc_id = ctx.entity_id_of(npc);
    if crate::g_timer::TIMER_Done(ctx, npc_id, c"attackDelay".as_ptr()) != 0 {
        let npc_id = ctx.entity_id_of(npc);
        let delay = ctx.world.bg_state.rng.Q_irand(500, 3000);
        crate::g_timer::TIMER_Set(ctx, npc_id, c"attackDelay".as_ptr(), delay);
        Remote_Fire(ctx);
    }

    // scriptFlags lives on gNPC_t (NPCInfo) — deref raw (2c).
    if unsafe { (*npc_info).scriptFlags & SCF_CHASE_ENEMIES } != 0 {
        Remote_Hunt(ctx, visible, advance, retreat);
    }
}

/// Raven `Remote_Attack`.
///
/// Source: `oracle/codemp/game/NPC_AI_Remote.c:290-332`
pub fn Remote_Attack(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let mut distance: f32;
    let mut visible: qboolean;
    let mut idealDist: f32;
    let mut advance: qboolean;
    let mut retreat: qboolean;

    let npc_id_opt = ctx.entity_id_of(npc);
    if crate::g_timer::TIMER_Done(ctx, npc_id_opt, c"spin".as_ptr()) != 0 {
        let npc_id_opt = ctx.entity_id_of(npc);
        let delay = ctx.world.bg_state.rng.Q_irand(250, 1500);
        crate::g_timer::TIMER_Set(ctx, npc_id_opt, c"spin".as_ptr(), delay);
        // desiredYaw lives on gNPC_t (NPCInfo) — deref raw (2c).
        unsafe {
            (*npc_info).desiredYaw += ctx.world.bg_state.rng.Q_irand(-200, 200) as f32;
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
    if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
        let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
        let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
        distance =
            crate::q_math::DistanceHorizontalSquared(npc_origin, enemy_origin) as c_int as f32;
        visible = if crate::NPC_utils::NPC_ClearLOS4(ctx, Some(enemy_id)) != 0 {
            qtrue
        } else {
            qfalse
        };
    } else {
        distance = 0.0;
        visible = qfalse;
    }

    idealDist = MIN_DISTANCE_SQR + (MIN_DISTANCE_SQR * ctx.world.bg_state.rng.flrand(0.0, 1.0));
    // C: `distance`(int) compared against `idealDist`(float)*1.25/0.75(double);
    // the products promote to double and the compare is in double.
    // Source: oracle/codemp/game/NPC_AI_Remote.c:317-318
    advance = if (distance as f64) > idealDist as f64 * 1.25 {
        qtrue
    } else {
        qfalse
    };
    retreat = if (distance as f64) < idealDist as f64 * 0.75 {
        qtrue
    } else {
        qfalse
    };

    // If we cannot see our target, move to see it (scriptFlags, NPCInfo raw 2c)
    if visible == qfalse {
        if unsafe { (*npc_info).scriptFlags & SCF_CHASE_ENEMIES } != 0 {
            Remote_Hunt(ctx, visible, advance, retreat);
            return;
        }
    }

    Remote_Ranged(ctx, visible, advance, retreat);
}

/// Raven `Remote_Idle`.
///
/// Source: `oracle/codemp/game/NPC_AI_Remote.c:339-344`
pub fn Remote_Idle(ctx: &mut GameContext) {
    Remote_MaintainHeight(ctx);
    crate::NPC_AI_Default::NPC_BSIdle(ctx);
}

/// Raven `Remote_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_Remote.c:351-367`
pub fn Remote_Patrol(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    Remote_MaintainHeight(ctx);

    // If we have somewhere to go, then do that
    if ctx.world.entity(npc_id).enemy.is_none() {
        let goal = crate::NPC_goal::UpdateGoal(ctx);
        if !goal.is_null() {
            // start loop sound once we move
            ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
        }
    }

    crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
}

/// Raven `NPC_BSRemote_Default`.
///
/// Source: `oracle/codemp/game/NPC_AI_Remote.c:375-389`
pub fn NPC_BSRemote_Default(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    if ctx.world.entity(npc_id).enemy.is_some() {
        Remote_Attack(ctx);
    } else if unsafe { (*npc_info).scriptFlags & SCF_LOOK_FOR_ENEMIES } != 0 {
        Remote_Patrol(ctx);
    } else {
        Remote_Idle(ctx);
    }
}
