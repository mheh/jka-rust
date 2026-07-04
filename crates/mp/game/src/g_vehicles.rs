// PORT-COMPLETE: g_vehicles.c 12/20
//! FAITHFUL port of `oracle/oracle/codemp/game/g_vehicles.c` (MP `_JK2MP` +
//! `QAGAME` compile path).
//!
//! Generated from the `fnskel.py` signature skeleton; bodies transcribed per the
//! settled jampgame fork rulings. STAGING ONLY — not yet wired into crates/.
//!
//! Parking pattern in this file (see the workflow's recurring escalations):
//! - `raw-ptr-skeleton-no-world-handle`: reads `level.time`/`g_entities`/cvar
//!   globals or calls engine traps, none reachable from the raw-pointer skeleton
//!   signature (rulings item 1: `level`/`g_entities`/cvars live on the world).
//! - `vec3-outparam-seam`: relies on `AngleVectors`/`VectorNormalize` out-params,
//!   whose resolved signatures take `vec3_t` ([f32;3]) by value and so cannot
//!   write back — the signature can't be re-declared here.
//! - `packet-contract`: passes a C `NULL` where the resolved callee (`G_Damage`)
//!   takes a `vec3_t` by value, which cannot express a null argument.
//! - `bg-anim-globals`: indexes the runtime-populated `bgAllAnims` global table,
//!   which has no handle in scope.
//! - `vehicle-vtable`: the `vehicleInfo_t` vtable fields are
//!   `Option<unsafe extern "C" fn>` but the ported member fns are plain-Rust —
//!   assigning them needs an unsettled extern-"C" seam (fork-7 dispatch).
//!
//! The `Ghost`/`UnGhost`/`SHIPSURF_*`/`SVF_*`/`EF_*`/`CONTENTS_*` constants are
//! spelled with their Raven names as bare identifiers (staging convention: the
//! integrator wires the const, the name preserves intent).
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::q_shared::Q_strncmp;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/oracle/codemp/game/q_shared.h`
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

// Raven angle-vector indices (`q_shared.h`): PITCH=0, YAW=1, ROLL=2.
const PITCH: usize = 0;
const YAW: usize = 1;
const ROLL: usize = 2;

// PORT-ESCALATION(bg-anim-globals): the MP path indexes the runtime-populated
// `bgAllAnims[ent->localAnimIndex].anims` global and calls `BG_SetAnim` against
// it; the global table has no handle in the raw-pointer skeleton signature.
/// Raven `Vehicle_SetAnim`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:91-100`
pub fn Vehicle_SetAnim(
    ent: *mut gentity_t,
    setAnimParts: c_int,
    anim: c_int,
    setAnimFlags: c_int,
    iBlend: c_int,
) {
    todo!("Port Vehicle_SetAnim — parked: bg-anim-globals")
}

// PORT-ESCALATION(seam-threading): calls `trap_Trace`, which routes through the
// engine handle; the skeleton signature threads no `&Engine`.
/// Raven `G_VehicleTrace`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:102-109`
pub fn G_VehicleTrace(
    results: *mut trace_t,
    start: vec3_t,
    tMins: vec3_t,
    tMaxs: vec3_t,
    end: vec3_t,
    passEntityNum: c_int,
    contentmask: c_int,
) {
    todo!("Port G_VehicleTrace — parked: seam-threading")
}

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): indexes
// `g_entities[ent->s.m_iVehicleNum]`, which lives on the world, not a static.
/// Raven `G_IsRidingVehicle`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:111-120`
pub fn G_IsRidingVehicle(
    pEnt: *mut gentity_t,
) -> *mut Vehicle_t {
    todo!("Port G_IsRidingVehicle — parked: raw-ptr-skeleton-no-world-handle")
}

