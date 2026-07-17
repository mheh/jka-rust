// PORT-COMPLETE: AnimalNPC.c
//! FAITHFUL port of `oracle/codemp/game/AnimalNPC.c` (MP `_JK2MP` +
//! `QAGAME` compile path).
//!
//! Filled by the jampgame mega-pass.
//!
//! Parking pattern in this file (mirrors `SpeederNPC.rs`/`g_vehicles.rs`):
//! - `raw-ptr-skeleton-no-world-handle` / `ambient-global (level.time)`: reads
//!   `level.time`, unreachable from the raw-pointer skeleton signature
//!   (rulings item 1: `level` lives on the world).
//! - `ambient-global (g_vehicleInfo)`: reads the file-static `g_vehicleInfo`
//!   table to dispatch the base vehicle-type vtable.
//! - `bg-dep (vehicleInfo_t)`: dereferences `Vehicle_t::m_pVehicleInfo`
//!   (`*mut vehicleInfo_t`) to read the base vehicle-type stats table.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_main::level_time;
use crate::prelude::*;

// `YAW` (angle-vector index) comes from the prelude (`crate::q_math::YAW`).

/// Raven `DeathUpdate` — update death sequence.
///
/// Source: `oracle/codemp/game/AnimalNPC.c:97-148`
pub fn DeathUpdate(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let level_time = level_time(ctx);
        if level_time >= (*pVeh).m_iDieTime {
            // If the vehicle is not empty. (`Inhabited`/`EjectAll` have
            // no Animal override, so dispatch resolves to the generic base.)
            if crate::veh_dispatch::inhabited(ctx, pVeh) != qfalse {
                crate::veh_dispatch::eject_all(ctx, pVeh);
            }
        }
    }
}

/// Raven `Update` — like a think or move command, this updates various
/// vehicle properties.
///
/// Source: `oracle/codemp/game/AnimalNPC.c:151-154`
pub fn Update(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pUcmd: *const usercmd_t) -> qboolean {
    // Animal `Update` delegates to the generic base body.
    crate::g_vehicles::Update(ctx, pVeh, pUcmd)
}

