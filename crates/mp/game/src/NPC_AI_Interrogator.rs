// PORT-COMPLETE: NPC_AI_Interrogator.c 9/10
//! Faithful port of `oracle/oracle/codemp/game/NPC_AI_Interrogator.c` (jampgame mega-pass).
//!
//! Interrogator droid NPC AI behavior: idle, patrol, hunt, strafe, melee attack.
//!
//! One function (`Interrogator_Strafe`) is parked due to trap_Trace requiring
//! an `&Engine` handle which these context-free AI functions don't have access to.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

/// Local state enums for Interrogator blade movement.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:8-13`
const LSTATE_BLADESTOP: c_int = 0;
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:11`
const LSTATE_BLADEUP: c_int = 1;
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:12`
const LSTATE_BLADEDOWN: c_int = 2;

/// Velocity decay factor for Interrogator hovering.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:129`
const VELOCITY_DECAY: f32 = 0.85;

/// Upward push for Interrogator during strafe.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:130`
const HUNTER_UPWARD_PUSH: c_int = 2;

/// Strafe velocity for Interrogator movement.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:231`
const HUNTER_STRAFE_VEL: c_int = 32;

/// Distance for Interrogator strafe.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:232`
const HUNTER_STRAFE_DIS: c_int = 200;

/// Forward base speed for Interrogator.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:287`
const HUNTER_FORWARD_BASE_SPEED: c_int = 10;

/// Forward speed multiplier for Interrogator.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:288`
const HUNTER_FORWARD_MULTIPLIER: c_int = 2;

/// Minimum distance for Interrogator melee attack.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:338`
const MIN_DISTANCE: c_int = 64;


/// Raven `NPC_Interrogator_Precache`.
///
/// Precache sounds and effects for the Interrogator NPC.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:20-28`
pub fn NPC_Interrogator_Precache(self_: *mut gentity_t) {
    G_SoundIndex(c"sound/chars/interrogator/misc/torture_droid_lp".as_ptr() as *const c_char);
    G_SoundIndex(c"sound/chars/mark1/misc/anger.wav".as_ptr() as *const c_char);
    G_SoundIndex(c"sound/chars/probe/misc/talk".as_ptr() as *const c_char);
    G_SoundIndex(
        c"sound/chars/interrogator/misc/torture_droid_inject".as_ptr() as *const c_char,
    );
    G_SoundIndex(c"sound/chars/interrogator/misc/int_droid_explo".as_ptr() as *const c_char);
    G_EffectIndex(c"explosions/droidexplosion1".as_ptr() as *const c_char);
}

/// Raven `Interrogator_die`.
///
/// Death behavior for Interrogator NPC. Sets velocity and clears flying flag.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:34-57`
pub fn Interrogator_die(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
    r#mod: c_int,
    dFlags: c_int,
    hitLoc: c_int,
) {
    unsafe {
        // SAFETY: self_ accessed through game context.
        self_.as_mut().map(|ent| {
            if !ent.client.is_null() {
                let client = &mut *ent.client;
                client.ps.velocity[2] = -100.0;

                // Clear flying flag and set random horizontal velocity
                client.ps.eFlags2 &= !(crate::prelude::EF2_FLYING as u32);
                client.ps.velocity[0] = Q_irand(-20, -10) as f32;
                client.ps.velocity[1] = Q_irand(-20, -10) as f32;
                client.ps.velocity[2] = -100.0;
            }
        });
    }
}

