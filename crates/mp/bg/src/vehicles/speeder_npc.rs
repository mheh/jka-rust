//! Shared (game + cgame) speeder vehicle steering/physics.
//!
//! The bg-compatible half of `oracle/codemp/game/SpeederNPC.c` — the functions
//! Raven compiles into BOTH the game and cgame builds (`JK2_cgame.vcproj` lists
//! `SpeederNPC.c`), so they live in the shared `mp_bg` crate and run inside
//! `Pmove` (game-tier `Update` chain for the Game host, cgame prediction for the
//! Cgame host). Game-only functions (`Update`, `AnimateVehicle`, `AnimateRiders`,
//! `G_CreateSpeederNPC`) stay in `mp_game`'s `SpeederNPC.rs`.
//!
//! `#ifdef QAGAME` / `#else` build arms are host-switched on `pmc.bg.host`
//! (`BgHost::Game` vs `BgHost::Cgame`); the `#ifdef QAGAME` islands that reach
//! game-only entity fields route through `GameCallbacks` upcalls under the Game
//! host and compile out under the Cgame host.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use mp_qshared::shared::q_math::AngleNormalize180;

/// Raven `VEH_StartStrafeRam`.
///
/// Raven: the `_JK2MP` build of this function is a stub — the strafe-ram
/// mechanic is SP-only (`#ifndef _JK2MP` guards the real implementation at
/// SpeederNPC.c:102-138); the MP build always returns `qfalse`.
/// Source: `oracle/codemp/game/SpeederNPC.c:140-143`
pub fn VEH_StartStrafeRam(_pVeh: *mut Vehicle_t, _Right: qboolean, _Duration: c_int) -> qboolean {
    qfalse
}

