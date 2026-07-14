// PORT-COMPLETE: NPC_AI_Interrogator.c 10/10
//! Faithful port of `oracle/codemp/game/NPC_AI_Interrogator.c` (jampgame mega-pass).
//!
//! Interrogator droid NPC AI behavior: idle, patrol, hunt, strafe, melee attack.
//!
//! All functions are now filled per pass-3 rulings: ai-context globals (NPC, NPCInfo, ucmd, level)
//! are threaded via GameContext; stored enemy/goalEntity fields use Option<EntityId>; traps via
//! ctx.engine; RNG via BgState; vec3 helpers use reshaped q_math signatures.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_utils::{G_EffectIndex, G_SoundIndex};
use crate::prelude::*;
use crate::trap;

// EntityId seam helper: resolve `Option<EntityId>` back to the raw pointer the
// verbatim body still expects (`None` -> null), per the `NPC_AI_Stormtrooper.rs`
// precedent.
#[inline]
unsafe fn ent_resolve_opt(ctx: &mut GameContext, id: Option<EntityId>) -> *mut gentity_t {
    match id {
        Some(i) => &mut ctx.world.g_entities[i.index()] as *mut gentity_t,
        None => core::ptr::null_mut(),
    }
}

/// Local state enums for Interrogator blade movement.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:8-13`
const LSTATE_BLADESTOP: c_int = 0;
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:11`
pub const LSTATE_BLADEUP: c_int = 1;
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:12`
pub const LSTATE_BLADEDOWN: c_int = 2;

/// Velocity decay factor for Interrogator hovering.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:129`
const VELOCITY_DECAY: f32 = 0.85;

/// Upward push for Interrogator during strafe.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:130`
const HUNTER_UPWARD_PUSH: c_int = 2;

/// Strafe velocity for Interrogator movement.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:231`
const HUNTER_STRAFE_VEL: c_int = 32;

/// Distance for Interrogator strafe.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:232`
const HUNTER_STRAFE_DIS: c_int = 200;

/// Forward base speed for Interrogator.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:287`
const HUNTER_FORWARD_BASE_SPEED: c_int = 10;

/// Forward speed multiplier for Interrogator.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:288`
const HUNTER_FORWARD_MULTIPLIER: c_int = 2;

/// Minimum distance for Interrogator melee attack.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:338`
const MIN_DISTANCE: c_int = 64;

/// Raven `NPC_Interrogator_Precache`.
///
/// Precache sounds and effects for the Interrogator NPC.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:20-28`
pub fn NPC_Interrogator_Precache(ctx: &mut GameContext, self_: Option<EntityId>) {
    // STAGE-1: EntityId param (unused by the body; caller may pass null/`None`).
    G_SoundIndex(c"sound/chars/interrogator/misc/torture_droid_lp".as_ptr() as *const c_char);
    G_SoundIndex(c"sound/chars/mark1/misc/anger.wav".as_ptr() as *const c_char);
    G_SoundIndex(c"sound/chars/probe/misc/talk".as_ptr() as *const c_char);
    G_SoundIndex(c"sound/chars/interrogator/misc/torture_droid_inject".as_ptr() as *const c_char);
    G_SoundIndex(c"sound/chars/interrogator/misc/int_droid_explo".as_ptr() as *const c_char);
    G_EffectIndex(c"explosions/droidexplosion1".as_ptr() as *const c_char);
}

/// Raven `Interrogator_die`.
///
/// Death behavior for Interrogator NPC. Sets velocity and clears flying flag.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:34-57`
pub fn Interrogator_die(
    ctx: &mut GameContext,
    self_: Option<EntityId>,
    inflictor: Option<EntityId>,
    attacker: Option<EntityId>,
    damage: c_int,
    r#mod: c_int,
    dFlags: c_int,
    hitLoc: c_int,
) {
    // STAGE-1: EntityId params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = unsafe { ent_resolve_opt(ctx, self_) };
    let inflictor: *mut gentity_t = unsafe { ent_resolve_opt(ctx, inflictor) };
    let attacker: *mut gentity_t = unsafe { ent_resolve_opt(ctx, attacker) };
    unsafe {
        // SAFETY: self_ accessed through game context.
        self_.as_mut().map(|ent| {
            if !ent.client.is_null() {
                let client = &mut *(ent.client as *mut gclient_t);
                client.ps.velocity[2] = -100.0;

                // Clear flying flag and set random horizontal velocity
                client.ps.eFlags2 &= !(crate::prelude::EF2_FLYING as c_int);
                // Raven passes the range reversed — `Q_irand(-10, -20)` — and irand's
                // arithmetic gives different values for a reversed range; keep it verbatim.
                // Source: `oracle/codemp/game/NPC_AI_Interrogator.c:49-50`
                client.ps.velocity[0] = ctx.world.bg_state.rng.Q_irand(-10, -20) as f32;
                client.ps.velocity[1] = ctx.world.bg_state.rng.Q_irand(-10, -20) as f32;
                client.ps.velocity[2] = -100.0;
            }
        });
    }
}

