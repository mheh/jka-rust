// PORT-COMPLETE: AnimalNPC.c 2/7
//! FAITHFUL port of `oracle/oracle/codemp/game/AnimalNPC.c` (MP `_JK2MP` +
//! `QAGAME` compile path).
//!
//! Generated from the `fnskel.py` signature skeleton; bodies transcribed per
//! the settled jampgame fork rulings. STAGING ONLY — not yet wired into
//! crates/.
//!
//! Parking pattern in this file (mirrors `SpeederNPC.rs`/`g_vehicles.rs`):
//! - `raw-ptr-skeleton-no-world-handle` / `ambient-global (level.time)`: reads
//!   `level.time`, unreachable from the raw-pointer skeleton signature
//!   (rulings item 1: `level` lives on the world).
//! - `ambient-global (g_vehicleInfo)`: reads the file-static `g_vehicleInfo`
//!   table to dispatch the base vehicle-type vtable.
//! - `bg-dep (vehicleInfo_t)`: dereferences `Vehicle_t::m_pVehicleInfo`, which
//!   is still a `*mut c_void` placeholder (`//TODO: Port vehicleInfo_t`,
//!   `bg_vehicles.h:586`) pending that type's pointer-field port.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

// Raven angle-vector index (`q_shared.h`): YAW=1.
const YAW: usize = 1;

/// Raven `DeathUpdate` — update death sequence.
///
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:97-148`
pub fn DeathUpdate(ctx: GameContext<'_>, pVeh: *mut Vehicle_t) {
    unsafe {
        let level_time = crate::g_main::level_time();
        if level_time >= (*pVeh).m_iDieTime {
            // If the vehicle is not empty. (Fork-7: `Inhabited`/`EjectAll` have
            // no Animal override, so dispatch resolves to the generic base.)
            if crate::veh_dispatch::inhabited(pVeh) != qfalse {
                crate::veh_dispatch::eject_all(ctx, pVeh);
            }
        }
    }
}

/// Raven `Update` — like a think or move command, this updates various
/// vehicle properties.
///
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:151-154`
pub fn Update(ctx: GameContext<'_>, pVeh: *mut Vehicle_t, pUcmd: *const usercmd_t) -> qboolean {
    unsafe {
        // Fork-7: Animal `Update` delegates to the generic base body.
        crate::g_vehicles::Update(ctx, pVeh, pUcmd)
    }
}

