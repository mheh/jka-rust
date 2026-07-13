// PORT-COMPLETE: NPC_AI_ImperialProbe.c 12/12
//! FAITHFUL port of `oracle/codemp/game/NPC_AI_ImperialProbe.c`.
//!
//! Imperial Probe droid AI behavior: idle, patrol, hunt, strafe, ranged attack.
#![allow(non_snake_case, unused, clippy::all)]

use crate::bg_misc::BG_GiveMeVectorFromMatrix;
use crate::bg_misc::{BG_FindItemForAmmo, BG_FindItemForWeapon};
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
pub fn NPC_Probe_Precache(ctx: GameContext<'_>) {
    for i in 1..4 {
        let s = format!("sound/chars/probe/misc/probetalk{}", i);
        let c_str = cstr(&s);
        G_SoundIndex(c_str.as_ptr());
    }
    G_SoundIndex(c"sound/chars/probe/misc/probedroidloop".as_ptr());
    G_SoundIndex(c"sound/chars/probe/misc/anger1".as_ptr());
    G_SoundIndex(c"sound/chars/probe/misc/fire".as_ptr());

    G_EffectIndex(c"chunks/probehead".as_ptr());
    G_EffectIndex(c"env/med_explode2".as_ptr());
    G_EffectIndex(c"explosions/probeexplosion1".as_ptr());
    G_EffectIndex(c"bryar/muzzle_flash".as_ptr());

    RegisterItem(ctx, BG_FindItemForAmmo(AMMO_BLASTER));
    RegisterItem(ctx, BG_FindItemForWeapon(WP_BRYAR_PISTOL));
}

/// Raven `ImperialProbe_MaintainHeight`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:49-170`
pub fn ImperialProbe_MaintainHeight(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let npc = world.globals.NPC;
        let npc_info = world.globals.NPCInfo;

        // Update our angles regardless
        NPC_UpdateAngles(ctx, qtrue, qtrue);

        // If we have an enemy, we should try to hover at about enemy eye level
        if let Some(enemy_id) = (*npc).enemy {
            let enemy = &mut world.g_entities[enemy_id.0 as usize];
            // Find the height difference
            let mut dif = (*enemy).r.currentOrigin[2] - (*npc).r.currentOrigin[2];

            // cap to prevent dramatic height shifts
            if dif.abs() > 8.0 {
                if dif.abs() > 16.0 {
                    dif = if dif < 0.0 { -16.0 } else { 16.0 };
                }

                (*((*npc).client as *mut gclient_t)).ps.velocity[2] =
                    ((*((*npc).client as *mut gclient_t)).ps.velocity[2] + dif) / 2.0;
            }
        } else {
            let mut goal: Option<*mut gentity_t> = None;

            if let Some(goal_entity_id) = (*npc_info).goalEntity {
                goal = Some(&mut world.g_entities[goal_entity_id.0 as usize]);
            } else if let Some(last_goal_id) = (*npc_info).lastGoalEntity {
                goal = Some(&mut world.g_entities[last_goal_id.0 as usize]);
            }

            if let Some(goal_ent) = goal {
                let mut dif = (*goal_ent).r.currentOrigin[2] - (*npc).r.currentOrigin[2];

                if dif.abs() > 24.0 {
                    world.globals.ucmd.upmove = if world.globals.ucmd.upmove < 0 { -4 } else { 4 };
                } else {
                    if (*((*npc).client as *mut gclient_t)).ps.velocity[2] != 0.0 {
                        (*((*npc).client as *mut gclient_t)).ps.velocity[2] *= VELOCITY_DECAY;

                        if (*((*npc).client as *mut gclient_t)).ps.velocity[2].abs() < 2.0 {
                            (*((*npc).client as *mut gclient_t)).ps.velocity[2] = 0.0;
                        }
                    }
                }
            } else if (*((*npc).client as *mut gclient_t)).ps.velocity[2] != 0.0 {
                // Apply friction
                (*((*npc).client as *mut gclient_t)).ps.velocity[2] *= VELOCITY_DECAY;

                if (*((*npc).client as *mut gclient_t)).ps.velocity[2].abs() < 1.0 {
                    (*((*npc).client as *mut gclient_t)).ps.velocity[2] = 0.0;
                }
            }
        }

        // Apply friction
        if (*((*npc).client as *mut gclient_t)).ps.velocity[0] != 0.0 {
            (*((*npc).client as *mut gclient_t)).ps.velocity[0] *= VELOCITY_DECAY;

            if (*((*npc).client as *mut gclient_t)).ps.velocity[0].abs() < 1.0 {
                (*((*npc).client as *mut gclient_t)).ps.velocity[0] = 0.0;
            }
        }

        if (*((*npc).client as *mut gclient_t)).ps.velocity[1] != 0.0 {
            (*((*npc).client as *mut gclient_t)).ps.velocity[1] *= VELOCITY_DECAY;

            if (*((*npc).client as *mut gclient_t)).ps.velocity[1].abs() < 1.0 {
                (*((*npc).client as *mut gclient_t)).ps.velocity[1] = 0.0;
            }
        }
    }
}