/// Raven `Interrogator_PartsMove`.
///
/// Move the syringe, scalpel, and claw parts of the Interrogator.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:64-127`
pub fn Interrogator_PartsMove(ctx: &mut GameContext) {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;

        // Syringe
        if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"syringeDelay".as_ptr()) != 0 {
            (*npc).pos1[1] = crate::q_math::AngleNormalize360((*npc).pos1[1]);

            if ((*npc).pos1[1] < 60.0) || ((*npc).pos1[1] > 300.0) {
                (*npc).pos1[1] += ctx.world.bg_state.rng.Q_irand(-20, 20) as f32;
            } else if (*npc).pos1[1] > 180.0 {
                (*npc).pos1[1] = ctx.world.bg_state.rng.Q_irand(300, 360) as f32;
            } else {
                (*npc).pos1[1] = ctx.world.bg_state.rng.Q_irand(0, 60) as f32;
            }

            crate::NPC_utils::NPC_SetBoneAngles(
                ctx,
                ctx.entity_id_of(npc).unwrap(),
                c"left_arm".as_ptr() as *mut c_char,
                (*npc).pos1,
            );

            let npc_id = ctx.entity_id_of(npc);
            let delay = ctx.world.bg_state.rng.Q_irand(100, 1000);
            crate::g_timer::TIMER_Set(ctx, npc_id, c"syringeDelay".as_ptr(), delay);
        }

        // Scalpel
        if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"scalpelDelay".as_ptr()) != 0 {
            // Change pitch
            if (*npc_info).localState == LSTATE_BLADEDOWN {
                // Blade is moving down
                (*npc).pos2[0] -= 30.0;
                if (*npc).pos2[0] < 180.0 {
                    (*npc).pos2[0] = 180.0;
                    (*npc_info).localState = LSTATE_BLADEUP; // Make it move up
                }
            } else {
                // Blade is coming back up
                (*npc).pos2[0] += 30.0;
                if (*npc).pos2[0] >= 360.0 {
                    (*npc).pos2[0] = 360.0;
                    let npc_id = ctx.entity_id_of(npc);
                    let delay = ctx.world.bg_state.rng.Q_irand(100, 1000);
                    (*npc_info).localState = LSTATE_BLADEDOWN; // Make it move down
                    crate::g_timer::TIMER_Set(ctx, npc_id, c"scalpelDelay".as_ptr(), delay);
                }
            }

            (*npc).pos2[0] = crate::q_math::AngleNormalize360((*npc).pos2[0]);

            crate::NPC_utils::NPC_SetBoneAngles(
                ctx,
                ctx.entity_id_of(npc).unwrap(),
                c"right_arm".as_ptr() as *mut c_char,
                (*npc).pos2,
            );
        }

        // Claw
        (*npc).pos3[1] += ctx.world.bg_state.rng.Q_irand(10, 30) as f32;
        (*npc).pos3[1] = crate::q_math::AngleNormalize360((*npc).pos3[1]);
        crate::NPC_utils::NPC_SetBoneAngles(
            ctx,
            ctx.entity_id_of(npc).unwrap(),
            c"claw".as_ptr() as *mut c_char,
            (*npc).pos3,
        );
    }
}

