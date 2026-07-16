// PORT-COMPLETE: NPC_AI_Interrogator.c 10/10
//! Faithful port of `oracle/codemp/game/NPC_AI_Interrogator.c` (jampgame mega-pass).
//!
//! Interrogator droid NPC AI behavior: idle, patrol, hunt, strafe, melee attack.
//!
//! All functions are now filled per pass-3 rulings: ai-context globals (NPC, NPCInfo, ucmd, level)
//! are threaded via GameContext; stored enemy/goalEntity fields use Option<EntityId>; traps via
//! ctx.engine; RNG via BgState; vec3 helpers use reshaped q_math signatures.
//!
//! Safe-state 2c: the NPC entity half is converted to `ctx.world.entity(npc_id)` /
//! `entity_mut` accessor borrows. Two irreducible raw-deref regimes remain (FLAGged
//! inline, task #7): `NPCInfo` is a `*mut gNPC_t` with no safe accessor, and `NPC`s
//! carry a `BG_Alloc`'d pool `gclient_t` (`gClPtrs`) that is not a `level.clients`
//! slot, so `NPC->client` is dereffed raw exactly as Raven does.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_combat::G_Damage;
use crate::g_timer::{TIMER_Done, TIMER_Set};
use crate::g_utils::{G_EffectIndex, G_Sound, G_SoundIndex, G_SoundOnEnt};
use crate::prelude::*;
use crate::q_math::{
    _VectorMA, _VectorSubtract, AngleNormalize360, AngleVectors, DistanceHorizontalSquared,
    VectorNormalize,
};
use crate::trap;
use crate::NPC_AI_Default::NPC_BSIdle;
use crate::NPC_AI_Stormtrooper::NPC_CheckPlayerTeamStealth;
use crate::NPC_move::NPC_GetMoveDirection;
use crate::NPC_utils::{
    NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy, NPC_SetBoneAngles, NPC_UpdateAngles,
};
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;

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
    // STAGE-1: EntityId params; only `self_` is read (inflictor/attacker unused, as in Raven).
    let Some(self_id) = self_ else {
        return;
    };
    // FLAG (task #7): NPC pool `gclient_t` (`gClPtrs`, g_utils.c:430) — not a
    // `level.clients` slot; the pointer is read via the safe entity borrow and
    // dereffed raw exactly as Raven does.
    let client = ctx.world.entity(self_id).client;
    if !client.is_null() {
        unsafe {
            let c = &mut *client;
            c.ps.velocity[2] = -100.0;

            // Clear flying flag and set random horizontal velocity
            c.ps.eFlags2 &= !(EF2_FLYING as c_int);
            // Raven passes the range reversed — `Q_irand(-10, -20)` — and irand's
            // arithmetic gives different values for a reversed range; keep it verbatim.
            // Source: `oracle/codemp/game/NPC_AI_Interrogator.c:49-50`
            c.ps.velocity[0] = ctx.world.bg_state.rng.Q_irand(-10, -20) as f32;
            c.ps.velocity[1] = ctx.world.bg_state.rng.Q_irand(-10, -20) as f32;
            c.ps.velocity[2] = -100.0;
        }
    }
}

/// Raven `Interrogator_PartsMove`.
///
/// Move the syringe, scalpel, and claw parts of the Interrogator.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:64-127`
pub fn Interrogator_PartsMove(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let npc_info = ctx.world.globals.NPCInfo;

    // Syringe
    if TIMER_Done(ctx, Some(npc_id), c"syringeDelay".as_ptr()) != 0 {
        let mut v = ctx.world.entity(npc_id).pos1[1];
        v = AngleNormalize360(v);

        if (v < 60.0) || (v > 300.0) {
            v += ctx.world.bg_state.rng.Q_irand(-20, 20) as f32;
        } else if v > 180.0 {
            v = ctx.world.bg_state.rng.Q_irand(300, 360) as f32;
        } else {
            v = ctx.world.bg_state.rng.Q_irand(0, 60) as f32;
        }
        ctx.world.entity_mut(npc_id).pos1[1] = v;

        let pos1 = ctx.world.entity(npc_id).pos1;
        NPC_SetBoneAngles(ctx, npc_id, c"left_arm".as_ptr() as *mut c_char, pos1);

        let delay = ctx.world.bg_state.rng.Q_irand(100, 1000);
        TIMER_Set(ctx, Some(npc_id), c"syringeDelay".as_ptr(), delay);
    }

    // Scalpel
    if TIMER_Done(ctx, Some(npc_id), c"scalpelDelay".as_ptr()) != 0 {
        let mut p2 = ctx.world.entity(npc_id).pos2[0];
        // Change pitch. FLAG (task #7): NPCInfo (gNPC_t) has no safe accessor;
        // localState read/write stays a raw deref.
        if unsafe { (*npc_info).localState } == LSTATE_BLADEDOWN {
            // Blade is moving down
            p2 -= 30.0;
            if p2 < 180.0 {
                p2 = 180.0;
                unsafe {
                    (*npc_info).localState = LSTATE_BLADEUP; // Make it move up
                }
            }
        } else {
            // Blade is coming back up
            p2 += 30.0;
            if p2 >= 360.0 {
                p2 = 360.0;
                unsafe {
                    (*npc_info).localState = LSTATE_BLADEDOWN; // Make it move down
                }
                let delay = ctx.world.bg_state.rng.Q_irand(100, 1000);
                TIMER_Set(ctx, Some(npc_id), c"scalpelDelay".as_ptr(), delay);
            }
        }

        p2 = AngleNormalize360(p2);
        ctx.world.entity_mut(npc_id).pos2[0] = p2;

        let pos2 = ctx.world.entity(npc_id).pos2;
        NPC_SetBoneAngles(ctx, npc_id, c"right_arm".as_ptr() as *mut c_char, pos2);
    }

    // Claw
    let draw = ctx.world.bg_state.rng.Q_irand(10, 30) as f32;
    let mut p3 = ctx.world.entity(npc_id).pos3[1];
    p3 += draw;
    p3 = AngleNormalize360(p3);
    ctx.world.entity_mut(npc_id).pos3[1] = p3;

    let pos3 = ctx.world.entity(npc_id).pos3;
    NPC_SetBoneAngles(ctx, npc_id, c"claw".as_ptr() as *mut c_char, pos3);
}