/// Raven `ImperialProbe_Strafe`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:182-209`
pub fn ImperialProbe_Strafe(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let npc = world.globals.NPC;
        let npc_info = world.globals.NPCInfo;

        let mut right = [0.0; 3];
        AngleVectors(
            (*((*npc).client as *mut gclient_t)).renderInfo.eyeAngles,
            None,
            Some(&mut right),
            None,
        );

        // Pick a random strafe direction, then check to see if doing a strafe would be
        // reasonable valid
        let dir = if (world.bg_state.rng.rand() & 1) != 0 {
            -1
        } else {
            1
        };
        let mut end = [0.0; 3];
        _VectorMA(
            (*npc).r.currentOrigin,
            (HUNTER_STRAFE_DIS * dir) as f32,
            right,
            &mut end,
        );

        let mut tr: trace_t = unsafe { core::mem::zeroed() };
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &(*npc).r.currentOrigin as *const vec3_t,
                core::ptr::null(),
                core::ptr::null(),
                &end as *const vec3_t,
                (*npc).s.number,
                MASK_SOLID,
            ),
        );

        // Close enough
        if tr.fraction > 0.9 {
            _VectorMA(
                (*((*npc).client as *mut gclient_t)).ps.velocity,
                (HUNTER_STRAFE_VEL * dir) as f32,
                right,
                &mut (*((*npc).client as *mut gclient_t)).ps.velocity,
            );

            // Add a slight upward push
            (*((*npc).client as *mut gclient_t)).ps.velocity[2] += HUNTER_UPWARD_PUSH as f32;

            // Set the strafe start time so we can do a controlled roll
            (*npc_info).standTime =
                world.level.time + 3000 + (world.bg_state.rng.random() * 500.0) as i32;
        }
    }
}

/// Raven `ImperialProbe_Hunt`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:220-261`
pub fn ImperialProbe_Hunt(ctx: GameContext<'_>, visible: qboolean, advance: qboolean) {
    unsafe {
        let world = &mut *ctx.world;
        let npc = world.globals.NPC;
        let npc_info = world.globals.NPCInfo;

        let mut forward = [0.0; 3];
        let mut distance = 0.0;

        NPC_SetAnim(
            ctx,
            npc,
            SETANIM_BOTH,
            BOTH_RUN1 as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );

        // If we're not supposed to stand still, pursue the player
        if (*npc_info).standTime < world.level.time {
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
            (*npc_info).goalEntity = (*npc).enemy;
            (*npc_info).goalRadius = 12;

            // Get our direction from the navigator if we can't see our target
            if NPC_GetMoveDirection(ctx, &mut forward, &mut distance as *mut f32) == 0 {
                return;
            }
        } else {
            if let Some(enemy_id) = (*npc).enemy {
                let enemy = &mut world.g_entities[enemy_id.0 as usize];
                _VectorSubtract(
                    (*enemy).r.currentOrigin,
                    (*npc).r.currentOrigin,
                    &mut forward,
                );
                distance = VectorNormalize(&mut forward);
            }
        }

        let speed = HUNTER_FORWARD_BASE_SPEED as f32
            + HUNTER_FORWARD_MULTIPLIER as f32 * world.cvars.g_spskill.integer as f32;
        _VectorMA(
            (*((*npc).client as *mut gclient_t)).ps.velocity,
            speed,
            forward,
            &mut (*((*npc).client as *mut gclient_t)).ps.velocity,
        );
    }
}

