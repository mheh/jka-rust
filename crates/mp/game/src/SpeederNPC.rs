// PORT-COMPLETE: SpeederNPC.c 3/5
//! FAITHFUL port of `oracle/codemp/game/SpeederNPC.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_vehicles::{VEH_MOUNT_THROW_LEFT, VEH_MOUNT_THROW_RIGHT};
use crate::prelude::*;
use core::ffi::c_int;

// Vehicle flags (`vehFlags_t`), buttons (`BUTTON_*`), weapons (`weapon_t`),
// entity effects (`EF_*`), set-anim flags (`SETANIM_FLAG_*`), and orientation
// indices (`PITCH`/`YAW`/`ROLL`) all resolve to their canonical workspace
// definitions through `crate::prelude::*`. The former per-file placeholder
// consts here carried guessed values (e.g. VEH_SLIDEBREAKING = 0x4 instead of
// the real 0x80, and the SETANIM_FLAG_* bits were all off by 8x) and shadowed
// the canonical items with live-buggy numbers, so they were removed.
//
// `vehFlags_t` variants are `#[repr(i32)]`, so use sites masking `m_ulFlags`
// (u64) cast with `as u64`, matching bg_pmove.rs / AnimalNPC.rs / g_vehicles.rs.

// `animNumber_t`/`BOTH_VS_IDLE`/… are the canonical `mp_bg::public::anim_number`
// enum + variants, reached via the prelude glob. The former per-file
// placeholder `AnimNum` enum (+ `use AnimNum::*;`) duplicated those variant
// names, causing a glob-glob ambiguity with the canonical import at every
// call site through `crate::prelude::*` (porting-rules §E dedupe-at-import
// rule).

// `PITCH`/`YAW`/`ROLL` (q_math) and `MAX_VEHICLE_EXHAUSTS` (vehicle_s) come from
// the prelude. `STRAFERAM_DURATION`/`STRAFERAM_ANGLE` (oracle SpeederNPC.c:97-98,
// both = 8) are used only inside the `#ifndef _JK2MP` SP-only strafe-ram code,
// which is dead in the `_JK2MP` MP build, so they are not defined here.

/// Raven `VEH_StartStrafeRam`.
///
/// Raven: the `_JK2MP` build of this function is a stub — the strafe-ram
/// mechanic is SP-only (`#ifndef _JK2MP` guards the real implementation at
/// SpeederNPC.c:102-138); the MP build always returns `qfalse`.
/// Source: `oracle/codemp/game/SpeederNPC.c:140-143`
pub fn VEH_StartStrafeRam(pVeh: *mut Vehicle_t, Right: qboolean, Duration: c_int) -> qboolean {
    0 // qfalse
}

/// Raven `Update`.
///
/// Raven: the `_JK2MP` build of this function is essentially a thin wrapper
/// around the base vehicle's Update method; the movement-direction, strafe-ram,
/// exhaust, and armor-effects code (lines 163-264) is guarded by `#ifndef _JK2MP`
/// and is SP-only dead code, dropped per porting-rules §10.
/// Source: `oracle/codemp/game/SpeederNPC.c:149-268`
pub fn Update(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pUcmd: *const usercmd_t) -> qboolean {
    unsafe {
        // `g_vehicleInfo[VEHICLE_BASE].Update` — the generic base body.
        if crate::g_vehicles::Update(ctx, pVeh, pUcmd) == qfalse {
            return qfalse;
        }

        // See whether this vehicle should be exploding.
        if (*pVeh).m_iDieTime != 0 {
            // `pVeh->m_pVehicleInfo->DeathUpdate(pVeh)` — Speeder's DeathUpdate slot is
            // commented in oracle setup, so this resolves to the base DeathUpdate.
            crate::veh_dispatch::death_update(ctx, pVeh);
        }

        // The rest of the function (movement direction, strafe ram, exhaust, armor effects)
        // is guarded by #ifndef _JK2MP and is SP-only code, dead in the MP build.

        qtrue
    }
}

