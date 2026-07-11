#![allow(non_camel_case_types, non_snake_case, clippy::missing_safety_doc)]

//! MP botlib `be_aas_file.cpp` — AAS (Area Awareness System) file I/O: the
//! endian swap of a loaded `aasworld`, the lump obfuscation cipher, and the
//! per-lump writer.
//!
//! Source: `oracle/codemp/botlib/be_aas_file.cpp`
//!
//! Destination `_fns` escape: the module name stays distinct from the
//! `aasfile/` type directory.

use core::ffi::{c_int, c_uchar, c_void};

use mp_qshared::shared::{fileHandle_t, qtrue};

use crate::aasfile::aas_header_s::aas_header_t;
use crate::BotLib;

/// Raven's `LittleLong`/`LittleShort`/`LittleFloat` are empty `#define`s on the
/// little-endian target the referee runs (`#define LittleLong`,
/// q_shared.h:172-176), so each expands to the identity of its argument; the
/// byte-swapping definitions compile only on big-endian builds. Modeled as
/// type-preserving identity helpers so each call site keeps Raven's macro name
/// (the `vec3_t` fields Raven feeds to `LittleLong` are a harmless quirk under
/// the empty macro).
/// Source: `oracle/codemp/game/q_shared.h:171-176`
#[inline]
fn LittleLong<T>(l: T) -> T {
    l
}
#[inline]
fn LittleShort<T>(l: T) -> T {
    l
}
#[inline]
fn LittleFloat<T>(l: T) -> T {
    l
}

