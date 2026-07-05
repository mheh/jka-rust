//! NPC AI Default behavior states for `oracle/oracle/codemp/game/NPC_AI_Default.c`.
//!
//! Pass-3 port: 15/15 functions transcribed from oracle source with settled
//! rulings 12-22. Game-tier functions thread `ctx: GameContext<'_>` as first param;
//! reach ambient AI state through `(*ctx.world).globals.*` (NPC, NPCInfo, client,
//! ucmd, enemyVisibility).
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::trap;

/// Raven `NPC_LostEnemyDecideChase`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Default.c:18-35`
pub fn NPC_LostEnemyDecideChase(ctx: GameContext<'_>) {
    let world = unsafe { &mut *ctx.world };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };
    let npc = unsafe { &mut *(*world).globals.NPC };

    match npc_info.behaviorState {
        BS_HUNT_AND_KILL => {
            if npc.enemy.is_some() && npc.lastWaypoint != WAYPOINT_NONE {
                if let Some(enemy_id) = npc.enemy {
                    let enemy = unsafe { &*(*world).g_entities.as_ptr().add(enemy_id.0 as usize) };
                    if unsafe { &*enemy }.lastWaypoint != WAYPOINT_NONE {
                        NPC_BSSearchStart(ctx, unsafe { &*enemy }.lastWaypoint, BS_SEARCH);
                    }
                }
            }
        }
        _ => {}
    }

    G_ClearEnemy(ctx, (*world).globals.NPC);
}

/// Raven `NPC_StandIdle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Default.c:42-85`
pub fn NPC_StandIdle() {
    // Function is completely commented out in oracle source. Port as no-op.
}

/// Raven `NPC_StandTrackAndShoot`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Default.c:87-158`
pub fn NPC_StandTrackAndShoot(
    ctx: GameContext<'_>,
    NPC: *mut gentity_t,
    canDuck: qboolean,
) -> qboolean {
    let world = unsafe { &mut *ctx.world };
    let npc = unsafe { &mut *NPC };
    let client = unsafe { &mut *(*world).globals.client };

    let mut attack_ok = false;
    let mut duck_ok = false;
    let mut attack_scale = 1.0;

    if canDuck != 0 {
        if npc.health < 20 {
            if (*world).bg_state.rng.random() > 0.0 {
                duck_ok = true;
            }
        }
    }

    if !duck_ok {
        attack_ok = NPC_CheckCanAttack(ctx, attack_scale, qtrue) != 0;
    }

    if canDuck != 0 && (duck_ok || (!attack_ok && client.ps.weaponTime <= 0)) && (*world).globals.ucmd.upmove != -127 {
        if !duck_ok {
            if let Some(enemy_id) = npc.enemy {
                let enemy = unsafe { &*(*world).g_entities.as_ptr().add(enemy_id.0 as usize) };
                if let Some(enemy_enemy) = unsafe { &*enemy }.enemy {
                    if enemy_enemy.0 == npc.s.number as u32 {
                        if (unsafe { &*(enemy.client as *mut gclient_t) }.buttons & BUTTON_ATTACK as i32) != 0 {
                            if NPC_CheckDefend(ctx, 1.0) != 0 {
                                duck_ok = true;
                            }
                        }
                    }
                }
            }
        }

        if duck_ok {
            attack_ok = false;
            unsafe { &mut *ctx.world }.globals.ucmd.upmove = -127;
            unsafe { &mut *(*world).globals.NPCInfo }.duckDebounceTime = (*world).level.time + 1000;
        }
    }

    false as qboolean
}

/// Raven `NPC_BSIdle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Default.c:161-177`
pub fn NPC_BSIdle(ctx: GameContext<'_>) {
    let world = unsafe { &mut *ctx.world };

    if UpdateGoal(ctx) != core::ptr::null_mut() {
        NPC_MoveToGoal(ctx, qtrue);
    }

    if (*world).globals.ucmd.forwardmove == 0 && (*world).globals.ucmd.rightmove == 0 && (*world).globals.ucmd.upmove == 0 {
        // NPC_StandIdle(); - commented out in oracle
    }

    NPC_UpdateAngles(ctx, qtrue, qtrue);
    unsafe { &mut *ctx.world }.globals.ucmd.buttons |= BUTTON_WALKING as i32;
}

