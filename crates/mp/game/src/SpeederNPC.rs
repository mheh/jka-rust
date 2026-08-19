//! Game-only half of `oracle/codemp/game/SpeederNPC.c`.
//!
//! The shared (game + cgame) `ProcessMoveCommands`/`ProcessOrientCommands` and the `VEH_StartStrafeRam` stub moved to `mp_bg::vehicles::speeder_npc`
//! (a cgame TU in `JK2_cgame.vcproj`), so the cgame vehicle `Pmove` can steer speeders during prediction.
//! What stays here is `#ifdef QAGAME`-only: `Update`, `AnimateVehicle`, `AnimateRiders`, `G_CreateSpeederNPC`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_vehicles::{VEH_MOUNT_THROW_LEFT, VEH_MOUNT_THROW_RIGHT};
use crate::prelude::*;

/// Raven `Update`.
///
/// Raven: the `_JK2MP` build of this function is a thin wrapper around the base vehicle's Update method.
/// The movement-direction, strafe-ram, exhaust, and armor-effects code (lines 163-264) is guarded by `#ifndef _JK2MP`.
/// That code is SP-only dead code, dropped per porting-rules §10.
/// Source: `oracle/codemp/game/SpeederNPC.c:149-268`
pub fn Update(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pUcmd: *const usercmd_t) -> qboolean {
    unsafe {
        // `g_vehicleInfo[VEHICLE_BASE].Update` — the generic base body.
        if crate::g_vehicles::Update(ctx, pVeh, pUcmd) == qfalse {
            return qfalse;
        }

        // See whether this vehicle should be exploding.
        if (*pVeh).m_iDieTime != 0 {
            // `pVeh->m_pVehicleInfo->DeathUpdate(pVeh)` resolves to the base DeathUpdate.
            // The Speeder DeathUpdate slot is commented out in the oracle setup.
            crate::veh_dispatch::death_update(ctx, pVeh);
        }

        qtrue
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
/// Raven: the `_JK2MP` build of this function only handles the boarding animation branch (`m_iBoarding != 0`).
/// The pilot-animation state machine (lines 744-1037) is guarded by `#ifdef _JK2MP` with `if (1) return;` at line 741.
/// It is dead code there, dropped per porting-rules §10.
/// Source: `oracle/codemp/game/SpeederNPC.c:630-1038`
pub fn AnimateRiders(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    unsafe {
        // Only the boarding branch runs in the MP build.
        if (*pVeh).m_iBoarding == 0 {
            return;
        }

        // We've just started moarding, set the amount of time it will take to finish moarding.
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

            // Set the delay time (which happens to be the time it takes for the animation to complete).
            // NOTE: Here I made it so the delay is actually 40% (0.4f) of the animation time.
            iAnimLen = (mp_bg::bg_panimate::BG_AnimLength(
                &ctx.world.bg_state,
                (*(*pVeh).m_pPilot).localAnimIndex,
                Anim as c_int,
            ) as f32
                * 0.4f32) as c_int;
            // MP `BG_GetTime()` is `level.time`, reachable through `ctx`.
            (*pVeh).m_iBoarding = ctx.world.level.time + iAnimLen;

            // Set the animation, which won't be interrupted until it's completed.
            let ps = (*(*pVeh).m_pPilot).playerState;
            let anims =
                (&ctx.world.bg_state.bgAllAnims)[(*(*pVeh).m_pPilot).localAnimIndex as usize].anims;
            let traps = crate::bg_channel::GameBgTraps::new(ctx.engine);
            let mut callbacks = crate::bg_channel::GameCallbacksImpl {
                // SEAM-BG-REENTRY (DEC-28, sanctioned).
                // GameCallbacksImpl.world is a `*mut GameWorld` field aliasing bg_state.
                // A raw store is required for bg-seam re-entry.
                world: ctx.world_raw(),
                engine: ctx.engine,
            };
            let mut pmc = crate::bg_channel::PmoveContext::new(
                &mut ctx.world.bg_state,
                &traps,
                &mut callbacks,
            );
            pmc.BG_SetAnim(ps, anims, SETANIM_BOTH, Anim as c_int, iFlags, iBlend);
        }

        // Old pilot handling - SP only, dead code in MP
    }
}

// `G_SetSpeederVehicleFunctions` is gone. It only assigned the removed `vehicleInfo_t` function-pointer slots.
// Vehicle dispatch is `vehicleType_t`-keyed in `crate::veh_dispatch`.

/// Raven `G_CreateSpeederNPC`.
///
/// Raven: "Create/Allocate a new Animal Vehicle (initializing it as well)."
/// The `_JK2MP` build uses `G_AllocateVehicleObject` on the game side, the QAGAME branch.
/// The cgame branch would use `BG_Alloc`, dropped here as dead code.
/// Source: `oracle/codemp/game/SpeederNPC.c:1092-1113`
pub fn G_CreateSpeederNPC(
    ctx: &mut GameContext,
    pVeh: *mut *mut Vehicle_t,
    strType: *const c_char,
) {
    unsafe {
        crate::g_utils::G_AllocateVehicleObject(ctx, pVeh);

        // Zero-initialize the vehicle
        core::ptr::write_bytes(*pVeh, 0, 1);

        // Set the vehicle info pointer based on vehicle type name.
        let mut callbacks = crate::bg_channel::GameCallbacksImpl {
            // SEAM-BG-REENTRY (DEC-28, sanctioned).
            // GameCallbacksImpl.world is a `*mut GameWorld` field aliasing bg_state.
            // A raw store is required for bg-seam re-entry.
            world: ctx.world_raw(),
            engine: ctx.engine,
        };
        let vehicleIndex: c_int = BG_VehicleGetIndex(
            strType,
            &mut ctx.world.bg_state,
            &crate::bg_channel::GameBgTraps::new(ctx.engine),
            &mut callbacks,
        );
        (*(*pVeh)).m_pVehicleInfo = &(&ctx.world.bg_state.g_vehicleInfo)[vehicleIndex as usize]
            as *const _ as *mut vehicleInfo_t;
    }
}
