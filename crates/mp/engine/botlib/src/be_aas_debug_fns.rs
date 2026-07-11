#![allow(
    non_camel_case_types,
    non_snake_case,
    unused_variables,
    unused_assignments
)]

//! Function bodies for Raven's `be_aas_debug.cpp` (AAS debug-draw helpers:
//! shown polygons/lines, area/face/reachability visualization, travel-type
//! printing, area flooding).
//!
//! Ported per the engine C-track packets (`botlib__0414`..`botlib__2233`).
//! Source: `oracle/codemp/botlib/be_aas_debug.cpp`.
//!
//! DESTINATION NOTE: the packet order names
//! `crates/mp/engine/botlib/src/be_aas_debug.rs`, but `be_aas_debug` already
//! exists as a directory module (`be_aas_debug/mod.rs`, constants-only) —
//! `_fns` escape per `_PREAMBLE.md`'s destination rule.
//!
//! PORT-NOTE(macros): Raven's vector `#define`s (`VectorCopy`, `VectorMA`,
//! `VectorSubtract`, `VectorScale`, `VectorClear`, `VectorSet`, `DotProduct`)
//! expand inline here, faithful to the preprocessor, matching the sibling
//! `be_aas_reach_fns.rs`/`be_aas_move.rs` convention. Only the genuine
//! q_math functions the packets flag as externals (`CrossProduct`,
//! `VectorNormalize`) are called through the existing `mp_game::q_math`
//! surface.
//! PORT-NOTE(unsafe): the AAS arena is a graph of raw pointers
//! (`aasworld.*`); bodies deref explicitly inside `unsafe` per
//! porting-rules §D11, matching the sibling files.
//!
// The `bot: &mut BotLib` / `common: &mut Common` receivers named in every
// signature below are the campaign's threaded-state aggregates (ruling 2);
// `BotLib` does not exist in this worktree slice yet (`_PREAMBLE.md`'s
// "botlib waves" note, `be_aas_cluster_fns.rs`/`be_aas_route_fns.rs`
// precedent). Every reference to `debuglines`/`debuglinevisible`/
// `numdebuglines`/`debugpolygons`/`aasworld`/`aassettings`/`botimport` below
// is the exact Raven global name per house rule, reached as a field on `bot`
// — resolved when the aggregate lands.

use core::ffi::c_int;

use mp_engine_qcommon::common::Common;
use mp_game::q_math::{CrossProduct, VectorNormalize};
use mp_qshared::common::mp::botlib::aas_clientmove_s::aas_clientmove_t;
use mp_qshared::common::mp::botlib::line_color::{
    LINECOLOR_BLUE, LINECOLOR_GREEN, LINECOLOR_NONE, LINECOLOR_RED, LINECOLOR_YELLOW,
};
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_MESSAGE};
use mp_qshared::shared::vec3_t;
use native_types::{qfalse, qtrue};

use crate::aasfile::aas_area_s::aas_area_t;
use crate::aasfile::aas_areasettings_s::aas_areasettings_t;
use crate::aasfile::aas_edge_s::aas_edge_t;
use crate::aasfile::aas_face_s::aas_face_t;
use crate::aasfile::aas_plane_s::aas_plane_t;
use crate::aasfile::aas_reachability_s::aas_reachability_t;
use crate::aasfile::area_contents::AREACONTENTS_VIEWPORTAL;
use crate::aasfile::face_flags::{FACE_GROUND, FACE_LADDER};
use crate::aasfile::presence_type::PRESENCE_NORMAL;
use crate::aasfile::travel_type::{
    TRAVELTYPE_MASK, TRAVEL_BARRIERJUMP, TRAVEL_BFGJUMP, TRAVEL_CROUCH, TRAVEL_ELEVATOR,
    TRAVEL_FUNCBOB, TRAVEL_GRAPPLEHOOK, TRAVEL_INVALID, TRAVEL_JUMP, TRAVEL_JUMPPAD, TRAVEL_LADDER,
    TRAVEL_ROCKETJUMP, TRAVEL_SWIM, TRAVEL_TELEPORT, TRAVEL_WALK, TRAVEL_WALKOFFLEDGE,
    TRAVEL_WATERJUMP,
};
use crate::be_aas_debug::be_aas_debug_cpp_consts::{MAX_DEBUGLINES, MAX_DEBUGPOLYGONS};
use crate::be_aas_sample_fns::{AAS_AreaCluster, AAS_PointAreaNum};
use crate::BotLib;

use crate::be_aas_main::AAS_Time;
use crate::l_memory_fns::GetClearedMemory;