/// Raven `G_CanJumpToEnemyVeh`.
///
/// Raven: the entire body is `#ifndef _JK2MP`; in the MP (`_JK2MP`) compile it
/// reduces to `return 0.0f;`.
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:124-183`
pub fn G_CanJumpToEnemyVeh(
    pVeh: *mut Vehicle_t,
    pUcmd: *const usercmd_t,
) -> f32 {
    0.0
}

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): calls `trap_LinkEntity`
// and reads `level.time` — no engine/world handle in the skeleton signature.
/// Raven `G_VehicleSpawn`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:186-244`
pub fn G_VehicleSpawn(
    self_: *mut gentity_t,
) {
    todo!("Port G_VehicleSpawn — parked: raw-ptr-skeleton-no-world-handle")
}

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): indexes `g_entities`, reads
// `level.time`, and calls `trap_G2API_*`/`trap_LinkEntity` — no world/engine
// handle in the skeleton signature.
/// Raven `G_AttachToVehicle`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:247-289`
pub fn G_AttachToVehicle(
    pEnt: *mut gentity_t,
    ucmd: *mut *mut usercmd_t,
) {
    todo!("Port G_AttachToVehicle — parked: raw-ptr-skeleton-no-world-handle")
}

/// Raven `Animate` — animate the vehicle and its riders.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:481-493`
pub fn Animate(
    pVeh: *mut Vehicle_t,
) {
    unsafe {
        let vi = (*pVeh).m_pVehicleInfo as *mut vehicleInfo_t;
        // Validate a pilot rider.
        if !(*pVeh).m_pPilot.is_null() {
            if let Some(animate_riders) = (*vi).AnimateRiders {
                animate_riders(pVeh);
            }
        }
        if let Some(animate_vehicle) = (*vi).AnimateVehicle {
            animate_vehicle(pVeh);
        }
    }
}

// PORT-ESCALATION(vec3-outparam-seam): relies on `AngleVectors`(out right) and
// `VectorNormalize`(in-place) whose resolved signatures take `vec3_t` by value
// and cannot write the out-params back; the signatures can't be re-declared.
/// Raven `ValidateBoard`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:496-594`
pub fn ValidateBoard(
    pVeh: *mut Vehicle_t,
    pEnt: *mut bgEntity_t,
) -> qboolean {
    todo!("Port ValidateBoard — parked: vec3-outparam-seam")
}

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): the `QAGAME` suspend branch
// reads `level.time` — no world handle in the skeleton signature.
/// Raven `Board`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:630-872`
pub fn Board(
    pVeh: *mut Vehicle_t,
    pEnt: *mut bgEntity_t,
) -> qboolean {
    todo!("Port Board — parked: raw-ptr-skeleton-no-world-handle")
}

// PORT-ESCALATION(vec3-outparam-seam): uses `AngleVectors`(out leave-dir) and
// `VectorNormalize`(in-place) whose resolved `vec3_t`-by-value signatures cannot
// return the out-params.
/// Raven `VEH_TryEject`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:874-987`
pub fn VEH_TryEject(
    pVeh: *mut Vehicle_t,
    parent: *mut gentity_t,
    ent: *mut gentity_t,
    ejectDir: c_int,
    vExitPos: vec3_t,
) -> qboolean {
    todo!("Port VEH_TryEject — parked: vec3-outparam-seam")
}

// PORT-ESCALATION(packet-contract): the `QAGAME` kill branch calls
// `G_Damage(droidEnt, NULL, NULL, NULL, ...)`; the resolved `G_Damage` takes its
// `dir` as a `vec3_t` by value, which cannot express the C `NULL` argument.
/// Raven `G_EjectDroidUnit`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:989-1016`
pub fn G_EjectDroidUnit(
    pVeh: *mut Vehicle_t,
    kill: qboolean,
) {
    todo!("Port G_EjectDroidUnit — parked: packet-contract")
}

