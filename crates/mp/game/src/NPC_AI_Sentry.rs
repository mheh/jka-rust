//! Faithful port of `oracle/codemp/game/NPC_AI_Sentry.c` (MP only).
//!
//! Sentry gun AI behavior: hovering turret NPC that maintains height, fires
//! at enemies, and has separate idle/patrol/attack states.
//!
//! Source: `oracle/codemp/game/NPC_AI_Sentry.c`
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
// Explicit imports to dedupe E0659 glob ambiguities (known MASK_*/CONTENTS_* debt,
// the SFL_*/SVF_* pattern extended to surface-flag consts): several game-tier
// modules glob-export local duplicates of these; the canonical definition is
// `mp_qshared::shared::surface_flags`.
use mp_qshared::shared::surface_flags::{CONTENTS_LIGHTSABER, MASK_SHOT};

/// Sentry hover height constants.
const SENTRY_HOVER_HEIGHT: f32 = 24.0f32;
const SENTRY_VELOCITY_DECAY: f32 = 0.85f32;
const SENTRY_STRAFE_DIS: f32 = 200.0f32;
const SENTRY_STRAFE_VEL: f32 = 256.0f32;
const SENTRY_UPWARD_PUSH: f32 = 32.0f32;
const SENTRY_FORWARD_BASE_SPEED: f32 = 10.0f32;
const SENTRY_FORWARD_MULTIPLIER: f32 = 5.0f32;
/// `MIN_DISTANCE` 256; `MIN_DISTANCE_SQR` = 256*256 (no separate `MIN_DISTANCE`
/// const ported — only the squared form is used at call sites).
/// Source: `oracle/codemp/game/NPC_AI_Sentry.c:9-10`
const MIN_DISTANCE_SQR: f32 = 65536.0f32;

/// Sentry `localState` enum (anonymous enum local to this TU).
///
/// Source: `oracle/codemp/game/NPC_AI_Sentry.c:23-30`
const LSTATE_NONE: i32 = 0;
const LSTATE_ASLEEP: i32 = 1;
const LSTATE_WAKEUP: i32 = 2;
const LSTATE_ACTIVE: i32 = 3;
const LSTATE_POWERING_UP: i32 = 4;
const LSTATE_ATTACKING: i32 = 5;

/// `NPC_Sentry_Precache` — Precache sounds and effects for the sentry gun.
///
/// Source: `oracle/codemp/game/NPC_AI_Sentry.c:37-57`
pub fn NPC_Sentry_Precache(ctx: &mut GameContext) {
    crate::g_utils::G_SoundIndex(cstr("sound/chars/sentry/misc/sentry_explo").as_ptr());
    crate::g_utils::G_SoundIndex(cstr("sound/chars/sentry/misc/sentry_pain").as_ptr());
    crate::g_utils::G_SoundIndex(cstr("sound/chars/sentry/misc/sentry_shield_open").as_ptr());
    crate::g_utils::G_SoundIndex(cstr("sound/chars/sentry/misc/sentry_shield_close").as_ptr());
    crate::g_utils::G_SoundIndex(cstr("sound/chars/sentry/misc/sentry_hover_1_lp").as_ptr());
    crate::g_utils::G_SoundIndex(cstr("sound/chars/sentry/misc/sentry_hover_2_lp").as_ptr());

    for i in 1..4 {
        let talk_idx = i;
        let s = format!("sound/chars/sentry/misc/talk{}", talk_idx);
        crate::g_utils::G_SoundIndex(cstr(&s).as_ptr());
    }

    crate::g_utils::G_EffectIndex(cstr("bryar/muzzle_flash").as_ptr());
    crate::g_utils::G_EffectIndex(cstr("env/med_explode").as_ptr());

    if let Some(item) = unsafe { crate::bg_misc::BG_FindItemForAmmo(AMMO_BLASTER).as_mut() } {
        crate::g_items::RegisterItem(ctx, item);
    }
}

/// `sentry_use` — Entrypoint when sentry gun is activated via trigger.
///
/// Source: `oracle/codemp/game/NPC_AI_Sentry.c:64-72`
pub fn sentry_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    crate::NPC_utils::G_ActivateBehavior(ctx, Some(self_), bSet_t::BSET_USE as c_int);

    ctx.world.entity_mut(self_).flags &= !FL_SHIELDED;
    crate::npc_c::NPC_SetAnim(
        ctx,
        self_,
        SETANIM_BOTH,
        BOTH_POWERUP1 as c_int,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
    );
    // FLAG: gNPC_t (NPCInfo) has no accessor; deref stays raw (recipe 2c).
    let npc_info = ctx.world.entity(self_).NPC;
    unsafe {
        (*npc_info).localState = LSTATE_ACTIVE;
    }
}

