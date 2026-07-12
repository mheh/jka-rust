#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_assignments
)]

//! Function bodies for Raven's `be_aas_cluster.cpp` (AAS cluster/portal
//! computation: cluster flooding, portal detection, view-portal handling).
//!
//! Ported per the engine C-track packets (`botlib__0406`..`botlib__1704`).
//! Source: `oracle/codemp/botlib/be_aas_cluster.cpp`.
//!
//! DESTINATION NOTE: the packet order named
//! `crates/mp/engine/botlib/src/be_aas_cluster.rs`, but `be_aas_cluster`
//! already exists as a directory module (`be_aas_cluster/mod.rs`,
//! constants-only) — `_fns` escape per `_PREAMBLE.md`'s destination rule.
//!
//! PORT-NOTE(unsafe): the AAS arena is a graph of raw pointers
//! (`aasworld.*`); bodies deref explicitly inside `unsafe` per
//! porting-rules §D11, matching the sibling `be_aas_reach_fns.rs`/
//! `be_aas_route_fns.rs` convention.

use core::ffi::c_char;
use core::ffi::c_int;

use native_types::{qboolean, qfalse, qtrue};

use crate::aasfile::aas_area_s::aas_area_t;
use crate::aasfile::aas_cluster_s::aas_cluster_t;
use crate::aasfile::aas_face_s::aas_face_t;
use crate::aasfile::aas_portal_s::aas_portal_t;
use crate::aasfile::area_contents::{
    AREACONTENTS_CLUSTERPORTAL, AREACONTENTS_ROUTEPORTAL, AREACONTENTS_VIEWPORTAL,
};
use crate::aasfile::area_flags::AREA_GROUNDED;
use crate::aasfile::face_flags::FACE_SOLID;
use crate::be_aas_cluster::consts::{
    AAS_MAX_CLUSTERS, AAS_MAX_PORTALINDEXSIZE, AAS_MAX_PORTALS, MAX_PORTALAREAS,
};
use mp_qshared::common::mp::botlib::print_type::PRT_MESSAGE;

use crate::BotLib;

use crate::be_aas_main::AAS_Error;
use crate::be_aas_reach_fns::AAS_AreaReachability;
use crate::l_libvar_fns::LibVarGetValue;
use crate::l_log_fns::Log_Write;
use crate::l_memory_fns::{FreeMemory, GetClearedMemory};
use mp_engine_qcommon::common_fns::Com_Memset;

/// Raven `AAS_RemoveClusterAreas` — clears every area's cluster mark.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:46-54`
pub fn AAS_RemoveClusterAreas(bot: &mut BotLib) {
    unsafe {
        for i in 1..bot.aasworld.numareas {
            (*bot.aasworld.areasettings.add(i as usize)).cluster = 0;
        }
    }
}

/// Raven `AAS_ClearCluster` — clears the cluster mark of every area
/// belonging to `clusternum`.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:61-72`
pub fn AAS_ClearCluster(bot: &mut BotLib, clusternum: c_int) {
    unsafe {
        for i in 1..bot.aasworld.numareas {
            let settings = bot.aasworld.areasettings.add(i as usize);
            if (*settings).cluster == clusternum {
                (*settings).cluster = 0;
            }
        }
    }
}

/// Raven `AAS_RemovePortalsClusterReference` — clears any portal's front/back
/// cluster reference matching `clusternum`.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:79-94`
pub fn AAS_RemovePortalsClusterReference(bot: &mut BotLib, clusternum: c_int) {
    unsafe {
        for portalnum in 1..bot.aasworld.numportals {
            let portal = bot.aasworld.portals.add(portalnum as usize);
            if (*portal).frontcluster == clusternum {
                (*portal).frontcluster = 0;
            }
            if (*portal).backcluster == clusternum {
                (*portal).backcluster = 0;
            }
        } //end for
    }
}

/// Raven `AAS_NumberClusterPortals` — numbers the portals of a cluster,
/// assigning each portal a `clusterareanum` slot on its front/back side.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:271-291`
pub fn AAS_NumberClusterPortals(bot: &mut BotLib, clusternum: c_int) {
    unsafe {
        let cluster: *mut aas_cluster_t = bot.aasworld.clusters.add(clusternum as usize);
        for i in 0..(*cluster).numportals {
            let portalnum = *bot
                .aasworld
                .portalindex
                .add(((*cluster).firstportal + i) as usize);
            let portal: *mut aas_portal_t = bot.aasworld.portals.add(portalnum as usize);
            if (*portal).frontcluster == clusternum {
                (*portal).clusterareanum[0] = (*cluster).numareas;
                (*cluster).numareas += 1;
            } else {
                (*portal).clusterareanum[1] = (*cluster).numareas;
                (*cluster).numareas += 1;
            } //end else
        } //end for
    }
}

