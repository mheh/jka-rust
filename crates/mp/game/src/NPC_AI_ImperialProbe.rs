// PORT-COMPLETE: NPC_AI_ImperialProbe.c
//! FAITHFUL port of `oracle/codemp/game/NPC_AI_ImperialProbe.c`.
//!
//! Imperial Probe droid AI behavior: idle, patrol, hunt, strafe, ranged attack.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_combat::G_Damage;
use crate::g_items::RegisterItem;
use crate::g_missile::CreateMissile;
use crate::g_timer::{TIMER_Done, TIMER_Set};
use crate::g_utils::{G_EffectIndex, G_PlayEffectID, G_Sound, G_SoundIndex, G_SoundOnEnt};
use crate::npc_c::NPC_SetAnim;
use crate::prelude::*;
use crate::q_math::{
    _VectorCopy, _VectorMA, _VectorSubtract, vectoangles, AngleNormalize360, AngleVectors,
    DistanceHorizontalSquared, VectorNormalize,
};
use crate::q_shared::va;
use crate::trap;
use crate::world::{ent_id, ent_id_opt};
use crate::NPC_AI_Default::NPC_BSIdle;
use crate::NPC_AI_Stormtrooper::NPC_CheckPlayerTeamStealth;
use crate::NPC_goal::UpdateGoal;
use crate::NPC_move::{NPC_GetMoveDirection, NPC_MoveToGoal};
use crate::NPC_reactions::{NPC_GetPainChance, NPC_Pain};
use crate::NPC_utils::{
    CalcEntitySpot, NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy, NPC_UpdateAngles,
};
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::bg_misc::BG_GiveMeVectorFromMatrix;
use mp_bg::bg_misc::{BG_FindItemForAmmo, BG_FindItemForWeapon};

// Local state enums
// Source: oracle/codemp/game/NPC_AI_ImperialProbe.c:10-17
const LSTATE_NONE: i32 = 0;
const LSTATE_BACKINGUP: i32 = 1;
const LSTATE_SPINNING: i32 = 2;
const LSTATE_PAIN: i32 = 3;
const LSTATE_DROP: i32 = 4;

// Height maintenance
// Source: oracle/codemp/game/NPC_AI_ImperialProbe.c:47
pub const VELOCITY_DECAY: f32 = 0.85;

// Strafe parameters
// Source: oracle/codemp/game/NPC_AI_ImperialProbe.c:178-181
pub const HUNTER_STRAFE_VEL: c_int = 256;
pub const HUNTER_STRAFE_DIS: c_int = 200;
pub const HUNTER_UPWARD_PUSH: c_int = 32;

// Hunt parameters
// Source: oracle/codemp/game/NPC_AI_ImperialProbe.c:217-218
pub const HUNTER_FORWARD_BASE_SPEED: c_int = 10;
pub const HUNTER_FORWARD_MULTIPLIER: c_int = 5;

// Melee range
// Source: oracle/codemp/game/NPC_AI_ImperialProbe.c:371-375
const MIN_MELEE_RANGE: c_int = 320;
const MIN_MELEE_RANGE_SQR: c_int = MIN_MELEE_RANGE * MIN_MELEE_RANGE;
const MIN_DISTANCE: c_int = 128;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

/// Raven `NPC_Probe_Precache`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:21-40`
pub fn NPC_Probe_Precache(ctx: &mut GameContext) {
    for i in 1..4 {
        let s = format!("sound/chars/probe/misc/probetalk{}", i);
        G_SoundIndex(&s);
    }
    G_SoundIndex("sound/chars/probe/misc/probedroidloop");
    G_SoundIndex("sound/chars/probe/misc/anger1");
    G_SoundIndex("sound/chars/probe/misc/fire");

    G_EffectIndex("chunks/probehead");
    G_EffectIndex("env/med_explode2");
    G_EffectIndex("explosions/probeexplosion1");
    G_EffectIndex("bryar/muzzle_flash");

    RegisterItem(ctx, BG_FindItemForAmmo(AMMO_BLASTER));
    RegisterItem(ctx, BG_FindItemForWeapon(WP_BRYAR_PISTOL));
}