/// Raven `NPC_BSRun`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Default.c:179-189`
pub fn NPC_BSRun(ctx: GameContext<'_>) {
    if UpdateGoal(ctx) != core::ptr::null_mut() {
        NPC_MoveToGoal(ctx, qtrue);
    }

    NPC_UpdateAngles(ctx, qtrue, qtrue);
}

/// Raven `NPC_BSStandGuard`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Default.c:191-224`
pub fn NPC_BSStandGuard(ctx: GameContext<'_>) {
    let world = unsafe { &mut *ctx.world };
    let npc = unsafe { &mut *(*world).globals.NPC };
    let client = unsafe { &mut *(*world).globals.client };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };

    if npc.enemy.is_none() {
        if (*world).bg_state.rng.random() < 0.5 {
            if client.enemyTeam != 0 {
                let new_enemy = NPC_PickEnemy(
                    ctx,
                    (*world).globals.NPC,
                    client.enemyTeam as c_int,
                    (npc.cantHitEnemyCounter < 10) as qboolean,
                    (client.enemyTeam as c_int == NPCTEAM_PLAYER) as qboolean,
                    qtrue,
                );
                if !new_enemy.is_null() {
                    G_SetEnemy(ctx, (*world).globals.NPC, new_enemy);
                }
            }
        }
    }

    if npc.enemy.is_some() {
        if npc_info.tempBehavior == BS_STAND_GUARD {
            npc_info.tempBehavior = BS_DEFAULT;
        }

        if npc_info.behaviorState == BS_STAND_GUARD {
            npc_info.behaviorState = BS_STAND_AND_SHOOT;
        }
    }

    NPC_UpdateAngles(ctx, qtrue, qtrue);
}

/// Raven `NPC_BSHuntAndKill`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Default.c:232-304`
pub fn NPC_BSHuntAndKill(ctx: GameContext<'_>) {
    let world = unsafe { &mut *ctx.world };
    let npc = unsafe { &mut *(*world).globals.NPC };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };

    let mut turned = false;
    let mut vec = [0.0; 3];
    let mut enemy_dist;
    let o_evis;

    NPC_CheckEnemy(ctx, (npc_info.tempBehavior != BS_HUNT_AND_KILL) as qboolean, qfalse, qtrue);

    if let Some(enemy_id) = npc.enemy {
        let enemy = unsafe { &*(*world).g_entities.as_ptr().add(enemy_id.0 as usize) };
        o_evis = NPC_CheckVisibility(ctx, enemy as *const _ as *mut _, CHECK_FOV | CHECK_SHOOT);
        (*world).globals.enemyVisibility = o_evis;

        if o_evis as i32 > VIS_PVS as i32 {
            if NPC_EnemyTooFar(ctx, enemy as *const _ as *mut _, 0.0, qtrue) == 0 {
                NPC_CheckCanAttack(ctx, 1.0, qfalse);
                turned = true;
            }
        }

        let cur_anim = unsafe { &*(*world).globals.client }.ps.legsAnim;
        if cur_anim as i32 != BOTH_ATTACK1 as i32 && cur_anim as i32 != BOTH_ATTACK2 as i32 && cur_anim as i32 != BOTH_ATTACK3 as i32
            && cur_anim as i32 != BOTH_MELEE1 as i32 && cur_anim as i32 != BOTH_MELEE2 as i32 {

            crate::q_math::_VectorSubtract(unsafe { &*enemy }.r.currentOrigin, npc.r.currentOrigin, &mut vec);
            enemy_dist = crate::q_math::VectorLength(vec);

            if enemy_dist > 48.0 && ((enemy_dist * 1.5) * (enemy_dist * 1.5) >= NPC_MaxDistSquaredForWeapon(ctx) ||
                o_evis != VIS_SHOOT ||
                enemy_dist > IdealDistance(ctx, (*world).globals.NPC) * 3.0) {

                npc_info.goalEntity = npc.enemy;
                NPC_MoveToGoal(ctx, qtrue);
            } else if enemy_dist < IdealDistance(ctx, (*world).globals.NPC) {
                npc_info.goalEntity = npc.enemy;
                npc_info.goalRadius = 12;
                NPC_MoveToGoal(ctx, qtrue);

                unsafe { &mut *ctx.world }.globals.ucmd.forwardmove *= -1;
                unsafe { &mut *ctx.world }.globals.ucmd.rightmove *= -1;
                crate::q_math::_VectorScale(unsafe { &mut *(*world).globals.client }.ps.moveDir, -1.0, &mut unsafe { &mut *(*world).globals.client }.ps.moveDir);

                unsafe { &mut *ctx.world }.globals.ucmd.buttons |= BUTTON_WALKING as i32;
            }
        }
    } else {
        if npc_info.tempBehavior == BS_HUNT_AND_KILL {
            npc_info.tempBehavior = BS_DEFAULT;
        } else {
            npc_info.tempBehavior = BS_STAND_GUARD;
            NPC_BSStandGuard(ctx);
        }
        return;
    }

    if !turned {
        NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}