/// Raven `AAS_ConnectedAreas_r` — recursively marks all areas connected (via
/// faces) to `curarea` in `connectedareas`.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:670-699`
pub fn AAS_ConnectedAreas_r(
    bot: &mut BotLib,
    areanums: *mut c_int,
    numareas: c_int,
    connectedareas: *mut c_int,
    curarea: c_int,
) {
    unsafe {
        *connectedareas.add(curarea as usize) = qtrue as c_int;
        let area: *mut aas_area_t = bot
            .aasworld
            .areas
            .add(*areanums.add(curarea as usize) as usize);
        for i in 0..(*area).numfaces {
            let facenum = (*bot.aasworld.faceindex.add(((*area).firstface + i) as usize)).abs();
            let face: *mut aas_face_t = bot.aasworld.faces.add(facenum as usize);
            //if the face is solid
            if (*face).faceflags & FACE_SOLID != 0 {
                continue;
            }
            //get the area at the other side of the face
            let otherareanum = if (*face).frontarea != *areanums.add(curarea as usize) {
                (*face).frontarea
            } else {
                (*face).backarea
            };
            //check if the face is leading to one of the other areas
            let mut j = 0;
            while j < numareas {
                if *areanums.add(j as usize) == otherareanum {
                    break;
                }
                j += 1;
            } //end for
              //if the face isn't leading to one of the other areas
            if j == numareas {
                continue;
            }
            //if the other area is already connected
            if *connectedareas.add(j as usize) != 0 {
                continue;
            }
            //recursively proceed with the other area
            AAS_ConnectedAreas_r(bot, areanums, numareas, connectedareas, j);
        } //end for
    }
}

/// Raven `AAS_RemoveAllPortals` — strips the cluster-portal content flag
/// from every area.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:920-928`
pub fn AAS_RemoveAllPortals(bot: &mut BotLib) {
    unsafe {
        for i in 1..bot.aasworld.numareas {
            (*bot.aasworld.areasettings.add(i as usize)).contents &= !AREACONTENTS_CLUSTERPORTAL;
        }
    }
}

/// Raven `AAS_CreateViewPortals` — promotes every cluster portal to a view
/// portal.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:1405-1416`
pub fn AAS_CreateViewPortals(bot: &mut BotLib) {
    unsafe {
        for i in 1..bot.aasworld.numareas {
            let settings = bot.aasworld.areasettings.add(i as usize);
            if (*settings).contents & AREACONTENTS_CLUSTERPORTAL != 0 {
                (*settings).contents |= AREACONTENTS_VIEWPORTAL;
            }
        }
    }
}

/// Raven `AAS_SetViewPortalsAsClusterPortals` — promotes every view portal
/// (forced by the map's view-portal brushes) to a cluster portal.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:1423-1434`
pub fn AAS_SetViewPortalsAsClusterPortals(bot: &mut BotLib) {
    unsafe {
        for i in 1..bot.aasworld.numareas {
            let settings = bot.aasworld.areasettings.add(i as usize);
            if (*settings).contents & AREACONTENTS_VIEWPORTAL != 0 {
                (*settings).contents |= AREACONTENTS_CLUSTERPORTAL;
            }
        }
    }
}

/// Raven `AAS_UpdatePortal` — attaches `clusternum` to the portal owning
/// `areanum` (front side first, then back side); demotes the portal to a
/// regular area if it would border a third cluster.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:101-153`
pub fn AAS_UpdatePortal(bot: &mut BotLib, areanum: c_int, clusternum: c_int) -> c_int {
    unsafe {
        //find the portal of the area
        let mut portalnum = 1;
        while portalnum < bot.aasworld.numportals {
            if (*bot.aasworld.portals.add(portalnum as usize)).areanum == areanum {
                break;
            }
            portalnum += 1;
        } //end for
          //
        if portalnum == bot.aasworld.numportals {
            let __ae =
                std::ffi::CString::new(format!("no portal of area {}", areanum)).unwrap_or_default();
            AAS_Error(bot, __ae.as_ptr() as *mut c_char);
            return qtrue as c_int;
        } //end if
          //
        let portal: *mut aas_portal_t = bot.aasworld.portals.add(portalnum as usize);
        //if the portal is already fully updated
        if (*portal).frontcluster == clusternum {
            return qtrue as c_int;
        }
        if (*portal).backcluster == clusternum {
            return qtrue as c_int;
        }
        //if the portal has no front cluster yet
        if (*portal).frontcluster == 0 {
            (*portal).frontcluster = clusternum;
        }
        //if the portal has no back cluster yet
        else if (*portal).backcluster == 0 {
            (*portal).backcluster = clusternum;
        } else {
            //remove the cluster portal flag contents
            (*bot.aasworld.areasettings.add(areanum as usize)).contents &=
                !AREACONTENTS_CLUSTERPORTAL;
            let __m = std::ffi::CString::new(format!(
                "portal area {} is seperating more than two clusters\r\n",
                areanum
            ))
            .unwrap_or_default();
            Log_Write(bot, __m.as_ptr() as *mut c_char);
            return qfalse as c_int;
        } //end else
        if bot.aasworld.portalindexsize >= AAS_MAX_PORTALINDEXSIZE {
            AAS_Error(bot, c"AAS_MAX_PORTALINDEXSIZE".as_ptr() as *mut c_char);
            return qtrue as c_int;
        } //end if
          //set the area cluster number to the negative portal number
        (*bot.aasworld.areasettings.add(areanum as usize)).cluster = -portalnum;
        //add the portal to the cluster using the portal index
        let cluster: *mut aas_cluster_t = bot.aasworld.clusters.add(clusternum as usize);
        *bot.aasworld
            .portalindex
            .add(((*cluster).firstportal + (*cluster).numportals) as usize) = portalnum;
        bot.aasworld.portalindexsize += 1;
        (*cluster).numportals += 1;
        qtrue as c_int
    }
}

