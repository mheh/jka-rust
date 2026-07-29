//! Shared (game + cgame) walker vehicle steering/physics.
//!
//! The bg-compatible half of `oracle/codemp/game/WalkerNPC.c` (a cgame TU in
//! `JK2_cgame.vcproj`). `ProcessMoveCommands`/`ProcessOrientCommands` and their
//! `WalkerYawAdjust` helper carry no game/cgame body split in the MP build
//! (`electrifyTime` compares against `m_ucmd.serverTime`, the vehicle ucmd copy
//! of `pm->cmd`), so they run identically under both hosts. Game-only functions
//! (`RegisterAssets`, `AnimateVehicle`, `Board`, `G_CreateWalkerNPC`) stay in
//! `mp_game`'s `WalkerNPC.rs`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use mp_qshared::shared::q_math::{AngleNormalize180, VectorClear};

/// Raven `ProcessMoveCommands`.
///
/// Source: `oracle/codemp/game/WalkerNPC.c:129-251`;
/// cgame arm `oracle/codemp/cgame/JK2_cgame.vcproj` (`WalkerNPC.c`)
pub fn ProcessMoveCommands(_pmc: &mut PmoveContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let parent = (*pVeh).m_pParentEntity;
        if parent.is_null() {
            return;
        }
        let parentPS: *mut playerState_t = (*parent).playerState;
        if parentPS.is_null() {
            return;
        }

        let vi = (*pVeh).m_pVehicleInfo;
        let speedIdleDec = (*vi).decelIdle * (*pVeh).m_fTimeModifier;
        let mut speedMax = (*vi).speedMax;
        let speedIdle = (*vi).speedIdle;
        let speedIdleAccel = (*vi).accelIdle * (*pVeh).m_fTimeModifier;
        let speedMin = (*vi).speedMin;
        let _ = speedIdleAccel;

        let mut speedInc;
        // Drifts to a stop when unoccupied (`!parentPS->m_iVehicleNum`).
        if (*parentPS).m_iVehicleNum == 0 {
            speedInc = speedIdle * (*pVeh).m_fTimeModifier;
            VectorClear(&mut (*parentPS).moveDir);
            (*parentPS).speed = 0.0;
        } else {
            speedInc = (*vi).acceleration * (*pVeh).m_fTimeModifier;
        }

        if (*parentPS).speed != 0.0
            || (*parentPS).groundEntityNum == ENTITYNUM_NONE
            || (*pVeh).m_ucmd.forwardmove != 0
            || (*pVeh).m_ucmd.upmove > 0
        {
            if (*pVeh).m_ucmd.forwardmove > 0 && speedInc != 0.0 {
                (*parentPS).speed += speedInc;
            } else if (*pVeh).m_ucmd.forwardmove < 0 {
                if (*parentPS).speed > speedIdle {
                    (*parentPS).speed -= speedInc;
                } else if (*parentPS).speed > speedMin {
                    (*parentPS).speed -= speedIdleDec;
                }
            } else if (*parentPS).speed > 0.0 {
                (*parentPS).speed -= speedIdleDec;
                if (*parentPS).speed < 0.0 {
                    (*parentPS).speed = 0.0;
                }
            } else if (*parentPS).speed < 0.0 {
                (*parentPS).speed += speedIdleDec;
                if (*parentPS).speed > 0.0 {
                    (*parentPS).speed = 0.0;
                }
            }
        } else {
            if (*pVeh).m_ucmd.forwardmove < 0 {
                (*pVeh).m_ucmd.forwardmove = 0;
            }
            if (*pVeh).m_ucmd.upmove < 0 {
                (*pVeh).m_ucmd.upmove = 0;
            }
            (*pVeh).m_ucmd.rightmove = 0;
        }

        // `pm->cmd.serverTime` == `m_ucmd.serverTime` (the vehicle ucmd copy).
        if (*parentPS).electrifyTime > (*pVeh).m_ucmd.serverTime {
            speedMax *= 0.5;
        }

        let fWalkSpeedMax = speedMax * 0.275;
        if (*pVeh).m_ucmd.buttons & (BUTTON_WALKING as c_int) != 0
            && (*parentPS).speed > fWalkSpeedMax
        {
            (*parentPS).speed = fWalkSpeedMax;
        } else if (*parentPS).speed > speedMax {
            (*parentPS).speed = speedMax;
        } else if (*parentPS).speed < speedMin {
            (*parentPS).speed = speedMin;
        }

        if (*parentPS).stats[STAT_HEALTH as usize] <= 0 {
            (*parentPS).speed = 0.0;
        }
    }
}

