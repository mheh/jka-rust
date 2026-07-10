#![allow(non_snake_case, non_camel_case_types)]

//! MP botlib `be_aas_sample.cpp` — AAS environment sampling (presence-type
//! bounding boxes, area/point queries, plane/face geometry, entity-area
//! linking, and client-bbox tracing through the AAS BSP tree).
//!
//! DESTINATION NOTE: the packet order names
//! `crates/mp/engine/botlib/src/be_aas_sample.rs`, but `be_aas_sample`
//! already exists as a directory module (`be_aas_sample/mod.rs`,
//! constants-only) — so this file lands at the `_fns` escape per
//! `_PREAMBLE.md`'s destination rule.
//!
//! PORT-NOTE(callee-signatures): several in-engine callees this file
//! reaches (`FreeMemory`/`GetClearedHunkMemory`/`GetHunkMemory`/
//! `LibVarValue`/`AAS_AreaReachability`/`AAS_EntityCollision`/
//! `Com_Memset`) are ported in sibling files/packets not linked here yet;
//! their signatures are the faithful shape inferred from the Raven call
//! sites (receivers per the packets' RESOLVED CALL SURFACE tables),
//! matching the established `extern "Rust"` forward-declare convention
//! used elsewhere in this crate (e.g. `be_ai_chat_fns.rs`).

use core::ffi::{c_int, c_void};

use mp_qshared::common::mp::botlib::aas_trace_s::aas_trace_t;
use mp_qshared::common::mp::botlib::bsp_trace_s::bsp_trace_t;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_FATAL, PRT_MESSAGE};
use mp_qshared::common::mp::qcommon::aas_areainfo::aas_areainfo_t;
use mp_qshared::shared::surface_flags::{CONTENTS_PLAYERCLIP, CONTENTS_SOLID};
use mp_qshared::shared::vec3_t;

use mp_qshared::shared::{qboolean, qfalse, qtrue};

use crate::aasfile::aas_area_s::aas_area_t;
use crate::aasfile::aas_edge_s::aas_edge_t;
use crate::aasfile::aas_face_s::aas_face_t;
use crate::aasfile::aas_node_s::aas_node_t;
use crate::aasfile::aas_plane_s::aas_plane_t;
use crate::aasfile::face_flags::FACE_GROUND;
use crate::aasfile::presence_type::{PRESENCE_CROUCH, PRESENCE_NONE, PRESENCE_NORMAL};
use crate::be_aas_bspq3::be_aas_bspq3_cpp_consts::ON_EPSILON;
use crate::be_aas_def::aas_link_s::aas_link_t;
use crate::be_aas_sample::be_aas_sample_cpp_consts::{BBOX_NORMAL_EPSILON, TRACEPLANE_EPSILON};
use crate::BotLib;

// ---------------------------------------------------------------------
// Externally-ported callees this file reaches (signatures inferred from
// the Raven call sites; ported in sibling packets outside this shard).
// PORT-NOTE(callee-signatures): see module doc comment.
// ---------------------------------------------------------------------
// PORT-NOTE(macros): Raven's `DotProduct`/`VectorCopy`/`VectorSubtract`/
// `VectorClear`/`VectorMA`/`AAS_OrthogonalToVectors` are `#define`s; they
// expand inline here, faithful to the preprocessor (matching the
// `be_aas_reach_fns.rs` convention). `CrossProduct`/`VectorLength`/
// `VectorNormalize`/`VectorInverse` are genuine q_math functions the packets
// flag as externals, called through the not-yet-wired q_math surface — see
// missing_symbols.
#[inline]
fn DotProduct(a: vec3_t, b: vec3_t) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

extern "Rust" {
    fn CrossProduct(v1: vec3_t, v2: vec3_t, cross: &mut vec3_t);
    fn VectorLength(v: vec3_t) -> f32;
    fn VectorNormalize(v: &mut vec3_t) -> f32;
    fn VectorInverse(v: &mut vec3_t);
    fn FreeMemory(bot: &mut BotLib, ptr: *mut c_void);
    fn GetClearedHunkMemory(bot: &mut BotLib, size: usize) -> *mut c_void;
    fn GetHunkMemory(bot: &mut BotLib, size: usize) -> *mut c_void;
    fn LibVarValue(
        bot: &mut BotLib,
        name: *const core::ffi::c_char,
        default: *const core::ffi::c_char,
    ) -> f32;
    fn AAS_AreaReachability(bot: &mut BotLib, areanum: c_int) -> c_int;
    fn AAS_EntityCollision(
        bot: &mut BotLib,
        entnum: c_int,
        start: vec3_t,
        boxmins: vec3_t,
        boxmaxs: vec3_t,
        end: vec3_t,
        contentmask: c_int,
        trace: *mut bsp_trace_t,
    ) -> qboolean;
    fn Com_Memset(dst: *mut c_void, val: c_int, n: usize);
}

/// A stack entry used while walking the AAS BSP tree during a line trace.
///
/// Raven `aas_tracestack_t` — a function-local (`be_aas_sample.cpp`) helper
/// struct, not part of the AAS file format; not in the type rosetta, so it
/// is defined here beside its sole users (`AAS_TraceAreas`/`AAS_TraceClientBBox`).
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp` (local `aas_tracestack_t`).
#[derive(Clone, Copy)]
struct aas_tracestack_t {
    start: vec3_t,
    end: vec3_t,
    planenum: c_int,
    nodenum: c_int,
}

/// A stack entry used while walking the AAS BSP tree during entity linking.
///
/// Raven `aas_linkstack_t` — a function-local helper struct (same treatment
/// as `aas_tracestack_t` above).
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp` (local `aas_linkstack_t`).
#[derive(Clone, Copy)]
struct aas_linkstack_t {
    nodenum: c_int,
}