/// Raven `AAS_CreatePortals` — turns every cluster-portal area into an
/// `aas_portal_t` entry.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:422-444`
pub fn AAS_CreatePortals(bot: &mut BotLib) {
    unsafe {
        for i in 1..bot.aasworld.numareas {
            //if the area is a cluster portal
            if (*bot.aasworld.areasettings.add(i as usize)).contents & AREACONTENTS_CLUSTERPORTAL
                != 0
            {
                if bot.aasworld.numportals >= AAS_MAX_PORTALS {
                    AAS_Error(bot, c"AAS_MAX_PORTALS".as_ptr() as *mut c_char);
                    return;
                } //end if
                let portal: *mut aas_portal_t =
                    bot.aasworld.portals.add(bot.aasworld.numportals as usize);
                (*portal).areanum = i;
                (*portal).frontcluster = 0;
                (*portal).backcluster = 0;
                bot.aasworld.numportals += 1;
            } //end if
        } //end for
    }
}

/// Raven `AAS_ConnectedAreas` — true if every area in `areanums` is
/// face-reachable from the first.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:706-719`
pub fn AAS_ConnectedAreas(bot: &mut BotLib, areanums: *mut c_int, numareas: c_int) -> qboolean {
    let mut connectedareas = [0 as c_int; MAX_PORTALAREAS as usize];

    Com_Memset(
        connectedareas.as_mut_ptr() as *mut (),
        0,
        core::mem::size_of_val(&connectedareas),
    );
    if numareas < 1 {
        return qfalse;
    }
    if numareas == 1 {
        return qtrue;
    }
    AAS_ConnectedAreas_r(bot, areanums, numareas, connectedareas.as_mut_ptr(), 0);
    for i in 0..numareas {
        if connectedareas[i as usize] == 0 {
            return qfalse;
        }
    } //end for
    qtrue
}

/// Raven `AAS_GetAdjacentAreasWithLessPresenceTypes_r` — recursively collects
/// `curareanum` plus every adjacent area whose presence types are a strict
/// subset of it, into `areanums`.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:727-769`
pub fn AAS_GetAdjacentAreasWithLessPresenceTypes_r(
    bot: &mut BotLib,
    areanums: *mut c_int,
    numareas: c_int,
    curareanum: c_int,
) -> c_int {
    unsafe {
        let mut numareas = numareas;
        *areanums.add(numareas as usize) = curareanum;
        numareas += 1;
        let area: *mut aas_area_t = bot.aasworld.areas.add(curareanum as usize);
        let presencetype = (*bot.aasworld.areasettings.add(curareanum as usize)).presencetype;
        for i in 0..(*area).numfaces {
            let facenum = (*bot.aasworld.faceindex.add(((*area).firstface + i) as usize)).abs();
            let face: *mut aas_face_t = bot.aasworld.faces.add(facenum as usize);
            //if the face is solid
            if (*face).faceflags & FACE_SOLID != 0 {
                continue;
            }
            //the area at the other side of the face
            let otherareanum = if (*face).frontarea != curareanum {
                (*face).frontarea
            } else {
                (*face).backarea
            };
            //
            let otherpresencetype =
                (*bot.aasworld.areasettings.add(otherareanum as usize)).presencetype;
            //if the other area has less presence types
            if (presencetype & !otherpresencetype) != 0 && (otherpresencetype & !presencetype) == 0
            {
                //check if the other area isn't already in the list
                let mut j = 0;
                while j < numareas {
                    if otherareanum == *areanums.add(j as usize) {
                        break;
                    }
                    j += 1;
                } //end for
                  //if the other area isn't already in the list
                if j == numareas {
                    if numareas >= MAX_PORTALAREAS {
                        AAS_Error(bot, c"MAX_PORTALAREAS".as_ptr() as *mut c_char);
                        return numareas;
                    } //end if
                    numareas = AAS_GetAdjacentAreasWithLessPresenceTypes_r(
                        bot,
                        areanums,
                        numareas,
                        otherareanum,
                    );
                } //end if
            } //end if
        } //end for
        numareas
    }
}

/// Raven `AAS_TestPortals` — verifies every portal has both a front and back
/// cluster; strips the cluster-portal flag and fails otherwise.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:1355-1377`
pub fn AAS_TestPortals(bot: &mut BotLib) -> c_int {
    unsafe {
        for i in 1..bot.aasworld.numportals {
            let portal: *mut aas_portal_t = bot.aasworld.portals.add(i as usize);
            if (*portal).frontcluster == 0 {
                (*bot.aasworld.areasettings.add((*portal).areanum as usize)).contents &=
                    !AREACONTENTS_CLUSTERPORTAL;
                let __m = std::ffi::CString::new(format!(
                    "portal area {} has no front cluster\r\n",
                    (*portal).areanum
                ))
                .unwrap_or_default();
                Log_Write(bot, __m.as_ptr() as *mut c_char);
                return qfalse as c_int;
            } //end if
            if (*portal).backcluster == 0 {
                (*bot.aasworld.areasettings.add((*portal).areanum as usize)).contents &=
                    !AREACONTENTS_CLUSTERPORTAL;
                let __m = std::ffi::CString::new(format!(
                    "portal area {} has no back cluster\r\n",
                    (*portal).areanum
                ))
                .unwrap_or_default();
                Log_Write(bot, __m.as_ptr() as *mut c_char);
                return qfalse as c_int;
            } //end if
        } //end for
        qtrue as c_int
    }
}