/// Raven `Interrogator_MaintainHeight`.
///
/// Maintain hover height relative to enemy or goal.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:137-229`
pub fn Interrogator_MaintainHeight(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let npc_info = ctx.world.globals.NPCInfo;

    let loop_sound = G_SoundIndex(c"sound/chars/interrogator/misc/torture_droid_lp".as_ptr());
    ctx.world.entity_mut(npc_id).s.loopSound = loop_sound;

    // Update our angles regardless
    NPC_UpdateAngles(ctx, 1, 1);

    let mut dif: f32;

    // FLAG (task #7): NPC pool `gclient_t` — dereffed raw for every velocity op.
    let client = ctx.world.entity(npc_id).client;
    let npc_origin_z = ctx.world.entity(npc_id).r.currentOrigin[2];
    let enemy = ctx.world.entity(npc_id).enemy;

    // If we have an enemy, we should try to hover at about enemy eye level
    if let Some(enemy_id) = enemy {
        // Find the height difference
        let e = ctx.world.entity(enemy_id);
        dif = (e.r.currentOrigin[2] + e.r.maxs[2]) - npc_origin_z;

        // cap to prevent dramatic height shifts
        if dif.abs() > 2.0 {
            if dif.abs() > 16.0 {
                dif = if dif < 0.0 { -16.0 } else { 16.0 };
            }

            unsafe {
                (*client).ps.velocity[2] = ((*client).ps.velocity[2] + dif) / 2.0;
            }
        }
    } else {
        // FLAG (task #7): NPCInfo (gNPC_t) goalEntity/lastGoalEntity — raw reads.
        let goal: Option<EntityId> = unsafe {
            if (*npc_info).goalEntity.is_some() {
                // Is there a goal?
                (*npc_info).goalEntity
            } else {
                (*npc_info).lastGoalEntity
            }
        };

        if let Some(goal_id) = goal {
            dif = ctx.world.entity(goal_id).r.currentOrigin[2] - npc_origin_z;

            if dif.abs() > 24.0 {
                ctx.world.globals.ucmd.upmove = if ctx.world.globals.ucmd.upmove < 0 {
                    -4
                } else {
                    4
                };
            } else {
                unsafe {
                    if (*client).ps.velocity[2] != 0.0 {
                        (*client).ps.velocity[2] *= VELOCITY_DECAY;

                        if (*client).ps.velocity[2].abs() < 2.0 {
                            (*client).ps.velocity[2] = 0.0;
                        }
                    }
                }
            }
        }
        // Apply friction
        else {
            unsafe {
                if (*client).ps.velocity[2] != 0.0 {
                    (*client).ps.velocity[2] *= VELOCITY_DECAY;

                    if (*client).ps.velocity[2].abs() < 1.0 {
                        (*client).ps.velocity[2] = 0.0;
                    }
                }
            }
        }
    }

    // Apply friction
    unsafe {
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

/// Raven `Interrogator_Strafe`.
///
/// Perform a strafe movement away from the target.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:238-279`
pub fn Interrogator_Strafe(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let npc_info = ctx.world.globals.NPCInfo;

    let mut end: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut tr: trace_t = unsafe { core::mem::zeroed() };

    // FLAG (task #7): NPC pool `gclient_t` — raw deref for eyeAngles / velocity.
    let client = ctx.world.entity(npc_id).client;
    let eye_angles = unsafe { (*client).renderInfo.eyeAngles };
    AngleVectors(eye_angles, None, Some(&mut right), None);

    // Pick a random strafe direction, then check to see if doing a strafe would be
    // reasonable valid
    let dir = if (ctx.world.bg_state.rng.rand() & 1) != 0 {
        -1
    } else {
        1
    };
    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
    _VectorMA(
        npc_origin,
        (HUNTER_STRAFE_DIS * dir) as f32,
        right,
        &mut end,
    );

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
            let vel = (*client).ps.velocity;
            _VectorMA(
                vel,
                (HUNTER_STRAFE_VEL * dir) as f32,
                right,
                &mut (*client).ps.velocity,
            );
        }

        // Add a slight upward push
        let enemy = ctx.world.entity(npc_id).enemy;
        if let Some(enemy_id) = enemy {
            // Find the height difference
            let enemy_origin_z = ctx.world.entity(enemy_id).r.currentOrigin[2];
            let npc_origin_z = ctx.world.entity(npc_id).r.currentOrigin[2];
            let mut dif = (enemy_origin_z + 32.0) - npc_origin_z;

            // cap to prevent dramatic height shifts
            if dif.abs() > 8.0 {
                dif = if dif < 0.0 {
                    -(HUNTER_UPWARD_PUSH as f32)
                } else {
                    HUNTER_UPWARD_PUSH as f32
                };
            }

            unsafe {
                (*client).ps.velocity[2] += dif;
            }
        }

        // Set the strafe start time. FLAG (task #7): NPCInfo (gNPC_t) standTime — raw write.
        let stand =
            ctx.world.level.time + 3000 + (ctx.world.bg_state.rng.random() * 500.0) as c_int;
        unsafe {
            (*npc_info).standTime = stand;
        }
    }
}