/// Raven `AAS_PresenceTypeBoundingBox` — bounding box size for each presence type.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:56-72`
pub fn AAS_PresenceTypeBoundingBox(
    bot: &mut BotLib,
    presencetype: c_int,
    mins: vec3_t,
    maxs: vec3_t,
) {
    //bounding box size for each presence type
    let boxmins: [vec3_t; 3] = [
        [0.0, 0.0, 0.0],
        [-15.0, -15.0, -24.0],
        [-15.0, -15.0, -24.0],
    ];
    let boxmaxs: [vec3_t; 3] = [[0.0, 0.0, 0.0], [15.0, 15.0, 32.0], [15.0, 15.0, 8.0]];

    let index: usize;
    if presencetype == PRESENCE_NORMAL {
        index = 1;
    } else if presencetype == PRESENCE_CROUCH {
        index = 2;
    } else {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"AAS_PresenceTypeBoundingBox: unknown presence type\n".as_ptr() as *mut _,
            );
        }
        index = 2;
    }
    let mut mins = mins;
    let mut maxs = maxs;
    mins = boxmins[index];
    maxs = boxmaxs[index];
    let _ = (mins, maxs);
}

/// Raven `AAS_AllocAASLink`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:129-148`
pub fn AAS_AllocAASLink(bot: &mut BotLib) -> *mut aas_link_t {
    unsafe {
        let link = bot.aasworld.freelinks;
        if link.is_null() {
            #[cfg(not(feature = "bspc"))]
            if bot.bot_developer != 0 {
                bot.botimport.Print.unwrap()(
                    PRT_FATAL,
                    c"empty aas link heap\n".as_ptr() as *mut _,
                );
            }
            return core::ptr::null_mut();
        }
        if !bot.aasworld.freelinks.is_null() {
            bot.aasworld.freelinks = (*bot.aasworld.freelinks).next_ent;
        }
        if !bot.aasworld.freelinks.is_null() {
            (*bot.aasworld.freelinks).prev_ent = core::ptr::null_mut();
        }
        bot.numaaslinks -= 1;
        link
    }
}

/// Raven `AAS_DeAllocAASLink`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:155-164`
pub fn AAS_DeAllocAASLink(bot: &mut BotLib, link: *mut aas_link_t) {
    unsafe {
        if !bot.aasworld.freelinks.is_null() {
            (*bot.aasworld.freelinks).prev_ent = link;
        }
        (*link).prev_ent = core::ptr::null_mut();
        (*link).next_ent = bot.aasworld.freelinks;
        (*link).prev_area = core::ptr::null_mut();
        (*link).next_area = core::ptr::null_mut();
        bot.aasworld.freelinks = link;
        bot.numaaslinks += 1;
    }
}

/// Raven `AAS_PointAreaNum`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:196-242`
pub fn AAS_PointAreaNum(bot: &mut BotLib, point: vec3_t) -> c_int {
    unsafe {
        if bot.aasworld.loaded == 0 {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"AAS_PointAreaNum: aas not loaded\n".as_ptr() as *mut _,
            );
            return 0;
        }

        //start with node 1 because node zero is a dummy used for solid leafs
        let mut nodenum: c_int = 1;
        while nodenum > 0 {
            let node: *mut aas_node_t = bot.aasworld.nodes.add(nodenum as usize);
            let plane: *mut aas_plane_t = bot.aasworld.planes.add((*node).planenum as usize);
            let dist = DotProduct(point, (*plane).normal) - (*plane).dist;
            if dist > 0.0 {
                nodenum = (*node).children[0];
            } else {
                nodenum = (*node).children[1];
            }
        }
        if nodenum == 0 {
            return 0;
        }
        -nodenum
    }
}

/// Raven `AAS_AreaCluster`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:291-299`
pub fn AAS_AreaCluster(bot: &mut BotLib, areanum: c_int) -> c_int {
    unsafe {
        if areanum <= 0 || areanum >= bot.aasworld.numareas {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"AAS_AreaCluster: invalid area number\n".as_ptr() as *mut _,
            );
            return 0;
        }
        (*bot.aasworld.areasettings.add(areanum as usize)).cluster
    }
}

/// Raven `AAS_AreaPresenceType`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:307-316`
pub fn AAS_AreaPresenceType(bot: &mut BotLib, areanum: c_int) -> c_int {
    unsafe {
        if bot.aasworld.loaded == 0 {
            return 0;
        }
        if areanum <= 0 || areanum >= bot.aasworld.numareas {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"AAS_AreaPresenceType: invalid area number\n".as_ptr() as *mut _,
            );
            return 0;
        }
        (*bot.aasworld.areasettings.add(areanum as usize)).presencetype
    }
}

/// Raven `AAS_BoxOriginDistanceFromPlane`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:349-382`
pub fn AAS_BoxOriginDistanceFromPlane(
    normal: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    side: c_int,
) -> f32 {
    let mut v1: vec3_t = [0.0; 3];

    //swap maxs and mins when on the other side of the plane
    if side != 0 {
        //get a point of the box that would be one of the first
        //to collide with the plane
        for i in 0..3 {
            if normal[i] > BBOX_NORMAL_EPSILON {
                v1[i] = maxs[i];
            } else if normal[i] < -BBOX_NORMAL_EPSILON {
                v1[i] = mins[i];
            } else {
                v1[i] = 0.0;
            }
        }
    } else {
        //get a point of the box that would be one of the first
        //to collide with the plane
        for i in 0..3 {
            if normal[i] > BBOX_NORMAL_EPSILON {
                v1[i] = mins[i];
            } else if normal[i] < -BBOX_NORMAL_EPSILON {
                v1[i] = maxs[i];
            } else {
                v1[i] = 0.0;
            }
        }
    }
    //
    let mut v2 = normal;
    unsafe {
        VectorInverse(&mut v2);
    }
    DotProduct(v1, v2)
}