/// Raven `ImperialProbe_MaintainHeight`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:49-170`
pub fn ImperialProbe_MaintainHeight(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: NPC carries a BG_Alloc'd pool client (not level.clients); deref raw
    // via the safe entity borrow, per trap 2b.
    let client = ctx.world.entity(npc_id).client;

    // Update our angles regardless
    NPC_UpdateAngles(ctx, qtrue, qtrue);

    unsafe {
        // If we have an enemy, we should try to hover at about enemy eye level
        if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
            // Find the height difference
            let enemy_z = ctx.world.entity(enemy_id).r.currentOrigin[2];
            let npc_z = ctx.world.entity(npc_id).r.currentOrigin[2];
            let mut dif = enemy_z - npc_z;

            // cap to prevent dramatic height shifts
            if dif.abs() > 8.0 {
                if dif.abs() > 16.0 {
                    dif = if dif < 0.0 { -16.0 } else { 16.0 };
                }

                (*client).ps.velocity[2] = ((*client).ps.velocity[2] + dif) / 2.0;
            }
        } else {
            let goal_id = if (*npc_info).goalEntity.is_some() {
                (*npc_info).goalEntity
            } else {
                (*npc_info).lastGoalEntity
            };

            if let Some(goal_id) = goal_id {
                let goal_z = ctx.world.entity(goal_id).r.currentOrigin[2];
                let npc_z = ctx.world.entity(npc_id).r.currentOrigin[2];
                let dif = goal_z - npc_z;

                if dif.abs() > 24.0 {
                    ctx.world.globals.ucmd.upmove = if ctx.world.globals.ucmd.upmove < 0 {
                        -4
                    } else {
                        4
                    };
                } else {
                    if (*client).ps.velocity[2] != 0.0 {
                        (*client).ps.velocity[2] *= VELOCITY_DECAY;

                        if (*client).ps.velocity[2].abs() < 2.0 {
                            (*client).ps.velocity[2] = 0.0;
                        }
                    }
                }
            } else if (*client).ps.velocity[2] != 0.0 {
                // Apply friction
                (*client).ps.velocity[2] *= VELOCITY_DECAY;

                if (*client).ps.velocity[2].abs() < 1.0 {
                    (*client).ps.velocity[2] = 0.0;
                }
            }
        }

        // Apply friction
        if (*client).ps.velocity[0] != 0.0 {
            (*client).ps.velocity[0] *= VELOCITY_DECAY;

            if (*client).ps.velocity[0].abs() < 1.0 {
                (*client).ps.velocity[0] = 0.0;
            }
        }

        if (*client).ps.velocity[1] != 0.0 {
            (*client).ps.velocity[1] *= VELOCITY_DECAY;

            if (*client).ps.velocity[1].abs() < 1.0 {
                (*client).ps.velocity[1] = 0.0;
            }
        }
    }
}

/// Raven `ImperialProbe_Strafe`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:182-209`
pub fn ImperialProbe_Strafe(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: NPC carries a BG_Alloc'd pool client (not level.clients); deref raw
    // via the safe entity borrow, per trap 2b.
    let client = ctx.world.entity(npc_id).client;

    let mut right = [0.0; 3];
    unsafe {
        AngleVectors((*client).renderInfo.eyeAngles, None, Some(&mut right), None);

        // Pick a random strafe direction, then check to see if doing a strafe would be
        // reasonable valid
        let dir = if (ctx.world.bg_state.rng.rand() & 1) != 0 {
            -1
        } else {
            1
        };
        let mut end = [0.0; 3];
        let current_origin = ctx.world.entity(npc_id).r.currentOrigin;
        _VectorMA(
            current_origin,
            (HUNTER_STRAFE_DIS * dir) as f32,
            right,
            &mut end,
        );

        let mut tr: trace_t = core::mem::zeroed();
        let s_number = ctx.world.entity(npc_id).s.number;
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &current_origin as *const vec3_t,
                core::ptr::null(),
                core::ptr::null(),
                &end as *const vec3_t,
                s_number,
                MASK_SOLID,
            ),
        );

        // Close enough
        if tr.fraction > 0.9 {
            _VectorMA(
                (*client).ps.velocity,
                (HUNTER_STRAFE_VEL * dir) as f32,
                right,
                &mut (*client).ps.velocity,
            );

            // Add a slight upward push
            (*client).ps.velocity[2] += HUNTER_UPWARD_PUSH as f32;

            // Set the strafe start time so we can do a controlled roll
            let level_time = ctx.world.level.time;
            let roll = (ctx.world.bg_state.rng.random() * 500.0) as i32;
            (*npc_info).standTime = level_time + 3000 + roll;
        }
    }
}

