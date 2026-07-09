// PORT-STATUS: g_vehicles.c — pass-3 blind fill: all 14 remaining fns bodied
// against the resolved (LAW) signatures. Boundary-set fns (Vehicle_SetAnim,
// Update, G_FlyVehicleImpactDir, G_SetVehDamageFlags, G_FlyVehicleDestroySurface)
// carry no ctx/bg channel in their fixed vtable/fn-ptr slot signatures yet reach
// world/engine/rng — those references are transcribed against the game channel
// (`ctx`) and flagged with PORT-NOTEs pending the vtable-dispatch retrofit.
//! FAITHFUL port of `oracle/codemp/game/g_vehicles.c` (MP `_JK2MP` +
//! `QAGAME` compile path).
//!
//! Generated from the `fnskel.py` signature skeleton; bodies transcribed per the
//! settled jampgame fork rulings. STAGING ONLY — not yet wired into crates/.
//!
//! Parking pattern in this file (see the workflow's recurring escalations):
//! - `raw-ptr-skeleton-no-world-handle`: reads `level.time`/`g_entities`/cvar
//!   globals or calls engine traps, none reachable from the raw-pointer skeleton
//!   signature (`level`/`g_entities`/cvars live on the world).
//! - `vec3-outparam-seam`: relies on `AngleVectors`/`VectorNormalize` out-params,
//!   whose resolved signatures take `vec3_t` ([f32;3]) by value and so cannot
//!   write back — the signature can't be re-declared here.
//! - `packet-contract`: passes a C `NULL` where the resolved callee (`G_Damage`)
//!   takes a `vec3_t` by value, which cannot express a null argument.
//! - `bg-anim-globals`: indexes the runtime-populated `bgAllAnims` global table,
//!   which has no handle in scope.
//! - `vehicle-vtable`: the `vehicleInfo_t` vtable fields are
//!   `Option<unsafe extern "C" fn>` but the ported member fns are plain-Rust —
//!   assigning them needs an unsettled extern-"C" seam (vtable dispatch).
//!
//! The `Ghost`/`UnGhost`/`SHIPSURF_*`/`SVF_*`/`EF_*`/`CONTENTS_*` constants are
//! spelled with their Raven names as bare identifiers (staging convention: the
//! integrator wires the const, the name preserves intent).
#![allow(non_snake_case, unused, clippy::all)]

use crate::bg_channel::GameBgTraps;
use crate::g_utils::G_SoundIndex;
use crate::prelude::*;
use crate::q_math::{
    _DotProduct, _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, vectoangles,
    AngleSubtract, AngleVectors, VectorNormalize,
};
use crate::q_shared::Q_strncmp;
use crate::trap;
use crate::NPC_spawn::NPC_Spawn_Do;
use mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs;
use mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs;
use mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs;
use mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::vehicles::vehicleType_t;

// Raven vehicle constants spelled locally per this file's staging convention
// (the integrator wires the const home later; the name preserves intent).
// Boarding sentinels stored in `m_iBoarding`.
// Source: `oracle/codemp/game/bg_vehicles.h:402-403`
pub const VEH_MOUNT_THROW_LEFT: c_int = -5;
pub const VEH_MOUNT_THROW_RIGHT: c_int = -6;
// Eject-direction anon enum.
// Source: `oracle/codemp/game/bg_vehicles.h:407-414`
pub const VEH_EJECT_LEFT: c_int = 0;
pub const VEH_EJECT_RIGHT: c_int = 1;
pub const VEH_EJECT_FRONT: c_int = 2;
pub const VEH_EJECT_REAR: c_int = 3;
pub const VEH_EJECT_TOP: c_int = 4;
pub const VEH_EJECT_BOTTOM: c_int = 5;
// Default player bbox z-extents (used for the MP eject-clearance trace).
// Canonical in `mp_bg::public::viewheight` (`c_int`, cast here to match the
// `vec3_t` components they seed).
// Source: `oracle/codemp/game/bg_public.h:41-42`
const DEFAULT_MINS_2: f32 = mp_bg::public::viewheight::DEFAULT_MINS_2 as f32;
const DEFAULT_MAXS_2: f32 = mp_bg::public::viewheight::DEFAULT_MAXS_2 as f32;

// `SVF_NOCLIENT` resolves via the crate prelude glob (`crate::g_public_consts`);
// the shadowing local copy was removed by the placeholder-const sweep.

/// Raven vehicle-surface indices (`bg_vehicles.h:427-430`).
pub const SHIPSURF_FRONT: c_int = 0;
pub const SHIPSURF_BACK: c_int = 1;
pub const SHIPSURF_RIGHT: c_int = 2;
pub const SHIPSURF_LEFT: c_int = 3;

/// Raven vehicle-surface damage-level indices (`bg_vehicles.h:432-439`).
pub const SHIPSURF_DAMAGE_FRONT_LIGHT: c_int = 0;
pub const SHIPSURF_DAMAGE_BACK_LIGHT: c_int = 1;
pub const SHIPSURF_DAMAGE_RIGHT_LIGHT: c_int = 2;
pub const SHIPSURF_DAMAGE_LEFT_LIGHT: c_int = 3;
pub const SHIPSURF_DAMAGE_FRONT_HEAVY: c_int = 4;
pub const SHIPSURF_DAMAGE_BACK_HEAVY: c_int = 5;
pub const SHIPSURF_DAMAGE_RIGHT_HEAVY: c_int = 6;
pub const SHIPSURF_DAMAGE_LEFT_HEAVY: c_int = 7;

/// Raven vehicle-surface "broken" bitflags (`bg_vehicles.h:442-448`).
pub const SHIPSURF_BROKEN_A: c_int = 1 << 0; // gear 1
pub const SHIPSURF_BROKEN_B: c_int = 1 << 1; // gear 1
pub const SHIPSURF_BROKEN_C: c_int = 1 << 2; // wing 1
pub const SHIPSURF_BROKEN_D: c_int = 1 << 3; // wing 2
pub const SHIPSURF_BROKEN_E: c_int = 1 << 4; // wing 3
pub const SHIPSURF_BROKEN_F: c_int = 1 << 5; // wing 4
pub const SHIPSURF_BROKEN_G: c_int = 1 << 6; // front

/// Raven `TURN_OFF` — `NPC_SetSurfaceOnOff` flag; this TU's local `#define`.
/// Source: `oracle/codemp/game/g_vehicles.c:2928`
const TURN_OFF: c_int = 0x0000_0100;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`



// `PITCH`/`YAW`/`ROLL` resolve via the crate prelude glob (`crate::q_math`);
// the shadowing local copies were removed by the placeholder-const sweep.

/// Raven `Vehicle_SetAnim`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:91-100`
pub fn Vehicle_SetAnim(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
    setAnimParts: c_int,
    anim: c_int,
    setAnimFlags: c_int,
    iBlend: c_int,
) {
    unsafe {
        // Raven: assert(ent->client);
        debug_assert!(!(*ent).client.is_null());
        let client = (*ent).client as *mut gclient_t;
        // MP `_JK2MP` path:
        //   BG_SetAnim(&client->ps, bgAllAnims[ent->localAnimIndex].anims,
        //              setAnimParts, anim, setAnimFlags, iBlend)
        // `BG_SetAnim` is a `PmoveContext` method (`bgAllAnims` off `BgState`);
        // build a pm-null per-call context from `ctx`, matching the `G_SetAnim`
        // game-tier wrapper precedent (`g_utils.rs`).
        let ps = &mut (*client).ps as *mut playerState_t;
        let anims = (*ctx.world).bg_state.bgAllAnims[(*ent).localAnimIndex as usize].anims;
        let traps = crate::bg_channel::GameBgTraps::new(ctx.engine);
        let mut callbacks = crate::bg_channel::GameCallbacksImpl {
            world: ctx.world,
            engine: ctx.engine,
        };
        let mut pmc = crate::bg_channel::PmoveContext::new(
            &mut (*ctx.world).bg_state,
            &traps,
            &mut callbacks,
        );
        pmc.BG_SetAnim(ps, anims, setAnimParts, anim, setAnimFlags, iBlend);
        (*ent).s.legsAnim = (*client).ps.legsAnim;
    }
}

/// Raven `G_VehicleTrace`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:102-109`
pub fn G_VehicleTrace(
    ctx: GameContext<'_>,
    results: *mut trace_t,
    start: vec3_t,
    tMins: vec3_t,
    tMaxs: vec3_t,
    end: vec3_t,
    passEntityNum: c_int,
    contentmask: c_int,
) {
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            results,
            &start as *const vec3_t,
            &tMins as *const vec3_t,
            &tMaxs as *const vec3_t,
            &end as *const vec3_t,
            passEntityNum,
            contentmask,
        ),
    );
}

/// Raven `G_IsRidingVehicle`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:111-120`
pub fn G_IsRidingVehicle(ctx: GameContext<'_>, pEnt: *mut gentity_t) -> *mut Vehicle_t {
    unsafe {
        let ent = pEnt;
        if !ent.is_null() && !(*ent).client.is_null() {
            let client = (*ent).client as *mut gclient_t;
            if (*client).NPC_class != CLASS_VEHICLE && (*ent).s.m_iVehicleNum != 0 {
                let vehNum = (*ent).s.m_iVehicleNum as usize;
                return (*ctx.world).g_entities[vehNum].m_pVehicle as *mut Vehicle_t;
            }
        }
        core::ptr::null_mut()
    }
}

/// Raven `G_CanJumpToEnemyVeh`.
///
/// Raven: the entire body is `#ifndef _JK2MP`; in the MP (`_JK2MP`) compile it
/// reduces to `return 0.0f;`.
/// Source: `oracle/codemp/game/g_vehicles.c:124-183`
pub fn G_CanJumpToEnemyVeh(pVeh: *mut Vehicle_t, pUcmd: *const usercmd_t) -> f32 {
    0.0
}

/// Raven `G_VehicleSpawn`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:186-244`
pub fn G_VehicleSpawn(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        (*self_).s.origin = (*self_).r.currentOrigin;
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(self_));

        if (*self_).count == 0 {
            (*self_).count = 1;
        }

        // save this because self gets removed in next func
        let yaw = (*self_).s.angles[YAW];

        let vehEnt = NPC_Spawn_Do(ctx, self_);
        if vehEnt.is_null() {
            return; // return NULL;
        }

        (*vehEnt).s.angles[YAW] = yaw;
        let vp = (*vehEnt).m_pVehicle as *mut Vehicle_t;
        let vi = (*vp).m_pVehicleInfo as *mut vehicleInfo_t;
        if (*vi).r#type != vehicleType_t::VH_ANIMAL {
            let npc = (*vehEnt).NPC as *mut gNPC_t;
            (*npc).behaviorState = bState_t::BS_CINEMATIC;
        }

        // special check in case someone disconnects/dies while boarding
        if (*vehEnt).spawnflags & 1 != 0 {
            // die without pilot
            if (*vehEnt).damage == 0 {
                // default 10 sec
                (*vehEnt).damage = 10000;
            }
            if (*vehEnt).speed == 0.0 {
                // default 512 units
                (*vehEnt).speed = 512.0;
            }
            (*vp).m_iPilotTime = (*ctx.world).level.time + (*vehEnt).damage;
        }
    }
}

/// Raven `G_AttachToVehicle`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:247-289`
pub fn G_AttachToVehicle(ctx: GameContext<'_>, pEnt: *mut gentity_t, ucmd: *mut *mut usercmd_t) {
    unsafe {
        if pEnt.is_null() || ucmd.is_null() {
            return;
        }

        let ent = pEnt;

        // MP: vehEnt = &g_entities[ent->r.ownerNum];
        let vehEnt = (*ctx.world)
            .g_entities
            .as_mut_ptr()
            .add((*ent).r.ownerNum as usize);
        (*ent).waypoint = (*vehEnt).waypoint; // take the veh's waypoint as your own

        if (*vehEnt).m_pVehicle.is_null() {
            return;
        }

        let crotchBolt = trap::G2API_AddBolt(
            ctx.engine,
            GG2AddboltArgs::new((*vehEnt).ghoul2 as *mut c_void, 0, cstr("*driver")),
        );

        // Get the driver tag.
        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
        let vp = (*vehEnt).m_pVehicle as *mut Vehicle_t;
        let entClient = (*ent).client as *mut gclient_t;
        trap::G2API_GetBoltMatrix(
            ctx.engine,
            GG2GetboltArgs::new(
                (*vehEnt).ghoul2 as *mut c_void,
                0,
                crotchBolt,
                &mut boltMatrix as *mut mdxaBone_t,
                // `m_vOrientation` is a `*mut f32` (see its PORT-NOTE below); the
                // pointer itself reinterprets as `*const vec3_t` — don't take a
                // reference to the pointer field.
                (*vp).m_vOrientation as *const vec3_t,
                &(*vehEnt).r.currentOrigin as *const vec3_t,
                (*ctx.world).level.time,
                core::ptr::null_mut(),
                &(*vehEnt).modelScale as *const vec3_t,
            ),
        );
        BG_GiveMeVectorFromMatrix(
            &boltMatrix,
            Eorientations::ORIGIN as c_int,
            &mut (*entClient).ps.origin,
        );
        crate::g_utils::G_SetOrigin(ent, (*entClient).ps.origin);
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));
    }
}

/// Raven `Animate` — animate the vehicle and its riders.
///
/// Source: `oracle/codemp/game/g_vehicles.c:481-493`
pub fn Animate(ctx: GameContext<'_>, pVeh: *mut Vehicle_t) {
    unsafe {
        // Validate a pilot rider. (The per-type dispatch no-ops for
        // vehicle types that leave the slot null, matching Raven's `if`-guard.)
        if !(*pVeh).m_pPilot.is_null() {
            crate::veh_dispatch::animate_riders(ctx, pVeh);
        }
        crate::veh_dispatch::animate_vehicle(ctx, pVeh);
    }
}

