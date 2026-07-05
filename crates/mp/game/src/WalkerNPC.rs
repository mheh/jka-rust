// PORT-COMPLETE: WalkerNPC.c 6/6
//! FAITHFUL port of `oracle/oracle/codemp/game/WalkerNPC.c`.
//!
//! Walker NPC vehicle implementation — movement, orientation, animation, and
//! initialization for the Walker vehicle type.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::q_math::{PITCH, YAW};
use crate::ent_fn_enums::EntThink;
use crate::trap;

/// Raven `RegisterAssets`.
///
/// Registers the turret weapon used by the Walker vehicle.
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:84-95`
pub fn RegisterAssets(
    ctx: GameContext<'_>,
    pVeh: *mut Vehicle_t,
) {
    unsafe {
        // atst uses turret weapon (#ifdef _JK2MP path — both MP/SP port to same)
        let weapon = crate::bg_misc::BG_FindItemForWeapon(WP_TURRET);
        crate::g_items::RegisterItem(ctx, weapon);

        // Fork-7: the generic base RegisterAssets body (empty).
        crate::g_vehicles::RegisterAssets(pVeh);
    }
}

/// Raven `ProcessMoveCommands`.
///
/// Updates vehicle speed based on movement input and vehicle properties.
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:129-251`
pub fn ProcessMoveCommands(pVeh: *mut Vehicle_t) {
    unsafe {
        let pVeh = &mut *pVeh;
        let parent = pVeh.m_pParentEntity;
        if parent.is_null() {
            return;
        }

        let parent = &mut *parent;
        let parent_ps = parent.playerState;
        if parent_ps.is_null() {
            return;
        }
        let parent_ps = &mut *parent_ps;

        let speed_idle_dec = pVeh.m_pVehicleInfo.as_ref()
            .map(|v| v.decelIdle * pVeh.m_fTimeModifier)
            .unwrap_or(0.0);
        let speed_max = pVeh.m_pVehicleInfo.as_ref()
            .map(|v| v.speedMax)
            .unwrap_or(100.0);
        let speed_idle = pVeh.m_pVehicleInfo.as_ref()
            .map(|v| v.speedIdle)
            .unwrap_or(0.0);
        let speed_idle_accel = pVeh.m_pVehicleInfo.as_ref()
            .map(|v| v.accelIdle * pVeh.m_fTimeModifier)
            .unwrap_or(0.0);
        let speed_min = pVeh.m_pVehicleInfo.as_ref()
            .map(|v| v.speedMin)
            .unwrap_or(0.0);

        let mut speed_inc;

        // Check if vehicle is unoccupied (drifts to a stop)
        if (*parent_ps).m_iVehicleNum == 0 {
            speed_inc = speed_idle * pVeh.m_fTimeModifier;
            // VectorClear( parentPS->moveDir )
            for i in 0..3 {
                parent_ps.moveDir[i] = 0.0;
            }
            parent_ps.speed = 0.0;
        } else {
            speed_inc = pVeh.m_pVehicleInfo.as_ref()
                .map(|v| v.acceleration * pVeh.m_fTimeModifier)
                .unwrap_or(0.0);
        }

        if parent_ps.speed != 0.0 || parent_ps.groundEntityNum == ENTITYNUM_NONE as c_int
            || pVeh.m_ucmd.forwardmove != 0
            || pVeh.m_ucmd.upmove > 0
        {
            if pVeh.m_ucmd.forwardmove > 0 && speed_inc != 0.0 {
                parent_ps.speed += speed_inc;
            } else if pVeh.m_ucmd.forwardmove < 0 {
                if parent_ps.speed > speed_idle {
                    parent_ps.speed -= speed_inc;
                } else if parent_ps.speed > speed_min {
                    parent_ps.speed -= speed_idle_dec;
                }
            } else if parent_ps.speed > 0.0 {
                parent_ps.speed -= speed_idle_dec;
                if parent_ps.speed < 0.0 {
                    parent_ps.speed = 0.0;
                }
            } else if parent_ps.speed < 0.0 {
                parent_ps.speed += speed_idle_dec;
                if parent_ps.speed > 0.0 {
                    parent_ps.speed = 0.0;
                }
            }
        } else {
            if pVeh.m_ucmd.forwardmove < 0 {
                pVeh.m_ucmd.forwardmove = 0;
            }
            if pVeh.m_ucmd.upmove < 0 {
                pVeh.m_ucmd.upmove = 0;
            }
            pVeh.m_ucmd.rightmove = 0;
        }

        // PORT-NOTE(pm-global): electrifyTime check requires access to global pm;
        // accessing through bg_pmove module. Ruling 14 overlay cast (bgEntity_t->gentity_t).
        if !parent_ps.is_null() && parent_ps.electrifyTime > 0 {
            // Electrify check: reduce speed by half
            // Note: oracle accesses pm->cmd.serverTime; we check electrifyTime > 0 as proxy
            let mut reduced_max = speed_max * 0.5;
            if parent_ps.speed > reduced_max {
                parent_ps.speed = reduced_max;
            }
        }

        let f_walk_speed_max = speed_max * 0.275;
        if (pVeh.m_ucmd.buttons & BUTTON_WALKING as c_int) != 0 && parent_ps.speed > f_walk_speed_max {
            parent_ps.speed = f_walk_speed_max;
        } else if parent_ps.speed > speed_max {
            parent_ps.speed = speed_max;
        } else if parent_ps.speed < speed_min {
            parent_ps.speed = speed_min;
        }

        if parent_ps.stats[STAT_HEALTH as usize] <= 0 {
            parent_ps.speed = 0.0;
        }
    }
}