/// Raven `AAS_TraceAreas`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:709-887`
pub fn AAS_TraceAreas(
    bot: &mut BotLib,
    start: vec3_t,
    end: vec3_t,
    areas: *mut c_int,
    points: *mut vec3_t,
    maxareas: c_int,
) -> c_int {
    unsafe {
        let mut numareas: c_int = 0;
        *areas = 0;
        if bot.aasworld.loaded == 0 {
            return numareas;
        }

        // §19: Raven leaves `tracestack` uninitialized before the first
        // write at `tracestack[0]`; each entry is fully written before use.
        let mut tracestack = [aas_tracestack_t {
            start: [0.0; 3],
            end: [0.0; 3],
            planenum: 0,
            nodenum: 0,
        }; 127];
        let mut tstack_p: usize = 0;
        //we start with the whole line on the stack
        tracestack[0].start = start;
        tracestack[0].end = end;
        tracestack[0].planenum = 0;
        //start with node 1 because node zero is a dummy for a solid leaf
        tracestack[0].nodenum = 1; //starting at the root of the tree
        tstack_p += 1;

        loop {
            //pop up the stack
            if tstack_p == 0 {
                return numareas;
            }
            tstack_p -= 1;
            let nodenum = tracestack[tstack_p].nodenum;
            //if it is an area
            if nodenum < 0 {
                areas.add(numareas as usize).write(-nodenum);
                if !points.is_null() {
                    *points.add(numareas as usize) = tracestack[tstack_p].start;
                }
                numareas += 1;
                if numareas >= maxareas {
                    return numareas;
                }
                continue;
            }
            //if it is a solid leaf
            if nodenum == 0 {
                continue;
            }
            //the node to test against
            let aasnode: *mut aas_node_t = bot.aasworld.nodes.add(nodenum as usize);
            //start point of current line to test against node
            let cur_start = tracestack[tstack_p].start;
            //end point of the current line to test against node
            let cur_end = tracestack[tstack_p].end;
            //the current node plane
            let plane: *mut aas_plane_t = bot.aasworld.planes.add((*aasnode).planenum as usize);

            let front = DotProduct(cur_start, (*plane).normal) - (*plane).dist;
            let back = DotProduct(cur_end, (*plane).normal) - (*plane).dist;

            //if the whole to be traced line is totally at the front of this node
            //only go down the tree with the front child
            if front > 0.0 && back > 0.0 {
                //keep the current start and end point on the stack
                //and go down the tree with the front child
                tracestack[tstack_p].nodenum = (*aasnode).children[0];
                tstack_p += 1;
                if tstack_p >= 127 {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"AAS_TraceAreas: stack overflow\n".as_ptr() as *mut _,
                    );
                    return numareas;
                }
            }
            //if the whole to be traced line is totally at the back of this node
            //only go down the tree with the back child
            else if front <= 0.0 && back <= 0.0 {
                //keep the current start and end point on the stack
                //and go down the tree with the back child
                tracestack[tstack_p].nodenum = (*aasnode).children[1];
                tstack_p += 1;
                if tstack_p >= 127 {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"AAS_TraceAreas: stack overflow\n".as_ptr() as *mut _,
                    );
                    return numareas;
                }
            }
            //go down the tree both at the front and back of the node
            else {
                let tmpplanenum = tracestack[tstack_p].planenum;
                //calculate the hitpoint with the node (split point of the line)
                //put the crosspoint TRACEPLANE_EPSILON pixels on the near side
                let mut frac = front / (front - back);
                if frac < 0.0 {
                    frac = 0.0;
                } else if frac > 1.0 {
                    frac = 1.0;
                }
                //
                let cur_mid: vec3_t = [
                    cur_start[0] + (cur_end[0] - cur_start[0]) * frac,
                    cur_start[1] + (cur_end[1] - cur_start[1]) * frac,
                    cur_start[2] + (cur_end[2] - cur_start[2]) * frac,
                ];

                //side the front part of the line is on
                let side = (front < 0.0) as usize;
                //first put the end part of the line on the stack (back side)
                tracestack[tstack_p].start = cur_mid;
                //not necesary to store because still on stack
                tracestack[tstack_p].planenum = (*aasnode).planenum;
                tracestack[tstack_p].nodenum = (*aasnode).children[1 - side];
                tstack_p += 1;
                if tstack_p >= 127 {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"AAS_TraceAreas: stack overflow\n".as_ptr() as *mut _,
                    );
                    return numareas;
                }
                //now put the part near the start of the line on the stack so we will
                //continue with thats part first. This way we'll find the first
                //hit of the bbox
                tracestack[tstack_p].start = cur_start;
                tracestack[tstack_p].end = cur_mid;
                tracestack[tstack_p].planenum = tmpplanenum;
                tracestack[tstack_p].nodenum = (*aasnode).children[side];
                tstack_p += 1;
                if tstack_p >= 127 {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"AAS_TraceAreas: stack overflow\n".as_ptr() as *mut _,
                    );
                    return numareas;
                }
            }
        }
    }
}

/// Raven `AAS_InsideFace`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:909-954`
pub fn AAS_InsideFace(
    bot: &mut BotLib,
    face: *mut aas_face_t,
    pnormal: vec3_t,
    point: vec3_t,
    epsilon: f32,
) -> qboolean {
    unsafe {
        if bot.aasworld.loaded == 0 {
            return qfalse;
        }

        for i in 0..(*face).numedges {
            let edgenum = *bot.aasworld.edgeindex.add(((*face).firstedge + i) as usize);
            let edge: *mut aas_edge_t = bot.aasworld.edges.add(edgenum.unsigned_abs() as usize);
            //get the first vertex of the edge
            let firstvertex = (edgenum < 0) as usize;
            let v0 = *bot.aasworld.vertexes.add((*edge).v[firstvertex] as usize);
            //edge vector
            let v1 = *bot
                .aasworld
                .vertexes
                .add((*edge).v[1 - firstvertex] as usize);
            let edgevec: vec3_t = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            //vector from first edge point to point possible in face
            let pointvec: vec3_t = [point[0] - v0[0], point[1] - v0[1], point[2] - v0[2]];
            //get a vector pointing inside the face orthogonal to both the
            //edge vector and the normal vector of the plane the face is in
            //this vector defines a plane through the origin (first vertex of
            //edge) and through both the edge vector and the normal vector
            //of the plane
            //AAS_OrthogonalToVectors(edgevec, pnormal, sepnormal) — macro, inlined
            let sepnormal: vec3_t = [
                edgevec[1] * pnormal[2] - edgevec[2] * pnormal[1],
                edgevec[2] * pnormal[0] - edgevec[0] * pnormal[2],
                edgevec[0] * pnormal[1] - edgevec[1] * pnormal[0],
            ];
            //check on wich side of the above plane the point is
            //this is done by checking the sign of the dot product of the
            //vector orthogonal vector from above and the vector from the
            //origin (first vertex of edge) to the point
            //if the dotproduct is smaller than zero the point is outside the face
            if DotProduct(pointvec, sepnormal) < -epsilon {
                return qfalse;
            }
        }
        qtrue
    }
}

