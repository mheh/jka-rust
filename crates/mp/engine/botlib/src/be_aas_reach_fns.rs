#![allow(
    non_camel_case_types,
    non_snake_case,
    unused_variables,
    unused_assignments
)]

//! Function bodies for Raven's `be_aas_reach.cpp` (AAS reachability calculation).
//!
//! Ported per the engine C-track packets (`botlib__0451`..`botlib__2168`).
//! Source: `oracle/codemp/botlib/be_aas_reach.cpp`.
//!
//! PORT-NOTE(macros): Raven's vector `#define`s (`VectorSubtract`, `VectorAdd`,
//! `VectorScale`, `VectorCopy`, `VectorClear`, `VectorSet`, `VectorMA`,
//! `DotProduct`) expand inline here, faithful to the preprocessor. Only the
//! genuine q_math functions the packets flag as externals
//! (`CrossProduct`/`VectorLength`/`VectorNormalize`/`VectorInverse`/
//! `AngleVectors`) are called through the (not-yet-wired) q_math surface — see
//! missing_symbols.
//! PORT-NOTE(out-vec): several resolved signatures take `vec3_t` out-params by
//! value (§C mechanical). Raven writes through the array; bodies write into the
//! `mut` copy. Reported in shape_mismatches — integration owns the seam shape.
//! PORT-NOTE(unsafe): the AAS arena is a graph of raw pointers (`aasworld.*`);
//! bodies deref explicitly inside `unsafe` per porting-rules §D11.

use core::ffi::{c_char, c_int, c_ushort};
use core::ptr;

use crate::aasfile::aas_area_s::aas_area_t;
use crate::aasfile::aas_areasettings_s::aas_areasettings_t;
use crate::aasfile::aas_edge_s::aas_edge_t;
use crate::aasfile::aas_face_s::aas_face_t;
use crate::aasfile::aas_plane_s::aas_plane_t;
use crate::aasfile::aas_reachability_s::aas_reachability_t;
use crate::be_aas_def::aas_link_s::aas_link_t;
use crate::be_aas_reach::aas_lreachability_s::aas_lreachability_t;

use crate::aasfile::area_contents::{
    AREACONTENTS_CLUSTERPORTAL, AREACONTENTS_DONOTENTER, AREACONTENTS_JUMPPAD, AREACONTENTS_LAVA,
    AREACONTENTS_SLIME, AREACONTENTS_TELEPORTER,
};
use crate::aasfile::area_flags::{AREA_GROUNDED, AREA_LADDER, AREA_LIQUID};
use crate::aasfile::face_flags::{FACE_GROUND, FACE_LADDER, FACE_SOLID};
use crate::aasfile::presence_type::{PRESENCE_CROUCH, PRESENCE_NORMAL};
use crate::aasfile::travel_type::{
    TRAVELFLAG_NOTTEAM1, TRAVELFLAG_NOTTEAM2, TRAVELTYPE_MASK, TRAVEL_BARRIERJUMP, TRAVEL_BFGJUMP,
    TRAVEL_ELEVATOR, TRAVEL_FUNCBOB, TRAVEL_GRAPPLEHOOK, TRAVEL_JUMP, TRAVEL_JUMPPAD,
    TRAVEL_LADDER, TRAVEL_ROCKETJUMP, TRAVEL_SWIM, TRAVEL_TELEPORT, TRAVEL_WALK,
    TRAVEL_WALKOFFLEDGE, TRAVEL_WATERJUMP,
};
use crate::be_aas_bsp::be_aas_bsp_consts::MAX_EPAIRKEY;
use crate::be_aas_reach::consts::{
    AAS_MAX_REACHABILITYSIZE, AREA_WEAPONJUMP, INSIDEUNITS, INSIDEUNITS_WALKEND,
    INSIDEUNITS_WALKSTART, INSIDEUNITS_WATERJUMP,
};

use mp_qshared::common::mp::botlib::aas_clientmove_s::aas_clientmove_t;
use mp_qshared::common::mp::botlib::aas_stop_event::{
    SE_ENTERLAVA, SE_ENTERSLIME, SE_ENTERWATER, SE_HITGROUND, SE_HITGROUNDAREA, SE_HITGROUNDDAMAGE,
    SE_TOUCHCLUSTERPORTAL, SE_TOUCHJUMPPAD, SE_TOUCHTELEPORTER,
};
use mp_qshared::common::mp::botlib::aas_trace_s::aas_trace_t;
use mp_qshared::common::mp::botlib::bsp_trace_s::bsp_trace_t;
use mp_qshared::common::mp::botlib::line_color::LINECOLOR_RED;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_MESSAGE, PRT_WARNING};
use mp_qshared::shared::surface_flags::{
    CONTENTS_LAVA, CONTENTS_SLIME, CONTENTS_SOLID, CONTENTS_WATER, MASK_WATER, SURF_SKY,
};
use mp_qshared::shared::q_math::{
    AngleVectors, CrossProduct, VectorInverse, VectorLength, VectorNormalize,
};
use mp_qshared::shared::{vec3_t, vec_t};

use mp_engine_qcommon::common_fns::Com_Memset;

use libc::{atoi, strcmp};

use native_types::{qboolean, qfalse, qtrue};

use crate::BotLib;

use crate::be_aas_bspq3_fns::{
    AAS_BSPModelMinsMaxsOrigin, AAS_FloatForBSPEpairKey, AAS_IntForBSPEpairKey, AAS_NextBSPEntity,
    AAS_PointContents, AAS_Trace, AAS_ValueForBSPEpairKey, AAS_VectorForBSPEpairKey,
};
use crate::be_aas_debug_fns::AAS_PermanentLine;
use crate::be_aas_main::AAS_Error;
use crate::be_aas_move::{
    AAS_BFGJumpZVelocity, AAS_ClientMovementHitBBox, AAS_DropToFloor, AAS_HorizontalVelocityForJump,
    AAS_PredictClientMovement, AAS_RocketJumpZVelocity,
};
use crate::be_aas_sample_fns::{
    AAS_AreaPresenceType, AAS_LinkEntityClientBBox, AAS_PointAreaNum, AAS_PointInsideFace,
    AAS_TraceAreas, AAS_TraceClientBBox, AAS_UnlinkFromAreas,
};
use crate::be_interface_fns::Sys_MilliSeconds;
use crate::l_libvar_fns::{LibVarGetValue, LibVarValue};
use crate::l_log_fns::Log_Write;
use crate::l_memory_fns::{FreeMemory, GetClearedMemory};

/// Raven `M_PI` (`<math.h>`).
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:3900` (usage site).
const M_PI: f64 = std::f64::consts::PI;

/// Raven `AAS_FaceArea` — area of one AAS face.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:95-120`
pub fn AAS_FaceArea(bot: &mut BotLib, face: *mut aas_face_t) -> f32 {
    unsafe {
        let mut edgenum: c_int;
        let mut side: usize;
        let mut edge: *mut aas_edge_t;
        let mut d1: vec3_t;
        let mut d2: vec3_t;
        let mut cross: vec3_t = [0.0; 3];

        edgenum = *bot.aasworld.edgeindex.add((*face).firstedge as usize);
        side = (edgenum < 0) as usize;
        edge = bot.aasworld.edges.add(edgenum.unsigned_abs() as usize);
        let v: vec3_t = *bot.aasworld.vertexes.add((*edge).v[side] as usize);

        let mut total: f32 = 0.0;
        let mut i = 1;
        while i < (*face).numedges - 1 {
            edgenum = *bot.aasworld.edgeindex.add(((*face).firstedge + i) as usize);
            side = (edgenum < 0) as usize;
            edge = bot.aasworld.edges.add(edgenum.unsigned_abs() as usize);
            let a = *bot.aasworld.vertexes.add((*edge).v[side] as usize);
            let b = *bot.aasworld.vertexes.add((*edge).v[1 - side] as usize);
            d1 = [a[0] - v[0], a[1] - v[1], a[2] - v[2]];
            d2 = [b[0] - v[0], b[1] - v[1], b[2] - v[2]];
            CrossProduct(d1, d2, &mut cross);
            total += 0.5 * VectorLength(cross);
            i += 1;
        }
        total
    }
}

/// Raven `AAS_FaceCenter` — centroid of an AAS face.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:547-565`
pub fn AAS_FaceCenter(bot: &mut BotLib, facenum: c_int, mut center: vec3_t) {
    unsafe {
        let face: *mut aas_face_t = bot.aasworld.faces.add(facenum as usize);

        center = [0.0; 3];
        let mut i = 0;
        while i < (*face).numedges {
            let edgeidx = *bot.aasworld.edgeindex.add(((*face).firstedge + i) as usize);
            let edge: *mut aas_edge_t = bot.aasworld.edges.add(edgeidx.unsigned_abs() as usize);
            let a = *bot.aasworld.vertexes.add((*edge).v[0] as usize);
            let b = *bot.aasworld.vertexes.add((*edge).v[1] as usize);
            center = [
                center[0] + a[0] + b[0],
                center[1] + a[1] + b[1],
                center[2] + a[2] + b[2],
            ];
            i += 1;
        }
        let scale = 0.5 / (*face).numedges as f32;
        center = [center[0] * scale, center[1] * scale, center[2] * scale];
    }
}

/// Raven `AAS_FallDamageDistance`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:574-582`
pub fn AAS_FallDamageDistance(bot: &mut BotLib) -> c_int {
    let maxzvelocity = ((30 * 10000) as f64).sqrt() as f32;
    let gravity = bot.aassettings.phys_gravity;
    let t = maxzvelocity / gravity;
    (0.5 * gravity as f64 * t as f64 * t as f64) as c_int
}

/// Raven `AAS_FallDelta`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:592-600`
pub fn AAS_FallDelta(bot: &mut BotLib, distance: f32) -> f32 {
    let gravity = bot.aassettings.phys_gravity;
    let t = ((distance.abs() * 2.0 / gravity) as f64).sqrt() as f32;
    let delta = t * gravity;
    delta * delta * 0.0001
}

/// Raven `AAS_MaxJumpHeight`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:607-614`
pub fn AAS_MaxJumpHeight(bot: &mut BotLib, phys_jumpvel: f32) -> f32 {
    let phys_gravity = bot.aassettings.phys_gravity;
    //maximum height a player can jump with the given initial z velocity
    0.5 * phys_gravity * (phys_jumpvel / phys_gravity) * (phys_jumpvel / phys_gravity)
}

/// Raven `AAS_MaxJumpDistance`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:622-632`
pub fn AAS_MaxJumpDistance(bot: &mut BotLib, phys_jumpvel: f32) -> f32 {
    let phys_gravity = bot.aassettings.phys_gravity;
    let phys_maxvelocity = bot.aassettings.phys_maxvelocity;
    //time a player takes to fall the height
    let t = ((bot.aassettings.rs_maxjumpfallheight / (0.5 * phys_gravity)) as f64).sqrt() as f32;
    //maximum distance
    phys_maxvelocity * (t + phys_jumpvel / phys_gravity)
}

/// Raven `AAS_AreaCrouch`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:640-644`
pub fn AAS_AreaCrouch(bot: &mut BotLib, areanum: c_int) -> c_int {
    unsafe {
        if (*bot.aasworld.areasettings.add(areanum as usize)).presencetype & PRESENCE_NORMAL == 0 {
            qtrue
        } else {
            qfalse
        }
    }
}

/// Raven `AAS_AreaSwim`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:652-656`
pub fn AAS_AreaSwim(bot: &mut BotLib, areanum: c_int) -> c_int {
    unsafe {
        if (*bot.aasworld.areasettings.add(areanum as usize)).areaflags & AREA_LIQUID != 0 {
            qtrue
        } else {
            qfalse
        }
    }
}

/// Raven `AAS_AreaLiquid`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:664-668`
pub fn AAS_AreaLiquid(bot: &mut BotLib, areanum: c_int) -> c_int {
    unsafe {
        if (*bot.aasworld.areasettings.add(areanum as usize)).areaflags & AREA_LIQUID != 0 {
            qtrue
        } else {
            qfalse
        }
    }
}

/// Raven `AAS_AreaLava`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:675-678`
pub fn AAS_AreaLava(bot: &mut BotLib, areanum: c_int) -> c_int {
    unsafe { (*bot.aasworld.areasettings.add(areanum as usize)).contents & AREACONTENTS_LAVA }
}

/// Raven `AAS_AreaSlime`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:685-688`
pub fn AAS_AreaSlime(bot: &mut BotLib, areanum: c_int) -> c_int {
    unsafe { (*bot.aasworld.areasettings.add(areanum as usize)).contents & AREACONTENTS_SLIME }
}

/// Raven `AAS_AreaGrounded`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:696-699`
pub fn AAS_AreaGrounded(bot: &mut BotLib, areanum: c_int) -> c_int {
    unsafe { (*bot.aasworld.areasettings.add(areanum as usize)).areaflags & AREA_GROUNDED }
}

/// Raven `AAS_AreaLadder`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:707-710`
pub fn AAS_AreaLadder(bot: &mut BotLib, areanum: c_int) -> c_int {
    unsafe { (*bot.aasworld.areasettings.add(areanum as usize)).areaflags & AREA_LADDER }
}

/// Raven `AAS_AreaJumpPad`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:717-720`
pub fn AAS_AreaJumpPad(bot: &mut BotLib, areanum: c_int) -> c_int {
    unsafe { (*bot.aasworld.areasettings.add(areanum as usize)).contents & AREACONTENTS_JUMPPAD }
}

/// Raven `AAS_AreaTeleporter`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:727-730`
pub fn AAS_AreaTeleporter(bot: &mut BotLib, areanum: c_int) -> c_int {
    unsafe { (*bot.aasworld.areasettings.add(areanum as usize)).contents & AREACONTENTS_TELEPORTER }
}

/// Raven `AAS_AreaClusterPortal`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:737-740`
pub fn AAS_AreaClusterPortal(bot: &mut BotLib, areanum: c_int) -> c_int {
    unsafe {
        (*bot.aasworld.areasettings.add(areanum as usize)).contents & AREACONTENTS_CLUSTERPORTAL
    }
}

/// Raven `AAS_AreaDoNotEnter`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:747-750`
pub fn AAS_AreaDoNotEnter(bot: &mut BotLib, areanum: c_int) -> c_int {
    unsafe { (*bot.aasworld.areasettings.add(areanum as usize)).contents & AREACONTENTS_DONOTENTER }
}

/// Raven `AAS_BarrierJumpTravelTime`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:758-761`
pub fn AAS_BarrierJumpTravelTime(bot: &mut BotLib) -> c_ushort {
    (bot.aassettings.phys_jumpvel / (bot.aassettings.phys_gravity * 0.1)) as c_ushort
}

/// Raven `AAS_ReachabilityExists`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:769-778`
pub fn AAS_ReachabilityExists(bot: &mut BotLib, area1num: c_int, area2num: c_int) -> qboolean {
    unsafe {
        let mut r: *mut aas_lreachability_t = *bot.areareachability.add(area1num as usize);
        while !r.is_null() {
            if (*r).areanum == area2num {
                return qtrue;
            }
            r = (*r).next;
        }
        qfalse
    }
}

/// Raven `VectorDistance`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:1593-1599`
pub fn VectorDistance(v1: vec3_t, v2: vec3_t) -> f32 {
    let dir: vec3_t = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
    VectorLength(dir)
}

/// Raven `VectorBetweenVectors`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:1607-1614`
pub fn VectorBetweenVectors(v: vec3_t, v1: vec3_t, v2: vec3_t) -> c_int {
    let dir1: vec3_t = [v[0] - v1[0], v[1] - v1[1], v[2] - v1[2]];
    let dir2: vec3_t = [v[0] - v2[0], v[1] - v2[1], v[2] - v2[2]];
    (dir1[0] * dir2[0] + dir1[1] * dir2[1] + dir1[2] * dir2[2] <= 0.0) as c_int
}

/// Raven `VectorMiddle`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:1622-1626`
pub fn VectorMiddle(v1: vec3_t, v2: vec3_t, mut middle: vec3_t) {
    middle = [v1[0] + v2[0], v1[1] + v2[1], v1[2] + v2[2]];
    middle = [middle[0] * 0.5, middle[1] * 0.5, middle[2] * 0.5];
}

/// Raven `AAS_AreaVolume`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:128-161`
pub fn AAS_AreaVolume(bot: &mut BotLib, areanum: c_int) -> f32 {
    unsafe {
        let mut edgenum: c_int;
        let mut facenum: c_int;
        let mut side: c_int;
        let mut plane: *mut aas_plane_t;
        let mut face: *mut aas_face_t;

        let area: *mut aas_area_t = bot.aasworld.areas.add(areanum as usize);
        facenum = *bot.aasworld.faceindex.add((*area).firstface as usize);
        face = bot.aasworld.faces.add(facenum.unsigned_abs() as usize);
        edgenum = *bot.aasworld.edgeindex.add((*face).firstedge as usize);
        let edge: *mut aas_edge_t = bot.aasworld.edges.add(edgenum.unsigned_abs() as usize);
        //
        let corner: vec3_t = *bot.aasworld.vertexes.add((*edge).v[0] as usize);

        //make tetrahedrons to all other faces
        let mut volume: f32 = 0.0;
        let mut i = 0;
        while i < (*area).numfaces {
            facenum = (*bot.aasworld.faceindex.add(((*area).firstface + i) as usize)).abs();
            face = bot.aasworld.faces.add(facenum as usize);
            side = ((*face).backarea != areanum) as c_int;
            plane = bot.aasworld.planes.add(((*face).planenum ^ side) as usize);
            let d = -((corner[0] * (*plane).normal[0]
                + corner[1] * (*plane).normal[1]
                + corner[2] * (*plane).normal[2])
                - (*plane).dist);
            let a = AAS_FaceArea(bot, face);
            volume += d * a;
            i += 1;
        }

        volume /= 3.0;
        volume
    }
}

/// Raven `AAS_ShutDownReachabilityHeap`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:459-463`
pub fn AAS_ShutDownReachabilityHeap(bot: &mut BotLib) {
    FreeMemory(bot, bot.reachabilityheap as *mut ());
    bot.numlreachabilities = 0;
}

/// Raven `AAS_AllocReachability`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:471-483`
pub fn AAS_AllocReachability(bot: &mut BotLib) -> *mut aas_lreachability_t {
    unsafe {
        if bot.nextreachability.is_null() {
            return ptr::null_mut();
        }
        //make sure the error message only shows up once
        if (*bot.nextreachability).next.is_null() {
            AAS_Error(bot, c"AAS_MAX_REACHABILITYSIZE".as_ptr() as *mut c_char);
        }
        //
        let r = bot.nextreachability;
        bot.nextreachability = (*bot.nextreachability).next;
        bot.numlreachabilities += 1;
        r
    }
}

/// Raven `AAS_FreeReachability`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:491-498`
pub fn AAS_FreeReachability(bot: &mut BotLib, lreach: *mut aas_lreachability_t) {
    unsafe {
        Com_Memset(
            lreach as *mut (),
            0,
            core::mem::size_of::<aas_lreachability_t>(),
        );

        (*lreach).next = bot.nextreachability;
        bot.nextreachability = lreach;
        bot.numlreachabilities -= 1;
    }
}

/// Raven `AAS_AreaReachability`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:506-514`
pub fn AAS_AreaReachability(bot: &mut BotLib, areanum: c_int) -> c_int {
    unsafe {
        if areanum < 0 || areanum >= bot.aasworld.numareas {
            AAS_Error(
                bot,
                c"AAS_AreaReachability: areanum %d out of range".as_ptr() as *mut c_char,
                areanum,
            );
            return 0;
        }
        (*bot.aasworld.areasettings.add(areanum as usize)).numreachableareas
    }
}

/// Raven `AAS_AreaGroundFaceArea`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:522-539`
pub fn AAS_AreaGroundFaceArea(bot: &mut BotLib, areanum: c_int) -> f32 {
    unsafe {
        let mut total: f32 = 0.0;
        let area: *mut aas_area_t = bot.aasworld.areas.add(areanum as usize);
        let mut i = 0;
        while i < (*area).numfaces {
            let faceidx = *bot.aasworld.faceindex.add(((*area).firstface + i) as usize);
            let face: *mut aas_face_t = bot.aasworld.faces.add(faceidx.unsigned_abs() as usize);
            if (*face).faceflags & FACE_GROUND == 0 {
                i += 1;
                continue;
            }
            //
            total += AAS_FaceArea(bot, face);
            i += 1;
        }
        total
    }
}

/// Raven `AAS_NearbySolidOrGap`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:787-811`
pub fn AAS_NearbySolidOrGap(bot: &mut BotLib, start: vec3_t, end: vec3_t) -> c_int {
    let mut dir: vec3_t = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
    dir[2] = 0.0;
    VectorNormalize(&mut dir);
    let mut testpoint: vec3_t = [
        end[0] + dir[0] * 48.0,
        end[1] + dir[1] * 48.0,
        end[2] + dir[2] * 48.0,
    ];

    let mut areanum = AAS_PointAreaNum(bot, testpoint);
    if areanum == 0 {
        testpoint[2] += 16.0;
        areanum = AAS_PointAreaNum(bot, testpoint);
        if areanum == 0 {
            return qtrue;
        }
    }
    testpoint = [
        end[0] + dir[0] * 64.0,
        end[1] + dir[1] * 64.0,
        end[2] + dir[2] * 64.0,
    ];
    areanum = AAS_PointAreaNum(bot, testpoint);
    if areanum != 0 && AAS_AreaSwim(bot, areanum) == 0 && AAS_AreaGrounded(bot, areanum) == 0 {
        return qtrue;
    }
    qfalse
}