/// Raven `AAS_CountForcedClusterPortals` — logs and reports the number of
/// areas still forced to be cluster portals.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:1384-1398`
pub fn AAS_CountForcedClusterPortals(bot: &mut BotLib) {
    unsafe {
        let mut num: c_int = 0;
        for i in 1..bot.aasworld.numareas {
            if (*bot.aasworld.areasettings.add(i as usize)).contents & AREACONTENTS_CLUSTERPORTAL
                != 0
            {
                let __m = std::ffi::CString::new(format!(
                    "area {} is a forced portal area\r\n",
                    i
                ))
                .unwrap_or_default();
                Log_Write(bot, __m.as_ptr() as *mut c_char);
                num += 1;
            } //end if
        } //end for
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"%6d forced portal areas\n".as_ptr() as *mut c_char,
            num,
        );
    }
}

/// Raven `AAS_FloodClusterAreas_r` — recursively floods `clusternum` into
/// `areanum` and its face/reachability neighbors, stopping at cluster
/// portals and other clusters.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:160-225`
pub fn AAS_FloodClusterAreas_r(bot: &mut BotLib, areanum: c_int, clusternum: c_int) -> c_int {
    unsafe {
        //
        if areanum <= 0 || areanum >= bot.aasworld.numareas {
            AAS_Error(
                bot,
                c"AAS_FloodClusterAreas_r: areanum out of range".as_ptr() as *mut c_char,
            );
            return qfalse as c_int;
        } //end if
          //if the area is already part of a cluster
        let settings = bot.aasworld.areasettings.add(areanum as usize);
        if (*settings).cluster > 0 {
            if (*settings).cluster == clusternum {
                return qtrue as c_int;
            }
            //
            //there's a reachability going from one cluster to another only in one direction
            //
            let __ae = std::ffi::CString::new(format!(
                "cluster {} touched cluster {} at area {}\r\n",
                clusternum,
                (*settings).cluster,
                areanum
            ))
            .unwrap_or_default();
            AAS_Error(bot, __ae.as_ptr() as *mut c_char);
            return qfalse as c_int;
        } //end if
          //don't add the cluster portal areas to the clusters
        if (*settings).contents & AREACONTENTS_CLUSTERPORTAL != 0 {
            return AAS_UpdatePortal(bot, areanum, clusternum);
        } //end if
          //set the area cluster number
        (*settings).cluster = clusternum;
        (*settings).clusterareanum = (*bot.aasworld.clusters.add(clusternum as usize)).numareas;
        //the cluster has an extra area
        (*bot.aasworld.clusters.add(clusternum as usize)).numareas += 1;

        let area: *mut aas_area_t = bot.aasworld.areas.add(areanum as usize);
        //use area faces to flood into adjacent areas
        if bot.nofaceflood == 0 {
            for i in 0..(*area).numfaces {
                let facenum = (*bot.aasworld.faceindex.add(((*area).firstface + i) as usize)).abs();
                let face: *mut aas_face_t = bot.aasworld.faces.add(facenum as usize);
                if (*face).frontarea == areanum {
                    if (*face).backarea != 0
                        && AAS_FloodClusterAreas_r(bot, (*face).backarea, clusternum) == 0
                    {
                        return qfalse as c_int;
                    } //end if
                }
                //end if
                else {
                    if (*face).frontarea != 0
                        && AAS_FloodClusterAreas_r(bot, (*face).frontarea, clusternum) == 0
                    {
                        return qfalse as c_int;
                    } //end if
                } //end else
            } //end for
        } //end if
          //use the reachabilities to flood into other areas
        let settings = bot.aasworld.areasettings.add(areanum as usize);
        for i in 0..(*settings).numreachableareas {
            let reach = bot
                .aasworld
                .reachability
                .add(((*settings).firstreachablearea + i) as usize);
            if (*reach).areanum == 0 {
                continue;
            } //end if
            if AAS_FloodClusterAreas_r(bot, (*reach).areanum, clusternum) == 0 {
                return qfalse as c_int;
            }
        } //end for
        qtrue as c_int
    }
}