/// Raven `AAS_PointInsideFace`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:961-993`
pub fn AAS_PointInsideFace(
    bot: &mut BotLib,
    facenum: c_int,
    point: vec3_t,
    epsilon: f32,
) -> qboolean {
    unsafe {
        if bot.aasworld.loaded == 0 {
            return qfalse;
        }

        let face: *mut aas_face_t = bot.aasworld.faces.add(facenum as usize);
        let plane: *mut aas_plane_t = bot.aasworld.planes.add((*face).planenum as usize);
        //
        for i in 0..(*face).numedges {
            let edgenum = *bot.aasworld.edgeindex.add(((*face).firstedge + i) as usize);
            let edge: *mut aas_edge_t = bot.aasworld.edges.add(edgenum.unsigned_abs() as usize);
            //get the first vertex of the edge
            let firstvertex = (edgenum < 0) as usize;
            let v1 = bot.aasworld.vertexes.add((*edge).v[firstvertex] as usize);
            let v2 = bot
                .aasworld
                .vertexes
                .add((*edge).v[1 - firstvertex] as usize);
            //edge vector
            let edgevec: vec3_t = [
                (*v2)[0] - (*v1)[0],
                (*v2)[1] - (*v1)[1],
                (*v2)[2] - (*v1)[2],
            ];
            //vector from first edge point to point possible in face
            let pointvec: vec3_t = [
                point[0] - (*v1)[0],
                point[1] - (*v1)[1],
                point[2] - (*v1)[2],
            ];
            //
            let mut sepnormal: vec3_t = [0.0; 3];
            CrossProduct(edgevec, (*plane).normal, &mut sepnormal);
            //
            if DotProduct(pointvec, sepnormal) < -epsilon {
                return qfalse;
            }
        }
        qtrue
    }
}

/// Raven `AAS_FacePlane`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:1035-1042`
pub fn AAS_FacePlane(bot: &mut BotLib, facenum: c_int, mut normal: vec3_t, dist: *mut f32) {
    unsafe {
        let plane: *mut aas_plane_t = bot
            .aasworld
            .planes
            .add((*bot.aasworld.faces.add(facenum as usize)).planenum as usize);
        normal = (*plane).normal;
        *dist = (*plane).dist;
        let _ = normal;
    }
}

/// Raven `AAS_BoxOnPlaneSide2`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:1110-1136`
pub fn AAS_BoxOnPlaneSide2(absmins: vec3_t, absmaxs: vec3_t, p: *mut aas_plane_t) -> c_int {
    unsafe {
        let mut corners: [vec3_t; 2] = [[0.0; 3]; 2];

        for i in 0..3 {
            if (*p).normal[i] < 0.0 {
                corners[0][i] = absmins[i];
                corners[1][i] = absmaxs[i];
            } else {
                corners[1][i] = absmins[i];
                corners[0][i] = absmaxs[i];
            }
        }
        let dist1 = DotProduct((*p).normal, corners[0]) - (*p).dist;
        let dist2 = DotProduct((*p).normal, corners[1]) - (*p).dist;
        let mut sides = 0;
        if dist1 >= 0.0 {
            sides = 1;
        }
        if dist2 < 0.0 {
            sides |= 2;
        }

        sides
    }
}

/// Raven `AAS_AreaInfo`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:1346-1365`
pub fn AAS_AreaInfo(bot: &mut BotLib, areanum: c_int, info: *mut aas_areainfo_t) -> c_int {
    unsafe {
        if info.is_null() {
            return 0;
        }
        if areanum <= 0 || areanum >= bot.aasworld.numareas {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"AAS_AreaInfo: areanum %d out of range\n".as_ptr() as *mut _,
                areanum,
            );
            return 0;
        }
        let settings = bot.aasworld.areasettings.add(areanum as usize);
        (*info).cluster = (*settings).cluster;
        (*info).contents = (*settings).contents;
        (*info).flags = (*settings).areaflags;
        (*info).presencetype = (*settings).presencetype;
        (*info).mins = (*bot.aasworld.areas.add(areanum as usize)).mins;
        (*info).maxs = (*bot.aasworld.areas.add(areanum as usize)).maxs;
        (*info).center = (*bot.aasworld.areas.add(areanum as usize)).center;
        core::mem::size_of::<aas_areainfo_t>() as c_int
    }
}

/// Raven `AAS_PlaneFromNum`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:1372-1377`
pub fn AAS_PlaneFromNum(bot: &mut BotLib, planenum: c_int) -> *mut aas_plane_t {
    unsafe {
        if bot.aasworld.loaded == 0 {
            return core::ptr::null_mut();
        }
        bot.aasworld.planes.add(planenum as usize)
    }
}

/// Raven `AAS_FreeAASLinkHeap`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:117-122`
pub fn AAS_FreeAASLinkHeap(bot: &mut BotLib) {
    unsafe {
        if !bot.aasworld.linkheap.is_null() {
            FreeMemory(bot, bot.aasworld.linkheap as *mut c_void);
        }
        bot.aasworld.linkheap = core::ptr::null_mut();
        bot.aasworld.linkheapsize = 0;
    }
}

/// Raven `AAS_FreeAASLinkedEntities`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:184-188`
pub fn AAS_FreeAASLinkedEntities(bot: &mut BotLib) {
    unsafe {
        if !bot.aasworld.arealinkedentities.is_null() {
            FreeMemory(bot, bot.aasworld.arealinkedentities as *mut c_void);
        }
        bot.aasworld.arealinkedentities = core::ptr::null_mut();
    }
}

/// Raven `AAS_PointPresenceType`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:324-333`
pub fn AAS_PointPresenceType(bot: &mut BotLib, point: vec3_t) -> c_int {
    unsafe {
        if bot.aasworld.loaded == 0 {
            return 0;
        }

        let areanum = AAS_PointAreaNum(bot, point);
        if areanum == 0 {
            return PRESENCE_NONE;
        }
        (*bot.aasworld.areasettings.add(areanum as usize)).presencetype
    }
}

