// PORT-STATUS: g_vehicles.c — pass-3 blind fill: all 14 remaining fns bodied
// against the resolved (LAW) signatures. Boundary-set fns (Vehicle_SetAnim,
// Update, G_FlyVehicleImpactDir, G_SetVehDamageFlags, G_FlyVehicleDestroySurface)
// carry no ctx/bg channel in their fixed vtable/fn-ptr slot signatures yet reach
// world/engine/rng — those references are transcribed against the game channel
// (`ctx`) pending the vtable-dispatch retrofit.
//! FAITHFUL port of `oracle/codemp/game/g_vehicles.c` (MP `_JK2MP` +
//! `QAGAME` compile path).
//!
//! Filled by the jampgame mega-pass.
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
//!
//! Safe-state migration **Stage 2c**: `gentity_t*` params are `EntityId` /
//! `Option<EntityId>` handles (§B5); ctx-free leaf helpers take `&mut gentity_t`.
//! Entity fields are reached through `ctx.world.entity(id)` / `entity_mut(id)` at
//! the point of use (no fn-top raw re-derives). `Vehicle_t*`/`bgEntity_t*` params
//! and the vehicle fn-pointer tables are NOT entity handles and stay raw (§D12
//! seam), as do the BG_Alloc pool clients (`.client`/`playerState`, recipe 2b) and
//! `gNPC_t` (`NPC`); those derefs remain in tight `unsafe` blocks through copied
//! pointer values. Entity pointers reached via a seam field (e.g.
//! `pVeh->m_pParentEntity`) are resolved to a handle with `ctx.entity_id_of(ptr)`
//! at the seam deref, then accessed through the accessor.
#![allow(non_snake_case, unused, clippy::all)]

use crate::bg_channel::GameBgTraps;
use crate::g_utils::G_SoundIndex;
use crate::prelude::*;
use crate::g_utils::G_EffectIndex;
use crate::q_math::{
    _DotProduct, _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, vectoangles,
    AngleSubtract, AngleVectors, VectorNormalize,
};
use crate::q_shared::Q_strncmp;
use crate::trap;
use crate::NPC_spawn::NPC_Spawn_Do;
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
    ctx: &mut GameContext,
    ent: EntityId,
    setAnimParts: c_int,
    anim: c_int,
    setAnimFlags: c_int,
    iBlend: c_int,
) {
    // `ent` is an `EntityId`; entity fields go through the accessor.
    // FLAG: `.client` is a BG_Alloc pool client on vehicle/NPC entities
    // (recipe 2b); the client deref stays raw.
    let client = ctx.world.entity(ent).client;
    // Raven: assert(ent->client);
    debug_assert!(!client.is_null());
    // MP `_JK2MP` path:
    //   BG_SetAnim(&client->ps, bgAllAnims[ent->localAnimIndex].anims,
    //              setAnimParts, anim, setAnimFlags, iBlend)
    // `BG_SetAnim` is a `PmoveContext` method (`bgAllAnims` off `BgState`);
    // build a pm-null per-call context from `ctx`, matching the `G_SetAnim`
    // game-tier wrapper precedent (`g_utils.rs`).
    let idx = ctx.world.entity(ent).localAnimIndex as usize;
    let anims = ctx.world.bg_state.bgAllAnims[idx].anims;
    unsafe {
        let ps = &mut (*client).ps as *mut playerState_t;
        let traps = crate::bg_channel::GameBgTraps::new(ctx.engine);
        let mut callbacks = crate::bg_channel::GameCallbacksImpl {
            // STAGE-2b: irreducible — `GameCallbacksImpl.world` is a `*mut GameWorld` bg-seam field; a raw store is required.
            world: ctx.world_raw(),
            engine: ctx.engine,
        };
        let mut pmc =
            crate::bg_channel::PmoveContext::new(&mut ctx.world.bg_state, &traps, &mut callbacks);
        pmc.BG_SetAnim(ps, anims, setAnimParts, anim, setAnimFlags, iBlend);
    }
    // FLAG: pool-client read (recipe 2b) stays raw.
    let legs = unsafe { (*client).ps.legsAnim };
    ctx.world.entity_mut(ent).s.legsAnim = legs;
}

/// Raven `G_VehicleTrace`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:102-109`
pub fn G_VehicleTrace(
    ctx: &mut GameContext,
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
pub fn G_IsRidingVehicle(ctx: &mut GameContext, pEnt: Option<EntityId>) -> *mut Vehicle_t {
    if let Some(id) = pEnt {
        // FLAG: `.client` is a pool client on vehicle/NPC entities (recipe 2b);
        // the deref stays raw.
        let client = ctx.world.entity(id).client;
        if !client.is_null() {
            let npc_class = unsafe { (*client).NPC_class };
            if npc_class != CLASS_VEHICLE && ctx.world.entity(id).s.m_iVehicleNum != 0 {
                let vehNum = ctx.world.entity(id).s.m_iVehicleNum as usize;
                return ctx.world.g_entities[vehNum].m_pVehicle;
            }
        }
    }
    core::ptr::null_mut()
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
pub fn G_VehicleSpawn(ctx: &mut GameContext, self_: EntityId) {
    // `self_` is an `EntityId`; entity fields go through the accessor.
    let cur = ctx.world.entity(self_).r.currentOrigin;
    ctx.world.entity_mut(self_).s.origin = cur;
    let self_ptr = ctx.world.entity_mut(self_) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(self_ptr.cast()));

    if ctx.world.entity(self_).count == 0 {
        ctx.world.entity_mut(self_).count = 1;
    }

    // save this because self gets removed in next func
    let yaw = ctx.world.entity(self_).s.angles[YAW];

    let vehEnt = NPC_Spawn_Do(ctx, self_);
    if vehEnt.is_null() {
        return; // return NULL;
    }
    let vehEnt_id = ctx.entity_id_of(vehEnt).unwrap();

    ctx.world.entity_mut(vehEnt_id).s.angles[YAW] = yaw;
    // FLAG: Vehicle_t / vehicleInfo_t / gNPC_t are seam types (recipe §D12 /
    // 2c) — the field is read through the accessor, the deref stays raw.
    let vp = ctx.world.entity(vehEnt_id).m_pVehicle;
    unsafe {
        let vi = (*vp).m_pVehicleInfo as *mut vehicleInfo_t;
        if (*vi).r#type != vehicleType_t::VH_ANIMAL {
            let npc = ctx.world.entity(vehEnt_id).NPC;
            (*npc).behaviorState = bState_t::BS_CINEMATIC;
        }
    }

    // special check in case someone disconnects/dies while boarding
    if ctx.world.entity(vehEnt_id).spawnflags & 1 != 0 {
        // die without pilot
        if ctx.world.entity(vehEnt_id).damage == 0 {
            // default 10 sec
            ctx.world.entity_mut(vehEnt_id).damage = 10000;
        }
        if ctx.world.entity(vehEnt_id).speed == 0.0 {
            // default 512 units
            ctx.world.entity_mut(vehEnt_id).speed = 512.0;
        }
        let t = ctx.world.level.time + ctx.world.entity(vehEnt_id).damage;
        // FLAG: Vehicle_t seam deref (recipe §D12) stays raw.
        unsafe {
            (*vp).m_iPilotTime = t;
        }
    }
}

/// Raven `G_AttachToVehicle`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:247-289`
pub fn G_AttachToVehicle(ctx: &mut GameContext, pEnt: Option<EntityId>, ucmd: *mut *mut usercmd_t) {
    if ucmd.is_null() {
        return;
    }
    let Some(ent_id) = pEnt else {
        return;
    };

    // MP: vehEnt = &g_entities[ent->r.ownerNum];
    let veh_id = EntityId(ctx.world.entity(ent_id).r.ownerNum as u32);
    let veh_waypoint = ctx.world.entity(veh_id).waypoint;
    ctx.world.entity_mut(ent_id).waypoint = veh_waypoint; // take the veh's waypoint as your own

    // FLAG: Vehicle_t / ghoul2 / pool-client are seam values (recipe §D12 / 2b);
    // the field is read through the accessor, the deref stays raw.
    let vp = ctx.world.entity(veh_id).m_pVehicle;
    if vp.is_null() {
        return;
    }
    let veh_ghoul2 = ctx.world.entity(veh_id).ghoul2 as *mut c_void;
    let crotchBolt = trap::G2API_AddBolt(ctx.engine, veh_ghoul2, 0, "*driver");

    // Get the driver tag.
    let entClient = ctx.world.entity(ent_id).client;
    let veh_origin = ctx.world.entity(veh_id).r.currentOrigin;
    let veh_scale = ctx.world.entity(veh_id).modelScale;
    let level_time = ctx.world.level.time;
    unsafe {
        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
        trap::G2API_GetBoltMatrix(
            ctx.engine,
            GG2GetboltArgs::new(
                veh_ghoul2,
                0,
                crotchBolt,
                &mut boltMatrix as *mut mdxaBone_t,
                (*vp).m_vOrientation as *const vec3_t,
                &veh_origin as *const vec3_t,
                level_time,
                core::ptr::null_mut(),
                &veh_scale as *const vec3_t,
            ),
        );
        BG_GiveMeVectorFromMatrix(
            &boltMatrix,
            Eorientations::ORIGIN as c_int,
            &mut (*entClient).ps.origin,
        );
        let new_origin = (*entClient).ps.origin;
        crate::g_utils::G_SetOrigin(ctx.world.entity_mut(ent_id), new_origin);
    }
    let ent_ptr = ctx.world.entity_mut(ent_id) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));
}