/// Raven `AAS_ClosestEdgePoints` — shortest distance between two ground edges.
///
/// PORT-NOTE(out-vec): `plane1`/`plane2` are read-only raw pointers; the
/// `beststart*`/`bestend*` out-vectors are by-value per §C (Raven writes
/// through). See shape_mismatches.
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:1814-2076`
pub fn AAS_ClosestEdgePoints(
    v1: vec3_t,
    v2: vec3_t,
    v3: vec3_t,
    v4: vec3_t,
    plane1: *mut aas_plane_t,
    plane2: *mut aas_plane_t,
    mut beststart1: vec3_t,
    mut bestend1: vec3_t,
    mut beststart2: vec3_t,
    mut bestend2: vec3_t,
    mut bestdist: f32,
) -> f32 {
    unsafe {
        let mut p1: vec3_t = [0.0; 3];
        let mut p2: vec3_t = [0.0; 3];
        let mut p3: vec3_t = [0.0; 3];
        let mut p4: vec3_t = [0.0; 3];
        let a1: f32;
        let a2: f32;
        let b1: f32;
        let b2: f32;
        let mut dist: f32;
        let mut dist1: f32;
        let mut dist2: f32;
        let mut founddist: c_int;

        //edge vectors
        let mut dir1: vec3_t = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
        let mut dir2: vec3_t = [v4[0] - v3[0], v4[1] - v3[1], v4[2] - v3[2]];
        //get the horizontal directions
        dir1[2] = 0.0;
        dir2[2] = 0.0;
        //
        if dir2[0] != 0.0 {
            a2 = dir2[1] / dir2[0];
            b2 = v3[1] - a2 * v3[0];
            //point on the edge vector of area2 closest to v1
            p1[0] = ((v1[0] * dir2[0] + v1[1] * dir2[1] + v1[2] * dir2[2])
                - (a2 * dir2[0] + b2 * dir2[1]))
                / dir2[0];
            p1[1] = a2 * p1[0] + b2;
            //point on the edge vector of area2 closest to v2
            p2[0] = ((v2[0] * dir2[0] + v2[1] * dir2[1] + v2[2] * dir2[2])
                - (a2 * dir2[0] + b2 * dir2[1]))
                / dir2[0];
            p2[1] = a2 * p2[0] + b2;
        } else {
            //point on the edge vector of area2 closest to v1
            p1[0] = v3[0];
            p1[1] = v1[1];
            //point on the edge vector of area2 closest to v2
            p2[0] = v3[0];
            p2[1] = v2[1];
        }
        //
        if dir1[0] != 0.0 {
            //
            a1 = dir1[1] / dir1[0];
            b1 = v1[1] - a1 * v1[0];
            //point on the edge vector of area1 closest to v3
            p3[0] = ((v3[0] * dir1[0] + v3[1] * dir1[1] + v3[2] * dir1[2])
                - (a1 * dir1[0] + b1 * dir1[1]))
                / dir1[0];
            p3[1] = a1 * p3[0] + b1;
            //point on the edge vector of area1 closest to v4
            p4[0] = ((v4[0] * dir1[0] + v4[1] * dir1[1] + v4[2] * dir1[2])
                - (a1 * dir1[0] + b1 * dir1[1]))
                / dir1[0];
            p4[1] = a1 * p4[0] + b1;
        } else {
            //point on the edge vector of area1 closest to v3
            p3[0] = v1[0];
            p3[1] = v3[1];
            //point on the edge vector of area1 closest to v4
            p4[0] = v1[0];
            p4[1] = v4[1];
        }
        //start with zero z-coordinates
        p1[2] = 0.0;
        p2[2] = 0.0;
        p3[2] = 0.0;
        p4[2] = 0.0;
        //calculate the z-coordinates from the ground planes
        p1[2] = ((*plane2).dist
            - ((*plane2).normal[0] * p1[0]
                + (*plane2).normal[1] * p1[1]
                + (*plane2).normal[2] * p1[2]))
            / (*plane2).normal[2];
        p2[2] = ((*plane2).dist
            - ((*plane2).normal[0] * p2[0]
                + (*plane2).normal[1] * p2[1]
                + (*plane2).normal[2] * p2[2]))
            / (*plane2).normal[2];
        p3[2] = ((*plane1).dist
            - ((*plane1).normal[0] * p3[0]
                + (*plane1).normal[1] * p3[1]
                + (*plane1).normal[2] * p3[2]))
            / (*plane1).normal[2];
        p4[2] = ((*plane1).dist
            - ((*plane1).normal[0] * p4[0]
                + (*plane1).normal[1] * p4[1]
                + (*plane1).normal[2] * p4[2]))
            / (*plane1).normal[2];
        //
        founddist = qfalse;
        //
        if VectorBetweenVectors(p1, v3, v4) != 0 {
            dist = VectorDistance(v1, p1);
            if dist > bestdist - 0.5 && dist < bestdist + 0.5 {
                dist1 = VectorDistance(beststart1, v1);
                dist2 = VectorDistance(beststart2, v1);
                if dist1 > dist2 {
                    if dist1 > VectorDistance(beststart1, beststart2) {
                        beststart2 = v1;
                    }
                } else if dist2 > VectorDistance(beststart1, beststart2) {
                    beststart1 = v1;
                }
                dist1 = VectorDistance(bestend1, p1);
                dist2 = VectorDistance(bestend2, p1);
                if dist1 > dist2 {
                    if dist1 > VectorDistance(bestend1, bestend2) {
                        bestend2 = p1;
                    }
                } else if dist2 > VectorDistance(bestend1, bestend2) {
                    bestend1 = p1;
                }
            } else if dist < bestdist {
                bestdist = dist;
                beststart1 = v1;
                beststart2 = v1;
                bestend1 = p1;
                bestend2 = p1;
            }
            founddist = qtrue;
        }
        if VectorBetweenVectors(p2, v3, v4) != 0 {
            dist = VectorDistance(v2, p2);
            if dist > bestdist - 0.5 && dist < bestdist + 0.5 {
                dist1 = VectorDistance(beststart1, v2);
                dist2 = VectorDistance(beststart2, v2);
                if dist1 > dist2 {
                    if dist1 > VectorDistance(beststart1, beststart2) {
                        beststart2 = v2;
                    }
                } else if dist2 > VectorDistance(beststart1, beststart2) {
                    beststart1 = v2;
                }
                dist1 = VectorDistance(bestend1, p2);
                dist2 = VectorDistance(bestend2, p2);
                if dist1 > dist2 {
                    if dist1 > VectorDistance(bestend1, bestend2) {
                        bestend2 = p2;
                    }
                } else if dist2 > VectorDistance(bestend1, bestend2) {
                    bestend1 = p2;
                }
            } else if dist < bestdist {
                bestdist = dist;
                beststart1 = v2;
                beststart2 = v2;
                bestend1 = p2;
                bestend2 = p2;
            }
            founddist = qtrue;
        }
        if VectorBetweenVectors(p3, v1, v2) != 0 {
            dist = VectorDistance(v3, p3);
            if dist > bestdist - 0.5 && dist < bestdist + 0.5 {
                dist1 = VectorDistance(beststart1, p3);
                dist2 = VectorDistance(beststart2, p3);
                if dist1 > dist2 {
                    if dist1 > VectorDistance(beststart1, beststart2) {
                        beststart2 = p3;
                    }
                } else if dist2 > VectorDistance(beststart1, beststart2) {
                    beststart1 = p3;
                }
                dist1 = VectorDistance(bestend1, v3);
                dist2 = VectorDistance(bestend2, v3);
                if dist1 > dist2 {
                    if dist1 > VectorDistance(bestend1, bestend2) {
                        bestend2 = v3;
                    }
                } else if dist2 > VectorDistance(bestend1, bestend2) {
                    bestend1 = v3;
                }
            } else if dist < bestdist {
                bestdist = dist;
                beststart1 = p3;
                beststart2 = p3;
                bestend1 = v3;
                bestend2 = v3;
            }
            founddist = qtrue;
        }
        if VectorBetweenVectors(p4, v1, v2) != 0 {
            dist = VectorDistance(v4, p4);
            if dist > bestdist - 0.5 && dist < bestdist + 0.5 {
                dist1 = VectorDistance(beststart1, p4);
                dist2 = VectorDistance(beststart2, p4);
                if dist1 > dist2 {
                    if dist1 > VectorDistance(beststart1, beststart2) {
                        beststart2 = p4;
                    }
                } else if dist2 > VectorDistance(beststart1, beststart2) {
                    beststart1 = p4;
                }
                dist1 = VectorDistance(bestend1, v4);
                dist2 = VectorDistance(bestend2, v4);
                if dist1 > dist2 {
                    if dist1 > VectorDistance(bestend1, bestend2) {
                        bestend2 = v4;
                    }
                } else if dist2 > VectorDistance(bestend1, bestend2) {
                    bestend1 = v4;
                }
            } else if dist < bestdist {
                bestdist = dist;
                beststart1 = p4;
                beststart2 = p4;
                bestend1 = v4;
                bestend2 = v4;
            }
            founddist = qtrue;
        }
        //if no shortest distance was found the shortest distance
        //is between one of the vertexes of edge1 and one of edge2
        if founddist == 0 {
            dist = VectorDistance(v1, v3);
            if dist < bestdist {
                bestdist = dist;
                beststart1 = v1;
                beststart2 = v1;
                bestend1 = v3;
                bestend2 = v3;
            }
            dist = VectorDistance(v1, v4);
            if dist < bestdist {
                bestdist = dist;
                beststart1 = v1;
                beststart2 = v1;
                bestend1 = v4;
                bestend2 = v4;
            }
            dist = VectorDistance(v2, v3);
            if dist < bestdist {
                bestdist = dist;
                beststart1 = v2;
                beststart2 = v2;
                bestend1 = v3;
                bestend2 = v3;
            }
            dist = VectorDistance(v2, v4);
            if dist < bestdist {
                bestdist = dist;
                beststart1 = v2;
                beststart2 = v2;
                bestend1 = v4;
                bestend2 = v4;
            }
        }
        bestdist
    }
}

/// Raven `AAS_BestReachableLinkArea`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:168-189`
pub fn AAS_BestReachableLinkArea(bot: &mut BotLib, areas: *mut aas_link_t) -> c_int {
    unsafe {
        let mut link: *mut aas_link_t = areas;
        while !link.is_null() {
            if AAS_AreaGrounded(bot, (*link).areanum) != 0
                || AAS_AreaSwim(bot, (*link).areanum) != 0
            {
                return (*link).areanum;
            }
            link = (*link).next_area;
        }
        //
        link = areas;
        while !link.is_null() {
            if (*link).areanum != 0 {
                return (*link).areanum;
            }
            // Raven note: this is a bad idea when the reachability is not yet
            // calculated when the level items are loaded
            if AAS_AreaReachability(bot, (*link).areanum) != 0 {
                return (*link).areanum;
            }
            link = (*link).next_area;
        }
        0
    }
}

/// Raven `AAS_SetupReachabilityHeap`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:439-452`
pub fn AAS_SetupReachabilityHeap(bot: &mut BotLib) {
    unsafe {
        bot.reachabilityheap = GetClearedMemory(
            bot,
            AAS_MAX_REACHABILITYSIZE as usize * core::mem::size_of::<aas_lreachability_t>(),
        ) as *mut aas_lreachability_t;
        let mut i = 0;
        while i < AAS_MAX_REACHABILITYSIZE - 1 {
            (*bot.reachabilityheap.add(i as usize)).next =
                bot.reachabilityheap.add((i + 1) as usize);
            i += 1;
        }
        (*bot
            .reachabilityheap
            .add((AAS_MAX_REACHABILITYSIZE - 1) as usize))
        .next = ptr::null_mut();
        bot.nextreachability = bot.reachabilityheap;
        bot.numlreachabilities = 0;
    }
}

/// Raven `AAS_Reachability_Swim` — swim reachability between two water areas.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:819-887`
pub fn AAS_Reachability_Swim(bot: &mut BotLib, area1num: c_int, area2num: c_int) -> c_int {
    unsafe {
        if AAS_AreaSwim(bot, area1num) == 0 || AAS_AreaSwim(bot, area2num) == 0 {
            return qfalse;
        }
        //if the second area is crouch only
        if (*bot.aasworld.areasettings.add(area2num as usize)).presencetype & PRESENCE_NORMAL == 0 {
            return qfalse;
        }

        let area1: *mut aas_area_t = bot.aasworld.areas.add(area1num as usize);
        let area2: *mut aas_area_t = bot.aasworld.areas.add(area2num as usize);

        //if the areas are not near anough
        let mut i = 0;
        while i < 3 {
            if (*area1).mins[i] > (*area2).maxs[i] + 10.0 {
                return qfalse;
            }
            if (*area1).maxs[i] < (*area2).mins[i] - 10.0 {
                return qfalse;
            }
            i += 1;
        }
        //find a shared face and create a reachability link
        let mut i = 0;
        while i < (*area1).numfaces {
            let mut face1num = *bot
                .aasworld
                .faceindex
                .add(((*area1).firstface + i) as usize);
            let side1 = (face1num < 0) as usize;
            face1num = face1num.abs();
            //
            let mut j = 0;
            while j < (*area2).numfaces {
                let face2num = (*bot
                    .aasworld
                    .faceindex
                    .add(((*area2).firstface + j) as usize))
                .abs();
                //
                if face1num == face2num {
                    let mut start: vec3_t = [0.0; 3];
                    AAS_FaceCenter(bot, face1num, start);
                    //
                    if AAS_PointContents(bot, start)
                        & (CONTENTS_LAVA | CONTENTS_SLIME | CONTENTS_WATER)
                        != 0
                    {
                        //
                        let face1: *mut aas_face_t = bot.aasworld.faces.add(face1num as usize);
                        let _areasettings: *mut aas_areasettings_t =
                            bot.aasworld.areasettings.add(area1num as usize);
                        //create a new reachability link
                        let lreach = AAS_AllocReachability(bot);
                        if lreach.is_null() {
                            return qfalse;
                        }
                        (*lreach).areanum = area2num;
                        (*lreach).facenum = face1num;
                        (*lreach).edgenum = 0;
                        (*lreach).start = start;
                        let plane: *mut aas_plane_t = bot
                            .aasworld
                            .planes
                            .add(((*face1).planenum ^ side1 as c_int) as usize);
                        (*lreach).end = [
                            (*lreach).start[0] + (*plane).normal[0] * ((-INSIDEUNITS) as f32),
                            (*lreach).start[1] + (*plane).normal[1] * ((-INSIDEUNITS) as f32),
                            (*lreach).start[2] + (*plane).normal[2] * ((-INSIDEUNITS) as f32),
                        ];
                        (*lreach).traveltype = TRAVEL_SWIM;
                        (*lreach).traveltime = 1;
                        //if the volume of the area is rather small
                        if AAS_AreaVolume(bot, area2num) < 800.0 {
                            (*lreach).traveltime += 200;
                        }
                        //if (!(AAS_PointContents(start) & MASK_WATER)) lreach->traveltime += 500;
                        //link the reachability
                        (*lreach).next = *bot.areareachability.add(area1num as usize);
                        *bot.areareachability.add(area1num as usize) = lreach;
                        bot.reach_swim += 1;
                        return qtrue;
                    }
                }
                j += 1;
            }
            i += 1;
        }
        qfalse
    }
}

/// Raven `AAS_Reachability_EqualFloorHeight` — walk between equal-height floors.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:896-1034`
pub fn AAS_Reachability_EqualFloorHeight(
    bot: &mut BotLib,
    area1num: c_int,
    area2num: c_int,
) -> c_int {
    unsafe {
        let mut edgenum: c_int;
        let mut side: c_int;
        let mut height: f32;
        let mut length: f32;
        let mut dir: vec3_t;
        let mut start: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let mut normal: vec3_t = [0.0; 3];
        let mut invgravity: vec3_t;
        let gravitydirection: vec3_t = [0.0, 0.0, -1.0];
        let mut edgevec: vec3_t;

        if AAS_AreaGrounded(bot, area1num) == 0 || AAS_AreaGrounded(bot, area2num) == 0 {
            return qfalse;
        }

        let area1: *mut aas_area_t = bot.aasworld.areas.add(area1num as usize);
        let area2: *mut aas_area_t = bot.aasworld.areas.add(area2num as usize);
        //if the areas are not near anough in the x-y direction
        let mut i = 0;
        while i < 2 {
            if (*area1).mins[i] > (*area2).maxs[i] + 10.0 {
                return qfalse;
            }
            if (*area1).maxs[i] < (*area2).mins[i] - 10.0 {
                return qfalse;
            }
            i += 1;
        }
        //if area 2 is too high above area 1
        if (*area2).mins[2] > (*area1).maxs[2] {
            return qfalse;
        }
        //
        invgravity = gravitydirection;
        VectorInverse(&mut invgravity);
        //
        let mut bestheight: f32 = 99999.0;
        let mut bestlength: f32 = 0.0;
        let mut foundreach: c_int = qfalse;
        let mut lr: aas_lreachability_t = core::mem::zeroed();
        Com_Memset(
            (&mut lr as *mut aas_lreachability_t) as *mut (),
            0,
            core::mem::size_of::<aas_lreachability_t>(),
        ); //make the compiler happy
           //
           //check if the areas have ground faces with a common edge
           //if existing use the lowest common edge for a reachability link
        let mut i = 0;
        while i < (*area1).numfaces {
            let faceidx = *bot
                .aasworld
                .faceindex
                .add(((*area1).firstface + i) as usize);
            let face1: *mut aas_face_t = bot.aasworld.faces.add(faceidx.unsigned_abs() as usize);
            if (*face1).faceflags & FACE_GROUND == 0 {
                i += 1;
                continue;
            }
            //
            let mut j = 0;
            while j < (*area2).numfaces {
                let f2idx = *bot
                    .aasworld
                    .faceindex
                    .add(((*area2).firstface + j) as usize);
                let face2: *mut aas_face_t = bot.aasworld.faces.add(f2idx.unsigned_abs() as usize);
                if (*face2).faceflags & FACE_GROUND == 0 {
                    j += 1;
                    continue;
                }
                //if there is a common edge
                let mut edgenum1 = 0;
                while edgenum1 < (*face1).numedges {
                    let mut edgenum2 = 0;
                    while edgenum2 < (*face2).numedges {
                        if (*bot
                            .aasworld
                            .edgeindex
                            .add(((*face1).firstedge + edgenum1) as usize))
                        .abs()
                            != (*bot
                                .aasworld
                                .edgeindex
                                .add(((*face2).firstedge + edgenum2) as usize))
                            .abs()
                        {
                            edgenum2 += 1;
                            continue;
                        }
                        edgenum = *bot
                            .aasworld
                            .edgeindex
                            .add(((*face1).firstedge + edgenum1) as usize);
                        side = (edgenum < 0) as c_int;
                        let edge: *mut aas_edge_t =
                            bot.aasworld.edges.add(edgenum.unsigned_abs() as usize);
                        //get the length of the edge
                        let ev1 = *bot.aasworld.vertexes.add((*edge).v[1] as usize);
                        let ev0 = *bot.aasworld.vertexes.add((*edge).v[0] as usize);
                        dir = [ev1[0] - ev0[0], ev1[1] - ev0[1], ev1[2] - ev0[2]];
                        length = VectorLength(dir);
                        //get the start point
                        start = [ev0[0] + ev1[0], ev0[1] + ev1[1], ev0[2] + ev1[2]];
                        start = [start[0] * 0.5, start[1] * 0.5, start[2] * 0.5];
                        end = start;
                        //get the end point several units inside area2
                        //and the start point several units inside area1
                        //NOTE: normal is pointing into area2 because the
                        //face edges are stored counter clockwise
                        let evs = *bot.aasworld.vertexes.add((*edge).v[side as usize] as usize);
                        let evns = *bot
                            .aasworld
                            .vertexes
                            .add((*edge).v[(1 - side) as usize] as usize);
                        edgevec = [evs[0] - evns[0], evs[1] - evns[1], evs[2] - evns[2]];
                        let plane2: *mut aas_plane_t =
                            bot.aasworld.planes.add((*face2).planenum as usize);
                        CrossProduct(edgevec, (*plane2).normal, &mut normal);
                        VectorNormalize(&mut normal);
                        //
                        //VectorMA(start, -1, normal, start);
                        end = [
                            end[0] + normal[0] * (INSIDEUNITS_WALKEND as f32),
                            end[1] + normal[1] * (INSIDEUNITS_WALKEND as f32),
                            end[2] + normal[2] * (INSIDEUNITS_WALKEND as f32),
                        ];
                        start = [
                            start[0] + normal[0] * (INSIDEUNITS_WALKSTART),
                            start[1] + normal[1] * (INSIDEUNITS_WALKSTART),
                            start[2] + normal[2] * (INSIDEUNITS_WALKSTART),
                        ];
                        end[2] += 0.125;
                        //
                        height = invgravity[0] * start[0]
                            + invgravity[1] * start[1]
                            + invgravity[2] * start[2];
                        //get the longest lowest edge
                        if height < bestheight || (height < bestheight + 1.0 && length > bestlength)
                        {
                            bestheight = height;
                            bestlength = length;
                            //create a new reachability link
                            lr.areanum = area2num;
                            lr.facenum = 0;
                            lr.edgenum = edgenum;
                            lr.start = start;
                            lr.end = end;
                            lr.traveltype = TRAVEL_WALK;
                            lr.traveltime = 1;
                            foundreach = qtrue;
                        }
                        edgenum2 += 1;
                    }
                    edgenum1 += 1;
                }
                j += 1;
            }
            i += 1;
        }
        if foundreach != 0 {
            //create a new reachability link
            let lreach = AAS_AllocReachability(bot);
            if lreach.is_null() {
                return qfalse;
            }
            (*lreach).areanum = lr.areanum;
            (*lreach).facenum = lr.facenum;
            (*lreach).edgenum = lr.edgenum;
            (*lreach).start = lr.start;
            (*lreach).end = lr.end;
            (*lreach).traveltype = lr.traveltype;
            (*lreach).traveltime = lr.traveltime;
            (*lreach).next = *bot.areareachability.add(area1num as usize);
            *bot.areareachability.add(area1num as usize) = lreach;
            //if going into a crouch area
            if AAS_AreaCrouch(bot, area1num) == 0 && AAS_AreaCrouch(bot, area2num) != 0 {
                (*lreach).traveltime += bot.aassettings.rs_startcrouch as c_ushort;
            }
            //avoid rather small areas
            //
            bot.reach_equalfloor += 1;
            return qtrue;
        }
        qfalse
    }
}

/// Raven `AAS_FindFaceReachabilities`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:3145-3261`
pub fn AAS_FindFaceReachabilities(
    bot: &mut BotLib,
    facepoints: *mut vec3_t,
    numpoints: c_int,
    plane: *mut aas_plane_t,
    towardsface: c_int,
) -> *mut aas_lreachability_t {
    unsafe {
        let mut facenum: c_int;
        let mut edgenum: c_int;
        let mut bestfacenum: c_int;
        let mut bestdist: f32;
        let mut speed: f32 = 0.0;
        let mut hordist: f32;
        let mut dist: f32;
        let mut beststart: vec3_t = [0.0; 3];
        let mut beststart2: vec3_t = [0.0; 3];
        let mut bestend: vec3_t = [0.0; 3];
        let mut bestend2: vec3_t = [0.0; 3];
        let mut tmp: vec3_t;
        let mut hordir: vec3_t;
        let mut testpoint: vec3_t;
        let mut bestfaceplane: *mut aas_plane_t;

        //
        let mut lreachabilities: *mut aas_lreachability_t = ptr::null_mut();
        bestfacenum = 0;
        bestfaceplane = ptr::null_mut();
        //
        let mut i = 1;
        while i < bot.aasworld.numareas {
            let area: *mut aas_area_t = bot.aasworld.areas.add(i as usize);
            // get the shortest distance between one of the func_bob start edges and
            // one of the face edges of area1
            bestdist = 999999.0;
            let mut j = 0;
            while j < (*area).numfaces {
                facenum = *bot.aasworld.faceindex.add(((*area).firstface + j) as usize);
                let face: *mut aas_face_t = bot.aasworld.faces.add(facenum.unsigned_abs() as usize);
                //if not a ground face
                if (*face).faceflags & FACE_GROUND == 0 {
                    j += 1;
                    continue;
                }
                //get the ground planes
                let faceplane: *mut aas_plane_t =
                    bot.aasworld.planes.add((*face).planenum as usize);
                //
                let mut k = 0;
                while k < (*face).numedges {
                    edgenum = (*bot.aasworld.edgeindex.add(((*face).firstedge + k) as usize)).abs();
                    let edge: *mut aas_edge_t = bot.aasworld.edges.add(edgenum as usize);
                    //calculate the minimum distance between the two edges
                    let v1 = *bot.aasworld.vertexes.add((*edge).v[0] as usize);
                    let v2 = *bot.aasworld.vertexes.add((*edge).v[1] as usize);
                    //
                    let mut l = 0;
                    while l < numpoints {
                        let v3 = *facepoints.add(l as usize);
                        let v4 = *facepoints.add(((l + 1) % numpoints) as usize);
                        dist = AAS_ClosestEdgePoints(
                            v1, v2, v3, v4, faceplane, plane, beststart, bestend, beststart2,
                            bestend2, bestdist,
                        );
                        if dist < bestdist {
                            bestfacenum = facenum;
                            bestfaceplane = faceplane;
                            bestdist = dist;
                        }
                        l += 1;
                    }
                    k += 1;
                }
                j += 1;
            }
            //
            if bestdist > 192.0 {
                i += 1;
                continue;
            }
            //
            beststart = [
                (beststart[0] + beststart2[0]) * 0.5,
                (beststart[1] + beststart2[1]) * 0.5,
                (beststart[2] + beststart2[2]) * 0.5,
            ];
            bestend = [
                (bestend[0] + bestend2[0]) * 0.5,
                (bestend[1] + bestend2[1]) * 0.5,
                (bestend[2] + bestend2[2]) * 0.5,
            ];
            //
            if towardsface == 0 {
                tmp = beststart;
                beststart = bestend;
                bestend = tmp;
            }
            //
            hordir = [
                bestend[0] - beststart[0],
                bestend[1] - beststart[1],
                bestend[2] - beststart[2],
            ];
            hordir[2] = 0.0;
            hordist = VectorLength(hordir);
            //
            if hordist > 2.0 * AAS_MaxJumpDistance(bot, bot.aassettings.phys_jumpvel) {
                i += 1;
                continue;
            }
            //the end point should not be significantly higher than the start point
            if bestend[2] - 32.0 > beststart[2] {
                i += 1;
                continue;
            }
            //don't fall down too far
            if bestend[2] < beststart[2] - 128.0 {
                i += 1;
                continue;
            }
            //the distance should not be too far
            if hordist > 32.0 {
                //check for walk off ledge
                if AAS_HorizontalVelocityForJump(bot, 0.0, beststart, bestend, &mut speed) == 0 {
                    i += 1;
                    continue;
                }
            }
            //
            beststart[2] += 1.0;
            bestend[2] += 1.0;
            //
            if towardsface != 0 {
                testpoint = bestend;
            } else {
                testpoint = beststart;
            }
            testpoint[2] = 0.0;
            testpoint[2] = ((*bestfaceplane).dist
                - ((*bestfaceplane).normal[0] * testpoint[0]
                    + (*bestfaceplane).normal[1] * testpoint[1]
                    + (*bestfaceplane).normal[2] * testpoint[2]))
                / (*bestfaceplane).normal[2];
            //
            if AAS_PointInsideFace(bot, bestfacenum, testpoint, 0.1) == 0 {
                //if the faces are not overlapping then only go down
                if bestend[2] - 16.0 > beststart[2] {
                    i += 1;
                    continue;
                }
            }
            let lreach = AAS_AllocReachability(bot);
            if lreach.is_null() {
                return lreachabilities;
            }
            (*lreach).areanum = i;
            (*lreach).facenum = 0;
            (*lreach).edgenum = 0;
            (*lreach).start = beststart;
            (*lreach).end = bestend;
            (*lreach).traveltype = 0;
            (*lreach).traveltime = 0;
            (*lreach).next = lreachabilities;
            lreachabilities = lreach;
            if towardsface != 0 {
                AAS_PermanentLine(bot, (*lreach).start, (*lreach).end, 1);
            } else {
                AAS_PermanentLine(bot, (*lreach).start, (*lreach).end, 2);
            }
            i += 1;
        }
        lreachabilities
    }
}