/// Raven `ImperialProbe_Hunt`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:220-261`
pub fn ImperialProbe_Hunt(ctx: &mut GameContext, visible: qboolean, advance: qboolean) {
    let npc = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: NPC carries a BG_Alloc'd pool client (not level.clients); deref raw
    // via the safe entity borrow, per trap 2b.
    let client = ctx.world.entity(npc_id).client;

    let mut forward = [0.0; 3];
    let mut distance = 0.0;

    NPC_SetAnim(
        ctx,
        npc_id,
        SETANIM_BOTH,
        BOTH_RUN1 as c_int,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
    );

    unsafe {
        // If we're not supposed to stand still, pursue the player
        if (*npc_info).standTime < ctx.world.level.time {
            // Only strafe when we can see the player
            if visible != 0 {
                ImperialProbe_Strafe(ctx);
                return;
            }
        }

        // If we don't want to advance, stop here
        if advance == 0 {
            return;
        }

        // Only try and navigate if the player is visible
        if visible == 0 {
            // Move towards our goal
            (*npc_info).goalEntity = ctx.world.entity(npc_id).enemy;
            (*npc_info).goalRadius = 12;

            // Get our direction from the navigator if we can't see our target
            if NPC_GetMoveDirection(ctx, &mut forward, &mut distance as *mut f32) == 0 {
                return;
            }
        } else {
            if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
                let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
                let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
                _VectorSubtract(enemy_origin, npc_origin, &mut forward);
                distance = VectorNormalize(&mut forward);
            }
        }

        let speed = HUNTER_FORWARD_BASE_SPEED as f32
            + HUNTER_FORWARD_MULTIPLIER as f32 * ctx.world.cvars.g_spskill.integer as f32;
        _VectorMA(
            (*client).ps.velocity,
            speed,
            forward,
            &mut (*client).ps.velocity,
        );
    }
}