/// Raven `AAS_NumberClusterAreas` — numbers every area (and portal side) of
/// `clusternum`, reachability-bearing areas first.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:298-365`
pub fn AAS_NumberClusterAreas(bot: &mut BotLib, clusternum: c_int) {
    unsafe {
        (*bot.aasworld.clusters.add(clusternum as usize)).numareas = 0;
        (*bot.aasworld.clusters.add(clusternum as usize)).numreachabilityareas = 0;
        //number all areas in this cluster WITH reachabilities
        for i in 1..bot.aasworld.numareas {
            //
            if (*bot.aasworld.areasettings.add(i as usize)).cluster != clusternum {
                continue;
            }
            //
            if AAS_AreaReachability(bot, i) == 0 {
                continue;
            }
            //
            let cluster = bot.aasworld.clusters.add(clusternum as usize);
            (*bot.aasworld.areasettings.add(i as usize)).clusterareanum = (*cluster).numareas;
            //the cluster has an extra area
            (*cluster).numareas += 1;
            (*cluster).numreachabilityareas += 1;
        } //end for
          //number all portals in this cluster WITH reachabilities
        let cluster: *mut aas_cluster_t = bot.aasworld.clusters.add(clusternum as usize);
        for i in 0..(*cluster).numportals {
            let portalnum = *bot
                .aasworld
                .portalindex
                .add(((*cluster).firstportal + i) as usize);
            let portal: *mut aas_portal_t = bot.aasworld.portals.add(portalnum as usize);
            if AAS_AreaReachability(bot, (*portal).areanum) == 0 {
                continue;
            }
            if (*portal).frontcluster == clusternum {
                (*portal).clusterareanum[0] = (*cluster).numareas;
                (*cluster).numareas += 1;
                (*cluster).numreachabilityareas += 1;
            }
            //end if
            else {
                (*portal).clusterareanum[1] = (*cluster).numareas;
                (*cluster).numareas += 1;
                (*cluster).numreachabilityareas += 1;
            } //end else
        } //end for
          //number all areas in this cluster WITHOUT reachabilities
        for i in 1..bot.aasworld.numareas {
            //
            if (*bot.aasworld.areasettings.add(i as usize)).cluster != clusternum {
                continue;
            }
            //
            if AAS_AreaReachability(bot, i) != 0 {
                continue;
            }
            //
            let cluster = bot.aasworld.clusters.add(clusternum as usize);
            (*bot.aasworld.areasettings.add(i as usize)).clusterareanum = (*cluster).numareas;
            //the cluster has an extra area
            (*cluster).numareas += 1;
        } //end for
          //number all portals in this cluster WITHOUT reachabilities
        let cluster: *mut aas_cluster_t = bot.aasworld.clusters.add(clusternum as usize);
        for i in 0..(*cluster).numportals {
            let portalnum = *bot
                .aasworld
                .portalindex
                .add(((*cluster).firstportal + i) as usize);
            let portal: *mut aas_portal_t = bot.aasworld.portals.add(portalnum as usize);
            if AAS_AreaReachability(bot, (*portal).areanum) != 0 {
                continue;
            }
            if (*portal).frontcluster == clusternum {
                (*portal).clusterareanum[0] = (*cluster).numareas;
                (*cluster).numareas += 1;
            }
            //end if
            else {
                (*portal).clusterareanum[1] = (*cluster).numareas;
                (*cluster).numareas += 1;
            } //end else
        } //end for
    }
}