// PORT-ESCALATION(packet-contract): the `QAGAME` kill-rider branches call
// `G_Damage(pilot, NULL, NULL, NULL, ...)`; the resolved `G_Damage` `dir`
// parameter is a `vec3_t` by value and cannot express the C `NULL`.
/// Raven `EjectAll`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:1377-1448`
pub fn EjectAll(
    pVeh: *mut Vehicle_t,
) -> qboolean {
    todo!("Port EjectAll — parked: packet-contract")
}

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): sets
// `m_iDieTime = level.time + ...` — no world handle in the skeleton signature.
/// Raven `StartDeathDelay`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:1451-1482`
pub fn StartDeathDelay(
    pVeh: *mut Vehicle_t,
    iDelayTimeOverride: c_int,
) {
    todo!("Port StartDeathDelay — parked: raw-ptr-skeleton-no-world-handle")
}

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): reads the `g_gravity` cvar
// global and the `bgAllAnims` global table (and calls `BG_SetAnim` against it) —
// no world handle in the skeleton signature.
/// Raven `Initialize`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:1626-1757`
pub fn Initialize(
    pVeh: *mut Vehicle_t,
) -> qboolean {
    todo!("Port Initialize — parked: raw-ptr-skeleton-no-world-handle")
}

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): reads `level.time`, indexes
// `g_entities`, and calls `Q_irand` (owned-RNG global) — no world handle in the
// skeleton signature.
/// Raven `Update`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:1763-2334`
pub fn Update(
    pVeh: *mut Vehicle_t,
    pUmcd: *const usercmd_t,
) -> qboolean {
    todo!("Port Update — parked: raw-ptr-skeleton-no-world-handle")
}

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): reads `level.time` and calls
// `trap_ICARUS_TaskIDPending` — no world/engine handle in the skeleton signature.
/// Raven `UpdateRider`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:2338-2588`
pub fn UpdateRider(
    pVeh: *mut Vehicle_t,
    pRider: *mut bgEntity_t,
    pUmcd: *mut usercmd_t,
) -> qboolean {
    todo!("Port UpdateRider — parked: raw-ptr-skeleton-no-world-handle")
}

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): reads `level.time` and calls
// `trap_G2API_*`/`trap_LinkEntity` — no world/engine handle in the skeleton.
/// Raven `AttachRiders`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:2598-2731`
pub fn AttachRiders(
    pVeh: *mut Vehicle_t,
) {
    todo!("Port AttachRiders — parked: raw-ptr-skeleton-no-world-handle")
}

/// Raven `Ghost` — make someone invisible and un-collidable.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:2734-2756`
pub fn Ghost(
    pVeh: *mut Vehicle_t,
    pEnt: *mut bgEntity_t,
) {
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
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:2759-2781`
pub fn UnGhost(
    pVeh: *mut Vehicle_t,
    pEnt: *mut bgEntity_t,
) {
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

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): calls `trap_Trace` (engine)
// and `G_Damage` — no engine/world handle in the skeleton signature.
/// Raven `G_VehicleDamageBoxSizing`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:2785-2840`
pub fn G_VehicleDamageBoxSizing(
    pVeh: *mut Vehicle_t,
) {
    todo!("Port G_VehicleDamageBoxSizing — parked: raw-ptr-skeleton-no-world-handle")
}

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): calls `trap_Trace` (engine)
// and relies on `AngleVectors` out-params — no engine handle, and the resolved
// `AngleVectors` `vec3_t`-by-value signature cannot return the out-params.
/// Raven `G_FlyVehicleImpactDir`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:2843-2924`
pub fn G_FlyVehicleImpactDir(
    veh: *mut gentity_t,
    trace: *mut trace_t,
) -> c_int {
    todo!("Port G_FlyVehicleImpactDir — parked: raw-ptr-skeleton-no-world-handle")
}

/// Raven `G_ShipSurfaceForSurfName` — map a surface name to its ship surface id.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:2930-2959`
pub fn G_ShipSurfaceForSurfName(
    surfaceName: *const c_char,
) -> c_int {
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

// PORT-ESCALATION(packet-contract): the `SHIPSURF_BACK`/destroyed droid branch
// calls `G_Damage(droidEnt, veh->enemy, veh->enemy, NULL, NULL, ...)`; the
// resolved `G_Damage` `dir`/`point` are `vec3_t` by value and cannot express the
// C `NULL` arguments.
/// Raven `G_SetVehDamageFlags`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:2961-3039`
pub fn G_SetVehDamageFlags(
    veh: *mut gentity_t,
    shipSurf: c_int,
    damageLevel: c_int,
) {
    todo!("Port G_SetVehDamageFlags — parked: packet-contract")
}

/// Raven `G_VehicleSetDamageLocFlags`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:3041-3100`
pub fn G_VehicleSetDamageLocFlags(
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
            G_SetVehDamageFlags(veh, impactDir, 3);
        } else if (*veh).locationDamage[impactDir as usize] <= lightDamagePoint {
            // light only
            G_SetVehDamageFlags(veh, impactDir, 1);
        } else if (*veh).locationDamage[impactDir as usize] <= heavyDamagePoint {
            // heavy only
            G_SetVehDamageFlags(veh, impactDir, 2);
        }
    }
}

// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): sets
// `veh->client->ps.electrifyTime = level.time + 10000` and calls
// `G_RadiusDamage`/`G_EntitySound` — no world handle in the skeleton signature.
/// Raven `G_FlyVehicleDestroySurface`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:3102-3188`
pub fn G_FlyVehicleDestroySurface(
    veh: *mut gentity_t,
    surface: c_int,
) -> qboolean {
    todo!("Port G_FlyVehicleDestroySurface — parked: raw-ptr-skeleton-no-world-handle")
}

/// Raven `G_FlyVehicleSurfaceDestruction`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:3190-3259`
pub fn G_FlyVehicleSurfaceDestruction(
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

        let mut impactDir = G_FlyVehicleImpactDir(veh, trace);
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
                    if G_FlyVehicleDestroySurface(veh, impactDir) != qfalse {
                        // actually took off a surface
                        G_VehicleSetDamageLocFlags(veh, impactDir, deathPoint);
                    }
                } else {
                    G_VehicleSetDamageLocFlags(veh, impactDir, deathPoint);
                }
            }

            if alreadyRebroken == qfalse {
                let secondImpact = G_FlyVehicleImpactDir(veh, trace);
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
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:3261-3273`
pub fn G_VehUpdateShields(
    targ: *mut gentity_t,
) {
    unsafe {
        if targ.is_null()
            || (*targ).client.is_null()
            || (*targ).m_pVehicle.is_null()
        {
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
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:3277-3277`
pub fn SetParent(
    pVeh: *mut Vehicle_t,
    pParentEntity: *mut bgEntity_t,
) {
    unsafe {
        (*pVeh).m_pParentEntity = pParentEntity;
    }
}

/// Raven `SetPilot` — add a pilot to the vehicle.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:3280-3280`
pub fn SetPilot(
    pVeh: *mut Vehicle_t,
    pPilot: *mut bgEntity_t,
) {
    unsafe {
        (*pVeh).m_pPilot = pPilot;
    }
}

/// Raven `AddPassenger` — add a passenger to the vehicle (false if we're full).
///
/// Raven: the generic implementation always returns false.
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:3283-3283`
pub fn AddPassenger(
    pVeh: *mut Vehicle_t,
) -> qboolean {
    qfalse
}

/// Raven `Inhabited` — whether this vehicle is currently inhabited (by anyone).
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:3286-3286`
pub fn Inhabited(
    pVeh: *mut Vehicle_t,
) -> qboolean {
    unsafe {
        if !(*pVeh).m_pPilot.is_null() || (*pVeh).m_iNumPassengers != 0 {
            qtrue
        } else {
            qfalse
        }
    }
}

// PORT-ESCALATION(vehicle-vtable): the `vehicleInfo_t` vtable slots are
// `Option<unsafe extern "C" fn>`, but the ported member fns here are plain-Rust
// `fn` items — assigning them needs the unsettled extern-"C" dispatch seam
// (fork-7). Additionally three assigned targets (`Eject`, `DeathUpdate`,
// `RegisterAssets`) are defined in `g_vehicles.c` but excluded from this file's
// manifest, so their Rust homes are unresolved here.
/// Raven `G_SetSharedVehicleFunctions`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:3290-3314`
pub fn G_SetSharedVehicleFunctions(
    pVehInfo: *mut vehicleInfo_t,
) {
    todo!("Port G_SetSharedVehicleFunctions — parked: vehicle-vtable")
}
