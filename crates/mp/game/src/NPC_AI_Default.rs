//! NPC AI Default behavior states for `oracle/codemp/game/NPC_AI_Default.c`.
//!
//! Pass-3 port: 15/15 functions transcribed from oracle source with settled
//! rulings 12-22. Game-tier functions thread `ctx: &mut GameContext` as first param;
//! reach ambient AI state through `(*ctx.world_raw()).globals.*` (NPC, NPCInfo, client,
//! ucmd, enemyVisibility).
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::trap;

/// Raven `NPC_LostEnemyDecideChase`.
///
/// Source: `oracle/codemp/game/NPC_AI_Default.c:18-35`
pub fn NPC_LostEnemyDecideChase(ctx: &mut GameContext) {
    let world = unsafe { &mut *ctx.world_raw() };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };
    let npc = unsafe { &mut *(*world).globals.NPC };

    let __h3 = ctx.entity_id_of((*world).globals.NPC).unwrap();
    match npc_info.behaviorState {
        BS_HUNT_AND_KILL => {
            // Oracle: `NPC->enemy == NPCInfo->goalEntity && NPC->enemy->lastWaypoint != WAYPOINT_NONE`.
            if let Some(enemy_id) = npc.enemy {
                if npc.enemy == npc_info.goalEntity {
                    let enemy = unsafe { &*(*world).g_entities.as_ptr().add(enemy_id.0 as usize) };
                    if enemy.lastWaypoint != WAYPOINT_NONE {
                        NPC_BSSearchStart(ctx, enemy.lastWaypoint, BS_SEARCH);
                    }
                }
            }
        }
        _ => {}
    }

    G_ClearEnemy(ctx, __h3);
}

/// Raven `NPC_StandIdle`.
///
/// Source: `oracle/codemp/game/NPC_AI_Default.c:42-85`
pub fn NPC_StandIdle() {
    // Function is completely commented out in oracle source. Port as no-op.
}

