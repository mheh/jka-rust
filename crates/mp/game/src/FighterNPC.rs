//! Game-only half of `oracle/codemp/game/FighterNPC.c`.
//!
//! The shared (game + cgame) steering and physics functions moved to `mp_bg::vehicles::fighter_npc`, a cgame TU in `JK2_cgame.vcproj`.
//! These are `BG_FighterUpdate`, `ProcessMoveCommands`, `ProcessOrientCommands`, and their helpers.
//! The move lets the cgame vehicle `Pmove` steer fighters during prediction.
//! What stays here is the `#ifdef QAGAME`-only surface:
//! `Board`, `Eject`, `Update`, `AnimateVehicle`, `AnimateRiders`, `FighterPitchClamp`, and `G_CreateFighterNPC`.
//! `FighterIsInSpace` also stays here, as the callback target reached from the moved bg code under the Game host.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

use crate::bg_channel::{GameBgTraps, GameCallbacksImpl, PmoveContext};
use crate::g_utils::G_AllocateVehicleObject;
use crate::g_vehicles::Update as vehicle_base_update;
use crate::veh_dispatch;
use mp_bg::bg_channel::BgHost;
use mp_bg::bg_pmove::BG_UnrestrainedPitchRoll;
use mp_bg::bg_vehicleLoad::BG_VehicleGetIndex;
use mp_bg::vehicles::fighter_npc::{BG_FighterUpdate, FighterIsLanded, FighterIsLanding};

// Constants used by the game-only fighter bodies.
// Values from the oracle.
// Source: `oracle/codemp/game/{bg_public.h,bg_vehicles.h,FighterNPC.c}`.
const HYPERSPACE_TIME: c_int = 4000; // bg_public.h:1679
const MIN_LANDING_SLOPE: f32 = 0.8; // bg_vehicles.h:400
const CHAN_AUTO: c_int = 0; // soundChannel_t CHAN_AUTO
                            // `vehFlags_t` masks as `u64` for `Vehicle_t::m_ulFlags`.
                            // Source: `bg_vehicles.h:417`.
const VEH_WINGSOPEN: u64 = 0x0000_0020;
const VEH_GEARSOPEN: u64 = 0x0000_0040;