/// Raven `AAS_CheckAreaForPossiblePortals` — checks whether `areanum` (plus
/// its lesser-presence-type neighbors) forms a valid cluster portal: exactly
/// one shared front plane and one shared back plane, both sides internally
/// connected, no shared edges between the two sides.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:776-896`
pub fn AAS_CheckAreaForPossiblePortals(bot: &mut BotLib, areanum: c_int) -> c_int {
    unsafe {
        let mut areanums = [0 as c_int; MAX_PORTALAREAS as usize];
        let mut numareafrontfaces = [0 as c_int; MAX_PORTALAREAS as usize];
        let mut numareabackfaces = [0 as c_int; MAX_PORTALAREAS as usize];
        let mut frontfacenums = [0 as c_int; MAX_PORTALAREAS as usize];
        let mut backfacenums = [0 as c_int; MAX_PORTALAREAS as usize];
        let mut frontareanums = [0 as c_int; MAX_PORTALAREAS as usize];
        let mut backareanums = [0 as c_int; MAX_PORTALAREAS as usize];

        //if it isn't already a portal
        if (*bot.aasworld.areasettings.add(areanum as usize)).contents & AREACONTENTS_CLUSTERPORTAL
            != 0
        {
            return 0;
        }
        //it must be a grounded area
        if (*bot.aasworld.areasettings.add(areanum as usize)).areaflags & AREA_GROUNDED == 0 {
            return 0;
        }
        //
        Com_Memset(
            numareafrontfaces.as_mut_ptr() as *mut (),
            0,
            core::mem::size_of_val(&numareafrontfaces),
        );
        Com_Memset(
            numareabackfaces.as_mut_ptr() as *mut (),
            0,
            core::mem::size_of_val(&numareabackfaces),
        );
        let mut numfrontfaces: c_int = 0;
        let mut numbackfaces: c_int = 0;
        let mut numfrontareas: c_int = 0;
        let mut numbackareas: c_int = 0;
        let mut frontplanenum: c_int = -1;
        let mut backplanenum: c_int = -1;
        //add any adjacent areas with less presence types
        let numareas =
            AAS_GetAdjacentAreasWithLessPresenceTypes_r(bot, areanums.as_mut_ptr(), 0, areanum);
        //
        for i in 0..numareas {
            let area: *mut aas_area_t = bot.aasworld.areas.add(areanums[i as usize] as usize);
            for j in 0..(*area).numfaces {
                let facenum = (*bot.aasworld.faceindex.add(((*area).firstface + j) as usize)).abs();
                let face: *mut aas_face_t = bot.aasworld.faces.add(facenum as usize);
                //if the face is solid
                if (*face).faceflags & FACE_SOLID != 0 {
                    continue;
                }
                //check if the face is shared with one of the other areas
                let mut k: c_int = 0;
                while k < numareas {
                    if k == i {
                        k += 1;
                        continue;
                    }
                    if (*face).frontarea == areanums[k as usize]
                        || (*face).backarea == areanums[k as usize]
                    {
                        break;
                    }
                    k += 1;
                } //end for
                  //if the face is shared
                if k != numareas {
                    continue;
                }
                //the number of the area at the other side of the face
                let otherareanum = if (*face).frontarea == areanums[i as usize] {
                    (*face).backarea
                } else {
                    (*face).frontarea
                };
                //if the other area already is a cluter portal
                if (*bot.aasworld.areasettings.add(otherareanum as usize)).contents
                    & AREACONTENTS_CLUSTERPORTAL
                    != 0
                {
                    return 0;
                }
                //number of the plane of the area
                let faceplanenum = (*face).planenum & !1;
                //
                if frontplanenum < 0 || faceplanenum == frontplanenum {
                    frontplanenum = faceplanenum;
                    frontfacenums[numfrontfaces as usize] = facenum;
                    numfrontfaces += 1;
                    let mut k2: c_int = 0;
                    while k2 < numfrontareas {
                        if frontareanums[k2 as usize] == otherareanum {
                            break;
                        }
                        k2 += 1;
                    } //end for
                    if k2 == numfrontareas {
                        frontareanums[numfrontareas as usize] = otherareanum;
                        numfrontareas += 1;
                    }
                    numareafrontfaces[i as usize] += 1;
                }
                //end if
                else if backplanenum < 0 || faceplanenum == backplanenum {
                    backplanenum = faceplanenum;
                    backfacenums[numbackfaces as usize] = facenum;
                    numbackfaces += 1;
                    let mut k3: c_int = 0;
                    while k3 < numbackareas {
                        if backareanums[k3 as usize] == otherareanum {
                            break;
                        }
                        k3 += 1;
                    } //end for
                    if k3 == numbackareas {
                        backareanums[numbackareas as usize] = otherareanum;
                        numbackareas += 1;
                    }
                    numareabackfaces[i as usize] += 1;
                }
                //end else
                else {
                    return 0;
                } //end else
            } //end for
        } //end for
          //every area should have at least one front face and one back face
        for i in 0..numareas {
            if numareafrontfaces[i as usize] == 0 || numareabackfaces[i as usize] == 0 {
                return 0;
            }
        } //end for
          //the front areas should all be connected
        if AAS_ConnectedAreas(bot, frontareanums.as_mut_ptr(), numfrontareas) == qfalse {
            return 0;
        }
        //the back areas should all be connected
        if AAS_ConnectedAreas(bot, backareanums.as_mut_ptr(), numbackareas) == qfalse {
            return 0;
        }
        //none of the front faces should have a shared edge with a back face
        let mut i: c_int = 0;
        while i < numfrontfaces {
            let frontface: *mut aas_face_t =
                bot.aasworld.faces.add(frontfacenums[i as usize] as usize);
            let mut fen: c_int = 0;
            while fen < (*frontface).numedges {
                let frontedgenum = (*bot
                    .aasworld
                    .edgeindex
                    .add(((*frontface).firstedge + fen) as usize))
                .abs();
                let mut j: c_int = 0;
                while j < numbackfaces {
                    let backface: *mut aas_face_t =
                        bot.aasworld.faces.add(backfacenums[j as usize] as usize);
                    let mut ben: c_int = 0;
                    while ben < (*backface).numedges {
                        let backedgenum = (*bot
                            .aasworld
                            .edgeindex
                            .add(((*backface).firstedge + ben) as usize))
                        .abs();
                        if frontedgenum == backedgenum {
                            break;
                        }
                        ben += 1;
                    } //end for
                    if ben != (*backface).numedges {
                        break;
                    }
                    j += 1;
                } //end for
                if j != numbackfaces {
                    break;
                }
                fen += 1;
            } //end for
            if fen != (*frontface).numedges {
                break;
            }
            i += 1;
        } //end for
        if i != numfrontfaces {
            return 0;
        }
        //set the cluster portal contents
        for i in 0..numareas {
            let settings = bot.aasworld.areasettings.add(areanums[i as usize] as usize);
            (*settings).contents |= AREACONTENTS_CLUSTERPORTAL;
            //this area can be used as a route portal
            (*settings).contents |= AREACONTENTS_ROUTEPORTAL;
            let __m = std::ffi::CString::new(format!(
                "possible portal: {}\r\n",
                areanums[i as usize]
            ))
            .unwrap_or_default();
            Log_Write(bot, __m.as_ptr() as *mut c_char);
        } //end for
          //
        numareas
    }
}