/// Raven `Interrogator_PartsMove`.
///
/// Move the syringe, scalpel, and claw parts of the Interrogator.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:64-127`
pub fn Interrogator_PartsMove() {
    unsafe {
        // SAFETY: NPC, NPCInfo accessed through game context (global AI state).
        // Syringe
        if TIMER_Done(NPC, b"syringeDelay\0".as_ptr() as *const c_char) != 0 {
            (*NPC).pos1[1] = AngleNormalize360((*NPC).pos1[1]);

            if ((*NPC).pos1[1] < 60.0) || ((*NPC).pos1[1] > 300.0) {
                (*NPC).pos1[1] += Q_irand(-20, 20) as f32; // Pitch
            } else if (*NPC).pos1[1] > 180.0 {
                (*NPC).pos1[1] = Q_irand(300, 360) as f32; // Pitch
            } else {
                (*NPC).pos1[1] = Q_irand(0, 60) as f32; // Pitch
            }

            NPC_SetBoneAngles(NPC, b"left_arm\0".as_ptr() as *const c_char, (*NPC).pos1);

            TIMER_Set(NPC, b"syringeDelay\0".as_ptr() as *const c_char, Q_irand(100, 1000));
        }

        // Scalpel
        if TIMER_Done(NPC, b"scalpelDelay\0".as_ptr() as *const c_char) != 0 {
            // Change pitch
            if (*NPCInfo).localState == LSTATE_BLADEDOWN {
                // Blade is moving down
                (*NPC).pos2[0] -= 30.0;
                if (*NPC).pos2[0] < 180.0 {
                    (*NPC).pos2[0] = 180.0;
                    (*NPCInfo).localState = LSTATE_BLADEUP; // Make it move up
                }
            } else {
                // Blade is coming back up
                (*NPC).pos2[0] += 30.0;
                if (*NPC).pos2[0] >= 360.0 {
                    (*NPC).pos2[0] = 360.0;
                    (*NPCInfo).localState = LSTATE_BLADEDOWN; // Make it move down
                    TIMER_Set(NPC, b"scalpelDelay\0".as_ptr() as *const c_char, Q_irand(100, 1000));
                }
            }

            (*NPC).pos2[0] = AngleNormalize360((*NPC).pos2[0]);

            NPC_SetBoneAngles(NPC, b"right_arm\0".as_ptr() as *const c_char, (*NPC).pos2);
        }

        // Claw
        (*NPC).pos3[1] += Q_irand(10, 30) as f32;
        (*NPC).pos3[1] = AngleNormalize360((*NPC).pos3[1]);

        NPC_SetBoneAngles(NPC, b"claw\0".as_ptr() as *const c_char, (*NPC).pos3);
    }
}

/// Raven `Interrogator_MaintainHeight`.
///
/// Maintain hover height relative to enemy or goal.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:137-229`
pub fn Interrogator_MaintainHeight() {
    unsafe {
        // SAFETY: NPC, NPCInfo, level, ucmd accessed through game context (global AI state).
        (*NPC).s.loopSound = G_SoundIndex(b"sound/chars/interrogator/misc/torture_droid_lp\0".as_ptr() as *const c_char);

        // Update our angles regardless
        NPC_UpdateAngles(qtrue, qtrue);

        // If we have an enemy, we should try to hover at about enemy eye level
        if !(*NPC).enemy.is_null() {
            let mut dif = ((*(*NPC).enemy).r.currentOrigin[2] + (*(*NPC).enemy).r.maxs[2])
                - (*NPC).r.currentOrigin[2];

            // Cap to prevent dramatic height shifts
            if dif.abs() > 2.0 {
                if dif.abs() > 16.0 {
                    dif = if dif < 0.0 { -16.0 } else { 16.0 };
                }

                if !(*NPC).client.is_null() {
                    (*(*NPC).client).ps.velocity[2] = ((*(*NPC).client).ps.velocity[2] + dif) / 2.0;
                }
            }
        } else {
            let mut goal: *mut gentity_t = std::ptr::null_mut();

            if !(*NPCInfo).goalEntity.is_null() {
                // Is there a goal?
                goal = (*NPCInfo).goalEntity;
            } else {
                goal = (*NPCInfo).lastGoalEntity;
            }

            if !goal.is_null() {
                let dif = (*goal).r.currentOrigin[2] - (*NPC).r.currentOrigin[2];

                if dif.abs() > 24.0 {
                    ucmd.upmove = if ucmd.upmove < 0 { -4 } else { 4 };
                } else {
                    if !(*NPC).client.is_null() && (*(*NPC).client).ps.velocity[2] != 0.0 {
                        (*(*NPC).client).ps.velocity[2] *= VELOCITY_DECAY;

                        if (*(*NPC).client).ps.velocity[2].abs() < 2.0 {
                            (*(*NPC).client).ps.velocity[2] = 0.0;
                        }
                    }
                }
            } else if !(*NPC).client.is_null() && (*(*NPC).client).ps.velocity[2] != 0.0 {
                // Apply friction
                (*(*NPC).client).ps.velocity[2] *= VELOCITY_DECAY;

                if (*(*NPC).client).ps.velocity[2].abs() < 1.0 {
                    (*(*NPC).client).ps.velocity[2] = 0.0;
                }
            }
        }

        // Apply friction to horizontal velocities
        if !(*NPC).client.is_null() && (*(*NPC).client).ps.velocity[0] != 0.0 {
            (*(*NPC).client).ps.velocity[0] *= VELOCITY_DECAY;

            if (*(*NPC).client).ps.velocity[0].abs() < 1.0 {
                (*(*NPC).client).ps.velocity[0] = 0.0;
            }
        }

        if !(*NPC).client.is_null() && (*(*NPC).client).ps.velocity[1] != 0.0 {
            (*(*NPC).client).ps.velocity[1] *= VELOCITY_DECAY;

            if (*(*NPC).client).ps.velocity[1].abs() < 1.0 {
                (*(*NPC).client).ps.velocity[1] = 0.0;
            }
        }
    }
}