/// `ProcessMoveCommands` the Vehicle.
///
/// Raven: MP RULE - ALL PROCESSMOVECOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
/// If you really need to violate this rule for SP, then use ifdefs.
/// By BG-compatible, I mean no use of game-specific data - ONLY use
/// stuff available in the MP bgEntity.
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:168-329`
pub extern "C" fn ProcessMoveCommands(pVeh: *mut Vehicle_t) {
    unsafe {
        let mut speedInc: f32;
        let mut speedIdleDec: f32;
        let speedIdle: f32;
        let mut speedIdleAccel: f32;
        let speedMin: f32;
        let mut speedMax: f32;
        let fWalkSpeedMax: f32;
        let curTime: c_int = crate::g_main::level_time();

        let parent = (*pVeh).m_pParentEntity;
        let parentPS = (*parent).playerState;

        speedIdleDec = (*(*pVeh).m_pVehicleInfo).decelIdle * (*pVeh).m_fTimeModifier;
        speedMax = (*(*pVeh).m_pVehicleInfo).speedMax;
        speedIdle = (*(*pVeh).m_pVehicleInfo).speedIdle;
        speedIdleAccel = (*(*pVeh).m_pVehicleInfo).accelIdle * (*pVeh).m_fTimeModifier;
        speedMin = (*(*pVeh).m_pVehicleInfo).speedMin;

        if !(*pVeh).m_pPilot.is_null()
            && ((*pVeh).m_ucmd.buttons & BUTTON_ALT_ATTACK) != 0
            && (*(*pVeh).m_pVehicleInfo).turboSpeed > 0.0f {
            if (curTime - (*pVeh).m_iTurboTime) > (*(*pVeh).m_pVehicleInfo).turboRecharge {
                (*pVeh).m_iTurboTime = curTime + (*(*pVeh).m_pVehicleInfo).turboDuration;
                (*parentPS).speed = (*(*pVeh).m_pVehicleInfo).turboSpeed;
            }
        }

        if curTime < (*pVeh).m_iTurboTime {
            speedMax = (*(*pVeh).m_pVehicleInfo).turboSpeed;
        } else {
            speedMax = (*(*pVeh).m_pVehicleInfo).speedMax;
        }

        if !(*parentPS).m_iVehicleNum == 0 {
            speedInc = speedIdle * (*pVeh).m_fTimeModifier;
            crate::q_math::VectorClear(&mut (*parentPS).moveDir);
            (*parentPS).speed = 0.0f;
        } else {
            speedInc = (*(*pVeh).m_pVehicleInfo).acceleration * (*pVeh).m_fTimeModifier;
        }

        if (*parentPS).speed != 0.0f || (*parentPS).groundEntityNum == ENTITYNUM_NONE as u32
            || (*pVeh).m_ucmd.forwardmove != 0 || (*pVeh).m_ucmd.upmove > 0 {
            if (*pVeh).m_ucmd.forwardmove > 0 && speedInc != 0.0f {
                (*parentPS).speed += speedInc;
            } else if (*pVeh).m_ucmd.forwardmove < 0 {
                if (*parentPS).speed > speedIdle {
                    (*parentPS).speed -= speedInc;
                } else if (*parentPS).speed > speedMin {
                    (*parentPS).speed -= speedIdleDec;
                }
            } else if (*parentPS).speed > 0.0f {
                (*parentPS).speed -= speedIdleDec;
                if (*parentPS).speed < 0.0f {
                    (*parentPS).speed = 0.0f;
                }
            } else if (*parentPS).speed < 0.0f {
                (*parentPS).speed += speedIdleDec;
                if (*parentPS).speed > 0.0f {
                    (*parentPS).speed = 0.0f;
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
        if curTime > (*pVeh).m_iTurboTime && ((*pVeh).m_ucmd.buttons & BUTTON_WALKING) != 0 && (*parentPS).speed > fWalkSpeedMax {
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
/// If you really need to violate this rule for SP, then use ifdefs.
/// By BG-compatible, I mean no use of game-specific data - ONLY use
/// stuff available in the MP bgEntity.
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:338-464`
pub extern "C" fn ProcessOrientCommands(pVeh: *mut Vehicle_t) {
    unsafe {
        let parent = (*pVeh).m_pParentEntity;
        let parentPS = (*parent).playerState;

        let rider = if (*parent).s.owner != ENTITYNUM_NONE as u32 {
            crate::bg_pmove::PM_BGEntForNum((*parent).s.owner as c_int)
        } else {
            core::ptr::null_mut()
        };

        if rider.is_null() || rider == parent as *mut bgEntity_t {
            let rider_ent = parent;
            let riderPS = (*rider_ent).playerState;

            if !rider.is_null() {
                let mut angDif =
                    crate::q_math::AngleSubtract(
                        (*pVeh).m_vOrientation[YAW],
                        (*riderPS).viewangles[YAW],
                    );
                if !parentPS.is_null() && (*parentPS).speed > 0.0f {
                    let mut s = (*parentPS).speed;
                    let maxDif = (*(*pVeh).m_pVehicleInfo).turningSpeed * 4.0f32;
                    if s < 0.0f {
                        s = -s;
                    }
                    angDif *= s / (*(*pVeh).m_pVehicleInfo).speedMax;
                    if angDif > maxDif {
                        angDif = maxDif;
                    } else if angDif < -maxDif {
                        angDif = -maxDif;
                    }
                    (*pVeh).m_vOrientation[YAW] = crate::q_math::AngleNormalize180(
                        (*pVeh).m_vOrientation[YAW] - angDif * ((*pVeh).m_fTimeModifier * 0.2f32),
                    );
                }
            }
        } else {
            let riderPS = (*rider).playerState;
            if !rider.is_null() {
                let mut angDif =
                    crate::q_math::AngleSubtract(
                        (*pVeh).m_vOrientation[YAW],
                        (*riderPS).viewangles[YAW],
                    );
                if !parentPS.is_null() && (*parentPS).speed > 0.0f {
                    let mut s = (*parentPS).speed;
                    let maxDif = (*(*pVeh).m_pVehicleInfo).turningSpeed * 4.0f32;
                    if s < 0.0f {
                        s = -s;
                    }
                    angDif *= s / (*(*pVeh).m_pVehicleInfo).speedMax;
                    if angDif > maxDif {
                        angDif = maxDif;
                    } else if angDif < -maxDif {
                        angDif = -maxDif;
                    }
                    (*pVeh).m_vOrientation[YAW] = crate::q_math::AngleNormalize180(
                        (*pVeh).m_vOrientation[YAW] - angDif * ((*pVeh).m_fTimeModifier * 0.2f32),
                    );
                }
            }
        }
    }
}