/// Raven `NPC_BSStandAndShoot`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Default.c:306-392`
pub fn NPC_BSStandAndShoot(ctx: GameContext<'_>) {
    let world = unsafe { &mut *ctx.world };
    let npc = unsafe { &mut *(*world).globals.NPC };
    let client = unsafe { &mut *(*world).globals.client };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };

    if client.playerTeam != 0 && client.enemyTeam != 0 {
        // many commented-out checks in oracle
    }

    NPC_CheckEnemy(ctx, qtrue, qfalse, qtrue);

    if npc_info.duckDebounceTime > (*world).level.time && client.ps.weapon as i32 != WP_SABER as i32 {
        unsafe { &mut *ctx.world }.globals.ucmd.upmove = -127;
        if npc.enemy.is_some() {
            NPC_CheckCanAttack(ctx, 1.0, qtrue);
        }
        return;
    }

    if npc.enemy.is_some() {
        if NPC_StandTrackAndShoot(ctx, (*world).globals.NPC, qtrue) == 0 {
            npc_info.desiredYaw = client.ps.viewangles[YAW];
            npc_info.desiredPitch = client.ps.viewangles[PITCH];
            NPC_UpdateAngles(ctx, qtrue, qtrue);
        }
    } else {
        npc_info.desiredYaw = client.ps.viewangles[YAW];
        npc_info.desiredPitch = client.ps.viewangles[PITCH];
        NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}

/// Raven `NPC_BSRunAndShoot`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Default.c:394-487`
pub fn NPC_BSRunAndShoot(ctx: GameContext<'_>) {
    let world = unsafe { &mut *ctx.world };
    let npc = unsafe { &mut *(*world).globals.NPC };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };

    NPC_CheckEnemy(ctx, qtrue, qfalse, qtrue);

    if npc_info.duckDebounceTime > (*world).level.time {
        unsafe { &mut *ctx.world }.globals.ucmd.upmove = -127;
        if npc.enemy.is_some() {
            NPC_CheckCanAttack(ctx, 1.0, qfalse);
        }
        return;
    }

    if npc.enemy.is_some() {
        let monitor = npc.cantHitEnemyCounter;
        NPC_StandTrackAndShoot(ctx, (*world).globals.NPC, qfalse);

        if (unsafe { &*ctx.world }.globals.ucmd.buttons & BUTTON_ATTACK as i32) == 0 && unsafe { &*ctx.world }.globals.ucmd.upmove >= 0 && npc.cantHitEnemyCounter > monitor {
            let mut vec = [0.0; 3];

            if let Some(enemy_id) = npc.enemy {
                let enemy = unsafe { &*(*world).g_entities.as_ptr().add(enemy_id.0 as usize) };
                crate::q_math::_VectorSubtract(unsafe { &*enemy }.r.currentOrigin, npc.r.currentOrigin, &mut vec);
                vec[2] = 0.0;

                if crate::q_math::VectorLength(vec) > 128.0 || npc.cantHitEnemyCounter >= 10 {
                    if npc.cantHitEnemyCounter > 60 {
                        npc.cantHitEnemyCounter = 60;
                    }

                    if npc.cantHitEnemyCounter >= (npc_info.stats.aggression + 1) * 10 {
                        NPC_LostEnemyDecideChase(ctx);
                    }

                    unsafe { &mut *ctx.world }.globals.ucmd.angles[YAW] = 0;
                    unsafe { &mut *ctx.world }.globals.ucmd.angles[PITCH] = 0;
                    npc_info.goalEntity = npc.enemy;
                    npc_info.goalRadius = 12;
                    NPC_MoveToGoal(ctx, qtrue);
                    NPC_UpdateAngles(ctx, qtrue, qtrue);
                }
            }
        } else {
            npc.cantHitEnemyCounter = 0;
        }
    } else {
        if npc_info.tempBehavior == BS_HUNT_AND_KILL {
            npc_info.tempBehavior = BS_DEFAULT;
            return;
        }
    }
}