/// Raven `Interrogator_Hunt`.
///
/// Hunt the enemy, using strafe and movement.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:290-336`
pub fn Interrogator_Hunt(ctx: &mut GameContext, visible: qboolean, advance: qboolean) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let npc_info = ctx.world.globals.NPCInfo;

    Interrogator_PartsMove(ctx);

    NPC_FaceEnemy(ctx, 0);

    // If we're not supposed to stand still, pursue the player.
    // FLAG (task #7): NPCInfo (gNPC_t) standTime — raw reads.
    if unsafe { (*npc_info).standTime } < ctx.world.level.time {
        // Only strafe when we can see the player
        if visible != 0 {
            Interrogator_Strafe(ctx);
            if unsafe { (*npc_info).standTime } > ctx.world.level.time {
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
        // Move towards our goal. FLAG (task #7): NPCInfo (gNPC_t) goalEntity/goalRadius — raw writes.
        let enemy = ctx.world.entity(npc_id).enemy;
        unsafe {
            (*npc_info).goalEntity = enemy;
            (*npc_info).goalRadius = 12;
        }

        // Get our direction from the navigator if we can't see our target
        if NPC_GetMoveDirection(ctx, &mut forward, &mut distance as *mut f32) == 0 {
            return;
        }
    } else {
        let enemy = ctx.world.entity(npc_id).enemy;
        let src = match enemy {
            Some(enemy_id) => ctx.world.entity(enemy_id).r.currentOrigin,
            None => ctx.world.entity(npc_id).r.currentOrigin,
        };
        let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
        _VectorSubtract(src, npc_origin, &mut forward);
        distance = VectorNormalize(&mut forward);
    }

    let speed = HUNTER_FORWARD_BASE_SPEED as f32
        + (HUNTER_FORWARD_MULTIPLIER as f32) * ctx.world.cvars.g_spskill.integer as f32;
    // FLAG (task #7): NPC pool `gclient_t` — raw deref for velocity.
    let client = ctx.world.entity(npc_id).client;
    unsafe {
        let vel = (*client).ps.velocity;
        _VectorMA(vel, speed, forward, &mut (*client).ps.velocity);
    }
}

/// Raven `Interrogator_Melee`.
///
/// Perform melee attack if close enough and within height range.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:345-374`
pub fn Interrogator_Melee(ctx: &mut GameContext, visible: qboolean, advance: qboolean) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let npc_info = ctx.world.globals.NPCInfo;

    if TIMER_Done(ctx, Some(npc_id), c"attackDelay".as_ptr()) != 0 {
        let enemy = ctx.world.entity(npc_id).enemy;

        if let Some(enemy_id) = enemy {
            // Make sure that we are within the height range before we allow any damage to happen
            let npc_z = ctx.world.entity(npc_id).r.currentOrigin[2];
            let npc_mins_z = ctx.world.entity(npc_id).r.mins[2];
            let e = ctx.world.entity(enemy_id);
            let e_z = e.r.currentOrigin[2];
            let e_mins_z = e.r.mins[2];
            let e_maxs_z = e.r.maxs[2];

            if npc_z >= e_z + e_mins_z && npc_z + npc_mins_z + 8.0 < e_z + e_maxs_z {
                let delay = ctx.world.bg_state.rng.Q_irand(500, 3000);
                TIMER_Set(ctx, Some(npc_id), c"attackDelay".as_ptr(), delay);
                G_Damage(
                    ctx,
                    Some(enemy_id),
                    Some(npc_id),
                    Some(npc_id),
                    None,
                    [0.0f32; 3],
                    2,
                    DAMAGE_NO_KNOCKBACK,
                    MOD_MELEE as c_int,
                );

                G_Sound(
                    ctx,
                    Some(npc_id),
                    CHAN_AUTO,
                    G_SoundIndex(
                        c"sound/chars/interrogator/misc/torture_droid_inject.mp3".as_ptr(),
                    ),
                );
            }
        }
    }

    // FLAG (task #7): NPCInfo (gNPC_t) scriptFlags — raw read.
    if unsafe { (*npc_info).scriptFlags } & SCF_CHASE_ENEMIES != 0 {
        Interrogator_Hunt(ctx, visible, advance);
    }
}