/// Raven `ValidateBoard`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:496-594`
pub fn ValidateBoard(
    ctx: GameContext<'_>,
    pVeh: *mut Vehicle_t,
    pEnt: *mut bgEntity_t,
) -> qboolean {
    unsafe {
        // Determine where the entity is entering the vehicle from (left, right, or back).
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let ent = pEnt as *mut gentity_t;
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;

        if (*pVeh).m_iDieTime > 0 {
            return qfalse;
        }

        if !(*pVeh).m_pPilot.is_null() {
            // already have a driver!
            if (*vi).r#type == vehicleType_t::VH_FIGHTER {
                // can never steal a fighter from its pilot
                if (*pVeh).m_iNumPassengers < (*vi).maxPassengers {
                    return qtrue;
                } else {
                    return qfalse;
                }
            } else if (*vi).r#type == vehicleType_t::VH_WALKER {
                // can only steal an occupied AT-ST if you're on top (by the hatch)
                let cl = (*ent).client as *mut gclient_t;
                if (*ent).client.is_null() || (*cl).ps.groundEntityNum != (*parent).s.number {
                    return qfalse;
                }
            } else if (*vi).r#type == vehicleType_t::VH_SPEEDER {
                // you can only steal the bike from the driver if you landed on the driver or bike
                if (*pVeh).m_iBoarding == VEH_MOUNT_THROW_LEFT
                    || (*pVeh).m_iBoarding == VEH_MOUNT_THROW_RIGHT
                {
                    return qtrue;
                } else {
                    return qfalse;
                }
            }
        } else if (*vi).r#type == vehicleType_t::VH_FIGHTER {
            // If you're a fighter, you allow everyone to enter you from all directions.
            return qtrue;
        }

        // Clear out all orientation axis except for the yaw.
        let vVehAngles: vec3_t = [0.0, (*parent).r.currentAngles[YAW], 0.0];

        // Vector from Entity to Vehicle.
        let mut vVehToEnt: vec3_t = [0.0; 3];
        _VectorSubtract(
            (*ent).r.currentOrigin,
            (*parent).r.currentOrigin,
            &mut vVehToEnt,
        );
        vVehToEnt[2] = 0.0;
        VectorNormalize(&mut vVehToEnt);

        // Get the right vector.
        let mut vVehDir: vec3_t = [0.0; 3];
        AngleVectors(vVehAngles, None, Some(&mut vVehDir), None);
        VectorNormalize(&mut vVehDir);

        // Find the angle between the vehicle right vector and the vehicle to entity vector.
        let fDot = _DotProduct(vVehToEnt, vVehDir);

        if fDot >= 0.5 {
            // Right board.
            (*pVeh).m_iBoarding = -2;
        } else if fDot <= -0.5 {
            // Left board.
            (*pVeh).m_iBoarding = -1;
        } else {
            // Maybe they're trying to board from the back... Jump board.
            (*pVeh).m_iBoarding = -3;
        }

        // If for some reason we couldn't board, leave...
        if (*pVeh).m_iBoarding > -1 {
            return qfalse;
        }

        qtrue
    }
}

/// Raven `Board`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:630-872`
pub fn Board(ctx: GameContext<'_>, pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> qboolean {
    unsafe {
        let ent = pEnt as *mut gentity_t;
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;

        // If it's not a valid entity, OR the vehicle is dead, OR we're already
        // being boarded, OR the person trying to get on is already in a vehicle.
        if ent.is_null() {
            return qfalse;
        }
        let entClient = (*ent).client as *mut gclient_t;
        // PORT-NOTE(m_vOrientation): Vehicle_t::m_vOrientation is transcribed as a
        // `vec3_t` value field (VectorClear/VectorCopy/VectorSet usage throughout
        // this TU treats it as a 3-float array). If the type port modeled it as a
        // `*mut f32` a fixer adjusts the copy sites here and in Update/Initialize.
        if (*parent).health <= 0 || (*pVeh).m_iBoarding > 0 || (*entClient).ps.m_iVehicleNum != 0 {
            return qfalse;
        }

        // Bucking so we can't do anything.
        if ((*pVeh).m_ulFlags & (VEH_BUCKING as u64)) != 0 {
            return qfalse;
        }

        // Validate the entity's ability to board this vehicle.
        if crate::veh_dispatch::validate_board(ctx, pVeh, pEnt) == qfalse {
            return qfalse;
        }

        // Tell everybody their status. ALWAYS let the player be the pilot.
        if (*ent).s.number < MAX_CLIENTS as c_int {
            (*pVeh).m_pOldPilot = (*pVeh).m_pPilot;

            if (*pVeh).m_pPilot.is_null() {
                // become the pilot, if there isn't one now
                crate::veh_dispatch::set_pilot(ctx, pVeh, ent as *mut bgEntity_t);
            } else if (*pVeh).m_iNumPassengers < (*vi).maxPassengers {
                // Find an empty slot and put that passenger here.
                let mut i: c_int = 0;
                while i < (*vi).maxPassengers {
                    if (*(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize)).is_null() {
                        *(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize) =
                            ent as *mut mp_bg::public::bg_entity::bgEntity_t;
                        // Server just needs to tell client which passengernum he is
                        if !(*ent).client.is_null() {
                            (*entClient).ps.generic1 = i + 1;
                        }
                        break;
                    }
                    i += 1;
                }
                (*pVeh).m_iNumPassengers += 1;
            } else {
                // We're full, sorry...
                return qfalse;
            }
            (*ent).s.m_iVehicleNum = (*parent).s.number;
            if !(*ent).client.is_null() {
                (*entClient).ps.m_iVehicleNum = (*ent).s.m_iVehicleNum;
            }
            if (*pVeh).m_pPilot == (ent as *mut mp_bg::public::bg_entity::bgEntity_t) {
                (*parent).r.ownerNum = (*ent).s.number;
                (*parent).s.owner = (*parent).r.ownerNum; // for prediction
            }

            // QAGAME: undock if we were being suspended.
            {
                let gParent = parent;
                if ((*gParent).spawnflags & 2) != 0 {
                    // was being suspended
                    (*gParent).spawnflags &= !2;
                    crate::g_utils::G_Sound(
                        ctx,
                        gParent,
                        CHAN_AUTO,
                        G_SoundIndex(c"sound/vehicles/common/release.wav".as_ptr()),
                    );
                    if (*gParent).fly_sound_debounce_time != 0 {
                        // we should drop like a rock for a few seconds
                        (*pVeh).m_iDropTime =
                            (*ctx.world).level.time + (*gParent).fly_sound_debounce_time;
                    }
                }
            }

            // Set the looping sound only when there is a pilot (vehicle is "on").
            if (*vi).soundLoop != 0 {
                let pc = (*parent).client as *mut gclient_t;
                (*parent).s.loopSound = (*vi).soundLoop;
                (*pc).ps.loopSound = (*vi).soundLoop;
            }
        } else {
            // If there's no pilot, try to drive this vehicle.
            if (*pVeh).m_pPilot.is_null() {
                crate::veh_dispatch::set_pilot(ctx, pVeh, ent as *mut bgEntity_t);
                // TODO: Set pilot should do all this stuff....
                (*parent).r.ownerNum = (*ent).s.number;
                (*parent).s.owner = (*parent).r.ownerNum; // for prediction

                // Set the looping sound only when there is a pilot.
                if (*vi).soundLoop != 0 {
                    let pc = (*parent).client as *mut gclient_t;
                    (*parent).s.loopSound = (*vi).soundLoop;
                    (*pc).ps.loopSound = (*vi).soundLoop;
                }

                let pc = (*parent).client as *mut gclient_t;
                (*pc).ps.speed = 0.0;
                (*pVeh).m_ucmd = core::mem::zeroed();
            } else if (*pVeh).m_iNumPassengers < (*vi).maxPassengers {
                // Find an empty slot and put that passenger here.
                let mut i: c_int = 0;
                while i < (*vi).maxPassengers {
                    if (*(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize)).is_null() {
                        *(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize) =
                            ent as *mut mp_bg::public::bg_entity::bgEntity_t;
                        // Server just needs to tell client which passengernum he is
                        if !(*ent).client.is_null() {
                            (*entClient).ps.generic1 = i + 1;
                        }
                        break;
                    }
                    i += 1;
                }
                (*pVeh).m_iNumPassengers += 1;
            } else {
                // We're full, sorry...
                return qfalse;
            }
        }

        // Make sure the entity knows it's in a vehicle. (MP)
        (*entClient).ps.m_iVehicleNum = (*parent).s.number;
        (*ent).r.ownerNum = (*parent).s.number;
        (*ent).s.owner = (*ent).r.ownerNum; // for prediction
        if (*pVeh).m_pPilot == (ent as *mut mp_bg::public::bg_entity::bgEntity_t) {
            let pc = (*parent).client as *mut gclient_t;
            // always gonna be under MAX_CLIENTS so no worries about 1 byte overflow
            (*pc).ps.m_iVehicleNum = (*ent).s.number + 1;
        }

        // numHands==2 switch-to-vehicle-weapon body is `#ifndef _JK2MP` (SP only):
        // MP does nothing here.

        if (*vi).hideRider != 0 {
            // hide the rider
            crate::veh_dispatch::ghost(ctx, pVeh, ent as *mut bgEntity_t);
        }

        // Play the start sounds.
        if (*vi).soundOn != 0 {
            crate::g_utils::G_Sound(ctx, parent, CHAN_AUTO, (*vi).soundOn);
        }

        let mut vPlayerDir: vec3_t = [0.0; 3];
        _VectorCopy(*((*pVeh).m_vOrientation as *const vec3_t), &mut vPlayerDir);
        vPlayerDir[ROLL] = 0.0;
        crate::g_client::SetClientViewAngle(ent, vPlayerDir);

        qtrue
    }
}

/// Raven `VEH_TryEject`.
///
/// `vExitPos` is Raven's out-param exit position (written through, never
/// NULL at any oracle caller) → `&mut vec3_t`.
/// Source: `oracle/codemp/game/g_vehicles.c:874-987`
pub fn VEH_TryEject(
    ctx: GameContext<'_>,
    pVeh: *mut Vehicle_t,
    parent: *mut gentity_t,
    ent: *mut gentity_t,
    ejectDir: c_int,
    vExitPos: &mut vec3_t,
) -> qboolean {
    unsafe {
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;

        // Make sure that the entity is not 'stuck' inside the vehicle (since their
        // bboxes will now intersect). Leave the vehicle from the right side.
        let vVehAngles: vec3_t = [0.0, (*parent).r.currentAngles[YAW], 0.0];
        let mut vVehLeaveDir: vec3_t = [0.0; 3];
        match ejectDir {
            VEH_EJECT_LEFT => {
                AngleVectors(vVehAngles, None, Some(&mut vVehLeaveDir), None);
                vVehLeaveDir[0] = -vVehLeaveDir[0];
                vVehLeaveDir[1] = -vVehLeaveDir[1];
                vVehLeaveDir[2] = -vVehLeaveDir[2];
            }
            VEH_EJECT_RIGHT => {
                AngleVectors(vVehAngles, None, Some(&mut vVehLeaveDir), None);
            }
            VEH_EJECT_FRONT => {
                AngleVectors(vVehAngles, Some(&mut vVehLeaveDir), None, None);
            }
            VEH_EJECT_REAR => {
                AngleVectors(vVehAngles, Some(&mut vVehLeaveDir), None, None);
                vVehLeaveDir[0] = -vVehLeaveDir[0];
                vVehLeaveDir[1] = -vVehLeaveDir[1];
                vVehLeaveDir[2] = -vVehLeaveDir[2];
            }
            VEH_EJECT_TOP => {
                AngleVectors(vVehAngles, None, None, Some(&mut vVehLeaveDir));
            }
            VEH_EJECT_BOTTOM => {}
            _ => {}
        }
        VectorNormalize(&mut vVehLeaveDir);

        // Diagonal Length == sqrt( sqr(Sidex/2) + sqr(Sidey/2) ).
        let mut fBias = 1.0f32;
        if (*vi).r#type == vehicleType_t::VH_WALKER {
            // hacktastic!
            fBias += 0.2;
        }
        _VectorCopy((*ent).r.currentOrigin, vExitPos);
        let fVehDiag = ((*parent).r.maxs[0] * (*parent).r.maxs[0]
            + (*parent).r.maxs[1] * (*parent).r.maxs[1])
            .sqrt();
        let mut vEntMaxs: vec3_t = (*ent).r.maxs;
        if (*ent).s.number < MAX_CLIENTS as c_int {
            // in MP, player client mins/maxs are never stored permanently, just set
            // to these hardcoded numbers in PMove.
            vEntMaxs[0] = 15.0;
            vEntMaxs[1] = 15.0;
        }
        let fEntDiag = (vEntMaxs[0] * vEntMaxs[0] + vEntMaxs[1] * vEntMaxs[1]).sqrt();
        vVehLeaveDir[0] *= (fVehDiag + fEntDiag) * fBias;
        vVehLeaveDir[1] *= (fVehDiag + fEntDiag) * fBias;
        vVehLeaveDir[2] *= (fVehDiag + fEntDiag) * fBias;
        let curExit = *vExitPos;
        _VectorAdd(curExit, vVehLeaveDir, vExitPos);

        // Check to see if this new position is a valid place for our entity to go.
        let vEntMins: vec3_t = [-15.0, -15.0, DEFAULT_MINS_2];
        let vEntMaxs2: vec3_t = [15.0, 15.0, DEFAULT_MAXS_2];
        let oldOwner = (*ent).r.ownerNum;
        (*ent).r.ownerNum = ENTITYNUM_NONE;
        let mut m_ExitTrace: trace_t = core::mem::zeroed();
        G_VehicleTrace(
            ctx,
            &mut m_ExitTrace,
            (*ent).r.currentOrigin,
            vEntMins,
            vEntMaxs2,
            *vExitPos,
            (*ent).s.number,
            (*ent).clipmask,
        );
        (*ent).r.ownerNum = oldOwner;

        if m_ExitTrace.allsolid != 0 || m_ExitTrace.startsolid != 0 {
            // in solid
            return qfalse;
        }
        // If the trace hit something, we can't go there!
        if m_ExitTrace.fraction < 1.0 {
            // not totally clear. In MP the inner `(parent->clipmask&ent->r.contents)`
            // guard is commented out in the oracle, so the "don't let them get out"
            // block runs unconditionally and the trace.endpos fallback below is dead.
            return qfalse;
        }
        qtrue
    }
}