/// `NPC_Sentry_Pain` — Sentry pain behavior (hit by damage).
///
/// Source: `oracle/codemp/game/NPC_AI_Sentry.c:79-105`
pub fn NPC_Sentry_Pain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    let mod_ = ctx.world.globals.gPainMOD;

    crate::NPC_reactions::NPC_Pain(ctx, self_, attacker, damage);

    if mod_ == MOD_DEMP2 as c_int || mod_ == MOD_DEMP2_ALT as c_int {
        // FLAG: gNPC_t (NPCInfo) has no accessor; deref stays raw (recipe 2c).
        let npc_info = ctx.world.entity(self_).NPC;
        unsafe {
            (*npc_info).burstCount = 0;
        }
        let atk_delay = ctx.world.bg_state.rng.Q_irand(9000, 12000);
        crate::g_timer::TIMER_Set(ctx, Some(self_), cstr("attackDelay").as_ptr(), atk_delay);
        ctx.world.entity_mut(self_).flags |= FL_SHIELDED;
        crate::npc_c::NPC_SetAnim(
            ctx,
            self_,
            SETANIM_BOTH,
            BOTH_FLY_SHIELDED as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        crate::g_utils::G_Sound(
            ctx,
            Some(self_),
            CHAN_AUTO,
            crate::g_utils::G_SoundIndex(cstr("sound/chars/sentry/misc/sentry_pain").as_ptr()),
        );

        unsafe {
            (*npc_info).localState = LSTATE_ACTIVE;
        }
    }
}

/// `Sentry_Fire` — Fire a bryar projectile from one of the muzzle bolts.
///
/// Source: `oracle/codemp/game/NPC_AI_Sentry.c:112-203`
pub fn Sentry_Fire(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(NPC).unwrap();

    let mut muzzle: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut vright: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };

    ctx.world.entity_mut(npc_id).flags &= !FL_SHIELDED;

    unsafe {
        if (*NPCInfo).localState == LSTATE_POWERING_UP {
            if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(NPC), cstr("powerup").as_ptr()) != 0
            {
                (*NPCInfo).localState = LSTATE_ATTACKING;
                crate::npc_c::NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_BOTH,
                    BOTH_ATTACK1 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
            } else {
                return;
            }
        } else if (*NPCInfo).localState == LSTATE_ACTIVE {
            (*NPCInfo).localState = LSTATE_POWERING_UP;

            crate::g_utils::G_Sound(
                ctx,
                Some(npc_id),
                CHAN_AUTO,
                crate::g_utils::G_SoundIndex(
                    cstr("sound/chars/sentry/misc/sentry_shield_open").as_ptr(),
                ),
            );
            crate::npc_c::NPC_SetAnim(
                ctx,
                npc_id,
                SETANIM_BOTH,
                BOTH_POWERUP1 as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            crate::g_timer::TIMER_Set(ctx, Some(npc_id), cstr("powerup").as_ptr(), 250);
            return;
        } else if (*NPCInfo).localState != LSTATE_ATTACKING {
            (*NPCInfo).localState = LSTATE_ACTIVE;
            return;
        }

        // Which muzzle to fire from?
        let which = (*NPCInfo).burstCount % 3;
        let ghoul2 = ctx.world.entity(npc_id).ghoul2;
        let bolt = match which {
            0 => crate::trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                    ghoul2,
                    0,
                    cstr("*flash1"),
                ),
            ),
            1 => crate::trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                    ghoul2,
                    0,
                    cstr("*flash2"),
                ),
            ),
            _ => crate::trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                    ghoul2,
                    0,
                    cstr("*flash03"),
                ),
            ),
        };

        let current_angles = ctx.world.entity(npc_id).r.currentAngles;
        let current_origin = ctx.world.entity(npc_id).r.currentOrigin;
        let model_scale = ctx.world.entity(npc_id).modelScale;
        let level_time = ctx.world.level.time;
        crate::trap::G2API_GetBoltMatrix(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                ghoul2,
                0,
                bolt,
                &mut boltMatrix as *mut mdxaBone_t,
                &current_angles as *const vec3_t,
                &current_origin as *const vec3_t,
                level_time,
                core::ptr::null_mut(),
                &model_scale as *const vec3_t,
            ),
        );

        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut muzzle);

        crate::q_math::AngleVectors(
            ctx.world.entity(npc_id).r.currentAngles,
            Some(&mut forward),
            Some(&mut vright),
            Some(&mut up),
        );

        crate::g_utils::G_PlayEffectID(
            crate::g_utils::G_EffectIndex(cstr("bryar/muzzle_flash").as_ptr()),
            muzzle,
            forward,
        );

        let missile =
            crate::g_missile::CreateMissile(ctx, muzzle, forward, 1600.0, 10000, npc_id, qfalse);
        let missile_id = ctx.entity_id_of(missile).unwrap();

        ctx.world.entity_mut(missile_id).classname = c"bryar_proj".as_ptr().cast_mut();
        ctx.world.entity_mut(missile_id).s.weapon = WP_BRYAR_PISTOL;

        ctx.world.entity_mut(missile_id).dflags = DAMAGE_DEATH_KNOCKBACK;
        ctx.world.entity_mut(missile_id).methodOfDeath = MOD_BRYAR_PISTOL as c_int;
        ctx.world.entity_mut(missile_id).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

        (*NPCInfo).burstCount += 1;
        let level_time = ctx.world.level.time;
        ctx.world.entity_mut(npc_id).attackDebounceTime = level_time + 50;
        ctx.world.entity_mut(missile_id).damage = 5;

        // now scale for difficulty
        if ctx.world.cvars.g_spskill.integer == 0 {
            ctx.world.entity_mut(npc_id).attackDebounceTime += 200;
            ctx.world.entity_mut(missile_id).damage = 1;
        } else if ctx.world.cvars.g_spskill.integer == 1 {
            ctx.world.entity_mut(npc_id).attackDebounceTime += 100;
            ctx.world.entity_mut(missile_id).damage = 3;
        }
    }
}

