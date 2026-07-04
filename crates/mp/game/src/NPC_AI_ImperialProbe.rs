// PORT-COMPLETE: NPC_AI_ImperialProbe.c 12/12
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c`.
//!
//! Imperial Probe droid AI behavior: idle, patrol, hunt, strafe, ranged attack.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

// Local state enums
// Source: oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:10-17
const LSTATE_NONE: i32 = 0;
const LSTATE_BACKINGUP: i32 = 1;
const LSTATE_SPINNING: i32 = 2;
const LSTATE_PAIN: i32 = 3;
const LSTATE_DROP: i32 = 4;

// Height maintenance
// Source: oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:124-127
const VELOCITY_DECAY: f32 = 0.85;

// Strafe parameters
// Source: oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:178-181
const HUNTER_STRAFE_VEL: c_int = 256;
const HUNTER_STRAFE_DIS: c_int = 200;
const HUNTER_UPWARD_PUSH: c_int = 32;

// Hunt parameters
// Source: oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:294-296
const HUNTER_FORWARD_BASE_SPEED: c_int = 10;
const HUNTER_FORWARD_MULTIPLIER: c_int = 5;

// Melee range
// Source: oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:448-452
const MIN_MELEE_RANGE: c_int = 320;
const MIN_MELEE_RANGE_SQR: c_int = MIN_MELEE_RANGE * MIN_MELEE_RANGE;
const MIN_DISTANCE: c_int = 128;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

/// Raven `NPC_Probe_Precache`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:21-40`
pub fn NPC_Probe_Precache() {
    for i in 1..4 {
        let path = va(b"sound/chars/probe/misc/probetalk%d\0".as_ptr() as *const c_char, i);
        G_SoundIndex(path);
    }
    G_SoundIndex(b"sound/chars/probe/misc/probedroidloop\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/probe/misc/anger1\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/probe/misc/fire\0".as_ptr() as *const c_char);

    G_EffectIndex(b"chunks/probehead\0".as_ptr() as *const c_char);
    G_EffectIndex(b"env/med_explode2\0".as_ptr() as *const c_char);
    G_EffectIndex(b"explosions/probeexplosion1\0".as_ptr() as *const c_char);
    G_EffectIndex(b"bryar/muzzle_flash\0".as_ptr() as *const c_char);

    if let Some(item) = BG_FindItemForAmmo(AMMO_BLASTER) {
        RegisterItem(item);
    }
    if let Some(item) = BG_FindItemForWeapon(WP_BRYAR_PISTOL) {
        RegisterItem(item);
    }
}

/// Raven `ImperialProbe_MaintainHeight`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:49-170`
pub fn ImperialProbe_MaintainHeight() {
    // Update our angles regardless
    NPC_UpdateAngles(qtrue, qtrue);

    // If we have an enemy, we should try to hover at about enemy eye level
    if let Some(enemy) = unsafe { NPC.enemy } {
        // Find the height difference
        let dif = unsafe { enemy.r.currentOrigin[2] - NPC.r.currentOrigin[2] };

        // cap to prevent dramatic height shifts
        if dif.abs() > 8.0 {
            let capped_dif = if dif.abs() > 16.0 {
                if dif < 0.0 { -16.0 } else { 16.0 }
            } else {
                dif
            };

            unsafe {
                NPC.client.ps.velocity[2] = (NPC.client.ps.velocity[2] + capped_dif) / 2.0;
            }
        }
    } else {
        let mut goal: *const gentity_t = std::ptr::null();

        unsafe {
            if !NPCInfo.goalEntity.is_null() {
                goal = NPCInfo.goalEntity;
            } else if !NPCInfo.lastGoalEntity.is_null() {
                goal = NPCInfo.lastGoalEntity;
            }

            if !goal.is_null() {
                let dif = (*goal).r.currentOrigin[2] - NPC.r.currentOrigin[2];

                if dif.abs() > 24.0 {
                    ucmd.upmove = if ucmd.upmove < 0 { -4 } else { 4 };
                } else {
                    if NPC.client.ps.velocity[2] != 0.0 {
                        NPC.client.ps.velocity[2] *= VELOCITY_DECAY;

                        if NPC.client.ps.velocity[2].abs() < 2.0 {
                            NPC.client.ps.velocity[2] = 0.0;
                        }
                    }
                }
            } else if NPC.client.ps.velocity[2] != 0.0 {
                // Apply friction
                NPC.client.ps.velocity[2] *= VELOCITY_DECAY;

                if NPC.client.ps.velocity[2].abs() < 1.0 {
                    NPC.client.ps.velocity[2] = 0.0;
                }
            }
        }
    }

    // Apply friction to X and Y
    unsafe {
        if NPC.client.ps.velocity[0] != 0.0 {
            NPC.client.ps.velocity[0] *= VELOCITY_DECAY;

            if NPC.client.ps.velocity[0].abs() < 1.0 {
                NPC.client.ps.velocity[0] = 0.0;
            }
        }

        if NPC.client.ps.velocity[1] != 0.0 {
            NPC.client.ps.velocity[1] *= VELOCITY_DECAY;

            if NPC.client.ps.velocity[1].abs() < 1.0 {
                NPC.client.ps.velocity[1] = 0.0;
            }
        }
    }
}