/// Raven `AAS_ClearShownPolygons`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:43-60`
pub fn AAS_ClearShownPolygons(bot: &mut BotLib) {
    unsafe {
        for i in 0..MAX_DEBUGPOLYGONS as usize {
            if bot.debugpolygons[i] != 0 {
                bot.botimport.DebugPolygonDelete.unwrap()(bot.debugpolygons[i]);
            }
            bot.debugpolygons[i] = 0;
        }
    }
}

/// Raven `AAS_ShowPolygon`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:67-79`
pub fn AAS_ShowPolygon(bot: &mut BotLib, color: c_int, numpoints: c_int, points: *mut vec3_t) {
    unsafe {
        for i in 0..MAX_DEBUGPOLYGONS as usize {
            if bot.debugpolygons[i] == 0 {
                bot.debugpolygons[i] =
                    bot.botimport.DebugPolygonCreate.unwrap()(color, numpoints, points);
                break;
            }
        }
    }
}

/// Raven `AAS_ClearShownDebugLines`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:86-101`
pub fn AAS_ClearShownDebugLines(bot: &mut BotLib) {
    unsafe {
        // make all lines invisible
        for i in 0..MAX_DEBUGLINES as usize {
            if bot.debuglines[i] != 0 {
                // botimport.DebugLineShow(debuglines[i], NULL, NULL, LINECOLOR_NONE);
                let _ = LINECOLOR_NONE;
                bot.botimport.DebugLineDelete.unwrap()(bot.debuglines[i]);
                bot.debuglines[i] = 0;
                bot.debuglinevisible[i] = qfalse as c_int;
            }
        }
    }
}

/// Raven `AAS_DebugLine`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:108-127`
pub fn AAS_DebugLine(bot: &mut BotLib, mut start: vec3_t, mut end: vec3_t, color: c_int) {
    unsafe {
        for line in 0..MAX_DEBUGLINES as usize {
            if bot.debuglines[line] == 0 {
                bot.debuglines[line] = bot.botimport.DebugLineCreate.unwrap()();
                bot.debuglinevisible[line] = qfalse as c_int;
                bot.numdebuglines += 1;
            }
            if bot.debuglinevisible[line] == 0 {
                bot.botimport.DebugLineShow.unwrap()(
                    bot.debuglines[line],
                    &mut start as *mut vec3_t,
                    &mut end as *mut vec3_t,
                    color,
                );
                bot.debuglinevisible[line] = qtrue as c_int;
                return;
            }
        }
    }
}

