#![allow(non_camel_case_types, non_snake_case, clippy::missing_safety_doc)]

//! MP botlib `be_aas_file.cpp` — AAS (Area Awareness System) file I/O: the
//! endian swap of a loaded `aasworld`, the lump obfuscation cipher, and the
//! per-lump writer.
//!
//! Source: `oracle/codemp/botlib/be_aas_file.cpp`
//!
//! Destination `_fns` escape: the module name stays distinct from the
//! `aasfile/` type directory.

use core::ffi::{c_char, c_int, c_long, c_uchar, c_void};

use mp_engine_qcommon::common_fns::Com_Memset;
use mp_qshared::common::mp::botlib::botlib_error::{
    BLERR_CANNOTOPENAASFILE, BLERR_CANNOTREADAASLUMP, BLERR_NOERROR, BLERR_WRONGAASFILEID,
    BLERR_WRONGAASFILEVERSION,
};
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_MESSAGE, PRT_WARNING};
use mp_qshared::shared::fs_origin::fsOrigin_t;
use mp_qshared::shared::{fileHandle_t, qboolean, qfalse, qtrue, FS_READ, FS_WRITE};

use crate::aasfile::aas_area_s::aas_area_t;
use crate::aasfile::aas_areasettings_s::aas_areasettings_t;
use crate::aasfile::aas_bbox_s::aas_bbox_t;
use crate::aasfile::aas_cluster_s::aas_cluster_t;
use crate::aasfile::aas_edge_s::aas_edge_t;
use crate::aasfile::aas_edgeindex_t::aas_edgeindex_t;
use crate::aasfile::aas_face_s::aas_face_t;
use crate::aasfile::aas_faceindex_t::aas_faceindex_t;
use crate::aasfile::aas_header_s::aas_header_t;
use crate::aasfile::aas_node_s::aas_node_t;
use crate::aasfile::aas_plane_s::aas_plane_t;
use crate::aasfile::aas_portal_s::aas_portal_t;
use crate::aasfile::aas_portalindex_t::aas_portalindex_t;
use crate::aasfile::aas_reachability_s::aas_reachability_t;
use crate::aasfile::aas_vertex_t::aas_vertex_t;
use crate::aasfile::header_consts::{AASID, AASVERSION, AASVERSION_OLD};
use crate::aasfile::lump_index::{
    AASLUMP_AREAS, AASLUMP_AREASETTINGS, AASLUMP_BBOXES, AASLUMP_CLUSTERS, AASLUMP_EDGEINDEX,
    AASLUMP_EDGES, AASLUMP_FACEINDEX, AASLUMP_FACES, AASLUMP_NODES, AASLUMP_PLANES,
    AASLUMP_PORTALINDEX, AASLUMP_PORTALS, AASLUMP_REACHABILITY, AASLUMP_VERTEXES,
};
use crate::be_aas_main::AAS_Error;
use crate::l_libvar_fns::LibVarGetString;
use crate::l_memory_fns::{FreeMemory, GetClearedHunkMemory};
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