/// Raven `AAS_AreaGroundFace`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:1001-1027`
pub fn AAS_AreaGroundFace(bot: &mut BotLib, areanum: c_int, point: vec3_t) -> *mut aas_face_t {
    unsafe {
        let up: vec3_t = [0.0, 0.0, 1.0];

        if bot.aasworld.loaded == 0 {
            return core::ptr::null_mut();
        }

        let area: *mut aas_area_t = bot.aasworld.areas.add(areanum as usize);
        for i in 0..(*area).numfaces {
            let facenum = *bot.aasworld.faceindex.add(((*area).firstface + i) as usize);
            let face: *mut aas_face_t = bot.aasworld.faces.add(facenum.unsigned_abs() as usize);
            //if this is a ground face
            if (*face).faceflags & FACE_GROUND != 0 {
                //get the up or down normal
                let normal: vec3_t =
                    if (*bot.aasworld.planes.add((*face).planenum as usize)).normal[2] < 0.0 {
                        [-up[0], -up[1], -up[2]]
                    } else {
                        up
                    };
                //check if the point is in the face
                if AAS_InsideFace(bot, face, normal, point, 0.01) != 0 {
                    return face;
                }
            }
        }
        core::ptr::null_mut()
    }
}

/// Raven `AAS_TraceEndFace`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:1050-1103`
pub fn AAS_TraceEndFace(bot: &mut BotLib, trace: *mut aas_trace_t) -> *mut aas_face_t {
    unsafe {
        let firstface: *mut aas_face_t = core::ptr::null_mut();

        if bot.aasworld.loaded == 0 {
            return core::ptr::null_mut();
        }

        //if started in solid no face was hit
        if (*trace).startsolid != 0 {
            return core::ptr::null_mut();
        }
        //trace->lastarea is the last area the trace was in
        let area: *mut aas_area_t = bot.aasworld.areas.add((*trace).lastarea as usize);
        //check which face the trace.endpos was in
        for i in 0..(*area).numfaces {
            let facenum = *bot.aasworld.faceindex.add(((*area).firstface + i) as usize);
            let face: *mut aas_face_t = bot.aasworld.faces.add(facenum.unsigned_abs() as usize);
            //if the face is in the same plane as the trace end point
            if ((*face).planenum & !1) == ((*trace).planenum & !1) {
                //firstface is used for optimization, if theres only one
                //face in the plane then it has to be the good one
                //if there are more faces in the same plane then always
                //check the one with the fewest edges first
                if AAS_InsideFace(
                    bot,
                    face,
                    (*bot.aasworld.planes.add((*face).planenum as usize)).normal,
                    (*trace).endpos,
                    0.01,
                ) != 0
                {
                    return face;
                }
            }
        }
        firstface
    }
}

/// Raven `AAS_UnlinkFromAreas`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:1175-1190`
pub fn AAS_UnlinkFromAreas(bot: &mut BotLib, areas: *mut aas_link_t) {
    unsafe {
        let mut link = areas;
        while !link.is_null() {
            //next area the entity is linked in
            let nextlink = (*link).next_area;
            //remove the entity from the linked list of this area
            if !(*link).prev_ent.is_null() {
                (*(*link).prev_ent).next_ent = (*link).next_ent;
            } else {
                *bot.aasworld
                    .arealinkedentities
                    .add((*link).areanum as usize) = (*link).next_ent;
            }
            if !(*link).next_ent.is_null() {
                (*(*link).next_ent).prev_ent = (*link).prev_ent;
            }
            //deallocate the link structure
            AAS_DeAllocAASLink(bot, link);
            link = nextlink;
        }
    }
}

/// Raven `AAS_AASLinkEntity`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:1206-1299`
pub fn AAS_AASLinkEntity(
    bot: &mut BotLib,
    absmins: vec3_t,
    absmaxs: vec3_t,
    entnum: c_int,
) -> *mut aas_link_t {
    unsafe {
        if bot.aasworld.loaded == 0 {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"AAS_LinkEntity: aas not loaded\n".as_ptr() as *mut _,
            );
            return core::ptr::null_mut();
        }

        let mut areas: *mut aas_link_t = core::ptr::null_mut();
        //
        // §19: Raven leaves `linkstack` uninitialized before the first
        // write at `linkstack[0]`; each entry is fully written before use.
        let mut linkstack = [aas_linkstack_t { nodenum: 0 }; 128];
        let mut lstack_p: usize = 0;
        //we start with the whole line on the stack
        //start with node 1 because node zero is a dummy used for solid leafs
        linkstack[0].nodenum = 1; //starting at the root of the tree
        lstack_p += 1;

        loop {
            //pop up the stack
            if lstack_p == 0 {
                break;
            }
            lstack_p -= 1;
            //number of the current node to test the line against
            let nodenum = linkstack[lstack_p].nodenum;
            //if it is an area
            if nodenum < 0 {
                //NOTE: the entity might have already been linked into this area
                // because several node children can point to the same area
                let mut link = *bot.aasworld.arealinkedentities.add((-nodenum) as usize);
                while !link.is_null() {
                    if (*link).entnum == entnum {
                        break;
                    }
                    link = (*link).next_ent;
                }
                if !link.is_null() {
                    continue;
                }
                //
                let link = AAS_AllocAASLink(bot);
                if link.is_null() {
                    return areas;
                }
                (*link).entnum = entnum;
                (*link).areanum = -nodenum;
                //put the link into the double linked area list of the entity
                (*link).prev_area = core::ptr::null_mut();
                (*link).next_area = areas;
                if !areas.is_null() {
                    (*areas).prev_area = link;
                }
                areas = link;
                //put the link into the double linked entity list of the area
                (*link).prev_ent = core::ptr::null_mut();
                (*link).next_ent = *bot.aasworld.arealinkedentities.add((-nodenum) as usize);
                if !(*bot.aasworld.arealinkedentities.add((-nodenum) as usize)).is_null() {
                    (*(*bot.aasworld.arealinkedentities.add((-nodenum) as usize))).prev_ent = link;
                }
                *bot.aasworld.arealinkedentities.add((-nodenum) as usize) = link;
                //
                continue;
            }
            //if solid leaf
            if nodenum == 0 {
                continue;
            }
            //the node to test against
            let aasnode: *mut aas_node_t = bot.aasworld.nodes.add(nodenum as usize);
            //the current node plane
            let plane: *mut aas_plane_t = bot.aasworld.planes.add((*aasnode).planenum as usize);
            //get the side(s) the box is situated relative to the plane
            let side = AAS_BoxOnPlaneSide2(absmins, absmaxs, plane);
            //if on the front side of the node
            if side & 1 != 0 {
                linkstack[lstack_p].nodenum = (*aasnode).children[0];
                lstack_p += 1;
            }
            if lstack_p >= 127 {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    c"AAS_LinkEntity: stack overflow\n".as_ptr() as *mut _,
                );
                break;
            }
            //if on the back side of the node
            if side & 2 != 0 {
                linkstack[lstack_p].nodenum = (*aasnode).children[1];
                lstack_p += 1;
            }
            if lstack_p >= 127 {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    c"AAS_LinkEntity: stack overflow\n".as_ptr() as *mut _,
                );
                break;
            }
        }
        areas
    }
}