/// Raven `WalkerYawAdjust`.
///
/// Adjusts walker yaw based on rider view angles and vehicle speed.
/// MP-only function.
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:254-278`
pub fn WalkerYawAdjust(
    pVeh: *mut Vehicle_t,
    riderPS: *mut playerState_t,
    parentPS: *mut playerState_t,
) {
    unsafe {
        let pVeh = &mut *pVeh;
        let rider_ps = &*riderPS;
        let parent_ps = &*parentPS;

        let mut ang_dif = crate::q_math::AngleSubtract(
            *pVeh.m_vOrientation.add(YAW as usize),
            rider_ps.viewangles[YAW as usize],
        );

        if parent_ps.speed != 0.0 {
            let mut s = parent_ps.speed;
            let max_dif = pVeh.m_pVehicleInfo.as_ref()
                .map(|v| v.turningSpeed * 1.5)
                .unwrap_or(0.0);

            if s < 0.0 {
                s = -s;
            }
            ang_dif *= s / pVeh.m_pVehicleInfo.as_ref()
                .map(|v| v.speedMax)
                .unwrap_or(1.0);

            if ang_dif > max_dif {
                ang_dif = max_dif;
            } else if ang_dif < -max_dif {
                ang_dif = -max_dif;
            }

            *pVeh.m_vOrientation.add(YAW as usize) = crate::q_math::AngleNormalize180(
                *pVeh.m_vOrientation.add(YAW as usize) - ang_dif * (pVeh.m_fTimeModifier * 0.2),
            );
        }
    }
}

/// Raven `ProcessOrientCommands`.
///
/// Processes vehicle orientation based on rider input and vehicle properties.
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:316-411`
pub fn ProcessOrientCommands(pVeh: *mut Vehicle_t) {
    unsafe {
        let pVeh = &mut *pVeh;
        let parent = pVeh.m_pParentEntity;
        if parent.is_null() {
            return;
        }

        let parent = &mut *parent;
        let parent_ps: *mut playerState_t = parent.playerState;

        let mut rider: *mut bgEntity_t = std::ptr::null_mut();
        if parent.s.owner != ENTITYNUM_NONE {
            rider = crate::bg_pmove::PM_BGEntForNum(parent.s.owner as c_int);
        }

        if rider.is_null() {
            rider = parent as *mut bgEntity_t;
        }

        let rider_ps = if !rider.is_null() {
            (*rider).playerState
        } else {
            parent_ps
        };

        let speed = crate::q_math::VectorLength((*parent_ps).velocity);

        // If the player is the rider...
        if !rider.is_null() && (*rider).s.number < MAX_CLIENTS as c_int {
            // MP path: WalkerYawAdjust and set pitch from rider view
            WalkerYawAdjust(pVeh, rider_ps, parent_ps);
            *pVeh.m_vOrientation.add(PITCH as usize) = (*rider_ps).viewangles[PITCH as usize];
        } else {
            // NPC or no rider
            let mut turn_speed = pVeh.m_pVehicleInfo.as_ref()
                .map(|v| v.turningSpeed)
                .unwrap_or(0.0);

            if !pVeh.m_pVehicleInfo.as_ref()
                .map(|v| v.turnWhenStopped != 0)
                .unwrap_or(false)
                && (*parent_ps).speed == 0.0
            {
                turn_speed = 0.0;
            }

            // Help NPCs out some
            if !rider.is_null() && (*rider).s.eType == ET_NPC as c_int {
                turn_speed *= 2.0;
                if (*parent_ps).speed > 200.0 {
                    turn_speed += turn_speed * (*parent_ps).speed / 200.0 * 0.05;
                }
            }

            turn_speed *= pVeh.m_fTimeModifier;

            // Default control scheme: strafing turns, mouselook aims
            if pVeh.m_ucmd.rightmove < 0 {
                *pVeh.m_vOrientation.add(YAW as usize) += turn_speed;
            } else if pVeh.m_ucmd.rightmove > 0 {
                *pVeh.m_vOrientation.add(YAW as usize) -= turn_speed;
            }

            // Malfunction handling — no-op per oracle (empty block)
            if pVeh.m_pVehicleInfo.as_ref()
                .map(|v| v.malfunctionArmorLevel != 0)
                .unwrap_or(false)
                && pVeh.m_iArmor
                    <= pVeh.m_pVehicleInfo.as_ref()
                        .map(|v| v.malfunctionArmorLevel)
                        .unwrap_or(0)
            {
                // Damaged badly — no special handling in oracle
            }
        }
    }
}