/// Raven `AAS_PermanentLine` (comment: "AAS_PermenentLine" — Raven typo
/// preserved in the source's end-of-function comment only, not the name).
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:134-140`
pub fn AAS_PermanentLine(bot: &mut BotLib, mut start: vec3_t, mut end: vec3_t, color: c_int) {
    unsafe {
        let line = bot.botimport.DebugLineCreate.unwrap()();
        bot.botimport.DebugLineShow.unwrap()(
            line,
            &mut start as *mut vec3_t,
            &mut end as *mut vec3_t,
            color,
        );
    }
}

/// Raven `AAS_DrawPlaneCross`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:169-218`
pub fn AAS_DrawPlaneCross(
    bot: &mut BotLib,
    point: vec3_t,
    normal: vec3_t,
    dist: f32,
    r#type: c_int,
    color: c_int,
) {
    unsafe {
        // make a cross in the hit plane at the hit point
        let mut start1: vec3_t = point;
        let mut end1: vec3_t = point;
        let mut start2: vec3_t = point;
        let mut end2: vec3_t = point;

        let n0 = (r#type % 3) as usize;
        let n1 = ((r#type + 1) % 3) as usize;
        let n2 = ((r#type + 2) % 3) as usize;
        start1[n1] -= 6.0;
        start1[n2] -= 6.0;
        end1[n1] += 6.0;
        end1[n2] += 6.0;
        start2[n1] += 6.0;
        start2[n2] -= 6.0;
        end2[n1] -= 6.0;
        end2[n2] += 6.0;

        start1[n0] = (dist - (start1[n1] * normal[n1] + start1[n2] * normal[n2])) / normal[n0];
        end1[n0] = (dist - (end1[n1] * normal[n1] + end1[n2] * normal[n2])) / normal[n0];
        start2[n0] = (dist - (start2[n1] * normal[n1] + start2[n2] * normal[n2])) / normal[n0];
        end2[n0] = (dist - (end2[n1] * normal[n1] + end2[n2] * normal[n2])) / normal[n0];

        let mut lines = [0i32; 2];
        let mut j = 0usize;
        let mut line = 0usize;
        while j < 2 && line < MAX_DEBUGLINES as usize {
            if bot.debuglines[line] == 0 {
                bot.debuglines[line] = bot.botimport.DebugLineCreate.unwrap()();
                lines[j] = bot.debuglines[line];
                j += 1;
                bot.debuglinevisible[line] = qtrue as c_int;
                bot.numdebuglines += 1;
            } else if bot.debuglinevisible[line] == 0 {
                lines[j] = bot.debuglines[line];
                j += 1;
                bot.debuglinevisible[line] = qtrue as c_int;
            }
            line += 1;
        }
        bot.botimport.DebugLineShow.unwrap()(
            lines[0],
            &mut start1 as *mut vec3_t,
            &mut end1 as *mut vec3_t,
            color,
        );
        bot.botimport.DebugLineShow.unwrap()(
            lines[1],
            &mut start2 as *mut vec3_t,
            &mut end2 as *mut vec3_t,
            color,
        );
    }
}

/// Raven `AAS_ShowArea`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:377-462`
pub fn AAS_ShowArea(bot: &mut BotLib, areanum: c_int, groundfacesonly: c_int) {
    unsafe {
        let mut areaedges: [c_int; MAX_DEBUGLINES as usize] = [0; MAX_DEBUGLINES as usize];
        let mut numareaedges: c_int = 0;

        if areanum < 0 || areanum >= bot.aasworld.numareas {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                format!(
                    "area {} out of range [0, {}]\n",
                    areanum, bot.aasworld.numareas
                )
                .as_ptr() as *mut core::ffi::c_char,
            );
            return;
        }
        // pointer to the convex area
        let area: *mut aas_area_t = bot.aasworld.areas.add(areanum as usize);
        // walk through the faces of the area
        for i in 0..(*area).numfaces {
            let facenum = (*bot.aasworld.faceindex.add(((*area).firstface + i) as usize)).abs();
            // check if face number is in range
            if facenum >= bot.aasworld.numfaces {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    format!("facenum {} out of range\n", facenum).as_ptr()
                        as *mut core::ffi::c_char,
                );
            }
            let face: *mut aas_face_t = bot.aasworld.faces.add(facenum as usize);
            // ground faces only
            if groundfacesonly != 0 {
                if (*face).faceflags & (FACE_GROUND | FACE_LADDER) == 0 {
                    continue;
                }
            }
            // walk through the edges of the face
            for j in 0..(*face).numedges {
                // edge number
                let edgenum = (*bot.aasworld.edgeindex.add(((*face).firstedge + j) as usize)).abs();
                // check if edge number is in range
                if edgenum >= bot.aasworld.numedges {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        format!("edgenum {} out of range\n", edgenum).as_ptr()
                            as *mut core::ffi::c_char,
                    );
                }
                // check if the edge is stored already
                let mut n = 0;
                while n < numareaedges {
                    if areaedges[n as usize] == edgenum {
                        break;
                    }
                    n += 1;
                }
                if n == numareaedges && numareaedges < MAX_DEBUGLINES {
                    areaedges[numareaedges as usize] = edgenum;
                    numareaedges += 1;
                }
            }
            // AAS_ShowFace(facenum);
        }
        // draw all the edges
        let mut color: c_int = 0;
        for n in 0..numareaedges {
            let mut line: usize = 0;
            while line < MAX_DEBUGLINES as usize {
                if bot.debuglines[line] == 0 {
                    bot.debuglines[line] = bot.botimport.DebugLineCreate.unwrap()();
                    bot.debuglinevisible[line] = qfalse as c_int;
                    bot.numdebuglines += 1;
                }
                if bot.debuglinevisible[line] == 0 {
                    break;
                }
                line += 1;
            }
            if line >= MAX_DEBUGLINES as usize {
                return;
            }
            let edge: *mut aas_edge_t = bot.aasworld.edges.add(areaedges[n as usize] as usize);
            if color == LINECOLOR_RED {
                color = LINECOLOR_BLUE;
            } else if color == LINECOLOR_BLUE {
                color = LINECOLOR_GREEN;
            } else if color == LINECOLOR_GREEN {
                color = LINECOLOR_YELLOW;
            } else {
                color = LINECOLOR_RED;
            }
            let mut v0 = *bot.aasworld.vertexes.add((*edge).v[0] as usize);
            let mut v1 = *bot.aasworld.vertexes.add((*edge).v[1] as usize);
            bot.botimport.DebugLineShow.unwrap()(
                bot.debuglines[line],
                &mut v0 as *mut vec3_t,
                &mut v1 as *mut vec3_t,
                color,
            );
            bot.debuglinevisible[line] = qtrue as c_int;
        } //end for*/
    }
}

/// Raven `AAS_PrintTravelType`.
///
/// PORT-NOTE(debug-ifdef): the whole body is `#ifdef DEBUG` in the oracle;
/// this build has no `DEBUG` preprocessor concept (release-build parity), so
/// the faithful translation of the retail configuration is a no-op — matches
/// Raven's own retail binary. `bot`/`botimport` are therefore not threaded
/// (packet carries no STATE THREADED table for this fn).
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:528-555`
pub fn AAS_PrintTravelType(traveltype: c_int) {
    let _ = traveltype;
    let _ = (
        TRAVEL_INVALID,
        TRAVEL_WALK,
        TRAVEL_CROUCH,
        TRAVEL_BARRIERJUMP,
        TRAVEL_JUMP,
        TRAVEL_LADDER,
        TRAVEL_WALKOFFLEDGE,
        TRAVEL_SWIM,
        TRAVEL_TELEPORT,
        TRAVEL_ELEVATOR,
        TRAVEL_ROCKETJUMP,
        TRAVEL_BFGJUMP,
        TRAVEL_GRAPPLEHOOK,
        TRAVEL_JUMPPAD,
        TRAVEL_FUNCBOB,
        TRAVELTYPE_MASK,
        PRT_MESSAGE,
    );
    // #ifdef DEBUG only — release build: no-op (see PORT-NOTE above).
}

/// Raven `AAS_DrawPermanentCross`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:147-162`
pub fn AAS_DrawPermanentCross(bot: &mut BotLib, origin: vec3_t, size: f32, color: c_int) {
    unsafe {
        for i in 0..3usize {
            let mut start: vec3_t = origin;
            start[i] += size;
            let mut end: vec3_t = origin;
            end[i] -= size;
            AAS_DebugLine(bot, start, end, color);
            let debugline = bot.botimport.DebugLineCreate.unwrap()();
            bot.botimport.DebugLineShow.unwrap()(
                debugline,
                &mut start as *mut vec3_t,
                &mut end as *mut vec3_t,
                color,
            );
        }
    }
}

/// Raven `AAS_ShowBoundingBox`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:225-278`
pub fn AAS_ShowBoundingBox(bot: &mut BotLib, origin: vec3_t, mins: vec3_t, maxs: vec3_t) {
    unsafe {
        let mut bboxcorners: [vec3_t; 8] = [[0.0; 3]; 8];
        // upper corners
        bboxcorners[0][0] = origin[0] + maxs[0];
        bboxcorners[0][1] = origin[1] + maxs[1];
        bboxcorners[0][2] = origin[2] + maxs[2];
        //
        bboxcorners[1][0] = origin[0] + mins[0];
        bboxcorners[1][1] = origin[1] + maxs[1];
        bboxcorners[1][2] = origin[2] + maxs[2];
        //
        bboxcorners[2][0] = origin[0] + mins[0];
        bboxcorners[2][1] = origin[1] + mins[1];
        bboxcorners[2][2] = origin[2] + maxs[2];
        //
        bboxcorners[3][0] = origin[0] + maxs[0];
        bboxcorners[3][1] = origin[1] + mins[1];
        bboxcorners[3][2] = origin[2] + maxs[2];
        // lower corners
        // Com_Memcpy(bboxcorners[4], bboxcorners[0], sizeof(vec3_t) * 4);
        for k in 0..4 {
            bboxcorners[4 + k] = bboxcorners[k];
        }
        for i in 0..4 {
            bboxcorners[4 + i][2] = origin[2] + mins[2];
        }
        // draw bounding box
        let mut lines = [0i32; 3];
        for i in 0..4usize {
            let mut j = 0usize;
            let mut line = 0usize;
            while j < 3 && line < MAX_DEBUGLINES as usize {
                if bot.debuglines[line] == 0 {
                    bot.debuglines[line] = bot.botimport.DebugLineCreate.unwrap()();
                    lines[j] = bot.debuglines[line];
                    j += 1;
                    bot.debuglinevisible[line] = qtrue as c_int;
                    bot.numdebuglines += 1;
                } else if bot.debuglinevisible[line] == 0 {
                    lines[j] = bot.debuglines[line];
                    j += 1;
                    bot.debuglinevisible[line] = qtrue as c_int;
                }
                line += 1;
            }
            // top plane
            let mut a = bboxcorners[i];
            let mut b = bboxcorners[(i + 1) & 3];
            bot.botimport.DebugLineShow.unwrap()(
                lines[0],
                &mut a as *mut vec3_t,
                &mut b as *mut vec3_t,
                LINECOLOR_RED,
            );
            // bottom plane
            let mut c = bboxcorners[4 + i];
            let mut d = bboxcorners[4 + ((i + 1) & 3)];
            bot.botimport.DebugLineShow.unwrap()(
                lines[1],
                &mut c as *mut vec3_t,
                &mut d as *mut vec3_t,
                LINECOLOR_RED,
            );
            // vertical lines
            let mut e = bboxcorners[i];
            let mut f = bboxcorners[4 + i];
            bot.botimport.DebugLineShow.unwrap()(
                lines[2],
                &mut e as *mut vec3_t,
                &mut f as *mut vec3_t,
                LINECOLOR_RED,
            );
        }
    }
}

/// Raven `AAS_ShowFace`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:285-325`
pub fn AAS_ShowFace(bot: &mut BotLib, facenum: c_int) {
    unsafe {
        let mut color = LINECOLOR_YELLOW;
        // check if face number is in range
        if facenum >= bot.aasworld.numfaces {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                format!("facenum {} out of range\n", facenum).as_ptr() as *mut core::ffi::c_char,
            );
        }
        let face: *mut aas_face_t = bot.aasworld.faces.add(facenum as usize);
        // walk through the edges of the face
        for i in 0..(*face).numedges {
            // edge number
            let edgenum = (*bot.aasworld.edgeindex.add(((*face).firstedge + i) as usize)).abs();
            // check if edge number is in range
            if edgenum >= bot.aasworld.numedges {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    format!("edgenum {} out of range\n", edgenum).as_ptr()
                        as *mut core::ffi::c_char,
                );
            }
            let edge: *mut aas_edge_t = bot.aasworld.edges.add(edgenum as usize);
            if color == LINECOLOR_RED {
                color = LINECOLOR_GREEN;
            } else if color == LINECOLOR_GREEN {
                color = LINECOLOR_BLUE;
            } else if color == LINECOLOR_BLUE {
                color = LINECOLOR_YELLOW;
            } else {
                color = LINECOLOR_RED;
            }
            let v0 = *bot.aasworld.vertexes.add((*edge).v[0] as usize);
            let v1 = *bot.aasworld.vertexes.add((*edge).v[1] as usize);
            AAS_DebugLine(bot, v0, v1, color);
        }
        let plane: *mut aas_plane_t = bot.aasworld.planes.add((*face).planenum as usize);
        let edgenum = (*bot.aasworld.edgeindex.add((*face).firstedge as usize)).abs();
        let edge: *mut aas_edge_t = bot.aasworld.edges.add(edgenum as usize);
        let start: vec3_t = *bot.aasworld.vertexes.add((*edge).v[0] as usize);
        let mut end: vec3_t = [0.0; 3];
        for k in 0..3 {
            end[k] = start[k] + 20.0 * (*plane).normal[k];
        }
        AAS_DebugLine(bot, start, end, LINECOLOR_RED);
    }
}