/// Raven `AAS_InitAASLinkedEntities`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:171-177`
pub fn AAS_InitAASLinkedEntities(bot: &mut BotLib) {
    unsafe {
        if bot.aasworld.loaded == 0 {
            return;
        }
        if !bot.aasworld.arealinkedentities.is_null() {
            FreeMemory(bot, bot.aasworld.arealinkedentities as *mut c_void);
        }
        bot.aasworld.arealinkedentities = GetClearedHunkMemory(
            bot,
            bot.aasworld.numareas as usize * core::mem::size_of::<*mut aas_link_t>(),
        ) as *mut *mut aas_link_t;
    }
}

/// Raven `AAS_PointReachabilityAreaIndex`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:249-284`
pub fn AAS_PointReachabilityAreaIndex(bot: &mut BotLib, origin: vec3_t) -> c_int {
    unsafe {
        if bot.aasworld.initialized == 0 {
            return 0;
        }

        // §19: Raven checks `!origin` (a null pointer check on its `vec3_t
        // *` param); `origin` here is passed by value (never null), so
        // that branch is dead under this signature and never taken.

        let areanum = AAS_PointAreaNum(bot, origin);
        if areanum == 0 || AAS_AreaReachability(bot, areanum) == 0 {
            return 0;
        }
        let mut cluster = (*bot.aasworld.areasettings.add(areanum as usize)).cluster;
        let mut areanum = (*bot.aasworld.areasettings.add(areanum as usize)).clusterareanum;
        if cluster < 0 {
            cluster = (*bot.aasworld.portals.add((-cluster) as usize)).frontcluster;
            areanum = (*bot.aasworld.portals.add((-cluster) as usize)).clusterareanum[0];
        }

        let mut index: c_int = 0;
        for i in 0..cluster {
            index += (*bot.aasworld.clusters.add(i as usize)).numreachabilityareas;
        }
        index += areanum;
        index
    }
}

/// Raven `AAS_AreaEntityCollision`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:389-424`
pub fn AAS_AreaEntityCollision(
    bot: &mut BotLib,
    areanum: c_int,
    start: vec3_t,
    end: vec3_t,
    presencetype: c_int,
    passent: c_int,
    trace: *mut aas_trace_t,
) -> qboolean {
    unsafe {
        let mut boxmins: vec3_t = [0.0; 3];
        let mut boxmaxs: vec3_t = [0.0; 3];
        AAS_PresenceTypeBoundingBox(bot, presencetype, boxmins, boxmaxs);

        let mut bsptrace = core::mem::zeroed::<bsp_trace_t>();
        Com_Memset(
            &mut bsptrace as *mut bsp_trace_t as *mut c_void,
            0,
            core::mem::size_of::<bsp_trace_t>(),
        ); //make compiler happy
           //assume no collision
        bsptrace.fraction = 1.0;
        let mut collision = qfalse;
        let mut link = *bot.aasworld.arealinkedentities.add(areanum as usize);
        while !link.is_null() {
            //ignore the pass entity
            if (*link).entnum != passent
                && AAS_EntityCollision(
                    bot,
                    (*link).entnum,
                    start,
                    boxmins,
                    boxmaxs,
                    end,
                    CONTENTS_SOLID | CONTENTS_PLAYERCLIP,
                    &mut bsptrace,
                ) != 0
            {
                collision = qtrue;
            }
            link = (*link).next_ent;
        }
        if collision != 0 {
            (*trace).startsolid = bsptrace.startsolid;
            (*trace).ent = bsptrace.ent;
            (*trace).endpos = bsptrace.endpos;
            (*trace).area = 0;
            (*trace).planenum = 0;
            return qtrue;
        }
        qfalse
    }
}

/// Raven `AAS_LinkEntityClientBBox`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:1306-1316`
pub fn AAS_LinkEntityClientBBox(
    bot: &mut BotLib,
    absmins: vec3_t,
    absmaxs: vec3_t,
    entnum: c_int,
    presencetype: c_int,
) -> *mut aas_link_t {
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    AAS_PresenceTypeBoundingBox(bot, presencetype, mins, maxs);
    let newabsmins: vec3_t = [
        absmins[0] - maxs[0],
        absmins[1] - maxs[1],
        absmins[2] - maxs[2],
    ];
    let newabsmaxs: vec3_t = [
        absmaxs[0] - mins[0],
        absmaxs[1] - mins[1],
        absmaxs[2] - mins[2],
    ];
    //relink the entity
    AAS_AASLinkEntity(bot, newabsmins, newabsmaxs, entnum)
}