/// Raven `AnimalProcessOri` — temp hack til mp speeder controls are sorted
/// (`_JK2MP` only).
///
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:467-470`
pub fn AnimalProcessOri(
    ctx: GameContext<'_>,pVeh: *mut Vehicle_t) {
    ProcessOrientCommands(pVeh);
}

/// Raven `AnimateVehicle`.
///
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:474-615`
pub extern "C" fn AnimateVehicle(pVeh: *mut Vehicle_t) {
    unsafe {
        let mut anim: animNumber_t = BOTH_VT_IDLE;
        let mut iFlags: c_int = SETANIM_FLAG_NORMAL;
        let mut iBlend: c_int = 300;
        let pilot = (*pVeh).m_pPilot as *mut gentity_t;
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let level_time = crate::g_main::level_time();

        // We're dead.
        if (*parent).health <= 0 {
            return;
        }

        // If they're bucking, play the animation and leave...
        if (*parent).client.is_null() == false && (*(*parent).client).ps.legsAnim == BOTH_VT_BUCK {
            if (*(*parent).client).ps.legsAnimTimer <= 0 {
                (*pVeh).m_ulFlags &= !VEH_BUCKING;
            } else {
                return;
            }
        } else if ((*pVeh).m_ulFlags & VEH_BUCKING) != 0 {
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
            anim = BOTH_VT_BUCK;
            iBlend = 500;
            Vehicle_SetAnim(parent, SETANIM_LEGS, BOTH_VT_BUCK, iFlags, iBlend);
            return;
        }

        // Boarding animation.
        if (*pVeh).m_iBoarding != 0 {
            if (*pVeh).m_iBoarding < 0 {
                let mut iAnimLen: c_int;

                if (*pVeh).m_iBoarding == -1 {
                    anim = BOTH_VT_MOUNT_L;
                } else if (*pVeh).m_iBoarding == -2 {
                    anim = BOTH_VT_MOUNT_R;
                } else if (*pVeh).m_iBoarding == -3 {
                    anim = BOTH_VT_MOUNT_B;
                }

                iAnimLen = (crate::bg_panimate::BG_AnimLength(
                    (*parent).localAnimIndex,
                    anim,
                ) as f32 * 0.7f32) as c_int;
                (*pVeh).m_iBoarding = level_time + iAnimLen;

                iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
                Vehicle_SetAnim(parent, SETANIM_LEGS, anim, iFlags, iBlend);
                if !pilot.is_null() {
                    Vehicle_SetAnim(pilot, SETANIM_BOTH, anim, iFlags, iBlend);
                }
                return;
            } else if (*pVeh).m_iBoarding <= level_time {
                (*pVeh).m_iBoarding = 0;
            }
        }

        let fSpeedPercToMax = if !(*parent).client.is_null() {
            (*(*parent).client).ps.speed / (*(*pVeh).m_pVehicleInfo).speedMax
        } else {
            0.0f
        };

        if fSpeedPercToMax < -0.01f32 {
            anim = BOTH_VT_WALK_REV;
            iBlend = 600;
        } else {
            let turbo = fSpeedPercToMax > 0.0f && level_time < (*pVeh).m_iTurboTime;
            let walking = if !(*parent).client.is_null() {
                fSpeedPercToMax > 0.0f
                    && (((*pVeh).m_ucmd.buttons & BUTTON_WALKING) != 0 || fSpeedPercToMax <= 0.275f)
            } else {
                false
            };
            let running = fSpeedPercToMax > 0.275f32;

            (*pVeh).m_ulFlags &= !VEH_CRASHING;

            if turbo {
                iBlend = 50;
                iFlags = SETANIM_FLAG_OVERRIDE;
                anim = BOTH_VT_TURBO;
            } else {
                iBlend = 300;
                iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLDLESS;
                anim = if walking {
                    BOTH_VT_WALK_FWD
                } else if running {
                    BOTH_VT_RUN_FWD
                } else {
                    BOTH_VT_IDLE1
                };
            }
        }

        Vehicle_SetAnim(parent, SETANIM_LEGS, anim, iFlags, iBlend);
    }
}