/// Raven `Interrogator_MaintainHeight`.
///
/// Maintain hover height relative to enemy or goal.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:137-229`
pub fn Interrogator_MaintainHeight(ctx: &mut GameContext) {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;
        let base = ctx.world.g_entities.as_mut_ptr();

        (*npc).s.loopSound = crate::g_utils::G_SoundIndex(
            c"sound/chars/interrogator/misc/torture_droid_lp".as_ptr(),
        );

        // Update our angles regardless
        crate::NPC_utils::NPC_UpdateAngles(ctx, 1, 1);

        let mut dif: f32;

        // If we have an enemy, we should try to hover at about enemy eye level
        if (*npc).enemy.is_some() {
            let enemy_ptr = match (*npc).enemy {
                Some(id) => base.add(id.index()),
                None => core::ptr::null_mut(),
            };

            if !enemy_ptr.is_null() {
                // Find the height difference
                dif = ((*enemy_ptr).r.currentOrigin[2] + (*enemy_ptr).r.maxs[2])
                    - (*npc).r.currentOrigin[2];

                // cap to prevent dramatic height shifts
                if dif.abs() > 2.0 {
                    if dif.abs() > 16.0 {
                        dif = if dif < 0.0 { -16.0 } else { 16.0 };
                    }

                    (*((*npc).client as *mut gclient_t)).ps.velocity[2] =
                        ((*((*npc).client as *mut gclient_t)).ps.velocity[2] + dif) / 2.0;
                }
            }
        } else {
            let mut goal: *mut gentity_t = core::ptr::null_mut();

            if (*npc_info).goalEntity.is_some() {
                // Is there a goal?
                goal = match (*npc_info).goalEntity {
                    Some(id) => base.add(id.index()),
                    None => core::ptr::null_mut(),
                };
            } else {
                goal = match (*npc_info).lastGoalEntity {
                    Some(id) => base.add(id.index()),
                    None => core::ptr::null_mut(),
                };
            }

            if !goal.is_null() {
                dif = (*goal).r.currentOrigin[2] - (*npc).r.currentOrigin[2];

                if dif.abs() > 24.0 {
                    ctx.world.globals.ucmd.upmove = if ctx.world.globals.ucmd.upmove < 0 {
                        -4
                    } else {
                        4
                    };
                } else {
                    if (*((*npc).client as *mut gclient_t)).ps.velocity[2] != 0.0 {
                        (*((*npc).client as *mut gclient_t)).ps.velocity[2] *= VELOCITY_DECAY;

                        if (*((*npc).client as *mut gclient_t)).ps.velocity[2].abs() < 2.0 {
                            (*((*npc).client as *mut gclient_t)).ps.velocity[2] = 0.0;
                        }
                    }
                }
            }
            // Apply friction
            else if (*((*npc).client as *mut gclient_t)).ps.velocity[2] != 0.0 {
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

/// Raven `Interrogator_Strafe`.
///
/// Perform a strafe movement away from the target.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:238-279`
pub fn Interrogator_Strafe(ctx: &mut GameContext) {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;
        let base = ctx.world.g_entities.as_mut_ptr();

        let mut end: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        let mut tr: trace_t = core::mem::zeroed();

        crate::q_math::AngleVectors(
            (*((*npc).client as *mut gclient_t)).renderInfo.eyeAngles,
            None,
            Some(&mut right),
            None,
        );

        // Pick a random strafe direction, then check to see if doing a strafe would be
        // reasonable valid
        let dir = if (ctx.world.bg_state.rng.rand() & 1) != 0 {
            -1
        } else {
            1
        };
        crate::q_math::_VectorMA(
            (*npc).r.currentOrigin,
            (HUNTER_STRAFE_DIS * dir) as f32,
            right,
            &mut end,
        );

        trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
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
        if tr.fraction > 0.9f32 {
            crate::q_math::_VectorMA(
                (*((*npc).client as *mut gclient_t)).ps.velocity,
                (HUNTER_STRAFE_VEL * dir) as f32,
                right,
                &mut (*((*npc).client as *mut gclient_t)).ps.velocity,
            );

            // Add a slight upward push
            if (*npc).enemy.is_some() {
                let enemy_ptr = match (*npc).enemy {
                    Some(id) => base.add(id.index()),
                    None => core::ptr::null_mut(),
                };

                if !enemy_ptr.is_null() {
                    // Find the height difference
                    let mut dif =
                        ((*enemy_ptr).r.currentOrigin[2] + 32.0) - (*npc).r.currentOrigin[2];

                    // cap to prevent dramatic height shifts
                    if dif.abs() > 8.0 {
                        dif = if dif < 0.0 {
                            -(HUNTER_UPWARD_PUSH as f32)
                        } else {
                            HUNTER_UPWARD_PUSH as f32
                        };
                    }

                    (*((*npc).client as *mut gclient_t)).ps.velocity[2] += dif;
                }
            }

            // Set the strafe start time
            (*npc_info).standTime =
                ctx.world.level.time + 3000 + (ctx.world.bg_state.rng.random() * 500.0) as c_int;
        }
    }
}

/// Raven `Interrogator_Hunt`.
///
/// Hunt the enemy, using strafe and movement.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:290-336`
pub fn Interrogator_Hunt(ctx: &mut GameContext, visible: qboolean, advance: qboolean) {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;
        let base = ctx.world.g_entities.as_mut_ptr();

        Interrogator_PartsMove(ctx);

        crate::NPC_utils::NPC_FaceEnemy(ctx, 0);

        // If we're not supposed to stand still, pursue the player
        if (*npc_info).standTime < ctx.world.level.time {
            // Only strafe when we can see the player
            if visible != 0 {
                Interrogator_Strafe(ctx);
                if (*npc_info).standTime > ctx.world.level.time {
                    // successfully strafed
                    return;
                }
            }
        }

        // If we don't want to advance, stop here
        if advance == 0 {
            return;
        }

        let mut forward: vec3_t = [0.0; 3];
        let mut distance: f32 = 0.0;

        // Only try and navigate if the player is visible
        if visible == 0 {
            // Move towards our goal
            (*npc_info).goalEntity = match (*npc).enemy {
                Some(id) => Some(id),
                None => None,
            };
            (*npc_info).goalRadius = 12;

            // Get our direction from the navigator if we can't see our target
            if crate::NPC_move::NPC_GetMoveDirection(ctx, &mut forward, &mut distance as *mut f32)
                == 0
            {
                return;
            }
        } else {
            crate::q_math::_VectorSubtract(
                match (*npc).enemy {
                    Some(id) => (*base.add(id.index())).r.currentOrigin,
                    None => (*npc).r.currentOrigin,
                },
                (*npc).r.currentOrigin,
                &mut forward,
            );
            distance = crate::q_math::VectorNormalize(&mut forward);
        }

        let speed = HUNTER_FORWARD_BASE_SPEED as f32
            + (HUNTER_FORWARD_MULTIPLIER as f32) * ctx.world.cvars.g_spskill.integer as f32;
        crate::q_math::_VectorMA(
            (*((*npc).client as *mut gclient_t)).ps.velocity,
            speed,
            forward,
            &mut (*((*npc).client as *mut gclient_t)).ps.velocity,
        );
    }
}

/// Raven `Interrogator_Melee`.
///
/// Perform melee attack if close enough and within height range.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:345-374`
pub fn Interrogator_Melee(ctx: &mut GameContext, visible: qboolean, advance: qboolean) {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;
        let base = ctx.world.g_entities.as_mut_ptr();

        if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"attackDelay".as_ptr()) != 0 {
            let enemy_ptr = match (*npc).enemy {
                Some(id) => base.add(id.index()),
                None => core::ptr::null_mut(),
            };

            if !enemy_ptr.is_null() {
                // Make sure that we are within the height range before we allow any damage to happen
                if (*npc).r.currentOrigin[2]
                    >= (*enemy_ptr).r.currentOrigin[2] + (*enemy_ptr).r.mins[2]
                    && (*npc).r.currentOrigin[2] + (*npc).r.mins[2] + 8.0
                        < (*enemy_ptr).r.currentOrigin[2] + (*enemy_ptr).r.maxs[2]
                {
                    let npc_id = ctx.entity_id_of(npc);
                    let delay = ctx.world.bg_state.rng.Q_irand(500, 3000);
                    crate::g_timer::TIMER_Set(ctx, npc_id, c"attackDelay".as_ptr(), delay);
                    crate::g_combat::G_Damage(
                        ctx,
                        ctx.entity_id_of(enemy_ptr),
                        ctx.entity_id_of(npc),
                        ctx.entity_id_of(npc),
                        None,
                        [0.0f32; 3],
                        2,
                        DAMAGE_NO_KNOCKBACK,
                        MOD_MELEE as c_int,
                    );

                    crate::g_utils::G_Sound(
                        ctx,
                        ctx.entity_id_of(npc),
                        CHAN_AUTO,
                        crate::g_utils::G_SoundIndex(
                            c"sound/chars/interrogator/misc/torture_droid_inject.mp3".as_ptr(),
                        ),
                    );
                }
            }
        }

        if (*npc_info).scriptFlags & SCF_CHASE_ENEMIES != 0 {
            Interrogator_Hunt(ctx, visible, advance);
        }
    }
}