/// Raven `AAS_StoreReachability` — copy the loading link lists into the file arena.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:4290-4321`
pub fn AAS_StoreReachability(bot: &mut BotLib) {
    unsafe {
        if !bot.aasworld.reachability.is_null() {
            FreeMemory(bot, bot.aasworld.reachability as *mut ());
        }
        bot.aasworld.reachability = GetClearedMemory(
            bot,
            (bot.numlreachabilities + 10) as usize * core::mem::size_of::<aas_reachability_t>(),
        ) as *mut aas_reachability_t;
        bot.aasworld.reachabilitysize = 1;
        let mut i = 0;
        while i < bot.aasworld.numareas {
            let areasettings: *mut aas_areasettings_t = bot.aasworld.areasettings.add(i as usize);
            (*areasettings).firstreachablearea = bot.aasworld.reachabilitysize;
            (*areasettings).numreachableareas = 0;
            let mut lreach: *mut aas_lreachability_t = *bot.areareachability.add(i as usize);
            while !lreach.is_null() {
                let reach: *mut aas_reachability_t = bot.aasworld.reachability.add(
                    ((*areasettings).firstreachablearea + (*areasettings).numreachableareas)
                        as usize,
                );
                (*reach).areanum = (*lreach).areanum;
                (*reach).facenum = (*lreach).facenum;
                (*reach).edgenum = (*lreach).edgenum;
                (*reach).start = (*lreach).start;
                (*reach).end = (*lreach).end;
                (*reach).traveltype = (*lreach).traveltype;
                (*reach).traveltime = (*lreach).traveltime;
                //
                (*areasettings).numreachableareas += 1;
                lreach = (*lreach).next;
            }
            bot.aasworld.reachabilitysize += (*areasettings).numreachableareas;
            i += 1;
        }
    }
}

/// Raven `AAS_TravelFlagsForTeam`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:2706-2717`
pub fn AAS_TravelFlagsForTeam(bot: &mut BotLib, ent: c_int) -> c_int {
    let mut notteam: c_int = 0;

    if AAS_IntForBSPEpairKey(
        bot,
        ent,
        c"bot_notteam".as_ptr() as *mut c_char,
        &mut notteam,
    ) == 0
    {
        return 0;
    }
    if notteam == 1 {
        return TRAVELFLAG_NOTTEAM1;
    }
    if notteam == 2 {
        return TRAVELFLAG_NOTTEAM2;
    }
    0
}

/// Raven `AAS_GetJumpPadInfo` — resolve a trigger_push into a launch velocity.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:196-266`
pub fn AAS_GetJumpPadInfo(
    bot: &mut BotLib,
    ent: c_int,
    mut areastart: vec3_t,
    mut absmins: vec3_t,
    mut absmaxs: vec3_t,
    mut velocity: vec3_t,
) -> c_int {
    unsafe {
        let mut modelnum: c_int;
        let mut speed: f32 = 0.0;
        let mut origin: vec3_t = [0.0; 3];
        let mut angles: vec3_t;
        let mut teststart: vec3_t;
        let mut ent2origin: vec3_t = [0.0; 3];
        let trace: aas_trace_t;
        let mut model: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];
        let mut target: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];
        let mut targetname: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];

        //
        AAS_FloatForBSPEpairKey(bot, ent, c"speed".as_ptr() as *mut c_char, &mut speed);
        if speed == 0.0 {
            speed = 1000.0;
        }
        angles = [0.0; 3];
        //get the mins, maxs and origin of the model
        AAS_ValueForBSPEpairKey(
            bot,
            ent,
            c"model".as_ptr() as *mut c_char,
            model.as_mut_ptr(),
            MAX_EPAIRKEY,
        );
        if model[0] != 0 {
            modelnum = atoi(model.as_ptr().wrapping_add(1));
        } else {
            modelnum = 0;
        }
        AAS_BSPModelMinsMaxsOrigin(bot, modelnum, angles, absmins, absmaxs, origin);
        absmins = [
            origin[0] + absmins[0],
            origin[1] + absmins[1],
            origin[2] + absmins[2],
        ];
        absmaxs = [
            origin[0] + absmaxs[0],
            origin[1] + absmaxs[1],
            origin[2] + absmaxs[2],
        ];
        origin = [
            absmins[0] + absmaxs[0],
            absmins[1] + absmaxs[1],
            absmins[2] + absmaxs[2],
        ];
        origin = [origin[0] * 0.5, origin[1] * 0.5, origin[2] * 0.5];
        //get the start areas
        teststart = origin;
        teststart[2] += 64.0;
        trace = AAS_TraceClientBBox(bot, teststart, origin, PRESENCE_CROUCH, -1);
        if trace.startsolid != 0 {
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"trigger_push start solid\n".as_ptr() as *mut c_char,
            );
            areastart = origin;
        } else {
            areastart = trace.endpos;
        }
        areastart[2] += 0.125;
        //
        //get the target entity
        AAS_ValueForBSPEpairKey(
            bot,
            ent,
            c"target".as_ptr() as *mut c_char,
            target.as_mut_ptr(),
            MAX_EPAIRKEY,
        );
        let mut ent2 = AAS_NextBSPEntity(bot, 0);
        while ent2 != 0 {
            if AAS_ValueForBSPEpairKey(
                bot,
                ent2,
                c"targetname".as_ptr() as *mut c_char,
                targetname.as_mut_ptr(),
                MAX_EPAIRKEY,
            ) == 0
            {
                ent2 = AAS_NextBSPEntity(bot, ent2);
                continue;
            }
            if strcmp(targetname.as_ptr(), target.as_ptr()) == 0 {
                break;
            }
            ent2 = AAS_NextBSPEntity(bot, ent2);
        }
        if ent2 == 0 {
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"trigger_push without target entity %s\n".as_ptr() as *mut c_char,
                target.as_ptr(),
            );
            return qfalse;
        }
        AAS_VectorForBSPEpairKey(bot, ent2, c"origin".as_ptr() as *mut c_char, ent2origin);
        //
        let height = ent2origin[2] - origin[2];
        let gravity = bot.aassettings.phys_gravity;
        let time = ((height / (0.5 * gravity)) as f64).sqrt() as f32;
        if time == 0.0 {
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"trigger_push without time\n".as_ptr() as *mut c_char,
            );
            return qfalse;
        }
        // set s.origin2 to the push velocity
        velocity = [
            ent2origin[0] - origin[0],
            ent2origin[1] - origin[1],
            ent2origin[2] - origin[2],
        ];
        let dist = VectorNormalize(&mut velocity);
        let mut forward = dist / time;
        // Raven note: why multiply by 1.1
        forward *= 1.1;
        velocity = [
            velocity[0] * forward,
            velocity[1] * forward,
            velocity[2] * forward,
        ];
        velocity[2] = time * gravity;
        qtrue
    }
}

/// Raven `AAS_BestReachableArea`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:343-432`
pub fn AAS_BestReachableArea(
    bot: &mut BotLib,
    origin: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    mut goalorigin: vec3_t,
) -> c_int {
    unsafe {
        let mut areanum: c_int;
        let mut start: vec3_t;
        let mut end: vec3_t;
        let trace: aas_trace_t;

        if bot.aasworld.loaded == 0 {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"AAS_BestReachableArea: aas not loaded\n".as_ptr() as *mut c_char,
            );
            return 0;
        }
        //find a point in an area
        start = origin;
        areanum = AAS_PointAreaNum(bot, start);
        //while no area found fudge around a little
        let mut i = 0;
        while i < 5 && areanum == 0 {
            let mut j = 0;
            while j < 5 && areanum == 0 {
                let mut k = -1;
                while k <= 1 && areanum == 0 {
                    let mut l = -1;
                    while l <= 1 && areanum == 0 {
                        start = origin;
                        start[0] += (j * 4 * k) as f32;
                        start[1] += (j * 4 * l) as f32;
                        start[2] += (i * 4) as f32;
                        areanum = AAS_PointAreaNum(bot, start);
                        l += 1;
                    }
                    k += 1;
                }
                j += 1;
            }
            i += 1;
        }
        //if an area was found
        if areanum != 0 {
            //drop client bbox down and try again
            end = start;
            start[2] += 0.25;
            end[2] -= 50.0;
            trace = AAS_TraceClientBBox(bot, start, end, PRESENCE_CROUCH, -1);
            if trace.startsolid == 0 {
                areanum = AAS_PointAreaNum(bot, trace.endpos);
                goalorigin = trace.endpos;
                // Raven note: cannot enable next line right now because the reachability
                // does not have to be calculated when the level items are loaded
                if areanum != 0 {
                    return areanum;
                }
            } else {
                //it can very well happen that the AAS_PointAreaNum function tells that
                //a point is in an area and that starting a AAS_TraceClientBBox from that
                //point will return trace.startsolid qtrue
                goalorigin = start;
                return areanum;
            }
        }
        //
        //NOTE: the goal origin does not have to be in the goal area
        // because the bot will have to move towards the item origin anyway
        goalorigin = origin;
        //
        let absmins: vec3_t = [
            origin[0] + mins[0],
            origin[1] + mins[1],
            origin[2] + mins[2],
        ];
        let absmaxs: vec3_t = [
            origin[0] + maxs[0],
            origin[1] + maxs[1],
            origin[2] + maxs[2],
        ];
        //link an invalid (-1) entity
        let areas = AAS_LinkEntityClientBBox(bot, absmins, absmaxs, -1, PRESENCE_CROUCH);
        //get the reachable link arae
        areanum = AAS_BestReachableLinkArea(bot, areas);
        //unlink the invalid entity
        AAS_UnlinkFromAreas(bot, areas);
        //
        areanum
    }
}

/// Raven `AAS_SetWeaponJumpAreaFlags`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:3917-3972`
pub fn AAS_SetWeaponJumpAreaFlags(bot: &mut BotLib) {
    unsafe {
        let mins: vec3_t = [-15.0, -15.0, -15.0];
        let maxs: vec3_t = [15.0, 15.0, 15.0];
        let mut origin: vec3_t = [0.0; 3];
        let mut areanum: c_int;
        let mut spawnflags: c_int;
        let mut classname: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];

        let mut weaponjumpareas = 0;
        let mut ent = AAS_NextBSPEntity(bot, 0);
        while ent != 0 {
            if AAS_ValueForBSPEpairKey(
                bot,
                ent,
                c"classname".as_ptr() as *mut c_char,
                classname.as_mut_ptr(),
                MAX_EPAIRKEY,
            ) == 0
            {
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            if strcmp(classname.as_ptr(), c"item_armor_body".as_ptr()) == 0
                || strcmp(classname.as_ptr(), c"item_health".as_ptr()) == 0
                || strcmp(classname.as_ptr(), c"weapon_disruptor".as_ptr()) == 0
                || strcmp(classname.as_ptr(), c"weapon_repeater".as_ptr()) == 0
                || strcmp(classname.as_ptr(), c"weapon_demp2".as_ptr()) == 0
                || strcmp(classname.as_ptr(), c"weapon_flechette".as_ptr()) == 0
                || strcmp(classname.as_ptr(), c"weapon_rocket_launcher".as_ptr()) == 0
            {
                if AAS_VectorForBSPEpairKey(bot, ent, c"origin".as_ptr() as *mut c_char, origin)
                    != 0
                {
                    spawnflags = 0;
                    AAS_IntForBSPEpairKey(
                        bot,
                        ent,
                        c"spawnflags".as_ptr() as *mut c_char,
                        &mut spawnflags,
                    );
                    //if not a stationary item
                    if spawnflags & 1 == 0 {
                        if AAS_DropToFloor(bot, origin, mins, maxs) == 0 {
                            bot.botimport.Print.unwrap()(
                                PRT_MESSAGE,
                                c"%s in solid at (%1.1f %1.1f %1.1f)\n".as_ptr() as *mut c_char,
                                classname.as_ptr(),
                                origin[0] as f64,
                                origin[1] as f64,
                                origin[2] as f64,
                            );
                        }
                    }
                    areanum = AAS_BestReachableArea(bot, origin, mins, maxs, origin);
                    //the bot may rocket jump towards this area
                    (*bot.aasworld.areasettings.add(areanum as usize)).areaflags |= AREA_WEAPONJUMP;
                    //
                    weaponjumpareas += 1;
                }
            }
            ent = AAS_NextBSPEntity(bot, ent);
        }
        let mut i = 1;
        while i < bot.aasworld.numareas {
            if (*bot.aasworld.areasettings.add(i as usize)).contents & AREACONTENTS_JUMPPAD != 0 {
                (*bot.aasworld.areasettings.add(i as usize)).areaflags |= AREA_WEAPONJUMP;
                weaponjumpareas += 1;
            }
            i += 1;
        }
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"%d weapon jump areas\n".as_ptr() as *mut c_char,
            weaponjumpareas,
        );
    }
}