/// Raven `WalkerYawAdjust` (`_JK2MP` only).
///
/// Source: `oracle/codemp/game/WalkerNPC.c:254-278`
pub fn WalkerYawAdjust(
    pVeh: *mut Vehicle_t,
    riderPS: *mut playerState_t,
    parentPS: *mut playerState_t,
) {
    unsafe {
        let mut angDif =
            AngleSubtract(*(*pVeh).m_vOrientation.add(YAW), (*riderPS).viewangles[YAW]);

        if !parentPS.is_null() && (*parentPS).speed != 0.0 {
            let mut s = (*parentPS).speed;
            let maxDif = (*(*pVeh).m_pVehicleInfo).turningSpeed * 1.5;

            if s < 0.0 {
                s = -s;
            }
            angDif *= s / (*(*pVeh).m_pVehicleInfo).speedMax;

            if angDif > maxDif {
                angDif = maxDif;
            } else if angDif < -maxDif {
                angDif = -maxDif;
            }

            *(*pVeh).m_vOrientation.add(YAW) = AngleNormalize180(
                *(*pVeh).m_vOrientation.add(YAW) - angDif * ((*pVeh).m_fTimeModifier * 0.2),
            );
        }
    }
}

/// Raven `ProcessOrientCommands`.
///
/// Source: `oracle/codemp/game/WalkerNPC.c:316-411`;
/// cgame arm `oracle/codemp/cgame/JK2_cgame.vcproj` (`WalkerNPC.c`)
pub fn ProcessOrientCommands(pmc: &mut PmoveContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let parent = (*pVeh).m_pParentEntity;
        if parent.is_null() {
            return;
        }

        // Oracle `_JK2MP`: `if (owner != ENTITYNUM_NONE) rider = PM_BGEntForNum(owner);`
        // then `if (!rider) rider = parent;`.
        let mut rider: *mut bgEntity_t = core::ptr::null_mut();
        if (*parent).s.owner != ENTITYNUM_NONE {
            rider = pmc.PM_BGEntForNum((*parent).s.owner);
        }
        if rider.is_null() {
            rider = parent;
        }

        let parentPS: *mut playerState_t = (*parent).playerState;
        let riderPS: *mut playerState_t = (*rider).playerState;

        let _speed = VectorLength((*parentPS).velocity);

        // If the player is the rider...
        if (*rider).s.number < MAX_CLIENTS as c_int {
            WalkerYawAdjust(pVeh, riderPS, parentPS);
            *(*pVeh).m_vOrientation.add(PITCH) = (*riderPS).viewangles[PITCH];
        } else {
            let mut turnSpeed = (*(*pVeh).m_pVehicleInfo).turningSpeed;
            if (*(*pVeh).m_pVehicleInfo).turnWhenStopped == 0 && (*parentPS).speed == 0.0 {
                turnSpeed = 0.0;
            }
            // Help NPCs out some.
            if (*rider).s.eType == ET_NPC as c_int {
                turnSpeed *= 2.0;
                if (*parentPS).speed > 200.0 {
                    turnSpeed += turnSpeed * (*parentPS).speed / 200.0 * 0.05;
                }
            }
            turnSpeed *= (*pVeh).m_fTimeModifier;

            // Default control scheme: strafing turns, mouselook aims.
            if (*pVeh).m_ucmd.rightmove < 0 {
                *(*pVeh).m_vOrientation.add(YAW) += turnSpeed;
            } else if (*pVeh).m_ucmd.rightmove > 0 {
                *(*pVeh).m_vOrientation.add(YAW) -= turnSpeed;
            }

            // Malfunction handling — no-op per oracle (empty block).
            if (*(*pVeh).m_pVehicleInfo).malfunctionArmorLevel != 0
                && (*pVeh).m_iArmor <= (*(*pVeh).m_pVehicleInfo).malfunctionArmorLevel
            {
                // Damaged badly — no special handling in oracle.
            }
        }
    }
}