/// Raven `ImperialProbe_Strafe`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:182-209`
pub fn ImperialProbe_Strafe() {
    let mut right = [0.0; 3];

    unsafe {
        AngleVectors(NPC.client.renderInfo.eyeAngles, std::ptr::null_mut(), right.as_mut_ptr(), std::ptr::null_mut());

        // Pick a random strafe direction, then check to see if doing a strafe would be reasonable valid
        let dir = if (rand() & 1) != 0 { -1 } else { 1 };
        let mut end = [0.0; 3];
        VectorMA(NPC.r.currentOrigin, (HUNTER_STRAFE_DIS * dir) as f32, right, end.as_mut_ptr());

        let mut tr = trace_t::default();
        trap::Trace(
            &mut tr,
            NPC.r.currentOrigin,
            None,
            None,
            end,
            NPC.s.number as c_int,
            MASK_SOLID,
        );

        // Close enough
        if tr.fraction > 0.9 {
            VectorMA(NPC.client.ps.velocity, (HUNTER_STRAFE_VEL * dir) as f32, right, NPC.client.ps.velocity.as_mut_ptr());

            // Add a slight upward push
            NPC.client.ps.velocity[2] += HUNTER_UPWARD_PUSH as f32;

            // Set the strafe start time so we can do a controlled roll
            NPCInfo.standTime = level.time + 3000 + (random() as i32 * 500);
        }
    }
}

/// Raven `ImperialProbe_Hunt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:220-261`
pub fn ImperialProbe_Hunt(visible: qboolean, advance: qboolean) {
    unsafe {
        NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_RUN1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);

        // If we're not supposed to stand still, pursue the player
        if NPCInfo.standTime < level.time {
            // Only strafe when we can see the player
            if visible != qfalse {
                ImperialProbe_Strafe();
                return;
            }
        }
    }

    // If we don't want to advance, stop here
    if advance == qfalse {
        return;
    }

    let mut forward = [0.0; 3];
    let mut distance = 0.0;

    unsafe {
        // Only try and navigate if the player is visible
        if visible == qfalse {
            // Move towards our goal
            NPCInfo.goalEntity = NPC.enemy;
            NPCInfo.goalRadius = 12;

            // Get our direction from the navigator if we can't see our target
            if NPC_GetMoveDirection(forward.as_mut_ptr(), &mut distance) == qfalse {
                return;
            }
        } else {
            if let Some(enemy) = NPC.enemy {
                VectorSubtract(enemy.r.currentOrigin, NPC.r.currentOrigin, forward.as_mut_ptr());
                distance = VectorNormalize(forward.as_mut_ptr());
            }
        }

        let speed = (HUNTER_FORWARD_BASE_SPEED + HUNTER_FORWARD_MULTIPLIER * g_spskill.integer) as f32;
        VectorMA(NPC.client.ps.velocity, speed, forward, NPC.client.ps.velocity.as_mut_ptr());
    }
}

/// Raven `ImperialProbe_FireBlaster`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:268-324`
pub fn ImperialProbe_FireBlaster() {
    unsafe {
        let gen_bolt1 = trap::G2API_AddBolt(NPC.ghoul2, 0, b"*flash\0".as_ptr() as *const c_char);

        let mut bolt_matrix = mdxaBone_t::default();
        trap::G2API_GetBoltMatrix(
            NPC.ghoul2,
            0,
            gen_bolt1,
            &mut bolt_matrix,
            NPC.r.currentAngles,
            NPC.r.currentOrigin,
            level.time,
            None,
            NPC.modelScale,
        );

        let mut muzzle1 = [0.0; 3];
        BG_GiveMeVectorFromMatrix(&bolt_matrix, ORIGIN, muzzle1.as_mut_ptr());

        G_PlayEffectID(
            G_EffectIndex(b"bryar/muzzle_flash\0".as_ptr() as *const c_char),
            muzzle1,
            vec3_origin,
        );

        G_Sound(NPC, CHAN_AUTO, G_SoundIndex(b"sound/chars/probe/misc/fire\0".as_ptr() as *const c_char));

        let mut forward = [0.0; 3];
        let mut vright = [0.0; 3];
        let mut up = [0.0; 3];

        if NPC.health > 0 {
            if let Some(enemy) = NPC.enemy {
                let mut enemy_org1 = [0.0; 3];
                CalcEntitySpot(enemy, SPOT_CHEST, enemy_org1.as_mut_ptr());
                enemy_org1[0] += Q_irand(0, 10) as f32;
                enemy_org1[1] += Q_irand(0, 10) as f32;

                let mut delta1 = [0.0; 3];
                VectorSubtract(enemy_org1, muzzle1, delta1.as_mut_ptr());

                let mut angle_to_enemy = [0.0; 3];
                vectoangles(delta1, angle_to_enemy.as_mut_ptr());
                AngleVectors(angle_to_enemy, forward.as_mut_ptr(), vright.as_mut_ptr(), up.as_mut_ptr());
            }
        } else {
            AngleVectors(NPC.r.currentAngles, forward.as_mut_ptr(), vright.as_mut_ptr(), up.as_mut_ptr());
        }

        if let Some(missile) = CreateMissile(muzzle1, forward, 1600.0, 10000, NPC, qfalse) {
            missile.classname = b"bryar_proj\0".as_ptr() as *mut c_char;
            missile.s.weapon = WP_BRYAR_PISTOL as c_int;

            missile.damage = if g_spskill.integer <= 1 { 5 } else { 10 };

            missile.dflags = DAMAGE_DEATH_KNOCKBACK;
            missile.methodOfDeath = MOD_UNKNOWN;
            missile.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
        }
    }
}