/// Raven `AAS_Reachability_Step_Barrier_WaterJump_WalkOffLedge`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:1042-1585`
pub fn AAS_Reachability_Step_Barrier_WaterJump_WalkOffLedge(
    bot: &mut BotLib,
    area1num: c_int,
    area2num: c_int,
) -> c_int {
    unsafe {
        let mut edge1num: c_int;
        let mut edge2num: c_int;
        let mut areas: [c_int; 10] = [0; 10];
        let numareas: c_int;
        let mut side1: c_int;
        let faceside1: c_int;
        let groundface1num: c_int;
        let mut dist: f32;
        let mut dist1: f32 = 0.0;
        let mut dist2: f32 = 0.0;
        let mut diff: f32;
        let mut x1: f32;
        let mut x2: f32;
        let mut x3: f32;
        let mut x4: f32;
        let mut y1: f32;
        let mut y2: f32;
        let mut y3: f32;
        let mut y4: f32;
        let mut tmp: f32;
        let mut y: f32;
        let mut length: f32;
        let mut normal: vec3_t = [0.0; 3];
        let mut ort: vec3_t = [0.0; 3];
        let mut edgevec: vec3_t;
        let mut start: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let mut dir: vec3_t;
        let mut v1: vec3_t;
        let mut v2: vec3_t;
        let mut v3: vec3_t;
        let mut v4: vec3_t;
        let mut tmpv: vec3_t;
        let mut p1area1: vec3_t = [0.0; 3];
        let mut p1area2: vec3_t = [0.0; 3];
        let mut p2area1: vec3_t = [0.0; 3];
        let mut p2area2: vec3_t = [0.0; 3];
        let mut ground_beststart: vec3_t = [0.0; 3];
        let mut ground_bestend: vec3_t = [0.0; 3];
        let mut ground_bestnormal: vec3_t = [0.0; 3];
        let mut water_beststart: vec3_t = [0.0; 3];
        let mut water_bestend: vec3_t = [0.0; 3];
        let mut water_bestnormal: vec3_t = [0.0; 3];
        let invgravity: vec3_t = [0.0, 0.0, 1.0];
        let mut testpoint: vec3_t;
        let mut plane: *mut aas_plane_t;
        let trace: aas_trace_t;

        //must be able to walk or swim in the first area
        if AAS_AreaGrounded(bot, area1num) == 0 && AAS_AreaSwim(bot, area1num) == 0 {
            return qfalse;
        }
        //
        if AAS_AreaGrounded(bot, area2num) == 0 && AAS_AreaSwim(bot, area2num) == 0 {
            return qfalse;
        }
        //
        let area1: *mut aas_area_t = bot.aasworld.areas.add(area1num as usize);
        let area2: *mut aas_area_t = bot.aasworld.areas.add(area2num as usize);
        //if the first area contains a liquid
        let area1swim = AAS_AreaSwim(bot, area1num);
        //if the areas are not near anough in the x-y direction
        let mut i = 0;
        while i < 2 {
            if (*area1).mins[i] > (*area2).maxs[i] + 10.0 {
                return qfalse;
            }
            if (*area1).maxs[i] < (*area2).mins[i] - 10.0 {
                return qfalse;
            }
            i += 1;
        }
        //
        let mut ground_foundreach = qfalse;
        let mut ground_bestdist: f32 = 99999.0;
        let mut ground_bestlength: f32 = 0.0;
        let mut ground_bestarea2groundedgenum: c_int = 0;
        //
        let mut water_foundreach = qfalse;
        let mut water_bestdist: f32 = 99999.0;
        let mut water_bestlength: f32 = 0.0;
        let mut water_bestarea2groundedgenum: c_int = 0;
        //
        let mut i = 0;
        while i < (*area1).numfaces {
            let groundface1num = *bot
                .aasworld
                .faceindex
                .add(((*area1).firstface + i) as usize);
            let faceside1 = (groundface1num < 0) as c_int;
            let groundface1: *mut aas_face_t = bot
                .aasworld
                .faces
                .add(groundface1num.unsigned_abs() as usize);
            //if this isn't a ground face
            if (*groundface1).faceflags & FACE_GROUND == 0 {
                //if we can swim in the first area
                if area1swim != 0 {
                    //face plane must be more or less horizontal
                    plane = bot.aasworld.planes.add(
                        ((*groundface1).planenum ^ (if faceside1 == 0 { 1 } else { 0 })) as usize,
                    );
                    if (*plane).normal[0] * invgravity[0]
                        + (*plane).normal[1] * invgravity[1]
                        + (*plane).normal[2] * invgravity[2]
                        < 0.7
                    {
                        i += 1;
                        continue;
                    }
                } else {
                    //if we can't swim in the area it must be a ground face
                    i += 1;
                    continue;
                }
            }
            //
            let mut k = 0;
            while k < (*groundface1).numedges {
                edge1num = *bot
                    .aasworld
                    .edgeindex
                    .add(((*groundface1).firstedge + k) as usize);
                side1 = (edge1num < 0) as c_int;
                //NOTE: for water faces we must take the side area 1 is
                // on into account because the face is shared and doesn't
                // have to be oriented correctly
                if (*groundface1).faceflags & FACE_GROUND == 0 {
                    side1 = (side1 == faceside1) as c_int;
                }
                edge1num = edge1num.abs();
                let edge1: *mut aas_edge_t = bot.aasworld.edges.add(edge1num as usize);
                //vertexes of the edge
                v1 = *bot
                    .aasworld
                    .vertexes
                    .add((*edge1).v[(1 - side1) as usize] as usize);
                v2 = *bot
                    .aasworld
                    .vertexes
                    .add((*edge1).v[side1 as usize] as usize);
                //get a vertical plane through the edge
                edgevec = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
                CrossProduct(edgevec, invgravity, &mut normal);
                VectorNormalize(&mut normal);
                let dist_n = normal[0] * v1[0] + normal[1] * v1[1] + normal[2] * v1[2];
                //check the faces from the second area
                let mut j = 0;
                while j < (*area2).numfaces {
                    let g2idx = *bot
                        .aasworld
                        .faceindex
                        .add(((*area2).firstface + j) as usize);
                    let groundface2: *mut aas_face_t =
                        bot.aasworld.faces.add(g2idx.unsigned_abs() as usize);
                    //must be a ground face
                    if (*groundface2).faceflags & FACE_GROUND == 0 {
                        j += 1;
                        continue;
                    }
                    //check the edges of this ground face
                    let mut l = 0;
                    while l < (*groundface2).numedges {
                        edge2num = (*bot
                            .aasworld
                            .edgeindex
                            .add(((*groundface2).firstedge + l) as usize))
                        .abs();
                        let edge2: *mut aas_edge_t = bot.aasworld.edges.add(edge2num as usize);
                        //vertexes of the edge
                        v3 = *bot.aasworld.vertexes.add((*edge2).v[0] as usize);
                        v4 = *bot.aasworld.vertexes.add((*edge2).v[1] as usize);
                        //check the distance between the two points and the vertical plane
                        diff = (normal[0] * v3[0] + normal[1] * v3[1] + normal[2] * v3[2]) - dist_n;
                        if diff < -0.1 || diff > 0.1 {
                            l += 1;
                            continue;
                        }
                        diff = (normal[0] * v4[0] + normal[1] * v4[1] + normal[2] * v4[2]) - dist_n;
                        if diff < -0.1 || diff > 0.1 {
                            l += 1;
                            continue;
                        }
                        //project the two ground edges into the step side plane
                        CrossProduct(invgravity, normal, &mut ort);
                        let _invgravitydot = invgravity[0] * invgravity[0]
                            + invgravity[1] * invgravity[1]
                            + invgravity[2] * invgravity[2];
                        let ortdot = ort[0] * ort[0] + ort[1] * ort[1] + ort[2] * ort[2];
                        //projection into the step plane
                        y1 = v1[2];
                        y2 = v2[2];
                        y3 = v3[2];
                        y4 = v4[2];
                        //
                        x1 = (v1[0] * ort[0] + v1[1] * ort[1] + v1[2] * ort[2]) / ortdot;
                        x2 = (v2[0] * ort[0] + v2[1] * ort[1] + v2[2] * ort[2]) / ortdot;
                        x3 = (v3[0] * ort[0] + v3[1] * ort[1] + v3[2] * ort[2]) / ortdot;
                        x4 = (v4[0] * ort[0] + v4[1] * ort[1] + v4[2] * ort[2]) / ortdot;
                        //
                        if x1 > x2 {
                            tmp = x1;
                            x1 = x2;
                            x2 = tmp;
                            tmp = y1;
                            y1 = y2;
                            y2 = tmp;
                            tmpv = v1;
                            v1 = v2;
                            v2 = tmpv;
                        }
                        if x3 > x4 {
                            tmp = x3;
                            x3 = x4;
                            x4 = tmp;
                            tmp = y3;
                            y3 = y4;
                            y4 = tmp;
                            tmpv = v3;
                            v3 = v4;
                            v4 = tmpv;
                        }
                        //if the two projected edge lines have no overlap
                        if x2 <= x3 || x4 <= x1 {
                            l += 1;
                            continue;
                        }
                        //if the two lines fully overlap
                        if (x1 - 0.5 < x3 && x4 < x2 + 0.5) && (x3 - 0.5 < x1 && x2 < x4 + 0.5) {
                            dist1 = y3 - y1;
                            dist2 = y4 - y2;
                            p1area1 = v1;
                            p2area1 = v2;
                            p1area2 = v3;
                            p2area2 = v4;
                        } else {
                            //if the points are equal
                            if x1 > x3 - 0.1 && x1 < x3 + 0.1 {
                                dist1 = y3 - y1;
                                p1area1 = v1;
                                p1area2 = v3;
                            } else if x1 < x3 {
                                y = y1 + (x3 - x1) * (y2 - y1) / (x2 - x1);
                                dist1 = y3 - y;
                                p1area1 = v3;
                                p1area1[2] = y;
                                p1area2 = v3;
                            } else {
                                y = y3 + (x1 - x3) * (y4 - y3) / (x4 - x3);
                                dist1 = y - y1;
                                p1area1 = v1;
                                p1area2 = v1;
                                p1area2[2] = y;
                            }
                            //if the points are equal
                            if x2 > x4 - 0.1 && x2 < x4 + 0.1 {
                                dist2 = y4 - y2;
                                p2area1 = v2;
                                p2area2 = v4;
                            } else if x2 < x4 {
                                y = y3 + (x2 - x3) * (y4 - y3) / (x4 - x3);
                                dist2 = y - y2;
                                p2area1 = v2;
                                p2area2 = v2;
                                p2area2[2] = y;
                            } else {
                                y = y1 + (x4 - x1) * (y2 - y1) / (x2 - x1);
                                dist2 = y4 - y;
                                p2area1 = v4;
                                p2area1[2] = y;
                                p2area2 = v4;
                            }
                        }
                        //if both distances are pretty much equal
                        //then we take the middle of the points
                        if dist1 > dist2 - 1.0 && dist1 < dist2 + 1.0 {
                            dist = dist1;
                            start = [
                                p1area1[0] + p2area1[0],
                                p1area1[1] + p2area1[1],
                                p1area1[2] + p2area1[2],
                            ];
                            start = [start[0] * 0.5, start[1] * 0.5, start[2] * 0.5];
                            end = [
                                p1area2[0] + p2area2[0],
                                p1area2[1] + p2area2[1],
                                p1area2[2] + p2area2[2],
                            ];
                            end = [end[0] * 0.5, end[1] * 0.5, end[2] * 0.5];
                        } else if dist1 < dist2 {
                            dist = dist1;
                            start = p1area1;
                            end = p1area2;
                        } else {
                            dist = dist2;
                            start = p2area1;
                            end = p2area2;
                        }
                        //get the length of the overlapping part of the edges of the two areas
                        dir = [
                            p2area2[0] - p1area2[0],
                            p2area2[1] - p1area2[1],
                            p2area2[2] - p1area2[2],
                        ];
                        length = VectorLength(dir);
                        //
                        if (*groundface1).faceflags & FACE_GROUND != 0 {
                            //if the vertical distance is smaller
                            if dist < ground_bestdist
                                || (dist < ground_bestdist + 1.0 && length > ground_bestlength)
                            {
                                ground_bestdist = dist;
                                ground_bestlength = length;
                                ground_foundreach = qtrue;
                                ground_bestarea2groundedgenum = edge1num;
                                //best point towards area1
                                ground_beststart = start;
                                //normal is pointing into area2
                                ground_bestnormal = normal;
                                //best point towards area2
                                ground_bestend = end;
                            }
                        } else {
                            //if the vertical distance is smaller
                            if dist < water_bestdist
                                || (dist < water_bestdist + 1.0 && length > water_bestlength)
                            {
                                water_bestdist = dist;
                                water_bestlength = length;
                                water_foundreach = qtrue;
                                water_bestarea2groundedgenum = edge1num;
                                //best point towards area1
                                water_beststart = start;
                                //normal is pointing into area2
                                water_bestnormal = normal;
                                //best point towards area2
                                water_bestend = end;
                            }
                        }
                        l += 1;
                    }
                    j += 1;
                }
                k += 1;
            }
            i += 1;
        }
        //
        // Steps
        //
        //check for a step reachability
        if ground_foundreach != 0 {
            //if area2 is higher but lower than the maximum step height
            if ground_bestdist >= 0.0 && ground_bestdist < bot.aassettings.phys_maxstep {
                //create walk reachability from area1 to area2
                let lreach = AAS_AllocReachability(bot);
                if lreach.is_null() {
                    return qfalse;
                }
                (*lreach).areanum = area2num;
                (*lreach).facenum = 0;
                (*lreach).edgenum = ground_bestarea2groundedgenum;
                (*lreach).start = [
                    ground_beststart[0] + ground_bestnormal[0] * INSIDEUNITS_WALKSTART,
                    ground_beststart[1] + ground_bestnormal[1] * INSIDEUNITS_WALKSTART,
                    ground_beststart[2] + ground_bestnormal[2] * INSIDEUNITS_WALKSTART,
                ];
                (*lreach).end = [
                    ground_bestend[0] + ground_bestnormal[0] * (INSIDEUNITS_WALKEND as f32),
                    ground_bestend[1] + ground_bestnormal[1] * (INSIDEUNITS_WALKEND as f32),
                    ground_bestend[2] + ground_bestnormal[2] * (INSIDEUNITS_WALKEND as f32),
                ];
                (*lreach).traveltype = TRAVEL_WALK;
                (*lreach).traveltime = 0; //1;
                                          //if going into a crouch area
                if AAS_AreaCrouch(bot, area1num) == 0 && AAS_AreaCrouch(bot, area2num) != 0 {
                    (*lreach).traveltime += bot.aassettings.rs_startcrouch as c_ushort;
                }
                (*lreach).next = *bot.areareachability.add(area1num as usize);
                *bot.areareachability.add(area1num as usize) = lreach;
                //
                bot.reach_step += 1;
                return qtrue;
            }
        }
        //
        // Water Jumps
        //
        //check for a waterjump reachability
        if water_foundreach != 0 {
            //get a test point a little bit towards area1
            testpoint = [
                water_bestend[0] + water_bestnormal[0] * ((-INSIDEUNITS) as f32),
                water_bestend[1] + water_bestnormal[1] * ((-INSIDEUNITS) as f32),
                water_bestend[2] + water_bestnormal[2] * ((-INSIDEUNITS) as f32),
            ];
            //go down the maximum waterjump height
            testpoint[2] -= bot.aassettings.phys_maxwaterjump;
            //if there IS water the sv_maxwaterjump height below the bestend point
            if (*bot
                .aasworld
                .areasettings
                .add(AAS_PointAreaNum(bot, testpoint) as usize))
            .areaflags
                & AREA_LIQUID
                != 0
            {
                //don't create rediculous water jump reachabilities from areas very far below
                //the water surface
                if water_bestdist < bot.aassettings.phys_maxwaterjump + 24.0 {
                    //waterjumping from or towards a crouch only area is not possible in Quake2
                    if (*bot.aasworld.areasettings.add(area1num as usize)).presencetype
                        & PRESENCE_NORMAL
                        != 0
                        && (*bot.aasworld.areasettings.add(area2num as usize)).presencetype
                            & PRESENCE_NORMAL
                            != 0
                    {
                        //create water jump reachability from area1 to area2
                        let lreach = AAS_AllocReachability(bot);
                        if lreach.is_null() {
                            return qfalse;
                        }
                        (*lreach).areanum = area2num;
                        (*lreach).facenum = 0;
                        (*lreach).edgenum = water_bestarea2groundedgenum;
                        (*lreach).start = water_beststart;
                        (*lreach).end = [
                            water_bestend[0] + water_bestnormal[0] * (INSIDEUNITS_WATERJUMP as f32),
                            water_bestend[1] + water_bestnormal[1] * (INSIDEUNITS_WATERJUMP as f32),
                            water_bestend[2] + water_bestnormal[2] * (INSIDEUNITS_WATERJUMP as f32),
                        ];
                        (*lreach).traveltype = TRAVEL_WATERJUMP;
                        (*lreach).traveltime = bot.aassettings.rs_waterjump as c_ushort;
                        (*lreach).next = *bot.areareachability.add(area1num as usize);
                        *bot.areareachability.add(area1num as usize) = lreach;
                        //we've got another waterjump reachability
                        bot.reach_waterjump += 1;
                        return qtrue;
                    }
                }
            }
        }
        //
        // Barrier Jumps
        //
        //check for a barrier jump reachability
        if ground_foundreach != 0 {
            //if area2 is higher but lower than the maximum barrier jump height
            if ground_bestdist > 0.0 && ground_bestdist < bot.aassettings.phys_maxbarrier {
                //if no water in area1 or a very thin layer of water on the ground
                if water_foundreach == 0 || (ground_bestdist - water_bestdist < 16.0) {
                    //cannot perform a barrier jump towards or from a crouch area in Quake2
                    if AAS_AreaCrouch(bot, area1num) == 0 && AAS_AreaCrouch(bot, area2num) == 0 {
                        //create barrier jump reachability from area1 to area2
                        let lreach = AAS_AllocReachability(bot);
                        if lreach.is_null() {
                            return qfalse;
                        }
                        (*lreach).areanum = area2num;
                        (*lreach).facenum = 0;
                        (*lreach).edgenum = ground_bestarea2groundedgenum;
                        (*lreach).start = [
                            ground_beststart[0] + ground_bestnormal[0] * INSIDEUNITS_WALKSTART,
                            ground_beststart[1] + ground_bestnormal[1] * INSIDEUNITS_WALKSTART,
                            ground_beststart[2] + ground_bestnormal[2] * INSIDEUNITS_WALKSTART,
                        ];
                        (*lreach).end = [
                            ground_bestend[0] + ground_bestnormal[0] * (INSIDEUNITS_WALKEND as f32),
                            ground_bestend[1] + ground_bestnormal[1] * (INSIDEUNITS_WALKEND as f32),
                            ground_bestend[2] + ground_bestnormal[2] * (INSIDEUNITS_WALKEND as f32),
                        ];
                        (*lreach).traveltype = TRAVEL_BARRIERJUMP;
                        (*lreach).traveltime = bot.aassettings.rs_barrierjump as c_ushort;
                        (*lreach).next = *bot.areareachability.add(area1num as usize);
                        *bot.areareachability.add(area1num as usize) = lreach;
                        //we've got another barrierjump reachability
                        bot.reach_barrier += 1;
                        return qtrue;
                    }
                }
            }
        }
        //
        // Walk and Walk Off Ledge
        //
        //check for a walk or walk off ledge reachability
        if ground_foundreach != 0 {
            if ground_bestdist < 0.0 {
                if ground_bestdist > -bot.aassettings.phys_maxstep {
                    //create walk reachability from area1 to area2
                    let lreach = AAS_AllocReachability(bot);
                    if lreach.is_null() {
                        return qfalse;
                    }
                    (*lreach).areanum = area2num;
                    (*lreach).facenum = 0;
                    (*lreach).edgenum = ground_bestarea2groundedgenum;
                    (*lreach).start = [
                        ground_beststart[0] + ground_bestnormal[0] * INSIDEUNITS_WALKSTART,
                        ground_beststart[1] + ground_bestnormal[1] * INSIDEUNITS_WALKSTART,
                        ground_beststart[2] + ground_bestnormal[2] * INSIDEUNITS_WALKSTART,
                    ];
                    (*lreach).end = [
                        ground_bestend[0] + ground_bestnormal[0] * (INSIDEUNITS_WALKEND as f32),
                        ground_bestend[1] + ground_bestnormal[1] * (INSIDEUNITS_WALKEND as f32),
                        ground_bestend[2] + ground_bestnormal[2] * (INSIDEUNITS_WALKEND as f32),
                    ];
                    (*lreach).traveltype = TRAVEL_WALK;
                    (*lreach).traveltime = 1;
                    (*lreach).next = *bot.areareachability.add(area1num as usize);
                    *bot.areareachability.add(area1num as usize) = lreach;
                    //we've got another walk reachability
                    bot.reach_walk += 1;
                    return qtrue;
                }
                // if no maximum fall height set or less than the max
                if bot.aassettings.rs_maxfallheight == 0.0
                    || ground_bestdist.abs() < bot.aassettings.rs_maxfallheight
                {
                    //trace a bounding box vertically to check for solids
                    ground_bestend = [
                        ground_bestend[0] + ground_bestnormal[0] * (INSIDEUNITS as f32),
                        ground_bestend[1] + ground_bestnormal[1] * (INSIDEUNITS as f32),
                        ground_bestend[2] + ground_bestnormal[2] * (INSIDEUNITS as f32),
                    ];
                    start = ground_bestend;
                    start[2] = ground_beststart[2];
                    end = ground_bestend;
                    end[2] += 4.0;
                    trace = AAS_TraceClientBBox(bot, start, end, PRESENCE_NORMAL, -1);
                    //if no solids were found
                    if trace.startsolid == 0 && trace.fraction >= 1.0 {
                        //the trace end point must be in the goal area
                        let mut endpos = trace.endpos;
                        endpos[2] += 1.0;
                        if AAS_PointAreaNum(bot, endpos) == area2num {
                            //if not going through a cluster portal
                            numareas = AAS_TraceAreas(
                                bot,
                                start,
                                end,
                                areas.as_mut_ptr(),
                                ptr::null_mut(),
                                (areas.len()) as c_int,
                            );
                            let mut ii = 0;
                            while ii < numareas {
                                if AAS_AreaClusterPortal(bot, areas[ii as usize]) != 0 {
                                    break;
                                }
                                ii += 1;
                            }
                            if ii >= numareas {
                                //create a walk off ledge reachability from area1 to area2
                                let lreach = AAS_AllocReachability(bot);
                                if lreach.is_null() {
                                    return qfalse;
                                }
                                (*lreach).areanum = area2num;
                                (*lreach).facenum = 0;
                                (*lreach).edgenum = ground_bestarea2groundedgenum;
                                (*lreach).start = ground_beststart;
                                (*lreach).end = ground_bestend;
                                (*lreach).traveltype = TRAVEL_WALKOFFLEDGE;
                                (*lreach).traveltime = (bot.aassettings.rs_startwalkoffledge
                                    + ground_bestdist.abs() * 50.0 / bot.aassettings.phys_gravity)
                                    as c_ushort;
                                //if falling from too high and not falling into water
                                if AAS_AreaSwim(bot, area2num) == 0
                                    && AAS_AreaJumpPad(bot, area2num) == 0
                                {
                                    if AAS_FallDelta(bot, ground_bestdist)
                                        > bot.aassettings.phys_falldelta5
                                    {
                                        (*lreach).traveltime +=
                                            bot.aassettings.rs_falldamage5 as c_ushort;
                                    }
                                    if AAS_FallDelta(bot, ground_bestdist)
                                        > bot.aassettings.phys_falldelta10
                                    {
                                        (*lreach).traveltime +=
                                            bot.aassettings.rs_falldamage10 as c_ushort;
                                    }
                                }
                                (*lreach).next = *bot.areareachability.add(area1num as usize);
                                *bot.areareachability.add(area1num as usize) = lreach;
                                //
                                bot.reach_walkoffledge += 1;
                                //NOTE: don't create a weapon (rl, bfg) jump reachability here
                                return qtrue;
                            }
                        }
                    }
                }
            }
        }
        qfalse
    }
}