/// Raven `AAS_ShowFacePolygon`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:332-370`
pub fn AAS_ShowFacePolygon(bot: &mut BotLib, facenum: c_int, color: c_int, flip: c_int) {
    unsafe {
        // check if face number is in range
        if facenum >= bot.aasworld.numfaces {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                format!("facenum {} out of range\n", facenum).as_ptr() as *mut core::ffi::c_char,
            );
        }
        let face: *mut aas_face_t = bot.aasworld.faces.add(facenum as usize);
        // walk through the edges of the face
        let mut points: [vec3_t; 128] = [[0.0; 3]; 128];
        let mut numpoints: usize = 0;
        if flip != 0 {
            let mut i = (*face).numedges - 1;
            while i >= 0 {
                // edge number
                let edgenum = *bot.aasworld.edgeindex.add(((*face).firstedge + i) as usize);
                let edge: *mut aas_edge_t = bot.aasworld.edges.add(edgenum.unsigned_abs() as usize);
                points[numpoints] = *bot
                    .aasworld
                    .vertexes
                    .add((*edge).v[(edgenum < 0) as usize] as usize);
                numpoints += 1;
                i -= 1;
            }
        } else {
            for i in 0..(*face).numedges {
                // edge number
                let edgenum = *bot.aasworld.edgeindex.add(((*face).firstedge + i) as usize);
                let edge: *mut aas_edge_t = bot.aasworld.edges.add(edgenum.unsigned_abs() as usize);
                points[numpoints] = *bot
                    .aasworld
                    .vertexes
                    .add((*edge).v[(edgenum < 0) as usize] as usize);
                numpoints += 1;
            }
        }
        AAS_ShowPolygon(bot, color, numpoints as c_int, points.as_mut_ptr());
    }
}