/// Raven `G_EjectDroidUnit`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:989-1016`
pub fn G_EjectDroidUnit(ctx: GameContext<'_>, pVeh: *mut Vehicle_t, kill: qboolean) {
    unsafe {
        let droid = (*pVeh).m_pDroidUnit as *mut gentity_t;
        (*droid).s.m_iVehicleNum = ENTITYNUM_NONE;
        (*droid).s.owner = ENTITYNUM_NONE; // MP

        // QAGAME
        let droidEnt = (*pVeh).m_pDroidUnit as *mut gentity_t;
        (*droidEnt).flags &= !FL_UNDYING;
        (*droidEnt).r.ownerNum = ENTITYNUM_NONE;
        if !(*droidEnt).client.is_null() {
            let dc = (*droidEnt).client as *mut gclient_t;
            (*dc).ps.m_iVehicleNum = ENTITYNUM_NONE;
        }
        if kill != qfalse {
            // Kill them, too.
            crate::g_utils::G_MuteSound(ctx, (*droidEnt).s.number, CHAN_VOICE);
            // PORT-NOTE(G_Damage-null-dir): Raven passes NULL for `dir`; the resolved
            // G_Damage takes `dir: &mut vec3_t` (reshape covered only OUT-params, not
            // nullable INs) — a zero vec3 stands in for the C NULL. See shape_mismatch.
            crate::g_combat::G_Damage(
                ctx,
                droidEnt,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                None,
                (*droidEnt).s.origin,
                10000,
                0,
                MOD_SUICIDE as c_int,
            );
        }

        (*pVeh).m_pDroidUnit = core::ptr::null_mut();
    }
}

/// Raven `EjectAll`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:1377-1448`
pub fn EjectAll(ctx: GameContext<'_>, pVeh: *mut Vehicle_t) -> qboolean {
    unsafe {
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;

        // TODO: Setup a default escape for every vehicle type.
        (*pVeh).m_EjectDir = VEH_EJECT_TOP;
        // Make sure no other boarding calls exist. We MUST exit.
        (*pVeh).m_iBoarding = 0;
        (*pVeh).m_bWasBoarding = qfalse;

        // Throw them off.
        if !(*pVeh).m_pPilot.is_null() {
            let pilot = (*pVeh).m_pPilot as *mut gentity_t;
            crate::veh_dispatch::eject(ctx, pVeh, (*pVeh).m_pPilot as *mut bgEntity_t, qtrue);
            if (*vi).killRiderOnDeath != qfalse && !pilot.is_null() {
                crate::g_utils::G_MuteSound(ctx, (*pilot).s.number, CHAN_VOICE);
                crate::g_combat::G_Damage(
                    ctx,
                    pilot,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    None,
                    (*pilot).s.origin,
                    10000,
                    0,
                    MOD_SUICIDE as c_int,
                );
            }
        }
        if !(*pVeh).m_pOldPilot.is_null() {
            let pilot = (*pVeh).m_pOldPilot as *mut gentity_t;
            crate::veh_dispatch::eject(ctx, pVeh, (*pVeh).m_pOldPilot as *mut bgEntity_t, qtrue);
            if (*vi).killRiderOnDeath != qfalse && !pilot.is_null() {
                crate::g_utils::G_MuteSound(ctx, (*pilot).s.number, CHAN_VOICE);
                crate::g_combat::G_Damage(
                    ctx,
                    pilot,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    None,
                    (*pilot).s.origin,
                    10000,
                    0,
                    MOD_SUICIDE as c_int,
                );
            }
        }
        if (*pVeh).m_iNumPassengers != 0 {
            let mut i: c_int = 0;
            while i < (*vi).maxPassengers {
                if !(*(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize)).is_null() {
                    let rider =
                        *(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize) as *mut gentity_t;
                    crate::veh_dispatch::eject(
                        ctx,
                        pVeh,
                        *(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize) as *mut gentity_t,
                        qtrue,
                    );
                    if (*vi).killRiderOnDeath != qfalse && !rider.is_null() {
                        crate::g_utils::G_MuteSound(ctx, (*rider).s.number, CHAN_VOICE);
                        crate::g_combat::G_Damage(
                            ctx,
                            rider,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            None,
                            (*rider).s.origin,
                            10000,
                            0,
                            MOD_SUICIDE as c_int,
                        );
                    }
                }
                i += 1;
            }
            (*pVeh).m_iNumPassengers = 0;
        }

        if !(*pVeh).m_pDroidUnit.is_null() {
            G_EjectDroidUnit(ctx, pVeh, (*vi).killRiderOnDeath);
        }

        qtrue
    }
}

/// Raven `StartDeathDelay`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:1451-1482`
pub fn StartDeathDelay(ctx: GameContext<'_>, pVeh: *mut Vehicle_t, iDelayTimeOverride: c_int) {
    unsafe {
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;
        let level_time = (*ctx.world).level.time;

        if iDelayTimeOverride != 0 {
            (*pVeh).m_iDieTime = level_time + iDelayTimeOverride;
        } else {
            (*pVeh).m_iDieTime = level_time + (*vi).explosionDelay;
        }

        if (*vi).flammable != qfalse {
            let snd = G_SoundIndex(c"sound/vehicles/common/fire_lp.wav".as_ptr());
            let client = (*parent).client as *mut gclient_t;
            (*parent).s.loopSound = snd;
            (*client).ps.loopSound = snd;
        }
    }
}

/// Raven `Initialize`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:1626-1757`
pub fn Initialize(ctx: GameContext<'_>, pVeh: *mut Vehicle_t) -> qboolean {
    unsafe {
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;

        if parent.is_null() || (*parent).client.is_null() {
            return qfalse;
        }
        let pc = (*parent).client as *mut gclient_t;

        (*pc).ps.m_iVehicleNum = 0; // MP
        (*parent).s.m_iVehicleNum = 0;
        {
            (*pVeh).m_iArmor = (*vi).armor;
            let hp = (*pVeh).m_iArmor;
            let npc = (*parent).NPC as *mut gNPC_t;
            (*pc).ps.stats[STAT_HEALTH as usize] = hp;
            (*parent).health = hp;
            (*npc).stats.health = hp;
            (*pc).ps.stats[STAT_MAX_HEALTH as usize] = hp;
            (*pc).pers.maxHealth = hp;
            (*pVeh).m_iShields = (*vi).shields;
            G_VehUpdateShields(parent); // MP
            (*pc).ps.stats[STAT_ARMOR as usize] = (*pVeh).m_iShields;
        }
        (*parent).mass = ((*vi).mass) as f32;

        // initialize the ammo to max
        let mut i: c_int = 0;
        while i < MAX_VEHICLE_WEAPONS as c_int {
            let m = (*vi).weapon[i as usize].ammoMax;
            (*pVeh).weaponStatus[i as usize].ammo = m;
            (*pc).ps.ammo[i as usize] = m;
            i += 1;
        }
        i = 0;
        while i < MAX_VEHICLE_TURRETS as c_int {
            (*pVeh).turretStatus[i as usize].nextMuzzle =
                (*vi).turret[i as usize].iMuzzle[i as usize] - 1;
            let m = (*vi).turret[i as usize].iAmmoMax;
            (*pVeh).turretStatus[i as usize].ammo = m;
            (*pc).ps.ammo[MAX_VEHICLE_WEAPONS + i as usize] = m;
            if (*vi).turret[i as usize].bAI != qfalse {
                // they're going to be finding enemies, init this to NONE
                (*pVeh).turretStatus[i as usize].enemyEntNum = ENTITYNUM_NONE;
            }
            i += 1;
        }
        // begin stopped...?
        (*pc).ps.speed = 0.0;

        *((*pVeh).m_vOrientation as *mut vec3_t) = [0.0; 3];
        *(*pVeh).m_vOrientation.add(YAW as usize) = (*parent).s.angles[YAW];

        // MP gravity
        if (*vi).gravity != 0 && (*vi).gravity as f32 != (*ctx.world).cvars.g_gravity.value {
            // not normal gravity
            if !(*parent).NPC.is_null() {
                let npc = (*parent).NPC as *mut gNPC_t;
                (*npc).aiFlags |= NPCAI_CUSTOM_GRAVITY;
            }
            (*pc).ps.gravity = (*vi).gravity;
        }

        if (*vi).maxPassengers > 0 {
            // MP uses the static pointer array; just NULL every slot.
            let mut i: c_int = 0;
            while i < (*vi).maxPassengers {
                *(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize) = core::ptr::null_mut();
                i += 1;
            }
        }

        (*pVeh).m_iNumPassengers = 0;
        (*pVeh).m_ulFlags = 0;
        (*pVeh).m_fTimeModifier = 1.0f32;
        (*pVeh).m_iBoarding = 0;
        (*pVeh).m_bWasBoarding = qfalse;
        (*pVeh).m_pOldPilot = core::ptr::null_mut();
        (*pVeh).m_vBoardingVelocity = [0.0; 3];
        (*pVeh).m_pPilot = core::ptr::null_mut();
        (*pVeh).m_ucmd = core::mem::zeroed();
        (*pVeh).m_iDieTime = 0;
        (*pVeh).m_EjectDir = VEH_EJECT_LEFT;

        // memset(-1) over int arrays: byte 0xFF fills each int with -1.
        (*pVeh).m_iExhaustTag.fill(-1);
        (*pVeh).m_iMuzzleTag.fill(-1);
        // m_Muzzles memset is `#ifndef _JK2MP` (SP only) — skipped.
        (*pVeh).m_iDroidUnitTag = -1;

        // initialize to blaster
        (*pc).ps.weapon = WP_BLASTER;
        (*pc).ps.weaponstate = WEAPON_READY as c_int;
        (*pc).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_BLASTER;

        // Initialize to landed (wings closed, gears down) animation.
        {
            let iFlags = SETANIM_FLAG_NORMAL;
            let iBlend = 300;
            (*pVeh).m_ulFlags |= (VEH_GEARSOPEN as u64); // MP
            // MP `_JK2MP` path:
            //   BG_SetAnim(pVeh->m_pParentEntity->playerState,
            //              bgAllAnims[pVeh->m_pParentEntity->localAnimIndex].anims,
            //              SETANIM_BOTH, BOTH_VS_IDLE, iFlags, iBlend)
            // `BG_SetAnim` is a `PmoveContext<'_>` method (bgAllAnims off BgState +
            // receiver); build a pm-null per-call context from `ctx`, matching the
            // `Vehicle_SetAnim` precedent above.
            let ps = &mut (*pc).ps as *mut playerState_t;
            let anims =
                (*ctx.world).bg_state.bgAllAnims[(*parent).localAnimIndex as usize].anims;
            let traps = GameBgTraps::new(ctx.engine);
            let mut callbacks = crate::bg_channel::GameCallbacksImpl {
                world: ctx.world,
                engine: ctx.engine,
            };
            let mut pmc = crate::bg_channel::PmoveContext::new(
                &mut (*ctx.world).bg_state,
                &traps,
                &mut callbacks,
            );
            pmc.BG_SetAnim(ps, anims, SETANIM_BOTH, BOTH_VS_IDLE as c_int, iFlags, iBlend);
        }

        qtrue
    }
}