/// Raven `AAS_Reachability_Ladder`.
///
/// PORT-NOTE(int-abs): Raven `abs(DotProduct(...))` truncates the float dot to
/// int before `abs` (a Raven quirk); ported as `(dot as c_int).abs()`.
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:2363-2699`
pub fn AAS_Reachability_Ladder(bot: &mut BotLib, area1num: c_int, mut area2num: c_int) -> c_int {
    unsafe {
        let mut edge1num: c_int;
        let mut edge2num: c_int;
        let mut sharededgenum: c_int;
        let mut lowestedgenum: c_int;
        let mut face1num: c_int;
        let mut face2num: c_int;
        let mut ladderface1num: c_int;
        let mut ladderface2num: c_int;
        let ladderface1vertical: c_int;
        let ladderface2vertical: c_int;
        let firstv: usize;
        let mut face1area: f32;
        let mut face2area: f32;
        let mut bestface1area: f32;
        let mut bestface2area: f32;
        let up: vec3_t = [0.0, 0.0, 1.0];
        let mut v1: vec3_t;
        let mut v2: vec3_t;
        let mut area1point: vec3_t = [0.0; 3];
        let mut area2point: vec3_t = [0.0; 3];
        let mut mid: vec3_t;
        let mut lowestpoint: vec3_t = [0.0; 3];
        let mut start: vec3_t;
        let mut end: vec3_t;
        let mut sharededgevec: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];
        let mut ladderface1: *mut aas_face_t;
        let mut ladderface2: *mut aas_face_t;
        let mut plane1: *mut aas_plane_t;
        let mut plane2: *mut aas_plane_t;
        let mut trace: aas_trace_t;

        if AAS_AreaLadder(bot, area1num) == 0 || AAS_AreaLadder(bot, area2num) == 0 {
            return qfalse;
        }
        //
        let phys_jumpvel = bot.aassettings.phys_jumpvel;
        //maximum height a player can jump with the given initial z velocity
        let maxjumpheight = AAS_MaxJumpHeight(bot, phys_jumpvel);

        let area1: *mut aas_area_t = bot.aasworld.areas.add(area1num as usize);
        let mut area2: *mut aas_area_t = bot.aasworld.areas.add(area2num as usize);
        //
        ladderface1 = ptr::null_mut();
        ladderface2 = ptr::null_mut();
        ladderface1num = 0; //make compiler happy
        ladderface2num = 0; //make compiler happy
        bestface1area = -9999.0;
        bestface2area = -9999.0;
        sharededgenum = 0; //make compiler happy
        lowestedgenum = 0; //make compiler happy
                           //
        let mut i = 0;
        while i < (*area1).numfaces {
            face1num = *bot
                .aasworld
                .faceindex
                .add(((*area1).firstface + i) as usize);
            let face1: *mut aas_face_t = bot.aasworld.faces.add(face1num.unsigned_abs() as usize);
            //if not a ladder face
            if (*face1).faceflags & FACE_LADDER == 0 {
                i += 1;
                continue;
            }
            //
            let mut j = 0;
            while j < (*area2).numfaces {
                face2num = *bot
                    .aasworld
                    .faceindex
                    .add(((*area2).firstface + j) as usize);
                let face2: *mut aas_face_t =
                    bot.aasworld.faces.add(face2num.unsigned_abs() as usize);
                //if not a ladder face
                if (*face2).faceflags & FACE_LADDER == 0 {
                    j += 1;
                    continue;
                }
                //check if the faces share an edge
                let mut k = 0;
                let mut l = 0;
                while k < (*face1).numedges {
                    edge1num = *bot
                        .aasworld
                        .edgeindex
                        .add(((*face1).firstedge + k) as usize);
                    l = 0;
                    while l < (*face2).numedges {
                        edge2num = *bot
                            .aasworld
                            .edgeindex
                            .add(((*face2).firstedge + l) as usize);
                        if edge1num.abs() == edge2num.abs() {
                            //get the face with the largest area
                            face1area = AAS_FaceArea(bot, face1);
                            face2area = AAS_FaceArea(bot, face2);
                            if face1area > bestface1area && face2area > bestface2area {
                                bestface1area = face1area;
                                bestface2area = face2area;
                                ladderface1 = face1;
                                ladderface2 = face2;
                                ladderface1num = face1num;
                                ladderface2num = face2num;
                                sharededgenum = edge1num;
                            }
                            break;
                        }
                        l += 1;
                    }
                    if l != (*face2).numedges {
                        break;
                    }
                    k += 1;
                }
                j += 1;
            }
            i += 1;
        }
        //
        if !ladderface1.is_null() && !ladderface2.is_null() {
            //get the middle of the shared edge
            let sharededge: *mut aas_edge_t = bot
                .aasworld
                .edges
                .add(sharededgenum.unsigned_abs() as usize);
            firstv = (sharededgenum < 0) as usize;
            //
            v1 = *bot.aasworld.vertexes.add((*sharededge).v[firstv] as usize);
            v2 = *bot
                .aasworld
                .vertexes
                .add((*sharededge).v[1 - firstv] as usize);
            area1point = [v1[0] + v2[0], v1[1] + v2[1], v1[2] + v2[2]];
            area1point = [
                area1point[0] * 0.5,
                area1point[1] * 0.5,
                area1point[2] * 0.5,
            ];
            area2point = area1point;
            //
            //if the face plane in area 1 is pretty much vertical
            plane1 = bot
                .aasworld
                .planes
                .add(((*ladderface1).planenum ^ (ladderface1num < 0) as c_int) as usize);
            plane2 = bot
                .aasworld
                .planes
                .add(((*ladderface2).planenum ^ (ladderface2num < 0) as c_int) as usize);
            //
            //get the points really into the areas
            sharededgevec = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
            CrossProduct((*plane1).normal, sharededgevec, &mut dir);
            VectorNormalize(&mut dir);
            //NOTE: 32 because that's larger than 16 (bot bbox x,y)
            area1point = [
                area1point[0] + dir[0] * -32.0,
                area1point[1] + dir[1] * -32.0,
                area1point[2] + dir[2] * -32.0,
            ];
            area2point = [
                area2point[0] + dir[0] * 32.0,
                area2point[1] + dir[1] * 32.0,
                area2point[2] + dir[2] * 32.0,
            ];
            //
            let ladderface1vertical = (((((*plane1).normal[0] * up[0]
                + (*plane1).normal[1] * up[1]
                + (*plane1).normal[2] * up[2]) as c_int)
                .abs() as f32)
                < 0.1) as c_int;
            let ladderface2vertical = (((((*plane2).normal[0] * up[0]
                + (*plane2).normal[1] * up[1]
                + (*plane2).normal[2] * up[2]) as c_int)
                .abs() as f32)
                < 0.1) as c_int;
            //there's only reachability between vertical ladder faces
            if ladderface1vertical == 0 && ladderface2vertical == 0 {
                return qfalse;
            }
            //if both vertical ladder faces
            if ladderface1vertical != 0
                && ladderface2vertical != 0
                && ((*plane1).normal[0] * (*plane2).normal[0]
                    + (*plane1).normal[1] * (*plane2).normal[1]
                    + (*plane1).normal[2] * (*plane2).normal[2])
                    > 0.7
                && (((sharededgevec[0] * up[0]
                    + sharededgevec[1] * up[1]
                    + sharededgevec[2] * up[2]) as c_int)
                    .abs() as f32)
                    < 0.7
            {
                //create a new reachability link
                let mut lreach = AAS_AllocReachability(bot);
                if lreach.is_null() {
                    return qfalse;
                }
                (*lreach).areanum = area2num;
                (*lreach).facenum = ladderface1num;
                (*lreach).edgenum = sharededgenum.abs();
                (*lreach).start = area1point;
                (*lreach).end = [
                    area2point[0] + (*plane1).normal[0] * -3.0,
                    area2point[1] + (*plane1).normal[1] * -3.0,
                    area2point[2] + (*plane1).normal[2] * -3.0,
                ];
                (*lreach).traveltype = TRAVEL_LADDER;
                (*lreach).traveltime = 10;
                (*lreach).next = *bot.areareachability.add(area1num as usize);
                *bot.areareachability.add(area1num as usize) = lreach;
                //
                bot.reach_ladder += 1;
                //create a new reachability link
                lreach = AAS_AllocReachability(bot);
                if lreach.is_null() {
                    return qfalse;
                }
                (*lreach).areanum = area1num;
                (*lreach).facenum = ladderface2num;
                (*lreach).edgenum = sharededgenum.abs();
                (*lreach).start = area2point;
                (*lreach).end = [
                    area1point[0] + (*plane1).normal[0] * -3.0,
                    area1point[1] + (*plane1).normal[1] * -3.0,
                    area1point[2] + (*plane1).normal[2] * -3.0,
                ];
                (*lreach).traveltype = TRAVEL_LADDER;
                (*lreach).traveltime = 10;
                (*lreach).next = *bot.areareachability.add(area2num as usize);
                *bot.areareachability.add(area2num as usize) = lreach;
                //
                bot.reach_ladder += 1;
                //
                return qtrue;
            }
            //if the second ladder face is also a ground face
            if ladderface1vertical != 0 && (*ladderface2).faceflags & FACE_GROUND != 0 {
                //create a new reachability link
                let mut lreach = AAS_AllocReachability(bot);
                if lreach.is_null() {
                    return qfalse;
                }
                (*lreach).areanum = area2num;
                (*lreach).facenum = ladderface1num;
                (*lreach).edgenum = sharededgenum.abs();
                (*lreach).start = area1point;
                (*lreach).end = area2point;
                (*lreach).end[2] += 16.0;
                (*lreach).end = [
                    (*lreach).end[0] + (*plane1).normal[0] * -15.0,
                    (*lreach).end[1] + (*plane1).normal[1] * -15.0,
                    (*lreach).end[2] + (*plane1).normal[2] * -15.0,
                ];
                (*lreach).traveltype = TRAVEL_LADDER;
                (*lreach).traveltime = 10;
                (*lreach).next = *bot.areareachability.add(area1num as usize);
                *bot.areareachability.add(area1num as usize) = lreach;
                //
                bot.reach_ladder += 1;
                //create a new reachability link
                lreach = AAS_AllocReachability(bot);
                if lreach.is_null() {
                    return qfalse;
                }
                (*lreach).areanum = area1num;
                (*lreach).facenum = ladderface2num;
                (*lreach).edgenum = sharededgenum.abs();
                (*lreach).start = area2point;
                (*lreach).end = area1point;
                (*lreach).traveltype = TRAVEL_WALKOFFLEDGE;
                (*lreach).traveltime = 10;
                (*lreach).next = *bot.areareachability.add(area2num as usize);
                *bot.areareachability.add(area2num as usize) = lreach;
                //
                bot.reach_walkoffledge += 1;
                //
                return qtrue;
            }
            //
            if ladderface1vertical != 0 {
                //find lowest edge of the ladder face
                lowestpoint[2] = 99999.0;
                let mut i = 0;
                while i < (*ladderface1).numedges {
                    edge1num = (*bot
                        .aasworld
                        .edgeindex
                        .add(((*ladderface1).firstedge + i) as usize))
                    .abs();
                    let edge1: *mut aas_edge_t = bot.aasworld.edges.add(edge1num as usize);
                    //
                    v1 = *bot.aasworld.vertexes.add((*edge1).v[0] as usize);
                    v2 = *bot.aasworld.vertexes.add((*edge1).v[1] as usize);
                    //
                    mid = [v1[0] + v2[0], v1[1] + v2[1], v1[2] + v2[2]];
                    mid = [mid[0] * 0.5, mid[1] * 0.5, mid[2] * 0.5];
                    //
                    if mid[2] < lowestpoint[2] {
                        lowestpoint = mid;
                        lowestedgenum = edge1num;
                    }
                    i += 1;
                }
                //
                plane1 = bot.aasworld.planes.add((*ladderface1).planenum as usize);
                //trace down in the middle of this edge
                start = [
                    lowestpoint[0] + (*plane1).normal[0] * 5.0,
                    lowestpoint[1] + (*plane1).normal[1] * 5.0,
                    lowestpoint[2] + (*plane1).normal[2] * 5.0,
                ];
                end = start;
                start[2] += 5.0;
                end[2] -= 100.0;
                //trace without entity collision
                trace = AAS_TraceClientBBox(bot, start, end, PRESENCE_NORMAL, -1);
                //
                trace.endpos[2] += 1.0;
                area2num = AAS_PointAreaNum(bot, trace.endpos);
                //
                area2 = bot.aasworld.areas.add(area2num as usize);
                let mut i = 0;
                while i < (*area2).numfaces {
                    face2num = *bot
                        .aasworld
                        .faceindex
                        .add(((*area2).firstface + i) as usize);
                    let face2: *mut aas_face_t =
                        bot.aasworld.faces.add(face2num.unsigned_abs() as usize);
                    //
                    if (*face2).faceflags & FACE_LADDER != 0 {
                        plane2 = bot.aasworld.planes.add((*face2).planenum as usize);
                        if (((((*plane2).normal[0] * up[0]
                            + (*plane2).normal[1] * up[1]
                            + (*plane2).normal[2] * up[2]) as c_int)
                            .abs() as f32)
                            < 0.1)
                        {
                            break;
                        }
                    }
                    i += 1;
                }
                //if from another area without vertical ladder faces
                if i >= (*area2).numfaces
                    && area2num != area1num
                    && AAS_ReachabilityExists(bot, area1num, area2num) == 0
                    && AAS_ReachabilityExists(bot, area2num, area1num) == 0
                {
                    //if the height is jumpable
                    if start[2] - trace.endpos[2] < maxjumpheight {
                        //create a new reachability link
                        let mut lreach = AAS_AllocReachability(bot);
                        if lreach.is_null() {
                            return qfalse;
                        }
                        (*lreach).areanum = area2num;
                        (*lreach).facenum = ladderface1num;
                        (*lreach).edgenum = lowestedgenum;
                        (*lreach).start = lowestpoint;
                        (*lreach).end = trace.endpos;
                        (*lreach).traveltype = TRAVEL_LADDER;
                        (*lreach).traveltime = 10;
                        (*lreach).next = *bot.areareachability.add(area1num as usize);
                        *bot.areareachability.add(area1num as usize) = lreach;
                        //
                        bot.reach_ladder += 1;
                        //create a new reachability link
                        lreach = AAS_AllocReachability(bot);
                        if lreach.is_null() {
                            return qfalse;
                        }
                        (*lreach).areanum = area1num;
                        (*lreach).facenum = ladderface1num;
                        (*lreach).edgenum = lowestedgenum;
                        (*lreach).start = trace.endpos;
                        //get the end point a little bit into the ladder
                        (*lreach).end = [
                            lowestpoint[0] + (*plane1).normal[0] * -5.0,
                            lowestpoint[1] + (*plane1).normal[1] * -5.0,
                            lowestpoint[2] + (*plane1).normal[2] * -5.0,
                        ];
                        //get the end point a little higher
                        (*lreach).end[2] += 10.0;
                        (*lreach).traveltype = TRAVEL_JUMP;
                        (*lreach).traveltime = 10;
                        (*lreach).next = *bot.areareachability.add(area2num as usize);
                        *bot.areareachability.add(area2num as usize) = lreach;
                        //
                        bot.reach_jump += 1;
                        //
                        return qtrue;
                    }
                }
            }
        }
        qfalse
    }
}

/// Raven `AAS_Reachability_Elevator` — func_plat reachabilities.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:2930-3138`
pub fn AAS_Reachability_Elevator(bot: &mut BotLib) {
    unsafe {
        let mut area1num: c_int;
        let mut area2num: c_int;
        let modelnum: c_int;
        let mut lip: f32 = 0.0;
        let mut height: f32 = 0.0;
        let mut speed: f32 = 0.0;
        let mut model: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];
        let mut classname: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut origin: vec3_t = [0.0; 3];
        let angles: vec3_t = [0.0, 0.0, 0.0];
        let mut pos1: vec3_t;
        let mut pos2: vec3_t;
        let mut mids: vec3_t;
        let mut platbottom: vec3_t = [0.0; 3];
        let mut plattop: vec3_t = [0.0; 3];
        let mut bottomorg: vec3_t = [0.0; 3];
        let mut toporg: vec3_t = [0.0; 3];
        let mut start: vec3_t;
        let mut end: vec3_t;
        let mut dir: vec3_t = [0.0; 3];
        let mut xvals: [vec_t; 8] = [0.0; 8];
        let mut yvals: [vec_t; 8] = [0.0; 8];
        let mut xvals_top: [vec_t; 8] = [0.0; 8];
        let mut yvals_top: [vec_t; 8] = [0.0; 8];
        let mut trace: aas_trace_t;

        let mut ent = AAS_NextBSPEntity(bot, 0);
        while ent != 0 {
            if AAS_ValueForBSPEpairKey(
                bot,
                ent,
                c"classname".as_ptr() as *mut c_char,
                classname.as_mut_ptr(),
                MAX_EPAIRKEY,
            ) == 0
            {
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            if strcmp(classname.as_ptr(), c"func_plat".as_ptr()) == 0 {
                if AAS_ValueForBSPEpairKey(
                    bot,
                    ent,
                    c"model".as_ptr() as *mut c_char,
                    model.as_mut_ptr(),
                    MAX_EPAIRKEY,
                ) == 0
                {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"func_plat without model\n".as_ptr() as *mut c_char,
                    );
                    ent = AAS_NextBSPEntity(bot, ent);
                    continue;
                }
                //get the model number, and skip the leading *
                modelnum = atoi(model.as_ptr().wrapping_add(1));
                if modelnum <= 0 {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"func_plat with invalid model number\n".as_ptr() as *mut c_char,
                    );
                    ent = AAS_NextBSPEntity(bot, ent);
                    continue;
                }
                //get the mins, maxs and origin of the model
                AAS_BSPModelMinsMaxsOrigin(bot, modelnum, angles, mins, maxs, origin);
                //
                AAS_VectorForBSPEpairKey(bot, ent, c"origin".as_ptr() as *mut c_char, origin);
                //pos1 is the top position, pos2 is the bottom
                pos1 = origin;
                pos2 = origin;
                //get the lip of the plat
                AAS_FloatForBSPEpairKey(bot, ent, c"lip".as_ptr() as *mut c_char, &mut lip);
                if lip == 0.0 {
                    lip = 8.0;
                }
                //get the movement height of the plat
                AAS_FloatForBSPEpairKey(bot, ent, c"height".as_ptr() as *mut c_char, &mut height);
                if height == 0.0 {
                    height = (maxs[2] - mins[2]) - lip;
                }
                //get the speed of the plat
                AAS_FloatForBSPEpairKey(bot, ent, c"speed".as_ptr() as *mut c_char, &mut speed);
                if speed == 0.0 {
                    speed = 200.0;
                }
                //get bottom position below pos1
                pos2[2] -= height;
                //
                //get a point just above the plat in the bottom position
                mids = [mins[0] + maxs[0], mins[1] + maxs[1], mins[2] + maxs[2]];
                platbottom = [
                    pos2[0] + mids[0] * 0.5,
                    pos2[1] + mids[1] * 0.5,
                    pos2[2] + mids[2] * 0.5,
                ];
                platbottom[2] = maxs[2] - (pos1[2] - pos2[2]) + 2.0;
                //get a point just above the plat in the top position
                mids = [mins[0] + maxs[0], mins[1] + maxs[1], mins[2] + maxs[2]];
                plattop = [
                    pos2[0] + mids[0] * 0.5,
                    pos2[1] + mids[1] * 0.5,
                    pos2[2] + mids[2] * 0.5,
                ];
                plattop[2] = maxs[2] + 2.0;
                //
                //get the mins and maxs a little larger
                let mut i = 0;
                while i < 3 {
                    mins[i] -= 1.0;
                    maxs[i] += 1.0;
                    i += 1;
                }
                //
                mids = [mins[0] + maxs[0], mins[1] + maxs[1], mins[2] + maxs[2]];
                mids = [mids[0] * 0.5, mids[1] * 0.5, mids[2] * 0.5];
                //
                xvals[0] = mins[0];
                xvals[1] = mids[0];
                xvals[2] = maxs[0];
                xvals[3] = mids[0];
                yvals[0] = mids[1];
                yvals[1] = maxs[1];
                yvals[2] = mids[1];
                yvals[3] = mins[1];
                //
                xvals[4] = mins[0];
                xvals[5] = maxs[0];
                xvals[6] = maxs[0];
                xvals[7] = mins[0];
                yvals[4] = maxs[1];
                yvals[5] = maxs[1];
                yvals[6] = mins[1];
                yvals[7] = mins[1];
                //find adjacent areas around the bottom of the plat
                let mut i = 0;
                while i < 9 {
                    if i < 8 {
                        //check at the sides of the plat
                        bottomorg[0] = origin[0] + xvals[i as usize];
                        bottomorg[1] = origin[1] + yvals[i as usize];
                        bottomorg[2] = platbottom[2] + 16.0;
                        //get a grounded or swim area near the plat in the bottom position
                        area1num = AAS_PointAreaNum(bot, bottomorg);
                        let mut k = 0;
                        while k < 16 {
                            if area1num != 0 {
                                if AAS_AreaGrounded(bot, area1num) != 0
                                    || AAS_AreaSwim(bot, area1num) != 0
                                {
                                    break;
                                }
                            }
                            bottomorg[2] += 4.0;
                            area1num = AAS_PointAreaNum(bot, bottomorg);
                            k += 1;
                        }
                        //if in solid
                        if k >= 16 {
                            i += 1;
                            continue;
                        }
                    } else {
                        //at the middle of the plat
                        bottomorg = plattop;
                        bottomorg[2] += 24.0;
                        area1num = AAS_PointAreaNum(bot, bottomorg);
                        if area1num == 0 {
                            i += 1;
                            continue;
                        }
                        bottomorg = platbottom;
                        bottomorg[2] += 24.0;
                    }
                    //look at adjacent areas around the top of the plat
                    let mut n = 0;
                    while n < 3 {
                        let mut k = 0;
                        while k < 3 {
                            mins[k] -= 4.0;
                            maxs[k] += 4.0;
                            k += 1;
                        }
                        xvals_top[0] = mins[0];
                        xvals_top[1] = mids[0];
                        xvals_top[2] = maxs[0];
                        xvals_top[3] = mids[0];
                        yvals_top[0] = mids[1];
                        yvals_top[1] = maxs[1];
                        yvals_top[2] = mids[1];
                        yvals_top[3] = mins[1];
                        //
                        xvals_top[4] = mins[0];
                        xvals_top[5] = maxs[0];
                        xvals_top[6] = maxs[0];
                        xvals_top[7] = mins[0];
                        yvals_top[4] = maxs[1];
                        yvals_top[5] = maxs[1];
                        yvals_top[6] = mins[1];
                        yvals_top[7] = mins[1];
                        //
                        let mut j = 0;
                        while j < 8 {
                            toporg[0] = origin[0] + xvals_top[j as usize];
                            toporg[1] = origin[1] + yvals_top[j as usize];
                            toporg[2] = plattop[2] + 16.0;
                            //get a grounded or swim area near the plat in the top position
                            area2num = AAS_PointAreaNum(bot, toporg);
                            let mut l = 0;
                            while l < 16 {
                                if area2num != 0 {
                                    if AAS_AreaGrounded(bot, area2num) != 0
                                        || AAS_AreaSwim(bot, area2num) != 0
                                    {
                                        start = plattop;
                                        start[2] += 32.0;
                                        end = toporg;
                                        end[2] += 1.0;
                                        trace = AAS_TraceClientBBox(
                                            bot,
                                            start,
                                            end,
                                            PRESENCE_CROUCH,
                                            -1,
                                        );
                                        if trace.fraction >= 1.0 {
                                            break;
                                        }
                                    }
                                }
                                toporg[2] += 4.0;
                                area2num = AAS_PointAreaNum(bot, toporg);
                                l += 1;
                            }
                            //if in solid
                            if l >= 16 {
                                j += 1;
                                continue;
                            }
                            //never create a reachability in the same area
                            if area2num == area1num {
                                j += 1;
                                continue;
                            }
                            //if the area isn't grounded
                            if AAS_AreaGrounded(bot, area2num) == 0 {
                                j += 1;
                                continue;
                            }
                            //if there already exists reachability between the areas
                            if AAS_ReachabilityExists(bot, area1num, area2num) != 0 {
                                j += 1;
                                continue;
                            }
                            //if the reachability start is within the elevator bounding box
                            dir = [
                                bottomorg[0] - platbottom[0],
                                bottomorg[1] - platbottom[1],
                                bottomorg[2] - platbottom[2],
                            ];
                            VectorNormalize(&mut dir);
                            dir[0] = bottomorg[0] + 24.0 * dir[0];
                            dir[1] = bottomorg[1] + 24.0 * dir[1];
                            dir[2] = bottomorg[2];
                            //
                            let mut p = 0;
                            while p < 3 {
                                if dir[p] < origin[p] + mins[p] || dir[p] > origin[p] + maxs[p] {
                                    break;
                                }
                                p += 1;
                            }
                            if p >= 3 {
                                j += 1;
                                continue;
                            }
                            //create a new reachability link
                            let lreach = AAS_AllocReachability(bot);
                            if lreach.is_null() {
                                j += 1;
                                continue;
                            }
                            (*lreach).areanum = area2num;
                            //the facenum is the model number
                            (*lreach).facenum = modelnum;
                            //the edgenum is the height
                            (*lreach).edgenum = height as c_int;
                            //
                            (*lreach).start = dir;
                            (*lreach).end = toporg;
                            (*lreach).traveltype = TRAVEL_ELEVATOR;
                            (*lreach).traveltype |= AAS_TravelFlagsForTeam(bot, ent);
                            (*lreach).traveltime = (bot.aassettings.rs_startelevator
                                + height * 100.0 / speed)
                                as c_ushort;
                            (*lreach).next = *bot.areareachability.add(area1num as usize);
                            *bot.areareachability.add(area1num as usize) = lreach;
                            //don't go any further to the outside
                            n = 9999;
                            //
                            bot.reach_elevator += 1;
                            j += 1;
                        }
                        n += 1;
                    }
                    i += 1;
                }
            }
            ent = AAS_NextBSPEntity(bot, ent);
        }
    }
}