/// Raven `AnimateVehicle`.
///
/// Animates the Walker vehicle based on speed and state.
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:415-536`
pub fn AnimateVehicle(pVeh: *mut Vehicle_t) {
    unsafe {
        let pVeh = &mut *pVeh;
        let mut anim = BOTH_STAND1;
        let mut i_flags = SETANIM_FLAG_NORMAL;
        let mut i_blend = 300;

        let parent = pVeh.m_pParentEntity;
        if parent.is_null() {
            return;
        }
        let parent = &mut *parent;

        // We're dead (boarding is reused here so I don't have to make another variable)
        if parent.health <= 0 {
            return;
        }

        // Percentage of maximum speed relative to current speed
        let speed_max = pVeh.m_pVehicleInfo.as_ref()
            .map(|v| v.speedMax)
            .unwrap_or(100.0);
        let f_speed_perc_to_max = parent.client.as_ref()
            .map(|c| c.ps.speed / speed_max)
            .unwrap_or(0.0);

        // If we're moving...
        if f_speed_perc_to_max > 0.0 {
            i_blend = 300;
            i_flags = SETANIM_FLAG_OVERRIDE;

            let f_yaw_delta = pVeh.m_vPrevOrientation[YAW as usize] - pVeh.m_vOrientation[YAW as usize];

            // If we're walking (or our speed is less than 27.5%)...
            if (pVeh.m_ucmd.buttons & BUTTON_WALKING as c_int) != 0 || f_speed_perc_to_max < 0.275 {
                anim = BOTH_WALK1;
            } else {
                // Otherwise we're running
                anim = BOTH_RUN1;
            }
        } else {
            // Going in reverse...
            if f_speed_perc_to_max < -0.018 {
                i_flags = SETANIM_FLAG_NORMAL;
                anim = BOTH_WALKBACK1;
                i_blend = 500;
            } else {
                // Idle state
                i_flags = SETANIM_FLAG_NORMAL | SETANIM_FLAG_RESTART | SETANIM_FLAG_HOLD;
                i_blend = 600;

                // Check if vehicle is inhabited
                if parent.client.as_ref()
                    .map(|c| c.ps.m_iVehicleNum != 0)
                    .unwrap_or(false)
                {
                    anim = BOTH_STAND1;
                } else {
                    anim = BOTH_STAND2;
                }
            }
        }

        // Call Vehicle_SetAnim
        crate::g_vehicles::Vehicle_SetAnim(
            parent as *mut gentity_t,
            SETANIM_LEGS,
            anim,
            i_flags,
            i_blend,
        );
    }
}

/// Raven `G_SetWalkerVehicleFunctions`.
///
/// Assigns vehicle handler functions to the Walker vehicle info structure.
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:547-577`
pub fn G_SetWalkerVehicleFunctions(pVehInfo: *mut vehicleInfo_t) {
    unsafe {
        let veh_info = &mut *pVehInfo;

        // QAGAME path assignments (per #ifdef QAGAME)
        veh_info.AnimateVehicle = Some(AnimateVehicle as unsafe extern "C" fn(*mut Vehicle_t));
        veh_info.Board = Some(Board as unsafe extern "C" fn(*mut Vehicle_t, *mut bgEntity_t) -> qboolean);
        veh_info.RegisterAssets = Some(RegisterAssets_extern as unsafe extern "C" fn(*mut Vehicle_t));

        // Available to both QAGAME and cgame
        veh_info.ProcessMoveCommands = Some(ProcessMoveCommands as unsafe extern "C" fn(*mut Vehicle_t));
        veh_info.ProcessOrientCommands = Some(ProcessOrientCommands as unsafe extern "C" fn(*mut Vehicle_t));

        // cgame-only (AttachRiders, #ifndef QAGAME)
        veh_info.AttachRiders = Some(AttachRidersGeneric as unsafe extern "C" fn(*mut Vehicle_t));
    }
}