// PORT-ESCALATION(trap-no-engine): `Interrogator_Strafe` calls `trap_Trace`
// which requires an `&Engine` handle; faithful context-free signature carries
// no threading mechanism to reach it (see NPC_utils.rs precedent).
/// Raven `Interrogator_Strafe`.
///
/// Perform a strafe movement away from the target.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:238-279`
pub fn Interrogator_Strafe() {
    todo!("Port Interrogator_Strafe — parked: trap-no-engine")
}

/// Raven `Interrogator_Hunt`.
///
/// Hunt the enemy, using strafe and movement.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:290-336`
pub fn Interrogator_Hunt(visible: qboolean, advance: qboolean) {
    unsafe {
        // SAFETY: NPC, NPCInfo, level, g_spskill accessed through game context (global AI state).
        Interrogator_PartsMove();

        NPC_FaceEnemy(qfalse);

        // If we're not supposed to stand still, pursue the player
        if (*NPCInfo).standTime < (*level).time {
            // Only strafe when we can see the player
            if visible != 0 {
                Interrogator_Strafe();
                if (*NPCInfo).standTime > (*level).time {
                    // Successfully strafed
                    return;
                }
            }
        }

        // If we don't want to advance, stop here
        if advance == 0 {
            return;
        }

        // Only try and navigate if the player is visible
        let mut forward: vec3_t = [0.0; 3];
        let mut distance: f32 = 0.0;

        if visible == 0 {
            // Move towards our goal
            (*NPCInfo).goalEntity = (*NPC).enemy;
            (*NPCInfo).goalRadius = 12.0;

            // Get our direction from the navigator if we can't see our target
            if NPC_GetMoveDirection(&mut forward, &mut distance) == 0 {
                return;
            }
        } else {
            VectorSubtract((*(*NPC).enemy).r.currentOrigin, (*NPC).r.currentOrigin, &mut forward);
            distance = VectorNormalize(forward);
        }

        let speed = HUNTER_FORWARD_BASE_SPEED as f32
            + HUNTER_FORWARD_MULTIPLIER as f32 * (*g_spskill).integer as f32;
        if !(*NPC).client.is_null() {
            VectorMA(
                (*(*NPC).client).ps.velocity,
                speed,
                forward,
                &mut (*(*NPC).client).ps.velocity,
            );
        }
    }
}