/// Raven `AAS_Reachability_FuncBobbing`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:3268-3475`
pub fn AAS_Reachability_FuncBobbing(bot: &mut BotLib) {
    unsafe {
        let mut spawnflags: c_int = 0;
        let modelnum: c_int;
        let axis: usize;
        let mut numareas: c_int;
        let mut areas: [c_int; 10] = [0; 10];
        let mut classname: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];
        let mut model: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];
        let mut origin: vec3_t = [0.0; 3];
        let mut move_end: vec3_t;
        let mut move_start: vec3_t;
        let mut move_start_top: vec3_t;
        let mut move_end_top: vec3_t;
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let angles: vec3_t = [0.0, 0.0, 0.0];
        let mut start_edgeverts: [vec3_t; 4] = [[0.0; 3]; 4];
        let mut end_edgeverts: [vec3_t; 4] = [[0.0; 3]; 4];
        let mut mid: vec3_t;
        let mut org: vec3_t;
        let mut start: vec3_t;
        let mut end: vec3_t;
        let mut dir: vec3_t = [0.0; 3];
        let mut points: [vec3_t; 10] = [[0.0; 3]; 10];
        let mut height: f32 = 0.0;
        let mut start_plane: aas_plane_t = core::mem::zeroed();
        let mut end_plane: aas_plane_t = core::mem::zeroed();

        let mut ent = AAS_NextBSPEntity(bot, 0);
        while ent != 0 {
            if AAS_ValueForBSPEpairKey(
                bot,
                ent,
                c"classname".as_ptr() as *mut c_char,
                classname.as_mut_ptr(),
                MAX_EPAIRKEY,
            ) == 0
            {
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            if strcmp(classname.as_ptr(), c"func_bobbing".as_ptr()) != 0 {
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            AAS_FloatForBSPEpairKey(bot, ent, c"height".as_ptr() as *mut c_char, &mut height);
            if height == 0.0 {
                height = 32.0;
            }
            //
            if AAS_ValueForBSPEpairKey(
                bot,
                ent,
                c"model".as_ptr() as *mut c_char,
                model.as_mut_ptr(),
                MAX_EPAIRKEY,
            ) == 0
            {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    c"func_bobbing without model\n".as_ptr() as *mut c_char,
                );
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            //get the model number, and skip the leading *
            modelnum = atoi(model.as_ptr().wrapping_add(1));
            if modelnum <= 0 {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    c"func_bobbing with invalid model number\n".as_ptr() as *mut c_char,
                );
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            //if the entity has an origin set then use it
            if AAS_VectorForBSPEpairKey(bot, ent, c"origin".as_ptr() as *mut c_char, origin) == 0 {
                origin = [0.0, 0.0, 0.0];
            }
            //
            AAS_BSPModelMinsMaxsOrigin(bot, modelnum, angles, mins, maxs, ptr::null_mut());
            //
            mins = [
                mins[0] + origin[0],
                mins[1] + origin[1],
                mins[2] + origin[2],
            ];
            maxs = [
                maxs[0] + origin[0],
                maxs[1] + origin[1],
                maxs[2] + origin[2],
            ];
            //
            mid = [mins[0] + maxs[0], mins[1] + maxs[1], mins[2] + maxs[2]];
            mid = [mid[0] * 0.5, mid[1] * 0.5, mid[2] * 0.5];
            origin = mid;
            //
            move_end = origin;
            move_start = origin;
            //
            AAS_IntForBSPEpairKey(
                bot,
                ent,
                c"spawnflags".as_ptr() as *mut c_char,
                &mut spawnflags,
            );
            // set the axis of bobbing
            if spawnflags & 1 != 0 {
                axis = 0;
            } else if spawnflags & 2 != 0 {
                axis = 1;
            } else {
                axis = 2;
            }
            //
            move_start[axis] -= height;
            move_end[axis] += height;
            //
            Log_Write(
                bot,
                c"funcbob model %d, start = {%1.1f, %1.1f, %1.1f} end = {%1.1f, %1.1f, %1.1f}\n"
                    .as_ptr() as *mut c_char,
                modelnum,
                move_start[0] as f64,
                move_start[1] as f64,
                move_start[2] as f64,
                move_end[0] as f64,
                move_end[1] as f64,
                move_end[2] as f64,
            );
            //
            let mut i = 0;
            while i < 4 {
                start_edgeverts[i] = move_start;
                start_edgeverts[i][2] += maxs[2] - mid[2]; //+ bbox maxs z
                start_edgeverts[i][2] += 24.0; //+ player origin to ground dist
                i += 1;
            }
            start_edgeverts[0][0] += maxs[0] - mid[0];
            start_edgeverts[0][1] += maxs[1] - mid[1];
            start_edgeverts[1][0] += maxs[0] - mid[0];
            start_edgeverts[1][1] += mins[1] - mid[1];
            start_edgeverts[2][0] += mins[0] - mid[0];
            start_edgeverts[2][1] += mins[1] - mid[1];
            start_edgeverts[3][0] += mins[0] - mid[0];
            start_edgeverts[3][1] += maxs[1] - mid[1];
            //
            start_plane.dist = start_edgeverts[0][2];
            start_plane.normal = [0.0, 0.0, 1.0];
            //
            let mut i = 0;
            while i < 4 {
                end_edgeverts[i] = move_end;
                end_edgeverts[i][2] += maxs[2] - mid[2]; //+ bbox maxs z
                end_edgeverts[i][2] += 24.0; //+ player origin to ground dist
                i += 1;
            }
            end_edgeverts[0][0] += maxs[0] - mid[0];
            end_edgeverts[0][1] += maxs[1] - mid[1];
            end_edgeverts[1][0] += maxs[0] - mid[0];
            end_edgeverts[1][1] += mins[1] - mid[1];
            end_edgeverts[2][0] += mins[0] - mid[0];
            end_edgeverts[2][1] += mins[1] - mid[1];
            end_edgeverts[3][0] += mins[0] - mid[0];
            end_edgeverts[3][1] += maxs[1] - mid[1];
            //
            end_plane.dist = end_edgeverts[0][2];
            end_plane.normal = [0.0, 0.0, 1.0];
            //
            move_start_top = move_start;
            move_start_top[2] += maxs[2] - mid[2] + 24.0; //+ bbox maxs z
            move_end_top = move_end;
            move_end_top[2] += maxs[2] - mid[2] + 24.0; //+ bbox maxs z
                                                        //
            if AAS_PointAreaNum(bot, move_start_top) == 0 {
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            if AAS_PointAreaNum(bot, move_end_top) == 0 {
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            //
            let mut i = 0;
            while i < 2 {
                let firststartreach: *mut aas_lreachability_t;
                let firstendreach: *mut aas_lreachability_t;
                //
                if i == 0 {
                    firststartreach = AAS_FindFaceReachabilities(
                        bot,
                        start_edgeverts.as_mut_ptr(),
                        4,
                        &mut start_plane,
                        qtrue,
                    );
                    firstendreach = AAS_FindFaceReachabilities(
                        bot,
                        end_edgeverts.as_mut_ptr(),
                        4,
                        &mut end_plane,
                        qfalse,
                    );
                } else {
                    firststartreach = AAS_FindFaceReachabilities(
                        bot,
                        end_edgeverts.as_mut_ptr(),
                        4,
                        &mut end_plane,
                        qtrue,
                    );
                    firstendreach = AAS_FindFaceReachabilities(
                        bot,
                        start_edgeverts.as_mut_ptr(),
                        4,
                        &mut start_plane,
                        qfalse,
                    );
                }
                //
                //create reachabilities from start to end
                let mut startreach = firststartreach;
                while !startreach.is_null() {
                    let nextstartreach = (*startreach).next;
                    //
                    let mut endreach = firstendreach;
                    while !endreach.is_null() {
                        let nextendreach = (*endreach).next;
                        //
                        Log_Write(
                            bot,
                            c"funcbob reach from area %d to %d\n".as_ptr() as *mut c_char,
                            (*startreach).areanum,
                            (*endreach).areanum,
                        );
                        //
                        if i == 0 {
                            org = move_start_top;
                        } else {
                            org = move_end_top;
                        }
                        dir = [
                            (*startreach).start[0] - org[0],
                            (*startreach).start[1] - org[1],
                            (*startreach).start[2] - org[2],
                        ];
                        dir[2] = 0.0;
                        VectorNormalize(&mut dir);
                        start = (*startreach).start;
                        start = [
                            (*startreach).start[0] + dir[0],
                            (*startreach).start[1] + dir[1],
                            (*startreach).start[2] + dir[2],
                        ];
                        start[2] += 1.0;
                        end = [
                            (*startreach).start[0] + dir[0] * 16.0,
                            (*startreach).start[1] + dir[1] * 16.0,
                            (*startreach).start[2] + dir[2] * 16.0,
                        ];
                        end[2] += 1.0;
                        //
                        numareas = AAS_TraceAreas(
                            bot,
                            start,
                            end,
                            areas.as_mut_ptr(),
                            points.as_mut_ptr(),
                            10,
                        );
                        if numareas <= 0 {
                            endreach = nextendreach;
                            continue;
                        }
                        if numareas > 1 {
                            (*startreach).start = points[1];
                        } else {
                            (*startreach).start = end;
                        }
                        //
                        if AAS_PointAreaNum(bot, (*startreach).start) == 0 {
                            endreach = nextendreach;
                            continue;
                        }
                        if AAS_PointAreaNum(bot, (*endreach).end) == 0 {
                            endreach = nextendreach;
                            continue;
                        }
                        //
                        let lreach = AAS_AllocReachability(bot);
                        (*lreach).areanum = (*endreach).areanum;
                        if i == 0 {
                            (*lreach).edgenum = ((move_start[axis] as c_int) << 16)
                                | ((move_end[axis] as c_int) & 0x0000ffff);
                        } else {
                            (*lreach).edgenum = ((move_end[axis] as c_int) << 16)
                                | ((move_start[axis] as c_int) & 0x0000ffff);
                        }
                        (*lreach).facenum = (spawnflags << 16) | modelnum;
                        (*lreach).start = (*startreach).start;
                        (*lreach).end = (*endreach).end;
                        (*lreach).traveltype = TRAVEL_FUNCBOB;
                        (*lreach).traveltype |= AAS_TravelFlagsForTeam(bot, ent);
                        (*lreach).traveltime = bot.aassettings.rs_funcbob as c_ushort;
                        bot.reach_funcbob += 1;
                        (*lreach).next = *bot.areareachability.add((*startreach).areanum as usize);
                        *bot.areareachability.add((*startreach).areanum as usize) = lreach;
                        //
                        endreach = nextendreach;
                    }
                    startreach = nextstartreach;
                }
                let mut startreach = firststartreach;
                while !startreach.is_null() {
                    let nextstartreach = (*startreach).next;
                    AAS_FreeReachability(bot, startreach);
                    startreach = nextstartreach;
                }
                let mut endreach = firstendreach;
                while !endreach.is_null() {
                    let nextendreach = (*endreach).next;
                    AAS_FreeReachability(bot, endreach);
                    endreach = nextendreach;
                }
                //only go up with func_bobbing entities that go up and down
                if spawnflags & 1 == 0 && spawnflags & 2 == 0 {
                    break;
                }
                i += 1;
            }
            ent = AAS_NextBSPEntity(bot, ent);
        }
    }
}

/// Raven `AAS_Reachability_Grapple`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:3776-3910`
pub fn AAS_Reachability_Grapple(bot: &mut BotLib, area1num: c_int, area2num: c_int) -> c_int {
    unsafe {
        let mut face2num: c_int;
        let mut areanum: c_int;
        let mut numareas: c_int;
        let mut areas: [c_int; 20] = [0; 20];
        let mingrappleangle: f32;
        let z: f32;
        let hordist: f32;
        let mut bsptrace: bsp_trace_t;
        let mut trace: aas_trace_t;
        let mut areastart: vec3_t = [0.0; 3];
        let mut facecenter: vec3_t = [0.0; 3];
        let mut start: vec3_t;
        let mut end: vec3_t;
        let mut dir: vec3_t;
        let down: vec3_t = [0.0, 0.0, -1.0];

        //only grapple when on the ground or swimming
        if AAS_AreaGrounded(bot, area1num) == 0 && AAS_AreaSwim(bot, area1num) == 0 {
            return qfalse;
        }
        //don't grapple from a crouch area
        if AAS_AreaPresenceType(bot, area1num) & PRESENCE_NORMAL == 0 {
            return qfalse;
        }
        //NOTE: disabled area swim it doesn't work right
        if AAS_AreaSwim(bot, area1num) != 0 {
            return qfalse;
        }
        //
        let area1: *mut aas_area_t = bot.aasworld.areas.add(area1num as usize);
        let area2: *mut aas_area_t = bot.aasworld.areas.add(area2num as usize);
        //don't grapple towards way lower areas
        if (*area2).maxs[2] < (*area1).mins[2] {
            return qfalse;
        }
        //
        start = (*bot.aasworld.areas.add(area1num as usize)).center;
        //if not a swim area
        if AAS_AreaSwim(bot, area1num) == 0 {
            if AAS_PointAreaNum(bot, start) == 0 {
                Log_Write(
                    bot,
                    c"area %d center %f %f %f in solid?\r\n".as_ptr() as *mut c_char,
                    area1num,
                    start[0] as f64,
                    start[1] as f64,
                    start[2] as f64,
                );
            }
            end = start;
            end[2] -= 1000.0;
            trace = AAS_TraceClientBBox(bot, start, end, PRESENCE_CROUCH, -1);
            if trace.startsolid != 0 {
                return qfalse;
            }
            areastart = trace.endpos;
        } else if AAS_PointContents(bot, start) & (CONTENTS_LAVA | CONTENTS_SLIME | CONTENTS_WATER)
            == 0
        {
            return qfalse;
        }
        //
        //start is now the start point
        //
        let mut i = 0;
        while i < (*area2).numfaces {
            face2num = *bot
                .aasworld
                .faceindex
                .add(((*area2).firstface + i) as usize);
            let face2: *mut aas_face_t = bot.aasworld.faces.add(face2num.unsigned_abs() as usize);
            //if it is not a solid face
            if (*face2).faceflags & FACE_SOLID == 0 {
                i += 1;
                continue;
            }
            //direction towards the first vertex of the face
            let firstedge = *bot.aasworld.edgeindex.add((*face2).firstedge as usize);
            let v = *bot
                .aasworld
                .vertexes
                .add((*bot.aasworld.edges.add(firstedge.unsigned_abs() as usize)).v[0] as usize);
            dir = [
                v[0] - areastart[0],
                v[1] - areastart[1],
                v[2] - areastart[2],
            ];
            //if the face plane is facing away
            let n2 = (*bot.aasworld.planes.add((*face2).planenum as usize)).normal;
            if n2[0] * dir[0] + n2[1] * dir[1] + n2[2] * dir[2] > 0.0 {
                i += 1;
                continue;
            }
            //get the center of the face
            AAS_FaceCenter(bot, face2num, facecenter);
            //only go higher up with the grapple
            if facecenter[2] < areastart[2] + 64.0 {
                i += 1;
                continue;
            }
            //only use vertical faces or downward facing faces
            if n2[0] * down[0] + n2[1] * down[1] + n2[2] * down[2] < 0.0 {
                i += 1;
                continue;
            }
            //direction towards the face center
            dir = [
                facecenter[0] - areastart[0],
                facecenter[1] - areastart[1],
                facecenter[2] - areastart[2],
            ];
            //
            z = dir[2];
            dir[2] = 0.0;
            hordist = VectorLength(dir);
            if hordist == 0.0 {
                i += 1;
                continue;
            }
            //if too far
            if hordist > 2000.0 {
                i += 1;
                continue;
            }
            //check the minimal angle of the movement
            mingrappleangle = 15.0; //15 degrees
            if (z / hordist) < ((2.0 * M_PI * mingrappleangle as f64 / 360.0).tan() as f32) {
                i += 1;
                continue;
            }
            //
            start = facecenter;
            let n2b = (*bot.aasworld.planes.add((*face2).planenum as usize)).normal;
            end = [
                facecenter[0] + n2b[0] * -500.0,
                facecenter[1] + n2b[1] * -500.0,
                facecenter[2] + n2b[2] * -500.0,
            ];
            //
            bsptrace = AAS_Trace(
                bot,
                start,
                ptr::null_mut(),
                ptr::null_mut(),
                end,
                0,
                CONTENTS_SOLID,
            );
            //the grapple won't stick to the sky and the grapple point should be near the AAS wall
            if bsptrace.surface.flags & SURF_SKY != 0 || (bsptrace.fraction * 500.0 > 32.0) {
                i += 1;
                continue;
            }
            //trace a full bounding box from the area center on the ground to
            //the center of the face
            dir = [
                facecenter[0] - areastart[0],
                facecenter[1] - areastart[1],
                facecenter[2] - areastart[2],
            ];
            VectorNormalize(&mut dir);
            start = [
                areastart[0] + dir[0] * 4.0,
                areastart[1] + dir[1] * 4.0,
                areastart[2] + dir[2] * 4.0,
            ];
            end = bsptrace.endpos;
            trace = AAS_TraceClientBBox(bot, start, end, PRESENCE_NORMAL, -1);
            dir = [
                trace.endpos[0] - facecenter[0],
                trace.endpos[1] - facecenter[1],
                trace.endpos[2] - facecenter[2],
            ];
            if VectorLength(dir) > 24.0 {
                i += 1;
                continue;
            }
            //
            start = trace.endpos;
            end = trace.endpos;
            end[2] -= AAS_FallDamageDistance(bot) as f32;
            trace = AAS_TraceClientBBox(bot, start, end, PRESENCE_NORMAL, -1);
            if trace.fraction >= 1.0 {
                i += 1;
                continue;
            }
            //area to end in
            areanum = AAS_PointAreaNum(bot, trace.endpos);
            //if not in lava or slime
            if (*bot.aasworld.areasettings.add(areanum as usize)).contents
                & (AREACONTENTS_SLIME | AREACONTENTS_LAVA)
                != 0
            {
                i += 1;
                continue;
            }
            //do not go the the source area
            if areanum == area1num {
                i += 1;
                continue;
            }
            //don't create reachabilities if they already exist
            if AAS_ReachabilityExists(bot, area1num, areanum) != 0 {
                i += 1;
                continue;
            }
            //only end in areas we can stand
            if AAS_AreaGrounded(bot, areanum) == 0 {
                i += 1;
                continue;
            }
            //never go through cluster portals!!
            numareas = AAS_TraceAreas(
                bot,
                areastart,
                bsptrace.endpos,
                areas.as_mut_ptr(),
                ptr::null_mut(),
                20,
            );
            if numareas >= 20 {
                i += 1;
                continue;
            }
            let mut j = 0;
            while j < numareas {
                if (*bot.aasworld.areasettings.add(areas[j as usize] as usize)).contents
                    & AREACONTENTS_CLUSTERPORTAL
                    != 0
                {
                    break;
                }
                j += 1;
            }
            if j < numareas {
                i += 1;
                continue;
            }
            //create a new reachability link
            let lreach = AAS_AllocReachability(bot);
            if lreach.is_null() {
                return qfalse;
            }
            (*lreach).areanum = areanum;
            (*lreach).facenum = face2num;
            (*lreach).edgenum = 0;
            (*lreach).start = areastart;
            (*lreach).end = bsptrace.endpos;
            (*lreach).traveltype = TRAVEL_GRAPPLEHOOK;
            dir = [
                (*lreach).end[0] - (*lreach).start[0],
                (*lreach).end[1] - (*lreach).start[1],
                (*lreach).end[2] - (*lreach).start[2],
            ];
            (*lreach).traveltime =
                (bot.aassettings.rs_startgrapple + VectorLength(dir) * 0.25) as c_ushort;
            (*lreach).next = *bot.areareachability.add(area1num as usize);
            *bot.areareachability.add(area1num as usize) = lreach;
            //
            bot.reach_grapple += 1;
            i += 1;
        }
        //
        qfalse
    }
}

/// Raven `AAS_Reachability_WalkOffLedge`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:4109-4283`
pub fn AAS_Reachability_WalkOffLedge(bot: &mut BotLib, areanum: c_int) {
    unsafe {
        let mut areas: [c_int; 10] = [0; 10];
        let mut numareas: c_int;
        let mut face1num: c_int;
        let mut face2num: c_int;
        let mut face3num: c_int;
        let mut edge1num: c_int;
        let mut edge2num: c_int;
        let mut edge3num: c_int;
        let mut otherareanum: c_int;
        let mut gap: c_int;
        let mut reachareanum: c_int;
        let side: usize;
        let mut sharededgevec: vec3_t;
        let mut mid: vec3_t;
        let mut dir: vec3_t = [0.0; 3];
        let mut testend: vec3_t;
        let mut trace: aas_trace_t;

        if AAS_AreaGrounded(bot, areanum) == 0 || AAS_AreaSwim(bot, areanum) != 0 {
            return;
        }
        //
        let area: *mut aas_area_t = bot.aasworld.areas.add(areanum as usize);
        //
        let mut i = 0;
        while i < (*area).numfaces {
            face1num = *bot.aasworld.faceindex.add(((*area).firstface + i) as usize);
            let face1: *mut aas_face_t = bot.aasworld.faces.add(face1num.unsigned_abs() as usize);
            //face 1 must be a ground face
            if (*face1).faceflags & FACE_GROUND == 0 {
                i += 1;
                continue;
            }
            //go through all the edges of this ground face
            let mut k = 0;
            while k < (*face1).numedges {
                edge1num = *bot
                    .aasworld
                    .edgeindex
                    .add(((*face1).firstedge + k) as usize);
                //find another not ground face using this same edge
                let mut j = 0;
                while j < (*area).numfaces {
                    face2num = *bot.aasworld.faceindex.add(((*area).firstface + j) as usize);
                    let face2: *mut aas_face_t =
                        bot.aasworld.faces.add(face2num.unsigned_abs() as usize);
                    //face 2 may not be a ground face
                    if (*face2).faceflags & FACE_GROUND != 0 {
                        j += 1;
                        continue;
                    }
                    //compare all the edges
                    let mut l = 0;
                    while l < (*face2).numedges {
                        edge2num = *bot
                            .aasworld
                            .edgeindex
                            .add(((*face2).firstedge + l) as usize);
                        if edge1num.abs() == edge2num.abs() {
                            //get the area at the other side of the face
                            if (*face2).frontarea == areanum {
                                otherareanum = (*face2).backarea;
                            } else {
                                otherareanum = (*face2).frontarea;
                            }
                            //
                            let area2: *mut aas_area_t =
                                bot.aasworld.areas.add(otherareanum as usize);
                            //if the other area is grounded!
                            if (*bot.aasworld.areasettings.add(otherareanum as usize)).areaflags
                                & AREA_GROUNDED
                                != 0
                            {
                                //check for a possible gap
                                gap = qfalse;
                                let mut n = 0;
                                while n < (*area2).numfaces {
                                    face3num = *bot
                                        .aasworld
                                        .faceindex
                                        .add(((*area2).firstface + n) as usize);
                                    //may not be the shared face of the two areas
                                    if face3num.abs() == face2num.abs() {
                                        n += 1;
                                        continue;
                                    }
                                    //
                                    let face3: *mut aas_face_t =
                                        bot.aasworld.faces.add(face3num.unsigned_abs() as usize);
                                    //find an edge shared by all three faces
                                    let mut m = 0;
                                    while m < (*face3).numedges {
                                        edge3num = *bot
                                            .aasworld
                                            .edgeindex
                                            .add(((*face3).firstedge + m) as usize);
                                        //but the edge should be shared by all three faces
                                        if edge3num.abs() == edge1num.abs() {
                                            if (*face3).faceflags & FACE_SOLID == 0 {
                                                gap = qtrue;
                                                break;
                                            }
                                            //
                                            if (*face3).faceflags & FACE_GROUND != 0 {
                                                gap = qfalse;
                                                break;
                                            }
                                            // Raven note: there are more situations to be handled
                                            gap = qtrue;
                                            break;
                                        }
                                        m += 1;
                                    }
                                    if m < (*face3).numedges {
                                        break;
                                    }
                                    n += 1;
                                }
                                if gap == 0 {
                                    break;
                                }
                            }
                            //check for a walk off ledge reachability
                            let edge: *mut aas_edge_t =
                                bot.aasworld.edges.add(edge1num.unsigned_abs() as usize);
                            side = (edge1num < 0) as usize;
                            //
                            let v1 = *bot.aasworld.vertexes.add((*edge).v[side] as usize);
                            let v2 = *bot.aasworld.vertexes.add((*edge).v[1 - side] as usize);
                            //
                            let plane: *mut aas_plane_t =
                                bot.aasworld.planes.add((*face1).planenum as usize);
                            //get the points really into the areas
                            sharededgevec = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
                            CrossProduct((*plane).normal, sharededgevec, &mut dir);
                            VectorNormalize(&mut dir);
                            //
                            mid = [v1[0] + v2[0], v1[1] + v2[1], v1[2] + v2[2]];
                            mid = [mid[0] * 0.5, mid[1] * 0.5, mid[2] * 0.5];
                            mid = [
                                mid[0] + dir[0] * 8.0,
                                mid[1] + dir[1] * 8.0,
                                mid[2] + dir[2] * 8.0,
                            ];
                            //
                            testend = mid;
                            testend[2] -= 1000.0;
                            trace = AAS_TraceClientBBox(bot, mid, testend, PRESENCE_CROUCH, -1);
                            //
                            if trace.startsolid != 0 {
                                break;
                            }
                            reachareanum = AAS_PointAreaNum(bot, trace.endpos);
                            if reachareanum == areanum {
                                break;
                            }
                            if AAS_ReachabilityExists(bot, areanum, reachareanum) != 0 {
                                break;
                            }
                            if AAS_AreaGrounded(bot, reachareanum) == 0
                                && AAS_AreaSwim(bot, reachareanum) == 0
                            {
                                break;
                            }
                            //
                            if (*bot.aasworld.areasettings.add(reachareanum as usize)).contents
                                & (AREACONTENTS_SLIME | AREACONTENTS_LAVA)
                                != 0
                            {
                                break;
                            }
                            //if not going through a cluster portal
                            numareas = AAS_TraceAreas(
                                bot,
                                mid,
                                testend,
                                areas.as_mut_ptr(),
                                ptr::null_mut(),
                                (areas.len()) as c_int,
                            );
                            let mut p = 0;
                            while p < numareas {
                                if AAS_AreaClusterPortal(bot, areas[p as usize]) != 0 {
                                    break;
                                }
                                p += 1;
                            }
                            if p < numareas {
                                break;
                            }
                            // if a maximum fall height is set and the bot would fall down further
                            if bot.aassettings.rs_maxfallheight != 0.0
                                && (mid[2] - trace.endpos[2]).abs()
                                    > bot.aassettings.rs_maxfallheight
                            {
                                break;
                            }
                            //
                            let lreach = AAS_AllocReachability(bot);
                            if lreach.is_null() {
                                break;
                            }
                            (*lreach).areanum = reachareanum;
                            (*lreach).facenum = 0;
                            (*lreach).edgenum = edge1num;
                            (*lreach).start = mid;
                            (*lreach).end = trace.endpos;
                            (*lreach).traveltype = TRAVEL_WALKOFFLEDGE;
                            (*lreach).traveltime = (bot.aassettings.rs_startwalkoffledge
                                + (mid[2] - trace.endpos[2]).abs() * 50.0
                                    / bot.aassettings.phys_gravity)
                                as c_ushort;
                            if AAS_AreaSwim(bot, reachareanum) == 0
                                && AAS_AreaJumpPad(bot, reachareanum) == 0
                            {
                                if AAS_FallDelta(bot, mid[2] - trace.endpos[2])
                                    > bot.aassettings.phys_falldelta5
                                {
                                    (*lreach).traveltime +=
                                        bot.aassettings.rs_falldamage5 as c_ushort;
                                } else if AAS_FallDelta(bot, mid[2] - trace.endpos[2])
                                    > bot.aassettings.phys_falldelta10
                                {
                                    (*lreach).traveltime +=
                                        bot.aassettings.rs_falldamage10 as c_ushort;
                                }
                            }
                            (*lreach).next = *bot.areareachability.add(areanum as usize);
                            *bot.areareachability.add(areanum as usize) = lreach;
                            //we've got another walk off ledge reachability
                            bot.reach_walkoffledge += 1;
                        }
                        l += 1;
                    }
                    j += 1;
                }
                k += 1;
            }
            i += 1;
        }
    }
}

/// Raven `AAS_InitReachability`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:4494-4525`
pub fn AAS_InitReachability(bot: &mut BotLib) {
    unsafe {
        if bot.aasworld.loaded == 0 {
            return;
        }

        if bot.aasworld.reachabilitysize != 0 {
            if (LibVarGetValue(bot, c"forcereachability".as_ptr() as *mut c_char) as c_int) == 0 {
                bot.aasworld.numreachabilityareas = bot.aasworld.numareas + 2;
                return;
            }
        }
        bot.calcgrapplereach =
            LibVarGetValue(bot, c"grapplereach".as_ptr() as *mut c_char) as c_int;
        bot.aasworld.savefile = qtrue;
        //start with area 1 because area zero is a dummy
        bot.aasworld.numreachabilityareas = 1;
        //setup the heap with reachability links
        AAS_SetupReachabilityHeap(bot);
        //allocate area reachability link array
        bot.areareachability = GetClearedMemory(
            bot,
            bot.aasworld.numareas as usize * core::mem::size_of::<*mut aas_lreachability_t>(),
        ) as *mut *mut aas_lreachability_t;
        //
        AAS_SetWeaponJumpAreaFlags(bot);
    }
}

/// Raven `AAS_BestReachableFromJumpPadArea`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:273-336`
pub fn AAS_BestReachableFromJumpPadArea(
    bot: &mut BotLib,
    origin: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
) -> c_int {
    unsafe {
        let mut area2num: c_int;
        let bot_visualizejumppads: c_int;
        let mut volume: f32;
        let mut bestareavolume: f32;
        let mut areastart: vec3_t = [0.0; 3];
        let mut cmdmove: vec3_t;
        let mut absmins: vec3_t = [0.0; 3];
        let mut absmaxs: vec3_t = [0.0; 3];
        let mut velocity: vec3_t = [0.0; 3];
        let mut r#move: aas_clientmove_t = core::mem::zeroed();
        let mut classname: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];

        bot_visualizejumppads = LibVarValue(
            bot,
            c"bot_visualizejumppads".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
        ) as c_int;
        let bboxmins: vec3_t = [
            origin[0] + mins[0],
            origin[1] + mins[1],
            origin[2] + mins[2],
        ];
        let bboxmaxs: vec3_t = [
            origin[0] + maxs[0],
            origin[1] + maxs[1],
            origin[2] + maxs[2],
        ];
        let mut ent = AAS_NextBSPEntity(bot, 0);
        while ent != 0 {
            if AAS_ValueForBSPEpairKey(
                bot,
                ent,
                c"classname".as_ptr() as *mut c_char,
                classname.as_mut_ptr(),
                MAX_EPAIRKEY,
            ) == 0
            {
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            if strcmp(classname.as_ptr(), c"trigger_push".as_ptr()) != 0 {
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            //
            if AAS_GetJumpPadInfo(bot, ent, areastart, absmins, absmaxs, velocity) == 0 {
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            //get the areas the jump pad brush is in
            let areas = AAS_LinkEntityClientBBox(bot, absmins, absmaxs, -1, PRESENCE_CROUCH);
            let mut link = areas;
            while !link.is_null() {
                if AAS_AreaJumpPad(bot, (*link).areanum) != 0 {
                    break;
                }
                link = (*link).next_area;
            }
            if link.is_null() {
                bot.botimport.Print.unwrap()(
                    PRT_MESSAGE,
                    c"trigger_push not in any jump pad area\n".as_ptr() as *mut c_char,
                );
                AAS_UnlinkFromAreas(bot, areas);
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            //
            cmdmove = [0.0, 0.0, 0.0];
            Com_Memset(
                (&mut r#move as *mut aas_clientmove_t) as *mut (),
                0,
                core::mem::size_of::<aas_clientmove_t>(),
            );
            area2num = 0;
            AAS_ClientMovementHitBBox(
                bot,
                &mut r#move,
                -1,
                areastart,
                PRESENCE_NORMAL,
                qfalse,
                velocity,
                cmdmove,
                0,
                30,
                0.1,
                bboxmins,
                bboxmaxs,
                bot_visualizejumppads,
            );
            if r#move.frames < 30 {
                let mut bestareanum = 0;
                bestareavolume = 0.0;
                let mut link = areas;
                while !link.is_null() {
                    if AAS_AreaJumpPad(bot, (*link).areanum) == 0 {
                        link = (*link).next_area;
                        continue;
                    }
                    volume = AAS_AreaVolume(bot, (*link).areanum);
                    if volume >= bestareavolume {
                        bestareanum = (*link).areanum;
                        bestareavolume = volume;
                    }
                    link = (*link).next_area;
                }
                AAS_UnlinkFromAreas(bot, areas);
                return bestareanum;
            }
            AAS_UnlinkFromAreas(bot, areas);
            ent = AAS_NextBSPEntity(bot, ent);
        }
        0
    }
}

/// Raven `AAS_Reachability_Jump`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:2092-2355`
pub fn AAS_Reachability_Jump(bot: &mut BotLib, area1num: c_int, area2num: c_int) -> c_int {
    unsafe {
        let mut face1num: c_int;
        let mut face2num: c_int;
        let mut edge1num: c_int;
        let mut edge2num: c_int;
        let mut traveltype: c_int = 0;
        let mut stopevent: c_int;
        let mut areas: [c_int; 10] = [0; 10];
        let mut numareas: c_int;
        let phys_jumpvel: f32;
        let maxjumpdistance: f32;
        let maxjumpheight: f32;
        let mut height: f32;
        let mut bestdist: f32;
        let mut speed: f32 = 0.0;
        let mut beststart: vec3_t = [0.0; 3];
        let mut beststart2: vec3_t = [0.0; 3];
        let mut bestend: vec3_t = [0.0; 3];
        let mut bestend2: vec3_t = [0.0; 3];
        let mut teststart: vec3_t;
        let mut testend: vec3_t = [0.0; 3];
        let mut dir: vec3_t;
        let mut velocity: vec3_t;
        let mut cmdmove: vec3_t = [0.0; 3];
        let up: vec3_t = [0.0, 0.0, 1.0];
        let mut sidewards: vec3_t = [0.0; 3];
        let mut plane: *mut aas_plane_t;
        let mut trace: aas_trace_t;
        let mut r#move: aas_clientmove_t = core::mem::zeroed();

        if AAS_AreaGrounded(bot, area1num) == 0 || AAS_AreaGrounded(bot, area2num) == 0 {
            return qfalse;
        }
        //cannot jump from or to a crouch area
        if AAS_AreaCrouch(bot, area1num) != 0 || AAS_AreaCrouch(bot, area2num) != 0 {
            return qfalse;
        }
        //
        let area1: *mut aas_area_t = bot.aasworld.areas.add(area1num as usize);
        let area2: *mut aas_area_t = bot.aasworld.areas.add(area2num as usize);
        //
        phys_jumpvel = bot.aassettings.phys_jumpvel;
        //maximum distance a player can jump
        maxjumpdistance = 2.0 * AAS_MaxJumpDistance(bot, phys_jumpvel);
        //maximum height a player can jump with the given initial z velocity
        maxjumpheight = AAS_MaxJumpHeight(bot, phys_jumpvel);

        //if the areas are not near anough in the x-y direction
        let mut i = 0;
        while i < 2 {
            if (*area1).mins[i] > (*area2).maxs[i] + maxjumpdistance {
                return qfalse;
            }
            if (*area1).maxs[i] < (*area2).mins[i] - maxjumpdistance {
                return qfalse;
            }
            i += 1;
        }
        //if area2 is way to high to jump up to
        if (*area2).mins[2] > (*area1).maxs[2] + maxjumpheight {
            return qfalse;
        }
        //
        bestdist = 999999.0;
        //
        let mut i = 0;
        while i < (*area1).numfaces {
            face1num = *bot
                .aasworld
                .faceindex
                .add(((*area1).firstface + i) as usize);
            let face1: *mut aas_face_t = bot.aasworld.faces.add(face1num.unsigned_abs() as usize);
            //if not a ground face
            if (*face1).faceflags & FACE_GROUND == 0 {
                i += 1;
                continue;
            }
            //
            let mut j = 0;
            while j < (*area2).numfaces {
                face2num = *bot
                    .aasworld
                    .faceindex
                    .add(((*area2).firstface + j) as usize);
                let face2: *mut aas_face_t =
                    bot.aasworld.faces.add(face2num.unsigned_abs() as usize);
                //if not a ground face
                if (*face2).faceflags & FACE_GROUND == 0 {
                    j += 1;
                    continue;
                }
                //
                let mut k = 0;
                while k < (*face1).numedges {
                    edge1num = (*bot
                        .aasworld
                        .edgeindex
                        .add(((*face1).firstedge + k) as usize))
                    .abs();
                    let edge1: *mut aas_edge_t = bot.aasworld.edges.add(edge1num as usize);
                    let mut l = 0;
                    while l < (*face2).numedges {
                        edge2num = (*bot
                            .aasworld
                            .edgeindex
                            .add(((*face2).firstedge + l) as usize))
                        .abs();
                        let edge2: *mut aas_edge_t = bot.aasworld.edges.add(edge2num as usize);
                        //calculate the minimum distance between the two edges
                        let v1 = *bot.aasworld.vertexes.add((*edge1).v[0] as usize);
                        let v2 = *bot.aasworld.vertexes.add((*edge1).v[1] as usize);
                        let v3 = *bot.aasworld.vertexes.add((*edge2).v[0] as usize);
                        let v4 = *bot.aasworld.vertexes.add((*edge2).v[1] as usize);
                        //get the ground planes
                        let plane1: *mut aas_plane_t =
                            bot.aasworld.planes.add((*face1).planenum as usize);
                        let plane2: *mut aas_plane_t =
                            bot.aasworld.planes.add((*face2).planenum as usize);
                        //
                        bestdist = AAS_ClosestEdgePoints(
                            v1, v2, v3, v4, plane1, plane2, beststart, bestend, beststart2,
                            bestend2, bestdist,
                        );
                        l += 1;
                    }
                    k += 1;
                }
                j += 1;
            }
            i += 1;
        }
        beststart = [
            (beststart[0] + beststart2[0]) * 0.5,
            (beststart[1] + beststart2[1]) * 0.5,
            (beststart[2] + beststart2[2]) * 0.5,
        ];
        bestend = [
            (bestend[0] + bestend2[0]) * 0.5,
            (bestend[1] + bestend2[1]) * 0.5,
            (bestend[2] + bestend2[2]) * 0.5,
        ];
        if bestdist > 4.0 && bestdist < maxjumpdistance {
            // if very close and almost no height difference then the bot can walk
            if bestdist <= 48.0 && (beststart[2] - bestend[2]).abs() < 8.0 {
                speed = 400.0;
                traveltype = TRAVEL_WALKOFFLEDGE;
            } else if AAS_HorizontalVelocityForJump(bot, 0.0, beststart, bestend, &mut speed) != 0 {
                // Raven note: why multiply with 1.2???
                speed *= 1.2;
                traveltype = TRAVEL_WALKOFFLEDGE;
            } else {
                //get the horizontal speed for the jump
                if AAS_HorizontalVelocityForJump(bot, phys_jumpvel, beststart, bestend, &mut speed)
                    == 0
                {
                    return qfalse;
                }
                speed *= 1.05;
                traveltype = TRAVEL_JUMP;
                //
                //NOTE: test if the horizontal distance isn't too small
                dir = [
                    bestend[0] - beststart[0],
                    bestend[1] - beststart[1],
                    bestend[2] - beststart[2],
                ];
                dir[2] = 0.0;
                if VectorLength(dir) < 10.0 {
                    return qfalse;
                }
            }
            //
            dir = [
                bestend[0] - beststart[0],
                bestend[1] - beststart[1],
                bestend[2] - beststart[2],
            ];
            VectorNormalize(&mut dir);
            teststart = [
                beststart[0] + dir[0],
                beststart[1] + dir[1],
                beststart[2] + dir[2],
            ];
            //
            testend = teststart;
            testend[2] -= 100.0;
            trace = AAS_TraceClientBBox(bot, teststart, testend, PRESENCE_NORMAL, -1);
            //
            if trace.startsolid != 0 {
                return qfalse;
            }
            if trace.fraction < 1.0 {
                plane = bot.aasworld.planes.add(trace.planenum as usize);
                // if the bot can stand on the surface
                if (*plane).normal[0] * up[0]
                    + (*plane).normal[1] * up[1]
                    + (*plane).normal[2] * up[2]
                    >= 0.7
                {
                    // if no lava or slime below
                    if AAS_PointContents(bot, trace.endpos) & (CONTENTS_LAVA | CONTENTS_SLIME) == 0
                    {
                        if teststart[2] - trace.endpos[2] <= bot.aassettings.phys_maxbarrier {
                            return qfalse;
                        }
                    }
                }
            }
            //
            teststart = [
                bestend[0] - dir[0],
                bestend[1] - dir[1],
                bestend[2] - dir[2],
            ];
            //
            testend = teststart;
            testend[2] -= 100.0;
            trace = AAS_TraceClientBBox(bot, teststart, testend, PRESENCE_NORMAL, -1);
            //
            if trace.startsolid != 0 {
                return qfalse;
            }
            if trace.fraction < 1.0 {
                plane = bot.aasworld.planes.add(trace.planenum as usize);
                // if the bot can stand on the surface
                if (*plane).normal[0] * up[0]
                    + (*plane).normal[1] * up[1]
                    + (*plane).normal[2] * up[2]
                    >= 0.7
                {
                    // if no lava or slime below
                    if AAS_PointContents(bot, trace.endpos) & (CONTENTS_LAVA | CONTENTS_SLIME) == 0
                    {
                        if teststart[2] - trace.endpos[2] <= bot.aassettings.phys_maxbarrier {
                            return qfalse;
                        }
                    }
                }
            }
            //
            // get command movement
            cmdmove = [0.0; 3];
            if (traveltype & TRAVELTYPE_MASK) == TRAVEL_JUMP {
                cmdmove[2] = bot.aassettings.phys_jumpvel;
            } else {
                cmdmove[2] = 0.0;
            }
            //
            dir = [
                bestend[0] - beststart[0],
                bestend[1] - beststart[1],
                bestend[2] - beststart[2],
            ];
            dir[2] = 0.0;
            VectorNormalize(&mut dir);
            CrossProduct(dir, up, &mut sidewards);
            //
            stopevent =
                SE_HITGROUND | SE_ENTERWATER | SE_ENTERSLIME | SE_ENTERLAVA | SE_HITGROUNDDAMAGE;
            if AAS_AreaClusterPortal(bot, area1num) == 0
                && AAS_AreaClusterPortal(bot, area2num) == 0
            {
                stopevent |= SE_TOUCHCLUSTERPORTAL;
            }
            //
            let mut i = 0;
            while i < 3 {
                //
                if i == 1 {
                    testend = [
                        testend[0] + sidewards[0],
                        testend[1] + sidewards[1],
                        testend[2] + sidewards[2],
                    ];
                } else if i == 2 {
                    testend = [
                        bestend[0] - sidewards[0],
                        bestend[1] - sidewards[1],
                        bestend[2] - sidewards[2],
                    ];
                } else {
                    testend = bestend;
                }
                dir = [
                    testend[0] - beststart[0],
                    testend[1] - beststart[1],
                    testend[2] - beststart[2],
                ];
                dir[2] = 0.0;
                VectorNormalize(&mut dir);
                velocity = [dir[0] * speed, dir[1] * speed, dir[2] * speed];
                //
                AAS_PredictClientMovement(
                    bot,
                    &mut r#move,
                    -1,
                    beststart,
                    PRESENCE_NORMAL,
                    qtrue,
                    velocity,
                    cmdmove,
                    3,
                    30,
                    0.1,
                    stopevent,
                    0,
                    qfalse,
                );
                // if prediction time wasn't enough to fully predict the movement
                if r#move.frames >= 30 {
                    return qfalse;
                }
                // don't enter slime or lava and don't fall from too high
                if r#move.stopevent & (SE_ENTERSLIME | SE_ENTERLAVA) != 0 {
                    return qfalse;
                }
                // never jump or fall through a cluster portal
                if r#move.stopevent & SE_TOUCHCLUSTERPORTAL != 0 {
                    return qfalse;
                }
                //the end position should be in area2
                teststart = [
                    r#move.endpos[0] - dir[0] * 64.0,
                    r#move.endpos[1] - dir[1] * 64.0,
                    r#move.endpos[2] - dir[2] * 64.0,
                ];
                teststart[2] += 1.0;
                numareas = AAS_TraceAreas(
                    bot,
                    r#move.endpos,
                    teststart,
                    areas.as_mut_ptr(),
                    ptr::null_mut(),
                    (areas.len()) as c_int,
                );
                let mut j = 0;
                while j < numareas {
                    if areas[j as usize] == area2num {
                        break;
                    }
                    j += 1;
                }
                if j < numareas {
                    break;
                }
                i += 1;
            }
            if i >= 3 {
                return qfalse;
            }
            //
            //create a new reachability link
            let lreach = AAS_AllocReachability(bot);
            if lreach.is_null() {
                return qfalse;
            }
            (*lreach).areanum = area2num;
            (*lreach).facenum = 0;
            (*lreach).edgenum = 0;
            (*lreach).start = beststart;
            (*lreach).end = bestend;
            (*lreach).traveltype = traveltype;

            dir = [
                bestend[0] - beststart[0],
                bestend[1] - beststart[1],
                bestend[2] - beststart[2],
            ];
            height = dir[2];
            dir[2] = 0.0;
            if (traveltype & TRAVELTYPE_MASK) == TRAVEL_WALKOFFLEDGE && height > VectorLength(dir) {
                (*lreach).traveltime = (bot.aassettings.rs_startwalkoffledge
                    + height * 50.0 / bot.aassettings.phys_gravity)
                    as c_ushort;
            } else {
                (*lreach).traveltime = (bot.aassettings.rs_startjump
                    + VectorDistance(bestend, beststart) * 240.0
                        / bot.aassettings.phys_maxwalkvelocity)
                    as c_ushort;
            }
            //
            if AAS_AreaJumpPad(bot, area2num) == 0 {
                if AAS_FallDelta(bot, beststart[2] - bestend[2]) > bot.aassettings.phys_falldelta5 {
                    (*lreach).traveltime += bot.aassettings.rs_falldamage5 as c_ushort;
                } else if AAS_FallDelta(bot, beststart[2] - bestend[2])
                    > bot.aassettings.phys_falldelta10
                {
                    (*lreach).traveltime += bot.aassettings.rs_falldamage10 as c_ushort;
                }
            }
            (*lreach).next = *bot.areareachability.add(area1num as usize);
            *bot.areareachability.add(area1num as usize) = lreach;
            //
            if (traveltype & TRAVELTYPE_MASK) == TRAVEL_JUMP {
                bot.reach_jump += 1;
            } else {
                bot.reach_walkoffledge += 1;
            }
        }
        qfalse
    }
}

/// Raven `AAS_Reachability_Teleport`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:2736-2921`
pub fn AAS_Reachability_Teleport(bot: &mut BotLib) {
    unsafe {
        let mut area1num: c_int;
        let mut area2num: c_int;
        let mut target: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];
        let mut targetname: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];
        let mut classname: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];
        let mut model: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];
        let mut angle: f32 = 0.0;
        let mut origin: vec3_t = [0.0; 3];
        let mut destorigin: vec3_t = [0.0; 3];
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut end: vec3_t;
        let mut angles: vec3_t = [0.0; 3];
        let mut mid: vec3_t;
        let mut velocity: vec3_t = [0.0; 3];
        let mut cmdmove: vec3_t;
        let mut r#move: aas_clientmove_t = core::mem::zeroed();
        let trace: aas_trace_t;

        let mut ent = AAS_NextBSPEntity(bot, 0);
        while ent != 0 {
            if AAS_ValueForBSPEpairKey(
                bot,
                ent,
                c"classname".as_ptr() as *mut c_char,
                classname.as_mut_ptr(),
                MAX_EPAIRKEY,
            ) == 0
            {
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            let mut dest;
            if strcmp(classname.as_ptr(), c"trigger_multiple".as_ptr()) == 0 {
                AAS_ValueForBSPEpairKey(
                    bot,
                    ent,
                    c"model".as_ptr() as *mut c_char,
                    model.as_mut_ptr(),
                    MAX_EPAIRKEY,
                );
                bot.botimport.Print.unwrap()(
                    PRT_MESSAGE,
                    c"trigger_multiple model = \"%s\"\n".as_ptr() as *mut c_char,
                    model.as_ptr(),
                );
                angles = [0.0; 3];
                AAS_BSPModelMinsMaxsOrigin(
                    bot,
                    atoi(model.as_ptr().wrapping_add(1)),
                    angles,
                    mins,
                    maxs,
                    origin,
                );
                //
                if AAS_ValueForBSPEpairKey(
                    bot,
                    ent,
                    c"target".as_ptr() as *mut c_char,
                    target.as_mut_ptr(),
                    MAX_EPAIRKEY,
                ) == 0
                {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"trigger_multiple at %1.0f %1.0f %1.0f without target\n".as_ptr()
                            as *mut c_char,
                        origin[0] as f64,
                        origin[1] as f64,
                        origin[2] as f64,
                    );
                    ent = AAS_NextBSPEntity(bot, ent);
                    continue;
                }
                dest = AAS_NextBSPEntity(bot, 0);
                while dest != 0 {
                    if AAS_ValueForBSPEpairKey(
                        bot,
                        dest,
                        c"classname".as_ptr() as *mut c_char,
                        classname.as_mut_ptr(),
                        MAX_EPAIRKEY,
                    ) == 0
                    {
                        dest = AAS_NextBSPEntity(bot, dest);
                        continue;
                    }
                    if strcmp(classname.as_ptr(), c"target_teleporter".as_ptr()) == 0 {
                        if AAS_ValueForBSPEpairKey(
                            bot,
                            dest,
                            c"targetname".as_ptr() as *mut c_char,
                            targetname.as_mut_ptr(),
                            MAX_EPAIRKEY,
                        ) == 0
                        {
                            dest = AAS_NextBSPEntity(bot, dest);
                            continue;
                        }
                        if strcmp(targetname.as_ptr(), target.as_ptr()) == 0 {
                            break;
                        }
                    }
                    dest = AAS_NextBSPEntity(bot, dest);
                }
                if dest == 0 {
                    ent = AAS_NextBSPEntity(bot, ent);
                    continue;
                }
                if AAS_ValueForBSPEpairKey(
                    bot,
                    dest,
                    c"target".as_ptr() as *mut c_char,
                    target.as_mut_ptr(),
                    MAX_EPAIRKEY,
                ) == 0
                {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"target_teleporter without target\n".as_ptr() as *mut c_char,
                    );
                    ent = AAS_NextBSPEntity(bot, ent);
                    continue;
                }
            } else if strcmp(classname.as_ptr(), c"trigger_teleport".as_ptr()) == 0 {
                AAS_ValueForBSPEpairKey(
                    bot,
                    ent,
                    c"model".as_ptr() as *mut c_char,
                    model.as_mut_ptr(),
                    MAX_EPAIRKEY,
                );
                bot.botimport.Print.unwrap()(
                    PRT_MESSAGE,
                    c"trigger_teleport model = \"%s\"\n".as_ptr() as *mut c_char,
                    model.as_ptr(),
                );
                angles = [0.0; 3];
                AAS_BSPModelMinsMaxsOrigin(
                    bot,
                    atoi(model.as_ptr().wrapping_add(1)),
                    angles,
                    mins,
                    maxs,
                    origin,
                );
                //
                if AAS_ValueForBSPEpairKey(
                    bot,
                    ent,
                    c"target".as_ptr() as *mut c_char,
                    target.as_mut_ptr(),
                    MAX_EPAIRKEY,
                ) == 0
                {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"trigger_teleport at %1.0f %1.0f %1.0f without target\n".as_ptr()
                            as *mut c_char,
                        origin[0] as f64,
                        origin[1] as f64,
                        origin[2] as f64,
                    );
                    ent = AAS_NextBSPEntity(bot, ent);
                    continue;
                }
            } else {
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            //
            dest = AAS_NextBSPEntity(bot, 0);
            while dest != 0 {
                if AAS_ValueForBSPEpairKey(
                    bot,
                    dest,
                    c"targetname".as_ptr() as *mut c_char,
                    targetname.as_mut_ptr(),
                    MAX_EPAIRKEY,
                ) != 0
                {
                    if strcmp(targetname.as_ptr(), target.as_ptr()) == 0 {
                        break;
                    }
                }
                dest = AAS_NextBSPEntity(bot, dest);
            }
            if dest == 0 {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    c"teleporter without misc_teleporter_dest (%s)\n".as_ptr() as *mut c_char,
                    target.as_ptr(),
                );
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            if AAS_VectorForBSPEpairKey(bot, dest, c"origin".as_ptr() as *mut c_char, destorigin)
                == 0
            {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    c"teleporter destination (%s) without origin\n".as_ptr() as *mut c_char,
                    target.as_ptr(),
                );
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            //
            area2num = AAS_PointAreaNum(bot, destorigin);
            //if not teleported into a teleporter or into a jumppad
            if AAS_AreaTeleporter(bot, area2num) == 0 && AAS_AreaJumpPad(bot, area2num) == 0 {
                end = destorigin;
                end[2] -= 64.0;
                trace = AAS_TraceClientBBox(bot, destorigin, end, PRESENCE_CROUCH, -1);
                if trace.startsolid != 0 {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"teleporter destination (%s) in solid\n".as_ptr() as *mut c_char,
                        target.as_ptr(),
                    );
                    ent = AAS_NextBSPEntity(bot, ent);
                    continue;
                }
                area2num = AAS_PointAreaNum(bot, trace.endpos);
                //
                {
                    //predict where you'll end up
                    AAS_FloatForBSPEpairKey(
                        bot,
                        dest,
                        c"angle".as_ptr() as *mut c_char,
                        &mut angle,
                    );
                    if angle != 0.0 {
                        angles = [0.0, angle, 0.0];
                        AngleVectors(angles, Some(&mut velocity), None, None);
                        velocity = [
                            velocity[0] * 400.0,
                            velocity[1] * 400.0,
                            velocity[2] * 400.0,
                        ];
                    } else {
                        velocity = [0.0; 3];
                    }
                    cmdmove = [0.0; 3];
                    AAS_PredictClientMovement(
                        bot,
                        &mut r#move,
                        -1,
                        destorigin,
                        PRESENCE_NORMAL,
                        qfalse,
                        velocity,
                        cmdmove,
                        0,
                        30,
                        0.1,
                        SE_HITGROUND
                            | SE_ENTERWATER
                            | SE_ENTERSLIME
                            | SE_ENTERLAVA
                            | SE_HITGROUNDDAMAGE
                            | SE_TOUCHJUMPPAD
                            | SE_TOUCHTELEPORTER,
                        0,
                        qfalse,
                    );
                    area2num = AAS_PointAreaNum(bot, r#move.endpos);
                    if r#move.stopevent & (SE_ENTERSLIME | SE_ENTERLAVA) != 0 {
                        bot.botimport.Print.unwrap()(
                            PRT_WARNING,
                            c"teleported into slime or lava at dest %s\n".as_ptr() as *mut c_char,
                            target.as_ptr(),
                        );
                    }
                    destorigin = r#move.endpos;
                }
            }
            //
            mins = [
                origin[0] + mins[0],
                origin[1] + mins[1],
                origin[2] + mins[2],
            ];
            maxs = [
                origin[0] + maxs[0],
                origin[1] + maxs[1],
                origin[2] + maxs[2],
            ];
            //
            mid = [mins[0] + maxs[0], mins[1] + maxs[1], mins[2] + maxs[2]];
            mid = [mid[0] * 0.5, mid[1] * 0.5, mid[2] * 0.5];
            //link an invalid (-1) entity
            let areas = AAS_LinkEntityClientBBox(bot, mins, maxs, -1, PRESENCE_CROUCH);
            if areas.is_null() {
                bot.botimport.Print.unwrap()(
                    PRT_MESSAGE,
                    c"trigger_multiple not in any area\n".as_ptr() as *mut c_char,
                );
            }
            //
            let mut link = areas;
            while !link.is_null() {
                if AAS_AreaTeleporter(bot, (*link).areanum) == 0 {
                    link = (*link).next_area;
                    continue;
                }
                //
                area1num = (*link).areanum;
                //create a new reachability link
                let lreach = AAS_AllocReachability(bot);
                if lreach.is_null() {
                    break;
                }
                (*lreach).areanum = area2num;
                (*lreach).facenum = 0;
                (*lreach).edgenum = 0;
                (*lreach).start = mid;
                (*lreach).end = destorigin;
                (*lreach).traveltype = TRAVEL_TELEPORT;
                (*lreach).traveltype |= AAS_TravelFlagsForTeam(bot, ent);
                (*lreach).traveltime = bot.aassettings.rs_teleport as c_ushort;
                (*lreach).next = *bot.areareachability.add(area1num as usize);
                *bot.areareachability.add(area1num as usize) = lreach;
                //
                bot.reach_teleport += 1;
                link = (*link).next_area;
            }
            //unlink the invalid entity
            AAS_UnlinkFromAreas(bot, areas);
            ent = AAS_NextBSPEntity(bot, ent);
        }
    }
}