/// Raven `Interrogator_Attack`.
///
/// Main attack function - handles distance, visibility, and attack selection.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:381-428`
pub fn Interrogator_Attack(ctx: &mut GameContext) {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;

        // Always keep a good height off the ground
        Interrogator_MaintainHeight(ctx);

        // randomly talk
        if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"patrolNoise".as_ptr()) != 0 {
            if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"angerNoise".as_ptr()) != 0 {
                // Raven: `va("sound/chars/probe/misc/talk.wav", Q_irand(1, 3))` — the
                // format string has no specifier, so the value is discarded, but the
                // Q_irand still advances the holdrand stream; keep the draw.
                // Source: `oracle/codemp/game/NPC_AI_Interrogator.c:395`
                let _ = ctx.world.bg_state.rng.Q_irand(1, 3);
                crate::g_utils::G_SoundOnEnt(
                    ctx,
                    ctx.entity_id_of(npc).unwrap(),
                    CHAN_AUTO,
                    c"sound/chars/probe/misc/talk.wav".as_ptr(),
                );

                let npc_id = ctx.entity_id_of(npc);
                let delay = ctx.world.bg_state.rng.Q_irand(4000, 10000);
                crate::g_timer::TIMER_Set(ctx, npc_id, c"patrolNoise".as_ptr(), delay);
            }
        }

        // If we don't have an enemy, just idle
        if crate::NPC_utils::NPC_CheckEnemyExt(ctx, 0) == 0 {
            Interrogator_Idle(ctx);
            return;
        }

        // Rate our distance to the target, and our visibility
        let distance = (crate::q_math::DistanceHorizontalSquared(
            (*npc).r.currentOrigin,
            match (*npc).enemy {
                Some(id) => {
                    let base = ctx.world.g_entities.as_mut_ptr();
                    (*base.add(id.index())).r.currentOrigin
                }
                None => (*npc).r.currentOrigin,
            },
        )) as c_int;

        let visible = crate::NPC_utils::NPC_ClearLOS4(ctx, (*npc).enemy);

        let mut advance = if distance > MIN_DISTANCE * MIN_DISTANCE {
            1
        } else {
            0
        };

        if visible == 0 {
            advance = 1;
        }

        if (*npc_info).scriptFlags & SCF_CHASE_ENEMIES != 0 {
            Interrogator_Hunt(ctx, visible, advance);
        }

        crate::NPC_utils::NPC_FaceEnemy(ctx, 1);

        if advance == 0 {
            Interrogator_Melee(ctx, visible, advance);
        }
    }
}

/// Raven `Interrogator_Idle`.
///
/// Idle behavior - check for stealth enemies and maintain height.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:435-447`
pub fn Interrogator_Idle(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;

    if crate::NPC_AI_Stormtrooper::NPC_CheckPlayerTeamStealth(ctx) != 0 {
        crate::g_utils::G_SoundOnEnt(
            ctx,
            ctx.entity_id_of(npc).unwrap(),
            CHAN_AUTO,
            c"sound/chars/mark1/misc/anger.wav".as_ptr(),
        );
        crate::NPC_utils::NPC_UpdateAngles(ctx, 1, 1);
        return;
    }

    Interrogator_MaintainHeight(ctx);

    crate::NPC_AI_Default::NPC_BSIdle(ctx);
}

/// Raven `NPC_BSInterrogator_Default`.
///
/// Default behavior state selector - attacks if enemy present, otherwise idles.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:454-467`
pub fn NPC_BSInterrogator_Default(ctx: &mut GameContext) {
    unsafe {
        let npc = ctx.world.globals.NPC;

        if (*npc).enemy.is_some() {
            Interrogator_Attack(ctx);
        } else {
            Interrogator_Idle(ctx);
        }
    }
}