/// `Sentry_MaintainHeight` — Maintain hover height relative to enemies/goals.
///
/// Source: `oracle/codemp/game/NPC_AI_Sentry.c:210-304`
pub fn Sentry_MaintainHeight(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: NPC carries a BG_Alloc'd pool client (not level.clients); deref raw
    // via the safe entity borrow, per trap 2b.
    let client = ctx.world.entity(npc_id).client;

    unsafe {
        ctx.world.entity_mut(npc_id).s.loopSound = crate::g_utils::G_SoundIndex(
            cstr("sound/chars/sentry/misc/sentry_hover_1_lp").as_ptr(),
        );

        // Update our angles regardless
        crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);

        // If we have an enemy, we should try to hover at about enemy eye level
        if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
            let enemy_top = ctx.world.entity(enemy_id).r.currentOrigin[2]
                + ctx.world.entity(enemy_id).r.maxs[2];
            let npc_z = ctx.world.entity(npc_id).r.currentOrigin[2];
            let mut dif: f32 = enemy_top - npc_z;

            // cap to prevent dramatic height shifts
            if dif.abs() > 8.0 {
                if dif.abs() > SENTRY_HOVER_HEIGHT {
                    dif = if dif < 0.0 { -24.0 } else { 24.0 };
                }

                (*client).ps.velocity[2] = ((*client).ps.velocity[2] + dif) / 2.0;
            }
        } else {
            let goal_id = if (*NPCInfo).goalEntity.is_some() {
                (*NPCInfo).goalEntity
            } else {
                (*NPCInfo).lastGoalEntity
            };

            if let Some(goal_id) = goal_id {
                let goal_z = ctx.world.entity(goal_id).r.currentOrigin[2];
                let npc_z = ctx.world.entity(npc_id).r.currentOrigin[2];
                let dif: f32 = goal_z - npc_z;

                if dif.abs() > SENTRY_HOVER_HEIGHT {
                    ctx.world.globals.ucmd.upmove = if ctx.world.globals.ucmd.upmove < 0 {
                        -4
                    } else {
                        4
                    };
                } else {
                    if (*client).ps.velocity[2] != 0.0 {
                        (*client).ps.velocity[2] *= SENTRY_VELOCITY_DECAY;

                        if (*client).ps.velocity[2].abs() < 2.0 {
                            (*client).ps.velocity[2] = 0.0;
                        }
                    }
                }
            } else if (*client).ps.velocity[2] != 0.0 {
                (*client).ps.velocity[2] *= SENTRY_VELOCITY_DECAY;

                if (*client).ps.velocity[2].abs() < 1.0 {
                    (*client).ps.velocity[2] = 0.0;
                }
            }
        }

        // Apply friction
        if (*client).ps.velocity[0] != 0.0 {
            (*client).ps.velocity[0] *= SENTRY_VELOCITY_DECAY;

            if (*client).ps.velocity[0].abs() < 1.0 {
                (*client).ps.velocity[0] = 0.0;
            }
        }

        if (*client).ps.velocity[1] != 0.0 {
            (*client).ps.velocity[1] *= SENTRY_VELOCITY_DECAY;

            if (*client).ps.velocity[1].abs() < 1.0 {
                (*client).ps.velocity[1] = 0.0;
            }
        }

        crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);
    }
}