/// `ProcessMoveCommands` the Vehicle.
///
/// Raven: MP RULE - ALL PROCESSMOVECOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
/// Source: `oracle/codemp/game/SpeederNPC.c:278-490`
pub fn ProcessMoveCommands(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
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

        // Get player states from parent and pilot
        // PORT-NOTE(vtable-access): m_pParentEntity.playerState access requires entity dereferencing
        parentPS = (*(*pVeh).m_pParentEntity).playerState;
        if !(*pVeh).m_pPilot.is_null() {
            pilotPS = (*(*pVeh).m_pPilot).playerState;
        }

        // Determine speed increment based on flying status
        if (*pVeh).m_ulFlags & (VEH_FLYING as u64) != 0 {
            speedInc = (*pVeh)
                .m_pVehicleInfo
                .as_ref()
                .map(|vi| vi.acceleration)
                .unwrap_or(0.0)
                * (*pVeh).m_fTimeModifier
                * 0.4f32;
        } else if (*parentPS).m_iVehicleNum == 0 {
            // Drifts to a stop. MP `#ifdef _JK2MP` branch is `!parentPS->m_iVehicleNum`
            // (the SP `#else` branch is `!Inhabited()`); these are not the same test.
            speedInc = 0.0f32;
        } else {
            speedInc = (*pVeh)
                .m_pVehicleInfo
                .as_ref()
                .map(|vi| vi.acceleration)
                .unwrap_or(0.0)
                * (*pVeh).m_fTimeModifier;
        }

        speedIdleDec = (*pVeh)
            .m_pVehicleInfo
            .as_ref()
            .map(|vi| vi.decelIdle)
            .unwrap_or(0.0)
            * (*pVeh).m_fTimeModifier;

        // QAGAME MP branch: `curTime = level.time`, reachable through `ctx`.
        curTime = (*ctx.world_raw()).level.time;

        // Handle turbo/acceleration
        if !(*pVeh).m_pPilot.is_null()
            && ((*pVeh).m_ucmd.buttons & BUTTON_ALT_ATTACK != 0)
            && (*pVeh)
                .m_pVehicleInfo
                .as_ref()
                .map(|vi| vi.turboSpeed)
                .unwrap_or(0.0)
                != 0.0
        {
            if ((!parentPS.is_null() && (*parentPS).electrifyTime > curTime)
                || (!pilotPS.is_null()
                    && ((*pilotPS).weapon == WP_MELEE
                        || ((*pilotPS).weapon == WP_SABER && BG_SabersOff(pilotPS) != 0))))
            {
                if (curTime - (*pVeh).m_iTurboTime)
                    > (*pVeh)
                        .m_pVehicleInfo
                        .as_ref()
                        .map(|vi| vi.turboRecharge)
                        .unwrap_or(0)
                {
                    (*pVeh).m_iTurboTime = curTime
                        + (*pVeh)
                            .m_pVehicleInfo
                            .as_ref()
                            .map(|vi| vi.turboDuration)
                            .unwrap_or(0);

                    if (*pVeh)
                        .m_pVehicleInfo
                        .as_ref()
                        .map(|vi| vi.iTurboStartFX)
                        .unwrap_or(0)
                        != 0
                    {
                        let mut i: c_int = 0;
                        while (i as usize) < MAX_VEHICLE_EXHAUSTS
                            && (*pVeh).m_iExhaustTag[i as usize] != -1
                        {
                            // PORT-NOTE(trap-access): G_PlayEffectID requires ctx/trap access
                            i += 1;
                        }
                    }

                    if !parentPS.is_null() {
                        (*parentPS).speed = (*pVeh)
                            .m_pVehicleInfo
                            .as_ref()
                            .map(|vi| vi.turboSpeed)
                            .unwrap_or(0.0) as f32;
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
            && (*(*pVeh).m_vOrientation.add(ROLL) as f32).abs() > 25.0f32
        {
            (*pVeh).m_ulFlags |= VEH_SLIDEBREAKING as u64;
        }

        // Determine speed max based on turbo
        if curTime < (*pVeh).m_iTurboTime {
            speedMax = (*pVeh)
                .m_pVehicleInfo
                .as_ref()
                .map(|vi| vi.turboSpeed)
                .unwrap_or(0.0) as f32;
            if !parentPS.is_null() {
                (*parentPS).eFlags |= EF_JETPACK_ACTIVE;
            }
        } else {
            speedMax = (*pVeh)
                .m_pVehicleInfo
                .as_ref()
                .map(|vi| vi.speedMax)
                .unwrap_or(0.0) as f32;
            if !parentPS.is_null() {
                (*parentPS).eFlags &= !EF_JETPACK_ACTIVE;
            }
        }

        speedIdle = (*pVeh)
            .m_pVehicleInfo
            .as_ref()
            .map(|vi| vi.speedIdle)
            .unwrap_or(0.0) as f32;
        speedIdleAccel = (*pVeh)
            .m_pVehicleInfo
            .as_ref()
            .map(|vi| vi.accelIdle)
            .unwrap_or(0.0)
            * (*pVeh).m_fTimeModifier;
        speedMin = (*pVeh)
            .m_pVehicleInfo
            .as_ref()
            .map(|vi| vi.speedMin)
            .unwrap_or(0.0) as f32;

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
/// Raven: the `_JK2MP` branch handles MP vehicle orientation (yaw control via view angles);
/// the `#else` SP branch (lines 553-594) is dead code and is dropped per porting-rules §10.
/// Source: `oracle/codemp/game/SpeederNPC.c:505-600`
pub fn ProcessOrientCommands(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let riderPS: *mut playerState_t;
        let parentPS: *mut playerState_t;
        let mut angDif: f32;

        // _JK2MP (MP) branch
        if !(*pVeh).m_pPilot.is_null() {
            riderPS = (*(*pVeh).m_pPilot).playerState;
        } else {
            riderPS = (*(*pVeh).m_pParentEntity).playerState;
        }
        parentPS = (*(*pVeh).m_pParentEntity).playerState;

        angDif = AngleSubtract(*(*pVeh).m_vOrientation.add(YAW), (*riderPS).viewangles[YAW]);

        if !parentPS.is_null() && (*parentPS).speed != 0.0f32 {
            let mut s: f32 = (*parentPS).speed;
            let maxDif: f32 = (*pVeh)
                .m_pVehicleInfo
                .as_ref()
                .map(|vi| vi.turningSpeed)
                .unwrap_or(0.0) as f32
                * 4.0f32;

            if s < 0.0f32 {
                s = -s;
            }

            angDif *= s
                / (*pVeh)
                    .m_pVehicleInfo
                    .as_ref()
                    .map(|vi| vi.speedMax)
                    .unwrap_or(1.0);

            if angDif > maxDif {
                angDif = maxDif;
            } else if angDif < -maxDif {
                angDif = -maxDif;
            }

            *(*pVeh).m_vOrientation.add(YAW) = AngleNormalize180(
                *(*pVeh).m_vOrientation.add(YAW) - angDif * ((*pVeh).m_fTimeModifier * 0.2f32),
            );

            // PORT-NOTE(vtable-access): pm->cmd.serverTime access requires bg-channel state;
            // electrify effect is guarded by pm access which needs threading
            // if parentPS->electrifyTime > pm->cmd.serverTime {
            //     pVeh->m_vOrientation[YAW] += (sin(pm->cmd.serverTime/1000.0f32)*3.0f32)*pVeh->m_fTimeModifier;
            // }
        }

        // SP (_JK2MP) code (lines 553-594) is dead code and dropped
    }
}

/// Raven `AnimateVehicle`.
///
/// Raven: "This function makes sure that the vehicle is properly animated."
/// The body is empty in the oracle (SpeederNPC.c:609) — a deliberate no-op.
/// Source: `oracle/codemp/game/SpeederNPC.c:608-610`
pub fn AnimateVehicle(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {}

/// Raven `AnimateRiders`.
///
/// Raven: "This function makes sure that the rider's in this vehicle are properly animated."
/// Raven: the `_JK2MP` build of this function only handles the boarding animation branch
/// (m_iBoarding != 0); the pilot-animation state machine (lines 744-1037) is guarded by
/// `#ifdef _JK2MP` with `if (1) return;` at line 741, making it dead code, dropped per
/// porting-rules §10.
/// Source: `oracle/codemp/game/SpeederNPC.c:630-1038`
pub fn AnimateRiders(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    unsafe {
        // Only handle boarding animation in MP build; pilot animation is dead code
        if (*pVeh).m_iBoarding == 0 {
            return;
        }

        // We've just started boarding, set the amount of time it will take to finish boarding
        if (*pVeh).m_iBoarding < 0 {
            let iAnimLen: c_int;
            let mut Anim: animNumber_t = BOTH_VS_IDLE;
            let iFlags: c_int = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
            let mut iBlend: c_int = 300;

            // Determine boarding animation based on direction
            if (*pVeh).m_iBoarding == -1 {
                Anim = BOTH_VS_MOUNT_L;
            } else if (*pVeh).m_iBoarding == -2 {
                Anim = BOTH_VS_MOUNT_R;
            } else if (*pVeh).m_iBoarding == -3 {
                Anim = BOTH_VS_MOUNTJUMP_L;
            } else if (*pVeh).m_iBoarding == VEH_MOUNT_THROW_LEFT {
                iBlend = 0;
                Anim = BOTH_VS_MOUNTTHROW_R;
            } else if (*pVeh).m_iBoarding == VEH_MOUNT_THROW_RIGHT {
                iBlend = 0;
                Anim = BOTH_VS_MOUNTTHROW_L;
            }

            // Set the delay time (40% of animation time). `ctx` now threads the bg
            // channel into this dispatch chain, so BG_AnimLength is reachable
            // (game-tier free-function form off `bg_state`).
            iAnimLen = (crate::bg_panimate::BG_AnimLength(
                &(*ctx.world_raw()).bg_state,
                (*(*pVeh).m_pPilot).localAnimIndex,
                Anim as c_int,
            ) as f32
                * 0.4f32) as c_int;
            // MP `BG_GetTime()` is `level.time`, reachable through `ctx`.
            (*pVeh).m_iBoarding = (*ctx.world_raw()).level.time + iAnimLen;

            // Set the animation which won't be interrupted until completed. `BG_SetAnim`
            // is a `PmoveContext` method (`bgAllAnims` off `BgState`); build a pm-null
            // per-call context from `ctx` (the `BG_ParseAnimationFile` game-tier
            // wrapper precedent; `BG_SetAnimFinal` null-guards the missing `pm`).
            let ps = (*(*pVeh).m_pPilot).playerState;
            let anims = (&(*ctx.world_raw()).bg_state.bgAllAnims)
                [(*(*pVeh).m_pPilot).localAnimIndex as usize]
                .anims;
            let traps = crate::bg_channel::GameBgTraps::new(ctx.engine);
            let mut callbacks = crate::bg_channel::GameCallbacksImpl {
                world: ctx.world_raw(),
                engine: ctx.engine,
            };
            let mut pmc = crate::bg_channel::PmoveContext::new(
                &mut (*ctx.world_raw()).bg_state,
                &traps,
                &mut callbacks,
            );
            pmc.BG_SetAnim(ps, anims, SETANIM_BOTH, Anim as c_int, iFlags, iBlend);
        }

        // Old pilot handling - SP only, dead code in MP
    }
}

// `G_SetSpeederVehicleFunctions` retired (2026-07-03) — it only assigned the now-removed
// `vehicleInfo_t` fn-ptr slots. Vehicle dispatch is `vehicleType_t`-keyed in
// `crate::veh_dispatch`. Source: see per-class setter in the oracle .c.

/// Raven `G_CreateSpeederNPC`.
///
/// Raven: "Create/Allocate a new Animal Vehicle (initializing it as well)."
/// The `_JK2MP` build uses `G_AllocateVehicleObject` on the game side (QAGAME branch);
/// the cgame branch would use `BG_Alloc` (dead code here, dropped).
/// Source: `oracle/codemp/game/SpeederNPC.c:1092-1113`
pub fn G_CreateSpeederNPC(
    ctx: &mut GameContext,
    pVeh: *mut *mut Vehicle_t,
    strType: *const c_char,
) {
    unsafe {
        // Allocate the Vehicle object
        // QAGAME branch (_JK2MP with QAGAME compile flag)
        crate::g_utils::G_AllocateVehicleObject(ctx, pVeh);

        // Zero-initialize the vehicle
        core::ptr::write_bytes(*pVeh, 0, 1);

        // Set the vehicle info pointer based on vehicle type name.
        let vehicleIndex: c_int = BG_VehicleGetIndex(
            strType,
            &mut (*ctx.world_raw()).bg_state,
            &crate::bg_channel::GameBgTraps::new(ctx.engine),
        );
        (*(*pVeh)).m_pVehicleInfo = &(&(*ctx.world_raw()).bg_state.g_vehicleInfo)
            [vehicleIndex as usize] as *const _
            as *mut vehicleInfo_t;
    }
}