/// Raven `Update`.
///
/// PORT-NOTE(bg-boundary): `Update` is stored as the `vehicleInfo_t`
/// `Update` vtable slot and dispatched from the bg/vehicle-update path, so its LAW
/// signature carries no channel. Its body nonetheless reads `level.time`/`g_entities`,
/// draws from the RNG, and calls ctx-requiring fns (`VEH_TurretThink`,
/// `G_VehicleDamageBoxSizing`, `BG_UnrestrainedPitchRoll`). Those are transcribed
/// against the game channel `ctx`, which must be threaded in by the
/// vtable-dispatch retrofit (see shape_mismatch). All other logic is faithful MP+QAGAME.
///
/// Source: `oracle/codemp/game/g_vehicles.c:1763-2334`
pub fn Update(ctx: GameContext<'_>, pVeh: *mut Vehicle_t, pUmcd: *const usercmd_t) -> qboolean {
    unsafe {
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;
        let pclient = (*parent).client as *mut gclient_t;
        // MP: parentPS = pVeh->m_pParentEntity->playerState (== &parent->client->ps)
        let parentPS = &mut (*pclient).ps as *mut playerState_t;

        // QAGAME: curTime = level.time
        let curTime = (*ctx.world).level.time;

        // increment the ammo for all rechargeable weapons
        let mut i: c_int = 0;
        while i < MAX_VEHICLE_WEAPONS as c_int {
            let iu = i as usize;
            if (*vi).weapon[iu].ID > VEH_WEAPON_BASE
                && (*vi).weapon[iu].ammoRechargeMS != 0
                && (*pVeh).weaponStatus[iu].ammo < (*vi).weapon[iu].ammoMax
                && (*pUmcd).serverTime - (*pVeh).weaponStatus[iu].lastAmmoInc
                    >= (*vi).weapon[iu].ammoRechargeMS
            {
                (*pVeh).weaponStatus[iu].lastAmmoInc = (*pUmcd).serverTime;
                (*pVeh).weaponStatus[iu].ammo += 1;
                if !parent.is_null() && !(*parent).client.is_null() {
                    (*pclient).ps.ammo[iu] = (*pVeh).weaponStatus[iu].ammo;
                }
            }
            i += 1;
        }
        i = 0;
        while i < MAX_VEHICLE_TURRETS as c_int {
            let iu = i as usize;
            if (*vi).turret[iu].iWeapon > VEH_WEAPON_BASE
                && (*vi).turret[iu].iAmmoRechargeMS != 0
                && (*pVeh).turretStatus[iu].ammo < (*vi).turret[iu].iAmmoMax
                && (*pUmcd).serverTime - (*pVeh).turretStatus[iu].lastAmmoInc
                    >= (*vi).turret[iu].iAmmoRechargeMS
            {
                (*pVeh).turretStatus[iu].lastAmmoInc = (*pUmcd).serverTime;
                (*pVeh).turretStatus[iu].ammo += 1;
                if !parent.is_null() && !(*parent).client.is_null() {
                    (*pclient).ps.ammo[MAX_VEHICLE_WEAPONS + i as usize] =
                        (*pVeh).turretStatus[iu].ammo;
                }
            }
            i += 1;
        }

        // increment shields for rechargable shields
        if (*vi).shieldRechargeMS != 0
            && (*parentPS).stats[STAT_ARMOR as usize] > 0
            && (*parentPS).stats[STAT_ARMOR as usize] < (*vi).shields
            && (*pUmcd).serverTime - (*pVeh).lastShieldInc >= (*vi).shieldRechargeMS
        {
            (*parentPS).stats[STAT_ARMOR as usize] += 1;
            if (*parentPS).stats[STAT_ARMOR as usize] > (*vi).shields {
                (*parentPS).stats[STAT_ARMOR as usize] = (*vi).shields;
            }
            (*pVeh).m_iShields = (*parentPS).stats[STAT_ARMOR as usize];
            G_VehUpdateShields(parent); // MP
        }

        // MP: sometimes owner gets out of whack
        if !parent.is_null() && (*parent).r.ownerNum != (*parent).s.owner {
            (*parent).s.owner = (*parent).r.ownerNum;
        }
        // keep the PS value in sync
        if (*pVeh).m_iBoarding != 0 {
            (*pclient).ps.vehBoarding = qtrue;
        } else {
            (*pclient).ps.vehBoarding = qfalse;
        }

        // See whether this vehicle should be dieing or dead. (MP: `m_iDieTime != 0`)
        if (*pVeh).m_iDieTime != 0 {
            // Keep track of the old orientation.
            _VectorCopy(
                *((*pVeh).m_vOrientation as *const vec3_t),
                &mut (*pVeh).m_vPrevOrientation,
            );
            crate::veh_dispatch::process_orient_commands(ctx, pVeh);
            SetClientViewAngle(parent, *((*pVeh).m_vOrientation as *const vec3_t));
            if !(*pVeh).m_pPilot.is_null() {
                SetClientViewAngle(
                    (*pVeh).m_pPilot as *mut gentity_t,
                    *((*pVeh).m_vOrientation as *const vec3_t),
                );
            }
            crate::veh_dispatch::process_move_commands(ctx, pVeh);
            if (*vi).r#type == vehicleType_t::VH_FIGHTER {
                AngleVectors(
                    *((*pVeh).m_vOrientation as *const vec3_t),
                    Some(&mut (*pclient).ps.moveDir),
                    None,
                    None,
                );
            } else {
                let vVehAngles: vec3_t = [0.0, *(*pVeh).m_vOrientation.add(YAW as usize), 0.0];
                AngleVectors(vVehAngles, Some(&mut (*pclient).ps.moveDir), None, None);
            }
            crate::veh_dispatch::death_update(ctx, pVeh);
            return qfalse;
        }
        // Vehicle dead! (MP)
        else if (*parent).health <= 0 {
            // Instant kill.
            if (*vi).r#type == vehicleType_t::VH_FIGHTER && (*pVeh).m_iLastImpactDmg > 500 {
                // explode instantly in inferno-y death (-1 causes instant death)
                crate::veh_dispatch::start_death_delay(ctx, pVeh, -1);
            } else {
                crate::veh_dispatch::start_death_delay(ctx, pVeh, 0);
            }
            crate::veh_dispatch::death_update(ctx, pVeh);
            return qfalse;
        }

        // MP QAGAME: special check in case someone disconnects/dies while boarding
        if (*parent).spawnflags & 1 != 0 {
            if !(*pVeh).m_pPilot.is_null() || (*pVeh).m_bHasHadPilot == qfalse {
                if !(*pVeh).m_pPilot.is_null() && (*pVeh).m_bHasHadPilot == qfalse {
                    (*pVeh).m_bHasHadPilot = qtrue;
                    (*pVeh).m_iPilotLastIndex = (*((*pVeh).m_pPilot as *mut gentity_t)).s.number;
                }
                (*pVeh).m_iPilotTime = (*ctx.world).level.time + (*parent).damage;
            } else if (*pVeh).m_iPilotTime != 0 {
                // die
                let oldPilot = (*ctx.world)
                    .g_entities
                    .as_mut_ptr()
                    .add((*pVeh).m_iPilotLastIndex as usize);
                let oldPilotConnected = !(*oldPilot).client.is_null()
                    && (*((*oldPilot).client as *mut gclient_t)).pers.connected == CON_CONNECTED;
                if (*oldPilot).inuse == qfalse || (*oldPilot).client.is_null() || !oldPilotConnected
                {
                    // no longer in the game?
                    crate::g_combat::G_Damage(
                        ctx,
                        parent,
                        parent,
                        parent,
                        None,
                        (*pclient).ps.origin,
                        99999,
                        DAMAGE_NO_PROTECTION,
                        MOD_SUICIDE as c_int,
                    );
                } else {
                    let oc = (*oldPilot).client as *mut gclient_t;
                    let mut v: vec3_t = [0.0; 3];
                    _VectorSubtract((*pclient).ps.origin, (*oc).ps.origin, &mut v);
                    if VectorLength(v) < (*parent).speed {
                        // still within the minimum distance to their vehicle
                        (*pVeh).m_iPilotTime = (*ctx.world).level.time + (*parent).damage;
                    } else if (*pVeh).m_iPilotTime < (*ctx.world).level.time {
                        // dying time
                        crate::g_combat::G_Damage(
                            ctx,
                            parent,
                            parent,
                            parent,
                            None,
                            (*pclient).ps.origin,
                            99999,
                            DAMAGE_NO_PROTECTION,
                            MOD_SUICIDE as c_int,
                        );
                    }
                }
            }
        }

        // MP: the "always knock guys around" block is `#ifndef _JK2MP` (SP only) — skipped.

        // MP: eject if the pilot disconnected/died while boarding
        if (*pVeh).m_iBoarding != 0 {
            let pilotEnt = (*pVeh).m_pPilot as *mut gentity_t;
            if !pilotEnt.is_null() {
                let pec = (*pilotEnt).client as *mut gclient_t;
                if (*pilotEnt).inuse == qfalse
                    || (*pilotEnt).client.is_null()
                    || (*pilotEnt).health <= 0
                    || (*pec).pers.connected != CON_CONNECTED
                {
                    crate::veh_dispatch::eject(
                        ctx,
                        pVeh,
                        (*pVeh).m_pPilot as *mut bgEntity_t,
                        qtrue,
                    );
                    return qfalse;
                }
            }
        }

        // If we're not done mounting, can't do anything.
        let mut boarding_maintain = false; // Raven `goto maintainSelfDuringBoarding`
        if (*pVeh).m_iBoarding != 0 {
            if (*pVeh).m_bWasBoarding == qfalse {
                _VectorCopy((*parentPS).velocity, &mut (*pVeh).m_vBoardingVelocity);
                (*pVeh).m_bWasBoarding = qtrue;
            }
            // See if we're done boarding.
            if (*pVeh).m_iBoarding > -1 && (*pVeh).m_iBoarding <= (*ctx.world).level.time {
                (*pVeh).m_bWasBoarding = qfalse;
                (*pVeh).m_iBoarding = 0;
            } else {
                boarding_maintain = true;
            }
        }

        if !boarding_maintain {
            let parent = (*pVeh).m_pParentEntity as *mut gentity_t;

            // Validate vehicle.
            if parent.is_null() || (*parent).client.is_null() || (*parent).health <= 0 {
                return qfalse;
            }

            // See if any of the riders are dead and if so kick em off.
            if !(*pVeh).m_pPilot.is_null() {
                let pilotEnt = (*pVeh).m_pPilot as *mut gentity_t;
                let pec = (*pilotEnt).client as *mut gclient_t;
                if (*pilotEnt).inuse == qfalse
                    || (*pilotEnt).client.is_null()
                    || (*pilotEnt).health <= 0
                    || (*pec).pers.connected != CON_CONNECTED
                {
                    crate::veh_dispatch::eject(
                        ctx,
                        pVeh,
                        (*pVeh).m_pPilot as *mut bgEntity_t,
                        qtrue,
                    );
                }
            }
            // If we're not empty...
            if (*pVeh).m_iNumPassengers > 0 {
                let mut i: c_int = 0;
                while i < (*vi).maxPassengers {
                    let psngr =
                        *(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize) as *mut gentity_t;
                    if !psngr.is_null() {
                        let sc = (*psngr).client as *mut gclient_t;
                        if (*psngr).inuse == qfalse
                            || (*psngr).client.is_null()
                            || (*psngr).health <= 0
                            || (*sc).pers.connected != CON_CONNECTED
                        {
                            crate::veh_dispatch::eject(
                                ctx,
                                pVeh,
                                *(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize)
                                    as *mut gentity_t,
                                qtrue,
                            );
                            (*pVeh).m_iNumPassengers -= 1;
                        }
                    }
                    i += 1;
                }
            }

            // MP: Copy over the commands for local storage.
            (*pclient).pers.cmd = (*pVeh).m_ucmd;
            (*pVeh).m_ucmd.buttons &= !(BUTTON_TALK); // don't want some of these buttons

            // check for weapon linking/unlinking command
            let mut linkHeld = qfalse;
            let mut i: c_int = 0;
            while i < MAX_VEHICLE_WEAPONS as c_int {
                let iu = i as usize;
                if (*vi).weapon[iu].linkable == 2 {
                    // always linked
                    if (*pVeh).weaponStatus[iu].linked == qfalse {
                        (*pVeh).weaponStatus[iu].linked = qtrue;
                    }
                } else if ((*pVeh).m_ucmd.buttons & BUTTON_USE_HOLDABLE) != 0 {
                    // pilot pressed the "weapon link" toggle button
                    // PORT-NOTE(PM_BGEntForNum): MP computes `rider`/`pilotPS` here via
                    // PM_BGEntForNum(parent->s.owner) but never reads pilotPS in this
                    // path; the pure lookup has no side effects, so it is elided.
                    if (*pVeh).linkWeaponToggleHeld == qfalse {
                        // okay to toggle
                        if (*vi).weapon[iu].linkable == 1 {
                            // link-toggleable
                            (*pVeh).weaponStatus[iu].linked =
                                if (*pVeh).weaponStatus[iu].linked != qfalse {
                                    qfalse
                                } else {
                                    qtrue
                                };
                        }
                    }
                    linkHeld = qtrue;
                }
                i += 1;
            }
            if linkHeld != qfalse {
                (*pVeh).linkWeaponToggleHeld = qtrue;
            } else {
                (*pVeh).linkWeaponToggleHeld = qfalse;
            }
            // MP: pass link state over the network so cgame knows
            (*parentPS).vehWeaponsLinked = qfalse;
            let mut i: c_int = 0;
            while i < MAX_VEHICLE_WEAPONS as c_int {
                if (*pVeh).weaponStatus[i as usize].linked != qfalse {
                    (*parentPS).vehWeaponsLinked = qtrue;
                }
                i += 1;
            }

            // QAGAME turrets
            let mut i: c_int = 0;
            while i < MAX_VEHICLE_TURRETS as c_int {
                crate::g_vehicleTurret::VEH_TurretThink(ctx, pVeh, parent, i);
                i += 1;
            }
        }

        // maintainSelfDuringBoarding: (MP)
        if !(*pVeh).m_pPilot.is_null()
            && !(*(*pVeh).m_pPilot).playerState.is_null()
            && (*pVeh).m_iBoarding != 0
        {
            let pilotPS = (*(*pVeh).m_pPilot).playerState;
            _VectorCopy(
                *((*pVeh).m_vOrientation as *const vec3_t),
                &mut (*pilotPS).viewangles,
            );
            (*pVeh).m_ucmd.buttons = 0;
            (*pVeh).m_ucmd.forwardmove = 0;
            (*pVeh).m_ucmd.rightmove = 0;
            (*pVeh).m_ucmd.upmove = 0;
        }

        // Keep track of the old orientation.
        _VectorCopy(
            *((*pVeh).m_vOrientation as *const vec3_t),
            &mut (*pVeh).m_vPrevOrientation,
        );

        // Process the orient commands.
        crate::veh_dispatch::process_orient_commands(ctx, pVeh);
        SetClientViewAngle(parent, *((*pVeh).m_vOrientation as *const vec3_t));
        if !(*pVeh).m_pPilot.is_null() {
            // MP
            let pilotPS = (*(*pVeh).m_pPilot).playerState;
            if crate::bg_pmove::BG_UnrestrainedPitchRoll(pilotPS, pVeh, &(*ctx.world).bg_state)
                == qfalse
            {
                let mut newVAngle: vec3_t = [0.0; 3];
                newVAngle[PITCH] = (*pilotPS).viewangles[PITCH];
                newVAngle[YAW] = (*pilotPS).viewangles[YAW];
                newVAngle[ROLL] = *(*pVeh).m_vOrientation.add(ROLL as usize);
                SetClientViewAngle((*pVeh).m_pPilot as *mut gentity_t, newVAngle);
            }
        }

        // Process the move commands.
        let prevSpeed = (*parentPS).speed;
        crate::veh_dispatch::process_move_commands(ctx, pVeh);
        let nextSpeed = (*parentPS).speed;
        let halfMaxSpeed = ((*vi).speedMax * 0.5f32) as c_int;

        // Shifting Sounds
        if (*pVeh).m_iTurboTime < curTime
            && (*pVeh).m_iSoundDebounceTimer < curTime
            && ((nextSpeed > prevSpeed
                && nextSpeed > (halfMaxSpeed) as f32
                && prevSpeed < (halfMaxSpeed) as f32)
                || (nextSpeed > (halfMaxSpeed) as f32
                    && (*ctx.world).bg_state.rng.Q_irand(0, 1000) == 0))
        {
            let mut shiftSound = (*ctx.world).bg_state.rng.Q_irand(1, 4);
            shiftSound = match shiftSound {
                1 => (*vi).soundShift1,
                2 => (*vi).soundShift2,
                3 => (*vi).soundShift3,
                4 => (*vi).soundShift4,
                _ => shiftSound,
            };
            if shiftSound != 0 {
                (*pVeh).m_iSoundDebounceTimer =
                    curTime + (*ctx.world).bg_state.rng.Q_irand(1000, 4000);
                // MP: TODO MP Shift Sound Playback (no playback in MP)
            }
        }

        // Setup the move direction.
        if (*vi).r#type == vehicleType_t::VH_FIGHTER {
            AngleVectors(
                *((*pVeh).m_vOrientation as *const vec3_t),
                Some(&mut (*pclient).ps.moveDir),
                None,
                None,
            );
        } else {
            let vVehAngles: vec3_t = [0.0, *(*pVeh).m_vOrientation.add(YAW as usize), 0.0];
            AngleVectors(vVehAngles, Some(&mut (*pclient).ps.moveDir), None, None);
        }

        // MP surface destruction
        if (*vi).surfDestruction != 0 {
            if (*pVeh).m_iRemovedSurfaces != 0 {
                G_VehicleDamageBoxSizing(ctx, pVeh);
                // 3 seconds max on death.
                let dmg = (*pclient).ps.stats[STAT_MAX_HEALTH as usize] as f32
                    * (*pVeh).m_fTimeModifier
                    / 180.0f32;
                crate::g_combat::G_DamageFromKiller(
                    ctx,
                    parent,
                    parent,
                    parent,
                    (*pclient).ps.origin,
                    dmg as c_int,
                    DAMAGE_NO_SELF_PROTECTION
                        | DAMAGE_NO_HIT_LOC
                        | DAMAGE_NO_PROTECTION
                        | DAMAGE_NO_ARMOR,
                    MOD_SUICIDE as c_int,
                );
            }
            // make sure playerstate value stays in sync
            (*pclient).ps.vehSurfaces = (*pVeh).m_iRemovedSurfaces;
        }

        // MP: keep the PS value in sync
        if (*pVeh).m_iBoarding != 0 {
            (*pclient).ps.vehBoarding = qtrue;
        } else {
            (*pclient).ps.vehBoarding = qfalse;
        }

        // The pilot-enemy copy block is `#ifndef _JK2MP` (SP only) — skipped.

        qtrue
    }
}

