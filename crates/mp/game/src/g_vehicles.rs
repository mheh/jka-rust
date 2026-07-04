// PORT-STATUS: g_vehicles.c — pass-2 added 6 ports (G_VehicleTrace,
// G_IsRidingVehicle, G_VehicleSpawn, StartDeathDelay, ValidateBoard, VEH_TryEject);
// 14 of the packet's 20 remain parked (see PORT-ESCALATION markers below).
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
use crate::trap;
use crate::NPC_spawn::NPC_Spawn_Do;
use crate::g_utils::G_SoundIndex;
use crate::q_math::{
    AngleVectors, VectorNormalize, _DotProduct, _VectorAdd, _VectorCopy, _VectorSubtract,
};
use mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::vehicles::vehicleType_t;

// Raven vehicle constants spelled locally per this file's staging convention
// (the integrator wires the const home later; the name preserves intent).
// Boarding sentinels stored in `m_iBoarding`.
// Source: `oracle/oracle/codemp/game/bg_vehicles.h:402-403`
const VEH_MOUNT_THROW_LEFT: c_int = -5;
const VEH_MOUNT_THROW_RIGHT: c_int = -6;
// Eject-direction anon enum.
// Source: `oracle/oracle/codemp/game/bg_vehicles.h:407-414`
const VEH_EJECT_LEFT: c_int = 0;
const VEH_EJECT_RIGHT: c_int = 1;
const VEH_EJECT_FRONT: c_int = 2;
const VEH_EJECT_REAR: c_int = 3;
const VEH_EJECT_TOP: c_int = 4;
const VEH_EJECT_BOTTOM: c_int = 5;
// Default player bbox z-extents (used for the MP eject-clearance trace).
// Source: `oracle/oracle/codemp/game/bg_public.h:41-42`
const DEFAULT_MINS_2: f32 = -24.0;
const DEFAULT_MAXS_2: f32 = 40.0;

/// Raven `SVF_NOCLIENT` — don't send entity to clients, even if it has effects.
/// Source: `oracle/oracle/codemp/game/g_public.h:17`
const SVF_NOCLIENT: c_int = 0x0000_0001;

/// Raven vehicle-surface indices (`bg_vehicles.h:427-430`).
const SHIPSURF_FRONT: c_int = 0;
const SHIPSURF_BACK: c_int = 1;
const SHIPSURF_RIGHT: c_int = 2;
const SHIPSURF_LEFT: c_int = 3;

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