/// Raven `AAS_FloodClusterAreasUsingReachabilities` — flood-fills remaining
/// unclustered, non-portal areas into `clusternum` by following
/// reachabilities from already-clustered areas.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:233-264`
pub fn AAS_FloodClusterAreasUsingReachabilities(bot: &mut BotLib, clusternum: c_int) -> c_int {
    unsafe {
        let mut i: c_int = 1;
        while i < bot.aasworld.numareas {
            //if this area already has a cluster set
            if (*bot.aasworld.areasettings.add(i as usize)).cluster != 0 {
                i += 1;
                continue;
            }
            //if this area is a cluster portal
            if (*bot.aasworld.areasettings.add(i as usize)).contents & AREACONTENTS_CLUSTERPORTAL
                != 0
            {
                i += 1;
                continue;
            }
            //loop over the reachable areas from this area
            let mut broke = false;
            let numreachableareas = (*bot.aasworld.areasettings.add(i as usize)).numreachableareas;
            for j in 0..numreachableareas {
                //the reachable area
                let firstreachablearea =
                    (*bot.aasworld.areasettings.add(i as usize)).firstreachablearea;
                let areanum = (*bot
                    .aasworld
                    .reachability
                    .add((firstreachablearea + j) as usize))
                .areanum;
                //if this area is a cluster portal
                if (*bot.aasworld.areasettings.add(areanum as usize)).contents
                    & AREACONTENTS_CLUSTERPORTAL
                    != 0
                {
                    continue;
                }
                //if this area has a cluster set
                if (*bot.aasworld.areasettings.add(areanum as usize)).cluster != 0 {
                    if AAS_FloodClusterAreas_r(bot, i, clusternum) == 0 {
                        return qfalse as c_int;
                    }
                    i = 0;
                    broke = true;
                    break;
                } //end if
            } //end for
            let _ = broke;
            i += 1;
        } //end for
        qtrue as c_int
    }
}

/// Raven `AAS_FindPossiblePortals` — scans every area for a possible cluster
/// portal and reports the count found.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:903-913`
pub fn AAS_FindPossiblePortals(bot: &mut BotLib) {
    unsafe {
        let mut numpossibleportals: c_int = 0;
        for i in 1..bot.aasworld.numareas {
            numpossibleportals += AAS_CheckAreaForPossiblePortals(bot, i);
        } //end for
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"\r%6d possible portal areas\n".as_ptr() as *mut c_char,
            numpossibleportals,
        );
    }
}

/// Raven `AAS_FindClusters` — floods every unclustered area into a new
/// cluster, numbering areas and (commented-out) portals as it goes.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:372-415`
pub fn AAS_FindClusters(bot: &mut BotLib) -> c_int {
    unsafe {
        AAS_RemoveClusterAreas(bot);
        //
        for i in 1..bot.aasworld.numareas {
            //if the area is already part of a cluster
            if (*bot.aasworld.areasettings.add(i as usize)).cluster != 0 {
                continue;
            }
            // if not flooding through faces only use areas that have reachabilities
            if bot.nofaceflood != 0
                && (*bot.aasworld.areasettings.add(i as usize)).numreachableareas == 0
            {
                continue;
            } //end if
              //if the area is a cluster portal
            if (*bot.aasworld.areasettings.add(i as usize)).contents & AREACONTENTS_CLUSTERPORTAL
                != 0
            {
                continue;
            } //end if
            if bot.aasworld.numclusters >= AAS_MAX_CLUSTERS {
                AAS_Error(bot, c"AAS_MAX_CLUSTERS".as_ptr() as *mut c_char);
                return qfalse as c_int;
            } //end if
            let cluster: *mut aas_cluster_t =
                bot.aasworld.clusters.add(bot.aasworld.numclusters as usize);
            (*cluster).numareas = 0;
            (*cluster).numreachabilityareas = 0;
            (*cluster).firstportal = bot.aasworld.portalindexsize;
            (*cluster).numportals = 0;
            //flood the areas in this cluster
            if AAS_FloodClusterAreas_r(bot, i, bot.aasworld.numclusters) == 0 {
                return qfalse as c_int;
            }
            if AAS_FloodClusterAreasUsingReachabilities(bot, bot.aasworld.numclusters) == 0 {
                return qfalse as c_int;
            }
            //number the cluster areas
            //AAS_NumberClusterPortals(aasworld.numclusters);
            AAS_NumberClusterAreas(bot, bot.aasworld.numclusters);
            //Log_Write("cluster %d has %d areas\r\n", aasworld.numclusters, cluster->numareas);
            bot.aasworld.numclusters += 1;
        } //end for
        qtrue as c_int
    }
}