/// Raven `Animate` — animate the vehicle and its riders.
///
/// Source: `oracle/codemp/game/g_vehicles.c:481-493`
pub fn Animate(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
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
    ctx: &mut GameContext,
    pVeh: *mut Vehicle_t,
    pEnt: *mut bgEntity_t,
) -> qboolean {
    unsafe {
        // Determine where the entity is entering the vehicle from (left, right, or back).
        // FLAG: Vehicle_t / vehicleInfo_t / pool-client seam derefs stay raw; the
        // parent/ent entity fields go through the accessor.
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let parent_id = ctx.entity_id_of(parent).unwrap();
        let ent = pEnt as *mut gentity_t;
        let ent_id = ctx.entity_id_of(ent).unwrap();
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
                let cl = ctx.world.entity(ent_id).client;
                let parent_number = ctx.world.entity(parent_id).s.number;
                if cl.is_null() || (*cl).ps.groundEntityNum != parent_number {
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
        let vVehAngles: vec3_t = [0.0, ctx.world.entity(parent_id).r.currentAngles[YAW], 0.0];

        // Vector from Entity to Vehicle.
        let mut vVehToEnt: vec3_t = [0.0; 3];
        let ent_origin = ctx.world.entity(ent_id).r.currentOrigin;
        let parent_origin = ctx.world.entity(parent_id).r.currentOrigin;
        _VectorSubtract(ent_origin, parent_origin, &mut vVehToEnt);
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
pub fn Board(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> qboolean {
    unsafe {
        // FLAG: Vehicle_t / vehicleInfo_t / pool-client seam derefs stay raw; the
        // ent/parent entity fields go through the accessor.
        let ent = pEnt as *mut gentity_t;
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;

        // If it's not a valid entity, OR the vehicle is dead, OR we're already
        // being boarded, OR the person trying to get on is already in a vehicle.
        if ent.is_null() {
            return qfalse;
        }
        let ent_id = ctx.entity_id_of(ent).unwrap();
        let parent_id = ctx.entity_id_of(parent).unwrap();
        let entClient = ctx.world.entity(ent_id).client;
        if ctx.world.entity(parent_id).health <= 0
            || (*pVeh).m_iBoarding > 0
            || (*entClient).ps.m_iVehicleNum != 0
        {
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
        if ctx.world.entity(ent_id).s.number < MAX_CLIENTS as c_int {
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
                        if !entClient.is_null() {
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
            let pn = ctx.world.entity(parent_id).s.number;
            ctx.world.entity_mut(ent_id).s.m_iVehicleNum = pn;
            if !entClient.is_null() {
                let vn = ctx.world.entity(ent_id).s.m_iVehicleNum;
                (*entClient).ps.m_iVehicleNum = vn;
            }
            if (*pVeh).m_pPilot == (ent as *mut mp_bg::public::bg_entity::bgEntity_t) {
                let en = ctx.world.entity(ent_id).s.number;
                ctx.world.entity_mut(parent_id).r.ownerNum = en;
                let on = ctx.world.entity(parent_id).r.ownerNum;
                ctx.world.entity_mut(parent_id).s.owner = on; // for prediction
            }

            // QAGAME: undock if we were being suspended.
            {
                if (ctx.world.entity(parent_id).spawnflags & 2) != 0 {
                    // was being suspended
                    ctx.world.entity_mut(parent_id).spawnflags &= !2;
                    crate::g_utils::G_Sound(
                        ctx,
                        Some(parent_id),
                        CHAN_AUTO,
                        G_SoundIndex("sound/vehicles/common/release.wav"),
                    );
                    let debounce = ctx.world.entity(parent_id).fly_sound_debounce_time;
                    if debounce != 0 {
                        // we should drop like a rock for a few seconds
                        (*pVeh).m_iDropTime = ctx.world.level.time + debounce;
                    }
                }
            }

            // Set the looping sound only when there is a pilot (vehicle is "on").
            if (*vi).soundLoop != 0 {
                let pc = ctx.world.entity(parent_id).client;
                let sl = (*vi).soundLoop;
                ctx.world.entity_mut(parent_id).s.loopSound = sl;
                (*pc).ps.loopSound = sl;
            }
        } else {
            // If there's no pilot, try to drive this vehicle.
            if (*pVeh).m_pPilot.is_null() {
                crate::veh_dispatch::set_pilot(ctx, pVeh, ent as *mut bgEntity_t);
                // TODO: Set pilot should do all this stuff....
                let en = ctx.world.entity(ent_id).s.number;
                ctx.world.entity_mut(parent_id).r.ownerNum = en;
                let on = ctx.world.entity(parent_id).r.ownerNum;
                ctx.world.entity_mut(parent_id).s.owner = on; // for prediction

                // Set the looping sound only when there is a pilot.
                if (*vi).soundLoop != 0 {
                    let pc = ctx.world.entity(parent_id).client;
                    let sl = (*vi).soundLoop;
                    ctx.world.entity_mut(parent_id).s.loopSound = sl;
                    (*pc).ps.loopSound = sl;
                }

                let pc = ctx.world.entity(parent_id).client;
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
                        if !entClient.is_null() {
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
        let pn = ctx.world.entity(parent_id).s.number;
        (*entClient).ps.m_iVehicleNum = pn;
        ctx.world.entity_mut(ent_id).r.ownerNum = pn;
        let eo = ctx.world.entity(ent_id).r.ownerNum;
        ctx.world.entity_mut(ent_id).s.owner = eo; // for prediction
        if (*pVeh).m_pPilot == (ent as *mut mp_bg::public::bg_entity::bgEntity_t) {
            let pc = ctx.world.entity(parent_id).client;
            // always gonna be under MAX_CLIENTS so no worries about 1 byte overflow
            let en1 = ctx.world.entity(ent_id).s.number + 1;
            (*pc).ps.m_iVehicleNum = en1;
        }

        // numHands==2 switch-to-vehicle-weapon body is `#ifndef _JK2MP` (SP only):
        // MP does nothing here.

        if (*vi).hideRider != 0 {
            // hide the rider
            crate::veh_dispatch::ghost(ctx, pVeh, ent as *mut bgEntity_t);
        }

        // Play the start sounds.
        if (*vi).soundOn != 0 {
            let son = (*vi).soundOn;
            crate::g_utils::G_Sound(ctx, Some(parent_id), CHAN_AUTO, son);
        }

        let mut vPlayerDir: vec3_t = [0.0; 3];
        _VectorCopy(*((*pVeh).m_vOrientation as *const vec3_t), &mut vPlayerDir);
        vPlayerDir[ROLL] = 0.0;
        crate::g_client::SetClientViewAngle(ctx.world.entity_mut(ent_id), vPlayerDir);

        qtrue
    }
}

/// Raven `VEH_TryEject`.
///
/// `vExitPos` is Raven's out-param exit position (written through, never
/// NULL at any oracle caller) → `&mut vec3_t`.
/// Source: `oracle/codemp/game/g_vehicles.c:874-987`
pub fn VEH_TryEject(
    ctx: &mut GameContext,
    pVeh: *mut Vehicle_t,
    parent: EntityId,
    ent: EntityId,
    ejectDir: c_int,
    vExitPos: &mut vec3_t,
) -> qboolean {
    // `parent`/`ent` are `EntityId`s; entity fields go through the accessor.
    // FLAG: Vehicle_t / vehicleInfo_t seam deref (recipe §D12) stays raw.
    let vi = unsafe { (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t };

    // Make sure that the entity is not 'stuck' inside the vehicle (since their
    // bboxes will now intersect). Leave the vehicle from the right side.
    let vVehAngles: vec3_t = [0.0, ctx.world.entity(parent).r.currentAngles[YAW], 0.0];
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
    if unsafe { (*vi).r#type } == vehicleType_t::VH_WALKER {
        // hacktastic!
        fBias += 0.2;
    }
    let ent_origin = ctx.world.entity(ent).r.currentOrigin;
    _VectorCopy(ent_origin, vExitPos);
    let pmaxs = ctx.world.entity(parent).r.maxs;
    let fVehDiag = (pmaxs[0] * pmaxs[0] + pmaxs[1] * pmaxs[1]).sqrt();
    let mut vEntMaxs: vec3_t = ctx.world.entity(ent).r.maxs;
    if ctx.world.entity(ent).s.number < MAX_CLIENTS as c_int {
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
    let oldOwner = ctx.world.entity(ent).r.ownerNum;
    ctx.world.entity_mut(ent).r.ownerNum = ENTITYNUM_NONE;
    let ent_origin2 = ctx.world.entity(ent).r.currentOrigin;
    let ent_number = ctx.world.entity(ent).s.number;
    let ent_clipmask = ctx.world.entity(ent).clipmask;
    let mut m_ExitTrace: trace_t = unsafe { core::mem::zeroed() };
    G_VehicleTrace(
        ctx,
        &mut m_ExitTrace,
        ent_origin2,
        vEntMins,
        vEntMaxs2,
        *vExitPos,
        ent_number,
        ent_clipmask,
    );
    ctx.world.entity_mut(ent).r.ownerNum = oldOwner;

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

/// Raven `G_EjectDroidUnit`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:989-1016`
pub fn G_EjectDroidUnit(ctx: &mut GameContext, pVeh: *mut Vehicle_t, kill: qboolean) {
    // FLAG: `m_pDroidUnit` is a Vehicle_t seam pointer (recipe §D12); read the
    // raw value, resolve to an id, reach entity fields through the accessor.
    let droid = unsafe { (*pVeh).m_pDroidUnit as *mut gentity_t };
    let droid_id = ctx.entity_id_of(droid).unwrap();
    ctx.world.entity_mut(droid_id).s.m_iVehicleNum = ENTITYNUM_NONE;
    ctx.world.entity_mut(droid_id).s.owner = ENTITYNUM_NONE; // MP

    // QAGAME
    ctx.world.entity_mut(droid_id).flags &= !FL_UNDYING;
    ctx.world.entity_mut(droid_id).r.ownerNum = ENTITYNUM_NONE;
    let dc = ctx.world.entity(droid_id).client;
    if !dc.is_null() {
        // FLAG: pool client (recipe 2b) — deref stays raw.
        unsafe {
            (*dc).ps.m_iVehicleNum = ENTITYNUM_NONE;
        }
    }
    if kill != qfalse {
        // Kill them, too.
        let num = ctx.world.entity(droid_id).s.number;
        crate::g_utils::G_MuteSound(ctx, num, CHAN_VOICE);
        let origin = ctx.world.entity(droid_id).s.origin;
        // Raven passes NULL for `dir`; carried as `None`.
        crate::g_combat::G_Damage(
            ctx,
            Some(droid_id),
            ctx.entity_id_of(core::ptr::null_mut()),
            ctx.entity_id_of(core::ptr::null_mut()),
            None,
            origin,
            10000,
            0,
            MOD_SUICIDE as c_int,
        );
    }

    // FLAG: Vehicle_t seam write (recipe §D12) stays raw.
    unsafe {
        (*pVeh).m_pDroidUnit = core::ptr::null_mut();
    }
}

/// Raven `EjectAll`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:1377-1448`
pub fn EjectAll(ctx: &mut GameContext, pVeh: *mut Vehicle_t) -> qboolean {
    unsafe {
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;

        // TODO: Setup a default escape for every vehicle type.
        (*pVeh).m_EjectDir = VEH_EJECT_TOP;
        // Make sure no other boarding calls exist. We MUST exit.
        (*pVeh).m_iBoarding = 0;
        (*pVeh).m_bWasBoarding = qfalse;

        // Throw them off. (FLAG: Vehicle_t / vehicleInfo_t seam derefs stay raw;
        // rider entity fields go through the accessor.)
        if !(*pVeh).m_pPilot.is_null() {
            let pilot = (*pVeh).m_pPilot as *mut gentity_t;
            crate::veh_dispatch::eject(ctx, pVeh, (*pVeh).m_pPilot as *mut bgEntity_t, qtrue);
            if (*vi).killRiderOnDeath != qfalse && !pilot.is_null() {
                let pilot_id = ctx.entity_id_of(pilot).unwrap();
                let num = ctx.world.entity(pilot_id).s.number;
                crate::g_utils::G_MuteSound(ctx, num, CHAN_VOICE);
                let origin = ctx.world.entity(pilot_id).s.origin;
                crate::g_combat::G_Damage(
                    ctx,
                    Some(pilot_id),
                    None,
                    None,
                    None,
                    origin,
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
                let pilot_id = ctx.entity_id_of(pilot).unwrap();
                let num = ctx.world.entity(pilot_id).s.number;
                crate::g_utils::G_MuteSound(ctx, num, CHAN_VOICE);
                let origin = ctx.world.entity(pilot_id).s.origin;
                crate::g_combat::G_Damage(
                    ctx,
                    Some(pilot_id),
                    None,
                    None,
                    None,
                    origin,
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
                        let rider_id = ctx.entity_id_of(rider).unwrap();
                        let num = ctx.world.entity(rider_id).s.number;
                        crate::g_utils::G_MuteSound(ctx, num, CHAN_VOICE);
                        let origin = ctx.world.entity(rider_id).s.origin;
                        crate::g_combat::G_Damage(
                            ctx,
                            Some(rider_id),
                            None,
                            None,
                            None,
                            origin,
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
pub fn StartDeathDelay(ctx: &mut GameContext, pVeh: *mut Vehicle_t, iDelayTimeOverride: c_int) {
    // FLAG: Vehicle_t / vehicleInfo_t are seam pointers (recipe §D12); read the
    // raw values, reach the parent entity through the accessor.
    let (parent, vi) = unsafe {
        (
            (*pVeh).m_pParentEntity as *mut gentity_t,
            (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t,
        )
    };
    let parent_id = ctx.entity_id_of(parent).unwrap();
    let level_time = ctx.world.level.time;

    unsafe {
        if iDelayTimeOverride != 0 {
            (*pVeh).m_iDieTime = level_time + iDelayTimeOverride;
        } else {
            (*pVeh).m_iDieTime = level_time + (*vi).explosionDelay;
        }
    }

    if unsafe { (*vi).flammable } != qfalse {
        let snd = G_SoundIndex("sound/vehicles/common/fire_lp.wav");
        let client = ctx.world.entity(parent_id).client;
        ctx.world.entity_mut(parent_id).s.loopSound = snd;
        // FLAG: pool client (recipe 2b) — deref stays raw.
        unsafe {
            (*client).ps.loopSound = snd;
        }
    }
}

/// Raven `Initialize`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:1626-1757`
pub fn Initialize(ctx: &mut GameContext, pVeh: *mut Vehicle_t) -> qboolean {
    unsafe {
        // FLAG: Vehicle_t / vehicleInfo_t / pool-client / gNPC_t seam derefs stay
        // raw; the parent entity fields go through the accessor.
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;

        if parent.is_null() || (*parent).client.is_null() {
            return qfalse;
        }
        let parent_id = ctx.entity_id_of(parent).unwrap();
        let pc = ctx.world.entity(parent_id).client;

        (*pc).ps.m_iVehicleNum = 0; // MP
        ctx.world.entity_mut(parent_id).s.m_iVehicleNum = 0;
        {
            (*pVeh).m_iArmor = (*vi).armor;
            let hp = (*pVeh).m_iArmor;
            let npc = ctx.world.entity(parent_id).NPC;
            (*pc).ps.stats[STAT_HEALTH as usize] = hp;
            ctx.world.entity_mut(parent_id).health = hp;
            (*npc).stats.health = hp;
            (*pc).ps.stats[STAT_MAX_HEALTH as usize] = hp;
            (*pc).pers.maxHealth = hp;
            (*pVeh).m_iShields = (*vi).shields;
            G_VehUpdateShields(ctx.world.entity_mut(parent_id)); // MP
            (*pc).ps.stats[STAT_ARMOR as usize] = (*pVeh).m_iShields;
        }
        ctx.world.entity_mut(parent_id).mass = ((*vi).mass) as f32;

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
        let ang = ctx.world.entity(parent_id).s.angles[YAW];
        *(*pVeh).m_vOrientation.add(YAW as usize) = ang;

        // MP gravity
        if (*vi).gravity != 0 && (*vi).gravity as f32 != ctx.world.cvars.g_gravity.value {
            // not normal gravity
            let npc = ctx.world.entity(parent_id).NPC;
            if !npc.is_null() {
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
            let idx = ctx.world.entity(parent_id).localAnimIndex as usize;
            let anims = ctx.world.bg_state.bgAllAnims[idx].anims;
            let traps = GameBgTraps::new(ctx.engine);
            let mut callbacks = crate::bg_channel::GameCallbacksImpl {
                // STAGE-2b: irreducible — `GameCallbacksImpl.world` is a `*mut GameWorld` bg-seam field; a raw store is required.
                world: ctx.world_raw(),
                engine: ctx.engine,
            };
            let mut pmc = crate::bg_channel::PmoveContext::new(
                &mut ctx.world.bg_state,
                &traps,
                &mut callbacks,
            );
            pmc.BG_SetAnim(
                ps,
                anims,
                SETANIM_BOTH,
                BOTH_VS_IDLE as c_int,
                iFlags,
                iBlend,
            );
        }

        qtrue
    }
}

/// Raven `Update`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:1763-2334`
pub fn Update(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pUmcd: *const usercmd_t) -> qboolean {
    unsafe {
        // FLAG: Vehicle_t / vehicleInfo_t / pool-client seam derefs stay raw; the
        // parent/pilot entity fields go through the accessor.
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let parent_id = ctx.entity_id_of(parent).unwrap();
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;
        let pclient = ctx.world.entity(parent_id).client;
        // MP: parentPS = pVeh->m_pParentEntity->playerState (== &parent->client->ps)
        let parentPS = &mut (*pclient).ps as *mut playerState_t;

        // QAGAME: curTime = level.time
        let curTime = ctx.world.level.time;

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
                if !parent.is_null() && !pclient.is_null() {
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
                if !parent.is_null() && !pclient.is_null() {
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
            G_VehUpdateShields(ctx.world.entity_mut(parent_id)); // MP
        }

        // MP: sometimes owner gets out of whack
        if !parent.is_null()
            && ctx.world.entity(parent_id).r.ownerNum != ctx.world.entity(parent_id).s.owner
        {
            let own = ctx.world.entity(parent_id).r.ownerNum;
            ctx.world.entity_mut(parent_id).s.owner = own;
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
            let orient = *((*pVeh).m_vOrientation as *const vec3_t);
            SetClientViewAngle(ctx.world.entity_mut(parent_id), orient);
            if !(*pVeh).m_pPilot.is_null() {
                let pilot = (*pVeh).m_pPilot as *mut gentity_t;
                let pilot_id = ctx.entity_id_of(pilot).unwrap();
                let orient2 = *((*pVeh).m_vOrientation as *const vec3_t);
                SetClientViewAngle(ctx.world.entity_mut(pilot_id), orient2);
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
        else if ctx.world.entity(parent_id).health <= 0 {
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
        if ctx.world.entity(parent_id).spawnflags & 1 != 0 {
            if !(*pVeh).m_pPilot.is_null() || (*pVeh).m_bHasHadPilot == qfalse {
                if !(*pVeh).m_pPilot.is_null() && (*pVeh).m_bHasHadPilot == qfalse {
                    (*pVeh).m_bHasHadPilot = qtrue;
                    let pilot = (*pVeh).m_pPilot as *mut gentity_t;
                    let pilot_id = ctx.entity_id_of(pilot).unwrap();
                    (*pVeh).m_iPilotLastIndex = ctx.world.entity(pilot_id).s.number;
                }
                (*pVeh).m_iPilotTime = ctx.world.level.time + ctx.world.entity(parent_id).damage;
            } else if (*pVeh).m_iPilotTime != 0 {
                // die
                let oldPilot_id = EntityId((*pVeh).m_iPilotLastIndex as u32);
                let old_client = ctx.world.entity(oldPilot_id).client;
                let oldPilotConnected =
                    !old_client.is_null() && (*old_client).pers.connected == CON_CONNECTED;
                if ctx.world.entity(oldPilot_id).inuse == qfalse
                    || old_client.is_null()
                    || !oldPilotConnected
                {
                    // no longer in the game?
                    crate::g_combat::G_Damage(
                        ctx,
                        Some(parent_id),
                        Some(parent_id),
                        Some(parent_id),
                        None,
                        (*pclient).ps.origin,
                        99999,
                        DAMAGE_NO_PROTECTION,
                        MOD_SUICIDE as c_int,
                    );
                } else {
                    let oc = old_client;
                    let mut v: vec3_t = [0.0; 3];
                    _VectorSubtract((*pclient).ps.origin, (*oc).ps.origin, &mut v);
                    if VectorLength(v) < ctx.world.entity(parent_id).speed {
                        // still within the minimum distance to their vehicle
                        (*pVeh).m_iPilotTime =
                            ctx.world.level.time + ctx.world.entity(parent_id).damage;
                    } else if (*pVeh).m_iPilotTime < ctx.world.level.time {
                        // dying time
                        crate::g_combat::G_Damage(
                            ctx,
                            Some(parent_id),
                            Some(parent_id),
                            Some(parent_id),
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
                let pilotEnt_id = ctx.entity_id_of(pilotEnt).unwrap();
                let pec = ctx.world.entity(pilotEnt_id).client;
                if ctx.world.entity(pilotEnt_id).inuse == qfalse
                    || pec.is_null()
                    || ctx.world.entity(pilotEnt_id).health <= 0
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
            if (*pVeh).m_iBoarding > -1 && (*pVeh).m_iBoarding <= ctx.world.level.time {
                (*pVeh).m_bWasBoarding = qfalse;
                (*pVeh).m_iBoarding = 0;
            } else {
                boarding_maintain = true;
            }
        }

        if !boarding_maintain {
            let parent = (*pVeh).m_pParentEntity as *mut gentity_t;

            // Validate vehicle.
            if parent.is_null() || pclient.is_null() || ctx.world.entity(parent_id).health <= 0 {
                return qfalse;
            }

            // See if any of the riders are dead and if so kick em off.
            if !(*pVeh).m_pPilot.is_null() {
                let pilotEnt = (*pVeh).m_pPilot as *mut gentity_t;
                let pilotEnt_id = ctx.entity_id_of(pilotEnt).unwrap();
                let pec = ctx.world.entity(pilotEnt_id).client;
                if ctx.world.entity(pilotEnt_id).inuse == qfalse
                    || pec.is_null()
                    || ctx.world.entity(pilotEnt_id).health <= 0
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
                        let psngr_id = ctx.entity_id_of(psngr).unwrap();
                        let sc = ctx.world.entity(psngr_id).client;
                        if ctx.world.entity(psngr_id).inuse == qfalse
                            || sc.is_null()
                            || ctx.world.entity(psngr_id).health <= 0
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
                    // Raven's PM_BGEntForNum(parent->s.owner) lookup here is pure
                    // (pilotPS is never read) and is elided.
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
                crate::g_vehicleTurret::VEH_TurretThink(ctx, pVeh, parent_id, i);
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
        let orient = *((*pVeh).m_vOrientation as *const vec3_t);
        SetClientViewAngle(ctx.world.entity_mut(parent_id), orient);
        if !(*pVeh).m_pPilot.is_null() {
            // MP
            let pilotPS = (*(*pVeh).m_pPilot).playerState;
            if mp_bg::bg_pmove::BG_UnrestrainedPitchRoll(pilotPS, pVeh, &ctx.world.bg_state)
                == qfalse
            {
                let mut newVAngle: vec3_t = [0.0; 3];
                newVAngle[PITCH] = (*pilotPS).viewangles[PITCH];
                newVAngle[YAW] = (*pilotPS).viewangles[YAW];
                newVAngle[ROLL] = *(*pVeh).m_vOrientation.add(ROLL as usize);
                let pilot = (*pVeh).m_pPilot as *mut gentity_t;
                let pilot_id = ctx.entity_id_of(pilot).unwrap();
                SetClientViewAngle(ctx.world.entity_mut(pilot_id), newVAngle);
            }
        }

        // Process the move commands.
        // Oracle declares `int prevSpeed`/`int nextSpeed` — truncating the float
        // `speed` to int — and gates the shift sound on integer compares; fractional
        // speeds must not flip the term or the Q_irand draw desyncs.
        // Source: g_vehicles.c:1770-1771,2245-2247
        let prevSpeed = (*parentPS).speed as c_int;
        crate::veh_dispatch::process_move_commands(ctx, pVeh);
        let nextSpeed = (*parentPS).speed as c_int;
        let halfMaxSpeed = ((*vi).speedMax * 0.5f32) as c_int;

        // Shifting Sounds
        if (*pVeh).m_iTurboTime < curTime
            && (*pVeh).m_iSoundDebounceTimer < curTime
            && ((nextSpeed > prevSpeed && nextSpeed > halfMaxSpeed && prevSpeed < halfMaxSpeed)
                || (nextSpeed > halfMaxSpeed && ctx.world.bg_state.rng.Q_irand(0, 1000) == 0))
        {
            let mut shiftSound = ctx.world.bg_state.rng.Q_irand(1, 4);
            shiftSound = match shiftSound {
                1 => (*vi).soundShift1,
                2 => (*vi).soundShift2,
                3 => (*vi).soundShift3,
                4 => (*vi).soundShift4,
                _ => shiftSound,
            };
            if shiftSound != 0 {
                (*pVeh).m_iSoundDebounceTimer =
                    curTime + ctx.world.bg_state.rng.Q_irand(1000, 4000);
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
                    Some(parent_id),
                    Some(parent_id),
                    Some(parent_id),
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
    ctx: &mut GameContext,
    pVeh: *mut Vehicle_t,
    pRider: *mut bgEntity_t,
    pUmcd: *mut usercmd_t,
) -> qboolean {
    unsafe {
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;

        if (*pVeh).m_iBoarding != 0 && (*pVeh).m_iDieTime == 0 {
            return qtrue;
        }

        // FLAG: Vehicle_t / vehicleInfo_t / pool-client seam derefs stay raw; the
        // rider/parent entity fields go through the accessor.
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let parent_id = ctx.entity_id_of(parent).unwrap();
        let rider = pRider as *mut gentity_t;
        let rider_id = ctx.entity_id_of(rider).unwrap();
        let pc = ctx.world.entity(parent_id).client;
        let rc = ctx.world.entity(rider_id).client;

        // MP: so they know who we're locking onto with our rockets, if anyone
        if !rider.is_null() && !rc.is_null() && !parent.is_null() && !pc.is_null() {
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
                        Vehicle_SetAnim(ctx, rider_id, SETANIM_BOTH, Anim, iFlags, iBlend);
                        // just to make sure it's cleared when roll is done
                        (*rc).ps.weaponTime = (*rc).ps.torsoTimer - 200;
                        crate::g_utils::G_AddEvent(
                            ctx.world.entity_mut(rider_id),
                            EV_ROLL as c_int,
                            0,
                        );
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
                        let lai = ctx.world.entity(rider_id).localAnimIndex;
                        let iAnimLen: c_int =
                            mp_bg::bg_panimate::BG_AnimLength(&ctx.world.bg_state, lai, Anim);
                        (*pVeh).m_iBoarding = ctx.world.level.time + iAnimLen;
                        // reuse flags: this should never be set in an entity
                        ctx.world.entity_mut(rider_id).flags |= FL_VEH_BOARDING; // MP
                                                                                 // Make sure they can't fire when leaving.
                        (*rc).ps.weaponTime = iAnimLen;
                    }

                    _VectorScale((*pc).ps.velocity, 0.25f32, &mut (*rc).ps.velocity);
                    Vehicle_SetAnim(ctx, rider_id, SETANIM_BOTH, Anim, iFlags, iBlend);
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
        if (*pVeh).m_iBoarding < ctx.world.level.time
            && (ctx.world.entity(rider_id).flags & FL_VEH_BOARDING) != 0
        {
            ctx.world.entity_mut(rider_id).flags &= !FL_VEH_BOARDING;
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
                        GIcarusTaskidpendingArgs::new(rider.cast(), TID_CHAN_VOICE as c_int),
                    ) == qfalse
                    {
                        crate::g_utils::G_AddEvent(
                            ctx.world.entity_mut(rider_id),
                            (EV_JUMP) as i32,
                            0,
                        );
                    }
                    Vehicle_SetAnim(
                        ctx,
                        rider_id,
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
                            rider_id,
                            SETANIM_BOTH,
                            Anim,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_HOLDLESS,
                            300,
                        );
                        // just to make sure it's cleared when roll is done
                        (*rc).ps.weaponTime = (*rc).ps.torsoTimer - 200;
                        crate::g_utils::G_AddEvent(
                            ctx.world.entity_mut(rider_id),
                            EV_ROLL as c_int,
                            0,
                        );
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
pub fn AttachRiders(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let mut i: c_int = 0;

        mp_bg::bg_vehicleLoad::AttachRidersGeneric(
            pVeh,
            &ctx.world.bg_state,
            &GameBgTraps::new(ctx.engine),
            ctx.world.level.time,
        );

        // FLAG: Vehicle_t / pool-client / ghoul2 seam derefs stay raw; parent and
        // rider entity fields go through the accessor.
        if !(*pVeh).m_pPilot.is_null() {
            let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
            let parent_id = ctx.entity_id_of(parent).unwrap();
            let pilot = (*pVeh).m_pPilot as *mut gentity_t;
            let pilot_id = ctx.entity_id_of(pilot).unwrap();
            let wp = ctx.world.entity(parent_id).waypoint;
            ctx.world.entity_mut(pilot_id).waypoint = wp; // take the veh's waypoint as your own

            // assuming we updated him relative to the bolt in AttachRidersGeneric
            let pcl = ctx.world.entity(pilot_id).client;
            let origin = (*pcl).ps.origin;
            crate::g_utils::G_SetOrigin(ctx.world.entity_mut(pilot_id), origin);
            let pilot_ptr = ctx.world.entity_mut(pilot_id) as *mut gentity_t;
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(pilot_ptr.cast()));
        }

        if !(*pVeh).m_pOldPilot.is_null() {
            let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
            let parent_id = ctx.entity_id_of(parent).unwrap();
            let oldpilot = (*pVeh).m_pOldPilot as *mut gentity_t;
            let oldpilot_id = ctx.entity_id_of(oldpilot).unwrap();
            let wp = ctx.world.entity(parent_id).waypoint;
            ctx.world.entity_mut(oldpilot_id).waypoint = wp;

            let pcl = ctx.world.entity(oldpilot_id).client;
            let origin = (*pcl).ps.origin;
            crate::g_utils::G_SetOrigin(ctx.world.entity_mut(oldpilot_id), origin);
            let oldpilot_ptr = ctx.world.entity_mut(oldpilot_id) as *mut gentity_t;
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(oldpilot_ptr.cast()));
        }

        // attach passengers
        while i < (*pVeh).m_iNumPassengers {
            if !(*(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize)).is_null() {
                let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
                let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
                let parent_id = ctx.entity_id_of(parent).unwrap();
                let pilot = *(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize) as *mut gentity_t;
                let pilot_id = ctx.entity_id_of(pilot).unwrap();

                let parent_ghoul2 = ctx.world.entity(parent_id).ghoul2 as *mut c_void;
                debug_assert!(!parent_ghoul2.is_null());
                let crotchBolt = trap::G2API_AddBolt(ctx.engine, parent_ghoul2, 0, "*driver");
                let ppcl = ctx.world.entity(parent_id).client;
                debug_assert!(!ppcl.is_null());
                debug_assert!(!ctx.world.entity(pilot_id).client.is_null());

                let yawOnlyAngles: vec3_t = [0.0, (*ppcl).ps.viewangles[YAW], 0.0];
                let ppcl_origin = (*ppcl).ps.origin;
                let parent_scale = ctx.world.entity(parent_id).modelScale;
                let level_time = ctx.world.level.time;

                // Get the driver tag.
                trap::G2API_GetBoltMatrix(
                    ctx.engine,
                    GG2GetboltArgs::new(
                        parent_ghoul2,
                        0,
                        crotchBolt,
                        &mut boltMatrix as *mut mdxaBone_t,
                        &yawOnlyAngles as *const vec3_t,
                        &ppcl_origin as *const vec3_t,
                        level_time,
                        core::ptr::null_mut(),
                        &parent_scale as *const vec3_t,
                    ),
                );
                let ppc = ctx.world.entity(pilot_id).client;
                BG_GiveMeVectorFromMatrix(
                    &boltMatrix,
                    Eorientations::ORIGIN as c_int,
                    &mut (*ppc).ps.origin,
                );

                let ppc_origin = (*ppc).ps.origin;
                crate::g_utils::G_SetOrigin(ctx.world.entity_mut(pilot_id), ppc_origin);
                let pilot_ptr = ctx.world.entity_mut(pilot_id) as *mut gentity_t;
                trap::LinkEntity(ctx.engine, GLinkentityArgs::new(pilot_ptr.cast()));
            }
            i += 1;
        }

        // attach droid
        if !(*pVeh).m_pDroidUnit.is_null() && (*pVeh).m_iDroidUnitTag != -1 {
            let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
            let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
            let parent_id = ctx.entity_id_of(parent).unwrap();
            let droid = (*pVeh).m_pDroidUnit as *mut gentity_t;
            let droid_id = ctx.entity_id_of(droid).unwrap();

            debug_assert!(!ctx.world.entity(parent_id).ghoul2.is_null());
            debug_assert!(!ctx.world.entity(parent_id).client.is_null());

            let dcl = ctx.world.entity(droid_id).client;
            if !dcl.is_null() {
                let ppcl = ctx.world.entity(parent_id).client;
                let yawOnlyAngles: vec3_t = [0.0, (*ppcl).ps.viewangles[YAW], 0.0];
                let parent_ghoul2 = ctx.world.entity(parent_id).ghoul2 as *mut c_void;
                let droidTag = (*pVeh).m_iDroidUnitTag;
                let parent_origin = ctx.world.entity(parent_id).r.currentOrigin;
                let parent_scale = ctx.world.entity(parent_id).modelScale;
                let level_time = ctx.world.level.time;

                // Get the droid tag.
                trap::G2API_GetBoltMatrix(
                    ctx.engine,
                    GG2GetboltArgs::new(
                        parent_ghoul2,
                        0,
                        droidTag,
                        &mut boltMatrix as *mut mdxaBone_t,
                        &yawOnlyAngles as *const vec3_t,
                        &parent_origin as *const vec3_t,
                        level_time,
                        core::ptr::null_mut(),
                        &parent_scale as *const vec3_t,
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

                let dcl_origin = (*dcl).ps.origin;
                let dcl_view = (*dcl).ps.viewangles;
                crate::g_utils::G_SetOrigin(ctx.world.entity_mut(droid_id), dcl_origin);
                crate::g_utils::G_SetAngles(ctx.world.entity_mut(droid_id), dcl_view);
                crate::g_client::SetClientViewAngle(ctx.world.entity_mut(droid_id), dcl_view);
                let droid_ptr = ctx.world.entity_mut(droid_id) as *mut gentity_t;
                trap::LinkEntity(ctx.engine, GLinkentityArgs::new(droid_ptr.cast()));

                let droid_npc = ctx.world.entity(droid_id).NPC;
                if !droid_npc.is_null() {
                    crate::npc_c::NPC_SetAnim(
                        ctx,
                        droid_id,
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
pub fn Ghost(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) {
    if pEnt.is_null() {
        return;
    }
    // FLAG: `pEnt` is a bgEntity_t seam pointer (recipe §D12); cast + id lookup
    // to reach the arena entity.
    let ent_id = ctx.entity_id_of(pEnt as *mut gentity_t).unwrap();

    // This was introduced to prevent one extra entity from being sent to the clients.
    ctx.world.entity_mut(ent_id).r.svFlags |= SVF_NOCLIENT;

    ctx.world.entity_mut(ent_id).s.eFlags |= EF_NODRAW;
    let client = ctx.world.entity(ent_id).client;
    if !client.is_null() {
        // FLAG: pool client (recipe 2b) — deref stays raw.
        unsafe {
            (*client).ps.eFlags |= EF_NODRAW;
        }
    }
    ctx.world.entity_mut(ent_id).r.contents = 0;
}

/// Raven `UnGhost` — make someone visible and collidable.
///
/// Source: `oracle/codemp/game/g_vehicles.c:2759-2781`
pub fn UnGhost(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) {
    if pEnt.is_null() {
        return;
    }
    // FLAG: `pEnt` is a bgEntity_t seam pointer (recipe §D12); cast + id lookup
    // to reach the arena entity.
    let ent_id = ctx.entity_id_of(pEnt as *mut gentity_t).unwrap();

    // make sure the client is sent again
    ctx.world.entity_mut(ent_id).r.svFlags &= !SVF_NOCLIENT;

    ctx.world.entity_mut(ent_id).s.eFlags &= !EF_NODRAW;
    let client = ctx.world.entity(ent_id).client;
    if !client.is_null() {
        // FLAG: pool client (recipe 2b) — deref stays raw.
        unsafe {
            (*client).ps.eFlags &= !EF_NODRAW;
        }
    }
    ctx.world.entity_mut(ent_id).r.contents = CONTENTS_BODY;
}

/// Raven `G_VehicleDamageBoxSizing`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:2785-2840`
pub fn G_VehicleDamageBoxSizing(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    let fDist = 256.0f32; // estimated distance to nose from origin
    let bDist = 256.0f32; // estimated distance to back from origin
    let wDist = 32.0f32; // width on each side from origin
    let hDist = 32.0f32; // height on each side from origin
                         // FLAG: Vehicle_t seam pointer (recipe §D12); resolve the parent to an id.
    let parent = unsafe { (*pVeh).m_pParentEntity as *mut gentity_t };
    let parent_id = ctx.entity_id_of(parent).unwrap();

    if ctx.world.entity(parent_id).ghoul2.is_null()
        || ctx.world.entity(parent_id).m_pVehicle.is_null()
        || ctx.world.entity(parent_id).client.is_null()
    {
        // shouldn't have gotten in here then
        return;
    }

    // only do anything if all wings are stripped off.
    // FLAG: Vehicle_t seam reads (recipe §D12) stay raw.
    let removed = unsafe { (*pVeh).m_iRemovedSurfaces };
    if (removed & SHIPSURF_BROKEN_C) == 0
        || (removed & SHIPSURF_BROKEN_D) == 0
        || (removed & SHIPSURF_BROKEN_E) == 0
        || (removed & SHIPSURF_BROKEN_F) == 0
    {
        return;
    }

    // get directions based on orientation
    let mut fwd: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let orient = unsafe { *((*pVeh).m_vOrientation as *const vec3_t) };
    AngleVectors(orient, Some(&mut fwd), Some(&mut right), Some(&mut up));

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
    // FLAG: pool client (recipe 2b) — `ps` deref stays raw.
    let pcl = ctx.world.entity(parent_id).client;
    let pcl_origin = unsafe { (*pcl).ps.origin };
    let parent_number = ctx.world.entity(parent_id).s.number;
    let parent_clipmask = ctx.world.entity(parent_id).clipmask;
    let mut trace: trace_t = unsafe { core::mem::zeroed() };
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut trace as *mut trace_t,
            &pcl_origin as *const vec3_t,
            &back as *const vec3_t,
            &nose as *const vec3_t,
            &pcl_origin as *const vec3_t,
            parent_number,
            parent_clipmask,
        ),
    );
    if trace.allsolid == 0 && trace.startsolid == 0 && trace.fraction == 1.0 {
        // all clear!
        _VectorCopy(nose, &mut ctx.world.entity_mut(parent_id).r.maxs);
        _VectorCopy(back, &mut ctx.world.entity_mut(parent_id).r.mins);
    } else {
        // oh well, DIE!
        crate::g_combat::G_Damage(
            ctx,
            Some(parent_id),
            Some(parent_id),
            Some(parent_id),
            None,
            pcl_origin,
            9999,
            DAMAGE_NO_PROTECTION,
            MOD_SUICIDE as c_int,
        );
    }
}

/// Raven `G_FlyVehicleImpactDir`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:2843-2924`
pub fn G_FlyVehicleImpactDir(ctx: &mut GameContext, veh: EntityId, trace: *mut trace_t) -> c_int {
    // `veh` is an `EntityId`; entity fields go through the accessor.
    // FLAG: Vehicle_t / pool-client are seam values (recipe §D12 / 2b) — read
    // through the accessor, the derefs stay raw.
    let pVeh = ctx.world.entity(veh).m_pVehicle;
    let vcl = ctx.world.entity(veh).client;
    if trace.is_null() || pVeh.is_null() || vcl.is_null() {
        return -1;
    }
    let veh_number = ctx.world.entity(veh).s.number;
    let veh_clipmask = ctx.world.entity(veh).clipmask;
    unsafe {
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
                veh_number,
                veh_clipmask,
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
                        veh_number,
                        veh_clipmask,
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
                        veh_number,
                        veh_clipmask,
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
        let impactAngle = mp_bg::bg_misc::vectoyaw((*trace).plane.normal);
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

/// Raven `G_SetVehDamageFlags`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:2961-3039`
pub fn G_SetVehDamageFlags(
    ctx: &mut GameContext,
    veh: EntityId,
    shipSurf: c_int,
    damageLevel: c_int,
) {
    // `veh` is an `EntityId`; entity fields go through the accessor.
    // FLAG: `.client` is a pool client on the vehicle (recipe 2b); the
    // `brokenLimbs` bitwork stays on the raw pool `ps`.
    let vcl = ctx.world.entity(veh).client;
    match damageLevel {
        3 => {
            // destroyed — add both flags so cgame knows this surf is GONE
            unsafe {
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_HEAVY + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs |= 1 << dmgFlag;
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_LIGHT + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs |= 1 << dmgFlag;
            }
            // copy down
            let bl = unsafe { (*vcl).ps.brokenLimbs };
            ctx.world.entity_mut(veh).s.brokenLimbs = bl;
            // check droid
            if shipSurf == SHIPSURF_BACK {
                // destroy the droid if we have one
                // FLAG: Vehicle_t seam deref (recipe §D12) stays raw.
                let vp = ctx.world.entity(veh).m_pVehicle;
                let droidEnt = if !vp.is_null() {
                    unsafe { (*vp).m_pDroidUnit as *mut gentity_t }
                } else {
                    core::ptr::null_mut()
                };
                if !vp.is_null() && !droidEnt.is_null() {
                    let droid_id = ctx.entity_id_of(droidEnt).unwrap();
                    let flags = ctx.world.entity(droid_id).flags;
                    let health = ctx.world.entity(droid_id).health;
                    if (flags & FL_UNDYING) != 0 || health > 0 {
                        // make it vulnerable, then blow it up
                        ctx.world.entity_mut(droid_id).flags &= !FL_UNDYING;
                        // resolve veh->enemy (Option<EntityId>) — round-trips to the
                        // same handle, so read it straight off the accessor.
                        let enemy_id = ctx.world.entity(veh).enemy;
                        // Raven passes NULL for both `dir` and `point`; `dir` is
                        // carried as `None`, `point` as the zero-vec convention.
                        let null_point: vec3_t = [0.0; 3];
                        crate::g_combat::G_Damage(
                            ctx,
                            Some(droid_id),
                            enemy_id,
                            enemy_id,
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
            unsafe {
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_HEAVY + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs |= 1 << dmgFlag;
                // remove light
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_LIGHT + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs &= !(1 << dmgFlag);
            }
            // copy down
            let bl = unsafe { (*vcl).ps.brokenLimbs };
            ctx.world.entity_mut(veh).s.brokenLimbs = bl;
            // check droid — make it vulnerable if we have one
            if shipSurf == SHIPSURF_BACK {
                let vp = ctx.world.entity(veh).m_pVehicle;
                let droidEnt = if !vp.is_null() {
                    unsafe { (*vp).m_pDroidUnit as *mut gentity_t }
                } else {
                    core::ptr::null_mut()
                };
                if !vp.is_null() && !droidEnt.is_null() {
                    let droid_id = ctx.entity_id_of(droidEnt).unwrap();
                    if (ctx.world.entity(droid_id).flags & FL_UNDYING) != 0 {
                        ctx.world.entity_mut(droid_id).flags &= !FL_UNDYING;
                    }
                }
            }
        }
        1 => {
            // light only
            unsafe {
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_LIGHT + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs |= 1 << dmgFlag;
                // remove heavy
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_HEAVY + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs &= !(1 << dmgFlag);
            }
            // copy down
            let bl = unsafe { (*vcl).ps.brokenLimbs };
            ctx.world.entity_mut(veh).s.brokenLimbs = bl;
        }
        _ => {
            // no damage (case 0 / default)
            unsafe {
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_HEAVY + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs &= !(1 << dmgFlag);
                let dmgFlag = SHIPSURF_DAMAGE_FRONT_LIGHT + (shipSurf - SHIPSURF_FRONT);
                (*vcl).ps.brokenLimbs &= !(1 << dmgFlag);
            }
            let bl = unsafe { (*vcl).ps.brokenLimbs };
            ctx.world.entity_mut(veh).s.brokenLimbs = bl;
        }
    }
}

/// Raven `G_VehicleSetDamageLocFlags`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:3041-3100`
pub fn G_VehicleSetDamageLocFlags(
    ctx: &mut GameContext,
    veh: EntityId,
    impactDir: c_int,
    deathPoint: c_int,
) {
    // `veh` is an `EntityId`; entity fields go through the accessor.
    if ctx.world.entity(veh).client.is_null() {
        return;
    }
    // Raven shadows the `deathPoint` parameter with a local of the same name.
    // FLAG: Vehicle_t / vehicleInfo_t seam derefs (recipe §D12) stay raw.
    let vp = ctx.world.entity(veh).m_pVehicle;

    let deathPoint: c_int;
    let heavyDamagePoint: c_int;
    let lightDamagePoint: c_int;
    unsafe {
        let vi = (*vp).m_pVehicleInfo as *mut vehicleInfo_t;

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

        if !vp.is_null()
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
    }

    let locDmg = ctx.world.entity(veh).locationDamage[impactDir as usize];
    if locDmg >= deathPoint {
        // destroyed
        G_SetVehDamageFlags(ctx, veh, impactDir, 3);
    } else if locDmg <= lightDamagePoint {
        // light only
        G_SetVehDamageFlags(ctx, veh, impactDir, 1);
    } else if locDmg <= heavyDamagePoint {
        // heavy only
        G_SetVehDamageFlags(ctx, veh, impactDir, 2);
    }
}

/// Raven `G_FlyVehicleDestroySurface`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:3102-3188`
pub fn G_FlyVehicleDestroySurface(
    ctx: &mut GameContext,
    veh: EntityId,
    surface: c_int,
) -> qboolean {
    // `veh` is an `EntityId`; entity fields go through the accessor.
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

    // FLAG: Vehicle_t seam deref (recipe §D12) stays raw.
    let vp = ctx.world.entity(veh).m_pVehicle;
    if unsafe { (*vp).m_iRemovedSurfaces } == 0 {
        // first time something got blown off
        let pilot = unsafe { (*vp).m_pPilot as *mut gentity_t };
        if !pilot.is_null() {
            // make the pilot scream to his death
            let pilot_id = ctx.entity_id_of(pilot).unwrap();
            crate::g_utils::G_EntitySound(
                ctx,
                pilot_id,
                CHAN_VOICE,
                G_SoundIndex("*falling1.wav"),
            );
        }
    }
    // so we can check what's broken
    unsafe {
        (*vp).m_iRemovedSurfaces |= smashedBits;
    }

    // do some explosive damage, but don't damage this ship with it
    // FLAG: pool client (recipe 2b) — `ps` deref stays raw.
    let vcl = ctx.world.entity(veh).client;
    let vcl_origin = unsafe { (*vcl).ps.origin };
    crate::g_combat::G_RadiusDamage(
        ctx,
        vcl_origin,
        Some(veh),
        100.0,
        500.0,
        Some(veh),
        None,
        MOD_VEH_EXPLOSION as c_int,
    );

    // when spiraling to your death, do the electical shader
    let t = ctx.world.level.time + 10000;
    unsafe {
        (*vcl).ps.electrifyTime = t;
    }

    qtrue
}

/// Raven `G_FlyVehicleSurfaceDestruction`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:3190-3259`
pub fn G_FlyVehicleSurfaceDestruction(
    ctx: &mut GameContext,
    veh: EntityId,
    trace: *mut trace_t,
    magnitude: c_int,
    force: qboolean,
) {
    // `veh` is an `EntityId`; entity fields go through the accessor.
    if ctx.world.entity(veh).ghoul2.is_null() || ctx.world.entity(veh).m_pVehicle.is_null() {
        // no g2 instance.. or no vehicle instance
        return;
    }

    // FLAG: Vehicle_t / vehicleInfo_t seam derefs (recipe §D12) stay raw.
    let vp = ctx.world.entity(veh).m_pVehicle;
    let vi = unsafe { (*vp).m_pVehicleInfo as *mut vehicleInfo_t };

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

        ctx.world.entity_mut(veh).locationDamage[impactDir as usize] += magnitude * 7;

        if impactDir == SHIPSURF_FRONT {
            deathPoint = unsafe { (*vi).health_front };
        } else if impactDir == SHIPSURF_BACK {
            deathPoint = unsafe { (*vi).health_back };
        } else if impactDir == SHIPSURF_RIGHT {
            deathPoint = unsafe { (*vi).health_right };
        } else if impactDir == SHIPSURF_LEFT {
            deathPoint = unsafe { (*vi).health_left };
        }

        if deathPoint != -1 {
            // got a valid health value
            if force != qfalse
                && ctx.world.entity(veh).locationDamage[impactDir as usize] < deathPoint
            {
                // force that surf to be destroyed
                ctx.world.entity_mut(veh).locationDamage[impactDir as usize] = deathPoint;
            }
            if ctx.world.entity(veh).locationDamage[impactDir as usize] >= deathPoint {
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

/// Raven `G_VehUpdateShields`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:3261-3273`
pub fn G_VehUpdateShields(targ: &mut gentity_t) {
    // `targ` is a live `&mut gentity_t`; the `targ.is_null()` guard is vacuous.
    let client = targ.client;
    let vp = targ.m_pVehicle;
    if client.is_null() || vp.is_null() {
        return;
    }
    // FLAG: Vehicle_t / vehicleInfo_t / pool-client are seam values (recipe §D12
    // / 2b); the derefs stay raw.
    unsafe {
        if (*vp).m_pVehicleInfo.is_null() {
            return;
        }
        let vi = (*vp).m_pVehicleInfo as *mut vehicleInfo_t;
        if (*vi).shields <= 0 {
            // doesn't have shields, so don't have to send it
            return;
        }
        (*client).ps.activeForcePass =
            (((*vp).m_iShields as f32 / (*vi).shields as f32) * 10.0).floor() as c_int;
    }
}

/// Raven `SetParent` — set the parent entity of this Vehicle NPC.
///
/// Source: `oracle/codemp/game/g_vehicles.c:3277-3277`
pub fn SetParent(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pParentEntity: *mut bgEntity_t) {
    unsafe {
        (*pVeh).m_pParentEntity = pParentEntity as *mut mp_bg::public::bg_entity::bgEntity_t;
    }
}

/// Raven `SetPilot` — add a pilot to the vehicle.
///
/// Source: `oracle/codemp/game/g_vehicles.c:3280-3280`
pub fn SetPilot(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pPilot: *mut bgEntity_t) {
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
pub fn Inhabited(ctx: &mut GameContext, pVeh: *mut Vehicle_t) -> qboolean {
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
    ctx: &mut GameContext,
    pVeh: *mut Vehicle_t,
    pEnt: *mut bgEntity_t,
    forceEject: qboolean,
) -> qboolean {
    unsafe {
        // FLAG: Vehicle_t / vehicleInfo_t / pool-client seam derefs stay raw; the
        // ent/parent/rider entity fields go through the accessor.
        let ent = pEnt as *mut gentity_t;
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;

        let mut taintedRider = qfalse;
        let mut deadRider = qfalse;

        if pEnt == (*pVeh).m_pDroidUnit as *mut bgEntity_t {
            G_EjectDroidUnit(ctx, pVeh, qfalse);
            return qtrue;
        }

        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let parent_id = ctx.entity_id_of(parent).unwrap();

        if !ent.is_null() {
            let eid = ctx.entity_id_of(ent).unwrap();
            let ec0 = ctx.world.entity(eid).client;
            if ctx.world.entity(eid).inuse == qfalse
                || ec0.is_null()
                || (*ec0).pers.connected != CON_CONNECTED
            {
                // MP: if someone disconnects on us, we still have to clear our owner
                // — jump straight to the ownership-cleanup section (`getItOutOfMe`).
                taintedRider = qtrue;
            } else if ctx.world.entity(eid).health < 1 {
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
            let ent_id = ctx.entity_id_of(ent).unwrap();
            let firstEjectDir = (*pVeh).m_EjectDir;
            let mut vExitPos: vec3_t = [0.0; 3];
            while VEH_TryEject(
                ctx,
                pVeh,
                parent_id,
                ent_id,
                (*pVeh).m_EjectDir,
                &mut vExitPos,
            ) == qfalse
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
                        let eo = ctx.world.entity(ent_id).r.currentOrigin;
                        _VectorCopy(eo, &mut vExitPos);
                        break;
                    } else {
                        // can't eject
                        return qfalse;
                    }
                }
            }

            // Move them to the exit position.
            G_SetOrigin(ctx.world.entity_mut(ent_id), vExitPos);
            let ec = ctx.world.entity(ent_id).client;
            let cur = ctx.world.entity(ent_id).r.currentOrigin;
            (*ec).ps.origin = cur;
            let ent_ptr = ctx.world.entity_mut(ent_id) as *mut gentity_t;
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));

            // If it's the player, stop overrides. (MP: the override-clear body is
            // `#ifndef _JK2MP` — nothing to do here.)
            if ctx.world.entity(ent_id).s.number < MAX_CLIENTS as c_int {}
        }

        // getItOutOfMe:
        let ent_id = ctx.entity_id_of(ent).unwrap();

        // If he's the pilot...
        if (*pVeh).m_pPilot == (ent as *mut mp_bg::public::bg_entity::bgEntity_t) {
            let pc = ctx.world.entity(parent_id).client;

            (*pVeh).m_pPilot = core::ptr::null_mut();
            ctx.world.entity_mut(parent_id).r.ownerNum = ENTITYNUM_NONE;
            let on = ctx.world.entity(parent_id).r.ownerNum;
            ctx.world.entity_mut(parent_id).s.owner = on; // for prediction

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
                    let newPilot_id = ctx.entity_id_of(newPilot).unwrap();
                    let np_num = ctx.world.entity(newPilot_id).s.number;
                    ctx.world.entity_mut(parent_id).r.ownerNum = np_num;
                    let on = ctx.world.entity(parent_id).r.ownerNum;
                    ctx.world.entity_mut(parent_id).s.owner = on; // for prediction
                    (*pc).ps.m_iVehicleNum = np_num + 1;

                    // rearrange the passenger slots now..
                    // QAGAME: server just needs to tell client he's not a passenger anymore
                    let np_client = ctx.world.entity(newPilot_id).client;
                    if !np_client.is_null() {
                        (*np_client).ps.generic1 = 0;
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
                            // FLAG: `moved` is a transient pointer read out of the
                            // Vehicle_t seam passenger array; kept raw exactly as Raven
                            // (which derefs it without a null guard).
                            let moved = *(*pVeh).m_ppPassengers.as_mut_ptr().add((k - 1) as usize)
                                as *mut gentity_t;
                            if !(*moved).client.is_null() {
                                (*((*moved).client)).ps.generic1 = k;
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
                    // (`psngr == ent`, so reach the client through the ent accessor.)
                    let pcl = ctx.world.entity(ent_id).client;
                    if !pcl.is_null() {
                        (*pcl).ps.generic1 = 0;
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
            let pc = ctx.world.entity(parent_id).client;
            ctx.world.entity_mut(parent_id).s.loopSound = 0;
            (*pc).ps.loopSound = 0;
            // Completely empty vehicle...?
            if (*pVeh).m_iNumPassengers == 0 {
                (*pc).ps.m_iVehicleNum = 0;
            }
        }

        if taintedRider != qfalse {
            // you can go now
            (*pVeh).m_iBoarding = ctx.world.level.time + 1000;
            return qtrue;
        }

        // Client not in a vehicle. (MP)
        let ec = ctx.world.entity(ent_id).client;
        (*ec).ps.m_iVehicleNum = 0;
        ctx.world.entity_mut(ent_id).r.ownerNum = ENTITYNUM_NONE;
        let eo = ctx.world.entity(ent_id).r.ownerNum;
        ctx.world.entity_mut(ent_id).s.owner = eo; // for prediction

        (*ec).ps.viewangles[PITCH as usize] = 0.0;
        (*ec).ps.viewangles[ROLL as usize] = 0.0;
        (*ec).ps.viewangles[YAW as usize] = *(*pVeh).m_vOrientation.add(YAW as usize);
        let view = (*ec).ps.viewangles;
        crate::g_client::SetClientViewAngle(ctx.world.entity_mut(ent_id), view);

        if (*ec).solidHack != 0 {
            (*ec).solidHack = 0;
            ctx.world.entity_mut(ent_id).r.contents = CONTENTS_BODY;
        }
        ctx.world.entity_mut(ent_id).s.m_iVehicleNum = 0;

        // The jump-out velocity, SP facing block, and the weapon-switch on-hop-off
        // logic are all `#ifndef _JK2MP` or commented-out in Raven — MP does nothing
        // in the weapon `if/else` here.

        mp_bg::bg_panimate::BG_SetLegsAnimTimer(&mut (*ec).ps, 0);
        mp_bg::bg_panimate::BG_SetTorsoAnimTimer(&mut (*ec).ps, 0);

        // Set how long until this vehicle can be boarded again.
        (*pVeh).m_iBoarding = ctx.world.level.time + 1000;

        qtrue
    }
}

/// Raven `DeathUpdate`.
///
/// Source: `oracle/codemp/game/g_vehicles.c:1485-1617`
pub fn DeathUpdate(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    unsafe {
        // FLAG: Vehicle_t / vehicleInfo_t / pool-client seam derefs stay raw;
        // the parent entity fields go through the accessor.
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let parent_id = ctx.entity_id_of(parent).unwrap();
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;

        if ctx.world.level.time >= (*pVeh).m_iDieTime {
            // If the vehicle is not empty.
            if crate::veh_dispatch::inhabited(ctx, pVeh) != qfalse {
                // MP: the SP-only `noRagTime` clear is `#ifndef _JK2MP`.

                crate::veh_dispatch::eject_all(ctx, pVeh);
                if crate::veh_dispatch::inhabited(ctx, pVeh) != qfalse {
                    // if we've still got people in us, just kill the bastards
                    let pc = ctx.world.entity(parent_id).client;
                    if !(*pVeh).m_pPilot.is_null() {
                        //FIXME: does this give proper credit to the enemy who shot you down?
                        let pilot_id = ctx.entity_id_of((*pVeh).m_pPilot as *mut gentity_t);
                        let pc_origin = (*pc).ps.origin;
                        crate::g_combat::G_Damage(
                            ctx,
                            pilot_id,
                            Some(parent_id),
                            Some(parent_id),
                            None,
                            pc_origin,
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
                                let psngr_id = ctx.entity_id_of(
                                    *(*pVeh).m_ppPassengers.as_mut_ptr().add(i as usize)
                                        as *mut gentity_t,
                                );
                                let pc_origin = (*pc).ps.origin;
                                crate::g_combat::G_Damage(
                                    ctx,
                                    psngr_id,
                                    Some(parent_id),
                                    Some(parent_id),
                                    None,
                                    pc_origin,
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

                let porigin = ctx.world.entity(parent_id).r.currentOrigin;
                let pnumber = ctx.world.entity(parent_id).s.number;
                if (*vi).iExplodeFX != 0 {
                    let mut fxAng: vec3_t = [-90.0, 0.0, 0.0];
                    crate::g_utils::G_PlayEffectID((*vi).iExplodeFX, porigin, fxAng);
                    // trace down and place mark
                    _VectorCopy(porigin, &mut bottom);
                    bottom[2] -= 80.0;
                    G_VehicleTrace(
                        ctx,
                        &mut trace,
                        porigin,
                        vec3_origin,
                        vec3_origin,
                        bottom,
                        pnumber,
                        CONTENTS_SOLID,
                    );
                    if trace.fraction < 1.0 {
                        _VectorCopy(trace.endpos, &mut bottom);
                        bottom[2] += 2.0;
                        fxAng = [-90.0, 0.0, 0.0];
                        crate::g_utils::G_PlayEffectID(
                            G_EffectIndex("ships/ship_explosion_mark"),
                            trace.endpos,
                            fxAng,
                        );
                    }
                }

                ctx.world.entity_mut(parent_id).takedamage = qfalse; // so we don't recursively damage ourselves
                if (*vi).explosionRadius > 0.0 && (*vi).explosionDamage > 0 {
                    let pmins = ctx.world.entity(parent_id).r.mins;
                    let pmaxs = ctx.world.entity(parent_id).r.maxs;
                    _VectorCopy(pmins, &mut lMins);
                    lMins[2] = -4.0; // to keep it off the ground a *little*
                    _VectorCopy(pmaxs, &mut lMaxs);
                    _VectorCopy(porigin, &mut bottom);
                    bottom[2] += pmins[2] - 32.0;
                    G_VehicleTrace(
                        ctx,
                        &mut trace,
                        porigin,
                        lMins,
                        lMaxs,
                        bottom,
                        pnumber,
                        CONTENTS_SOLID,
                    );
                    //FIXME: extern damage and radius or base on fuel
                    crate::g_combat::G_RadiusDamage(
                        ctx,
                        trace.endpos,
                        Some(parent_id),
                        (*vi).explosionDamage as f32,
                        (*vi).explosionRadius,
                        None,
                        None,
                        MOD_VEH_EXPLOSION as c_int,
                    );
                }

                ctx.world.entity_mut(parent_id).think = Some(EntThink::G_FreeEntity).into();
                let nt = ctx.world.level.time + FRAMETIME;
                ctx.world.entity_mut(parent_id).nextthink = nt;
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