/// Raven `ImperialProbe_FireBlaster`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:268-324`
pub fn ImperialProbe_FireBlaster(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let mut muzzle1 = [0.0; 3];
    let mut enemy_org1 = [0.0; 3];
    let mut delta1 = [0.0; 3];
    let mut angleToEnemy1 = [0.0; 3];
    let mut forward = [0.0; 3];
    let mut vright = [0.0; 3];
    let mut up = [0.0; 3];

    let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };

    let ghoul2 = ctx.world.entity(npc_id).ghoul2;
    let gen_bolt_1 = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*flash");

    let current_angles = ctx.world.entity(npc_id).r.currentAngles;
    let current_origin = ctx.world.entity(npc_id).r.currentOrigin;
    let model_scale = ctx.world.entity(npc_id).modelScale;
    let level_time = ctx.world.level.time;
    trap::G2API_GetBoltMatrix(
        ctx.engine,
        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
            ghoul2,
            0,
            gen_bolt_1,
            &mut boltMatrix as *mut mdxaBone_t,
            &current_angles as *const vec3_t,
            &current_origin as *const vec3_t,
            level_time,
            core::ptr::null_mut(),
            &model_scale as *const vec3_t,
        ),
    );

    BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut muzzle1);

    G_PlayEffectID(
        G_EffectIndex("bryar/muzzle_flash"),
        muzzle1,
        [0.0; 3],
    );

    G_Sound(
        ctx,
        Some(npc_id),
        CHAN_AUTO,
        G_SoundIndex("sound/chars/probe/misc/fire"),
    );

    if ctx.world.entity(npc_id).health != 0 {
        // `ctx.entity_id_of(enemy_ptr)` round-trips the entity's `.enemy`
        // handle (None when null), so read it straight off the accessor.
        let enemy_eid = ctx.world.entity(npc_id).enemy;
        CalcEntitySpot(ctx, enemy_eid, SPOT_CHEST, &mut enemy_org1);
        enemy_org1[0] += ctx.world.bg_state.rng.Q_irand(0, 10) as f32;
        enemy_org1[1] += ctx.world.bg_state.rng.Q_irand(0, 10) as f32;
        _VectorSubtract(enemy_org1, muzzle1, &mut delta1);
        vectoangles(delta1, &mut angleToEnemy1);
        AngleVectors(
            angleToEnemy1,
            Some(&mut forward),
            Some(&mut vright),
            Some(&mut up),
        );
    } else {
        let current_angles = ctx.world.entity(npc_id).r.currentAngles;
        AngleVectors(
            current_angles,
            Some(&mut forward),
            Some(&mut vright),
            Some(&mut up),
        );
    }

    let missile_id = CreateMissile(ctx, muzzle1, forward, 1600.0, 10000, npc_id, false);

    ctx.ent_set(missile_id, PrefixSet::ClassnameStatic(c"bryar_proj"));
    ctx.world.entity_mut(missile_id).s.weapon = WP_BRYAR_PISTOL as c_int;

    if ctx.world.cvars.g_spskill.integer <= 1 {
        ctx.world.entity_mut(missile_id).damage = 5;
    } else {
        ctx.world.entity_mut(missile_id).damage = 10;
    }

    ctx.world.entity_mut(missile_id).dflags = DAMAGE_DEATH_KNOCKBACK;
    ctx.world.entity_mut(missile_id).methodOfDeath = MOD_UNKNOWN as c_int;
    ctx.world.entity_mut(missile_id).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
}

/// Raven `ImperialProbe_Ranged`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:331-363`
pub fn ImperialProbe_Ranged(ctx: &mut GameContext, visible: qboolean, advance: qboolean) {
    let npc = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    if TIMER_Done(ctx, Some(npc_id), c"attackDelay".as_ptr()) != 0 {
        let (_delay_min, _delay_max) = if ctx.world.cvars.g_spskill.integer == 0 {
            (500, 3000)
        } else if ctx.world.cvars.g_spskill.integer > 1 {
            (500, 2000)
        } else {
            (300, 1500)
        };

        let atk_delay = ctx.world.bg_state.rng.Q_irand(500, 3000);
        TIMER_Set(ctx, Some(npc_id), c"attackDelay".as_ptr(), atk_delay);
        ImperialProbe_FireBlaster(ctx);
    }

    unsafe {
        if ((*npc_info).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
            ImperialProbe_Hunt(ctx, visible, advance);
        }
    }
}

/// Raven `ImperialProbe_AttackDecision`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:377-426`
pub fn ImperialProbe_AttackDecision(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    // Always keep a good height off the ground
    ImperialProbe_MaintainHeight(ctx);

    // randomly talk
    if TIMER_Done(ctx, Some(npc_id), c"patrolNoise".as_ptr()) != 0 {
        if TIMER_Done(ctx, Some(npc_id), c"angerNoise".as_ptr()) != 0 {
            let sound_idx = ctx.world.bg_state.rng.Q_irand(1, 3);
            let s = format!("sound/chars/probe/misc/probetalk{}", sound_idx);
            G_SoundOnEnt(ctx, npc_id, CHAN_AUTO, &s);

            let patrol_delay = ctx.world.bg_state.rng.Q_irand(4000, 10000);
            TIMER_Set(ctx, Some(npc_id), c"patrolNoise".as_ptr(), patrol_delay);
        }
    }

    // If we don't have an enemy, just idle
    if NPC_CheckEnemyExt(ctx, 0) == 0 {
        ImperialProbe_Idle(ctx);
        return;
    }

    NPC_SetAnim(
        ctx,
        npc_id,
        SETANIM_BOTH,
        BOTH_RUN1 as c_int,
        SETANIM_FLAG_NORMAL,
    );

    // Rate our distance to the target, and our visibility
    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
    let enemy_origin = if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
        ctx.world.entity(enemy_id).r.currentOrigin
    } else {
        [0.0; 3]
    };
    let distance = DistanceHorizontalSquared(npc_origin, enemy_origin) as c_int;
    let visible = if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
        NPC_ClearLOS4(ctx, Some(enemy_id))
    } else {
        0
    };
    let advance = if distance > MIN_DISTANCE_SQR { 1 } else { 0 };

    // If we cannot see our target, move to see it
    if visible == 0 {
        let script_flags = unsafe { (*npc_info).scriptFlags };
        if (script_flags & SCF_CHASE_ENEMIES) != 0 {
            ImperialProbe_Hunt(ctx, visible, advance);
            return;
        }
    }

    // Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
    NPC_FaceEnemy(ctx, 1);

    // Decide what type of attack to do
    ImperialProbe_Ranged(ctx, visible, advance);
}

