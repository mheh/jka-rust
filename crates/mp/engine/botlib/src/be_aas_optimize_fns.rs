#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_assignments
)]

//! Function bodies for Raven's `be_aas_optimize.cpp` (AAS file optimization:
//! collapsing dead vertexes/edges/faces from a compiled AAS file).
//!
//! Ported per the engine C-track packets (`botlib__0449`..`botlib__1538`).
//! Source: `oracle/codemp/botlib/be_aas_optimize.cpp`.
//!
//! DESTINATION NOTE: the packet order named
//! `crates/mp/engine/botlib/src/be_aas_optimize.rs`, but `be_aas_optimize`
//! already exists as a directory module (`be_aas_optimize/optimized_s.rs`) —
//! `_fns` escape per `_PREAMBLE.md`'s destination rule.
//!
//! PORT-NOTE(unsafe): the AAS arena and `optimized_t` scratch buffers are raw
//! pointers (`aasworld.*`, `optimized->*`); bodies deref explicitly inside
//! `unsafe` per porting-rules §D11, matching the sibling
//! `be_aas_cluster_fns.rs`/`be_aas_reach_fns.rs` convention.

use core::ffi::c_char;
use core::ffi::c_int;

use crate::aasfile::aas_edge_s::aas_edge_t;
use crate::aasfile::aas_face_s::aas_face_t;
use crate::aasfile::face_flags::FACE_LADDER;
use crate::aasfile::travel_type::{
    TRAVELTYPE_MASK, TRAVEL_ELEVATOR, TRAVEL_FUNCBOB, TRAVEL_JUMPPAD,
};
use crate::be_aas_optimize::optimized_s::optimized_t;
use mp_qshared::common::mp::botlib::print_type::PRT_MESSAGE;

use crate::BotLib;

use crate::l_memory_fns::{FreeMemory, GetClearedMemory};
use mp_engine_qcommon::common_fns::Com_Memcpy;

/// Raven `AAS_KeepEdge` — always keeps an edge (no dead-edge removal).
///
/// Source: `oracle/codemp/botlib/be_aas_optimize.cpp:61-64`
pub fn AAS_KeepEdge(edge: *mut aas_edge_t) -> c_int {
    1
}

/// Raven `AAS_KeepFace` — only ladder faces are kept.
///
/// Source: `oracle/codemp/botlib/be_aas_optimize.cpp:116-120`
pub fn AAS_KeepFace(face: *mut aas_face_t) -> c_int {
    unsafe {
        if (*face).faceflags & FACE_LADDER == 0 {
            0
        } else {
            1
        }
    }
}

/// Raven `AAS_OptimizeEdge` — copies edge `edgenum` into the optimized edge
/// array (memoized via `edgeoptimizeindex`), preserving the reversed-edge
/// sign convention.
///
/// Source: `oracle/codemp/botlib/be_aas_optimize.cpp:71-109`
pub fn AAS_OptimizeEdge(bot: &mut BotLib, optimized: *mut optimized_t, edgenum: c_int) -> c_int {
    unsafe {
        let edge: *mut aas_edge_t = bot.aasworld.edges.add(edgenum.unsigned_abs() as usize);
        if AAS_KeepEdge(edge) == 0 {
            return 0;
        }

        let mut optedgenum = *(*optimized)
            .edgeoptimizeindex
            .add(edgenum.unsigned_abs() as usize);
        if optedgenum != 0 {
            // keep the edge reversed sign
            return if edgenum > 0 { optedgenum } else { -optedgenum };
        }

        let optedge: *mut aas_edge_t = (*optimized).edges.add((*optimized).numedges as usize);

        for i in 0..2usize {
            let v = (*edge).v[i];
            let vidx = *(*optimized).vertexoptimizeindex.add(v as usize);
            if vidx != 0 {
                (*optedge).v[i] = vidx;
            } else {
                let numvertexes = (*optimized).numvertexes;
                let dst = (*optimized).vertexes.add(numvertexes as usize);
                let src = bot.aasworld.vertexes.add(v as usize);
                core::ptr::copy_nonoverlapping(src, dst, 1);
                (*optedge).v[i] = numvertexes;
                *(*optimized).vertexoptimizeindex.add(v as usize) = numvertexes;
                (*optimized).numvertexes += 1;
            }
        }
        *(*optimized)
            .edgeoptimizeindex
            .add(edgenum.unsigned_abs() as usize) = (*optimized).numedges;
        optedgenum = (*optimized).numedges;
        (*optimized).numedges += 1;
        // keep the edge reversed sign
        if edgenum > 0 {
            optedgenum
        } else {
            -optedgenum
        }
    }
}