/// Raven `ImperialProbe_Ranged`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:331-363`
pub fn ImperialProbe_Ranged(visible: qboolean, advance: qboolean) {
    unsafe {
        if TIMER_Done(NPC, b"attackDelay\0".as_ptr() as *const c_char) != qfalse {
            let (_delay_min, _delay_max) = if g_spskill.integer == 0 {
                (500, 3000)
            } else if g_spskill.integer > 1 {
                (500, 2000)
            } else {
                (300, 1500)
            };

            TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(500, 3000));
            ImperialProbe_FireBlaster();
        }

        if (NPCInfo.scriptFlags & SCF_CHASE_ENEMIES) != 0 {
            ImperialProbe_Hunt(visible, advance);
        }
    }
}

/// Raven `ImperialProbe_AttackDecision`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:377-426`
pub fn ImperialProbe_AttackDecision() {
    // Always keep a good height off the ground
    ImperialProbe_MaintainHeight();

    // Randomly talk
    unsafe {
        if TIMER_Done(NPC, b"patrolNoise\0".as_ptr() as *const c_char) != qfalse {
            if TIMER_Done(NPC, b"angerNoise\0".as_ptr() as *const c_char) != qfalse {
                let path = va(b"sound/chars/probe/misc/probetalk%d\0".as_ptr() as *const c_char, Q_irand(1, 3));
                G_SoundOnEnt(NPC, CHAN_AUTO, path);

                TIMER_Set(NPC, b"patrolNoise\0".as_ptr() as *const c_char, Q_irand(4000, 10000));
            }
        }

        // If we don't have an enemy, just idle
        if NPC_CheckEnemyExt(qfalse) == qfalse {
            ImperialProbe_Idle();
            return;
        }

        NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_RUN1, SETANIM_FLAG_NORMAL);

        // Rate our distance to the target, and our visibility
        if let Some(enemy) = NPC.enemy {
            let distance = DistanceHorizontalSquared(NPC.r.currentOrigin, enemy.r.currentOrigin);
            let visible = NPC_ClearLOS4(enemy);
            let advance = (distance > (MIN_DISTANCE_SQR as f32)) as qboolean;

            // If we cannot see our target, move to see it
            if visible == qfalse {
                if (NPCInfo.scriptFlags & SCF_CHASE_ENEMIES) != 0 {
                    ImperialProbe_Hunt(visible, advance);
                    return;
                }
            }

            // Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
            NPC_FaceEnemy(qtrue);

            // Decide what type of attack to do
            ImperialProbe_Ranged(visible, advance);
        }
    }
}