/// Raven `ImperialProbe_FireBlaster`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:268-324`
pub fn ImperialProbe_FireBlaster(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let npc = world.globals.NPC;

        let mut muzzle1 = [0.0; 3];
        let mut enemy_org1 = [0.0; 3];
        let mut delta1 = [0.0; 3];
        let mut angleToEnemy1 = [0.0; 3];
        let mut forward = [0.0; 3];
        let mut vright = [0.0; 3];
        let mut up = [0.0; 3];

        let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };

        let gen_bolt_1 = trap::G2API_AddBolt(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                (*npc).ghoul2,
                0,
                c"*flash".to_owned(),
            ),
        );

        trap::G2API_GetBoltMatrix(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                (*npc).ghoul2,
                0,
                gen_bolt_1,
                &mut boltMatrix as *mut mdxaBone_t,
                &(*npc).r.currentAngles as *const vec3_t,
                &(*npc).r.currentOrigin as *const vec3_t,
                world.level.time,
                core::ptr::null_mut(),
                &(*npc).modelScale as *const vec3_t,
            ),
        );

        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut muzzle1);

        G_PlayEffectID(
            G_EffectIndex(c"bryar/muzzle_flash".as_ptr()),
            muzzle1,
            [0.0; 3],
        );

        G_Sound(
            ctx,
            ctx.entity_id_of(npc),
            CHAN_AUTO,
            G_SoundIndex(c"sound/chars/probe/misc/fire".as_ptr()),
        );

        if (*npc).health != 0 {
            let enemy_ptr = if let Some(enemy_id) = (*npc).enemy {
                &mut world.g_entities[enemy_id.0 as usize] as *mut gentity_t
            } else {
                core::ptr::null_mut()
            };
            CalcEntitySpot(ctx, enemy_ptr, SPOT_CHEST, &mut enemy_org1);
            enemy_org1[0] += world.bg_state.rng.Q_irand(0, 10) as f32;
            enemy_org1[1] += world.bg_state.rng.Q_irand(0, 10) as f32;
            _VectorSubtract(enemy_org1, muzzle1, &mut delta1);
            vectoangles(delta1, &mut angleToEnemy1);
            AngleVectors(
                angleToEnemy1,
                Some(&mut forward),
                Some(&mut vright),
                Some(&mut up),
            );
        } else {
            AngleVectors(
                (*npc).r.currentAngles,
                Some(&mut forward),
                Some(&mut vright),
                Some(&mut up),
            );
        }

        let missile = CreateMissile(
            ctx,
            muzzle1,
            forward,
            1600.0,
            10000,
            ctx.entity_id_of(npc).unwrap(),
            0,
        );

        (*missile).classname = c"bryar_proj".as_ptr().cast_mut();
        (*missile).s.weapon = WP_BRYAR_PISTOL as c_int;

        if world.cvars.g_spskill.integer <= 1 {
            (*missile).damage = 5;
        } else {
            (*missile).damage = 10;
        }

        (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
        (*missile).methodOfDeath = MOD_UNKNOWN as c_int;
        (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
    }
}

/// Raven `ImperialProbe_Ranged`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:331-363`
pub fn ImperialProbe_Ranged(ctx: GameContext<'_>, visible: qboolean, advance: qboolean) {
    unsafe {
        let world = &mut *ctx.world;
        let npc = world.globals.NPC;
        let npc_info = world.globals.NPCInfo;

        if TIMER_Done(ctx, ctx.entity_id_of(npc), c"attackDelay".as_ptr()) != 0 {
            let (_delay_min, _delay_max) = if world.cvars.g_spskill.integer == 0 {
                (500, 3000)
            } else if world.cvars.g_spskill.integer > 1 {
                (500, 2000)
            } else {
                (300, 1500)
            };

            TIMER_Set(
                ctx,
                ctx.entity_id_of(npc),
                c"attackDelay".as_ptr(),
                world.bg_state.rng.Q_irand(500, 3000),
            );
            ImperialProbe_FireBlaster(ctx);
        }

        if ((*npc_info).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
            ImperialProbe_Hunt(ctx, visible, advance);
        }
    }
}