/// Raven `NPC_Probe_Pain`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:433-498`
pub fn NPC_Probe_Pain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    let mod_ = ctx.world.globals.gPainMOD;

    // VectorCopy( self->NPC->lastPathAngles, self->s.angles )
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let self_npc = ctx.world.entity(self_).NPC;
    let last_path_angles = unsafe { (*self_npc).lastPathAngles };
    ctx.world.entity_mut(self_).s.angles = last_path_angles;

    if ctx.world.entity(self_).health < 30
        || mod_ == MOD_DEMP2 as c_int
        || mod_ == MOD_DEMP2_ALT as c_int
    {
        let current_origin = ctx.world.entity(self_).r.currentOrigin;
        let mut end_pos = [
            current_origin[0],
            current_origin[1],
            current_origin[2] - 128.0,
        ];
        let mut trace: trace_t = unsafe { core::mem::zeroed() };
        let s_number = ctx.world.entity(self_).s.number;

        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut trace as *mut trace_t,
                &current_origin as *const vec3_t,
                core::ptr::null(),
                core::ptr::null(),
                &end_pos as *const vec3_t,
                s_number,
                MASK_SOLID,
            ),
        );

        if trace.fraction == 1.0 || mod_ == MOD_DEMP2 as c_int {
            if (mod_ == MOD_DEMP2 as c_int || mod_ == MOD_DEMP2_ALT as c_int) && attacker.is_some()
            {
                let attacker_id = attacker.unwrap();
                let mut dir = [0.0; 3];

                NPC_SetAnim(
                    ctx,
                    self_,
                    SETANIM_BOTH,
                    BOTH_PAIN1 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );

                let self_origin = ctx.world.entity(self_).r.currentOrigin;
                let other_origin = ctx.world.entity(attacker_id).r.currentOrigin;
                _VectorSubtract(self_origin, other_origin, &mut dir);
                VectorNormalize(&mut dir);

                // FLAG: pool client, deref raw via safe entity borrow (trap 2b).
                let client = ctx.world.entity(self_).client;
                unsafe {
                    _VectorMA(
                        (*client).ps.velocity,
                        550.0,
                        dir,
                        &mut (*client).ps.velocity,
                    );
                    (*client).ps.velocity[2] -= 127.0;
                }
            }

            let level_time = ctx.world.level.time;
            // FLAG: pool client, deref raw via safe entity borrow (trap 2b).
            let client = ctx.world.entity(self_).client;
            unsafe {
                (*client).ps.electrifyTime = level_time + 3000;
            }

            // FLAG: gNPC_t (NPCInfo) has no accessor; deref stays raw (recipe 2c).
            unsafe {
                (*self_npc).localState = LSTATE_DROP;
            }
        }
    } else {
        let pain_chance = NPC_GetPainChance(ctx, self_, damage);

        if ctx.world.bg_state.rng.random() < pain_chance {
            NPC_SetAnim(
                ctx,
                self_,
                SETANIM_BOTH,
                BOTH_PAIN1 as c_int,
                SETANIM_FLAG_OVERRIDE,
            );
        }
    }

    NPC_Pain(ctx, self_, attacker, damage);
}