/// Raven `NPC_BSFace`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Default.c:490-503`
pub fn NPC_BSFace(ctx: GameContext<'_>) {
    let world = unsafe { &mut *ctx.world };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };
    let client = unsafe { &mut *(*world).globals.client };

    if NPC_UpdateAngles(ctx, qtrue, qtrue) != 0 {
        trap::ICARUS_TaskIDComplete(
            ctx.engine,
            mp_abi::game::syscalls::G_ICARUS_TASKIDCOMPLETE::GIcarusTaskidcompleteArgs::new(
                (*world).globals.NPC,
                TID_BSTATE as c_int,
            ),
        );

        npc_info.desiredYaw = client.ps.viewangles[YAW];
        npc_info.desiredPitch = client.ps.viewangles[PITCH];

        npc_info.aimTime = 0;
    }
}

/// Raven `NPC_BSPointShoot`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Default.c:505-610`
pub fn NPC_BSPointShoot(ctx: GameContext<'_>, shoot: qboolean) {
    let world = unsafe { &mut *ctx.world };
    let npc = unsafe { &mut *(*world).globals.NPC };
    let client = unsafe { &mut *(*world).globals.client };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };

    let mut muzzle = [0.0; 3];
    let mut dir = [0.0; 3];
    let mut angles = [0.0; 3];
    let mut org = [0.0; 3];

    if npc.enemy.is_none() {
        trap::ICARUS_TaskIDComplete(
            ctx.engine,
            mp_abi::game::syscalls::G_ICARUS_TASKIDCOMPLETE::GIcarusTaskidcompleteArgs::new(
                (*world).globals.NPC,
                TID_BSTATE as c_int,
            ),
        );
        return;
    }

    if let Some(enemy_id) = npc.enemy {
        let enemy = unsafe { &*(*world).g_entities.as_ptr().add(enemy_id.0 as usize) };
        if (unsafe { &*enemy }.inuse as qboolean) == 0 || (unsafe { &*enemy }.NPC != core::ptr::null_mut() && unsafe { &*enemy }.health <= 0) {
            trap::ICARUS_TaskIDComplete(
                ctx.engine,
                mp_abi::game::syscalls::G_ICARUS_TASKIDCOMPLETE::GIcarusTaskidcompleteArgs::new(
                    (*world).globals.NPC,
                    TID_BSTATE as c_int,
                ),
            );
            return;
        }
    }

    CalcEntitySpot(ctx, (*world).globals.NPC, SPOT_WEAPON, &mut muzzle);
    if let Some(enemy_id) = npc.enemy {
        let enemy = unsafe { &*(*world).g_entities.as_ptr().add(enemy_id.0 as usize) };
        CalcEntitySpot(ctx, enemy as *const _ as *mut _, SPOT_HEAD, &mut org);

        if !unsafe { &*enemy }.client.is_null() {
            org[2] -= 12.0;
        }
    }

    crate::q_math::_VectorSubtract(org, muzzle, &mut dir);
    crate::q_math::vectoangles(dir, &mut angles);

    match client.ps.weapon as i32 {
        x if x == WP_NONE as i32 || x == WP_STUN_BATON as i32 || x == WP_SABER as i32 => {
            // don't change pitch
        }
        _ => {
            let val = crate::q_math::AngleNormalize360(angles[PITCH]);
            npc_info.desiredPitch = val;
            npc_info.lockedDesiredPitch = val;
        }
    }

    let val = crate::q_math::AngleNormalize360(angles[YAW]);
    npc_info.desiredYaw = val;
    npc_info.lockedDesiredYaw = val;

    if NPC_UpdateAngles(ctx, qtrue, qtrue) != 0 {
        if shoot != 0 {
            unsafe { &mut *ctx.world }.globals.ucmd.buttons |= BUTTON_ATTACK as i32;
        }

        trap::ICARUS_TaskIDComplete(
            ctx.engine,
            mp_abi::game::syscalls::G_ICARUS_TASKIDCOMPLETE::GIcarusTaskidcompleteArgs::new(
                (*world).globals.NPC,
                TID_BSTATE as c_int,
            ),
        );
    }
}