/// Raven `AAS_OptimizeFace` — copies face `facenum` into the optimized face
/// array (memoized via `faceoptimizeindex`), optimizing its edges along the
/// way and preserving the reversed-face sign convention.
///
/// Source: `oracle/codemp/botlib/be_aas_optimize.cpp:127-165`
pub fn AAS_OptimizeFace(bot: &mut BotLib, optimized: *mut optimized_t, facenum: c_int) -> c_int {
    unsafe {
        let face: *mut aas_face_t = bot.aasworld.faces.add(facenum.unsigned_abs() as usize);
        if AAS_KeepFace(face) == 0 {
            return 0;
        }

        let mut optfacenum = *(*optimized)
            .faceoptimizeindex
            .add(facenum.unsigned_abs() as usize);
        if optfacenum != 0 {
            // keep the face side sign
            return if facenum > 0 { optfacenum } else { -optfacenum };
        }

        let optface: *mut aas_face_t = (*optimized).faces.add((*optimized).numfaces as usize);
        Com_Memcpy(
            optface as *mut (),
            face as *const (),
            core::mem::size_of::<aas_face_t>(),
        );

        (*optface).numedges = 0;
        (*optface).firstedge = (*optimized).edgeindexsize;
        for i in 0..(*face).numedges {
            let edgenum = *bot.aasworld.edgeindex.add(((*face).firstedge + i) as usize);
            let optedgenum = AAS_OptimizeEdge(bot, optimized, edgenum);
            if optedgenum != 0 {
                *(*optimized)
                    .edgeindex
                    .add(((*optface).firstedge + (*optface).numedges) as usize) = optedgenum;
                (*optface).numedges += 1;
                (*optimized).edgeindexsize += 1;
            }
        }
        *(*optimized)
            .faceoptimizeindex
            .add(facenum.unsigned_abs() as usize) = (*optimized).numfaces;
        optfacenum = (*optimized).numfaces;
        (*optimized).numfaces += 1;
        // keep the face side sign
        if facenum > 0 {
            optfacenum
        } else {
            -optfacenum
        }
    }
}