/// Raven `UpdateRider`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:2338-2588`
pub fn UpdateRider(
    ctx: GameContext<'_>,
    pVeh: *mut Vehicle_t,
    pRider: *mut bgEntity_t,
    pUmcd: *mut usercmd_t,
) -> qboolean {
    unsafe {
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;

        if (*pVeh).m_iBoarding != 0 && (*pVeh).m_iDieTime == 0 {
            return qtrue;
        }

        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let rider = pRider as *mut gentity_t;
        let pc = (*parent).client as *mut gclient_t;
        let rc = (*rider).client as *mut gclient_t;

        // MP: so they know who we're locking onto with our rockets, if anyone
        if !rider.is_null()
            && !(*rider).client.is_null()
            && !parent.is_null()
            && !(*parent).client.is_null()
        {
            (*rc).ps.rocketLockIndex = (*pc).ps.rocketLockIndex;
            (*rc).ps.rocketLockTime = (*pc).ps.rocketLockTime;
            (*rc).ps.rocketTargetTime = (*pc).ps.rocketTargetTime;
        }

        // Regular exit.
        if ((*pUmcd).buttons & BUTTON_USE) != 0 && (*vi).r#type != vehicleType_t::VH_SPEEDER {
            if (*vi).r#type == vehicleType_t::VH_WALKER {
                // just get the fuck out
                (*pVeh).m_EjectDir = VEH_EJECT_REAR;
                if crate::veh_dispatch::eject(ctx, pVeh, pRider, qfalse) != qfalse {
                    return qfalse;
                }
            } else if ((*pVeh).m_ulFlags & (VEH_FLYING as u64)) == 0 {
                // If going too fast, roll off.
                if (*pc).ps.speed <= (600) as f32 && (*pUmcd).rightmove != 0 {
                    if crate::veh_dispatch::eject(ctx, pVeh, pRider, qfalse) != qfalse {
                        let iFlags =
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_HOLDLESS;
                        let iBlend = 300;
                        let Anim: c_int;
                        if (*pUmcd).rightmove > 0 {
                            Anim = animNumber_t::BOTH_ROLL_R as c_int;
                            (*pVeh).m_EjectDir = VEH_EJECT_RIGHT;
                        } else {
                            Anim = animNumber_t::BOTH_ROLL_L as c_int;
                            (*pVeh).m_EjectDir = VEH_EJECT_LEFT;
                        }
                        _VectorScale((*pc).ps.velocity, 0.25f32, &mut (*rc).ps.velocity);
                        Vehicle_SetAnim(ctx, rider, SETANIM_BOTH, Anim, iFlags, iBlend);
                        // just to make sure it's cleared when roll is done
                        (*rc).ps.weaponTime = (*rc).ps.torsoTimer - 200;
                        crate::g_utils::G_AddEvent(rider, EV_ROLL as c_int, 0);
                        return qfalse;
                    }
                } else {
                    let iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
                    let iBlend = 500;
                    let Anim: c_int;
                    if (*pUmcd).rightmove > 0 {
                        Anim = animNumber_t::BOTH_VS_DISMOUNT_R as c_int;
                        (*pVeh).m_EjectDir = VEH_EJECT_RIGHT;
                    } else {
                        Anim = animNumber_t::BOTH_VS_DISMOUNT_L as c_int;
                        (*pVeh).m_EjectDir = VEH_EJECT_LEFT;
                    }

                    if (*pVeh).m_iBoarding <= 1 {
                        // MP: iAnimLen = BG_AnimLength(rider->localAnimIndex, Anim).
                        // Reachable now that `ctx` threads the bg channel into this
                        // dispatch chain (game-tier free-function form off `bg_state`).
                        let iAnimLen: c_int = crate::bg_panimate::BG_AnimLength(
                            &(*ctx.world).bg_state,
                            (*rider).localAnimIndex,
                            Anim,
                        );
                        (*pVeh).m_iBoarding = (*ctx.world).level.time + iAnimLen;
                        // reuse flags: this should never be set in an entity
                        (*rider).flags |= FL_VEH_BOARDING; // MP
                                                           // Make sure they can't fire when leaving.
                        (*rc).ps.weaponTime = iAnimLen;
                    }

                    _VectorScale((*pc).ps.velocity, 0.25f32, &mut (*rc).ps.velocity);
                    Vehicle_SetAnim(ctx, rider, SETANIM_BOTH, Anim, iFlags, iBlend);
                }
            } else {
                // Flying, so just fall off.
                (*pVeh).m_EjectDir = VEH_EJECT_LEFT;
                if crate::veh_dispatch::eject(ctx, pVeh, pRider, qfalse) != qfalse {
                    return qfalse;
                }
            }
        }

        // Getting off animation complete (if we had one going)? (MP)
        if (*pVeh).m_iBoarding < (*ctx.world).level.time && ((*rider).flags & FL_VEH_BOARDING) != 0
        {
            (*rider).flags &= !FL_VEH_BOARDING;
            // Eject this guy now.
            if crate::veh_dispatch::eject(ctx, pVeh, pRider, qfalse) != qfalse {
                return qfalse;
            }
        }

        if (*vi).r#type != vehicleType_t::VH_FIGHTER && (*vi).r#type != vehicleType_t::VH_WALKER {
            // Jump off.
            if (*pUmcd).upmove > 0 {
                // The G_CanJumpToEnemyVeh / enemy-veh-boarding block is `#ifndef _JK2MP`
                // (SP only) — skipped.
                if crate::veh_dispatch::eject(ctx, pVeh, pRider, qfalse) != qfalse {
                    // Allow them to force jump off.
                    _VectorScale((*pc).ps.velocity, 0.5f32, &mut (*rc).ps.velocity);
                    (*rc).ps.velocity[2] += JUMP_VELOCITY;
                    (*rc).ps.fd.forceJumpZStart = (*rc).ps.origin[2]; // MP

                    if trap::ICARUS_TaskIDPending(
                        ctx.engine,
                        GIcarusTaskidpendingArgs::new(rider, TID_CHAN_VOICE as c_int),
                    ) == qfalse
                    {
                        crate::g_utils::G_AddEvent(rider, (EV_JUMP) as i32, 0);
                    }
                    Vehicle_SetAnim(
                        ctx,
                        rider,
                        SETANIM_BOTH,
                        animNumber_t::BOTH_JUMP1 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        300,
                    );
                    return qfalse;
                }
            }

            // Roll off.
            if (*pUmcd).upmove < 0 {
                let mut Anim: c_int = animNumber_t::BOTH_ROLL_B as c_int;
                (*pVeh).m_EjectDir = VEH_EJECT_REAR;
                if (*pUmcd).rightmove > 0 {
                    Anim = animNumber_t::BOTH_ROLL_R as c_int;
                    (*pVeh).m_EjectDir = VEH_EJECT_RIGHT;
                } else if (*pUmcd).rightmove < 0 {
                    Anim = animNumber_t::BOTH_ROLL_L as c_int;
                    (*pVeh).m_EjectDir = VEH_EJECT_LEFT;
                } else if (*pUmcd).forwardmove < 0 {
                    Anim = animNumber_t::BOTH_ROLL_B as c_int;
                    (*pVeh).m_EjectDir = VEH_EJECT_REAR;
                } else if (*pUmcd).forwardmove > 0 {
                    Anim = animNumber_t::BOTH_ROLL_F as c_int;
                    (*pVeh).m_EjectDir = VEH_EJECT_FRONT;
                }

                if crate::veh_dispatch::eject(ctx, pVeh, pRider, qfalse) != qfalse {
                    if ((*pVeh).m_ulFlags & (VEH_FLYING as u64)) == 0 {
                        _VectorScale((*pc).ps.velocity, 0.25f32, &mut (*rc).ps.velocity);
                        Vehicle_SetAnim(
                            ctx,
                            rider,
                            SETANIM_BOTH,
                            Anim,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_HOLDLESS,
                            300,
                        );
                        // just to make sure it's cleared when roll is done
                        (*rc).ps.weaponTime = (*rc).ps.torsoTimer - 200;
                        crate::g_utils::G_AddEvent(rider, EV_ROLL as c_int, 0);
                    }
                    return qfalse;
                }
            }
        }

        qtrue
    }
}

/// Raven `AttachRiders`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:2598-2731`
pub fn AttachRiders(ctx: GameContext<'_>, pVeh: *mut Vehicle_t) {
    unsafe {
        let mut i: c_int = 0;

        crate::bg_vehicleLoad::AttachRidersGeneric(
            pVeh,
            &(*ctx.world).bg_state,
            &GameBgTraps::new(ctx.engine),
            (*ctx.world).level.time,
        );

        if !(*pVeh).m_pPilot.is_null() {
            let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
            let pilot = (*pVeh).m_pPilot as *mut gentity_t;
            (*pilot).waypoint = (*parent).waypoint; // take the veh's waypoint as your own

            // assuming we updated him relative to the bolt in AttachRidersGeneric
            let pcl = (*pilot).client as *mut gclient_t;
            crate::g_utils::G_SetOrigin(pilot, (*pcl).ps.origin);
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(pilot));
        }

        if !(*pVeh).m_pOldPilot.is_null() {
            let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
            let oldpilot = (*pVeh).m_pOldPilot as *mut gentity_t;
            (*oldpilot).waypoint = (*parent).waypoint;

            let pcl = (*oldpilot).client as *mut gclient_t;
            crate::g_utils::G_SetOrigin(oldpilot, (*pcl).ps.origin);
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(oldpilot));
        }

        // attach passengers
        while i < (*pVeh).m_iNumPassengers {
            if !(*(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize)).is_null() {
                let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
                let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
                let pilot = *(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize) as *mut gentity_t;

                debug_assert!(!(*parent).ghoul2.is_null());
                let crotchBolt = trap::G2API_AddBolt(
                    ctx.engine,
                    GG2AddboltArgs::new((*parent).ghoul2 as *mut c_void, 0, cstr("*driver")),
                );
                debug_assert!(!(*parent).client.is_null());
                debug_assert!(!(*pilot).client.is_null());
                let ppcl = (*parent).client as *mut gclient_t;

                let yawOnlyAngles: vec3_t = [0.0, (*ppcl).ps.viewangles[YAW], 0.0];

                // Get the driver tag.
                trap::G2API_GetBoltMatrix(
                    ctx.engine,
                    GG2GetboltArgs::new(
                        (*parent).ghoul2 as *mut c_void,
                        0,
                        crotchBolt,
                        &mut boltMatrix as *mut mdxaBone_t,
                        &yawOnlyAngles as *const vec3_t,
                        &(*ppcl).ps.origin as *const vec3_t,
                        (*ctx.world).level.time,
                        core::ptr::null_mut(),
                        &(*parent).modelScale as *const vec3_t,
                    ),
                );
                let ppc = (*pilot).client as *mut gclient_t;
                BG_GiveMeVectorFromMatrix(
                    &boltMatrix,
                    Eorientations::ORIGIN as c_int,
                    &mut (*ppc).ps.origin,
                );

                crate::g_utils::G_SetOrigin(pilot, (*ppc).ps.origin);
                trap::LinkEntity(ctx.engine, GLinkentityArgs::new(pilot));
            }
            i += 1;
        }

        // attach droid
        if !(*pVeh).m_pDroidUnit.is_null() && (*pVeh).m_iDroidUnitTag != -1 {
            let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
            let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
            let droid = (*pVeh).m_pDroidUnit as *mut gentity_t;

            debug_assert!(!(*parent).ghoul2.is_null());
            debug_assert!(!(*parent).client.is_null());

            if !(*droid).client.is_null() {
                let dcl = (*droid).client as *mut gclient_t;
                let ppcl = (*parent).client as *mut gclient_t;
                let yawOnlyAngles: vec3_t = [0.0, (*ppcl).ps.viewangles[YAW], 0.0];

                // Get the droid tag.
                trap::G2API_GetBoltMatrix(
                    ctx.engine,
                    GG2GetboltArgs::new(
                        (*parent).ghoul2 as *mut c_void,
                        0,
                        (*pVeh).m_iDroidUnitTag,
                        &mut boltMatrix as *mut mdxaBone_t,
                        &yawOnlyAngles as *const vec3_t,
                        &(*parent).r.currentOrigin as *const vec3_t,
                        (*ctx.world).level.time,
                        core::ptr::null_mut(),
                        &(*parent).modelScale as *const vec3_t,
                    ),
                );
                let mut fwd: vec3_t = [0.0; 3];
                BG_GiveMeVectorFromMatrix(
                    &boltMatrix,
                    Eorientations::ORIGIN as c_int,
                    &mut (*dcl).ps.origin,
                );
                BG_GiveMeVectorFromMatrix(
                    &boltMatrix,
                    Eorientations::NEGATIVE_Y as c_int,
                    &mut fwd,
                );
                vectoangles(fwd, &mut (*dcl).ps.viewangles);

                crate::g_utils::G_SetOrigin(droid, (*dcl).ps.origin);
                crate::g_utils::G_SetAngles(droid, (*dcl).ps.viewangles);
                crate::g_client::SetClientViewAngle(droid, (*dcl).ps.viewangles);
                trap::LinkEntity(ctx.engine, GLinkentityArgs::new(droid));

                if !(*droid).NPC.is_null() {
                    crate::npc_c::NPC_SetAnim(
                        ctx,
                        droid,
                        SETANIM_BOTH,
                        animNumber_t::BOTH_STAND2 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    (*dcl).ps.legsTimer = 500;
                    (*dcl).ps.torsoTimer = 500;
                }
            }
        }
    }
}