/// `ProcessMoveCommands` the Vehicle.
///
/// Raven: MP RULE - ALL PROCESSMOVECOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
/// If you really need to violate this rule for SP, then use ifdefs.
/// By BG-compatible, I mean no use of game-specific data - ONLY use
/// stuff available in the MP bgEntity.
/// Source: `oracle/codemp/game/AnimalNPC.c:168-329`
pub fn ProcessMoveCommands(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let mut speedInc: f32;
        let mut speedIdleDec: f32;
        let speedIdle: f32;
        let mut speedIdleAccel: f32;
        let speedMin: f32;
        let mut speedMax: f32;
        let fWalkSpeedMax: f32;
        let curTime: c_int = level_time(ctx);

        // `pVeh` (Vehicle_t) and its `m_pVehicleInfo` (bg vehicleInfo_t) are pool
        // objects with no accessor — their fields stay raw. `m_pParentEntity` is a
        // g_entities arena entity: recover its handle and read `playerState` (a
        // pool-client ptr, derefed raw below) through the accessor.
        let parent = (*pVeh).m_pParentEntity;
        let parent_id = ctx.entity_id_of(parent as *const gentity_t).unwrap();
        let parentPS = ctx.world.entity(parent_id).playerState;

        speedIdleDec = (*(*pVeh).m_pVehicleInfo).decelIdle * (*pVeh).m_fTimeModifier;
        speedMax = (*(*pVeh).m_pVehicleInfo).speedMax;
        speedIdle = (*(*pVeh).m_pVehicleInfo).speedIdle;
        speedIdleAccel = (*(*pVeh).m_pVehicleInfo).accelIdle * (*pVeh).m_fTimeModifier;
        speedMin = (*(*pVeh).m_pVehicleInfo).speedMin;

        if !(*pVeh).m_pPilot.is_null()
            && ((*pVeh).m_ucmd.buttons & BUTTON_ALT_ATTACK) != 0
            && (*(*pVeh).m_pVehicleInfo).turboSpeed > 0.0f32
        {
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

        if (*parentPS).m_iVehicleNum == 0 {
            speedInc = speedIdle * (*pVeh).m_fTimeModifier;
            crate::q_math::VectorClear(&mut (*parentPS).moveDir);
            (*parentPS).speed = 0.0f32;
        } else {
            speedInc = (*(*pVeh).m_pVehicleInfo).acceleration * (*pVeh).m_fTimeModifier;
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
/// If you really need to violate this rule for SP, then use ifdefs.
/// By BG-compatible, I mean no use of game-specific data - ONLY use
/// stuff available in the MP bgEntity.
/// Source: `oracle/codemp/game/AnimalNPC.c:338-464`
pub fn ProcessOrientCommands(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    unsafe {
        // `pVeh` (Vehicle_t) and `m_pVehicleInfo` have no accessor — raw. `parent`
        // and the owner-derived `rider` are g_entities arena entities: read their
        // `playerState` (pool-client ptrs, derefed raw) through the accessor.
        let parent = (*pVeh).m_pParentEntity;
        let parent_id = ctx.entity_id_of(parent as *const gentity_t).unwrap();

        // Oracle `_JK2MP`: `if (owner != ENTITYNUM_NONE) rider =
        // PM_BGEntForNum(owner);` (== `&g_entities[owner]`) then `if (!rider)
        // rider = parent;`. `EntityId::from_num` maps NONE → the parent fallback.
        let owner = ctx.world.entity(parent_id).s.owner;
        let rider_id = EntityId::from_num(owner).unwrap_or(parent_id);

        let parentPS = ctx.world.entity(parent_id).playerState;
        let riderPS = ctx.world.entity(rider_id).playerState;

        // Oracle: `if (rider)` — always true after the fallback above.
        let mut angDif = crate::q_math::AngleSubtract(
            *(*pVeh).m_vOrientation.add(YAW),
            (*riderPS).viewangles[YAW],
        );
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
            *(*pVeh).m_vOrientation.add(YAW) = crate::q_math::AngleNormalize180(
                *(*pVeh).m_vOrientation.add(YAW) - angDif * ((*pVeh).m_fTimeModifier * 0.2f32),
            );
        }
    }
}

/// Raven `AnimalProcessOri` — temp hack til mp speeder controls are sorted
/// (`_JK2MP` only).
///
/// Source: `oracle/codemp/game/AnimalNPC.c:467-470`
pub fn AnimalProcessOri(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    ProcessOrientCommands(ctx, pVeh);
}

/// Raven `AnimateVehicle`.
///
/// Source: `oracle/codemp/game/AnimalNPC.c:474-615`
pub fn AnimateVehicle(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let mut anim: animNumber_t = BOTH_VT_IDLE;
        let mut iFlags: c_int = SETANIM_FLAG_NORMAL;
        let mut iBlend: c_int = 300;
        let pilot = (*pVeh).m_pPilot as *mut gentity_t;
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let parent_id = ctx.entity_id_of(parent).unwrap();
        let level_time = level_time(ctx);

        // We're dead.
        if ctx.world.entity(parent_id).health <= 0 {
            return;
        }

        // If they're bucking, play the animation and leave... `parent->client` is
        // a pool-allocated gclient_t for vehicle NPCs: read the ptr via the
        // accessor, deref it raw (recipe 2c pool-client).
        let parent_client = ctx.world.entity(parent_id).client;
        if parent_client.is_null() == false && (*parent_client).ps.legsAnim == BOTH_VT_BUCK as c_int
        {
            if (*parent_client).ps.legsTimer <= 0 {
                (*pVeh).m_ulFlags &= !(VEH_BUCKING as u64);
            } else {
                return;
            }
        } else if ((*pVeh).m_ulFlags & (VEH_BUCKING as u64)) != 0 {
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
            anim = BOTH_VT_BUCK;
            iBlend = 500;
            Vehicle_SetAnim(
                ctx,
                ctx.entity_id_of(parent).unwrap(),
                SETANIM_LEGS,
                BOTH_VT_BUCK as c_int,
                iFlags,
                iBlend,
            );
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

                let local_anim_index = ctx.world.entity(parent_id).localAnimIndex;
                iAnimLen = (mp_bg::bg_panimate::BG_AnimLength(
                    &ctx.world.bg_state,
                    local_anim_index,
                    anim as c_int,
                ) as f32
                    * 0.7f32) as c_int;
                (*pVeh).m_iBoarding = level_time + iAnimLen;

                iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
                Vehicle_SetAnim(
                    ctx,
                    ctx.entity_id_of(parent).unwrap(),
                    SETANIM_LEGS,
                    anim as c_int,
                    iFlags,
                    iBlend,
                );
                if !pilot.is_null() {
                    Vehicle_SetAnim(
                        ctx,
                        ctx.entity_id_of(pilot).unwrap(),
                        SETANIM_BOTH,
                        anim as c_int,
                        iFlags,
                        iBlend,
                    );
                }
                return;
            } else if (*pVeh).m_iBoarding <= level_time {
                (*pVeh).m_iBoarding = 0;
            }
        }

        let parent_client = ctx.world.entity(parent_id).client;
        let fSpeedPercToMax = if !parent_client.is_null() {
            (*parent_client).ps.speed / (*(*pVeh).m_pVehicleInfo).speedMax
        } else {
            0.0f32
        };

        if fSpeedPercToMax < -0.01f32 {
            anim = BOTH_VT_WALK_REV;
            iBlend = 600;
        } else {
            let turbo = fSpeedPercToMax > 0.0f32 && level_time < (*pVeh).m_iTurboTime;
            let walking = if !parent_client.is_null() {
                fSpeedPercToMax > 0.0f32
                    && (((*pVeh).m_ucmd.buttons & BUTTON_WALKING) != 0
                        || fSpeedPercToMax <= 0.275f32)
            } else {
                false
            };
            let running = fSpeedPercToMax > 0.275f32;

            (*pVeh).m_ulFlags &= !(VEH_CRASHING as u64);

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

        Vehicle_SetAnim(
            ctx,
            ctx.entity_id_of(parent).unwrap(),
            SETANIM_LEGS,
            anim as c_int,
            iFlags,
            iBlend,
        );
    }
}

/// Raven `AnimateRiders` — makes sure the riders in this vehicle are
/// properly animated.
///
/// Raven: rwwFIXMEFIXME: This is all going to have to be predicted I think,
/// or it will feel awful and lagged.
/// Source: `oracle/codemp/game/AnimalNPC.c:620-849`
pub fn AnimateRiders(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let mut anim: animNumber_t = BOTH_VT_IDLE;
        let mut iFlags: c_int = SETANIM_FLAG_NORMAL;
        let mut iBlend: c_int = 500;
        let pilot = (*pVeh).m_pPilot as *mut gentity_t;
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let parent_id = ctx.entity_id_of(parent).unwrap();
        // `pilot` may be NULL; when present it's an arena entity — read its
        // `playerState` (pool-client ptr, derefed raw below) via the accessor.
        let pilotPS = if pilot.is_null() {
            None
        } else {
            Some(
                ctx.world
                    .entity(ctx.entity_id_of(pilot).unwrap())
                    .playerState,
            )
        };
        let parentPS = ctx.world.entity(parent_id).playerState;
        let level_time = level_time(ctx);

        // Boarding animation.
        if (*pVeh).m_iBoarding != 0 {
            return;
        }

        let parent_client = ctx.world.entity(parent_id).client;
        let fSpeedPercToMax = if !parent_client.is_null() {
            (*parent_client).ps.speed / (*(*pVeh).m_pVehicleInfo).speedMax
        } else {
            0.0f32
        };

        // MP `#ifdef _JK2MP` guards the reverse-anim branch as `if (0)` (reverse
        // is handled in pmove in MP), so the else block always runs here.
        if false {
            anim = BOTH_VT_WALK_REV;
            iBlend = 600;
        } else {
            let hasWeapon = !pilotPS.is_none()
                && !pilotPS.unwrap().is_null()
                && (*pilotPS.unwrap()).weapon != WP_NONE
                && (*pilotPS.unwrap()).weapon != WP_MELEE;
            let attacking = hasWeapon && ((*pVeh).m_ucmd.buttons & BUTTON_ATTACK) != 0;
            let right = (*pVeh).m_ucmd.rightmove > 0;
            let left = (*pVeh).m_ucmd.rightmove < 0;
            let turbo = fSpeedPercToMax > 0.0f32 && level_time < (*pVeh).m_iTurboTime;
            let walking = fSpeedPercToMax > 0.0f32
                && (((*pVeh).m_ucmd.buttons & BUTTON_WALKING) != 0 || fSpeedPercToMax <= 0.275f32);
            let running = fSpeedPercToMax > 0.275f32;
            let mut weapon_pose: EWeaponPose = WPOSE_NONE;

            (*pVeh).m_ulFlags &= !(VEH_CRASHING as u64);

            // MP `#ifdef _JK2MP`: don't interrupt attack anims — if a shot is
            // mid-fire the current anim is left untouched (skips pose + SetAnim).
            if let Some(pps) = pilotPS {
                if !pps.is_null() && (*pps).weaponTime > 0 {
                    return;
                }
            }

            // Compute The Weapon Pose
            if !pilotPS.is_none() && !pilotPS.unwrap().is_null() {
                if (*pilotPS.unwrap()).weapon == WP_BLASTER {
                    weapon_pose = WPOSE_BLASTER;
                } else if (*pilotPS.unwrap()).weapon == WP_SABER {
                    if ((*pVeh).m_ulFlags & (VEH_SABERINLEFTHAND as u64)) != 0
                        && (*pilotPS.unwrap()).torsoAnim == BOTH_VT_ATL_TO_R_S as c_int
                    {
                        (*pVeh).m_ulFlags &= !(VEH_SABERINLEFTHAND as u64);
                    }
                    if ((*pVeh).m_ulFlags & (VEH_SABERINLEFTHAND as u64)) == 0
                        && (*pilotPS.unwrap()).torsoAnim == BOTH_VT_ATR_TO_L_S as c_int
                    {
                        (*pVeh).m_ulFlags |= (VEH_SABERINLEFTHAND as u64);
                    }
                    weapon_pose = if ((*pVeh).m_ulFlags & (VEH_SABERINLEFTHAND as u64)) != 0 {
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
                    // The enemy-direction auto-aim block is `#ifndef _JK2MP` — dead
                    // in MP; only the WP_SABER fallback survives preprocessing.
                    // Source: oracle/codemp/game/AnimalNPC.c:746-777
                    if !pilotPS.is_none()
                        && !pilotPS.unwrap().is_null()
                        && (*pilotPS.unwrap()).weapon == WP_SABER
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

        Vehicle_SetAnim(
            ctx,
            ctx.entity_id_of(pilot).unwrap(),
            SETANIM_BOTH,
            anim as c_int,
            iFlags,
            iBlend,
        );
    }
}

// `G_SetAnimalVehicleFunctions` retired — it only assigned the now-removed
// `vehicleInfo_t` fn-ptr slots. Vehicle dispatch is `vehicleType_t`-keyed in
// `crate::veh_dispatch`. Source: see per-class setter in the oracle .c.

/// Raven `G_CreateAnimalNPC` — create/allocate a new Animal Vehicle
/// (initializing it as well).
///
/// Raven: this is a BG function too in MP so don't un-bg-compatibilify it.
/// Source: `oracle/codemp/game/AnimalNPC.c:904-925`
pub fn G_CreateAnimalNPC(
    ctx: &mut GameContext,
    pVeh: *mut *mut Vehicle_t,
    strAnimalType: *const c_char,
) {
    unsafe {
        crate::g_utils::G_AllocateVehicleObject(ctx, pVeh);
        core::ptr::write_bytes(*pVeh as *mut u8, 0, core::mem::size_of::<Vehicle_t>());
        let mut callbacks = crate::bg_channel::GameCallbacksImpl {
            // SEAM-BG-REENTRY (DEC-28, sanctioned) — GameCallbacksImpl.world is a `*mut GameWorld`
            // field aliasing bg_state; a raw store is required (bg-seam re-entry).
            world: ctx.world_raw(),
            engine: ctx.engine,
        };
        let veh_index = mp_bg::bg_vehicleLoad::BG_VehicleGetIndex(
            strAnimalType,
            &mut ctx.world.bg_state,
            &crate::bg_channel::GameBgTraps::new(ctx.engine),
            &mut callbacks,
        ) as usize;
        (*(*pVeh)).m_pVehicleInfo = &mut (&mut ctx.world.bg_state.g_vehicleInfo)[veh_index];
    }
}