/// Raven `NPC_BSMove`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Default.c:616-637`
pub fn NPC_BSMove(ctx: GameContext<'_>) {
    let world = unsafe { &mut *ctx.world };
    let npc = unsafe { &mut *(*world).globals.NPC };

    NPC_CheckEnemy(ctx, qtrue, qfalse, qtrue);
    if npc.enemy.is_some() {
        NPC_CheckCanAttack(ctx, 1.0, qfalse);
    } else {
        NPC_UpdateAngles(ctx, qtrue, qtrue);
    }

    if UpdateGoal(ctx) != core::ptr::null_mut() {
        NPC_SlideMoveToGoal(ctx);
    }
}

/// Raven `NPC_BSShoot`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Default.c:644-656`
pub fn NPC_BSShoot(ctx: GameContext<'_>) {
    let world = unsafe { &mut *ctx.world };
    let client = unsafe { &mut *(*world).globals.client };

    (*world).globals.enemyVisibility = VIS_SHOOT;

    if client.ps.weaponstate as i32 != WEAPON_READY as i32 && client.ps.weaponstate as i32 != WEAPON_FIRING as i32 {
        unsafe { (*(*ctx.world).globals.client).ps.weaponstate = WEAPON_READY };
    }

    WeaponThink(ctx, qtrue);
}

/// Raven `NPC_BSPatrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Default.c:664-703`
pub fn NPC_BSPatrol(ctx: GameContext<'_>) {
    let world = unsafe { &mut *ctx.world };
    let npc = unsafe { &mut *(*world).globals.NPC };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };

    if (*world).level.time > npc_info.enemyCheckDebounceTime {
        npc_info.enemyCheckDebounceTime = (*world).level.time + (npc_info.stats.vigilance * 1000.0) as c_int;
        NPC_CheckEnemy(ctx, qtrue, qfalse, qtrue);
        if npc.enemy.is_some() {
            npc_info.behaviorState = BS_HUNT_AND_KILL;
            return;
        }
    }

    npc_info.investigateSoundDebounceTime = 0;

    if UpdateGoal(ctx) != core::ptr::null_mut() {
        NPC_MoveToGoal(ctx, qtrue);
    }

    NPC_UpdateAngles(ctx, qtrue, qtrue);

    unsafe { &mut *ctx.world }.globals.ucmd.buttons |= BUTTON_WALKING as i32;
}