/// Raven `AAS_InitClustering` — top-level entry: builds view/forced
/// portals, retries portal+cluster detection until stable, allocates the
/// portal/portal-index/cluster arenas, and reports statistics.
///
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:1441-1528`
pub fn AAS_InitClustering(bot: &mut BotLib) {
    unsafe {
        if bot.aasworld.loaded == 0 {
            return;
        }
        //if there are clusters
        if bot.aasworld.numclusters >= 1 {
            // PORT-NOTE(BSPC): Raven's `#ifndef BSPC` early-out reads the
            // "forceclustering"/"forcereachability" libvars; BSPC is not
            // built here, so the check always applies.
            let force_clustering =
                LibVarGetValue(bot, c"forceclustering".as_ptr() as *mut c_char) as c_int;
            let force_reachability =
                LibVarGetValue(bot, c"forcereachability".as_ptr() as *mut c_char) as c_int;
            if force_clustering == 0 && force_reachability == 0 {
                return;
            }
        } //end if
          //set all view portals as cluster portals in case we re-calculate the reachabilities and clusters (with -reach)
        AAS_SetViewPortalsAsClusterPortals(bot);
        //count the number of forced cluster portals
        AAS_CountForcedClusterPortals(bot);
        //remove all area cluster marks
        AAS_RemoveClusterAreas(bot);
        //find possible cluster portals
        AAS_FindPossiblePortals(bot);
        //craete portals to for the bot view
        AAS_CreateViewPortals(bot);
        //remove all portals that are not closing a cluster
        //AAS_RemoveNotClusterClosingPortals();
        //initialize portal memory
        if !bot.aasworld.portals.is_null() {
            FreeMemory(bot, bot.aasworld.portals as *mut ());
        }
        bot.aasworld.portals = GetClearedMemory(
            bot,
            (AAS_MAX_PORTALS as core::ffi::c_ulong)
                * (core::mem::size_of::<aas_portal_t>() as core::ffi::c_ulong),
        ) as *mut aas_portal_t;
        //initialize portal index memory
        if !bot.aasworld.portalindex.is_null() {
            FreeMemory(bot, bot.aasworld.portalindex as *mut ());
        }
        bot.aasworld.portalindex = GetClearedMemory(
            bot,
            (AAS_MAX_PORTALINDEXSIZE as core::ffi::c_ulong)
                * (core::mem::size_of::<crate::aasfile::aas_portalindex_t::aas_portalindex_t>()
                    as core::ffi::c_ulong),
        )
            as *mut crate::aasfile::aas_portalindex_t::aas_portalindex_t;
        //initialize cluster memory
        if !bot.aasworld.clusters.is_null() {
            FreeMemory(bot, bot.aasworld.clusters as *mut ());
        }
        bot.aasworld.clusters = GetClearedMemory(
            bot,
            (AAS_MAX_CLUSTERS as core::ffi::c_ulong)
                * (core::mem::size_of::<aas_cluster_t>() as core::ffi::c_ulong),
        ) as *mut aas_cluster_t;
        //
        let mut removedPortalAreas: c_int = 0;
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"\r%6d removed portal areas".as_ptr() as *mut c_char,
            removedPortalAreas,
        );
        loop {
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"\r%6d".as_ptr() as *mut c_char,
                removedPortalAreas,
            );
            //initialize the number of portals and clusters
            bot.aasworld.numportals = 1; //portal 0 is a dummy
            bot.aasworld.portalindexsize = 0;
            bot.aasworld.numclusters = 1; //cluster 0 is a dummy
                                          //create the portals from the portal areas
            AAS_CreatePortals(bot);
            //
            removedPortalAreas += 1;
            //find the clusters
            if AAS_FindClusters(bot) == 0 {
                continue;
            }
            //test the portals
            if AAS_TestPortals(bot) == 0 {
                continue;
            }
            //
            break;
        } //end while
        bot.botimport.Print.unwrap()(PRT_MESSAGE, c"\n".as_ptr() as *mut c_char);
        //the AAS file should be saved
        bot.aasworld.savefile = qtrue as c_int;
        //write the portal areas to the log file
        for i in 1..bot.aasworld.numportals {
            let __m = std::ffi::CString::new(format!(
                "portal {}: area {}\r\n",
                i,
                (*bot.aasworld.portals.add(i as usize)).areanum
            ))
            .unwrap_or_default();
            Log_Write(bot, __m.as_ptr() as *mut c_char);
        } //end for
          // report cluster info
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"%6d portals created\n".as_ptr() as *mut c_char,
            bot.aasworld.numportals,
        );
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"%6d clusters created\n".as_ptr() as *mut c_char,
            bot.aasworld.numclusters,
        );
        for i in 1..bot.aasworld.numclusters {
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"cluster %d has %d reachability areas\n".as_ptr() as *mut c_char,
                i,
                (*bot.aasworld.clusters.add(i as usize)).numreachabilityareas,
            );
        } //end for
          // report AAS file efficiency
        let mut numreachabilityareas: c_int = 0;
        let mut total: c_int = 0;
        for i in 0..bot.aasworld.numclusters {
            let n = (*bot.aasworld.clusters.add(i as usize)).numreachabilityareas;
            numreachabilityareas += n;
            total += n * n;
        }
        total += numreachabilityareas * bot.aasworld.numportals;
        //
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"%6i total reachability areas\n".as_ptr() as *mut c_char,
            numreachabilityareas,
        );
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"%6i AAS memory/CPU usage (the lower the better)\n".as_ptr() as *mut c_char,
            total * 3,
        );
    }
}