/// Raven `AAS_OptimizeAlloc` — allocates the scratch buffers for
/// optimization, sized against the (pre-optimization) `aasworld` counts.
///
/// Source: `oracle/codemp/botlib/be_aas_optimize.cpp:201-219`
pub fn AAS_OptimizeAlloc(bot: &mut BotLib, optimized: *mut optimized_t) {
    unsafe {
        (*optimized).vertexes = GetClearedMemory(
            bot,
            (bot.aasworld.numvertexes as usize
                * core::mem::size_of::<crate::aasfile::aas_vertex_t::aas_vertex_t>())
                as core::ffi::c_ulong,
        ) as *mut crate::aasfile::aas_vertex_t::aas_vertex_t;
        (*optimized).numvertexes = 0;
        (*optimized).edges = GetClearedMemory(
            bot,
            (bot.aasworld.numedges as usize * core::mem::size_of::<aas_edge_t>())
                as core::ffi::c_ulong,
        ) as *mut aas_edge_t;
        (*optimized).numedges = 1; // edge zero is a dummy
        (*optimized).edgeindex = GetClearedMemory(
            bot,
            (bot.aasworld.edgeindexsize as usize
                * core::mem::size_of::<crate::aasfile::aas_edgeindex_t::aas_edgeindex_t>())
                as core::ffi::c_ulong,
        ) as *mut crate::aasfile::aas_edgeindex_t::aas_edgeindex_t;
        (*optimized).edgeindexsize = 0;
        (*optimized).faces = GetClearedMemory(
            bot,
            (bot.aasworld.numfaces as usize * core::mem::size_of::<aas_face_t>())
                as core::ffi::c_ulong,
        ) as *mut aas_face_t;
        (*optimized).numfaces = 1; // face zero is a dummy
        (*optimized).faceindex = GetClearedMemory(
            bot,
            (bot.aasworld.faceindexsize as usize
                * core::mem::size_of::<crate::aasfile::aas_faceindex_t::aas_faceindex_t>())
                as core::ffi::c_ulong,
        ) as *mut crate::aasfile::aas_faceindex_t::aas_faceindex_t;
        (*optimized).faceindexsize = 0;
        (*optimized).areas = GetClearedMemory(
            bot,
            (bot.aasworld.numareas as usize
                * core::mem::size_of::<crate::aasfile::aas_area_s::aas_area_t>())
                as core::ffi::c_ulong,
        ) as *mut crate::aasfile::aas_area_s::aas_area_t;
        (*optimized).numareas = bot.aasworld.numareas;
        //
        (*optimized).vertexoptimizeindex = GetClearedMemory(
            bot,
            (bot.aasworld.numvertexes as usize * core::mem::size_of::<c_int>())
                as core::ffi::c_ulong,
        ) as *mut c_int;
        (*optimized).edgeoptimizeindex = GetClearedMemory(
            bot,
            (bot.aasworld.numedges as usize * core::mem::size_of::<c_int>()) as core::ffi::c_ulong,
        ) as *mut c_int;
        (*optimized).faceoptimizeindex = GetClearedMemory(
            bot,
            (bot.aasworld.numfaces as usize * core::mem::size_of::<c_int>()) as core::ffi::c_ulong,
        ) as *mut c_int;
    }
}

/// Raven `AAS_OptimizeArea` — copies area `areanum` into the optimized area
/// array (areas keep their original index; unlike edges/faces there is no
/// memoization index), optimizing its faces along the way.
///
/// Source: `oracle/codemp/botlib/be_aas_optimize.cpp:172-194`
pub fn AAS_OptimizeArea(bot: &mut BotLib, optimized: *mut optimized_t, areanum: c_int) {
    unsafe {
        let area: *mut crate::aasfile::aas_area_s::aas_area_t =
            bot.aasworld.areas.add(areanum as usize);
        let optarea: *mut crate::aasfile::aas_area_s::aas_area_t =
            (*optimized).areas.add(areanum as usize);
        Com_Memcpy(
            optarea as *mut (),
            area as *const (),
            core::mem::size_of::<crate::aasfile::aas_area_s::aas_area_t>(),
        );

        (*optarea).numfaces = 0;
        (*optarea).firstface = (*optimized).faceindexsize;
        for i in 0..(*area).numfaces {
            let facenum = *bot.aasworld.faceindex.add(((*area).firstface + i) as usize);
            let optfacenum = AAS_OptimizeFace(bot, optimized, facenum);
            if optfacenum != 0 {
                *(*optimized)
                    .faceindex
                    .add(((*optarea).firstface + (*optarea).numfaces) as usize) = optfacenum;
                (*optarea).numfaces += 1;
                (*optimized).faceindexsize += 1;
            }
        }
    }
}

