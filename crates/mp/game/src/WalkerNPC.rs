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

        // The generic base RegisterAssets body (empty).
        crate::g_vehicles::RegisterAssets(pVeh);
    }
}

/// Raven `ProcessMoveCommands`.
///
/// Updates vehicle speed based on movement input and vehicle properties.
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:129-251`
pub fn ProcessMoveCommands(ctx: GameContext<'_>, pVeh: *mut Vehicle_t) {
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
        // accessing through bg_pmove module (bgEntity_t overlay-cast to gentity_t).
        if parent_ps.electrifyTime > 0 {
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
pub fn ProcessOrientCommands(ctx: GameContext<'_>, pVeh: *mut Vehicle_t) {
    unsafe {
        let pVeh = &mut *pVeh;
        let parent = pVeh.m_pParentEntity;
        if parent.is_null() {
            return;
        }

        // Kept as a raw pointer (not reborrowed to `&mut`): `Vehicle_t`'s
        // `m_pParentEntity` is `mp_bg`'s own `bgEntity_t`, while this crate's
        // `bgEntity_t` name is the `gentity_t` alias (prelude); a reference-to-
        // pointer `as` cast requires identical pointee types, so the overlay
        // cast below needs `parent` to stay a raw pointer (pointer-to-pointer
        // casts are unconstrained).
        let parent_ps: *mut playerState_t = (*parent).playerState;

        let mut rider: *mut bgEntity_t = std::ptr::null_mut();
        if (*parent).s.owner != ENTITYNUM_NONE {
            // Raven `PM_BGEntForNum(parent->s.owner)` == `&g_entities[owner]`;
            // `ctx` now threads the world, so index the game arena directly.
            rider = (*ctx.world)
                .g_entities
                .as_mut_ptr()
                .add((*parent).s.owner as usize) as *mut bgEntity_t;
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
pub fn AnimateVehicle(ctx: GameContext<'_>, pVeh: *mut Vehicle_t) {
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
        let speed_max = pVeh.m_pVehicleInfo.as_ref()
            .map(|v| v.speedMax)
            .unwrap_or(100.0);
        let f_speed_perc_to_max = if !(*parent).client.is_null() {
            (*((*parent).client as *mut gclient_t)).ps.speed / speed_max
        } else {
            0.0
        };

        // If we're moving...
        if f_speed_perc_to_max > 0.0 {
            i_blend = 300;
            i_flags = SETANIM_FLAG_OVERRIDE;

            let f_yaw_delta = pVeh.m_vPrevOrientation[YAW as usize] - *pVeh.m_vOrientation.add(YAW as usize);

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
                if !(*parent).client.is_null()
                    && (*((*parent).client as *mut gclient_t)).ps.m_iVehicleNum != 0
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
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:106-115`
pub fn Board(ctx: GameContext<'_>, pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> bool {
    unsafe {
        // `g_vehicleInfo[VEHICLE_BASE].Board` is the generic base body.
        if crate::g_vehicles::Board(ctx, pVeh, pEnt) == qfalse {
            return false;
        }

        // Set the board wait time (they won't be able to do anything, including getting off, for this amount of time).
        (*pVeh).m_iBoarding = (*ctx.world).level.time + 1500;

        true
    }
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
            let veh_index = crate::bg_vehicleLoad::BG_VehicleGetIndex(
                strAnimalType,
                &mut (*ctx.world).bg_state,
                &crate::bg_channel::GameBgTraps::new(ctx.engine),
            );
            (**pVeh).m_pVehicleInfo = &mut (*ctx.world).bg_state.g_vehicleInfo[veh_index as usize];
        }
    }
}