/// Raven `ImperialProbe_Idle`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:506-511`
pub fn ImperialProbe_Idle(ctx: &mut GameContext) {
    ImperialProbe_MaintainHeight(ctx);
    NPC_BSIdle(ctx);
}

/// Raven `ImperialProbe_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:518-556`
pub fn ImperialProbe_Patrol(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    ImperialProbe_MaintainHeight(ctx);

    if NPC_CheckPlayerTeamStealth(ctx) != 0 {
        NPC_UpdateAngles(ctx, qtrue, qtrue);
        return;
    }

    // If we have somewhere to go, then do that
    if ctx.world.entity(npc_id).enemy.is_none() {
        NPC_SetAnim(
            ctx,
            npc_id,
            SETANIM_BOTH,
            BOTH_RUN1 as c_int,
            SETANIM_FLAG_NORMAL,
        );

        if UpdateGoal(ctx) != core::ptr::null_mut() {
            // start loop sound once we move
            let loop_sound = G_SoundIndex("sound/chars/probe/misc/probedroidloop");
            ctx.world.entity_mut(npc_id).s.loopSound = loop_sound;
            ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(ctx, 1);
        }
        // randomly talk
        if TIMER_Done(ctx, Some(npc_id), c"patrolNoise".as_ptr()) != 0 {
            let sound_idx = ctx.world.bg_state.rng.Q_irand(1, 3);
            let s = format!("sound/chars/probe/misc/probetalk{}", sound_idx);
            G_SoundOnEnt(ctx, npc_id, CHAN_AUTO, &s);

            let patrol_delay = ctx.world.bg_state.rng.Q_irand(2000, 4000);
            TIMER_Set(ctx, Some(npc_id), c"patrolNoise".as_ptr(), patrol_delay);
        }
    } else {
        // He's got an enemy. Make him angry.
        G_SoundOnEnt(
            ctx,
            npc_id,
            CHAN_AUTO,
            "sound/chars/probe/misc/anger1");
        let anger_delay = ctx.world.bg_state.rng.Q_irand(2000, 4000);
        TIMER_Set(ctx, Some(npc_id), c"angerNoise".as_ptr(), anger_delay);
    }

    NPC_UpdateAngles(ctx, qtrue, qtrue);
}

/// Raven `ImperialProbe_Wait`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:563-582`
pub fn ImperialProbe_Wait(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    unsafe {
        if (*npc_info).localState == LSTATE_DROP {
            let current_origin = ctx.world.entity(npc_id).r.currentOrigin;
            let mut end_pos = [
                current_origin[0],
                current_origin[1],
                current_origin[2] - 32.0,
            ];
            let mut trace: trace_t = core::mem::zeroed();

            (*npc_info).desiredYaw = AngleNormalize360((*npc_info).desiredYaw + 25.0);

            let s_number = ctx.world.entity(npc_id).s.number;
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut trace as *mut trace_t,
                    &current_origin as *const vec3_t,
                    core::ptr::null(),
                    core::ptr::null(),
                    &end_pos as *const vec3_t,
                    s_number,
                    MASK_SOLID,
                ),
            );

            if trace.fraction != 1.0 {
                // `ctx.entity_id_of(enemy_ptr)` round-trips the entity's `.enemy`
                // handle (None when null), so read it straight off the accessor.
                let enemy_id = ctx.world.entity(npc_id).enemy;
                G_Damage(
                    ctx,
                    Some(npc_id),
                    enemy_id,
                    enemy_id,
                    None,
                    [0.0; 3],
                    2000,
                    0,
                    MOD_UNKNOWN as c_int,
                );
            }
        }

        NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}

/// Raven `NPC_BSImperialProbe_Default`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:589-609`
pub fn NPC_BSImperialProbe_Default(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    unsafe {
        if ctx.world.entity(npc_id).enemy.is_some() {
            (*npc_info).goalEntity = ctx.world.entity(npc_id).enemy;
            ImperialProbe_AttackDecision(ctx);
        } else if ((*npc_info).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            ImperialProbe_Patrol(ctx);
        } else if (*npc_info).localState == LSTATE_DROP {
            ImperialProbe_Wait(ctx);
        } else {
            ImperialProbe_Idle(ctx);
        }
    }
}