/// Raven `AAS_SwapAASData`.
///
/// Source: `oracle/codemp/botlib/be_aas_file.cpp:37-158`
pub fn AAS_SwapAASData(bot: &mut BotLib) {
    unsafe {
        //bounding boxes
        for i in 0..bot.aasworld.numbboxes {
            let bb = bot.aasworld.bboxes.add(i as usize);
            (*bb).presencetype = LittleLong((*bb).presencetype);
            (*bb).flags = LittleLong((*bb).flags);
            for j in 0..3usize {
                (*bb).mins[j] = LittleLong((*bb).mins[j]);
                (*bb).maxs[j] = LittleLong((*bb).maxs[j]);
            } //end for
        } //end for
        //vertexes
        for i in 0..bot.aasworld.numvertexes {
            let v = bot.aasworld.vertexes.add(i as usize);
            for j in 0..3usize {
                (*v)[j] = LittleFloat((*v)[j]);
            }
        } //end for
        //planes
        for i in 0..bot.aasworld.numplanes {
            let p = bot.aasworld.planes.add(i as usize);
            for j in 0..3usize {
                (*p).normal[j] = LittleFloat((*p).normal[j]);
            }
            (*p).dist = LittleFloat((*p).dist);
            (*p).r#type = LittleLong((*p).r#type);
        } //end for
        //edges
        for i in 0..bot.aasworld.numedges {
            let e = bot.aasworld.edges.add(i as usize);
            (*e).v[0] = LittleLong((*e).v[0]);
            (*e).v[1] = LittleLong((*e).v[1]);
        } //end for
        //edgeindex
        for i in 0..bot.aasworld.edgeindexsize {
            let ei = bot.aasworld.edgeindex.add(i as usize);
            *ei = LittleLong(*ei);
        } //end for
        //faces
        for i in 0..bot.aasworld.numfaces {
            let f = bot.aasworld.faces.add(i as usize);
            (*f).planenum = LittleLong((*f).planenum);
            (*f).faceflags = LittleLong((*f).faceflags);
            (*f).numedges = LittleLong((*f).numedges);
            (*f).firstedge = LittleLong((*f).firstedge);
            (*f).frontarea = LittleLong((*f).frontarea);
            (*f).backarea = LittleLong((*f).backarea);
        } //end for
        //face index
        for i in 0..bot.aasworld.faceindexsize {
            let fi = bot.aasworld.faceindex.add(i as usize);
            *fi = LittleLong(*fi);
        } //end for
        //convex areas
        for i in 0..bot.aasworld.numareas {
            let a = bot.aasworld.areas.add(i as usize);
            (*a).areanum = LittleLong((*a).areanum);
            (*a).numfaces = LittleLong((*a).numfaces);
            (*a).firstface = LittleLong((*a).firstface);
            for j in 0..3usize {
                (*a).mins[j] = LittleFloat((*a).mins[j]);
                (*a).maxs[j] = LittleFloat((*a).maxs[j]);
                (*a).center[j] = LittleFloat((*a).center[j]);
            } //end for
        } //end for
        //area settings
        for i in 0..bot.aasworld.numareasettings {
            let s = bot.aasworld.areasettings.add(i as usize);
            (*s).contents = LittleLong((*s).contents);
            (*s).areaflags = LittleLong((*s).areaflags);
            (*s).presencetype = LittleLong((*s).presencetype);
            (*s).cluster = LittleLong((*s).cluster);
            (*s).clusterareanum = LittleLong((*s).clusterareanum);
            (*s).numreachableareas = LittleLong((*s).numreachableareas);
            (*s).firstreachablearea = LittleLong((*s).firstreachablearea);
        } //end for
        //area reachability
        for i in 0..bot.aasworld.reachabilitysize {
            let r = bot.aasworld.reachability.add(i as usize);
            (*r).areanum = LittleLong((*r).areanum);
            (*r).facenum = LittleLong((*r).facenum);
            (*r).edgenum = LittleLong((*r).edgenum);
            for j in 0..3usize {
                (*r).start[j] = LittleFloat((*r).start[j]);
                (*r).end[j] = LittleFloat((*r).end[j]);
            } //end for
            (*r).traveltype = LittleLong((*r).traveltype);
            (*r).traveltime = LittleShort((*r).traveltime);
        } //end for
        //nodes
        for i in 0..bot.aasworld.numnodes {
            let n = bot.aasworld.nodes.add(i as usize);
            (*n).planenum = LittleLong((*n).planenum);
            (*n).children[0] = LittleLong((*n).children[0]);
            (*n).children[1] = LittleLong((*n).children[1]);
        } //end for
        //cluster portals
        for i in 0..bot.aasworld.numportals {
            let po = bot.aasworld.portals.add(i as usize);
            (*po).areanum = LittleLong((*po).areanum);
            (*po).frontcluster = LittleLong((*po).frontcluster);
            (*po).backcluster = LittleLong((*po).backcluster);
            (*po).clusterareanum[0] = LittleLong((*po).clusterareanum[0]);
            (*po).clusterareanum[1] = LittleLong((*po).clusterareanum[1]);
        } //end for
        //cluster portal index
        for i in 0..bot.aasworld.portalindexsize {
            let pi = bot.aasworld.portalindex.add(i as usize);
            *pi = LittleLong(*pi);
        } //end for
        //cluster
        for i in 0..bot.aasworld.numclusters {
            let c = bot.aasworld.clusters.add(i as usize);
            (*c).numareas = LittleLong((*c).numareas);
            (*c).numreachabilityareas = LittleLong((*c).numreachabilityareas);
            (*c).numportals = LittleLong((*c).numportals);
            (*c).firstportal = LittleLong((*c).firstportal);
        } //end for
    }
} //end of the function AAS_SwapAASData

/// Raven `AAS_DData`.
///
/// Source: `oracle/codemp/botlib/be_aas_file.cpp:310-318`
pub fn AAS_DData(data: *mut c_uchar, size: c_int) {
    for i in 0..size {
        unsafe {
            *data.add(i as usize) ^= (i as c_uchar).wrapping_mul(119);
        }
    } //end for
} //end of the function AAS_DData

/// Raven `AAS_WriteAASLump`.
///
/// Source: `oracle/codemp/botlib/be_aas_file.cpp:481-498`
pub fn AAS_WriteAASLump(
    bot: &mut BotLib,
    fp: fileHandle_t,
    h: *mut aas_header_t,
    lumpnum: c_int,
    data: *mut (),
    length: c_int,
) -> c_int {
    unsafe {
        let lump = &mut (*h).lumps[lumpnum as usize];

        lump.fileofs = LittleLong(bot.AAS_WriteAASLump_offset); //LittleLong(ftell(fp));
        lump.filelen = LittleLong(length);

        if length > 0 {
            (bot.botimport.FS_Write.unwrap())(data as *const c_void, length, fp);
        } //end if

        bot.AAS_WriteAASLump_offset += length;

        qtrue
    }
} //end of the function AAS_WriteAASLump
