//! Shared (game + cgame) animal vehicle steering/physics.
//!
//! The bg-compatible half of `oracle/codemp/game/AnimalNPC.c` (a cgame TU in
//! `JK2_cgame.vcproj`). `ProcessMoveCommands` differs between build arms only in
//! `curTime` (`level.time` under `QAGAME`, `pm->cmd.serverTime` under `CGAME`);
//! `ProcessOrientCommands` is arm-identical. Game-only functions (`DeathUpdate`,
//! `Update`, `AnimateVehicle`, `AnimateRiders`, `AnimalProcessOri`,
//! `G_CreateAnimalNPC`) stay in `mp_game`'s `AnimalNPC.rs`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use mp_qshared::shared::q_math::{AngleNormalize180, VectorClear};

/// `ProcessMoveCommands` the Vehicle.
///
/// Raven: MP RULE - ALL PROCESSMOVECOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
/// Source: `oracle/codemp/game/AnimalNPC.c:168-329`;
/// cgame arm `oracle/codemp/cgame/JK2_cgame.vcproj` (`AnimalNPC.c`)
pub fn ProcessMoveCommands(pmc: &mut PmoveContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let mut speedInc: f32;
        let speedIdleDec: f32;
        let speedIdle: f32;
        let speedIdleAccel: f32;
        let speedMin: f32;
        let mut speedMax: f32;
        let fWalkSpeedMax: f32;

        // QAGAME `curTime = level.time`; CGAME `curTime = pm->cmd.serverTime`.
        // Source: oracle/codemp/game/AnimalNPC.c:185-192
        let curTime: c_int = if pmc.bg.host == BgHost::Game {
            pmc.callbacks.get_time()
        } else {
            (*pmc.pm).cmd.serverTime
        };

        let parentPS: *mut playerState_t = (*(*pVeh).m_pParentEntity).playerState;
        let vi = (*pVeh).m_pVehicleInfo;

        speedIdleDec = (*vi).decelIdle * (*pVeh).m_fTimeModifier;
        speedMax = (*vi).speedMax;
        speedIdle = (*vi).speedIdle;
        speedIdleAccel = (*vi).accelIdle * (*pVeh).m_fTimeModifier;
        speedMin = (*vi).speedMin;
        let _ = speedIdleAccel;

        if !(*pVeh).m_pPilot.is_null()
            && ((*pVeh).m_ucmd.buttons & BUTTON_ALT_ATTACK) != 0
            && (*vi).turboSpeed > 0.0f32
        {
            if (curTime - (*pVeh).m_iTurboTime) > (*vi).turboRecharge {
                (*pVeh).m_iTurboTime = curTime + (*vi).turboDuration;
                (*parentPS).speed = (*vi).turboSpeed;
            }
        }

        if curTime < (*pVeh).m_iTurboTime {
            speedMax = (*vi).turboSpeed;
        } else {
            speedMax = (*vi).speedMax;
        }

        if (*parentPS).m_iVehicleNum == 0 {
            speedInc = speedIdle * (*pVeh).m_fTimeModifier;
            VectorClear(&mut (*parentPS).moveDir);
            (*parentPS).speed = 0.0f32;
        } else {
            speedInc = (*vi).acceleration * (*pVeh).m_fTimeModifier;
        }

        if (*parentPS).speed != 0.0f32
            || (*parentPS).groundEntityNum == ENTITYNUM_NONE
            || (*pVeh).m_ucmd.forwardmove != 0
            || (*pVeh).m_ucmd.upmove > 0
        {
            if (*pVeh).m_ucmd.forwardmove > 0 && speedInc != 0.0f32 {
                (*parentPS).speed += speedInc;
            } else if (*pVeh).m_ucmd.forwardmove < 0 {
                if (*parentPS).speed > speedIdle {
                    (*parentPS).speed -= speedInc;
                } else if (*parentPS).speed > speedMin {
                    (*parentPS).speed -= speedIdleDec;
                }
            } else if (*parentPS).speed > 0.0f32 {
                (*parentPS).speed -= speedIdleDec;
                if (*parentPS).speed < 0.0f32 {
                    (*parentPS).speed = 0.0f32;
                }
            } else if (*parentPS).speed < 0.0f32 {
                (*parentPS).speed += speedIdleDec;
                if (*parentPS).speed > 0.0f32 {
                    (*parentPS).speed = 0.0f32;
                }
            }
        } else {
            if (*pVeh).m_ucmd.forwardmove < 0 {
                (*pVeh).m_ucmd.forwardmove = 0;
            }
            if (*pVeh).m_ucmd.upmove < 0 {
                (*pVeh).m_ucmd.upmove = 0;
            }
        }

        fWalkSpeedMax = speedMax * 0.275f32;
        if curTime > (*pVeh).m_iTurboTime
            && ((*pVeh).m_ucmd.buttons & BUTTON_WALKING) != 0
            && (*parentPS).speed > fWalkSpeedMax
        {
            (*parentPS).speed = fWalkSpeedMax;
        } else if (*parentPS).speed > speedMax {
            (*parentPS).speed = speedMax;
        } else if (*parentPS).speed < speedMin {
            (*parentPS).speed = speedMin;
        }
    }
}

/// `ProcessOrientCommands` the Vehicle.
///
/// Raven: MP RULE - ALL PROCESSORIENTCOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
/// Source: `oracle/codemp/game/AnimalNPC.c:338-464`;
/// cgame arm `oracle/codemp/cgame/JK2_cgame.vcproj` (`AnimalNPC.c`)
pub fn ProcessOrientCommands(pmc: &mut PmoveContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let parent = (*pVeh).m_pParentEntity;

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

        // Oracle: `if (rider)` — always true after the fallback above.
        let mut angDif =
            AngleSubtract(*(*pVeh).m_vOrientation.add(YAW), (*riderPS).viewangles[YAW]);
        if !parentPS.is_null() && (*parentPS).speed != 0.0f32 {
            let mut s = (*parentPS).speed;
            let maxDif = (*(*pVeh).m_pVehicleInfo).turningSpeed * 4.0f32;
            if s < 0.0f32 {
                s = -s;
            }
            angDif *= s / (*(*pVeh).m_pVehicleInfo).speedMax;
            if angDif > maxDif {
                angDif = maxDif;
            } else if angDif < -maxDif {
                angDif = -maxDif;
            }
            *(*pVeh).m_vOrientation.add(YAW) = AngleNormalize180(
                *(*pVeh).m_vOrientation.add(YAW) - angDif * ((*pVeh).m_fTimeModifier * 0.2f32),
            );
        }
    }
}