/// Raven `AAS_OptimizeStore` — stores the optimized buffers back into
/// `aasworld`, freeing the previous buffers and the now-unused optimize
/// indexes.
///
/// Source: `oracle/codemp/botlib/be_aas_optimize.cpp:226-256`
pub fn AAS_OptimizeStore(bot: &mut BotLib, optimized: *mut optimized_t) {
    unsafe {
        // store the optimized vertexes
        if !bot.aasworld.vertexes.is_null() {
            FreeMemory(bot, bot.aasworld.vertexes as *mut ());
        }
        bot.aasworld.vertexes = (*optimized).vertexes;
        bot.aasworld.numvertexes = (*optimized).numvertexes;
        // store the optimized edges
        if !bot.aasworld.edges.is_null() {
            FreeMemory(bot, bot.aasworld.edges as *mut ());
        }
        bot.aasworld.edges = (*optimized).edges;
        bot.aasworld.numedges = (*optimized).numedges;
        // store the optimized edge index
        if !bot.aasworld.edgeindex.is_null() {
            FreeMemory(bot, bot.aasworld.edgeindex as *mut ());
        }
        bot.aasworld.edgeindex = (*optimized).edgeindex;
        bot.aasworld.edgeindexsize = (*optimized).edgeindexsize;
        // store the optimized faces
        if !bot.aasworld.faces.is_null() {
            FreeMemory(bot, bot.aasworld.faces as *mut ());
        }
        bot.aasworld.faces = (*optimized).faces;
        bot.aasworld.numfaces = (*optimized).numfaces;
        // store the optimized face index
        if !bot.aasworld.faceindex.is_null() {
            FreeMemory(bot, bot.aasworld.faceindex as *mut ());
        }
        bot.aasworld.faceindex = (*optimized).faceindex;
        bot.aasworld.faceindexsize = (*optimized).faceindexsize;
        // store the optimized areas
        if !bot.aasworld.areas.is_null() {
            FreeMemory(bot, bot.aasworld.areas as *mut ());
        }
        bot.aasworld.areas = (*optimized).areas;
        bot.aasworld.numareas = (*optimized).numareas;
        // free optimize indexes
        FreeMemory(bot, (*optimized).vertexoptimizeindex as *mut ());
        FreeMemory(bot, (*optimized).edgeoptimizeindex as *mut ());
        FreeMemory(bot, (*optimized).faceoptimizeindex as *mut ());
    }
}

/// Raven `AAS_Optimize` — top-level AAS file optimization: allocates
/// scratch buffers, optimizes every area (and transitively its faces and
/// edges), fixes up reachability face/edge references to the new indexes,
/// then stores the optimized data back into `aasworld`.
///
/// Source: `oracle/codemp/botlib/be_aas_optimize.cpp:263-295`
pub fn AAS_Optimize(bot: &mut BotLib) {
    unsafe {
        // §19: `optimized` is a C local struct read before being fully
        // written by `AAS_OptimizeAlloc` (which sets every field) — the
        // Rust local is zero-initialized to avoid reading uninitialized
        // memory before that call.
        let mut optimized: optimized_t = core::mem::zeroed();

        AAS_OptimizeAlloc(bot, &mut optimized);
        for i in 1..bot.aasworld.numareas {
            AAS_OptimizeArea(bot, &mut optimized, i);
        }
        // reset the reachability face pointers
        for i in 0..bot.aasworld.reachabilitysize {
            let reach = bot.aasworld.reachability.add(i as usize);
            // NOTE: for TRAVEL_ELEVATOR the facenum is the model number of
            //		the elevator
            if (*reach).traveltype & TRAVELTYPE_MASK == TRAVEL_ELEVATOR {
                continue;
            }
            // NOTE: for TRAVEL_JUMPPAD the facenum is the Z velocity and the edgenum is the hor velocity
            if (*reach).traveltype & TRAVELTYPE_MASK == TRAVEL_JUMPPAD {
                continue;
            }
            // NOTE: for TRAVEL_FUNCBOB the facenum and edgenum contain other coded information
            if (*reach).traveltype & TRAVELTYPE_MASK == TRAVEL_FUNCBOB {
                continue;
            }
            //
            let mut sign = (*reach).facenum;
            (*reach).facenum = *optimized
                .faceoptimizeindex
                .add((*reach).facenum.unsigned_abs() as usize);
            if sign < 0 {
                (*reach).facenum = -(*reach).facenum;
            }
            sign = (*reach).edgenum;
            (*reach).edgenum = *optimized
                .edgeoptimizeindex
                .add((*reach).edgenum.unsigned_abs() as usize);
            if sign < 0 {
                (*reach).edgenum = -(*reach).edgenum;
            }
        }
        // store the optimized AAS data into aasworld
        AAS_OptimizeStore(bot, &mut optimized);
        // print some nice stuff :)
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"AAS data optimized.\n".as_ptr() as *mut c_char,
        );
    }
}