/// Raven `AnimateRiders` — makes sure the riders in this vehicle are
/// properly animated.
///
/// Raven: rwwFIXMEFIXME: This is all going to have to be predicted I think,
/// or it will feel awful and lagged.
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:620-849`
pub extern "C" fn AnimateRiders(pVeh: *mut Vehicle_t) {
    unsafe {
        let mut anim: animNumber_t = BOTH_VT_IDLE;
        let mut iFlags: c_int = SETANIM_FLAG_NORMAL;
        let mut iBlend: c_int = 500;
        let pilot = (*pVeh).m_pPilot as *mut gentity_t;
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let pilotPS = (*pVeh).m_pPilot.as_ref().map(|p| (*p).playerState);
        let parentPS = (*parent).playerState;
        let level_time = crate::g_main::level_time();

        // Boarding animation.
        if (*pVeh).m_iBoarding != 0 {
            return;
        }

        let fSpeedPercToMax = if !(*parent).client.is_null() {
            (*(*parent).client).ps.speed / (*(*pVeh).m_pVehicleInfo).speedMax
        } else {
            0.0f
        };

        if fSpeedPercToMax < -0.01f32 {
            anim = BOTH_VT_WALK_REV;
            iBlend = 600;
        } else {
            let hasWeapon = !pilotPS.is_none()
                && !pilotPS.unwrap().is_null()
                && (*pilotPS.unwrap()).weapon != WP_NONE as u32
                && (*pilotPS.unwrap()).weapon != WP_MELEE as u32;
            let attacking = hasWeapon
                && !pilotPS.is_none()
                && !pilotPS.unwrap().is_null()
                && ((*pilotPS.unwrap()).weaponTime > 0 || ((*pVeh).m_ucmd.buttons & BUTTON_ATTACK) != 0);
            let right = (*pVeh).m_ucmd.rightmove > 0;
            let left = (*pVeh).m_ucmd.rightmove < 0;
            let turbo = fSpeedPercToMax > 0.0f && level_time < (*pVeh).m_iTurboTime;
            let walking = fSpeedPercToMax > 0.0f
                && (((*pVeh).m_ucmd.buttons & BUTTON_WALKING) != 0 || fSpeedPercToMax <= 0.275f32);
            let running = fSpeedPercToMax > 0.275f32;
            let mut weapon_pose: EWeaponPose = WPOSE_NONE;

            (*pVeh).m_ulFlags &= !VEH_CRASHING;

            // Compute The Weapon Pose
            if !pilotPS.is_none() && !pilotPS.unwrap().is_null() {
                if (*pilotPS.unwrap()).weapon == WP_BLASTER as u32 {
                    weapon_pose = WPOSE_BLASTER;
                } else if (*pilotPS.unwrap()).weapon == WP_SABER as u32 {
                    if ((*pVeh).m_ulFlags & VEH_SABERINLEFTHAND) != 0
                        && (*pilotPS.unwrap()).torsoAnim == BOTH_VT_ATL_TO_R_S as u32
                    {
                        (*pVeh).m_ulFlags &= !VEH_SABERINLEFTHAND;
                    }
                    if ((*pVeh).m_ulFlags & VEH_SABERINLEFTHAND) == 0
                        && (*pilotPS.unwrap()).torsoAnim == BOTH_VT_ATR_TO_L_S as u32
                    {
                        (*pVeh).m_ulFlags |= VEH_SABERINLEFTHAND;
                    }
                    weapon_pose = if ((*pVeh).m_ulFlags & VEH_SABERINLEFTHAND) != 0 {
                        WPOSE_SABERLEFT
                    } else {
                        WPOSE_SABERRIGHT
                    };
                }
            }

            if attacking && weapon_pose != WPOSE_NONE {
                iBlend = 100;
                iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_RESTART;

                let mut right_mut = right;
                let mut left_mut = left;

                if turbo {
                    right_mut = true;
                    left_mut = false;
                }

                if !left_mut && !right_mut {
                    if !pilot.is_null() && !(*pilot).enemy.is_none() {
                        let to_enemy_dist: f32;
                        let mut to_enemy: [f32; 3] = [0.0f; 3];
                        let mut actor_right: [f32; 3] = [0.0f; 3];
                        let actor_right_dot: f32;

                        crate::q_math::_VectorSubtract(
                            (*pilot).r.currentOrigin,
                            (*(*pilot).enemy.unwrap()).r.currentOrigin,
                            &mut to_enemy,
                        );
                        to_enemy_dist = crate::q_math::VectorNormalize(&mut to_enemy);

                        crate::q_math::AngleVectors(
                            (*parent).r.currentAngles,
                            core::ptr::null_mut(),
                            &mut actor_right,
                            core::ptr::null_mut(),
                        );
                        actor_right_dot = crate::q_math::_DotProduct(to_enemy, actor_right);

                        if actor_right_dot.abs() > 0.5f32 || !pilotPS.is_none() && !pilotPS.unwrap().is_null() && (*pilotPS.unwrap()).weapon == WP_SABER as u32 {
                            left_mut = actor_right_dot > 0.0f32;
                            right_mut = !left_mut;
                        } else {
                            right_mut = false;
                            left_mut = false;
                        }
                    } else if !pilotPS.is_none()
                        && !pilotPS.unwrap().is_null()
                        && (*pilotPS.unwrap()).weapon == WP_SABER as u32
                        && !left_mut
                        && !right_mut
                    {
                        left_mut = weapon_pose == WPOSE_SABERLEFT;
                        right_mut = !left_mut;
                    }
                }

                if left_mut {
                    anim = match weapon_pose {
                        WPOSE_BLASTER => BOTH_VT_ATL_G,
                        WPOSE_SABERLEFT => BOTH_VT_ATL_S,
                        WPOSE_SABERRIGHT => BOTH_VT_ATR_TO_L_S,
                        _ => BOTH_VT_ATL_G,
                    };
                } else if right_mut {
                    anim = match weapon_pose {
                        WPOSE_BLASTER => BOTH_VT_ATR_G,
                        WPOSE_SABERLEFT => BOTH_VT_ATL_TO_R_S,
                        WPOSE_SABERRIGHT => BOTH_VT_ATR_S,
                        _ => BOTH_VT_ATR_G,
                    };
                } else {
                    anim = match weapon_pose {
                        WPOSE_BLASTER => BOTH_VT_ATF_G,
                        _ => BOTH_VT_ATF_G,
                    };
                }
            } else if turbo {
                iBlend = 50;
                iFlags = SETANIM_FLAG_OVERRIDE;
                anim = BOTH_VT_TURBO;
            } else {
                iBlend = 300;
                iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLDLESS;

                if weapon_pose == WPOSE_NONE {
                    anim = if walking {
                        BOTH_VT_WALK_FWD
                    } else if running {
                        BOTH_VT_RUN_FWD
                    } else {
                        BOTH_VT_IDLE1
                    };
                } else {
                    anim = match weapon_pose {
                        WPOSE_BLASTER => BOTH_VT_IDLE_G,
                        WPOSE_SABERLEFT => BOTH_VT_IDLE_SL,
                        WPOSE_SABERRIGHT => BOTH_VT_IDLE_SR,
                        _ => BOTH_VT_IDLE1,
                    };
                }
            }
        }

        Vehicle_SetAnim(pilot, SETANIM_BOTH, anim, iFlags, iBlend);
    }
}

/// Raven `G_SetAnimalVehicleFunctions` — on the client this function will
/// only set up the process command funcs.
///
/// Raven: installs this file's vehicle-vtable functions onto a
/// `vehicleInfo_t` (ruling 7: enum-over-vehicle-type dispatch lives in the
/// caller — `bg_vehicleLoad.rs` matches on `vehicleType_t::VH_ANIMAL` — this
/// fn just fills the already-ported `Option<unsafe extern "C" fn(...)>`
/// vtable fields directly). Only the `#ifdef QAGAME` (game-side) and shared
/// assignments are live for jampgame; the `#ifndef QAGAME` (cgame
/// `AttachRidersGeneric`) arm is dead here.
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:857-887`
pub fn G_SetAnimalVehicleFunctions(pVehInfo: *mut vehicleInfo_t) {
    unsafe {
        (*pVehInfo).AnimateVehicle = Some(AnimateVehicle);
        (*pVehInfo).AnimateRiders = Some(AnimateRiders);
        (*pVehInfo).DeathUpdate = Some(DeathUpdate);
        (*pVehInfo).Update = Some(Update);

        // shared
        (*pVehInfo).ProcessMoveCommands = Some(ProcessMoveCommands);
        (*pVehInfo).ProcessOrientCommands = Some(ProcessOrientCommands);
    }
}

/// Raven `G_CreateAnimalNPC` — create/allocate a new Animal Vehicle
/// (initializing it as well).
///
/// Raven: this is a BG function too in MP so don't un-bg-compatibilify it.
/// Source: `oracle/oracle/codemp/game/AnimalNPC.c:904-925`
pub fn G_CreateAnimalNPC(
    ctx: GameContext<'_>,
    pVeh: *mut *mut Vehicle_t,
    strAnimalType: *const c_char,
) {
    unsafe {
        crate::g_utils::G_AllocateVehicleObject(ctx, pVeh);
        core::ptr::write_bytes(*pVeh as *mut u8, 0, core::mem::size_of::<Vehicle_t>());
        (*(*pVeh)).m_pVehicleInfo = &mut (*ctx.world).bg_state.g_vehicleInfo
            [crate::bg_vehicleLoad::BG_VehicleGetIndex(strAnimalType) as usize];
    }
}