/// Raven `AAS_DumpAASData` — dump the current loaded aas file.
///
/// Source: `oracle/codemp/botlib/be_aas_file.cpp:166-215`
pub fn AAS_DumpAASData(bot: &mut BotLib) {
    unsafe {
        bot.aasworld.numbboxes = 0;
        if !bot.aasworld.bboxes.is_null() {
            FreeMemory(bot, bot.aasworld.bboxes as *mut ());
        }
        bot.aasworld.bboxes = core::ptr::null_mut();
        bot.aasworld.numvertexes = 0;
        if !bot.aasworld.vertexes.is_null() {
            FreeMemory(bot, bot.aasworld.vertexes as *mut ());
        }
        bot.aasworld.vertexes = core::ptr::null_mut();
        bot.aasworld.numplanes = 0;
        if !bot.aasworld.planes.is_null() {
            FreeMemory(bot, bot.aasworld.planes as *mut ());
        }
        bot.aasworld.planes = core::ptr::null_mut();
        bot.aasworld.numedges = 0;
        if !bot.aasworld.edges.is_null() {
            FreeMemory(bot, bot.aasworld.edges as *mut ());
        }
        bot.aasworld.edges = core::ptr::null_mut();
        bot.aasworld.edgeindexsize = 0;
        if !bot.aasworld.edgeindex.is_null() {
            FreeMemory(bot, bot.aasworld.edgeindex as *mut ());
        }
        bot.aasworld.edgeindex = core::ptr::null_mut();
        bot.aasworld.numfaces = 0;
        if !bot.aasworld.faces.is_null() {
            FreeMemory(bot, bot.aasworld.faces as *mut ());
        }
        bot.aasworld.faces = core::ptr::null_mut();
        bot.aasworld.faceindexsize = 0;
        if !bot.aasworld.faceindex.is_null() {
            FreeMemory(bot, bot.aasworld.faceindex as *mut ());
        }
        bot.aasworld.faceindex = core::ptr::null_mut();
        bot.aasworld.numareas = 0;
        if !bot.aasworld.areas.is_null() {
            FreeMemory(bot, bot.aasworld.areas as *mut ());
        }
        bot.aasworld.areas = core::ptr::null_mut();
        bot.aasworld.numareasettings = 0;
        if !bot.aasworld.areasettings.is_null() {
            FreeMemory(bot, bot.aasworld.areasettings as *mut ());
        }
        bot.aasworld.areasettings = core::ptr::null_mut();
        bot.aasworld.reachabilitysize = 0;
        if !bot.aasworld.reachability.is_null() {
            FreeMemory(bot, bot.aasworld.reachability as *mut ());
        }
        bot.aasworld.reachability = core::ptr::null_mut();
        bot.aasworld.numnodes = 0;
        if !bot.aasworld.nodes.is_null() {
            FreeMemory(bot, bot.aasworld.nodes as *mut ());
        }
        bot.aasworld.nodes = core::ptr::null_mut();
        bot.aasworld.numportals = 0;
        if !bot.aasworld.portals.is_null() {
            FreeMemory(bot, bot.aasworld.portals as *mut ());
        }
        bot.aasworld.portals = core::ptr::null_mut();
        bot.aasworld.numportals = 0;
        if !bot.aasworld.portalindex.is_null() {
            FreeMemory(bot, bot.aasworld.portalindex as *mut ());
        }
        bot.aasworld.portalindex = core::ptr::null_mut();
        bot.aasworld.portalindexsize = 0;
        if !bot.aasworld.clusters.is_null() {
            FreeMemory(bot, bot.aasworld.clusters as *mut ());
        }
        bot.aasworld.clusters = core::ptr::null_mut();
        bot.aasworld.numclusters = 0;
        //
        bot.aasworld.loaded = qfalse;
        bot.aasworld.initialized = qfalse;
        bot.aasworld.savefile = qfalse;
    }
} //end of the function AAS_DumpAASData

/// Raven `AAS_LoadAASLump` — allocate memory and read a lump of an AAS file.
///
/// Source: `oracle/codemp/botlib/be_aas_file.cpp:273-303`
pub fn AAS_LoadAASLump(
    bot: &mut BotLib,
    fp: fileHandle_t,
    offset: c_int,
    length: c_int,
    lastoffset: *mut c_int,
    size: c_int,
) -> *mut c_char {
    unsafe {
        //
        if length == 0 {
            //just alloc a dummy
            return GetClearedHunkMemory(bot, (size + 1) as core::ffi::c_ulong) as *mut c_char;
        }
        //seek to the data
        if offset != *lastoffset {
            bot.botimport.Print.unwrap()(
                PRT_WARNING,
                c"AAS file not sequentially read\n".as_ptr() as *mut c_char,
            );
            if (bot.botimport.FS_Seek.unwrap())(fp, offset as c_long, fsOrigin_t::FS_SEEK_SET as c_int)
                != 0
            {
                AAS_Error(bot, c"can't seek to aas lump\n".as_ptr() as *mut c_char);
                AAS_DumpAASData(bot);
                (bot.botimport.FS_FCloseFile.unwrap())(fp);
                return core::ptr::null_mut();
            }
        }
        //allocate memory
        let buf = GetClearedHunkMemory(bot, (length + 1) as core::ffi::c_ulong) as *mut c_char;
        //read the data
        if length != 0 {
            (bot.botimport.FS_Read.unwrap())(buf as *mut c_void, length, fp);
            *lastoffset += length;
        }
        buf
    }
} //end of the function AAS_LoadAASLump