/// Raven `Interrogator_Melee`.
///
/// Perform melee attack if close enough and within height range.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:345-374`
pub fn Interrogator_Melee(visible: qboolean, advance: qboolean) {
    unsafe {
        // SAFETY: NPC, NPCInfo accessed through game context (global AI state).
        if TIMER_Done(NPC, b"attackDelay\0".as_ptr() as *const c_char) != 0 {
            // Attack?
            // Make sure that we are within the height range before we allow any damage to happen
            if (*NPC).r.currentOrigin[2] >= (*(*NPC).enemy).r.currentOrigin[2] + (*(*NPC).enemy).r.mins[2]
                && (*NPC).r.currentOrigin[2] + (*NPC).r.mins[2] + 8.0
                    < (*(*NPC).enemy).r.currentOrigin[2] + (*(*NPC).enemy).r.maxs[2]
            {
                TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(500, 3000));
                G_Damage(
                    (*NPC).enemy,
                    NPC,
                    NPC,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    2,
                    crate::prelude::DAMAGE_NO_KNOCKBACK,
                    crate::prelude::MOD_MELEE,
                );

                G_Sound(
                    NPC,
                    crate::prelude::CHAN_AUTO,
                    G_SoundIndex(
                        b"sound/chars/interrogator/misc/torture_droid_inject.mp3\0".as_ptr()
                            as *const c_char,
                    ),
                );
            }
        }

        if ((*NPCInfo).scriptFlags & crate::prelude::SCF_CHASE_ENEMIES) != 0 {
            Interrogator_Hunt(visible, advance);
        }
    }
}

/// Raven `Interrogator_Attack`.
///
/// Main attack function - handles distance, visibility, and attack selection.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:381-428`
pub fn Interrogator_Attack() {
    unsafe {
        // SAFETY: NPC, NPCInfo accessed through game context (global AI state).
        // Always keep a good height off the ground
        Interrogator_MaintainHeight();

        // Randomly talk
        if TIMER_Done(NPC, b"patrolNoise\0".as_ptr() as *const c_char) != 0 {
            if TIMER_Done(NPC, b"angerNoise\0".as_ptr() as *const c_char) != 0 {
                G_SoundOnEnt(
                    NPC,
                    crate::prelude::CHAN_AUTO,
                    va(
                        b"sound/chars/probe/misc/talk.wav\0".as_ptr() as *const c_char,
                        Q_irand(1, 3),
                    ),
                );

                TIMER_Set(NPC, b"patrolNoise\0".as_ptr() as *const c_char, Q_irand(4000, 10000));
            }
        }

        // If we don't have an enemy, just idle
        if NPC_CheckEnemyExt(qfalse) == 0 {
            Interrogator_Idle();
            return;
        }

        // Rate our distance to the target, and our visibility
        let distance = DistanceHorizontalSquared((*NPC).r.currentOrigin, (*(*NPC).enemy).r.currentOrigin);
        let visible = NPC_ClearLOS4((*NPC).enemy);
        let mut advance = if distance > (MIN_DISTANCE * MIN_DISTANCE) as f32 {
            qtrue
        } else {
            qfalse
        };

        if visible == 0 {
            advance = qtrue;
        }

        if ((*NPCInfo).scriptFlags & crate::prelude::SCF_CHASE_ENEMIES) != 0 {
            Interrogator_Hunt(visible, advance);
        }

        NPC_FaceEnemy(qtrue);

        if advance == 0 {
            Interrogator_Melee(visible, advance);
        }
    }
}

/// Raven `Interrogator_Idle`.
///
/// Idle behavior - check for stealth enemies and maintain height.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:435-447`
pub fn Interrogator_Idle() {
    unsafe {
        // SAFETY: NPC accessed through game context (global AI state).
        if NPC_CheckPlayerTeamStealth() != 0 {
            G_SoundOnEnt(
                NPC,
                crate::prelude::CHAN_AUTO,
                b"sound/chars/mark1/misc/anger.wav\0".as_ptr() as *const c_char,
            );
            NPC_UpdateAngles(qtrue, qtrue);
            return;
        }

        Interrogator_MaintainHeight();

        NPC_BSIdle();
    }
}

/// Raven `NPC_BSInterrogator_Default`.
///
/// Default behavior state selector - attacks if enemy present, otherwise idles.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:454-467`
pub fn NPC_BSInterrogator_Default() {
    unsafe {
        // SAFETY: NPC accessed through game context (global AI state).
        if !(*NPC).enemy.is_null() {
            Interrogator_Attack();
        } else {
            Interrogator_Idle();
        }
    }
}