/// Raven `G_VehicleTrace`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:102-109`
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
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:111-120`
pub fn G_IsRidingVehicle(
    ctx: GameContext<'_>,
    pEnt: *mut gentity_t,
) -> *mut Vehicle_t {
    unsafe {
        let ent = pEnt;
        if !ent.is_null() && !(*ent).client.is_null() {
            let client = (*ent).client as *mut gclient_t;
            if (*client).NPC_class != CLASS_VEHICLE && (*ent).s.m_iVehicleNum != 0 {
                let vehNum = (*ent).s.m_iVehicleNum as usize;
                return (*ctx.world).entities[vehNum].m_pVehicle as *mut Vehicle_t;
            }
        }
        core::ptr::null_mut()
    }
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

/// Raven `G_VehicleSpawn`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:186-244`
pub fn G_VehicleSpawn(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
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

// PORT-ESCALATION(helper-visibility): needs `BG_GiveMeVectorFromMatrix` to pull
// the driver-tag origin out of the bolt matrix, but the only ported copy is a
// private `fn` inside `NPC_AI_Mark2.rs` (not `pub`, no shared home) — unreachable
// from here. (ctx now supplies g_entities/level.time/G2 traps.)
/// Raven `G_AttachToVehicle`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:247-289`
pub fn G_AttachToVehicle(
    ctx: GameContext<'_>,
    pEnt: *mut gentity_t,
    ucmd: *mut *mut usercmd_t,
) {
    todo!("Port G_AttachToVehicle — parked: helper-visibility (BG_GiveMeVectorFromMatrix not pub)")
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

/// Raven `ValidateBoard`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:496-594`
pub fn ValidateBoard(
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
        _VectorSubtract((*ent).r.currentOrigin, (*parent).r.currentOrigin, &mut vVehToEnt);
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

// PORT-ESCALATION(struct-layout): portable in principle (ctx now supplies
// level.time/G_Sound), but the body touches a wide playerState/entityState/
// vehicleInfo field surface (generic1, loopSound, m_iVehicleNum, owner/ownerNum,
// soundLoop/soundOn/numHands/hideRider, m_iDropTime, …) and copies
// Vehicle_t::m_vOrientation, which the type port models as `*mut f32` (not an
// array) — the exact Rust field paths/repr aren't given in the packet, so a
// faithful transcription can't be pinned down without exploring those layouts.
/// Raven `Board`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:630-872`
pub fn Board(
    ctx: GameContext<'_>,
    pVeh: *mut Vehicle_t,
    pEnt: *mut bgEntity_t,
) -> qboolean {
    todo!("Port Board — parked: struct-layout beyond packet")
}

/// Raven `VEH_TryEject`.
///
/// `vExitPos` is Raven's out-param exit position (fork-9: written through, never
/// NULL at any oracle caller) → `&mut vec3_t`.
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:874-987`
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
        let fVehDiag =
            ((*parent).r.maxs[0] * (*parent).r.maxs[0] + (*parent).r.maxs[1] * (*parent).r.maxs[1])
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

// PORT-ESCALATION(packet-contract): the `QAGAME` kill branch calls
// `G_Damage(droidEnt, NULL, NULL, NULL, ...)`; the resolved `G_Damage` takes its
// `dir` as a `vec3_t` by value, which cannot express the C `NULL` argument.
/// Raven `G_EjectDroidUnit`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:989-1016`
pub fn G_EjectDroidUnit(
    ctx: GameContext<'_>,
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
    ctx: GameContext<'_>,
    pVeh: *mut Vehicle_t,
) -> qboolean {
    todo!("Port EjectAll — parked: packet-contract")
}

/// Raven `StartDeathDelay`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:1451-1482`
pub fn StartDeathDelay(
    ctx: GameContext<'_>,
    pVeh: *mut Vehicle_t,
    iDelayTimeOverride: c_int,
) {
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

// PORT-ESCALATION(bg-anim-globals): ctx now reaches the `g_gravity` cvar, but the
// closing landed-anim block still indexes the runtime `bgAllAnims` table and calls
// `BG_SetAnim` against it; `bgAllAnims` is bg-owned (ruling 11 threads it via a bg
// context) and has no handle here, so the fn can't be finished faithfully.
/// Raven `Initialize`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:1626-1757`
pub fn Initialize(
    ctx: GameContext<'_>,
    pVeh: *mut Vehicle_t,
) -> qboolean {
    todo!("Port Initialize — parked: bg-anim-globals (bgAllAnims not threaded)")
}

// PORT-ESCALATION(bg-boundary): fork-8a — `Update` is a `vehicleInfo_t` vtable
// member whose fixed slot signature carries NO `ctx`, yet its body reads
// `level.time` and calls ctx-requiring fns (`VEH_TurretThink`, `G_VehUpdateShields`,
// `G_VehicleTrace`). With no ctx channel it can't reach the world/engine; needs the
// vtable-dispatch ctx-threading resolution (same seam as `G_SetSharedVehicleFunctions`).
/// Raven `Update`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:1763-2334`
pub fn Update(
    pVeh: *mut Vehicle_t,
    pUmcd: *const usercmd_t,
) -> qboolean {
    todo!("Port Update — parked: bg-boundary (vtable member, no ctx in slot signature)")
}

// PORT-ESCALATION(struct-layout): portable in principle (ctx now supplies
// level.time / trap_ICARUS_TaskIDPending), but the body routes core control through
// the `Eject` vtable slot and touches a broad playerState/entityState field surface
// (velocity, weaponTime, torsoAnimTimer, rocketLock*, eFlags/flags boarding bits, …)
// not enumerated in the packet — needs field-layout confirmation to transcribe
// faithfully without guessing.
/// Raven `UpdateRider`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:2338-2588`
pub fn UpdateRider(
    ctx: GameContext<'_>,
    pVeh: *mut Vehicle_t,
    pRider: *mut bgEntity_t,
    pUmcd: *mut usercmd_t,
) -> qboolean {
    todo!("Port UpdateRider — parked: struct-layout beyond packet")
}

// PORT-ESCALATION(helper-visibility): ctx now supplies level.time and the G2/link
// traps, but the passenger/droid attach loops need `BG_GiveMeVectorFromMatrix` to
// read origins out of the bolt matrix, and the only ported copy is a private `fn`
// in `NPC_AI_Mark2.rs` (not `pub`, no shared home) — unreachable from here.
/// Raven `AttachRiders`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:2598-2731`
pub fn AttachRiders(
    ctx: GameContext<'_>,
    pVeh: *mut Vehicle_t,
) {
    todo!("Port AttachRiders — parked: helper-visibility (BG_GiveMeVectorFromMatrix not pub)")
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

// PORT-ESCALATION(packet-contract): ctx now reaches trap_Trace, but the "oh well,
// DIE!" branch calls `G_Damage(parent, parent, parent, NULL, origin, …)` with a
// NULL `dir`; the resolved `G_Damage` takes `dir: vec3_t` by value (fork-9 only
// reshaped vec3 OUT-params, not nullable INs), so the C NULL can't be expressed.
/// Raven `G_VehicleDamageBoxSizing`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:2785-2840`
pub fn G_VehicleDamageBoxSizing(
    ctx: GameContext<'_>,
    pVeh: *mut Vehicle_t,
) {
    todo!("Port G_VehicleDamageBoxSizing — parked: packet-contract (G_Damage NULL dir)")
}

// PORT-ESCALATION(bg-boundary): fork-8a — the retrofit did NOT thread `ctx` into
// this signature (its caller `G_FlyVehicleSurfaceDestruction` invokes it ctx-free),
// yet the body needs `trap_Trace` (engine). `AngleVectors` is now reshaped/usable,
// so the ONLY blocker is the missing ctx channel; needs a ctx param to proceed.
/// Raven `G_FlyVehicleImpactDir`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:2843-2924`
pub fn G_FlyVehicleImpactDir(
    veh: *mut gentity_t,
    trace: *mut trace_t,
) -> c_int {
    todo!("Port G_FlyVehicleImpactDir — parked: bg-boundary (no ctx, needs trap_Trace)")
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

// PORT-ESCALATION(bg-boundary): fork-8a — the retrofit did NOT thread `ctx` into
// this signature (its caller `G_FlyVehicleSurfaceDestruction` invokes it ctx-free),
// yet the body reads `level.time` and calls ctx-requiring fns (`NPC_SetSurfaceOnOff`,
// `G_RadiusDamage`, `G_EntitySound`, `G_SoundIndex`). Needs a ctx param to proceed.
/// Raven `G_FlyVehicleDestroySurface`.
///
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:3102-3188`
pub fn G_FlyVehicleDestroySurface(
    veh: *mut gentity_t,
    surface: c_int,
) -> qboolean {
    todo!("Port G_FlyVehicleDestroySurface — parked: bg-boundary (no ctx, needs level.time/traps)")
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