/// Raven `NPC_StandTrackAndShoot`.
///
/// Source: `oracle/codemp/game/NPC_AI_Default.c:87-158`
pub fn NPC_StandTrackAndShoot(ctx: &mut GameContext, NPC: EntityId, canDuck: qboolean) -> qboolean {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let NPC: *mut gentity_t = ctx.entity_mut(NPC);
    let world = unsafe { &mut *ctx.world_raw() };
    let npc = unsafe { &mut *NPC };
    let client = unsafe { &mut *(*world).globals.client };

    let mut attack_ok = false;
    let mut duck_ok = false;
    let mut faced = false;
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
        faced = true;
    }

    if canDuck != 0
        && (duck_ok || (!attack_ok && client.ps.weaponTime <= 0))
        && (*world).globals.ucmd.upmove != -127
    {
        if !duck_ok {
            if let Some(enemy_id) = npc.enemy {
                let enemy = unsafe { &*(*world).g_entities.as_ptr().add(enemy_id.0 as usize) };
                if !enemy.client.is_null() {
                    if let Some(enemy_enemy) = unsafe { &*enemy }.enemy {
                        if enemy_enemy.0 == npc.s.number as u32 {
                            if (unsafe { &*(enemy.client as *mut gclient_t) }.buttons
                                & BUTTON_ATTACK as i32)
                                != 0
                            {
                                if NPC_CheckDefend(ctx, 1.0) != 0 {
                                    duck_ok = true;
                                }
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

    faced as qboolean
}

/// Raven `NPC_BSIdle`.
///
/// Source: `oracle/codemp/game/NPC_AI_Default.c:161-177`
pub fn NPC_BSIdle(ctx: &mut GameContext) {
    let world = unsafe { &mut *ctx.world_raw() };

    if UpdateGoal(ctx) != core::ptr::null_mut() {
        NPC_MoveToGoal(ctx, qtrue);
    }

    if (*world).globals.ucmd.forwardmove == 0
        && (*world).globals.ucmd.rightmove == 0
        && (*world).globals.ucmd.upmove == 0
    {
        // NPC_StandIdle(); - commented out in oracle
    }

    NPC_UpdateAngles(ctx, qtrue, qtrue);
    unsafe { &mut *ctx.world }.globals.ucmd.buttons |= BUTTON_WALKING as i32;
}

/// Raven `NPC_BSRun`.
///
/// Source: `oracle/codemp/game/NPC_AI_Default.c:179-189`
pub fn NPC_BSRun(ctx: &mut GameContext) {
    if UpdateGoal(ctx) != core::ptr::null_mut() {
        NPC_MoveToGoal(ctx, qtrue);
    }

    NPC_UpdateAngles(ctx, qtrue, qtrue);
}

/// Raven `NPC_BSStandGuard`.
///
/// Source: `oracle/codemp/game/NPC_AI_Default.c:191-224`
pub fn NPC_BSStandGuard(ctx: &mut GameContext) {
    let world = unsafe { &mut *ctx.world_raw() };
    let npc = unsafe { &mut *(*world).globals.NPC };
    let client = unsafe { &mut *(*world).globals.client };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };

    if npc.enemy.is_none() {
        if (*world).bg_state.rng.random() < 0.5 {
            if client.enemyTeam != 0 {
                let __h4 = ctx.entity_id_of((*world).globals.NPC);
                let new_enemy = NPC_PickEnemy(
                    ctx,
                    __h4,
                    client.enemyTeam as c_int,
                    (npc.cantHitEnemyCounter < 10) as qboolean,
                    (client.enemyTeam as c_int == NPCTEAM_PLAYER) as qboolean,
                    qtrue,
                );
                if !new_enemy.is_null() {
                    let __h5 = ctx.entity_id_of((*world).globals.NPC).unwrap();
                    let __h6 = ctx.entity_id_of(new_enemy);
                    G_SetEnemy(ctx, __h5, __h6);
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
/// Source: `oracle/codemp/game/NPC_AI_Default.c:232-304`
pub fn NPC_BSHuntAndKill(ctx: &mut GameContext) {
    let world = unsafe { &mut *ctx.world_raw() };
    let npc = unsafe { &mut *(*world).globals.NPC };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };

    let mut turned = false;
    let mut vec = [0.0; 3];
    let mut enemy_dist;
    let o_evis;

    NPC_CheckEnemy(
        ctx,
        (npc_info.tempBehavior != BS_HUNT_AND_KILL) as qboolean,
        qfalse,
        qtrue,
    );

    if let Some(enemy_id) = npc.enemy {
        let enemy = unsafe { &*(*world).g_entities.as_ptr().add(enemy_id.0 as usize) };
        let __h7 = ctx.entity_id_of(enemy as *const _ as *mut _);
        o_evis = NPC_CheckVisibility(ctx, __h7, CHECK_FOV | CHECK_SHOOT);
        (*world).globals.enemyVisibility = o_evis;

        if o_evis as i32 > VIS_PVS as i32 {
            if NPC_EnemyTooFar(
                ctx,
                ctx.entity_id_of(enemy as *const _ as *mut _),
                0.0,
                qtrue,
            ) == 0
            {
                NPC_CheckCanAttack(ctx, 1.0, qfalse);
                turned = true;
            }
        }

        let cur_anim = unsafe { &*(*world).globals.client }.ps.legsAnim;
        if cur_anim as i32 != BOTH_ATTACK1 as i32
            && cur_anim as i32 != BOTH_ATTACK2 as i32
            && cur_anim as i32 != BOTH_ATTACK3 as i32
            && cur_anim as i32 != BOTH_MELEE1 as i32
            && cur_anim as i32 != BOTH_MELEE2 as i32
        {
            crate::q_math::_VectorSubtract(
                unsafe { &*enemy }.r.currentOrigin,
                npc.r.currentOrigin,
                &mut vec,
            );
            enemy_dist = crate::q_math::VectorLength(vec);

            // `1.5` is a double literal, so the scaled square is computed in f64
            // (the float weapon range promotes) before the comparison.
            if enemy_dist > 48.0
                && ((enemy_dist as f64 * 1.5) * (enemy_dist as f64 * 1.5)
                    >= NPC_MaxDistSquaredForWeapon(ctx) as f64
                    || o_evis != VIS_SHOOT
                    || enemy_dist
                        > IdealDistance(ctx, ctx.entity_id_of((*world).globals.NPC).unwrap()) * 3.0)
            {
                npc_info.goalEntity = npc.enemy;
                NPC_MoveToGoal(ctx, qtrue);
            } else if enemy_dist
                < IdealDistance(ctx, ctx.entity_id_of((*world).globals.NPC).unwrap())
            {
                npc_info.goalEntity = npc.enemy;
                npc_info.goalRadius = 12;
                NPC_MoveToGoal(ctx, qtrue);

                unsafe { &mut *ctx.world }.globals.ucmd.forwardmove *= -1;
                unsafe { &mut *ctx.world }.globals.ucmd.rightmove *= -1;
                crate::q_math::_VectorScale(
                    unsafe { &mut *(*world).globals.client }.ps.moveDir,
                    -1.0,
                    &mut unsafe { &mut *(*world).globals.client }.ps.moveDir,
                );

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
/// Source: `oracle/codemp/game/NPC_AI_Default.c:306-392`
pub fn NPC_BSStandAndShoot(ctx: &mut GameContext) {
    let world = unsafe { &mut *ctx.world_raw() };
    let npc = unsafe { &mut *(*world).globals.NPC };
    let client = unsafe { &mut *(*world).globals.client };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };

    if client.playerTeam != 0 && client.enemyTeam != 0 {
        // many commented-out checks in oracle
    }

    NPC_CheckEnemy(ctx, qtrue, qfalse, qtrue);

    if npc_info.duckDebounceTime > (*world).level.time && client.ps.weapon as i32 != WP_SABER as i32
    {
        unsafe { &mut *ctx.world }.globals.ucmd.upmove = -127;
        if npc.enemy.is_some() {
            NPC_CheckCanAttack(ctx, 1.0, qtrue);
        }
        return;
    }

    if npc.enemy.is_some() {
        if NPC_StandTrackAndShoot(ctx, ctx.entity_id_of((*world).globals.NPC).unwrap(), qtrue) == 0
        {
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
/// Source: `oracle/codemp/game/NPC_AI_Default.c:394-487`
pub fn NPC_BSRunAndShoot(ctx: &mut GameContext) {
    let world = unsafe { &mut *ctx.world_raw() };
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
        let __h8 = ctx.entity_id_of((*world).globals.NPC).unwrap();
        NPC_StandTrackAndShoot(ctx, __h8, qfalse);

        if (unsafe { &*ctx.world }.globals.ucmd.buttons & BUTTON_ATTACK as i32) == 0
            && unsafe { &*ctx.world }.globals.ucmd.upmove >= 0
            && npc.cantHitEnemyCounter > monitor
        {
            let mut vec = [0.0; 3];

            if let Some(enemy_id) = npc.enemy {
                let enemy = unsafe { &*(*world).g_entities.as_ptr().add(enemy_id.0 as usize) };
                crate::q_math::_VectorSubtract(
                    unsafe { &*enemy }.r.currentOrigin,
                    npc.r.currentOrigin,
                    &mut vec,
                );
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
/// Source: `oracle/codemp/game/NPC_AI_Default.c:490-503`
pub fn NPC_BSFace(ctx: &mut GameContext) {
    let world = unsafe { &mut *ctx.world_raw() };
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
/// Source: `oracle/codemp/game/NPC_AI_Default.c:505-610`
pub fn NPC_BSPointShoot(ctx: &mut GameContext, shoot: qboolean) {
    let world = unsafe { &mut *ctx.world_raw() };
    let npc = unsafe { &mut *(*world).globals.NPC };
    let client = unsafe { &mut *(*world).globals.client };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };

    let mut muzzle = [0.0; 3];
    let mut dir = [0.0; 3];
    let mut angles = [0.0; 3];
    let mut org = [0.0; 3];

    let __h9 = ctx.entity_id_of((*world).globals.NPC);
    if npc.enemy.is_none() {
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
        return;
    }

    if let Some(enemy_id) = npc.enemy {
        let enemy = unsafe { &*(*world).g_entities.as_ptr().add(enemy_id.0 as usize) };
        if (unsafe { &*enemy }.inuse as qboolean) == 0
            || (unsafe { &*enemy }.NPC != core::ptr::null_mut() && unsafe { &*enemy }.health <= 0)
        {
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
            return;
        }
    }

    CalcEntitySpot(ctx, __h9, SPOT_WEAPON, &mut muzzle);
    if let Some(enemy_id) = npc.enemy {
        let enemy = unsafe { &*(*world).g_entities.as_ptr().add(enemy_id.0 as usize) };
        let __h10 = ctx.entity_id_of(enemy as *const _ as *mut _);
        CalcEntitySpot(ctx, __h10, SPOT_HEAD, &mut org);

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
        npc_info.desiredYaw = client.ps.viewangles[YAW];
        npc_info.desiredPitch = client.ps.viewangles[PITCH];
        npc_info.aimTime = 0;
    }
}

/// Raven `NPC_BSMove`.
///
/// Source: `oracle/codemp/game/NPC_AI_Default.c:616-637`
pub fn NPC_BSMove(ctx: &mut GameContext) {
    let world = unsafe { &mut *ctx.world_raw() };
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
/// Source: `oracle/codemp/game/NPC_AI_Default.c:644-656`
pub fn NPC_BSShoot(ctx: &mut GameContext) {
    let world = unsafe { &mut *ctx.world_raw() };
    let client = unsafe { &mut *(*world).globals.client };

    (*world).globals.enemyVisibility = VIS_SHOOT;

    if client.ps.weaponstate as i32 != WEAPON_READY as i32
        && client.ps.weaponstate as i32 != WEAPON_FIRING as i32
    {
        unsafe { (*(*ctx.world_raw()).globals.client).ps.weaponstate = WEAPON_READY as i32 };
    }

    WeaponThink(ctx, qtrue);
}

/// Raven `NPC_BSPatrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_Default.c:664-703`
pub fn NPC_BSPatrol(ctx: &mut GameContext) {
    let world = unsafe { &mut *ctx.world_raw() };
    let npc = unsafe { &mut *(*world).globals.NPC };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };

    if (*world).level.time > npc_info.enemyCheckDebounceTime {
        npc_info.enemyCheckDebounceTime =
            (*world).level.time + (npc_info.stats.vigilance * 1000.0) as c_int;
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
/// Source: `oracle/codemp/game/NPC_AI_Default.c:712-957`
pub fn NPC_BSDefault(ctx: &mut GameContext) {
    let world = unsafe { &mut *ctx.world_raw() };
    let npc = unsafe { &mut *(*world).globals.NPC };
    let client = unsafe { &mut *(*world).globals.client };
    let npc_info = unsafe { &mut *(*world).globals.NPCInfo };

    let mut move_ = true;

    if (npc_info.scriptFlags & SCF_FIRE_WEAPON) != 0 {
        WeaponThink(ctx, qtrue);
    }

    if (npc_info.scriptFlags & SCF_FORCED_MARCH) != 0 {
        if client.ps.torsoAnim as i32 != TORSO_SURRENDER_START as i32 {
            let __h11 = ctx.entity_id_of((*world).globals.NPC).unwrap();
            NPC_SetAnim(
                ctx,
                __h11,
                SETANIM_TORSO as i32,
                TORSO_SURRENDER_START as i32,
                SETANIM_FLAG_HOLD as i32,
            );
        }
    }

    NPC_CheckEnemy(
        ctx,
        ((npc_info.scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0) as qboolean,
        qfalse,
        qtrue,
    );

    if npc.enemy.is_none() {
        if (npc_info.scriptFlags & SCF_IGNORE_ALERTS) == 0 {
            let alert_event =
                NPC_CheckAlertEvents(ctx, qtrue, qtrue, -1, qtrue, AEL_DISCOVERED as i32);

            if alert_event >= 0 {
                let alert_entry = unsafe { &(*world).level.alertEvents[alert_event as usize] };
                if alert_entry.ID != npc_info.lastAlertID
                    && alert_entry.level as i32 >= AEL_DISCOVERED as i32
                    && (npc_info.scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0
                {
                    if !alert_entry.owner.is_null() {
                        let alert_owner = unsafe { &*alert_entry.owner };
                        if !alert_owner.client.is_null() && alert_owner.health >= 0 {
                            if unsafe { &*(alert_owner.client as *mut gclient_t) }.playerTeam
                                == client.enemyTeam
                            {
                                let __h12 = ctx.entity_id_of((*world).globals.NPC).unwrap();
                                let __h13 = ctx.entity_id_of(alert_entry.owner);
                                G_SetEnemy(ctx, __h12, __h13);
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
            ) == 0
        {
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
            ) == 0
        {
            NPC_BSFollowLeader(ctx);
        } else {
            if (npc_info.scriptFlags & SCF_FACE_MOVE_DIR) != 0 || npc_info.goalEntity != npc.enemy {
                npc_info.combatMove = 0;
            } else {
                let mut dir = [0.0; 3];
                let mut angles = [0.0; 3];

                npc_info.combatMove = 0;

                if let Some(goal_id) = npc_info.goalEntity {
                    let goal_entity =
                        unsafe { &*(*world).g_entities.as_ptr().add(goal_id.0 as usize) };
                    crate::q_math::_VectorSubtract(
                        unsafe { &*goal_entity }.r.currentOrigin,
                        npc.r.currentOrigin,
                        &mut dir,
                    );
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
                if NPC_SomeoneLookingAtMe(ctx, ctx.entity_id_of((*world).globals.NPC).unwrap()) == 0
                {
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