/// `Sentry_Idle` — Idle behavior; sleeping/waking up states.
///
/// Source: `oracle/codemp/game/NPC_AI_Sentry.c:311-331`
pub fn Sentry_Idle(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(NPC).unwrap();

    Sentry_MaintainHeight(ctx);

    unsafe {
        // Is he waking up?
        if (*NPCInfo).localState == LSTATE_WAKEUP {
            // FLAG: pool client, deref raw via safe entity borrow (trap 2b).
            let client = ctx.world.entity(npc_id).client;
            if (*client).ps.torsoTimer <= 0 {
                (*NPCInfo).scriptFlags |= SCF_LOOK_FOR_ENEMIES;
                (*NPCInfo).burstCount = 0;
            }
        } else {
            crate::npc_c::NPC_SetAnim(
                ctx,
                npc_id,
                SETANIM_BOTH,
                BOTH_SLEEP1 as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            ctx.world.entity_mut(npc_id).flags |= FL_SHIELDED;

            crate::NPC_AI_Default::NPC_BSIdle(ctx);
        }
    }
}

/// `Sentry_Strafe` — Perform a strafe maneuver.
///
/// Source: `oracle/codemp/game/NPC_AI_Sentry.c:338-365`
pub fn Sentry_Strafe(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: pool client, deref raw via safe entity borrow (trap 2b).
    let client = ctx.world.entity(npc_id).client;

    let mut right: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut tr: trace_t = unsafe { core::mem::zeroed() };

    unsafe {
        crate::q_math::AngleVectors((*client).renderInfo.eyeAngles, None, Some(&mut right), None);

        // Pick a random strafe direction, then check to see if doing a strafe would be
        // reasonable valid
        let dir = if (ctx.world.bg_state.rng.rand() & 1) != 0 {
            -1
        } else {
            1
        };
        let current_origin = ctx.world.entity(npc_id).r.currentOrigin;
        crate::q_math::_VectorMA(
            current_origin,
            (SENTRY_STRAFE_DIS * dir as f32),
            right,
            &mut end,
        );

        let s_number = ctx.world.entity(npc_id).s.number;
        crate::trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut tr as *mut trace_t,
                &current_origin as *const vec3_t,
                core::ptr::null::<vec3_t>(),
                core::ptr::null::<vec3_t>(),
                &end as *const vec3_t,
                s_number,
                MASK_SOLID,
            ),
        );

        // Close enough
        if tr.fraction > 0.9f32 {
            crate::q_math::_VectorMA(
                (*client).ps.velocity,
                (SENTRY_STRAFE_VEL * dir as f32),
                right,
                &mut (*client).ps.velocity,
            );

            // Add a slight upward push
            (*client).ps.velocity[2] += SENTRY_UPWARD_PUSH;

            // Set the strafe start time so we can do a controlled roll
            let level_time = ctx.world.level.time;
            let roll = (ctx.world.bg_state.rng.random() * 500.0) as c_int;
            (*NPCInfo).standTime = level_time + 3000 + roll;
        }
    }
}

