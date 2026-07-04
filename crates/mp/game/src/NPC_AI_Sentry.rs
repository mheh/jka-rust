// PORT-COMPLETE: NPC_AI_Sentry.c 12/12
//! Faithful port of `oracle/oracle/codemp/game/NPC_AI_Sentry.c`.
//!
//! Sentry droid NPC AI behavior.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

// Local state enums
const LSTATE_NONE: i32 = 0;
const LSTATE_ASLEEP: i32 = 1;
const LSTATE_WAKEUP: i32 = 2;
const LSTATE_ACTIVE: i32 = 3;
const LSTATE_POWERING_UP: i32 = 4;
const LSTATE_ATTACKING: i32 = 5;

const MIN_DISTANCE: i32 = 256;
const MIN_DISTANCE_SQR: i32 = MIN_DISTANCE * MIN_DISTANCE;

const SENTRY_FORWARD_BASE_SPEED: i32 = 10;
const SENTRY_FORWARD_MULTIPLIER: i32 = 5;

const SENTRY_VELOCITY_DECAY: f32 = 0.85f32;
const SENTRY_STRAFE_VEL: i32 = 256;
const SENTRY_STRAFE_DIS: i32 = 200;
const SENTRY_UPWARD_PUSH: i32 = 32;
const SENTRY_HOVER_HEIGHT: i32 = 24;

/// Raven `NPC_Sentry_Precache`.
///
/// Precache sentry sounds and effects.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:37-57`
pub fn NPC_Sentry_Precache() {
    G_SoundIndex(c"sound/chars/sentry/misc/sentry_explo".as_ptr());
    G_SoundIndex(c"sound/chars/sentry/misc/sentry_pain".as_ptr());
    G_SoundIndex(c"sound/chars/sentry/misc/sentry_shield_open".as_ptr());
    G_SoundIndex(c"sound/chars/sentry/misc/sentry_shield_close".as_ptr());
    G_SoundIndex(c"sound/chars/sentry/misc/sentry_hover_1_lp".as_ptr());
    G_SoundIndex(c"sound/chars/sentry/misc/sentry_hover_2_lp".as_ptr());

    for i in 1..4 {
        G_SoundIndex(va(c"sound/chars/sentry/misc/talk%d".as_ptr(), i));
    }

    G_EffectIndex(c"bryar/muzzle_flash".as_ptr());
    G_EffectIndex(c"env/med_explode".as_ptr());

    RegisterItem(BG_FindItemForAmmo(AMMO_BLASTER));
}