/// Raven `AAS_DrawCross`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:508-521`
pub fn AAS_DrawCross(bot: &mut BotLib, origin: vec3_t, size: f32, color: c_int) {
    for i in 0..3usize {
        let mut start: vec3_t = origin;
        start[i] += size;
        let mut end: vec3_t = origin;
        end[i] -= size;
        AAS_DebugLine(bot, start, end, color);
    }
}

/// Raven `AAS_DrawArrow`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:562-581`
pub fn AAS_DrawArrow(
    bot: &mut BotLib,
    start: vec3_t,
    end: vec3_t,
    linecolor: c_int,
    arrowcolor: c_int,
) {
    let mut dir: vec3_t = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
    VectorNormalize(&mut dir);
    let up: vec3_t = [0.0, 0.0, 1.0];
    let dot = dir[0] * up[0] + dir[1] * up[1] + dir[2] * up[2];
    let mut cross: vec3_t = [0.0; 3];
    if dot > 0.99 || dot < -0.99 {
        cross = [1.0, 0.0, 0.0];
    } else {
        CrossProduct(dir, up, &mut cross);
    }

    let mut p1: vec3_t = [
        end[0] + -6.0 * dir[0],
        end[1] + -6.0 * dir[1],
        end[2] + -6.0 * dir[2],
    ];
    let mut p2: vec3_t = p1;
    p1 = [
        p1[0] + 6.0 * cross[0],
        p1[1] + 6.0 * cross[1],
        p1[2] + 6.0 * cross[2],
    ];
    p2 = [
        p2[0] + -6.0 * cross[0],
        p2[1] + -6.0 * cross[1],
        p2[2] + -6.0 * cross[2],
    ];

    AAS_DebugLine(bot, start, end, linecolor);
    AAS_DebugLine(bot, p1, end, arrowcolor);
    AAS_DebugLine(bot, p2, end, arrowcolor);
}