/// `Sentry_Hunt` — Hunt the enemy, either strafing or chasing.
///
/// Source: `oracle/codemp/game/NPC_AI_Sentry.c:372-411`
pub fn Sentry_Hunt(ctx: &mut GameContext, visible: qboolean, advance: qboolean) {
    let NPC = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(NPC).unwrap();
    // FLAG: pool client, deref raw via safe entity borrow (trap 2b).
    let client = ctx.world.entity(npc_id).client;

    let mut forward: vec3_t = [0.0; 3];
    let mut distance: f32 = 0.0;

    unsafe {
        // If we're not supposed to stand still, pursue the player
        if (*NPCInfo).standTime < ctx.world.level.time {
            // Only strafe when we can see the player
            if visible != qfalse {
                Sentry_Strafe(ctx);
                return;
            }
        }

        // If we don't want to advance, stop here
        if advance == qfalse && visible != qfalse {
            return;
        }

        // Only try and navigate if the player is visible
        if visible == qfalse {
            // Move towards our goal
            if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
                (*NPCInfo).goalEntity = Some(enemy_id);
            }
            (*NPCInfo).goalRadius = 12;

            // Get our direction from the navigator if we can't see our target
            if crate::NPC_move::NPC_GetMoveDirection(ctx, &mut forward, &mut distance) == qfalse {
                return;
            }
        } else {
            if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
                let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
                let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
                crate::q_math::_VectorSubtract(enemy_origin, npc_origin, &mut forward);
                distance = crate::q_math::VectorNormalize(&mut forward);
            }
        }

        let speed = SENTRY_FORWARD_BASE_SPEED
            + (SENTRY_FORWARD_MULTIPLIER * ctx.world.cvars.g_spskill.integer as f32);
        crate::q_math::_VectorMA(
            (*client).ps.velocity,
            speed,
            forward,
            &mut (*client).ps.velocity,
        );
    }
}

/// `Sentry_RangedAttack` — Ranged attack behavior.
///
/// Source: `oracle/codemp/game/NPC_AI_Sentry.c:418-448`
pub fn Sentry_RangedAttack(ctx: &mut GameContext, visible: qboolean, advance: qboolean) {
    let NPC = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(NPC).unwrap();

    unsafe {
        if crate::g_timer::TIMER_Done(ctx, Some(npc_id), cstr("attackDelay").as_ptr()) != qfalse
            && ctx.world.entity(npc_id).attackDebounceTime < ctx.world.level.time
            && visible != qfalse
        {
            if (*NPCInfo).burstCount > 6 {
                if ctx.world.entity(npc_id).fly_sound_debounce_time == 0 {
                    // delay closing down to give the player an opening
                    let level_time = ctx.world.level.time;
                    let delay = ctx.world.bg_state.rng.Q_irand(500, 2000);
                    ctx.world.entity_mut(npc_id).fly_sound_debounce_time = level_time + delay;
                } else if ctx.world.entity(npc_id).fly_sound_debounce_time < ctx.world.level.time {
                    (*NPCInfo).localState = LSTATE_ACTIVE;
                    ctx.world.entity_mut(npc_id).fly_sound_debounce_time = 0;
                    (*NPCInfo).burstCount = 0;
                    let atk_delay = ctx.world.bg_state.rng.Q_irand(2000, 3500);
                    crate::g_timer::TIMER_Set(
                        ctx,
                        Some(npc_id),
                        cstr("attackDelay").as_ptr(),
                        atk_delay,
                    );
                    ctx.world.entity_mut(npc_id).flags |= FL_SHIELDED;
                    crate::npc_c::NPC_SetAnim(
                        ctx,
                        npc_id,
                        SETANIM_BOTH,
                        BOTH_FLY_SHIELDED as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    crate::g_utils::G_SoundOnEnt(
                        ctx,
                        npc_id,
                        CHAN_AUTO,
                        cstr("sound/chars/sentry/misc/sentry_shield_close").as_ptr(),
                    );
                }
            } else {
                Sentry_Fire(ctx);
            }
        }

        if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
            Sentry_Hunt(ctx, visible, advance);
        }
    }
}