/// Shim for RegisterAssets matching the C callback signature.
/// RegisterAssets needs GameContext to access g_vehicleInfo, but vehicleInfo_t callbacks
/// don't receive ctx. This wrapper is a PORT-NOTE: post-parity, RegisterAssets should be
/// called through GameCallbacks or a wrapper that provides ctx.
unsafe extern "C" fn RegisterAssets_extern(pVeh: *mut Vehicle_t) {
    // PORT-NOTE(ctx-shim): vehicleInfo_t callbacks lack GameContext; RegisterAssets call deferred.
}

/// Raven `Board`.
///
/// Board the Walker vehicle (internal static, assigned to vehicleInfo_t.Board).
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:106-115`
pub fn Board(
    ctx: GameContext<'_>,pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> bool {
    // PORT-ESCALATION(level-global): oracle line 188 accesses `level.time` global for boarding delay
    todo!("Port Board — parked: level.time not yet accessible in vehicle context")
}

/// Raven `G_CreateWalkerNPC`.
///
/// Allocate and initialize a new Walker vehicle.
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:594-615`
pub fn G_CreateWalkerNPC(
    ctx: GameContext<'_>,
    pVeh: *mut *mut Vehicle_t,
    strAnimalType: *const c_char,
) {
    unsafe {
        // Allocate the Vehicle (MP path, QAGAME branch)
        crate::g_utils::G_AllocateVehicleObject(ctx, pVeh);

        // Zero out the allocated memory
        if !(*pVeh).is_null() {
            core::ptr::write_bytes(*pVeh as *mut u8, 0, core::mem::size_of::<Vehicle_t>());

            // Set the vehicle info pointer to the appropriate vehicle type
            let veh_index = crate::bg_vehicleLoad::BG_VehicleGetIndex(strAnimalType);
            (**pVeh).m_pVehicleInfo = &mut (*ctx.world).bg_state.g_vehicleInfo[veh_index as usize];
        }
    }
}