/// Raven `Board`.
///
/// Source: `oracle/codemp/game/FighterNPC.c:212-221`
pub fn Board(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> qboolean {
    unsafe {
        // `g_vehicleInfo[VEHICLE_BASE].Board` is the generic base body.
        if crate::g_vehicles::Board(ctx, pVeh, pEnt) == qfalse {
            return qfalse;
        }

        // Set the board wait time (they won't be able to do anything, including getting off, for this amount of time).
        (*pVeh).m_iBoarding = ctx.world.level.time + 1500;

        qtrue
    }
}

/// Raven `Eject`.
///
/// Source: `oracle/codemp/game/FighterNPC.c:224-232`
pub fn Eject(
    ctx: &mut GameContext,
    pVeh: *mut Vehicle_t,
    pEnt: *mut bgEntity_t,
    forceEject: qboolean,
) -> qboolean {
    // `g_vehicleInfo[VEHICLE_BASE].Eject` is the generic base body.
    if crate::g_vehicles::Eject(ctx, pVeh, pEnt, forceEject) != qfalse {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `Update`, the fighter's per-frame update slot on the QAGAME game side.
///
/// The `#ifdef QAGAME` `Ghost` loop hoisted out of `BG_FighterUpdate` runs here first.
/// It reaches the game-only vehicle and passenger entities, and it only ran game-side.
/// This matches the oracle's call order, where `Update` calls `BG_FighterUpdate` and its top runs the `Ghost` loop.
/// The moved bg `BG_FighterUpdate` runs next, with fighter gravity and the landing trace, built with a `pm`-null `PmoveContext`.
/// The generic base `Update` runs last.
/// `trMins` and `trMaxs` are the parent gentity's `r.mins` and `r.maxs`.
/// Gravity is the `g_gravity` cvar.
/// Source: `oracle/codemp/game/FighterNPC.c:105-114,188-209`
pub fn Update(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pUcmd: *const usercmd_t) -> qboolean {
    unsafe {
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        debug_assert!(!parent.is_null());
        let parent_id = ctx.entity_id_of(parent).unwrap();

        // QAGAME: make the riders non-visible and non-collidable (`Ghost`).
        // Source: oracle/codemp/game/FighterNPC.c:105-114
        veh_dispatch::ghost(ctx, pVeh, (*pVeh).m_pPilot.cast::<gentity_t>());
        {
            let maxPassengers = (*pVeh)
                .m_pVehicleInfo
                .as_ref()
                .map(|vi| vi.maxPassengers)
                .unwrap_or(0);
            let mut i: c_int = 0;
            while i < maxPassengers {
                veh_dispatch::ghost(
                    ctx,
                    pVeh,
                    (*pVeh).m_ppPassengers[i as usize].cast::<gentity_t>(),
                );
                i += 1;
            }
        }

        let gravity = ctx.world.cvars.g_gravity.value;
        let (trMins, trMaxs) = {
            let e = ctx.world.entity(parent_id);
            (e.r.mins, e.r.maxs)
        };

        // `BG_FighterUpdate` now lives in bg.
        // Build a `pm`-null `PmoveContext`.
        let traps = GameBgTraps::new(ctx.engine);
        let mut callbacks = GameCallbacksImpl {
            // SEAM-BG-REENTRY (DEC-28, sanctioned).
            // GameCallbacksImpl.world is a `*mut GameWorld` field aliasing bg_state.
            // A raw store is required for bg-seam re-entry.
            world: ctx.world_raw(),
            engine: ctx.engine,
        };
        let bail = {
            let mut pmc = PmoveContext::new(&mut ctx.world.bg_state, &traps, &mut callbacks);
            BG_FighterUpdate(&mut pmc, pVeh, pUcmd, trMins, trMaxs, gravity) == qfalse
        };
        if bail {
            return qfalse;
        }

        // `g_vehicleInfo[VEHICLE_BASE].Update` — the generic base body.
        if vehicle_base_update(ctx, pVeh, pUcmd) == qfalse {
            return qfalse;
        }

        qtrue
    }
}

/// Raven `FighterIsInSpace`.
///
/// The whole function is `#ifdef QAGAME`.
/// It stays game-side as the target of the bg `fighter_is_in_space` upcall, because it reads `client->inSpaceIndex`.
/// Source: `oracle/codemp/game/FighterNPC.c:276-286`
pub fn FighterIsInSpace(gParent: &gentity_t) -> qboolean {
    unsafe {
        let gParent: *const gentity_t = gParent;
        if !gParent.is_null() {
            let ent = &*gParent;
            if !ent.client.is_null() {
                let client = &*(ent.client);
                if client.inSpaceIndex != 0 && client.inSpaceIndex < 2047 {
                    // ENTITYNUM_WORLD
                    return qtrue;
                }
            }
        }
        qfalse
    }
}

/// Raven `FighterPitchClamp`.
///
/// Source: `oracle/codemp/game/FighterNPC.c:1352-1370`
pub fn FighterPitchClamp(
    ctx: &mut GameContext,
    pVeh: *mut Vehicle_t,
    riderPS: *mut playerState_t,
    parentPS: *mut playerState_t,
    curTime: c_int,
) {
    unsafe {
        if BG_UnrestrainedPitchRoll(riderPS, pVeh, &ctx.world.bg_state) == qfalse {
            //cap pitch reasonably
            if let Some(vi) = (*pVeh).m_pVehicleInfo.as_ref() {
                if vi.pitchLimit != -1.0
                    && (*pVeh).m_iRemovedSurfaces == 0
                    && (*parentPS).electrifyTime < curTime
                {
                    if *(*pVeh).m_vOrientation.add(0) > vi.pitchLimit {
                        *(*pVeh).m_vOrientation.add(0) = vi.pitchLimit;
                    } else if *(*pVeh).m_vOrientation.add(0) < -vi.pitchLimit {
                        *(*pVeh).m_vOrientation.add(0) = -vi.pitchLimit;
                    }
                }
            }
        }
    }
}

/// Raven `AnimateVehicle`.
///
/// It syncs the fighter's wing, gear, and hyperspace anim state to its flight state.
/// This is the MP (`_JK2MP` and `QAGAME`) build, and `curTime` is `level.time`.
/// The landing-state predicates are the moved bg helpers, called with `BgHost::Game`.
/// Source: `oracle/codemp/game/FighterNPC.c:1836-1937`
pub fn AnimateVehicle(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let mut Anim: c_int = -1;
        let parent_id = ctx
            .entity_id_of((*pVeh).m_pParentEntity as *const gentity_t)
            .unwrap();
        let parentPS: *mut playerState_t = ctx.world.entity(parent_id).playerState;
        let curTime: c_int = ctx.world.level.time;
        let vi = (*pVeh).m_pVehicleInfo;

        if (*parentPS).hyperSpaceTime != 0 && curTime - (*parentPS).hyperSpaceTime < HYPERSPACE_TIME
        {
            //Going to Hyperspace
            //close the wings (FIXME: makes sense on X-Wing, not Shuttle?)
            if (*pVeh).m_ulFlags & VEH_WINGSOPEN != 0 {
                (*pVeh).m_ulFlags &= !VEH_WINGSOPEN;
                Anim = BOTH_WINGS_CLOSE as c_int;
            }
        } else {
            let isLanding = FighterIsLanding(BgHost::Game, pVeh, parentPS);
            let isLanded = FighterIsLanded(pVeh, parentPS);

            // if we're above launch height (way up in the air)...
            if isLanding == qfalse && isLanded == qfalse {
                if (*pVeh).m_ulFlags & VEH_WINGSOPEN == 0 {
                    (*pVeh).m_ulFlags |= VEH_WINGSOPEN;
                    (*pVeh).m_ulFlags &= !VEH_GEARSOPEN;
                    Anim = BOTH_WINGS_OPEN as c_int;
                }
            } else if ((*pVeh).m_ucmd.forwardmove < 0
                || (*pVeh).m_ucmd.upmove < 0
                || isLanded != qfalse)
                && (*pVeh).m_LandTrace.fraction <= 0.4
                && (*pVeh).m_LandTrace.plane.normal[2] >= MIN_LANDING_SLOPE
            {
                //already landed or trying to land and close to ground
                // Open gears.
                if (*pVeh).m_ulFlags & VEH_GEARSOPEN == 0 {
                    if (*vi).soundLand != 0 {
                        //just landed?
                        crate::g_utils::G_EntitySound(ctx, parent_id, CHAN_AUTO, (*vi).soundLand);
                    }
                    (*pVeh).m_ulFlags |= VEH_GEARSOPEN;
                    Anim = BOTH_GEARS_OPEN as c_int;
                }
            } else if (*pVeh).m_ulFlags & VEH_GEARSOPEN != 0 {
                //trying to take off and almost halfway off the ground
                // Close gears (if they're open).
                (*pVeh).m_ulFlags &= !VEH_GEARSOPEN;
                Anim = BOTH_GEARS_CLOSE as c_int;
            } else if (*pVeh).m_ulFlags & VEH_WINGSOPEN != 0 {
                // If gears are closed, and we are below launch height, close the wings.
                (*pVeh).m_ulFlags &= !VEH_WINGSOPEN;
                Anim = BOTH_WINGS_CLOSE as c_int;
            }
        }

        if Anim != -1 {
            // `BG_SetAnim` is a `PmoveContext` method.
            // Build a pm-null per-call context from `ctx`.
            let idx = ctx.world.entity(parent_id).localAnimIndex as usize;
            let anims = ctx.world.bg_state.bgAllAnims[idx].anims;
            let traps = GameBgTraps::new(ctx.engine);
            let mut callbacks = GameCallbacksImpl {
                // SEAM-BG-REENTRY (DEC-28, sanctioned).
                // GameCallbacksImpl.world is a `*mut GameWorld` field aliasing bg_state.
                // A raw store is required for bg-seam re-entry.
                world: ctx.world_raw(),
                engine: ctx.engine,
            };
            let mut pmc = PmoveContext::new(&mut ctx.world.bg_state, &traps, &mut callbacks);
            pmc.BG_SetAnim(
                parentPS,
                anims,
                SETANIM_BOTH,
                Anim,
                SETANIM_FLAG_NORMAL,
                300,
            );
        }
    }
}

/// Raven `AnimateRiders`.
///
/// The `_JK2MP`/`QAGAME` body is empty, matching `SpeederNPC`'s.
/// Source: `oracle/codemp/game/FighterNPC.c:1938-1944`
pub fn AnimateRiders(_ctx: &mut GameContext, _pVeh: *mut Vehicle_t) {}

// `G_SetFighterVehicleFunctions` is retired.
// It only assigned the now-removed `vehicleInfo_t` fn-ptr slots.
// Vehicle dispatch is `vehicleType_t`-keyed in `crate::veh_dispatch`.
// Source: see the per-class setter in the oracle .c file.

/// Raven `G_CreateFighterNPC`.
///
/// Source: `oracle/codemp/game/FighterNPC.c:1994-2014`
pub fn G_CreateFighterNPC(
    ctx: &mut GameContext,
    pVeh: *mut *mut Vehicle_t,
    strType: *const c_char,
) {
    unsafe {
        // Allocate the Vehicle.
        G_AllocateVehicleObject(ctx, pVeh);

        // Zero out the Vehicle structure.
        std::ptr::write_bytes(*pVeh, 0, 1);

        // Set the vehicle info pointer based on vehicle type name.
        let mut callbacks = GameCallbacksImpl {
            // SEAM-BG-REENTRY (DEC-28, sanctioned).
            // GameCallbacksImpl.world is a `*mut GameWorld` field aliasing bg_state.
            // A raw store is required for bg-seam re-entry.
            world: ctx.world_raw(),
            engine: ctx.engine,
        };
        let veh_index = BG_VehicleGetIndex(
            strType,
            &mut ctx.world.bg_state,
            &GameBgTraps::new(ctx.engine),
            &mut callbacks,
        );
        (*(*pVeh)).m_pVehicleInfo = &(&ctx.world.bg_state.g_vehicleInfo)[veh_index as usize]
            as *const _ as *mut vehicleInfo_t;
    }
}