/// `ProcessMoveCommands` the Vehicle.
///
/// Raven: MP RULE - ALL PROCESSMOVECOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
/// `curTime` is `level.time` under `QAGAME`, `pm->cmd.serverTime` under `CGAME`
/// (oracle SpeederNPC.c:327-332); host-switched here. The turbo-start effect
/// loop body is `#ifdef QAGAME` — routed through `veh_turbo_start_fx` under the
/// Game host, skipped under the Cgame host.
/// Source: `oracle/codemp/game/SpeederNPC.c:278-490`;
/// cgame arm `oracle/codemp/cgame/JK2_cgame.vcproj` (`SpeederNPC.c`)
pub fn ProcessMoveCommands(pmc: &mut PmoveContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let mut speedInc: f32;
        let speedIdleDec: f32;
        let speedIdle: f32;
        let speedIdleAccel: f32;
        let speedMin: f32;
        let mut speedMax: f32;
        let parentPS: *mut playerState_t;
        let mut pilotPS: *mut playerState_t = core::ptr::null_mut();
        let curTime: c_int;

        // Get player states from parent and pilot (bg-reachable `bgEntity_t`).
        parentPS = (*(*pVeh).m_pParentEntity).playerState;
        if !(*pVeh).m_pPilot.is_null() {
            pilotPS = (*(*pVeh).m_pPilot).playerState;
        }

        // Determine speed increment based on flying status
        if (*pVeh).m_ulFlags & (VEH_FLYING as u64) != 0 {
            speedInc = (*(*pVeh).m_pVehicleInfo).acceleration * (*pVeh).m_fTimeModifier * 0.4f32;
        } else if (*parentPS).m_iVehicleNum == 0 {
            // Drifts to a stop. MP `#ifdef _JK2MP` branch is `!parentPS->m_iVehicleNum`.
            speedInc = 0.0f32;
        } else {
            speedInc = (*(*pVeh).m_pVehicleInfo).acceleration * (*pVeh).m_fTimeModifier;
        }

        speedIdleDec = (*(*pVeh).m_pVehicleInfo).decelIdle * (*pVeh).m_fTimeModifier;

        // QAGAME `curTime = level.time`; CGAME `curTime = pm->cmd.serverTime`.
        // Source: oracle/codemp/game/SpeederNPC.c:327-332
        curTime = if pmc.bg.host == BgHost::Game {
            pmc.callbacks.get_time()
        } else {
            (*pmc.pm).cmd.serverTime
        };

        // Handle turbo/acceleration
        if !(*pVeh).m_pPilot.is_null()
            && ((*pVeh).m_ucmd.buttons & BUTTON_ALT_ATTACK != 0)
            && (*(*pVeh).m_pVehicleInfo).turboSpeed != 0.0
        {
            if (!parentPS.is_null() && (*parentPS).electrifyTime > curTime)
                || (!pilotPS.is_null()
                    && ((*pilotPS).weapon == WP_MELEE
                        || ((*pilotPS).weapon == WP_SABER && BG_SabersOff(pilotPS) != 0)))
            {
                if (curTime - (*pVeh).m_iTurboTime) > (*(*pVeh).m_pVehicleInfo).turboRecharge {
                    (*pVeh).m_iTurboTime = curTime + (*(*pVeh).m_pVehicleInfo).turboDuration;

                    if (*(*pVeh).m_pVehicleInfo).iTurboStartFX != 0 {
                        // The per-exhaust `trap_G2API_GetBoltMatrix`/`G_PlayEffectID`
                        // loop body is `#ifdef QAGAME` (reaches `ghoul2`/`modelScale`),
                        // so the Game host routes it through the upcall and the Cgame
                        // host skips it. Source: oracle/codemp/game/SpeederNPC.c:350-371
                        if pmc.bg.host == BgHost::Game {
                            pmc.callbacks
                                .veh_turbo_start_fx((*(*pVeh).m_pParentEntity).s.number);
                        }
                    }

                    if !parentPS.is_null() {
                        (*parentPS).speed = (*(*pVeh).m_pVehicleInfo).turboSpeed as f32;
                    }
                }
            }
        }

        // Slide breaking
        if (*pVeh).m_ulFlags & (VEH_SLIDEBREAKING as u64) != 0 {
            if (*pVeh).m_ucmd.forwardmove >= 0 {
                (*pVeh).m_ulFlags &= !(VEH_SLIDEBREAKING as u64);
            }
            if !parentPS.is_null() {
                (*parentPS).speed = 0.0f32;
            }
        } else if (curTime > (*pVeh).m_iTurboTime)
            && ((*pVeh).m_ulFlags & (VEH_FLYING as u64) == 0)
            && ((*pVeh).m_ucmd.forwardmove < 0)
            && (*(*pVeh).m_vOrientation.add(ROLL)).abs() > 25.0f32
        {
            (*pVeh).m_ulFlags |= VEH_SLIDEBREAKING as u64;
        }

        // Determine speed max based on turbo
        if curTime < (*pVeh).m_iTurboTime {
            speedMax = (*(*pVeh).m_pVehicleInfo).turboSpeed as f32;
            if !parentPS.is_null() {
                (*parentPS).eFlags |= EF_JETPACK_ACTIVE;
            }
        } else {
            speedMax = (*(*pVeh).m_pVehicleInfo).speedMax as f32;
            if !parentPS.is_null() {
                (*parentPS).eFlags &= !EF_JETPACK_ACTIVE;
            }
        }

        speedIdle = (*(*pVeh).m_pVehicleInfo).speedIdle as f32;
        speedIdleAccel = (*(*pVeh).m_pVehicleInfo).accelIdle * (*pVeh).m_fTimeModifier;
        speedMin = (*(*pVeh).m_pVehicleInfo).speedMin as f32;
        let _ = speedIdleAccel;

        // Handle forward/backward movement
        if (!parentPS.is_null() && (*parentPS).speed != 0.0f32)
            || (!parentPS.is_null() && (*parentPS).groundEntityNum == ENTITYNUM_NONE)
            || (*pVeh).m_ucmd.forwardmove != 0
            || (*pVeh).m_ucmd.upmove > 0
        {
            if (*pVeh).m_ucmd.forwardmove > 0 && speedInc != 0.0f32 {
                if !parentPS.is_null() {
                    (*parentPS).speed += speedInc;
                }
            } else if (*pVeh).m_ucmd.forwardmove < 0 {
                if !parentPS.is_null() {
                    if (*parentPS).speed > speedIdle {
                        (*parentPS).speed -= speedInc;
                    } else if (*parentPS).speed > speedMin {
                        (*parentPS).speed -= speedIdleDec;
                    }
                }
            } else if !parentPS.is_null() && (*parentPS).speed > 0.0f32 {
                (*parentPS).speed -= speedIdleDec;
                if (*parentPS).speed < 0.0f32 {
                    (*parentPS).speed = 0.0f32;
                }
            } else if !parentPS.is_null() && (*parentPS).speed < 0.0f32 {
                (*parentPS).speed += speedIdleDec;
                if (*parentPS).speed > 0.0f32 {
                    (*parentPS).speed = 0.0f32;
                }
            }
        }

        // Clamp speed to limits
        if !parentPS.is_null() {
            if (*parentPS).speed > speedMax {
                (*parentPS).speed = speedMax;
            } else if (*parentPS).speed < speedMin {
                (*parentPS).speed = speedMin;
            }

            // Electrify effect
            if (*parentPS).electrifyTime > curTime {
                (*parentPS).speed *= (*pVeh).m_fTimeModifier / 60.0f32;
            }
        }
    }
}

