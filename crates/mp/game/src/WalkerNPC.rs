// PORT-COMPLETE: WalkerNPC.c
//! Game-only half of `oracle/codemp/game/WalkerNPC.c`.
//!
//! The shared (game + cgame) `ProcessMoveCommands`/`ProcessOrientCommands` and
//! the `WalkerYawAdjust` helper moved to `mp_bg::vehicles::walker_npc` (a cgame
//! TU in `JK2_cgame.vcproj`) so the cgame vehicle `Pmove` can steer walkers
//! during prediction. What stays here is `#ifdef QAGAME`-only: `RegisterAssets`,
//! `AnimateVehicle`, `Board`, `G_CreateWalkerNPC`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::q_math::YAW;

/// Raven `RegisterAssets`.
///
/// Registers the turret weapon used by the Walker vehicle.
/// Source: `oracle/codemp/game/WalkerNPC.c:84-95`
pub fn RegisterAssets(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    unsafe {
        // atst uses turret weapon (#ifdef _JK2MP path — both MP/SP port to same)
        let weapon = mp_bg::bg_misc::BG_FindItemForWeapon(WP_TURRET);
        crate::g_items::RegisterItem(ctx, weapon);

        // The generic base RegisterAssets body (empty).
        crate::g_vehicles::RegisterAssets(pVeh);
    }
}

/// Raven `AnimateVehicle`.
///
/// Animates the Walker vehicle based on speed and state.
/// Source: `oracle/codemp/game/WalkerNPC.c:415-536`
pub fn AnimateVehicle(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let pVeh = &mut *pVeh;
        let mut anim = BOTH_STAND1;
        let mut i_flags = SETANIM_FLAG_NORMAL;
        let mut i_blend = 300;

        let parent_bg = pVeh.m_pParentEntity;
        if parent_bg.is_null() {
            return;
        }
        // Overlay cast: `bgEntity_t` is only the shared head of
        // `gentity_t`; `health`/`client` live past that head on the real object.
        let parent = parent_bg as *mut gentity_t;

        // We're dead (boarding is reused here so I don't have to make another variable)
        if (*parent).health <= 0 {
            return;
        }

        // Percentage of maximum speed relative to current speed
        let speed_max = pVeh
            .m_pVehicleInfo
            .as_ref()
            .map(|v| v.speedMax)
            .unwrap_or(100.0);
        let f_speed_perc_to_max = if !(*parent).client.is_null() {
            (*((*parent).client)).ps.speed / speed_max
        } else {
            0.0
        };

        // If we're moving...
        if f_speed_perc_to_max > 0.0 {
            i_blend = 300;
            i_flags = SETANIM_FLAG_OVERRIDE;

            let f_yaw_delta =
                pVeh.m_vPrevOrientation[YAW as usize] - *pVeh.m_vOrientation.add(YAW as usize);

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
                if !(*parent).client.is_null() && (*((*parent).client)).ps.m_iVehicleNum != 0 {
                    anim = BOTH_STAND1;
                } else {
                    anim = BOTH_STAND2;
                }
            }
        }

        // Call Vehicle_SetAnim
        crate::g_vehicles::Vehicle_SetAnim(
            ctx,
            ctx.entity_id_of(parent as *mut gentity_t).unwrap(),
            SETANIM_LEGS,
            anim as c_int,
            i_flags,
            i_blend,
        );
    }
}

// `G_SetWalkerVehicleFunctions` retired — it only assigned the now-removed
// `vehicleInfo_t` fn-ptr slots. Vehicle dispatch is `vehicleType_t`-keyed in
// `crate::veh_dispatch`. Source: see per-class setter in the oracle .c.

/// Raven `Board`.
///
/// Board the Walker vehicle (reached via `crate::veh_dispatch::board`).
/// Source: `oracle/codemp/game/WalkerNPC.c:106-115`
pub fn Board(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> bool {
    unsafe {
        // `g_vehicleInfo[VEHICLE_BASE].Board` is the generic base body.
        if crate::g_vehicles::Board(ctx, pVeh, pEnt) == qfalse {
            return false;
        }

        // Set the board wait time (they won't be able to do anything, including getting off, for this amount of time).
        (*pVeh).m_iBoarding = ctx.world.level.time + 1500;

        true
    }
}

/// Raven `G_CreateWalkerNPC`.
///
/// Allocate and initialize a new Walker vehicle.
/// Source: `oracle/codemp/game/WalkerNPC.c:594-615`
pub fn G_CreateWalkerNPC(
    ctx: &mut GameContext,
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
            );
            (**pVeh).m_pVehicleInfo =
                &mut (&mut ctx.world.bg_state.g_vehicleInfo)[veh_index as usize];
        }
    }
}