/// Raven `Interrogator_Attack`.
///
/// Main attack function - handles distance, visibility, and attack selection.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:381-428`
pub fn Interrogator_Attack(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let npc_info = ctx.world.globals.NPCInfo;

    // Always keep a good height off the ground
    Interrogator_MaintainHeight(ctx);

    // randomly talk
    if TIMER_Done(ctx, Some(npc_id), c"patrolNoise".as_ptr()) != 0 {
        if TIMER_Done(ctx, Some(npc_id), c"angerNoise".as_ptr()) != 0 {
            // Raven: `va("sound/chars/probe/misc/talk.wav", Q_irand(1, 3))` — the
            // format string has no specifier, so the value is discarded, but the
            // Q_irand still advances the holdrand stream; keep the draw.
            // Source: `oracle/codemp/game/NPC_AI_Interrogator.c:395`
            let _ = ctx.world.bg_state.rng.Q_irand(1, 3);
            G_SoundOnEnt(
                ctx,
                npc_id,
                CHAN_AUTO,
                c"sound/chars/probe/misc/talk.wav".as_ptr(),
            );

            let delay = ctx.world.bg_state.rng.Q_irand(4000, 10000);
            TIMER_Set(ctx, Some(npc_id), c"patrolNoise".as_ptr(), delay);
        }
    }

    // If we don't have an enemy, just idle
    if NPC_CheckEnemyExt(ctx, 0) == 0 {
        Interrogator_Idle(ctx);
        return;
    }

    // Rate our distance to the target, and our visibility
    let enemy = ctx.world.entity(npc_id).enemy;
    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
    let enemy_origin = match enemy {
        Some(enemy_id) => ctx.world.entity(enemy_id).r.currentOrigin,
        None => npc_origin,
    };
    let distance = DistanceHorizontalSquared(npc_origin, enemy_origin) as c_int;

    let visible = NPC_ClearLOS4(ctx, enemy);

    let mut advance = if distance > MIN_DISTANCE * MIN_DISTANCE {
        1
    } else {
        0
    };

    if visible == 0 {
        advance = 1;
    }

    // FLAG (task #7): NPCInfo (gNPC_t) scriptFlags — raw read.
    if unsafe { (*npc_info).scriptFlags } & SCF_CHASE_ENEMIES != 0 {
        Interrogator_Hunt(ctx, visible, advance);
    }

    NPC_FaceEnemy(ctx, 1);

    if advance == 0 {
        Interrogator_Melee(ctx, visible, advance);
    }
}

/// Raven `Interrogator_Idle`.
///
/// Idle behavior - check for stealth enemies and maintain height.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:435-447`
pub fn Interrogator_Idle(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    if NPC_CheckPlayerTeamStealth(ctx) != 0 {
        G_SoundOnEnt(
            ctx,
            npc_id,
            CHAN_AUTO,
            c"sound/chars/mark1/misc/anger.wav".as_ptr(),
        );
        NPC_UpdateAngles(ctx, 1, 1);
        return;
    }

    Interrogator_MaintainHeight(ctx);

    NPC_BSIdle(ctx);
}

/// Raven `NPC_BSInterrogator_Default`.
///
/// Default behavior state selector - attacks if enemy present, otherwise idles.
/// Source: `oracle/codemp/game/NPC_AI_Interrogator.c:454-467`
pub fn NPC_BSInterrogator_Default(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    if ctx.world.entity(npc_id).enemy.is_some() {
        Interrogator_Attack(ctx);
    } else {
        Interrogator_Idle(ctx);
    }
}