/// Raven `NPC_BSDefault`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Default.c:712-957`
pub fn NPC_BSDefault(ctx: GameContext<'_>) {
    let world = unsafe { &mut *ctx.world };
    let npc = unsafe { &mut *(*world).globals.NPC };
    let client = unsafe { &mut *(*world).globals.client };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };

    let mut move_ = true;

    if (npc_info.scriptFlags & SCF_FIRE_WEAPON) != 0 {
        WeaponThink(ctx, qtrue);
    }

    if (npc_info.scriptFlags & SCF_FORCED_MARCH) != 0 {
        if client.ps.torsoAnim as i32 != TORSO_SURRENDER_START as i32 {
            NPC_SetAnim((*world).globals.NPC, SETANIM_TORSO as i32, TORSO_SURRENDER_START as i32, SETANIM_FLAG_HOLD as i32);
        }
    }

    NPC_CheckEnemy(ctx, ((npc_info.scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0) as qboolean, qfalse, qtrue);

    if npc.enemy.is_none() {
        if (npc_info.scriptFlags & SCF_IGNORE_ALERTS) == 0 {
            let alert_event = NPC_CheckAlertEvents(ctx, qtrue, qtrue, -1, qtrue, AEL_DISCOVERED as i32);

            if alert_event >= 0 {
                let alert_entry = unsafe { &(*world).level.alertEvents[alert_event as usize] };
                if alert_entry.ID != npc_info.lastAlertID && alert_entry.level as i32 >= AEL_DISCOVERED as i32 && (npc_info.scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
                    if !alert_entry.owner.is_null() {
                        let alert_owner = unsafe { &*alert_entry.owner };
                        if !alert_owner.client.is_null() && alert_owner.health >= 0 {
                            if unsafe { &*(alert_owner.client as *mut gclient_t) }.playerTeam == client.enemyTeam {
                                G_SetEnemy(ctx, (*world).globals.NPC, alert_entry.owner);
                            }
                        }
                    }
                }
            }
        }
    }

    if npc.enemy.is_some() && (npc_info.scriptFlags & SCF_FORCED_MARCH) == 0 {
        NPC_CheckGetNewWeapon(ctx);
        if !client.leader.is_none()
            && npc_info.goalEntity == client.leader
            && trap::ICARUS_TaskIDPending(
                ctx.engine,
                mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs::new(
                    (*world).globals.NPC,
                    TID_MOVE_NAV as c_int,
                ),
            ) == 0 {
            NPC_ClearGoal(ctx);
        }
        NPC_BSST_Attack(ctx);
        return;
    }

    if UpdateGoal(ctx) != core::ptr::null_mut() {
        if npc.enemy.is_none()
            && !client.leader.is_none()
            && npc_info.goalEntity == client.leader
            && trap::ICARUS_TaskIDPending(
                ctx.engine,
                mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs::new(
                    (*world).globals.NPC,
                    TID_MOVE_NAV as c_int,
                ),
            ) == 0 {
            NPC_BSFollowLeader(ctx);
        } else {
            if (npc_info.scriptFlags & SCF_FACE_MOVE_DIR) != 0 || npc_info.goalEntity != npc.enemy {
                npc_info.combatMove = 0;
            } else {
                let mut dir = [0.0; 3];
                let mut angles = [0.0; 3];

                npc_info.combatMove = 0;

                if let Some(goal_id) = npc_info.goalEntity {
                    let goal_entity = unsafe { &*(*world).g_entities.as_ptr().add(goal_id.0 as usize) };
                    crate::q_math::_VectorSubtract(unsafe { &*goal_entity }.r.currentOrigin, npc.r.currentOrigin, &mut dir);
                    crate::q_math::vectoangles(dir, &mut angles);
                    npc_info.desiredYaw = angles[YAW];
                    if npc_info.goalEntity == npc.enemy {
                        npc_info.desiredPitch = angles[PITCH];
                    }
                }
            }

            if (npc_info.scriptFlags & SCF_RUNNING) != 0 {
                unsafe { &mut *ctx.world }.globals.ucmd.buttons &= !(BUTTON_WALKING as i32);
            } else if (npc_info.scriptFlags & SCF_WALKING) != 0 {
                unsafe { &mut *ctx.world }.globals.ucmd.buttons |= BUTTON_WALKING as i32;
            } else if npc_info.goalEntity == npc.enemy {
                unsafe { &mut *ctx.world }.globals.ucmd.buttons &= !(BUTTON_WALKING as i32);
            } else {
                unsafe { &mut *ctx.world }.globals.ucmd.buttons |= BUTTON_WALKING as i32;
            }

            if (npc_info.scriptFlags & SCF_FORCED_MARCH) != 0 {
                if NPC_SomeoneLookingAtMe(ctx, (*world).globals.NPC) == 0 {
                    move_ = false;
                }
            }

            if move_ {
                NPC_MoveToGoal(ctx, qtrue);
            }
        }
    } else if npc.enemy.is_none() && !client.leader.is_none() {
        NPC_BSFollowLeader(ctx);
    }

    NPC_UpdateAngles(ctx, qtrue, qtrue);
}