/// Raven `NPC_Probe_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:433-498`
pub fn NPC_Probe_Pain(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    unsafe {
        let self_ent = &mut *self_;
        let attacker_ent = &*attacker;

        VectorCopy(self_ent.NPC.lastPathAngles, self_ent.s.angles.as_mut_ptr());

        if self_ent.health < 30 || gPainMOD == MOD_DEMP2 || gPainMOD == MOD_DEMP2_ALT {
            let mut end_pos = [0.0; 3];
            end_pos[0] = self_ent.r.currentOrigin[0];
            end_pos[1] = self_ent.r.currentOrigin[1];
            end_pos[2] = self_ent.r.currentOrigin[2] - 128.0;

            let mut trace = trace_t::default();
            trap::Trace(
                &mut trace,
                self_ent.r.currentOrigin,
                None,
                None,
                end_pos,
                self_ent.s.number as c_int,
                MASK_SOLID,
            );

            if trace.fraction == 1.0 || gPainMOD == MOD_DEMP2 {
                if (gPainMOD == MOD_DEMP2 || gPainMOD == MOD_DEMP2_ALT) && !attacker.is_null() {
                    let mut dir = [0.0; 3];
                    VectorSubtract(self_ent.r.currentOrigin, attacker_ent.r.currentOrigin, dir.as_mut_ptr());
                    VectorNormalize(dir.as_mut_ptr());

                    NPC_SetAnim(self_ent, SETANIM_BOTH, BOTH_PAIN1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);

                    VectorMA(self_ent.client.ps.velocity, 550.0, dir, self_ent.client.ps.velocity.as_mut_ptr());
                    self_ent.client.ps.velocity[2] -= 127.0;
                }

                self_ent.client.ps.electrifyTime = level.time + 3000;
                self_ent.NPC.localState = LSTATE_DROP;
            }
        } else {
            let pain_chance = NPC_GetPainChance(self_ent, damage);

            if (random() as f32) < pain_chance {
                NPC_SetAnim(self_ent, SETANIM_BOTH, BOTH_PAIN1, SETANIM_FLAG_OVERRIDE);
            }
        }

        NPC_Pain(self_ent, &mut *attacker, damage);
    }
}

/// Raven `ImperialProbe_Idle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:506-511`
pub fn ImperialProbe_Idle() {
    ImperialProbe_MaintainHeight();
    NPC_BSIdle();
}

/// Raven `ImperialProbe_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:518-556`
pub fn ImperialProbe_Patrol() {
    ImperialProbe_MaintainHeight();

    if NPC_CheckPlayerTeamStealth() != qfalse {
        NPC_UpdateAngles(qtrue, qtrue);
        return;
    }

    unsafe {
        // If we have somewhere to go, then do that
        if NPC.enemy.is_none() {
            NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_RUN1, SETANIM_FLAG_NORMAL);

            if !UpdateGoal().is_null() {
                // Start loop sound once we move
                NPC.s.loopSound = G_SoundIndex(b"sound/chars/probe/misc/probedroidloop\0".as_ptr() as *const c_char);
                ucmd.buttons |= BUTTON_WALKING;
                NPC_MoveToGoal(qtrue);
            }

            // Randomly talk
            if TIMER_Done(NPC, b"patrolNoise\0".as_ptr() as *const c_char) != qfalse {
                let path = va(b"sound/chars/probe/misc/probetalk%d\0".as_ptr() as *const c_char, Q_irand(1, 3));
                G_SoundOnEnt(NPC, CHAN_AUTO, path);

                TIMER_Set(NPC, b"patrolNoise\0".as_ptr() as *const c_char, Q_irand(2000, 4000));
            }
        } else {
            // He's got an enemy. Make him angry.
            G_SoundOnEnt(NPC, CHAN_AUTO, b"sound/chars/probe/misc/anger1\0".as_ptr() as *const c_char);
            TIMER_Set(NPC, b"angerNoise\0".as_ptr() as *const c_char, Q_irand(2000, 4000));
        }

        NPC_UpdateAngles(qtrue, qtrue);
    }
}

/// Raven `ImperialProbe_Wait`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:563-582`
pub fn ImperialProbe_Wait() {
    unsafe {
        if NPCInfo.localState == LSTATE_DROP {
            let mut end_pos = [0.0; 3];
            end_pos[0] = NPC.r.currentOrigin[0];
            end_pos[1] = NPC.r.currentOrigin[1];
            end_pos[2] = NPC.r.currentOrigin[2] - 32.0;

            let mut trace = trace_t::default();
            trap::Trace(
                &mut trace,
                NPC.r.currentOrigin,
                None,
                None,
                end_pos,
                NPC.s.number as c_int,
                MASK_SOLID,
            );

            if trace.fraction != 1.0 {
                NPCInfo.desiredYaw = AngleNormalize360(NPCInfo.desiredYaw + 25.0);

                if let Some(enemy) = NPC.enemy {
                    G_Damage(NPC, NPC, enemy, std::ptr::null_mut(), std::ptr::null_mut(), 2000, 0, MOD_UNKNOWN);
                }
            }
        }

        NPC_UpdateAngles(qtrue, qtrue);
    }
}

/// Raven `NPC_BSImperialProbe_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:589-609`
pub fn NPC_BSImperialProbe_Default() {
    unsafe {
        if NPC.enemy.is_some() {
            NPCInfo.goalEntity = NPC.enemy;
            ImperialProbe_AttackDecision();
        } else if (NPCInfo.scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            ImperialProbe_Patrol();
        } else if NPCInfo.localState == LSTATE_DROP {
            ImperialProbe_Wait();
        } else {
            ImperialProbe_Idle();
        }
    }
}