/// Raven `AAS_BBoxAreas`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:1323-1339`
pub fn AAS_BBoxAreas(
    bot: &mut BotLib,
    absmins: vec3_t,
    absmaxs: vec3_t,
    areas: *mut c_int,
    maxareas: c_int,
) -> c_int {
    unsafe {
        let linkedareas = AAS_AASLinkEntity(bot, absmins, absmaxs, -1);
        let mut num: c_int = 0;
        let mut link = linkedareas;
        while !link.is_null() {
            *areas.add(num as usize) = (*link).areanum;
            num += 1;
            if num >= maxareas {
                break;
            }
            link = (*link).next_area;
        }
        AAS_UnlinkFromAreas(bot, linkedareas);
        num
    }
}

/// Raven `AAS_TraceClientBBox`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:432-701`
pub fn AAS_TraceClientBBox(
    bot: &mut BotLib,
    start: vec3_t,
    end: vec3_t,
    presencetype: c_int,
    passent: c_int,
) -> aas_trace_t {
    unsafe {
        //clear the trace structure
        let mut trace = core::mem::zeroed::<aas_trace_t>();
        Com_Memset(
            &mut trace as *mut aas_trace_t as *mut c_void,
            0,
            core::mem::size_of::<aas_trace_t>(),
        );

        if bot.aasworld.loaded == 0 {
            return trace;
        }

        // §19: Raven leaves `tracestack` uninitialized before the first
        // write at `tracestack[0]`; each entry is fully written before use.
        let mut tracestack = [aas_tracestack_t {
            start: [0.0; 3],
            end: [0.0; 3],
            planenum: 0,
            nodenum: 0,
        }; 127];
        let mut tstack_p: usize = 0;
        //we start with the whole line on the stack
        tracestack[0].start = start;
        tracestack[0].end = end;
        tracestack[0].planenum = 0;
        //start with node 1 because node zero is a dummy for a solid leaf
        tracestack[0].nodenum = 1; //starting at the root of the tree
        tstack_p += 1;

        loop {
            //pop up the stack
            if tstack_p == 0 {
                //nothing was hit
                trace.startsolid = qfalse;
                trace.fraction = 1.0;
                //endpos is the end of the line
                trace.endpos = end;
                //nothing hit
                trace.ent = 0;
                trace.area = 0;
                trace.planenum = 0;
                return trace;
            }
            tstack_p -= 1;
            //number of the current node to test the line against
            let nodenum = tracestack[tstack_p].nodenum;
            //if it is an area
            if nodenum < 0 {
                //if can't enter the area because it hasn't got the right presence type
                if (*bot.aasworld.areasettings.add((-nodenum) as usize)).presencetype & presencetype
                    == 0
                {
                    let mut v1: vec3_t;
                    //if the start point is still the initial start point
                    //NOTE: no need for epsilons because the points will be
                    //exactly the same when they're both the start point
                    if tracestack[tstack_p].start[0] == start[0]
                        && tracestack[tstack_p].start[1] == start[1]
                        && tracestack[tstack_p].start[2] == start[2]
                    {
                        trace.startsolid = qtrue;
                        trace.fraction = 0.0;
                        v1 = [0.0; 3];
                    } else {
                        trace.startsolid = qfalse;
                        v1 = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
                        let v2: vec3_t = [
                            tracestack[tstack_p].start[0] - start[0],
                            tracestack[tstack_p].start[1] - start[1],
                            tracestack[tstack_p].start[2] - start[2],
                        ];
                        let v2len = VectorLength(v2);
                        let v1len = VectorNormalize(&mut v1);
                        trace.fraction = v2len / v1len;
                        tracestack[tstack_p].start = [
                            tracestack[tstack_p].start[0] + -0.125 * v1[0],
                            tracestack[tstack_p].start[1] + -0.125 * v1[1],
                            tracestack[tstack_p].start[2] + -0.125 * v1[2],
                        ];
                    }
                    trace.endpos = tracestack[tstack_p].start;
                    trace.ent = 0;
                    trace.area = -nodenum;
                    trace.planenum = tracestack[tstack_p].planenum;
                    //always take the plane with normal facing towards the trace start
                    let plane = bot.aasworld.planes.add(trace.planenum as usize);
                    if DotProduct(v1, (*plane).normal) > 0.0 {
                        trace.planenum ^= 1;
                    }
                    return trace;
                } else {
                    if passent >= 0
                        && AAS_AreaEntityCollision(
                            bot,
                            -nodenum,
                            tracestack[tstack_p].start,
                            tracestack[tstack_p].end,
                            presencetype,
                            passent,
                            &mut trace,
                        ) != 0
                    {
                        if trace.startsolid == 0 {
                            let v1: vec3_t =
                                [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
                            let v2: vec3_t = [
                                trace.endpos[0] - start[0],
                                trace.endpos[1] - start[1],
                                trace.endpos[2] - start[2],
                            ];
                            trace.fraction = VectorLength(v2) / VectorLength(v1);
                        }
                        return trace;
                    }
                }
                trace.lastarea = -nodenum;
                continue;
            }
            //if it is a solid leaf
            if nodenum == 0 {
                let mut v1: vec3_t;
                //if the start point is still the initial start point
                //NOTE: no need for epsilons because the points will be
                //exactly the same when they're both the start point
                if tracestack[tstack_p].start[0] == start[0]
                    && tracestack[tstack_p].start[1] == start[1]
                    && tracestack[tstack_p].start[2] == start[2]
                {
                    trace.startsolid = qtrue;
                    trace.fraction = 0.0;
                    v1 = [0.0; 3];
                } else {
                    trace.startsolid = qfalse;
                    v1 = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
                    let v2: vec3_t = [
                        tracestack[tstack_p].start[0] - start[0],
                        tracestack[tstack_p].start[1] - start[1],
                        tracestack[tstack_p].start[2] - start[2],
                    ];
                    let v2len = VectorLength(v2);
                    let v1len = VectorNormalize(&mut v1);
                    trace.fraction = v2len / v1len;
                    tracestack[tstack_p].start = [
                        tracestack[tstack_p].start[0] + -0.125 * v1[0],
                        tracestack[tstack_p].start[1] + -0.125 * v1[1],
                        tracestack[tstack_p].start[2] + -0.125 * v1[2],
                    ];
                }
                trace.endpos = tracestack[tstack_p].start;
                trace.ent = 0;
                trace.area = 0; //hit solid leaf
                trace.planenum = tracestack[tstack_p].planenum;
                //always take the plane with normal facing towards the trace start
                let plane = bot.aasworld.planes.add(trace.planenum as usize);
                if DotProduct(v1, (*plane).normal) > 0.0 {
                    trace.planenum ^= 1;
                }
                return trace;
            }
            //the node to test against
            let aasnode: *mut aas_node_t = bot.aasworld.nodes.add(nodenum as usize);
            //start point of current line to test against node
            let cur_start = tracestack[tstack_p].start;
            //end point of the current line to test against node
            let cur_end = tracestack[tstack_p].end;
            //the current node plane
            let plane: *mut aas_plane_t = bot.aasworld.planes.add((*aasnode).planenum as usize);

            let mut front = DotProduct(cur_start, (*plane).normal) - (*plane).dist;
            let back = DotProduct(cur_end, (*plane).normal) - (*plane).dist;
            // bk010221 - old location of FPE hack and divide by zero expression
            //if the whole to be traced line is totally at the front of this node
            //only go down the tree with the front child
            if front >= -ON_EPSILON && back >= -ON_EPSILON {
                //keep the current start and end point on the stack
                //and go down the tree with the front child
                tracestack[tstack_p].nodenum = (*aasnode).children[0];
                tstack_p += 1;
                if tstack_p >= 127 {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"AAS_TraceBoundingBox: stack overflow\n".as_ptr() as *mut _,
                    );
                    return trace;
                }
            }
            //if the whole to be traced line is totally at the back of this node
            //only go down the tree with the back child
            else if front < ON_EPSILON && back < ON_EPSILON {
                //keep the current start and end point on the stack
                //and go down the tree with the back child
                tracestack[tstack_p].nodenum = (*aasnode).children[1];
                tstack_p += 1;
                if tstack_p >= 127 {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"AAS_TraceBoundingBox: stack overflow\n".as_ptr() as *mut _,
                    );
                    return trace;
                }
            }
            //go down the tree both at the front and back of the node
            else {
                let tmpplanenum = tracestack[tstack_p].planenum;
                // bk010221 - new location of divide by zero (see above)
                if front == back {
                    front -= 0.001; // bk0101022 - hack/FPE
                }
                //calculate the hitpoint with the node (split point of the line)
                //put the crosspoint TRACEPLANE_EPSILON pixels on the near side
                let mut frac = if front < 0.0 {
                    (front + TRACEPLANE_EPSILON) / (front - back)
                } else {
                    (front - TRACEPLANE_EPSILON) / (front - back) // bk010221
                };
                //
                if frac < 0.0 {
                    frac = 0.001; //0
                } else if frac > 1.0 {
                    frac = 0.999; //1
                }
                //
                let cur_mid: vec3_t = [
                    cur_start[0] + (cur_end[0] - cur_start[0]) * frac,
                    cur_start[1] + (cur_end[1] - cur_start[1]) * frac,
                    cur_start[2] + (cur_end[2] - cur_start[2]) * frac,
                ];

                //side the front part of the line is on
                let side = (front < 0.0) as usize;
                //first put the end part of the line on the stack (back side)
                tracestack[tstack_p].start = cur_mid;
                //not necesary to store because still on stack
                tracestack[tstack_p].planenum = (*aasnode).planenum;
                tracestack[tstack_p].nodenum = (*aasnode).children[1 - side];
                tstack_p += 1;
                if tstack_p >= 127 {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"AAS_TraceBoundingBox: stack overflow\n".as_ptr() as *mut _,
                    );
                    return trace;
                }
                //now put the part near the start of the line on the stack so we will
                //continue with thats part first. This way we'll find the first
                //hit of the bbox
                tracestack[tstack_p].start = cur_start;
                tracestack[tstack_p].end = cur_mid;
                tracestack[tstack_p].planenum = tmpplanenum;
                tracestack[tstack_p].nodenum = (*aasnode).children[side];
                tstack_p += 1;
                if tstack_p >= 127 {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"AAS_TraceBoundingBox: stack overflow\n".as_ptr() as *mut _,
                    );
                    return trace;
                }
            }
        }
    }
}