/// Raven `AAS_ShowAreaPolygons`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:469-501`
pub fn AAS_ShowAreaPolygons(
    bot: &mut BotLib,
    areanum: c_int,
    color: c_int,
    groundfacesonly: c_int,
) {
    unsafe {
        if areanum < 0 || areanum >= bot.aasworld.numareas {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                format!(
                    "area {} out of range [0, {}]\n",
                    areanum, bot.aasworld.numareas
                )
                .as_ptr() as *mut core::ffi::c_char,
            );
            return;
        }
        // pointer to the convex area
        let area: *mut aas_area_t = bot.aasworld.areas.add(areanum as usize);
        // walk through the faces of the area
        for i in 0..(*area).numfaces {
            let facenum = (*bot.aasworld.faceindex.add(((*area).firstface + i) as usize)).abs();
            // check if face number is in range
            if facenum >= bot.aasworld.numfaces {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    format!("facenum {} out of range\n", facenum).as_ptr()
                        as *mut core::ffi::c_char,
                );
            }
            let face: *mut aas_face_t = bot.aasworld.faces.add(facenum as usize);
            // ground faces only
            if groundfacesonly != 0 && (*face).faceflags & (FACE_GROUND | FACE_LADDER) == 0 {
                continue;
            }
            let flip = ((*face).frontarea != areanum) as c_int;
            AAS_ShowFacePolygon(bot, facenum, color, flip);
        }
    }
}

/// Raven `AAS_FloodAreas_r`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:696-750`
pub fn AAS_FloodAreas_r(bot: &mut BotLib, areanum: c_int, cluster: c_int, done: *mut c_int) {
    unsafe {
        AAS_ShowAreaPolygons(bot, areanum, 1, qtrue as c_int);
        // pointer to the convex area
        let area: *mut aas_area_t = bot.aasworld.areas.add(areanum as usize);
        let settings: *mut aas_areasettings_t = bot.aasworld.areasettings.add(areanum as usize);
        // walk through the faces of the area
        for i in 0..(*area).numfaces {
            let facenum = (*bot.aasworld.faceindex.add(((*area).firstface + i) as usize)).abs();
            let face: *mut aas_face_t = bot.aasworld.faces.add(facenum as usize);
            let mut nextareanum = if (*face).frontarea == areanum {
                (*face).backarea
            } else {
                (*face).frontarea
            };
            if nextareanum == 0 {
                continue;
            }
            if *done.add(nextareanum as usize) != 0 {
                continue;
            }
            *done.add(nextareanum as usize) = qtrue as c_int;
            if (*bot.aasworld.areasettings.add(nextareanum as usize)).contents
                & AREACONTENTS_VIEWPORTAL
                != 0
            {
                continue;
            }
            if AAS_AreaCluster(bot, nextareanum) != cluster {
                continue;
            }
            AAS_FloodAreas_r(bot, nextareanum, cluster, done);
            let _ = nextareanum;
        }
        //
        for i in 0..(*settings).numreachableareas {
            let reach: *mut aas_reachability_t = bot
                .aasworld
                .reachability
                .add(((*settings).firstreachablearea + i) as usize);
            let nextareanum = (*reach).areanum;
            if nextareanum == 0 {
                continue;
            }
            if *done.add(nextareanum as usize) != 0 {
                continue;
            }
            *done.add(nextareanum as usize) = qtrue as c_int;
            if (*bot.aasworld.areasettings.add(nextareanum as usize)).contents
                & AREACONTENTS_VIEWPORTAL
                != 0
            {
                continue;
            }
            if AAS_AreaCluster(bot, nextareanum) != cluster {
                continue;
            }
            /*
            if ((reach->traveltype & TRAVELTYPE_MASK) == TRAVEL_WALKOFFLEDGE)
            {
                AAS_DebugLine(reach->start, reach->end, 1);
            }
            */
            AAS_FloodAreas_r(bot, nextareanum, cluster, done);
        }
    }
}