/// Raven `ImperialProbe_AttackDecision`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:377-426`
pub fn ImperialProbe_AttackDecision(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let npc = world.globals.NPC;
        let npc_info = world.globals.NPCInfo;

        // Always keep a good height off the ground
        ImperialProbe_MaintainHeight(ctx);

        // randomly talk
        if TIMER_Done(ctx, ctx.entity_id_of(npc), c"patrolNoise".as_ptr()) != 0 {
            if TIMER_Done(ctx, ctx.entity_id_of(npc), c"angerNoise".as_ptr()) != 0 {
                let sound_idx = world.bg_state.rng.Q_irand(1, 3);
                let s = format!("sound/chars/probe/misc/probetalk{}", sound_idx);
                G_SoundOnEnt(
                    ctx,
                    ctx.entity_id_of(npc).unwrap(),
                    CHAN_AUTO,
                    cstr(&s).as_ptr(),
                );

                TIMER_Set(
                    ctx,
                    ctx.entity_id_of(npc),
                    c"patrolNoise".as_ptr(),
                    world.bg_state.rng.Q_irand(4000, 10000),
                );
            }
        }

        // If we don't have an enemy, just idle
        if NPC_CheckEnemyExt(ctx, 0) == 0 {
            ImperialProbe_Idle(ctx);
            return;
        }

        NPC_SetAnim(
            ctx,
            npc,
            SETANIM_BOTH,
            BOTH_RUN1 as c_int,
            SETANIM_FLAG_NORMAL,
        );

        // Rate our distance to the target, and our visibility
        let distance = DistanceHorizontalSquared(
            (*npc).r.currentOrigin,
            if let Some(enemy_id) = (*npc).enemy {
                (*world.g_entities.as_mut_ptr().add(enemy_id.0 as usize))
                    .r
                    .currentOrigin
            } else {
                [0.0; 3]
            },
        ) as c_int;
        let visible = if let Some(enemy_id) = (*npc).enemy {
            NPC_ClearLOS4(ctx, &mut world.g_entities[enemy_id.0 as usize])
        } else {
            0
        };
        let advance = if distance > MIN_DISTANCE_SQR { 1 } else { 0 };

        // If we cannot see our target, move to see it
        if visible == 0 {
            if ((*npc_info).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
                ImperialProbe_Hunt(ctx, visible, advance);
                return;
            }
        }

        // Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
        NPC_FaceEnemy(ctx, 1);

        // Decide what type of attack to do
        ImperialProbe_Ranged(ctx, visible, advance);
    }
}