/// Raven `AAS_InitAASLinkHeap`.
///
/// Source: `oracle/codemp/botlib/be_aas_sample.cpp:79-110`
pub fn AAS_InitAASLinkHeap(bot: &mut BotLib) {
    unsafe {
        let mut max_aaslinks = bot.aasworld.linkheapsize;
        //if there's no link heap present
        if bot.aasworld.linkheap.is_null() {
            #[cfg(feature = "bspc")]
            {
                max_aaslinks = 6144;
            }
            #[cfg(not(feature = "bspc"))]
            {
                max_aaslinks =
                    LibVarValue(bot, c"max_aaslinks".as_ptr(), c"6144".as_ptr()) as c_int;
            }
            if max_aaslinks < 0 {
                max_aaslinks = 0;
            }
            bot.aasworld.linkheapsize = max_aaslinks;
            bot.aasworld.linkheap = GetHunkMemory(
                bot,
                max_aaslinks as usize * core::mem::size_of::<aas_link_t>(),
            ) as *mut aas_link_t;
        }
        //link the links on the heap
        (*bot.aasworld.linkheap).prev_ent = core::ptr::null_mut();
        (*bot.aasworld.linkheap).next_ent = bot.aasworld.linkheap.add(1);
        for i in 1..(max_aaslinks - 1) {
            (*bot.aasworld.linkheap.add(i as usize)).prev_ent =
                bot.aasworld.linkheap.add((i - 1) as usize);
            (*bot.aasworld.linkheap.add(i as usize)).next_ent =
                bot.aasworld.linkheap.add((i + 1) as usize);
        }
        (*bot.aasworld.linkheap.add((max_aaslinks - 1) as usize)).prev_ent =
            bot.aasworld.linkheap.add((max_aaslinks - 2) as usize);
        (*bot.aasworld.linkheap.add((max_aaslinks - 1) as usize)).next_ent = core::ptr::null_mut();
        //pointer to the first free link
        bot.aasworld.freelinks = bot.aasworld.linkheap;
        //
        bot.numaaslinks = max_aaslinks;
    }
}