/// Raven `AAS_FloodAreas`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:752-760`
pub fn AAS_FloodAreas(bot: &mut BotLib, origin: vec3_t) {
    unsafe {
        let done = GetClearedMemory(
            bot,
            bot.aasworld.numareas as u64 * core::mem::size_of::<c_int>() as u64,
        ) as *mut c_int;
        let areanum = AAS_PointAreaNum(bot, origin);
        let cluster = AAS_AreaCluster(bot, areanum);
        AAS_FloodAreas_r(bot, areanum, cluster, done);
    }
}

/// Raven `AAS_ShowReachability`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:588-660`
pub fn AAS_ShowReachability(common: &mut Common, bot: &mut BotLib, reach: *mut aas_reachability_t) {
    let _ = &common;
    unsafe {
        AAS_ShowAreaPolygons(bot, (*reach).areanum, 5, qtrue as c_int);
        // AAS_ShowArea(reach->areanum, qtrue);
        AAS_DrawArrow(
            bot,
            (*reach).start,
            (*reach).end,
            LINECOLOR_BLUE,
            LINECOLOR_YELLOW,
        );
        //
        let traveltype = (*reach).traveltype & TRAVELTYPE_MASK;
        if traveltype == TRAVEL_JUMP || traveltype == TRAVEL_WALKOFFLEDGE {
            let mut speed: f32 = 0.0;
            crate::be_aas_move::AAS_HorizontalVelocityForJump(
                bot,
                bot.aassettings.phys_jumpvel,
                (*reach).start,
                (*reach).end,
                &mut speed as *mut f32,
            );
            //
            let mut dir: vec3_t = [
                (*reach).end[0] - (*reach).start[0],
                (*reach).end[1] - (*reach).start[1],
                (*reach).end[2] - (*reach).start[2],
            ];
            dir[2] = 0.0;
            VectorNormalize(&mut dir);
            // set the velocity
            let velocity: vec3_t = [dir[0] * speed, dir[1] * speed, dir[2] * speed];
            // set the command movement
            let mut cmdmove: vec3_t = [0.0; 3];
            cmdmove[2] = bot.aassettings.phys_jumpvel;
            //
            let mut r#move: aas_clientmove_t = core::mem::zeroed();
            crate::be_aas_move::AAS_PredictClientMovement(
                bot,
                &mut r#move as *mut aas_clientmove_t,
                -1,
                (*reach).start,
                PRESENCE_NORMAL,
                qtrue as c_int,
                velocity,
                cmdmove,
                3,
                30,
                0.1f32,
                mp_qshared::common::mp::botlib::aas_stop_event::SE_HITGROUND
                    | mp_qshared::common::mp::botlib::aas_stop_event::SE_ENTERWATER
                    | mp_qshared::common::mp::botlib::aas_stop_event::SE_ENTERSLIME
                    | mp_qshared::common::mp::botlib::aas_stop_event::SE_ENTERLAVA
                    | mp_qshared::common::mp::botlib::aas_stop_event::SE_HITGROUNDDAMAGE,
                0,
                qtrue as c_int,
            );
            //
            if traveltype == TRAVEL_JUMP {
                crate::be_aas_move::AAS_JumpReachRunStart(common, bot, reach, dir);
                AAS_DrawCross(bot, dir, 4.0, LINECOLOR_BLUE);
            }
        } else if traveltype == TRAVEL_ROCKETJUMP {
            let zvel = crate::be_aas_move::AAS_RocketJumpZVelocity(bot, (*reach).start);
            let mut speed: f32 = 0.0;
            crate::be_aas_move::AAS_HorizontalVelocityForJump(
                bot,
                zvel,
                (*reach).start,
                (*reach).end,
                &mut speed as *mut f32,
            );
            //
            let mut dir: vec3_t = [
                (*reach).end[0] - (*reach).start[0],
                (*reach).end[1] - (*reach).start[1],
                (*reach).end[2] - (*reach).start[2],
            ];
            dir[2] = 0.0;
            VectorNormalize(&mut dir);
            // get command movement
            let cmdmove: vec3_t = [dir[0] * speed, dir[1] * speed, dir[2] * speed];
            let velocity: vec3_t = [0.0, 0.0, zvel];
            //
            let mut r#move: aas_clientmove_t = core::mem::zeroed();
            crate::be_aas_move::AAS_PredictClientMovement(
                bot,
                &mut r#move as *mut aas_clientmove_t,
                -1,
                (*reach).start,
                PRESENCE_NORMAL,
                qtrue as c_int,
                velocity,
                cmdmove,
                30,
                30,
                0.1f32,
                mp_qshared::common::mp::botlib::aas_stop_event::SE_ENTERWATER
                    | mp_qshared::common::mp::botlib::aas_stop_event::SE_ENTERSLIME
                    | mp_qshared::common::mp::botlib::aas_stop_event::SE_ENTERLAVA
                    | mp_qshared::common::mp::botlib::aas_stop_event::SE_HITGROUNDDAMAGE
                    | mp_qshared::common::mp::botlib::aas_stop_event::SE_TOUCHJUMPPAD
                    | mp_qshared::common::mp::botlib::aas_stop_event::SE_HITGROUNDAREA,
                (*reach).areanum,
                qtrue as c_int,
            );
        } else if traveltype == TRAVEL_JUMPPAD {
            let mut cmdmove: vec3_t = [0.0, 0.0, 0.0];
            //
            let mut dir: vec3_t = [
                (*reach).end[0] - (*reach).start[0],
                (*reach).end[1] - (*reach).start[1],
                (*reach).end[2] - (*reach).start[2],
            ];
            dir[2] = 0.0;
            VectorNormalize(&mut dir);
            // set the velocity
            // NOTE: the edgenum is the horizontal velocity
            let mut velocity: vec3_t = [
                dir[0] * (*reach).edgenum as f32,
                dir[1] * (*reach).edgenum as f32,
                dir[2] * (*reach).edgenum as f32,
            ];
            // NOTE: the facenum is the Z velocity
            velocity[2] = (*reach).facenum as f32;
            //
            let mut r#move: aas_clientmove_t = core::mem::zeroed();
            crate::be_aas_move::AAS_PredictClientMovement(
                bot,
                &mut r#move as *mut aas_clientmove_t,
                -1,
                (*reach).start,
                PRESENCE_NORMAL,
                qtrue as c_int,
                velocity,
                cmdmove,
                30,
                30,
                0.1f32,
                mp_qshared::common::mp::botlib::aas_stop_event::SE_ENTERWATER
                    | mp_qshared::common::mp::botlib::aas_stop_event::SE_ENTERSLIME
                    | mp_qshared::common::mp::botlib::aas_stop_event::SE_ENTERLAVA
                    | mp_qshared::common::mp::botlib::aas_stop_event::SE_HITGROUNDDAMAGE
                    | mp_qshared::common::mp::botlib::aas_stop_event::SE_TOUCHJUMPPAD
                    | mp_qshared::common::mp::botlib::aas_stop_event::SE_HITGROUNDAREA,
                (*reach).areanum,
                qtrue as c_int,
            );
            let _ = cmdmove;
        }
    }
}

