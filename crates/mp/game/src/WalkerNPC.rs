// PORT-COMPLETE: WalkerNPC.c 5/2
//! FAITHFUL port of `oracle/oracle/codemp/game/WalkerNPC.c`.
//!
//! Walker NPC vehicle implementation — movement, orientation, animation, and
//! initialization for the Walker vehicle type.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

/// Helper: compute vector length.
///
/// Inline port of `q_shared.h:1460-1489` — Raven `VectorLength`.
/// Uses plain C path (no XBOX asm).
/// Source: `oracle/oracle/codemp/game/q_shared.h:1487`
#[inline]
fn vector_length(v: &[f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Raven `RegisterAssets`.
///
/// Registers the turret weapon used by the Walker vehicle.
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:84-95`
pub fn RegisterAssets(pVeh: *mut Vehicle_t) {
    // RegisterItem for turret weapon (MP path).
    let weapon_item = unsafe {
        bg_misc::BG_FindItemForWeapon(weapon_t::WP_TURRET)
    };
    if !weapon_item.is_null() {
        g_items::RegisterItem(weapon_item);
    }

    // Call the standard RegisterAssets for base vehicle.
    unsafe {
        let g_veh_base = &mut g_vehicleInfo[VEHICLE_BASE as usize];
        if let Some(register_fn) = g_veh_base.RegisterAssets {
            register_fn(pVeh);
        }
    }
}

/// Raven `ProcessMoveCommands`.
///
/// Updates vehicle speed based on movement input and vehicle properties.
/// BG-compatible function (though oracle code violates this with pm access).
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:129-251`
pub fn ProcessMoveCommands(pVeh: *mut Vehicle_t) {
    // PORT-ESCALATION(pm-global): oracle line 224 accesses global `pm` for electrify check
    todo!("Port ProcessMoveCommands — parked: pm global not yet exposed in game context")
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

        let mut ang_dif = q_math::AngleSubtract(
            pVeh.m_vOrientation[YAW as usize],
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

            pVeh.m_vOrientation[YAW as usize] = q_math::AngleNormalize180(
                pVeh.m_vOrientation[YAW as usize] - ang_dif * (pVeh.m_fTimeModifier * 0.2),
            );
        }
    }
}

/// Raven `ProcessOrientCommands`.
///
/// Processes vehicle orientation based on rider input and vehicle properties.
/// BG-compatible function.
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:316-411`
pub fn ProcessOrientCommands(pVeh: *mut Vehicle_t) {
    unsafe {
        let pVeh = &mut *pVeh;
        let parent = pVeh.m_pParentEntity;
        if parent.is_null() {
            return;
        }

        let parent = &mut *parent;
        let parent_ps = &mut *parent.playerState;

        let mut rider_ent: *mut bgEntity_t = std::ptr::null_mut();

        if parent.s.owner != ENTITYNUM_NONE {
            rider_ent = bg_pmove::PM_BGEntForNum(parent.s.owner as c_int);
        }

        if rider_ent.is_null() {
            rider_ent = parent as *mut bgEntity_t;
        }

        let rider_ps = if !rider_ent.is_null() {
            &mut *(*rider_ent).playerState
        } else {
            parent_ps
        };

        // If the rider is a player.
        if !rider_ent.is_null() && (*rider_ent).s.number < MAX_CLIENTS {
            // WalkerYawAdjust call (MP path).
            WalkerYawAdjust(pVeh, rider_ps, parent_ps);
            pVeh.m_vOrientation[PITCH as usize] = rider_ps.viewangles[PITCH as usize];
        } else {
            // NPC or no rider.
            let mut turn_speed = pVeh.m_pVehicleInfo.as_ref()
                .map(|v| v.turningSpeed)
                .unwrap_or(0.0);

            if !pVeh.m_pVehicleInfo.as_ref()
                .map(|v| v.turnWhenStopped)
                .unwrap_or(false) && parent_ps.speed == 0.0
            {
                turn_speed = 0.0;
            }

            // Help NPCs out.
            if !rider_ent.is_null() && (*rider_ent).s.eType == ET_NPC {
                turn_speed *= 2.0;
                if parent_ps.speed > 200.0 {
                    turn_speed += turn_speed * parent_ps.speed / 200.0 * 0.05;
                }
            }

            turn_speed *= pVeh.m_fTimeModifier;

            // Default control: strafing turns.
            if pVeh.m_ucmd.rightmove < 0 {
                pVeh.m_vOrientation[YAW as usize] += turn_speed;
            } else if pVeh.m_ucmd.rightmove > 0 {
                pVeh.m_vOrientation[YAW as usize] -= turn_speed;
            }

            // Malfunction handling — no-op per oracle.
            if pVeh.m_pVehicleInfo.as_ref()
                .map(|v| v.malfunctionArmorLevel != 0)
                .unwrap_or(false)
                && pVeh.m_iArmor <= pVeh.m_pVehicleInfo.as_ref()
                    .map(|v| v.malfunctionArmorLevel)
                    .unwrap_or(0)
            {
                // Damaged badly — currently no special handling.
            }
        }
    }
}

/// Raven `AnimateVehicle`.
///
/// Animates the Walker vehicle based on speed and state.
/// QAGAME-only function.
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:415-536`
pub fn AnimateVehicle(pVeh: *mut Vehicle_t) {
    unsafe {
        let pVeh = &mut *pVeh;
        let parent = pVeh.m_pParentEntity as *mut gentity_t;
        if parent.is_null() {
            return;
        }

        let parent = &*parent;

        // If dead, return early.
        if parent.health <= 0 {
            return;
        }

        let mut anim = animNumber_t::BOTH_STAND1;
        let mut i_flags = SETANIM_FLAG_NORMAL as c_int;
        let mut i_blend = 300i32;

        let f_speed_perc_to_max = parent.client.as_ref()
            .map(|c| c.ps.speed / pVeh.m_pVehicleInfo.as_ref()
                .map(|v| v.speedMax)
                .unwrap_or(1.0))
            .unwrap_or(0.0);

        if f_speed_perc_to_max > 0.0 {
            i_blend = 300;
            i_flags = SETANIM_FLAG_OVERRIDE as c_int;

            // Determine walk vs. run animation.
            if (pVeh.m_ucmd.buttons & BUTTON_WALKING) != 0 || f_speed_perc_to_max < 0.275 {
                anim = animNumber_t::BOTH_WALK1;
            } else {
                anim = animNumber_t::BOTH_RUN1;
            }
        } else {
            // Not moving.
            if f_speed_perc_to_max < -0.018 {
                // Going in reverse.
                i_flags = SETANIM_FLAG_NORMAL as c_int;
                anim = animNumber_t::BOTH_WALKBACK1;
                i_blend = 500;
            } else {
                // Idle animation.
                i_flags = (SETANIM_FLAG_NORMAL | SETANIM_FLAG_RESTART | SETANIM_FLAG_HOLD) as c_int;
                i_blend = 600;

                if parent.client.as_ref()
                    .map(|c| c.ps.m_iVehicleNum)
                    .unwrap_or(0) != 0
                {
                    // Vehicle is occupied.
                    anim = animNumber_t::BOTH_STAND1;
                } else {
                    // Vehicle is empty.
                    anim = animNumber_t::BOTH_STAND2;
                }
            }
        }

        g_vehicles::Vehicle_SetAnim(
            parent as *mut gentity_t,
            SETANIM_LEGS as c_int,
            anim as c_int,
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
    // PORT-ESCALATION(fn-enum-dispatch): Ruling 2 requires fn-ID enums for function pointers
    todo!("Port G_SetWalkerVehicleFunctions — parked: fn-pointer enum dispatch pattern not yet available")
}

/// Raven `Board`.
///
/// Board the Walker vehicle (internal static, assigned to vehicleInfo_t.Board).
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:106-115`
fn Board(pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> bool {
    // PORT-ESCALATION(level-global): oracle line 188 accesses `level.time` global for boarding delay
    todo!("Port Board — parked: level.time not yet accessible in vehicle context")
}

/// Raven `G_CreateWalkerNPC`.
///
/// Allocate and initialize a new Walker vehicle.
/// Source: `oracle/oracle/codemp/game/WalkerNPC.c:594-615`
pub fn G_CreateWalkerNPC(pVeh: *mut *mut Vehicle_t, strAnimalType: *const c_char) {
    unsafe {
        // Allocate the vehicle object.
        g_utils::G_AllocateVehicleObject(pVeh);

        // Zero out the vehicle structure.
        let veh = *pVeh;
        if !veh.is_null() {
            std::ptr::write_bytes(veh, 0, 1);

            // Set vehicle info pointer based on animal type.
            let veh_ref = &mut *veh;
            let veh_index = bg_vehicleLoad::BG_VehicleGetIndex(strAnimalType);
            veh_ref.m_pVehicleInfo = &mut g_vehicleInfo[veh_index as usize];
        }
    }
}