/// Raven `AAS_LoadAASFile` — load an aas file.
///
/// Source: `oracle/codemp/botlib/be_aas_file.cpp:326-472`
pub fn AAS_LoadAASFile(bot: &mut BotLib, filename: *mut c_char) -> c_int {
    unsafe {
        let mut fp: fileHandle_t = 0;
        let mut header: aas_header_t = core::mem::zeroed();
        let mut offset: c_int;
        let mut length: c_int;
        let mut lastoffset: c_int;

        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"trying to load %s\n".as_ptr() as *mut c_char,
            filename,
        );
        //dump current loaded aas file
        AAS_DumpAASData(bot);
        //open the file
        (bot.botimport.FS_FOpenFile.unwrap())(filename, &mut fp, FS_READ);
        if fp == 0 {
            let msg = std::ffi::CString::new(format!(
                "can't open {}\n",
                core::ffi::CStr::from_ptr(filename).to_string_lossy()
            ))
            .unwrap_or_default();
            AAS_Error(bot, msg.as_ptr() as *mut c_char);
            return BLERR_CANNOTOPENAASFILE;
        }
        //read the header
        (bot.botimport.FS_Read.unwrap())(
            &mut header as *mut _ as *mut c_void,
            core::mem::size_of::<aas_header_t>() as c_int,
            fp,
        );
        lastoffset = core::mem::size_of::<aas_header_t>() as c_int;
        //check header identification
        header.ident = LittleLong(header.ident);
        if header.ident != AASID {
            let msg = std::ffi::CString::new(format!(
                "{} is not an AAS file\n",
                core::ffi::CStr::from_ptr(filename).to_string_lossy()
            ))
            .unwrap_or_default();
            AAS_Error(bot, msg.as_ptr() as *mut c_char);
            (bot.botimport.FS_FCloseFile.unwrap())(fp);
            return BLERR_WRONGAASFILEID;
        }
        //check the version
        header.version = LittleLong(header.version);
        //
        if header.version != AASVERSION_OLD && header.version != AASVERSION {
            let msg = std::ffi::CString::new(format!(
                "aas file {} is version {}, not {}\n",
                core::ffi::CStr::from_ptr(filename).to_string_lossy(),
                header.version,
                AASVERSION
            ))
            .unwrap_or_default();
            AAS_Error(bot, msg.as_ptr() as *mut c_char);
            (bot.botimport.FS_FCloseFile.unwrap())(fp);
            return BLERR_WRONGAASFILEVERSION;
        }
        //
        if header.version == AASVERSION {
            AAS_DData(
                (&mut header as *mut _ as *mut c_uchar).add(8),
                core::mem::size_of::<aas_header_t>() as c_int - 8,
            );
        }
        //
        bot.aasworld.bspchecksum =
            libc::atoi(LibVarGetString(bot, c"sv_mapChecksum".as_ptr() as *mut c_char));
        if LittleLong(header.bspchecksum) != bot.aasworld.bspchecksum {
            let msg = std::ffi::CString::new(format!(
                "aas file {} is out of date\n",
                core::ffi::CStr::from_ptr(filename).to_string_lossy()
            ))
            .unwrap_or_default();
            AAS_Error(bot, msg.as_ptr() as *mut c_char);
            (bot.botimport.FS_FCloseFile.unwrap())(fp);
            return BLERR_WRONGAASFILEVERSION;
        }
        //load the lumps:
        //bounding boxes
        offset = LittleLong(header.lumps[AASLUMP_BBOXES as usize].fileofs);
        length = LittleLong(header.lumps[AASLUMP_BBOXES as usize].filelen);
        bot.aasworld.bboxes = AAS_LoadAASLump(
            bot,
            fp,
            offset,
            length,
            &mut lastoffset,
            core::mem::size_of::<aas_bbox_t>() as c_int,
        ) as *mut aas_bbox_t;
        bot.aasworld.numbboxes = length / core::mem::size_of::<aas_bbox_t>() as c_int;
        if bot.aasworld.numbboxes != 0 && bot.aasworld.bboxes.is_null() {
            return BLERR_CANNOTREADAASLUMP;
        }
        //vertexes
        offset = LittleLong(header.lumps[AASLUMP_VERTEXES as usize].fileofs);
        length = LittleLong(header.lumps[AASLUMP_VERTEXES as usize].filelen);
        bot.aasworld.vertexes = AAS_LoadAASLump(
            bot,
            fp,
            offset,
            length,
            &mut lastoffset,
            core::mem::size_of::<aas_vertex_t>() as c_int,
        ) as *mut aas_vertex_t;
        bot.aasworld.numvertexes = length / core::mem::size_of::<aas_vertex_t>() as c_int;
        if bot.aasworld.numvertexes != 0 && bot.aasworld.vertexes.is_null() {
            return BLERR_CANNOTREADAASLUMP;
        }
        //planes
        offset = LittleLong(header.lumps[AASLUMP_PLANES as usize].fileofs);
        length = LittleLong(header.lumps[AASLUMP_PLANES as usize].filelen);
        bot.aasworld.planes = AAS_LoadAASLump(
            bot,
            fp,
            offset,
            length,
            &mut lastoffset,
            core::mem::size_of::<aas_plane_t>() as c_int,
        ) as *mut aas_plane_t;
        bot.aasworld.numplanes = length / core::mem::size_of::<aas_plane_t>() as c_int;
        if bot.aasworld.numplanes != 0 && bot.aasworld.planes.is_null() {
            return BLERR_CANNOTREADAASLUMP;
        }
        //edges
        offset = LittleLong(header.lumps[AASLUMP_EDGES as usize].fileofs);
        length = LittleLong(header.lumps[AASLUMP_EDGES as usize].filelen);
        bot.aasworld.edges = AAS_LoadAASLump(
            bot,
            fp,
            offset,
            length,
            &mut lastoffset,
            core::mem::size_of::<aas_edge_t>() as c_int,
        ) as *mut aas_edge_t;
        bot.aasworld.numedges = length / core::mem::size_of::<aas_edge_t>() as c_int;
        if bot.aasworld.numedges != 0 && bot.aasworld.edges.is_null() {
            return BLERR_CANNOTREADAASLUMP;
        }
        //edgeindex
        offset = LittleLong(header.lumps[AASLUMP_EDGEINDEX as usize].fileofs);
        length = LittleLong(header.lumps[AASLUMP_EDGEINDEX as usize].filelen);
        bot.aasworld.edgeindex = AAS_LoadAASLump(
            bot,
            fp,
            offset,
            length,
            &mut lastoffset,
            core::mem::size_of::<aas_edgeindex_t>() as c_int,
        ) as *mut aas_edgeindex_t;
        bot.aasworld.edgeindexsize = length / core::mem::size_of::<aas_edgeindex_t>() as c_int;
        if bot.aasworld.edgeindexsize != 0 && bot.aasworld.edgeindex.is_null() {
            return BLERR_CANNOTREADAASLUMP;
        }
        //faces
        offset = LittleLong(header.lumps[AASLUMP_FACES as usize].fileofs);
        length = LittleLong(header.lumps[AASLUMP_FACES as usize].filelen);
        bot.aasworld.faces = AAS_LoadAASLump(
            bot,
            fp,
            offset,
            length,
            &mut lastoffset,
            core::mem::size_of::<aas_face_t>() as c_int,
        ) as *mut aas_face_t;
        bot.aasworld.numfaces = length / core::mem::size_of::<aas_face_t>() as c_int;
        if bot.aasworld.numfaces != 0 && bot.aasworld.faces.is_null() {
            return BLERR_CANNOTREADAASLUMP;
        }
        //faceindex
        offset = LittleLong(header.lumps[AASLUMP_FACEINDEX as usize].fileofs);
        length = LittleLong(header.lumps[AASLUMP_FACEINDEX as usize].filelen);
        bot.aasworld.faceindex = AAS_LoadAASLump(
            bot,
            fp,
            offset,
            length,
            &mut lastoffset,
            core::mem::size_of::<aas_faceindex_t>() as c_int,
        ) as *mut aas_faceindex_t;
        bot.aasworld.faceindexsize = length / core::mem::size_of::<aas_faceindex_t>() as c_int;
        if bot.aasworld.faceindexsize != 0 && bot.aasworld.faceindex.is_null() {
            return BLERR_CANNOTREADAASLUMP;
        }
        //convex areas
        offset = LittleLong(header.lumps[AASLUMP_AREAS as usize].fileofs);
        length = LittleLong(header.lumps[AASLUMP_AREAS as usize].filelen);
        bot.aasworld.areas = AAS_LoadAASLump(
            bot,
            fp,
            offset,
            length,
            &mut lastoffset,
            core::mem::size_of::<aas_area_t>() as c_int,
        ) as *mut aas_area_t;
        bot.aasworld.numareas = length / core::mem::size_of::<aas_area_t>() as c_int;
        if bot.aasworld.numareas != 0 && bot.aasworld.areas.is_null() {
            return BLERR_CANNOTREADAASLUMP;
        }
        //area settings
        offset = LittleLong(header.lumps[AASLUMP_AREASETTINGS as usize].fileofs);
        length = LittleLong(header.lumps[AASLUMP_AREASETTINGS as usize].filelen);
        bot.aasworld.areasettings = AAS_LoadAASLump(
            bot,
            fp,
            offset,
            length,
            &mut lastoffset,
            core::mem::size_of::<aas_areasettings_t>() as c_int,
        ) as *mut aas_areasettings_t;
        bot.aasworld.numareasettings =
            length / core::mem::size_of::<aas_areasettings_t>() as c_int;
        if bot.aasworld.numareasettings != 0 && bot.aasworld.areasettings.is_null() {
            return BLERR_CANNOTREADAASLUMP;
        }
        //reachability list
        offset = LittleLong(header.lumps[AASLUMP_REACHABILITY as usize].fileofs);
        length = LittleLong(header.lumps[AASLUMP_REACHABILITY as usize].filelen);
        bot.aasworld.reachability = AAS_LoadAASLump(
            bot,
            fp,
            offset,
            length,
            &mut lastoffset,
            core::mem::size_of::<aas_reachability_t>() as c_int,
        ) as *mut aas_reachability_t;
        bot.aasworld.reachabilitysize =
            length / core::mem::size_of::<aas_reachability_t>() as c_int;
        if bot.aasworld.reachabilitysize != 0 && bot.aasworld.reachability.is_null() {
            return BLERR_CANNOTREADAASLUMP;
        }
        //nodes
        offset = LittleLong(header.lumps[AASLUMP_NODES as usize].fileofs);
        length = LittleLong(header.lumps[AASLUMP_NODES as usize].filelen);
        bot.aasworld.nodes = AAS_LoadAASLump(
            bot,
            fp,
            offset,
            length,
            &mut lastoffset,
            core::mem::size_of::<aas_node_t>() as c_int,
        ) as *mut aas_node_t;
        bot.aasworld.numnodes = length / core::mem::size_of::<aas_node_t>() as c_int;
        if bot.aasworld.numnodes != 0 && bot.aasworld.nodes.is_null() {
            return BLERR_CANNOTREADAASLUMP;
        }
        //cluster portals
        offset = LittleLong(header.lumps[AASLUMP_PORTALS as usize].fileofs);
        length = LittleLong(header.lumps[AASLUMP_PORTALS as usize].filelen);
        bot.aasworld.portals = AAS_LoadAASLump(
            bot,
            fp,
            offset,
            length,
            &mut lastoffset,
            core::mem::size_of::<aas_portal_t>() as c_int,
        ) as *mut aas_portal_t;
        bot.aasworld.numportals = length / core::mem::size_of::<aas_portal_t>() as c_int;
        if bot.aasworld.numportals != 0 && bot.aasworld.portals.is_null() {
            return BLERR_CANNOTREADAASLUMP;
        }
        //cluster portal index
        offset = LittleLong(header.lumps[AASLUMP_PORTALINDEX as usize].fileofs);
        length = LittleLong(header.lumps[AASLUMP_PORTALINDEX as usize].filelen);
        bot.aasworld.portalindex = AAS_LoadAASLump(
            bot,
            fp,
            offset,
            length,
            &mut lastoffset,
            core::mem::size_of::<aas_portalindex_t>() as c_int,
        ) as *mut aas_portalindex_t;
        bot.aasworld.portalindexsize =
            length / core::mem::size_of::<aas_portalindex_t>() as c_int;
        if bot.aasworld.portalindexsize != 0 && bot.aasworld.portalindex.is_null() {
            return BLERR_CANNOTREADAASLUMP;
        }
        //clusters
        offset = LittleLong(header.lumps[AASLUMP_CLUSTERS as usize].fileofs);
        length = LittleLong(header.lumps[AASLUMP_CLUSTERS as usize].filelen);
        bot.aasworld.clusters = AAS_LoadAASLump(
            bot,
            fp,
            offset,
            length,
            &mut lastoffset,
            core::mem::size_of::<aas_cluster_t>() as c_int,
        ) as *mut aas_cluster_t;
        bot.aasworld.numclusters = length / core::mem::size_of::<aas_cluster_t>() as c_int;
        if bot.aasworld.numclusters != 0 && bot.aasworld.clusters.is_null() {
            return BLERR_CANNOTREADAASLUMP;
        }
        //swap everything
        AAS_SwapAASData(bot);
        //aas file is loaded
        bot.aasworld.loaded = qtrue;
        //close the file
        (bot.botimport.FS_FCloseFile.unwrap())(fp);
        //
        BLERR_NOERROR
    }
} //end of the function AAS_LoadAASFile