/// Raven `Ghost` — make someone invisible and un-collidable.
///
/// Source: `oracle/codemp/game/g_vehicles.c:2734-2756`
pub fn Ghost(ctx: GameContext<'_>, pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) {
    unsafe {
        if pEnt.is_null() {
            return;
        }
        let ent = pEnt as *mut gentity_t;

        // This was introduced to prevent one extra entity from being sent to the clients.
        (*ent).r.svFlags |= SVF_NOCLIENT;

        (*ent).s.eFlags |= EF_NODRAW;
        if !(*ent).client.is_null() {
            let client = (*ent).client as *mut gclient_t;
            (*client).ps.eFlags |= EF_NODRAW;
        }
        (*ent).r.contents = 0;
    }
}

/// Raven `UnGhost` — make someone visible and collidable.
///
/// Source: `oracle/codemp/game/g_vehicles.c:2759-2781`
pub fn UnGhost(ctx: GameContext<'_>, pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) {
    unsafe {
        if pEnt.is_null() {
            return;
        }
        let ent = pEnt as *mut gentity_t;

        // make sure the client is sent again
        (*ent).r.svFlags &= !SVF_NOCLIENT;

        (*ent).s.eFlags &= !EF_NODRAW;
        if !(*ent).client.is_null() {
            let client = (*ent).client as *mut gclient_t;
            (*client).ps.eFlags &= !EF_NODRAW;
        }
        (*ent).r.contents = CONTENTS_BODY;
    }
}

/// Raven `G_VehicleDamageBoxSizing`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:2785-2840`
pub fn G_VehicleDamageBoxSizing(ctx: GameContext<'_>, pVeh: *mut Vehicle_t) {
    unsafe {
        let fDist = 256.0f32; // estimated distance to nose from origin
        let bDist = 256.0f32; // estimated distance to back from origin
        let wDist = 32.0f32; // width on each side from origin
        let hDist = 32.0f32; // height on each side from origin
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;

        if (*parent).ghoul2.is_null()
            || (*parent).m_pVehicle.is_null()
            || (*parent).client.is_null()
        {
            // shouldn't have gotten in here then
            return;
        }

        // only do anything if all wings are stripped off.
        if ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_C) == 0
            || ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_D) == 0
            || ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_E) == 0
            || ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_F) == 0
        {
            return;
        }

        // get directions based on orientation
        let mut fwd: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        let mut up: vec3_t = [0.0; 3];
        AngleVectors(
            *((*pVeh).m_vOrientation as *const vec3_t),
            Some(&mut fwd),
            Some(&mut right),
            Some(&mut up),
        );

        // nose == maxs, back == mins (relative to 0)
        let mut nose: vec3_t = [0.0; 3];
        let mut back: vec3_t = [0.0; 3];
        _VectorMA(vec3_origin, fDist, fwd, &mut nose);
        _VectorMA(vec3_origin, -bDist, fwd, &mut back);

        // move to opposite right/left
        let a = nose;
        _VectorMA(a, wDist, right, &mut nose);
        let a = nose;
        _VectorMA(a, -wDist, right, &mut back);

        // same for up/down
        let a = nose;
        _VectorMA(a, hDist, up, &mut nose);
        let a = nose;
        _VectorMA(a, -hDist, up, &mut back);

        // trace and see if our new mins/maxs are safe
        let pcl = (*parent).client as *mut gclient_t;
        let mut trace: trace_t = core::mem::zeroed();
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut trace as *mut trace_t,
                &(*pcl).ps.origin as *const vec3_t,
                &back as *const vec3_t,
                &nose as *const vec3_t,
                &(*pcl).ps.origin as *const vec3_t,
                (*parent).s.number,
                (*parent).clipmask,
            ),
        );
        if trace.allsolid == 0 && trace.startsolid == 0 && trace.fraction == 1.0 {
            // all clear!
            _VectorCopy(nose, &mut (*parent).r.maxs);
            _VectorCopy(back, &mut (*parent).r.mins);
        } else {
            // oh well, DIE!
            crate::g_combat::G_Damage(
                ctx,
                parent,
                parent,
                parent,
                None,
                (*pcl).ps.origin,
                9999,
                DAMAGE_NO_PROTECTION,
                MOD_SUICIDE as c_int,
            );
        }
    }
}

/// Raven `G_FlyVehicleImpactDir`.
///
/// PORT-NOTE(bg-boundary): the LAW signature is ctx-free (its caller
/// `G_FlyVehicleSurfaceDestruction` invokes it ctx-free), yet the body needs
/// `trap_Trace` (engine). The trap calls are transcribed against the game channel
/// `ctx`, which must be threaded in by the dispatch retrofit (see shape_mismatch).
///
/// Source: `oracle/codemp/game/g_vehicles.c:2843-2924`
pub fn G_FlyVehicleImpactDir(
    ctx: GameContext<'_>,
    veh: *mut gentity_t,
    trace: *mut trace_t,
) -> c_int {
    unsafe {
        let pVeh = (*veh).m_pVehicle as *mut Vehicle_t;
        if trace.is_null() || pVeh.is_null() || (*veh).client.is_null() {
            return -1;
        }
        let vcl = (*veh).client as *mut gclient_t;

        let mut fwd: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        AngleVectors((*vcl).ps.viewangles, Some(&mut fwd), Some(&mut right), None);
        let testMins: vec3_t = [-24.0, -24.0, -24.0];
        let testMaxs: vec3_t = [24.0, 24.0, 24.0];

        // do a trace to determine if the nose is clear
        let mut fPos: vec3_t = [0.0; 3];
        _VectorMA((*vcl).ps.origin, 256.0f32, fwd, &mut fPos);
        let mut localTrace: trace_t = core::mem::zeroed();
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut localTrace as *mut trace_t,
                &(*vcl).ps.origin as *const vec3_t,
                &testMins as *const vec3_t,
                &testMaxs as *const vec3_t,
                &fPos as *const vec3_t,
                (*veh).s.number,
                (*veh).clipmask,
            ),
        );
        let mut noseClear = qfalse;
        if localTrace.startsolid == 0 && localTrace.allsolid == 0 && localTrace.fraction == 1.0 {
            noseClear = qtrue;
        }

        if noseClear != qfalse {
            // if nose is clear check for tearing the wings off
            let mut rWing: vec3_t = [0.0; 3];
            let mut lWing: vec3_t = [0.0; 3];
            _VectorMA((*vcl).ps.origin, 128.0f32, right, &mut rWing);
            _VectorMA((*vcl).ps.origin, -128.0f32, right, &mut lWing);

            // test the right wing - unless it's already removed
            if ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_E) == 0
                || ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_F) == 0
            {
                _VectorMA(rWing, 256.0f32, fwd, &mut fPos);
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut localTrace as *mut trace_t,
                        &rWing as *const vec3_t,
                        &testMins as *const vec3_t,
                        &testMaxs as *const vec3_t,
                        &fPos as *const vec3_t,
                        (*veh).s.number,
                        (*veh).clipmask,
                    ),
                );
                if localTrace.startsolid != 0
                    || localTrace.allsolid != 0
                    || localTrace.fraction != 1.0
                {
                    return SHIPSURF_RIGHT;
                }
            }

            // test the left wing - unless it's already removed
            if ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_C) == 0
                || ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_D) == 0
            {
                _VectorMA(lWing, 256.0f32, fwd, &mut fPos);
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut localTrace as *mut trace_t,
                        &lWing as *const vec3_t,
                        &testMins as *const vec3_t,
                        &testMaxs as *const vec3_t,
                        &fPos as *const vec3_t,
                        (*veh).s.number,
                        (*veh).clipmask,
                    ),
                );
                if localTrace.startsolid != 0
                    || localTrace.allsolid != 0
                    || localTrace.fraction != 1.0
                {
                    return SHIPSURF_LEFT;
                }
            }
        }

        // try to use the trace plane normal
        let impactAngle = crate::bg_misc::vectoyaw((*trace).plane.normal);
        let relativeAngle = AngleSubtract(impactAngle, (*vcl).ps.viewangles[YAW]);

        if relativeAngle > 130.0 || relativeAngle < -130.0 {
            // consider this front
            return SHIPSURF_FRONT;
        } else if relativeAngle > 0.0 {
            return SHIPSURF_RIGHT;
        } else if relativeAngle < 0.0 {
            return SHIPSURF_LEFT;
        }

        SHIPSURF_BACK
    }
}

/// Raven `G_ShipSurfaceForSurfName` — map a surface name to its ship surface id.
///
/// Source: `oracle/codemp/game/g_vehicles.c:2930-2959`
pub fn G_ShipSurfaceForSurfName(surfaceName: *const c_char) -> c_int {
    unsafe {
        if surfaceName.is_null() {
            return -1;
        }
        if Q_strncmp(c"nose".as_ptr(), surfaceName, 4) == 0
            || Q_strncmp(c"f_gear".as_ptr(), surfaceName, 6) == 0
            || Q_strncmp(c"glass".as_ptr(), surfaceName, 5) == 0
        {
            return SHIPSURF_FRONT;
        }
        if Q_strncmp(c"body".as_ptr(), surfaceName, 4) == 0 {
            return SHIPSURF_BACK;
        }
        if Q_strncmp(c"r_wing1".as_ptr(), surfaceName, 7) == 0
            || Q_strncmp(c"r_wing2".as_ptr(), surfaceName, 7) == 0
            || Q_strncmp(c"r_gear".as_ptr(), surfaceName, 6) == 0
        {
            return SHIPSURF_RIGHT;
        }
        if Q_strncmp(c"l_wing1".as_ptr(), surfaceName, 7) == 0
            || Q_strncmp(c"l_wing2".as_ptr(), surfaceName, 7) == 0
            || Q_strncmp(c"l_gear".as_ptr(), surfaceName, 6) == 0
        {
            return SHIPSURF_LEFT;
        }
        -1
    }
}

/// Raven `G_SetVehDamageFlags`.
///
/// PORT-NOTE(bg-boundary): the LAW signature is ctx-free. Only the
/// destroyed-droid sub-branch (`damageLevel==3`, `SHIPSURF_BACK`) needs the world
/// (to resolve `veh->enemy: Option<EntityId>` to a `*mut gentity_t` for G_Damage);
/// that resolution is transcribed against the game channel `ctx`, threaded in by
/// the dispatch retrofit (see shape_mismatch). The bit flag bulk is faithful.
///
/// Source: `oracle/codemp/game/g_vehicles.c:2961-3039`
pub fn G_SetVehDamageFlags(
    ctx: GameContext<'_>,
    veh: *mut gentity_t,
    shipSurf: c_int,
    damageLevel: c_int,
) {
    unsafe {
        let vcl = (*veh).client as *mut gclient_t;
        match damageLevel {
            3 => {
                // destroyed — add both flags so cgame knows this surf is GONE
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_HEAVY + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs |= 1 << dmgFlag;
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_LIGHT + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs |= 1 << dmgFlag;
                // copy down
                (*veh).s.brokenLimbs = (*vcl).ps.brokenLimbs;
                // check droid
                if shipSurf == SHIPSURF_BACK {
                    // destroy the droid if we have one
                    let vp = (*veh).m_pVehicle as *mut Vehicle_t;
                    if !(*veh).m_pVehicle.is_null() && !(*vp).m_pDroidUnit.is_null() {
                        let droidEnt = (*vp).m_pDroidUnit as *mut gentity_t;
                        if !droidEnt.is_null()
                            && (((*droidEnt).flags & FL_UNDYING) != 0 || (*droidEnt).health > 0)
                        {
                            // make it vulnerable, then blow it up
                            (*droidEnt).flags &= !FL_UNDYING;
                            // resolve veh->enemy (Option<EntityId>) to a raw ptr
                            let enemy_ptr = match (*veh).enemy {
                                Some(id) => (*ctx.world).g_entities.as_mut_ptr().add(id.0 as usize),
                                None => core::ptr::null_mut(),
                            };
                            // PORT-NOTE(G_Damage-null-dir/point): Raven passes NULL for both
                            // `dir` and `point`; resolved sig takes `dir: &mut vec3_t` and
                            // `point: vec3_t`, so zero vecs stand in.
                            let null_point: vec3_t = [0.0; 3];
                            crate::g_combat::G_Damage(
                                ctx,
                                droidEnt,
                                enemy_ptr,
                                enemy_ptr,
                                None,
                                null_point,
                                99999,
                                0,
                                MOD_UNKNOWN as c_int,
                            );
                        }
                    }
                }
            }
            2 => {
                // heavy only
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_HEAVY + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs |= 1 << dmgFlag;
                // remove light
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_LIGHT + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs &= !(1 << dmgFlag);
                // copy down
                (*veh).s.brokenLimbs = (*vcl).ps.brokenLimbs;
                // check droid — make it vulnerable if we have one
                if shipSurf == SHIPSURF_BACK {
                    let vp = (*veh).m_pVehicle as *mut Vehicle_t;
                    if !(*veh).m_pVehicle.is_null() && !(*vp).m_pDroidUnit.is_null() {
                        let droidEnt = (*vp).m_pDroidUnit as *mut gentity_t;
                        if !droidEnt.is_null() && ((*droidEnt).flags & FL_UNDYING) != 0 {
                            (*droidEnt).flags &= !FL_UNDYING;
                        }
                    }
                }
            }
            1 => {
                // light only
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_LIGHT + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs |= 1 << dmgFlag;
                // remove heavy
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_HEAVY + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs &= !(1 << dmgFlag);
                // copy down
                (*veh).s.brokenLimbs = (*vcl).ps.brokenLimbs;
            }
            _ => {
                // no damage (case 0 / default)
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_HEAVY + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs &= !(1 << dmgFlag);
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_LIGHT + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs &= !(1 << dmgFlag);
                (*veh).s.brokenLimbs = (*vcl).ps.brokenLimbs;
            }
        }
    }
}