/// Raven `AAS_Reachability_WeaponJump`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:3984-4101`
pub fn AAS_Reachability_WeaponJump(bot: &mut BotLib, area1num: c_int, area2num: c_int) -> c_int {
    unsafe {
        let mut face2num: c_int;
        let mut ret: c_int;
        let visualize: c_int;
        let mut speed: f32 = 0.0;
        let mut zvel: f32;
        let mut hordist: f32;
        let mut areastart: vec3_t = [0.0; 3];
        let mut facecenter: vec3_t = [0.0; 3];
        let mut start: vec3_t;
        let mut end: vec3_t;
        let mut dir: vec3_t;
        let mut cmdmove: vec3_t;
        let mut velocity: vec3_t = [0.0; 3];
        let mut r#move: aas_clientmove_t = core::mem::zeroed();
        let trace: aas_trace_t;

        visualize = qfalse;
        if AAS_AreaGrounded(bot, area1num) == 0 || AAS_AreaSwim(bot, area1num) != 0 {
            return qfalse;
        }
        if AAS_AreaGrounded(bot, area2num) == 0 {
            return qfalse;
        }
        //NOTE: only weapon jump towards areas with an interesting item in it??
        if (*bot.aasworld.areasettings.add(area2num as usize)).areaflags & AREA_WEAPONJUMP == 0 {
            return qfalse;
        }
        //
        let area1: *mut aas_area_t = bot.aasworld.areas.add(area1num as usize);
        let area2: *mut aas_area_t = bot.aasworld.areas.add(area2num as usize);
        //don't weapon jump towards way lower areas
        if (*area2).maxs[2] < (*area1).mins[2] {
            return qfalse;
        }
        //
        start = (*bot.aasworld.areas.add(area1num as usize)).center;
        //if not a swim area
        if AAS_PointAreaNum(bot, start) == 0 {
            Log_Write(
                bot,
                c"area %d center %f %f %f in solid?\r\n".as_ptr() as *mut c_char,
                area1num,
                start[0] as f64,
                start[1] as f64,
                start[2] as f64,
            );
        }
        end = start;
        end[2] -= 1000.0;
        trace = AAS_TraceClientBBox(bot, start, end, PRESENCE_CROUCH, -1);
        if trace.startsolid != 0 {
            return qfalse;
        }
        areastart = trace.endpos;
        //
        //areastart is now the start point
        //
        let mut i = 0;
        while i < (*area2).numfaces {
            face2num = *bot
                .aasworld
                .faceindex
                .add(((*area2).firstface + i) as usize);
            let face2: *mut aas_face_t = bot.aasworld.faces.add(face2num.unsigned_abs() as usize);
            //if it is not a solid face
            if (*face2).faceflags & FACE_GROUND == 0 {
                i += 1;
                continue;
            }
            //get the center of the face
            AAS_FaceCenter(bot, face2num, facecenter);
            //only go higher up with weapon jumps
            if facecenter[2] < areastart[2] + 64.0 {
                i += 1;
                continue;
            }
            //NOTE: set to 2 to allow bfg jump reachabilities
            let mut n = 0;
            while n < 1
            /*2*/
            {
                //get the rocket jump z velocity
                if n != 0 {
                    zvel = AAS_BFGJumpZVelocity(bot, areastart);
                } else {
                    zvel = AAS_RocketJumpZVelocity(bot, areastart);
                }
                //get the horizontal speed for the jump
                ret = AAS_HorizontalVelocityForJump(bot, zvel, areastart, facecenter, &mut speed);
                if ret != 0 && speed < 300.0 {
                    //direction towards the face center
                    dir = [
                        facecenter[0] - areastart[0],
                        facecenter[1] - areastart[1],
                        facecenter[2] - areastart[2],
                    ];
                    dir[2] = 0.0;
                    hordist = VectorNormalize(&mut dir);
                    {
                        //get command movement
                        cmdmove = [dir[0] * speed, dir[1] * speed, dir[2] * speed];
                        velocity = [0.0, 0.0, zvel];
                        //
                        AAS_PredictClientMovement(
                            bot,
                            &mut r#move,
                            -1,
                            areastart,
                            PRESENCE_NORMAL,
                            qtrue,
                            velocity,
                            cmdmove,
                            30,
                            30,
                            0.1,
                            SE_ENTERWATER
                                | SE_ENTERSLIME
                                | SE_ENTERLAVA
                                | SE_HITGROUNDDAMAGE
                                | SE_TOUCHJUMPPAD
                                | SE_HITGROUND
                                | SE_HITGROUNDAREA,
                            area2num,
                            visualize,
                        );
                        //if prediction time wasn't enough to fully predict the movement
                        if r#move.frames < 30
                            && (r#move.stopevent
                                & (SE_ENTERSLIME | SE_ENTERLAVA | SE_HITGROUNDDAMAGE)
                                == 0)
                            && (r#move.stopevent & (SE_HITGROUNDAREA | SE_TOUCHJUMPPAD) != 0)
                        {
                            //create a rocket or bfg jump reachability from area1 to area2
                            let lreach = AAS_AllocReachability(bot);
                            if lreach.is_null() {
                                return qfalse;
                            }
                            (*lreach).areanum = area2num;
                            (*lreach).facenum = 0;
                            (*lreach).edgenum = 0;
                            (*lreach).start = areastart;
                            (*lreach).end = facecenter;
                            if n != 0 {
                                (*lreach).traveltype = TRAVEL_BFGJUMP;
                                (*lreach).traveltime = bot.aassettings.rs_bfgjump as c_ushort;
                            } else {
                                (*lreach).traveltype = TRAVEL_ROCKETJUMP;
                                (*lreach).traveltime = bot.aassettings.rs_rocketjump as c_ushort;
                            }
                            (*lreach).next = *bot.areareachability.add(area1num as usize);
                            *bot.areareachability.add(area1num as usize) = lreach;
                            //
                            bot.reach_rocketjump += 1;
                            return qtrue;
                        }
                    }
                }
                n += 1;
            }
            i += 1;
        }
        //
        qfalse
    }
}

/// Raven `AAS_Reachability_JumpPad`.
///
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:3482-3767`
pub fn AAS_Reachability_JumpPad(bot: &mut BotLib) {
    unsafe {
        let mut face2num: c_int;
        let mut ret: c_int;
        let mut area2num: c_int;
        let visualize: c_int;
        let bot_visualizejumppads: c_int;
        let mut speed: f32 = 0.0;
        let mut zvel: f32;
        let mut hordist: f32;
        let mut facecenter: vec3_t = [0.0; 3];
        let mut dir: vec3_t;
        let mut cmdmove: vec3_t;
        let mut velocity: vec3_t = [0.0; 3];
        let mut absmins: vec3_t = [0.0; 3];
        let mut absmaxs: vec3_t = [0.0; 3];
        let mut areastart: vec3_t = [0.0; 3];
        let mut r#move: aas_clientmove_t = core::mem::zeroed();
        let mut classname: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];

        bot_visualizejumppads = LibVarValue(
            bot,
            c"bot_visualizejumppads".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
        ) as c_int;
        let mut ent = AAS_NextBSPEntity(bot, 0);
        while ent != 0 {
            if AAS_ValueForBSPEpairKey(
                bot,
                ent,
                c"classname".as_ptr() as *mut c_char,
                classname.as_mut_ptr(),
                MAX_EPAIRKEY,
            ) == 0
            {
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            if strcmp(classname.as_ptr(), c"trigger_push".as_ptr()) != 0 {
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            //
            if AAS_GetJumpPadInfo(bot, ent, areastart, absmins, absmaxs, velocity) == 0 {
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            //get the areas the jump pad brush is in
            let areas = AAS_LinkEntityClientBBox(bot, absmins, absmaxs, -1, PRESENCE_CROUCH);
            let mut link = areas;
            while !link.is_null() {
                if AAS_AreaJumpPad(bot, (*link).areanum) != 0 {
                    break;
                }
                link = (*link).next_area;
            }
            if link.is_null() {
                bot.botimport.Print.unwrap()(
                    PRT_MESSAGE,
                    c"trigger_push not in any jump pad area\n".as_ptr() as *mut c_char,
                );
                AAS_UnlinkFromAreas(bot, areas);
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            //
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"found a trigger_push with velocity %f %f %f\n".as_ptr() as *mut c_char,
                velocity[0] as f64,
                velocity[1] as f64,
                velocity[2] as f64,
            );
            //if there is a horizontal velocity check for a reachability without air control
            if velocity[0] != 0.0 || velocity[1] != 0.0 {
                cmdmove = [0.0, 0.0, 0.0];
                Com_Memset(
                    (&mut r#move as *mut aas_clientmove_t) as *mut (),
                    0,
                    core::mem::size_of::<aas_clientmove_t>(),
                );
                area2num = 0;
                let mut i = 0;
                while i < 20 {
                    AAS_PredictClientMovement(
                        bot,
                        &mut r#move,
                        -1,
                        areastart,
                        PRESENCE_NORMAL,
                        qfalse,
                        velocity,
                        cmdmove,
                        0,
                        30,
                        0.1,
                        SE_HITGROUND
                            | SE_ENTERWATER
                            | SE_ENTERSLIME
                            | SE_ENTERLAVA
                            | SE_HITGROUNDDAMAGE
                            | SE_TOUCHJUMPPAD
                            | SE_TOUCHTELEPORTER,
                        0,
                        bot_visualizejumppads,
                    );
                    area2num = r#move.endarea;
                    link = areas;
                    while !link.is_null() {
                        if AAS_AreaJumpPad(bot, (*link).areanum) == 0 {
                            link = (*link).next_area;
                            continue;
                        }
                        if (*link).areanum == area2num {
                            break;
                        }
                        link = (*link).next_area;
                    }
                    if link.is_null() {
                        break;
                    }
                    areastart = r#move.endpos;
                    velocity = r#move.velocity;
                    i += 1;
                }
                if area2num != 0 && i < 20 {
                    link = areas;
                    while !link.is_null() {
                        if AAS_AreaJumpPad(bot, (*link).areanum) == 0 {
                            link = (*link).next_area;
                            continue;
                        }
                        if AAS_ReachabilityExists(bot, (*link).areanum, area2num) != 0 {
                            link = (*link).next_area;
                            continue;
                        }
                        //create a rocket or bfg jump reachability from area1 to area2
                        let lreach = AAS_AllocReachability(bot);
                        if lreach.is_null() {
                            AAS_UnlinkFromAreas(bot, areas);
                            return;
                        }
                        (*lreach).areanum = area2num;
                        //NOTE: the facenum is the Z velocity
                        (*lreach).facenum = velocity[2] as c_int;
                        //NOTE: the edgenum is the horizontal velocity
                        (*lreach).edgenum = ((velocity[0] * velocity[0] + velocity[1] * velocity[1])
                            as f64)
                            .sqrt() as c_int;
                        (*lreach).start = areastart;
                        (*lreach).end = r#move.endpos;
                        (*lreach).traveltype = TRAVEL_JUMPPAD;
                        (*lreach).traveltype |= AAS_TravelFlagsForTeam(bot, ent);
                        (*lreach).traveltime = bot.aassettings.rs_jumppad as c_ushort;
                        (*lreach).next = *bot.areareachability.add((*link).areanum as usize);
                        *bot.areareachability.add((*link).areanum as usize) = lreach;
                        //
                        bot.reach_jumppad += 1;
                        link = (*link).next_area;
                    }
                }
            }
            //
            if velocity[0].abs() > 100.0 || velocity[1].abs() > 100.0 {
                ent = AAS_NextBSPEntity(bot, ent);
                continue;
            }
            //check for areas we can reach with air control
            area2num = 1;
            while area2num < bot.aasworld.numareas {
                visualize = qfalse;
                //never try to go back to one of the original jumppad areas
                //and don't create reachabilities if they already exist
                link = areas;
                while !link.is_null() {
                    if AAS_ReachabilityExists(bot, (*link).areanum, area2num) != 0 {
                        break;
                    }
                    if AAS_AreaJumpPad(bot, (*link).areanum) != 0 {
                        if (*link).areanum == area2num {
                            break;
                        }
                    }
                    link = (*link).next_area;
                }
                if !link.is_null() {
                    area2num += 1;
                    continue;
                }
                //
                let area2: *mut aas_area_t = bot.aasworld.areas.add(area2num as usize);
                let mut i = 0;
                while i < (*area2).numfaces {
                    face2num = *bot
                        .aasworld
                        .faceindex
                        .add(((*area2).firstface + i) as usize);
                    let face2: *mut aas_face_t =
                        bot.aasworld.faces.add(face2num.unsigned_abs() as usize);
                    //if it is not a ground face
                    if (*face2).faceflags & FACE_GROUND == 0 {
                        i += 1;
                        continue;
                    }
                    //get the center of the face
                    AAS_FaceCenter(bot, face2num, facecenter);
                    //only go higher up
                    if facecenter[2] < areastart[2] {
                        i += 1;
                        continue;
                    }
                    //get the jumppad jump z velocity
                    zvel = velocity[2];
                    //get the horizontal speed for the jump
                    ret =
                        AAS_HorizontalVelocityForJump(bot, zvel, areastart, facecenter, &mut speed);
                    if ret != 0 && speed < 150.0 {
                        //direction towards the face center
                        dir = [
                            facecenter[0] - areastart[0],
                            facecenter[1] - areastart[1],
                            facecenter[2] - areastart[2],
                        ];
                        dir[2] = 0.0;
                        hordist = VectorNormalize(&mut dir);
                        {
                            //get command movement
                            cmdmove = [dir[0] * speed, dir[1] * speed, dir[2] * speed];
                            //
                            AAS_PredictClientMovement(
                                bot,
                                &mut r#move,
                                -1,
                                areastart,
                                PRESENCE_NORMAL,
                                qfalse,
                                velocity,
                                cmdmove,
                                30,
                                30,
                                0.1,
                                SE_ENTERWATER
                                    | SE_ENTERSLIME
                                    | SE_ENTERLAVA
                                    | SE_HITGROUNDDAMAGE
                                    | SE_TOUCHJUMPPAD
                                    | SE_TOUCHTELEPORTER
                                    | SE_HITGROUNDAREA,
                                area2num,
                                visualize,
                            );
                            //if prediction time wasn't enough to fully predict the movement
                            if r#move.frames < 30
                                && (r#move.stopevent
                                    & (SE_ENTERSLIME | SE_ENTERLAVA | SE_HITGROUNDDAMAGE)
                                    == 0)
                                && (r#move.stopevent
                                    & (SE_HITGROUNDAREA | SE_TOUCHJUMPPAD | SE_TOUCHTELEPORTER)
                                    != 0)
                            {
                                //never go back to the same jumppad
                                link = areas;
                                while !link.is_null() {
                                    if (*link).areanum == r#move.endarea {
                                        break;
                                    }
                                    link = (*link).next_area;
                                }
                                if link.is_null() {
                                    link = areas;
                                    while !link.is_null() {
                                        if AAS_AreaJumpPad(bot, (*link).areanum) == 0 {
                                            link = (*link).next_area;
                                            continue;
                                        }
                                        if AAS_ReachabilityExists(bot, (*link).areanum, area2num)
                                            != 0
                                        {
                                            link = (*link).next_area;
                                            continue;
                                        }
                                        //create a jumppad reachability from area1 to area2
                                        let lreach = AAS_AllocReachability(bot);
                                        if lreach.is_null() {
                                            AAS_UnlinkFromAreas(bot, areas);
                                            return;
                                        }
                                        (*lreach).areanum = r#move.endarea;
                                        //NOTE: the facenum is the Z velocity
                                        (*lreach).facenum = velocity[2] as c_int;
                                        //NOTE: the edgenum is the horizontal velocity
                                        (*lreach).edgenum = ((cmdmove[0] * cmdmove[0]
                                            + cmdmove[1] * cmdmove[1])
                                            as f64)
                                            .sqrt()
                                            as c_int;
                                        (*lreach).start = areastart;
                                        (*lreach).end = facecenter;
                                        (*lreach).traveltype = TRAVEL_JUMPPAD;
                                        (*lreach).traveltype |= AAS_TravelFlagsForTeam(bot, ent);
                                        (*lreach).traveltime =
                                            bot.aassettings.rs_aircontrolledjumppad as c_ushort;
                                        (*lreach).next =
                                            *bot.areareachability.add((*link).areanum as usize);
                                        *bot.areareachability.add((*link).areanum as usize) =
                                            lreach;
                                        //
                                        bot.reach_jumppad += 1;
                                        link = (*link).next_area;
                                    }
                                }
                            }
                        }
                    }
                    i += 1;
                }
                area2num += 1;
            }
            AAS_UnlinkFromAreas(bot, areas);
            ent = AAS_NextBSPEntity(bot, ent);
        }
    }
}

/// Raven `AAS_ContinueInitReachability`.
///
/// PORT-NOTE(statics): the `framereachability`/`reachability_delay`/
/// `lastpercentage` function-scope statics are genuine cross-frame state
/// (fork-3 kind 3) → threaded fields on `bot`.
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:4347-4487`
pub fn AAS_ContinueInitReachability(bot: &mut BotLib, time: f32) -> c_int {
    unsafe {
        if bot.aasworld.loaded == 0 {
            return qfalse;
        }
        //if reachability is calculated for all areas
        if bot.aasworld.numreachabilityareas >= bot.aasworld.numareas + 2 {
            return qfalse;
        }
        //if starting with area 1 (area 0 is a dummy)
        if bot.aasworld.numreachabilityareas == 1 {
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"calculating reachability...\n".as_ptr() as *mut c_char,
            );
            bot.lastpercentage = 0;
            bot.framereachability = 2000.0;
            bot.reachability_delay = 1000.0;
        }
        //number of areas to calculate reachability for this cycle
        let todo = bot.aasworld.numreachabilityareas + bot.framereachability as c_int;
        let start_time = Sys_MilliSeconds();
        //loop over the areas
        let mut i = bot.aasworld.numreachabilityareas;
        while i < bot.aasworld.numareas && i < todo {
            bot.aasworld.numreachabilityareas += 1;
            //only create jumppad reachabilities from jumppad areas
            if (*bot.aasworld.areasettings.add(i as usize)).contents & AREACONTENTS_JUMPPAD != 0 {
                i += 1;
                continue;
            }
            //loop over the areas
            let mut j = 1;
            while j < bot.aasworld.numareas {
                if i == j {
                    j += 1;
                    continue;
                }
                //never create reachabilities from teleporter or jumppad areas to regular areas
                if (*bot.aasworld.areasettings.add(i as usize)).contents
                    & (AREACONTENTS_TELEPORTER | AREACONTENTS_JUMPPAD)
                    != 0
                {
                    if (*bot.aasworld.areasettings.add(j as usize)).contents
                        & (AREACONTENTS_TELEPORTER | AREACONTENTS_JUMPPAD)
                        == 0
                    {
                        j += 1;
                        continue;
                    }
                }
                //if there already is a reachability link from area i to j
                if AAS_ReachabilityExists(bot, i, j) != 0 {
                    j += 1;
                    continue;
                }
                //check for a swim reachability
                if AAS_Reachability_Swim(bot, i, j) != 0 {
                    j += 1;
                    continue;
                }
                //check for a simple walk on equal floor height reachability
                if AAS_Reachability_EqualFloorHeight(bot, i, j) != 0 {
                    j += 1;
                    continue;
                }
                //check for step, barrier, waterjump and walk off ledge reachabilities
                if AAS_Reachability_Step_Barrier_WaterJump_WalkOffLedge(bot, i, j) != 0 {
                    j += 1;
                    continue;
                }
                //check for ladder reachabilities
                if AAS_Reachability_Ladder(bot, i, j) != 0 {
                    j += 1;
                    continue;
                }
                //check for a jump reachability
                if AAS_Reachability_Jump(bot, i, j) != 0 {
                    j += 1;
                    continue;
                }
                j += 1;
            }
            //never create these reachabilities from teleporter or jumppad areas
            if (*bot.aasworld.areasettings.add(i as usize)).contents
                & (AREACONTENTS_TELEPORTER | AREACONTENTS_JUMPPAD)
                != 0
            {
                i += 1;
                continue;
            }
            //loop over the areas
            let mut j = 1;
            while j < bot.aasworld.numareas {
                if i == j {
                    j += 1;
                    continue;
                }
                //
                if AAS_ReachabilityExists(bot, i, j) != 0 {
                    j += 1;
                    continue;
                }
                //check for a grapple hook reachability
                if bot.calcgrapplereach != 0 {
                    AAS_Reachability_Grapple(bot, i, j);
                }
                //check for a weapon jump reachability
                AAS_Reachability_WeaponJump(bot, i, j);
                j += 1;
            }
            //if the calculation took more time than the max reachability delay
            if Sys_MilliSeconds() - start_time > bot.reachability_delay as c_int {
                break;
            }
            //
            if bot.aasworld.numreachabilityareas * 1000 / bot.aasworld.numareas > bot.lastpercentage
            {
                break;
            }
            i += 1;
        }
        //
        if bot.aasworld.numreachabilityareas == bot.aasworld.numareas {
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"\r%6.1f%%".as_ptr() as *mut c_char,
                100.0_f64,
            );
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"\nplease wait while storing reachability...\n".as_ptr() as *mut c_char,
            );
            bot.aasworld.numreachabilityareas += 1;
        }
        //if this is the last step in the reachability calculations
        else if bot.aasworld.numreachabilityareas == bot.aasworld.numareas + 1 {
            //create additional walk off ledge reachabilities for every area
            let mut i = 1;
            while i < bot.aasworld.numareas {
                //only create jumppad reachabilities from jumppad areas
                if (*bot.aasworld.areasettings.add(i as usize)).contents & AREACONTENTS_JUMPPAD != 0
                {
                    i += 1;
                    continue;
                }
                AAS_Reachability_WalkOffLedge(bot, i);
                i += 1;
            }
            //create jump pad reachabilities
            AAS_Reachability_JumpPad(bot);
            //create teleporter reachabilities
            AAS_Reachability_Teleport(bot);
            //create elevator (func_plat) reachabilities
            AAS_Reachability_Elevator(bot);
            //create func_bobbing reachabilities
            AAS_Reachability_FuncBobbing(bot);
            //
            //store all the reachabilities
            AAS_StoreReachability(bot);
            //free the reachability link heap
            AAS_ShutDownReachabilityHeap(bot);
            //
            FreeMemory(bot, bot.areareachability as *mut ());
            //
            bot.aasworld.numreachabilityareas += 1;
            //
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"calculating clusters...\n".as_ptr() as *mut c_char,
            );
        } else {
            bot.lastpercentage = bot.aasworld.numreachabilityareas * 1000 / bot.aasworld.numareas;
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"\r%6.1f%%".as_ptr() as *mut c_char,
                (bot.lastpercentage as f32 / 10.0) as f64,
            );
        }
        //not yet finished
        qtrue
    }
}