/// Raven `AAS_WriteAASFile` — aas data is useless after writing to file because
/// it is byte swapped.
///
/// Source: `oracle/codemp/botlib/be_aas_file.cpp:506-565`
pub fn AAS_WriteAASFile(bot: &mut BotLib, filename: *mut c_char) -> qboolean {
    unsafe {
        let mut header: aas_header_t = core::mem::zeroed();
        let mut fp: fileHandle_t = 0;

        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"writing %s\n".as_ptr() as *mut c_char,
            filename,
        );
        //swap the aas data
        AAS_SwapAASData(bot);
        //initialize the file header
        Com_Memset(
            &mut header as *mut _ as *mut (),
            0,
            core::mem::size_of::<aas_header_t>(),
        );
        header.ident = LittleLong(AASID);
        header.version = LittleLong(AASVERSION);
        header.bspchecksum = LittleLong(bot.aasworld.bspchecksum);
        //open a new file
        (bot.botimport.FS_FOpenFile.unwrap())(filename, &mut fp, FS_WRITE);
        if fp == 0 {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"error opening %s\n".as_ptr() as *mut c_char,
                filename,
            );
            return qfalse;
        }
        //write the header
        (bot.botimport.FS_Write.unwrap())(
            &header as *const _ as *const c_void,
            core::mem::size_of::<aas_header_t>() as c_int,
            fp,
        );
        bot.AAS_WriteAASLump_offset = core::mem::size_of::<aas_header_t>() as c_int;
        //add the data lumps to the file
        let data = bot.aasworld.bboxes as *mut ();
        let length = bot.aasworld.numbboxes * core::mem::size_of::<aas_bbox_t>() as c_int;
        if AAS_WriteAASLump(bot, fp, &mut header, AASLUMP_BBOXES, data, length) == 0 {
            return qfalse;
        }
        let data = bot.aasworld.vertexes as *mut ();
        let length = bot.aasworld.numvertexes * core::mem::size_of::<aas_vertex_t>() as c_int;
        if AAS_WriteAASLump(bot, fp, &mut header, AASLUMP_VERTEXES, data, length) == 0 {
            return qfalse;
        }
        let data = bot.aasworld.planes as *mut ();
        let length = bot.aasworld.numplanes * core::mem::size_of::<aas_plane_t>() as c_int;
        if AAS_WriteAASLump(bot, fp, &mut header, AASLUMP_PLANES, data, length) == 0 {
            return qfalse;
        }
        let data = bot.aasworld.edges as *mut ();
        let length = bot.aasworld.numedges * core::mem::size_of::<aas_edge_t>() as c_int;
        if AAS_WriteAASLump(bot, fp, &mut header, AASLUMP_EDGES, data, length) == 0 {
            return qfalse;
        }
        let data = bot.aasworld.edgeindex as *mut ();
        let length = bot.aasworld.edgeindexsize * core::mem::size_of::<aas_edgeindex_t>() as c_int;
        if AAS_WriteAASLump(bot, fp, &mut header, AASLUMP_EDGEINDEX, data, length) == 0 {
            return qfalse;
        }
        let data = bot.aasworld.faces as *mut ();
        let length = bot.aasworld.numfaces * core::mem::size_of::<aas_face_t>() as c_int;
        if AAS_WriteAASLump(bot, fp, &mut header, AASLUMP_FACES, data, length) == 0 {
            return qfalse;
        }
        let data = bot.aasworld.faceindex as *mut ();
        let length = bot.aasworld.faceindexsize * core::mem::size_of::<aas_faceindex_t>() as c_int;
        if AAS_WriteAASLump(bot, fp, &mut header, AASLUMP_FACEINDEX, data, length) == 0 {
            return qfalse;
        }
        let data = bot.aasworld.areas as *mut ();
        let length = bot.aasworld.numareas * core::mem::size_of::<aas_area_t>() as c_int;
        if AAS_WriteAASLump(bot, fp, &mut header, AASLUMP_AREAS, data, length) == 0 {
            return qfalse;
        }
        let data = bot.aasworld.areasettings as *mut ();
        let length =
            bot.aasworld.numareasettings * core::mem::size_of::<aas_areasettings_t>() as c_int;
        if AAS_WriteAASLump(bot, fp, &mut header, AASLUMP_AREASETTINGS, data, length) == 0 {
            return qfalse;
        }
        let data = bot.aasworld.reachability as *mut ();
        let length =
            bot.aasworld.reachabilitysize * core::mem::size_of::<aas_reachability_t>() as c_int;
        if AAS_WriteAASLump(bot, fp, &mut header, AASLUMP_REACHABILITY, data, length) == 0 {
            return qfalse;
        }
        let data = bot.aasworld.nodes as *mut ();
        let length = bot.aasworld.numnodes * core::mem::size_of::<aas_node_t>() as c_int;
        if AAS_WriteAASLump(bot, fp, &mut header, AASLUMP_NODES, data, length) == 0 {
            return qfalse;
        }
        let data = bot.aasworld.portals as *mut ();
        let length = bot.aasworld.numportals * core::mem::size_of::<aas_portal_t>() as c_int;
        if AAS_WriteAASLump(bot, fp, &mut header, AASLUMP_PORTALS, data, length) == 0 {
            return qfalse;
        }
        let data = bot.aasworld.portalindex as *mut ();
        let length =
            bot.aasworld.portalindexsize * core::mem::size_of::<aas_portalindex_t>() as c_int;
        if AAS_WriteAASLump(bot, fp, &mut header, AASLUMP_PORTALINDEX, data, length) == 0 {
            return qfalse;
        }
        let data = bot.aasworld.clusters as *mut ();
        let length = bot.aasworld.numclusters * core::mem::size_of::<aas_cluster_t>() as c_int;
        if AAS_WriteAASLump(bot, fp, &mut header, AASLUMP_CLUSTERS, data, length) == 0 {
            return qfalse;
        }
        //rewrite the header with the added lumps
        (bot.botimport.FS_Seek.unwrap())(fp, 0 as c_long, fsOrigin_t::FS_SEEK_SET as c_int);
        AAS_DData(
            (&mut header as *mut _ as *mut c_uchar).add(8),
            core::mem::size_of::<aas_header_t>() as c_int - 8,
        );
        (bot.botimport.FS_Write.unwrap())(
            &header as *const _ as *const c_void,
            core::mem::size_of::<aas_header_t>() as c_int,
            fp,
        );
        //close the file
        (bot.botimport.FS_FCloseFile.unwrap())(fp);
        qtrue
    }
} //end of the function AAS_WriteAASFile