/// Raven `sentry_use`.
///
/// Callback when sentry is used. Activates behavior and transitions from sleep.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:64-72`
pub fn sentry_use(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    G_ActivateBehavior(self_, BSET_USE);

    unsafe {
        (*self_).flags &= !FL_SHIELDED;
        NPC_SetAnim(
            self_,
            SETANIM_BOTH,
            BOTH_POWERUP1,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        if !(*self_).NPC.is_null() {
            (*(*self_).NPC).localState = LSTATE_ACTIVE;
        }
    }
}

/// Raven `NPC_Sentry_Pain`.
///
/// Handle sentry taking damage. Special handling for DEMP2 disable.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:79-105`
pub fn NPC_Sentry_Pain(
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    let mod_ = gPainMOD;

    NPC_Pain(self_, attacker, damage);

    if mod_ == MOD_DEMP2 || mod_ == MOD_DEMP2_ALT {
        unsafe {
            if !(*self_).NPC.is_null() {
                (*(*self_).NPC).burstCount = 0;
                TIMER_Set(self_, c"attackDelay".as_ptr(), Q_irand(9000, 12000));
                (*self_).flags |= FL_SHIELDED;
                NPC_SetAnim(
                    self_,
                    SETANIM_BOTH,
                    BOTH_FLY_SHIELDED,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                G_Sound(
                    self_,
                    CHAN_AUTO,
                    G_SoundIndex(c"sound/chars/sentry/misc/sentry_pain".as_ptr()),
                );

                (*(*self_).NPC).localState = LSTATE_ACTIVE;
            }
        }
    }
}

/// Raven `Sentry_Fire`.
///
/// Fire a blaster bolt from one of three muzzles. Difficulty-scaled damage.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:112-203`
pub fn Sentry_Fire() {
    // Static vectors for direction angles (ruling 5: function-scope statics become owned values).
    let mut forward: vec3_t = [0.0; 3];
    let mut vright: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];

    let mut muzzle: vec3_t = [0.0; 3];
    let mut boltMatrix: mdxaBone_t = unsafe { std::mem::zeroed() };

    unsafe {
        (*NPC).flags &= !FL_SHIELDED;

        if (*NPCInfo).localState == LSTATE_POWERING_UP {
            if TIMER_Done(NPC, c"powerup".as_ptr()) {
                (*NPCInfo).localState = LSTATE_ATTACKING;
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_ATTACK1,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
            } else {
                // can't do anything right now
                return;
            }
        } else if (*NPCInfo).localState == LSTATE_ACTIVE {
            (*NPCInfo).localState = LSTATE_POWERING_UP;

            G_Sound(
                NPC,
                CHAN_AUTO,
                G_SoundIndex(c"sound/chars/sentry/misc/sentry_shield_open".as_ptr()),
            );
            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_POWERUP1,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            TIMER_Set(NPC, c"powerup".as_ptr(), 250);
            return;
        } else if (*NPCInfo).localState != LSTATE_ATTACKING {
            // bad because we are uninitialized
            (*NPCInfo).localState = LSTATE_ACTIVE;
            return;
        }

        // Which muzzle to fire from?
        let which = (*NPCInfo).burstCount % 3;
        let bolt = match which {
            0 => trap::G2API_AddBolt(NPC, (*NPC).ghoul2, 0, c"*flash1".as_ptr()),
            1 => trap::G2API_AddBolt(NPC, (*NPC).ghoul2, 0, c"*flash2".as_ptr()),
            _ => trap::G2API_AddBolt(NPC, (*NPC).ghoul2, 0, c"*flash03".as_ptr()),
        };

        trap::G2API_GetBoltMatrix(
            NPC,
            (*NPC).ghoul2,
            0,
            bolt,
            &mut boltMatrix,
            (*NPC).r.currentAngles,
            (*NPC).r.currentOrigin,
            level.time,
            std::ptr::null_mut(),
            (*NPC).modelScale,
        );

        BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut muzzle);

        AngleVectors((*NPC).r.currentAngles, &mut forward, &mut vright, &mut up);

        G_PlayEffectID(
            G_EffectIndex(c"bryar/muzzle_flash".as_ptr()),
            muzzle,
            forward,
        );

        let missile = CreateMissile(muzzle, forward, 1600, 10000, NPC, qfalse);

        (*missile).classname = c"bryar_proj".as_ptr();
        (*missile).s.weapon = WP_BRYAR_PISTOL;

        (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
        (*missile).methodOfDeath = MOD_BRYAR_PISTOL;
        (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

        (*NPCInfo).burstCount += 1;
        (*NPC).attackDebounceTime = level.time + 50;
        (*missile).damage = 5;

        // now scale for difficulty
        if g_spskill.integer == 0 {
            (*NPC).attackDebounceTime += 200;
            (*missile).damage = 1;
        } else if g_spskill.integer == 1 {
            (*NPC).attackDebounceTime += 100;
            (*missile).damage = 3;
        }
    }
}

/// Raven `Sentry_MaintainHeight`.
///
/// Maintain hovering height relative to enemy or goal. Apply friction to velocity.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:210-304`
pub fn Sentry_MaintainHeight() {
    unsafe {
        (*NPC).s.loopSound = G_SoundIndex(c"sound/chars/sentry/misc/sentry_hover_1_lp".as_ptr());

        // Update our angles regardless
        NPC_UpdateAngles(qtrue, qtrue);

        // If we have an enemy, we should try to hover at about enemy eye level
        if !(*NPC).enemy.is_null() {
            // Find the height difference
            let dif = ((*(*NPC).enemy).r.currentOrigin[2]
                + (*(*NPC).enemy).r.maxs[2])
                - (*NPC).r.currentOrigin[2];

            // cap to prevent dramatic height shifts
            if dif.abs() > 8.0 {
                let adjusted_dif = if dif.abs() > SENTRY_HOVER_HEIGHT as f32 {
                    if dif < 0.0 {
                        -24.0
                    } else {
                        24.0
                    }
                } else {
                    dif
                };

                (*(*NPC).client).ps.velocity[2] =
                    ((*(*NPC).client).ps.velocity[2] + adjusted_dif) / 2.0;
            }
        } else {
            let mut goal: *mut gentity_t = std::ptr::null_mut();

            if !(*NPCInfo).goalEntity.is_null() {
                goal = (*NPCInfo).goalEntity;
            } else {
                goal = (*NPCInfo).lastGoalEntity;
            }

            if !goal.is_null() {
                let dif = (*goal).r.currentOrigin[2] - (*NPC).r.currentOrigin[2];

                if dif.abs() > SENTRY_HOVER_HEIGHT as f32 {
                    ucmd.upmove = if ucmd.upmove < 0 { -4 } else { 4 };
                } else {
                    if (*(*NPC).client).ps.velocity[2] != 0.0 {
                        (*(*NPC).client).ps.velocity[2] *= SENTRY_VELOCITY_DECAY;

                        if (*(*NPC).client).ps.velocity[2].abs() < 2.0 {
                            (*(*NPC).client).ps.velocity[2] = 0.0;
                        }
                    }
                }
            }
            // Apply friction to Z
            else if (*(*NPC).client).ps.velocity[2] != 0.0 {
                (*(*NPC).client).ps.velocity[2] *= SENTRY_VELOCITY_DECAY;

                if (*(*NPC).client).ps.velocity[2].abs() < 1.0 {
                    (*(*NPC).client).ps.velocity[2] = 0.0;
                }
            }
        }

        // Apply friction
        if (*(*NPC).client).ps.velocity[0] != 0.0 {
            (*(*NPC).client).ps.velocity[0] *= SENTRY_VELOCITY_DECAY;

            if (*(*NPC).client).ps.velocity[0].abs() < 1.0 {
                (*(*NPC).client).ps.velocity[0] = 0.0;
            }
        }

        if (*(*NPC).client).ps.velocity[1] != 0.0 {
            (*(*NPC).client).ps.velocity[1] *= SENTRY_VELOCITY_DECAY;

            if (*(*NPC).client).ps.velocity[1].abs() < 1.0 {
                (*(*NPC).client).ps.velocity[1] = 0.0;
            }
        }

        NPC_FaceEnemy(qtrue);
    }
}

/// Raven `Sentry_Idle`.
///
/// Idle behavior: sleep or wake up based on local state.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:311-331`
pub fn Sentry_Idle() {
    Sentry_MaintainHeight();

    unsafe {
        // Is he waking up?
        if (*NPCInfo).localState == LSTATE_WAKEUP {
            if (*(*NPC).client).ps.torsoTimer <= 0 {
                (*NPCInfo).scriptFlags |= SCF_LOOK_FOR_ENEMIES;
                (*NPCInfo).burstCount = 0;
            }
        } else {
            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_SLEEP1,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            (*NPC).flags |= FL_SHIELDED;

            NPC_BSIdle();
        }
    }
}

/// Raven `Sentry_Strafe`.
///
/// Strafe horizontally away from enemy. Trace to check validity.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:338-365`
pub fn Sentry_Strafe() {
    let mut end: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut tr: trace_t = unsafe { std::mem::zeroed() };

    unsafe {
        AngleVectors(
            (*(*NPC).client).renderInfo.eyeAngles,
            std::ptr::null_mut(),
            &mut right,
            std::ptr::null_mut(),
        );

        // Pick a random strafe direction, then check to see if doing a strafe would be
        // reasonable valid
        let dir = if (rand() & 1) != 0 { -1 } else { 1 };
        VectorMA(
            (*NPC).r.currentOrigin,
            (SENTRY_STRAFE_DIS * dir) as f32,
            right,
            &mut end,
        );

        trap::Trace(
            NPC,
            &mut tr,
            (*NPC).r.currentOrigin,
            std::ptr::null(),
            std::ptr::null(),
            end,
            (*NPC).s.number,
            MASK_SOLID,
        );

        // Close enough
        if tr.fraction > 0.9f32 {
            VectorMA(
                (*(*NPC).client).ps.velocity,
                (SENTRY_STRAFE_VEL * dir) as f32,
                right,
                &mut (*(*NPC).client).ps.velocity,
            );

            // Add a slight upward push
            (*(*NPC).client).ps.velocity[2] += SENTRY_UPWARD_PUSH as f32;

            // Set the strafe start time so we can do a controlled roll
            (*NPCInfo).standTime = level.time + 3000 + (random() * 500.0) as c_int;
        }
    }
}

/// Raven `Sentry_Hunt`.
///
/// Hunt the enemy. Move toward or strafe, depending on visibility.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:372-411`
pub fn Sentry_Hunt(visible: qboolean, advance: qboolean) {
    let mut forward: vec3_t = [0.0; 3];
    let mut distance: f32 = 0.0;

    unsafe {
        //If we're not supposed to stand still, pursue the player
        if (*NPCInfo).standTime < level.time {
            // Only strafe when we can see the player
            if visible != qfalse {
                Sentry_Strafe();
                return;
            }
        }

        //If we don't want to advance, stop here
        if advance == qfalse && visible != qfalse {
            return;
        }

        //Only try and navigate if the player is visible
        if visible == qfalse {
            // Move towards our goal
            (*NPCInfo).goalEntity = (*NPC).enemy;
            (*NPCInfo).goalRadius = 12;

            //Get our direction from the navigator if we can't see our target
            if NPC_GetMoveDirection(&mut forward, &mut distance) == qfalse {
                return;
            }
        } else {
            VectorSubtract(
                (*(*NPC).enemy).r.currentOrigin,
                (*NPC).r.currentOrigin,
                &mut forward,
            );
            distance = VectorNormalize(&mut forward);
        }

        let speed =
            (SENTRY_FORWARD_BASE_SPEED + SENTRY_FORWARD_MULTIPLIER * g_spskill.integer) as f32;
        VectorMA(
            (*(*NPC).client).ps.velocity,
            speed,
            forward,
            &mut (*(*NPC).client).ps.velocity,
        );
    }
}

/// Raven `Sentry_RangedAttack`.
///
/// Ranged attack: fire or close shield. Hunt if pursuing.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:418-448`
pub fn Sentry_RangedAttack(visible: qboolean, advance: qboolean) {
    unsafe {
        if TIMER_Done(NPC, c"attackDelay".as_ptr())
            && (*NPC).attackDebounceTime < level.time
            && visible != qfalse
        {
            // Attack?
            if (*NPCInfo).burstCount > 6 {
                if (*NPC).fly_sound_debounce_time == 0 {
                    //delay closing down to give the player an opening
                    (*NPC).fly_sound_debounce_time = level.time + Q_irand(500, 2000);
                } else if (*NPC).fly_sound_debounce_time < level.time {
                    (*NPCInfo).localState = LSTATE_ACTIVE;
                    (*NPC).fly_sound_debounce_time = 0;
                    (*NPCInfo).burstCount = 0;
                    TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(2000, 3500));
                    (*NPC).flags |= FL_SHIELDED;
                    NPC_SetAnim(
                        NPC,
                        SETANIM_BOTH,
                        BOTH_FLY_SHIELDED,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    G_SoundOnEnt(
                        NPC,
                        CHAN_AUTO,
                        c"sound/chars/sentry/misc/sentry_shield_close".as_ptr(),
                    );
                }
            } else {
                Sentry_Fire();
            }
        }

        if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
            Sentry_Hunt(visible, advance);
        }
    }
}

/// Raven `Sentry_AttackDecision`.
///
/// Decide how to attack: maintain height, check enemy, determine if visible/in range.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:455-510`
pub fn Sentry_AttackDecision() {
    let mut distance: f32 = 0.0;
    let mut visible: qboolean;
    let mut advance: qboolean;

    unsafe {
        // Always keep a good height off the ground
        Sentry_MaintainHeight();

        (*NPC).s.loopSound = G_SoundIndex(c"sound/chars/sentry/misc/sentry_hover_2_lp".as_ptr());

        //randomly talk
        if TIMER_Done(NPC, c"patrolNoise".as_ptr()) {
            if TIMER_Done(NPC, c"angerNoise".as_ptr()) {
                G_SoundOnEnt(
                    NPC,
                    CHAN_AUTO,
                    va(
                        c"sound/chars/sentry/misc/talk%d".as_ptr(),
                        Q_irand(1, 3),
                    ),
                );

                TIMER_Set(NPC, c"patrolNoise".as_ptr(), Q_irand(4000, 10000));
            }
        }

        // He's dead.
        if (*(*NPC).enemy).health < 1 {
            (*NPC).enemy = std::ptr::null_mut();
            Sentry_Idle();
            return;
        }

        // If we don't have an enemy, just idle
        if NPC_CheckEnemyExt(qfalse) == qfalse {
            Sentry_Idle();
            return;
        }

        // Rate our distance to the target and visibility
        distance = DistanceHorizontalSquared((*NPC).r.currentOrigin, (*(*NPC).enemy).r.currentOrigin);
        visible = NPC_ClearLOS4((*NPC).enemy);
        advance = (distance > MIN_DISTANCE_SQR as f32) as qboolean;

        // If we cannot see our target, move to see it
        if visible == qfalse {
            if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
                Sentry_Hunt(visible, advance);
                return;
            }
        }

        NPC_FaceEnemy(qtrue);

        Sentry_RangedAttack(visible, advance);
    }
}