/// `Sentry_AttackDecision` — Decide how to attack the enemy.
///
/// Source: `oracle/codemp/game/NPC_AI_Sentry.c:455-510`
pub fn Sentry_AttackDecision(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(NPC).unwrap();

    let mut distance: f32 = 0.0;
    let visible: qboolean;
    let advance: qboolean;

    // Always keep a good height off the ground
    Sentry_MaintainHeight(ctx);

    ctx.world.entity_mut(npc_id).s.loopSound =
        crate::g_utils::G_SoundIndex(cstr("sound/chars/sentry/misc/sentry_hover_2_lp").as_ptr());

    unsafe {
        // randomly talk
        if crate::g_timer::TIMER_Done(ctx, Some(npc_id), cstr("patrolNoise").as_ptr()) != qfalse {
            if crate::g_timer::TIMER_Done(ctx, Some(npc_id), cstr("angerNoise").as_ptr()) != qfalse
            {
                let talk_idx = ctx.world.bg_state.rng.Q_irand(1, 3);
                let s = format!("sound/chars/sentry/misc/talk{}", talk_idx);
                crate::g_utils::G_SoundOnEnt(ctx, npc_id, CHAN_AUTO, cstr(&s).as_ptr());

                let patrol_delay = ctx.world.bg_state.rng.Q_irand(4000, 10000);
                crate::g_timer::TIMER_Set(
                    ctx,
                    Some(npc_id),
                    cstr("patrolNoise").as_ptr(),
                    patrol_delay,
                );
            }
        }

        // He's dead.
        if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
            let enemy_health = ctx.world.entity(enemy_id).health;
            if enemy_health < 1 {
                ctx.world.entity_mut(npc_id).enemy = None;
                Sentry_Idle(ctx);
                return;
            }
        }

        // If we don't have an enemy, just idle
        if crate::NPC_utils::NPC_CheckEnemyExt(ctx, qfalse) == qfalse {
            Sentry_Idle(ctx);
            return;
        }

        // Rate our distance to the target and visibilty
        if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
            let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
            let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
            distance =
                crate::q_math::DistanceHorizontalSquared(npc_origin, enemy_origin) as c_int as f32;
            visible = crate::NPC_utils::NPC_ClearLOS4(ctx, Some(enemy_id));
            advance = if distance > MIN_DISTANCE_SQR {
                qtrue
            } else {
                qfalse
            };
        } else {
            visible = qfalse;
            advance = qfalse;
        }

        // If we cannot see our target, move to see it
        if visible == qfalse {
            if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
                Sentry_Hunt(ctx, visible, advance);
                return;
            }
        }

        crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);

        Sentry_RangedAttack(ctx, visible, advance);
    }
}

/// `NPC_Sentry_Patrol` — Patrol behavior when no enemy.
///
/// Source: `oracle/codemp/game/NPC_AI_Sentry.c:519-550`
pub fn NPC_Sentry_Patrol(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(NPC).unwrap();

    Sentry_MaintainHeight(ctx);

    // If we have somewhere to go, then do that
    if ctx.world.entity(npc_id).enemy.is_none() {
        if crate::NPC_AI_Stormtrooper::NPC_CheckPlayerTeamStealth(ctx) != qfalse {
            crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }

        if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
            // start loop sound once we move
            ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
        }

        // randomly talk
        if crate::g_timer::TIMER_Done(ctx, Some(npc_id), cstr("patrolNoise").as_ptr()) != qfalse {
            let talk_idx = ctx.world.bg_state.rng.Q_irand(1, 3);
            let s = format!("sound/chars/sentry/misc/talk{}", talk_idx);
            crate::g_utils::G_SoundOnEnt(ctx, npc_id, CHAN_AUTO, cstr(&s).as_ptr());

            let patrol_delay = ctx.world.bg_state.rng.Q_irand(2000, 4000);
            crate::g_timer::TIMER_Set(
                ctx,
                Some(npc_id),
                cstr("patrolNoise").as_ptr(),
                patrol_delay,
            );
        }
    }

    crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
}

/// `NPC_BSSentry_Default` — Main behavior state for sentry gun.
///
/// Source: `oracle/codemp/game/NPC_AI_Sentry.c:557-577`
pub fn NPC_BSSentry_Default(ctx: &mut GameContext) {
    let NPC = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let NPCInfo = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(NPC).unwrap();

    if !ctx.world.entity(npc_id).targetname.is_null() {
        ctx.world.entity_mut(npc_id).use_ = Some(crate::ent_fn_enums::EntUse::sentry_use).into();
    }

    unsafe {
        if ctx.world.entity(npc_id).enemy.is_some() && (*NPCInfo).localState != LSTATE_WAKEUP {
            // Don't attack if waking up or if no enemy
            Sentry_AttackDecision(ctx);
        } else if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            NPC_Sentry_Patrol(ctx);
        } else {
            Sentry_Idle(ctx);
        }
    }
}