/// Raven `AAS_ShowReachableAreas` (Raven's own end-comment reads
/// "ShowReachableAreas").
///
/// Function-scope statics `reach`/`index`/`lastareanum`/`lasttime` are
/// genuine cross-frame state (fork-3 rule) → fields on `bot`, named exactly
/// per Raven, matching the `hidetraveltimes` precedent in
/// `be_aas_route_fns.rs`.
///
/// Source: `oracle/codemp/botlib/be_aas_debug.cpp:667-694`
pub fn AAS_ShowReachableAreas(common: &mut Common, bot: &mut BotLib, areanum: c_int) {
    unsafe {
        if areanum != bot.lastareanum {
            bot.index = 0;
            bot.lastareanum = areanum;
        }
        let settings: *mut aas_areasettings_t = bot.aasworld.areasettings.add(areanum as usize);
        //
        if (*settings).numreachableareas == 0 {
            return;
        }
        //
        if bot.index >= (*settings).numreachableareas {
            bot.index = 0;
        }
        //
        if AAS_Time(bot) - bot.lasttime > 1.5 {
            bot.reach = *bot
                .aasworld
                .reachability
                .add(((*settings).firstreachablearea + bot.index) as usize);
            bot.index += 1;
            bot.lasttime = AAS_Time(bot);
            AAS_PrintTravelType(bot.reach.traveltype & TRAVELTYPE_MASK);
            bot.botimport.Print.unwrap()(PRT_MESSAGE, c"\n".as_ptr() as *mut core::ffi::c_char);
        }
        let __reach_ptr = core::ptr::addr_of_mut!(bot.reach);
        AAS_ShowReachability(common, bot, __reach_ptr);
    }
}