extern "C" {
    pub fn NPC_CheckPlayerTeamStealth() -> qboolean;
}

/// Raven `NPC_Sentry_Patrol`.
///
/// Patrol behavior: maintain height, check for stealth, update goal, talk.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:519-550`
pub fn NPC_Sentry_Patrol() {
    Sentry_MaintainHeight();

    unsafe {
        //If we have somewhere to go, then do that
        if (*NPC).enemy.is_null() {
            if NPC_CheckPlayerTeamStealth() != qfalse {
                //NPC_AngerSound();
                NPC_UpdateAngles(qtrue, qtrue);
                return;
            }

            let goal_ent = UpdateGoal();
            if !goal_ent.is_null() {
                //start loop sound once we move
                ucmd.buttons |= BUTTON_WALKING;
                NPC_MoveToGoal(qtrue);
            }

            //randomly talk
            if TIMER_Done(NPC, c"patrolNoise".as_ptr()) {
                G_SoundOnEnt(
                    NPC,
                    CHAN_AUTO,
                    va(c"sound/chars/sentry/misc/talk%d".as_ptr(), Q_irand(1, 3)),
                );

                TIMER_Set(NPC, c"patrolNoise".as_ptr(), Q_irand(2000, 4000));
            }
        }

        NPC_UpdateAngles(qtrue, qtrue);
    }
}

/// Raven `NPC_BSSentry_Default`.
///
/// Main behavior selector: handle use callbacks, attack if enemy, patrol, or idle.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:557-577`
pub fn NPC_BSSentry_Default() {
    unsafe {
        if !(*NPC).targetname.is_null() {
            (*NPC).use = Some(sentry_use);
        }

        if !(*NPC).enemy.is_null() && (*NPCInfo).localState != LSTATE_WAKEUP {
            // Don't attack if waking up or if no enemy
            Sentry_AttackDecision();
        } else if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            NPC_Sentry_Patrol();
        } else {
            Sentry_Idle();
        }
    }
}