/// Raven `G_VehicleSetDamageLocFlags`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:3041-3100`
pub fn G_VehicleSetDamageLocFlags(
    ctx: GameContext<'_>,
    veh: *mut gentity_t,
    impactDir: c_int,
    deathPoint: c_int,
) {
    unsafe {
        if (*veh).client.is_null() {
            return;
        }
        // Raven shadows the `deathPoint` parameter with a local of the same name.
        let vp = (*veh).m_pVehicle as *mut Vehicle_t;
        let vi = (*vp).m_pVehicleInfo as *mut vehicleInfo_t;

        let deathPoint: c_int;
        if impactDir == SHIPSURF_FRONT {
            deathPoint = (*vi).health_front;
        } else if impactDir == SHIPSURF_BACK {
            deathPoint = (*vi).health_back;
        } else if impactDir == SHIPSURF_RIGHT {
            deathPoint = (*vi).health_right;
        } else if impactDir == SHIPSURF_LEFT {
            deathPoint = (*vi).health_left;
        } else {
            return;
        }

        let heavyDamagePoint: c_int;
        let lightDamagePoint: c_int;
        if !(*veh).m_pVehicle.is_null()
            && !(*vp).m_pVehicleInfo.is_null()
            && (*vi).malfunctionArmorLevel != 0
            && (*vi).armor != 0
        {
            let mut perc = (*vi).malfunctionArmorLevel as f32 / (*vi).armor as f32;
            if perc > 0.99 {
                perc = 0.99;
            }
            lightDamagePoint = (deathPoint as f32 * perc * 0.25).ceil() as c_int;
            heavyDamagePoint = (deathPoint as f32 * perc).ceil() as c_int;
        } else {
            heavyDamagePoint = (deathPoint as f32 * 0.66).ceil() as c_int;
            lightDamagePoint = (deathPoint as f32 * 0.14).ceil() as c_int;
        }

        if (*veh).locationDamage[impactDir as usize] >= deathPoint {
            // destroyed
            G_SetVehDamageFlags(ctx, veh, impactDir, 3);
        } else if (*veh).locationDamage[impactDir as usize] <= lightDamagePoint {
            // light only
            G_SetVehDamageFlags(ctx, veh, impactDir, 1);
        } else if (*veh).locationDamage[impactDir as usize] <= heavyDamagePoint {
            // heavy only
            G_SetVehDamageFlags(ctx, veh, impactDir, 2);
        }
    }
}

/// Raven `G_FlyVehicleDestroySurface`.
///
/// PORT-NOTE(bg-boundary): the LAW signature is ctx-free (caller
/// `G_FlyVehicleSurfaceDestruction` invokes it ctx-free), yet the body reads
/// `level.time` and calls ctx-requiring fns (`NPC_SetSurfaceOnOff`, `G_RadiusDamage`,
/// `G_EntitySound`). Those are transcribed against the game channel `ctx`, threaded
/// in by the dispatch retrofit (see shape_mismatch).
///
/// Source: `oracle/codemp/game/g_vehicles.c:3102-3188`
pub fn G_FlyVehicleDestroySurface(
    ctx: GameContext<'_>,
    veh: *mut gentity_t,
    surface: c_int,
) -> qboolean {
    unsafe {
        let mut surfName: [*const c_char; 4] = [core::ptr::null(); 4]; // up to 4 surfs at once
        let mut numSurfs: c_int = 0;
        let mut smashedBits: c_int = 0;

        if surface == -1 {
            // not valid?
            return qfalse;
        }

        match surface {
            SHIPSURF_FRONT => {
                // break the nose off
                surfName[0] = c"nose".as_ptr();
                smashedBits = SHIPSURF_BROKEN_G;
                numSurfs = 1;
            }
            SHIPSURF_BACK => {
                // break both the bottom wings off for a backward impact
                surfName[0] = c"r_wing2".as_ptr();
                surfName[1] = c"l_wing2".as_ptr();
                // get rid of the landing gear
                surfName[2] = c"r_gear".as_ptr();
                surfName[3] = c"l_gear".as_ptr();
                smashedBits =
                    SHIPSURF_BROKEN_A | SHIPSURF_BROKEN_B | SHIPSURF_BROKEN_D | SHIPSURF_BROKEN_F;
                numSurfs = 4;
            }
            SHIPSURF_RIGHT => {
                // break both right wings off
                surfName[0] = c"r_wing1".as_ptr();
                surfName[1] = c"r_wing2".as_ptr();
                // get rid of the landing gear
                surfName[2] = c"r_gear".as_ptr();
                smashedBits = SHIPSURF_BROKEN_B | SHIPSURF_BROKEN_E | SHIPSURF_BROKEN_F;
                numSurfs = 3;
            }
            SHIPSURF_LEFT => {
                // break both left wings off
                surfName[0] = c"l_wing1".as_ptr();
                surfName[1] = c"l_wing2".as_ptr();
                // get rid of the landing gear
                surfName[2] = c"l_gear".as_ptr();
                smashedBits = SHIPSURF_BROKEN_A | SHIPSURF_BROKEN_C | SHIPSURF_BROKEN_D;
                numSurfs = 3;
            }
            _ => {}
        }

        if numSurfs < 1 {
            // didn't get any valid surfs..
            return qfalse;
        }

        while numSurfs > 0 {
            numSurfs -= 1;
            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, veh, surfName[numSurfs as usize], TURN_OFF);
        }

        let vp = (*veh).m_pVehicle as *mut Vehicle_t;
        if (*vp).m_iRemovedSurfaces == 0 {
            // first time something got blown off
            if !(*vp).m_pPilot.is_null() {
                // make the pilot scream to his death
                crate::g_utils::G_EntitySound(
                    ctx,
                    (*vp).m_pPilot as *mut gentity_t,
                    CHAN_VOICE,
                    G_SoundIndex(c"*falling1.wav".as_ptr()),
                );
            }
        }
        // so we can check what's broken
        (*vp).m_iRemovedSurfaces |= smashedBits;

        // do some explosive damage, but don't damage this ship with it
        let vcl = (*veh).client as *mut gclient_t;
        crate::g_combat::G_RadiusDamage(
            ctx,
            (*vcl).ps.origin,
            veh,
            100.0,
            500.0,
            veh,
            core::ptr::null_mut(),
            MOD_VEH_EXPLOSION as c_int,
        );

        // when spiraling to your death, do the electical shader
        (*vcl).ps.electrifyTime = (*ctx.world).level.time + 10000;

        qtrue
    }
}

/// Raven `G_FlyVehicleSurfaceDestruction`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:3190-3259`
pub fn G_FlyVehicleSurfaceDestruction(
    ctx: GameContext<'_>,
    veh: *mut gentity_t,
    trace: *mut trace_t,
    magnitude: c_int,
    force: qboolean,
) {
    unsafe {
        if (*veh).ghoul2.is_null() || (*veh).m_pVehicle.is_null() {
            // no g2 instance.. or no vehicle instance
            return;
        }

        let vp = (*veh).m_pVehicle as *mut Vehicle_t;
        let vi = (*vp).m_pVehicleInfo as *mut vehicleInfo_t;

        let mut impactDir = G_FlyVehicleImpactDir(ctx, veh, trace);
        let mut alreadyRebroken = qfalse;
        // Raven declares `deathPoint = -1` before the `anotherImpact` label; the
        // goto-loop keeps the prior value when a `default` impactDir leaves it
        // unset, so it is declared outside the rewritten loop.
        let mut deathPoint: c_int = -1;

        // Raven: `anotherImpact:` goto-loop.
        loop {
            if impactDir == -1 {
                // not valid?
                return;
            }

            (*veh).locationDamage[impactDir as usize] += magnitude * 7;

            if impactDir == SHIPSURF_FRONT {
                deathPoint = (*vi).health_front;
            } else if impactDir == SHIPSURF_BACK {
                deathPoint = (*vi).health_back;
            } else if impactDir == SHIPSURF_RIGHT {
                deathPoint = (*vi).health_right;
            } else if impactDir == SHIPSURF_LEFT {
                deathPoint = (*vi).health_left;
            }

            if deathPoint != -1 {
                // got a valid health value
                if force != qfalse && (*veh).locationDamage[impactDir as usize] < deathPoint {
                    // force that surf to be destroyed
                    (*veh).locationDamage[impactDir as usize] = deathPoint;
                }
                if (*veh).locationDamage[impactDir as usize] >= deathPoint {
                    // do it
                    if G_FlyVehicleDestroySurface(ctx, veh, impactDir) != qfalse {
                        // actually took off a surface
                        G_VehicleSetDamageLocFlags(ctx, veh, impactDir, deathPoint);
                    }
                } else {
                    G_VehicleSetDamageLocFlags(ctx, veh, impactDir, deathPoint);
                }
            }

            if alreadyRebroken == qfalse {
                let secondImpact = G_FlyVehicleImpactDir(ctx, veh, trace);
                if impactDir != secondImpact {
                    // can break off another piece in this same impact.. but only
                    // break off up to 2 at once
                    alreadyRebroken = qtrue;
                    impactDir = secondImpact;
                    continue;
                }
            }
            break;
        }
    }
}

/// Raven `G_VehUpdateShields`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:3261-3273`
pub fn G_VehUpdateShields(targ: *mut gentity_t) {
    unsafe {
        if targ.is_null() || (*targ).client.is_null() || (*targ).m_pVehicle.is_null() {
            return;
        }
        let vp = (*targ).m_pVehicle as *mut Vehicle_t;
        if (*vp).m_pVehicleInfo.is_null() {
            return;
        }
        let vi = (*vp).m_pVehicleInfo as *mut vehicleInfo_t;
        if (*vi).shields <= 0 {
            // doesn't have shields, so don't have to send it
            return;
        }
        let client = (*targ).client as *mut gclient_t;
        (*client).ps.activeForcePass =
            (((*vp).m_iShields as f32 / (*vi).shields as f32) * 10.0).floor() as c_int;
    }
}

/// Raven `SetParent` — set the parent entity of this Vehicle NPC.
///
/// Source: `oracle/codemp/game/g_vehicles.c:3277-3277`
pub fn SetParent(ctx: GameContext<'_>, pVeh: *mut Vehicle_t, pParentEntity: *mut bgEntity_t) {
    unsafe {
        (*pVeh).m_pParentEntity = pParentEntity as *mut mp_bg::public::bg_entity::bgEntity_t;
    }
}

/// Raven `SetPilot` — add a pilot to the vehicle.
///
/// Source: `oracle/codemp/game/g_vehicles.c:3280-3280`
pub fn SetPilot(ctx: GameContext<'_>, pVeh: *mut Vehicle_t, pPilot: *mut bgEntity_t) {
    unsafe {
        (*pVeh).m_pPilot = pPilot as *mut mp_bg::public::bg_entity::bgEntity_t;
    }
}

/// Raven `AddPassenger` — add a passenger to the vehicle (false if we're full).
///
/// Raven: the generic implementation always returns false.
/// Source: `oracle/codemp/game/g_vehicles.c:3283-3283`
pub fn AddPassenger(pVeh: *mut Vehicle_t) -> qboolean {
    qfalse
}

/// Raven `Inhabited` — whether this vehicle is currently inhabited (by anyone).
///
/// Source: `oracle/codemp/game/g_vehicles.c:3286-3286`
pub fn Inhabited(ctx: GameContext<'_>, pVeh: *mut Vehicle_t) -> qboolean {
    unsafe {
        if !(*pVeh).m_pPilot.is_null() || (*pVeh).m_iNumPassengers != 0 {
            qtrue
        } else {
            qfalse
        }
    }
}