/// `ProcessOrientCommands` the Vehicle.
///
/// Raven: MP RULE - ALL PROCESSORIENTCOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
/// The `_JK2MP` branch handles MP vehicle orientation (yaw control via view
/// angles) and is identical under both build arms; the electrify wobble uses
/// `pm->cmd.serverTime` (== `m_ucmd.serverTime`, the vehicle ucmd copy).
/// Source: `oracle/codemp/game/SpeederNPC.c:505-600`;
/// cgame arm `oracle/codemp/cgame/JK2_cgame.vcproj` (`SpeederNPC.c`)
pub fn ProcessOrientCommands(_pmc: &mut PmoveContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let riderPS: *mut playerState_t;
        let parentPS: *mut playerState_t;
        let mut angDif: f32;

        if !(*pVeh).m_pPilot.is_null() {
            riderPS = (*(*pVeh).m_pPilot).playerState;
        } else {
            riderPS = (*(*pVeh).m_pParentEntity).playerState;
        }
        parentPS = (*(*pVeh).m_pParentEntity).playerState;

        angDif = AngleSubtract(*(*pVeh).m_vOrientation.add(YAW), (*riderPS).viewangles[YAW]);

        if !parentPS.is_null() && (*parentPS).speed != 0.0f32 {
            let mut s: f32 = (*parentPS).speed;
            let maxDif: f32 = (*(*pVeh).m_pVehicleInfo).turningSpeed as f32 * 4.0f32;

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

            // `pm->cmd.serverTime` is `m_ucmd.serverTime` (assigned pm->cmd before
            // dispatch). C widths: `serverTime/1000.0f` is an f32 divide; `sin` is
            // double libm; `*3.0f`/`*m_fTimeModifier` compute in double; narrows once.
            // Source: oracle/codemp/game/SpeederNPC.c:547-550
            if (*parentPS).electrifyTime > (*pVeh).m_ucmd.serverTime {
                let yaw_ref = (*pVeh).m_vOrientation.add(YAW);
                *yaw_ref = (*yaw_ref as f64
                    + (((*pVeh).m_ucmd.serverTime as f32 / 1000.0f32) as f64).sin()
                        * 3.0f64
                        * (*pVeh).m_fTimeModifier as f64) as f32;
            }
        }
    }
}