/// Raven `NPC_Probe_Pain`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:433-498`
pub fn NPC_Probe_Pain(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    unsafe {
        let world = &mut *ctx.world;

        let other = attacker;
        let mod_ = world.globals.gPainMOD;

        // VectorCopy( self->NPC->lastPathAngles, self->s.angles )
        _VectorCopy(
            (*((*self_).NPC as *mut gNPC_t)).lastPathAngles,
            &mut (*self_).s.angles,
        );

        if (*self_).health < 30 || mod_ == MOD_DEMP2 as c_int || mod_ == MOD_DEMP2_ALT as c_int {
            let mut end_pos = [
                (*self_).r.currentOrigin[0],
                (*self_).r.currentOrigin[1],
                (*self_).r.currentOrigin[2] - 128.0,
            ];
            let mut trace: trace_t = unsafe { core::mem::zeroed() };

            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut trace as *mut trace_t,
                    &(*self_).r.currentOrigin as *const vec3_t,
                    core::ptr::null(),
                    core::ptr::null(),
                    &end_pos as *const vec3_t,
                    (*self_).s.number,
                    MASK_SOLID,
                ),
            );

            if trace.fraction == 1.0 || mod_ == MOD_DEMP2 as c_int {
                if (mod_ == MOD_DEMP2 as c_int || mod_ == MOD_DEMP2_ALT as c_int)
                    && !other.is_null()
                {
                    let mut dir = [0.0; 3];

                    NPC_SetAnim(
                        ctx,
                        self_,
                        SETANIM_BOTH,
                        BOTH_PAIN1 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );

                    _VectorSubtract((*self_).r.currentOrigin, (*other).r.currentOrigin, &mut dir);
                    VectorNormalize(&mut dir);

                    _VectorMA(
                        (*((*self_).client as *mut gclient_t)).ps.velocity,
                        550.0,
                        dir,
                        &mut (*((*self_).client as *mut gclient_t)).ps.velocity,
                    );
                    (*((*self_).client as *mut gclient_t)).ps.velocity[2] -= 127.0;
                }

                (*((*self_).client as *mut gclient_t)).ps.electrifyTime = world.level.time + 3000;

                (*((*self_).NPC as *mut gNPC_t)).localState = LSTATE_DROP;
            }
        } else {
            let pain_chance = NPC_GetPainChance(ctx, self_, damage);

            if world.bg_state.rng.random() < pain_chance {
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
}

/// Raven `ImperialProbe_Idle`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:506-511`
pub fn ImperialProbe_Idle(ctx: GameContext<'_>) {
    ImperialProbe_MaintainHeight(ctx);
    NPC_BSIdle(ctx);
}

/// Raven `ImperialProbe_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:518-556`
pub fn ImperialProbe_Patrol(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let npc = world.globals.NPC;

        ImperialProbe_MaintainHeight(ctx);

        if NPC_CheckPlayerTeamStealth(ctx) != 0 {
            NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }

        // If we have somewhere to go, then do that
        if (*npc).enemy.is_none() {
            NPC_SetAnim(
                ctx,
                npc,
                SETANIM_BOTH,
                BOTH_RUN1 as c_int,
                SETANIM_FLAG_NORMAL,
            );

            if UpdateGoal(ctx) != core::ptr::null_mut() {
                // start loop sound once we move
                (*npc).s.loopSound =
                    G_SoundIndex(c"sound/chars/probe/misc/probedroidloop".as_ptr());
                world.globals.ucmd.buttons |= BUTTON_WALKING;
                NPC_MoveToGoal(ctx, 1);
            }
            // randomly talk
            if TIMER_Done(ctx, ctx.entity_id_of(npc), c"patrolNoise".as_ptr()) != 0 {
                let sound_idx = world.bg_state.rng.Q_irand(1, 3);
                let s = format!("sound/chars/probe/misc/probetalk{}", sound_idx);
                G_SoundOnEnt(
                    ctx,
                    ctx.entity_id_of(npc).unwrap(),
                    CHAN_AUTO,
                    cstr(&s).as_ptr(),
                );

                TIMER_Set(
                    ctx,
                    ctx.entity_id_of(npc),
                    c"patrolNoise".as_ptr(),
                    world.bg_state.rng.Q_irand(2000, 4000),
                );
            }
        } else {
            // He's got an enemy. Make him angry.
            G_SoundOnEnt(
                ctx,
                ctx.entity_id_of(npc).unwrap(),
                CHAN_AUTO,
                c"sound/chars/probe/misc/anger1".as_ptr(),
            );
            TIMER_Set(
                ctx,
                ctx.entity_id_of(npc),
                c"angerNoise".as_ptr(),
                world.bg_state.rng.Q_irand(2000, 4000),
            );
        }

        NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}

/// Raven `ImperialProbe_Wait`.
///
/// Source: `oracle/codemp/game/NPC_AI_ImperialProbe.c:563-582`
pub fn ImperialProbe_Wait(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let npc = world.globals.NPC;
        let npc_info = world.globals.NPCInfo;

        if (*npc_info).localState == LSTATE_DROP {
            let mut end_pos = [
                (*npc).r.currentOrigin[0],
                (*npc).r.currentOrigin[1],
                (*npc).r.currentOrigin[2] - 32.0,
            ];
            let mut trace: trace_t = unsafe { core::mem::zeroed() };

            (*npc_info).desiredYaw = AngleNormalize360((*npc_info).desiredYaw + 25.0);

            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut trace as *mut trace_t,
                    &(*npc).r.currentOrigin as *const vec3_t,
                    core::ptr::null(),
                    core::ptr::null(),
                    &end_pos as *const vec3_t,
                    (*npc).s.number,
                    MASK_SOLID,
                ),
            );

            if trace.fraction != 1.0 {
                let enemy_ptr = if let Some(enemy_id) = (*npc).enemy {
                    &mut world.g_entities[enemy_id.0 as usize]
                } else {
                    core::ptr::null_mut()
                };
                G_Damage(
                    ctx,
                    ctx.entity_id_of(npc),
                    ctx.entity_id_of(enemy_ptr),
                    ctx.entity_id_of(enemy_ptr),
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
pub fn NPC_BSImperialProbe_Default(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let npc = world.globals.NPC;
        let npc_info = world.globals.NPCInfo;

        if (*npc).enemy.is_some() {
            (*npc_info).goalEntity = (*npc).enemy;
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