/// Raven `Eject`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:1019-1376`
pub fn Eject(
    ctx: GameContext<'_>,
    pVeh: *mut Vehicle_t,
    pEnt: *mut bgEntity_t,
    forceEject: qboolean,
) -> qboolean {
    unsafe {
        let ent = pEnt as *mut gentity_t;
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;

        let mut taintedRider = qfalse;
        let mut deadRider = qfalse;

        if pEnt == (*pVeh).m_pDroidUnit as *mut bgEntity_t {
            G_EjectDroidUnit(ctx, pVeh, qfalse);
            return qtrue;
        }

        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;

        if !ent.is_null() {
            if (*ent).inuse == qfalse
                || (*ent).client.is_null()
                || (*((*ent).client as *mut gclient_t)).pers.connected != CON_CONNECTED
            {
                // MP: if someone disconnects on us, we still have to clear our owner
                // — jump straight to the ownership-cleanup section (`getItOutOfMe`).
                taintedRider = qtrue;
            } else if (*ent).health < 1 {
                deadRider = qtrue;
            }
        }

        // The `!taintedRider` guard models Raven's `goto getItOutOfMe`: the tainted
        // path skips validation, the eject-direction search, and the reposition.
        if taintedRider == qfalse {
            // Validate.
            if ent.is_null() {
                return qfalse;
            }
            if forceEject == qfalse {
                if !((*pVeh).m_iBoarding == 0
                    || (*pVeh).m_iBoarding == -999
                    || ((*pVeh).m_iBoarding < -3 && (*pVeh).m_iBoarding >= -9))
                {
                    // MP: I don't care, if he's dead get him off even if he died
                    // while boarding.
                    deadRider = qtrue;
                    (*pVeh).m_iBoarding = 0;
                    (*pVeh).m_bWasBoarding = qfalse;
                }
            }

            // Try ejecting in every direction.
            if (*pVeh).m_EjectDir < VEH_EJECT_LEFT {
                (*pVeh).m_EjectDir = VEH_EJECT_LEFT;
            } else if (*pVeh).m_EjectDir > VEH_EJECT_BOTTOM {
                (*pVeh).m_EjectDir = VEH_EJECT_BOTTOM;
            }
            let firstEjectDir = (*pVeh).m_EjectDir;
            let mut vExitPos: vec3_t = [0.0; 3];
            while VEH_TryEject(ctx, pVeh, parent, ent, (*pVeh).m_EjectDir, &mut vExitPos) == qfalse
            {
                (*pVeh).m_EjectDir += 1;
                if (*pVeh).m_EjectDir > VEH_EJECT_BOTTOM {
                    (*pVeh).m_EjectDir = VEH_EJECT_LEFT;
                }
                if (*pVeh).m_EjectDir == firstEjectDir {
                    // they all failed
                    if deadRider == qfalse {
                        // if he's dead.. just shove him in solid, who cares.
                        return qfalse;
                    }
                    if forceEject != qfalse {
                        // we want to always get out, just eject him here
                        _VectorCopy((*ent).r.currentOrigin, &mut vExitPos);
                        break;
                    } else {
                        // can't eject
                        return qfalse;
                    }
                }
            }

            // Move them to the exit position.
            G_SetOrigin(ent, vExitPos);
            (*((*ent).client as *mut gclient_t)).ps.origin = (*ent).r.currentOrigin;
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));

            // If it's the player, stop overrides. (MP: the override-clear body is
            // `#ifndef _JK2MP` — nothing to do here.)
            if (*ent).s.number < MAX_CLIENTS as c_int {}
        }

        // getItOutOfMe:

        // If he's the pilot...
        if (*pVeh).m_pPilot == (ent as *mut mp_bg::public::bg_entity::bgEntity_t) {
            let pc = (*parent).client as *mut gclient_t;

            (*pVeh).m_pPilot = core::ptr::null_mut();
            (*parent).r.ownerNum = ENTITYNUM_NONE;
            (*parent).s.owner = (*parent).r.ownerNum; // for prediction

            // keep these current angles
            (*pc).pers.cmd = core::mem::zeroed();
            (*pVeh).m_ucmd = core::mem::zeroed();

            // if there are some passengers, promote the first passenger to pilot
            let mut j: c_int = 0;
            while j < (*pVeh).m_iNumPassengers {
                if !(*(*pVeh).m_ppPassengers.as_mut_ptr().add(j as usize)).is_null() {
                    let mut k: c_int = 1;
                    crate::veh_dispatch::set_pilot(
                        ctx,
                        pVeh,
                        *(*pVeh).m_ppPassengers.as_mut_ptr().add(j as usize) as *mut gentity_t,
                    );
                    let newPilot =
                        *(*pVeh).m_ppPassengers.as_mut_ptr().add(j as usize) as *mut gentity_t;
                    (*parent).r.ownerNum = (*newPilot).s.number;
                    (*parent).s.owner = (*parent).r.ownerNum; // for prediction
                    (*pc).ps.m_iVehicleNum = (*newPilot).s.number + 1;

                    // rearrange the passenger slots now..
                    // QAGAME: server just needs to tell client he's not a passenger anymore
                    if !(*newPilot).client.is_null() {
                        (*((*newPilot).client as *mut gclient_t)).ps.generic1 = 0;
                    }
                    *(*pVeh).m_ppPassengers.as_mut_ptr().add(j as usize) = core::ptr::null_mut();
                    while k < (*pVeh).m_iNumPassengers {
                        if (*(*pVeh).m_ppPassengers.as_mut_ptr().add((k - 1) as usize)).is_null() {
                            // move down
                            *(*pVeh).m_ppPassengers.as_mut_ptr().add((k - 1) as usize) =
                                *(*pVeh).m_ppPassengers.as_mut_ptr().add(k as usize);
                            *(*pVeh).m_ppPassengers.as_mut_ptr().add(k as usize) =
                                core::ptr::null_mut();
                            // QAGAME: server just needs to tell client which passenger he is
                            let moved = *(*pVeh).m_ppPassengers.as_mut_ptr().add((k - 1) as usize)
                                as *mut gentity_t;
                            if !(*moved).client.is_null() {
                                (*((*moved).client as *mut gclient_t)).ps.generic1 = k;
                            }
                        }
                        k += 1;
                    }
                    (*pVeh).m_iNumPassengers -= 1;

                    break;
                }
                j += 1;
            }
        } else if ent == ((*pVeh).m_pOldPilot as *mut gentity_t) {
            (*pVeh).m_pOldPilot = core::ptr::null_mut();
        } else {
            // Look for this guy in the passenger list.
            let mut i: c_int = 0;
            while i < (*vi).maxPassengers {
                let psngr = *(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize) as *mut gentity_t;
                // If we found him...
                if psngr == ent {
                    // QAGAME: server just needs to tell client he's not a passenger anymore
                    if !(*psngr).client.is_null() {
                        (*((*psngr).client as *mut gclient_t)).ps.generic1 = 0;
                    }
                    *(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize) = core::ptr::null_mut();
                    (*pVeh).m_iNumPassengers -= 1;
                    break;
                }
                i += 1;
            }

            // Didn't find him, can't eject because they aren't in the vehicle (hopefully)!
            if i == (*vi).maxPassengers {
                return qfalse;
            }
        }

        // MP: I hate adding these!
        if taintedRider == qfalse {
            if (*vi).hideRider != 0 {
                crate::veh_dispatch::un_ghost(ctx, pVeh, ent as *mut bgEntity_t);
            }
        }

        // If the vehicle now has no pilot...
        if (*pVeh).m_pPilot.is_null() {
            let pc = (*parent).client as *mut gclient_t;
            (*parent).s.loopSound = 0;
            (*pc).ps.loopSound = 0;
            // Completely empty vehicle...?
            if (*pVeh).m_iNumPassengers == 0 {
                (*pc).ps.m_iVehicleNum = 0;
            }
        }

        if taintedRider != qfalse {
            // you can go now
            (*pVeh).m_iBoarding = (*ctx.world).level.time + 1000;
            return qtrue;
        }

        // Client not in a vehicle. (MP)
        let ec = (*ent).client as *mut gclient_t;
        (*ec).ps.m_iVehicleNum = 0;
        (*ent).r.ownerNum = ENTITYNUM_NONE;
        (*ent).s.owner = (*ent).r.ownerNum; // for prediction

        (*ec).ps.viewangles[PITCH as usize] = 0.0;
        (*ec).ps.viewangles[ROLL as usize] = 0.0;
        (*ec).ps.viewangles[YAW as usize] = *(*pVeh).m_vOrientation.add(YAW as usize);
        crate::g_client::SetClientViewAngle(ent, (*ec).ps.viewangles);

        if (*ec).solidHack != 0 {
            (*ec).solidHack = 0;
            (*ent).r.contents = CONTENTS_BODY;
        }
        (*ent).s.m_iVehicleNum = 0;

        // The jump-out velocity, SP facing block, and the weapon-switch on-hop-off
        // logic are all `#ifndef _JK2MP` or commented-out in Raven — MP does nothing
        // in the weapon `if/else` here.

        crate::bg_panimate::BG_SetLegsAnimTimer(&mut (*ec).ps, 0);
        crate::bg_panimate::BG_SetTorsoAnimTimer(&mut (*ec).ps, 0);

        // Set how long until this vehicle can be boarded again.
        (*pVeh).m_iBoarding = (*ctx.world).level.time + 1000;

        qtrue
    }
}

/// Raven `DeathUpdate`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:1485-1617`
pub fn DeathUpdate(ctx: GameContext<'_>, pVeh: *mut Vehicle_t) {
    unsafe {
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;

        if (*ctx.world).level.time >= (*pVeh).m_iDieTime {
            // If the vehicle is not empty.
            if crate::veh_dispatch::inhabited(ctx, pVeh) != qfalse {
                // MP: the SP-only `noRagTime` clear is `#ifndef _JK2MP`.

                crate::veh_dispatch::eject_all(ctx, pVeh);
                if crate::veh_dispatch::inhabited(ctx, pVeh) != qfalse {
                    // if we've still got people in us, just kill the bastards
                    let pc = (*parent).client as *mut gclient_t;
                    if !(*pVeh).m_pPilot.is_null() {
                        //FIXME: does this give proper credit to the enemy who shot you down?
                        crate::g_combat::G_Damage(
                            ctx,
                            (*pVeh).m_pPilot as *mut gentity_t,
                            parent,
                            parent,
                            None,
                            (*pc).ps.origin,
                            999,
                            DAMAGE_NO_PROTECTION,
                            MOD_SUICIDE as c_int, // #define MOD_EXPLOSIVE MOD_SUICIDE
                        );
                    }
                    if (*pVeh).m_iNumPassengers != 0 {
                        let mut i: c_int = 0;
                        while i < (*vi).maxPassengers {
                            if !(*(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize)).is_null() {
                                //FIXME: does this give proper credit to the enemy who shot you down?
                                crate::g_combat::G_Damage(
                                    ctx,
                                    *(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize)
                                        as *mut gentity_t,
                                    parent,
                                    parent,
                                    None,
                                    (*pc).ps.origin,
                                    999,
                                    DAMAGE_NO_PROTECTION,
                                    MOD_SUICIDE as c_int, // #define MOD_EXPLOSIVE MOD_SUICIDE
                                );
                            }
                            i += 1;
                        }
                    }
                }
            }

            if crate::veh_dispatch::inhabited(ctx, pVeh) == qfalse {
                // explode now as long as we managed to kick everyone out
                let mut lMins: vec3_t = [0.0; 3];
                let mut lMaxs: vec3_t = [0.0; 3];
                let mut bottom: vec3_t = [0.0; 3];
                let mut trace: trace_t = core::mem::zeroed();

                // MP: the "Kill All Client Side Looping Effects" teardown is
                // `#ifndef _JK2MP` (SP only).

                if (*vi).iExplodeFX != 0 {
                    let mut fxAng: vec3_t = [-90.0, 0.0, 0.0];
                    crate::g_utils::G_PlayEffectID(
                        (*vi).iExplodeFX,
                        (*parent).r.currentOrigin,
                        fxAng,
                    );
                    // trace down and place mark
                    _VectorCopy((*parent).r.currentOrigin, &mut bottom);
                    bottom[2] -= 80.0;
                    G_VehicleTrace(
                        ctx,
                        &mut trace,
                        (*parent).r.currentOrigin,
                        vec3_origin,
                        vec3_origin,
                        bottom,
                        (*parent).s.number,
                        CONTENTS_SOLID,
                    );
                    if trace.fraction < 1.0 {
                        _VectorCopy(trace.endpos, &mut bottom);
                        bottom[2] += 2.0;
                        fxAng = [-90.0, 0.0, 0.0];
                        crate::g_utils::G_PlayEffectID(
                            crate::g_utils::G_EffectIndex(c"ships/ship_explosion_mark".as_ptr()),
                            trace.endpos,
                            fxAng,
                        );
                    }
                }

                (*parent).takedamage = qfalse; // so we don't recursively damage ourselves
                if (*vi).explosionRadius > 0.0 && (*vi).explosionDamage > 0 {
                    _VectorCopy((*parent).r.mins, &mut lMins);
                    lMins[2] = -4.0; // to keep it off the ground a *little*
                    _VectorCopy((*parent).r.maxs, &mut lMaxs);
                    _VectorCopy((*parent).r.currentOrigin, &mut bottom);
                    bottom[2] += (*parent).r.mins[2] - 32.0;
                    G_VehicleTrace(
                        ctx,
                        &mut trace,
                        (*parent).r.currentOrigin,
                        lMins,
                        lMaxs,
                        bottom,
                        (*parent).s.number,
                        CONTENTS_SOLID,
                    );
                    //FIXME: extern damage and radius or base on fuel
                    crate::g_combat::G_RadiusDamage(
                        ctx,
                        trace.endpos,
                        parent,
                        (*vi).explosionDamage as f32,
                        (*vi).explosionRadius,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        MOD_VEH_EXPLOSION as c_int,
                    );
                }

                (*parent).think = Some(EntThink::G_FreeEntity).into();
                (*parent).nextthink = (*ctx.world).level.time + FRAMETIME;
            }
        }
        // MP: the `else` "let everyone around me know I'm gonna blow" danger-sound
        // block is `#ifndef _JK2MP` (SP only) — omitted.
    }
}

// Source: oracle/codemp/game/g_vehicles.c:1618-1620
/// Raven `RegisterAssets` — register all the assets used by this vehicle. The
/// base implementation is an empty function body in Raven (see cite); this
/// stub matches that faithfully rather than panicking.
pub unsafe extern "C" fn RegisterAssets(pVeh: *mut Vehicle_t) {}

// 2026-07-03: `G_SetSharedVehicleFunctions` retired — it only assigned the now-removed
// `vehicleInfo_t` fn-ptr slots. Vehicle dispatch is `vehicleType_t`-keyed in
// `crate::veh_dispatch`. Source: see per-class setter in the oracle .c.
