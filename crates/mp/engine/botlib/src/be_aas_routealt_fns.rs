#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_assignments
)]

//! Function bodies for Raven's `be_aas_routealt.cpp` (alternate-route
//! computation: mid-range area flooding and alternate route-goal picking).
//!
//! Ported per the engine C-track packets (`botlib__0487`, `botlib__1078`,
//! `botlib__1079`, `botlib__2172`).
//! Source: `oracle/codemp/botlib/be_aas_routealt.cpp`.
//!
//! DESTINATION NOTE: the packet order named
//! `crates/mp/engine/botlib/src/be_aas_routealt.rs`, but `be_aas_routealt`
//! already exists as a directory module (`be_aas_routealt/mod.rs`,
//! constants-only) — `_fns` escape per `_PREAMBLE.md`'s destination rule.
//!
//! PORT-NOTE(unsafe): the AAS arena is a graph of raw pointers
//! (`aasworld.*`, `bot.midrangeareas`, `bot.clusterareas`); bodies deref
//! explicitly inside `unsafe` per porting-rules §D11, matching the sibling
//! `be_aas_reach_fns.rs`/`be_aas_cluster_fns.rs`/`be_aas_route_fns.rs`
//! convention.
//!
//! PORT-NOTE(ENABLE_ALTROUTING): `crate::be_aas_routealt::be_aas_routealt_cpp_consts::ENABLE_ALTROUTING`
//! is already ported as `true` (Raven defines it unconditionally at this
//! site) — the `#ifdef ENABLE_ALTROUTING` bodies below therefore always
//! execute, matching the shipped build.

use core::ffi::c_int;

use mp_qshared::shared::{qfalse, qtrue, vec3_t};

use crate::aasfile::area_contents::{AREACONTENTS_CLUSTERPORTAL, AREACONTENTS_VIEWPORTAL};
use crate::be_aas_routealt::be_aas_routealt_cpp_consts::ENABLE_ALTROUTING;
use crate::BotLib;

use mp_qshared::common::mp::botlib::aas_altroutegoal_flags::{
    ALTROUTEGOAL_ALL, ALTROUTEGOAL_CLUSTERPORTALS, ALTROUTEGOAL_VIEWPORTALS,
};
use mp_qshared::common::mp::botlib::aas_altroutegoal_s::aas_altroutegoal_t;

use crate::be_aas_reach_fns::AAS_AreaReachability;
use crate::be_aas_route_fns::AAS_AreaTravelTimeToGoalArea;
use crate::l_log_fns::Log_Write;
use crate::l_memory_fns::{FreeMemory, GetMemory};
use mp_engine_qcommon::common_fns::Com_Memset;

/// Raven `AAS_AltRoutingFloodCluster_r`.
///
/// Recursively floods through faces to gather every area of the current
/// mid-range-area cluster into `bot.clusterareas`.
///
/// Source: `oracle/codemp/botlib/be_aas_routealt.cpp:49-75`
pub fn AAS_AltRoutingFloodCluster_r(bot: &mut BotLib, areanum: c_int) {
    unsafe {
        // add the current area to the areas of the current cluster
        *bot.clusterareas.add(bot.numclusterareas as usize) = areanum;
        bot.numclusterareas += 1;
        // remove the area from the mid range areas
        (*bot.midrangeareas.add(areanum as usize)).valid = qfalse;
        // flood to other areas through the faces of this area
        let area = bot.aasworld.areas.add(areanum as usize);
        for i in 0..(*area).numfaces {
            let face = bot.aasworld.faces.add(
                (*bot.aasworld.faceindex.add(((*area).firstface + i) as usize)).unsigned_abs()
                    as usize,
            );
            // get the area at the other side of the face
            let otherareanum = if (*face).frontarea == areanum {
                (*face).backarea
            } else {
                (*face).frontarea
            };
            // if there is an area at the other side of this face
            if otherareanum == 0 {
                continue;
            }
            // if the other area is not a midrange area
            if (*bot.midrangeareas.add(otherareanum as usize)).valid == qfalse {
                continue;
            }
            AAS_AltRoutingFloodCluster_r(bot, otherareanum);
        }
    }
}

/// Raven `AAS_AlternativeRouteGoals`.
///
/// Source: `oracle/codemp/botlib/be_aas_routealt.cpp:82-192`
#[allow(clippy::too_many_arguments)]
pub fn AAS_AlternativeRouteGoals(
    bot: &mut BotLib,
    start: vec3_t,
    startareanum: c_int,
    goal: vec3_t,
    goalareanum: c_int,
    travelflags: c_int,
    altroutegoals: *mut aas_altroutegoal_t,
    maxaltroutegoals: c_int,
    r#type: c_int,
) -> c_int {
    if !ENABLE_ALTROUTING {
        return 0;
    }
    unsafe {
        if startareanum == 0 || goalareanum == 0 {
            return 0;
        }
        // travel time towards the goal area
        let goaltraveltime =
            AAS_AreaTravelTimeToGoalArea(bot, startareanum, start, goalareanum, travelflags);
        // clear the midrange areas
        Com_Memset(
            bot.midrangeareas as *mut (),
            0,
            bot.aasworld.numareas as usize * core::mem::size_of_val(&*bot.midrangeareas.add(0)),
        );
        let mut numaltroutegoals: c_int = 0;
        let mut nummidrangeareas: c_int = 0;

        for i in 1..bot.aasworld.numareas {
            if r#type & ALTROUTEGOAL_ALL == 0 {
                let has_clusterportal = r#type & ALTROUTEGOAL_CLUSTERPORTALS != 0
                    && (*bot.aasworld.areasettings.add(i as usize)).contents
                        & AREACONTENTS_CLUSTERPORTAL
                        != 0;
                if !has_clusterportal {
                    let has_viewportal = r#type & ALTROUTEGOAL_VIEWPORTALS != 0
                        && (*bot.aasworld.areasettings.add(i as usize)).contents
                            & AREACONTENTS_VIEWPORTAL
                            != 0;
                    if !has_viewportal {
                        continue;
                    }
                }
            }
            // if the area has no reachabilities
            if AAS_AreaReachability(bot, i) == 0 {
                continue;
            }
            // travel time from the area to the start area
            let starttime = AAS_AreaTravelTimeToGoalArea(bot, startareanum, start, i, travelflags);
            if starttime == 0 {
                continue;
            }
            // if the travel time from the start to the area is greater than the shortest goal travel time
            if starttime as f32 > 1.1 * goaltraveltime as f32 {
                continue;
            }
            // travel time from the area to the goal area
            // PORT-NOTE(null-origin): Raven passes NULL for `origin` here (only used
            // when the from-area equals the current area, which never holds for `i`);
            // the resolved signature takes `vec3_t` by value, so a zeroed vec3 is
            // passed — matches the callee never reading it on this path.
            let goaltime =
                AAS_AreaTravelTimeToGoalArea(bot, i, [0.0, 0.0, 0.0], goalareanum, travelflags);
            if goaltime == 0 {
                continue;
            }
            // if the travel time from the area to the goal is greater than the shortest goal travel time
            if goaltime as f32 > 0.8 * goaltraveltime as f32 {
                continue;
            }
            // this is a mid range area
            let mra = bot.midrangeareas.add(i as usize);
            (*mra).valid = qtrue;
            (*mra).starttime = starttime as u16;
            (*mra).goaltime = goaltime as u16;
            let __m = std::ffi::CString::new(format!("{} midrange area {}", nummidrangeareas, i))
                .unwrap_or_default();
            Log_Write(bot, __m.as_ptr() as *mut core::ffi::c_char);
            nummidrangeareas += 1;
        }

        for i in 1..bot.aasworld.numareas {
            if (*bot.midrangeareas.add(i as usize)).valid == qfalse {
                continue;
            }
            // get the areas in one cluster
            bot.numclusterareas = 0;
            AAS_AltRoutingFloodCluster_r(bot, i);
            // now we've got a cluster with areas through which an alternative route could go
            // get the 'center' of the cluster
            let mut mid: vec3_t = [0.0, 0.0, 0.0];
            for j in 0..bot.numclusterareas {
                let carea = *bot.clusterareas.add(j as usize);
                let center = (*bot.aasworld.areas.add(carea as usize)).center;
                mid[0] += center[0];
                mid[1] += center[1];
                mid[2] += center[2];
            }
            let inv = 1.0 / bot.numclusterareas as f32;
            mid = [mid[0] * inv, mid[1] * inv, mid[2] * inv];
            // get the area closest to the center of the cluster
            let mut bestdist: f32 = 999999.0;
            let mut bestareanum: c_int = 0;
            for j in 0..bot.numclusterareas {
                let carea = *bot.clusterareas.add(j as usize);
                let center = (*bot.aasworld.areas.add(carea as usize)).center;
                let dir = [mid[0] - center[0], mid[1] - center[1], mid[2] - center[2]];
                let dist = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
                if dist < bestdist {
                    bestdist = dist;
                    bestareanum = carea;
                }
            }
            // now we've got an area for an alternative route
            // FIXME: add alternative goal origin
            let goal = &mut *altroutegoals.add(numaltroutegoals as usize);
            goal.origin = (*bot.aasworld.areas.add(bestareanum as usize)).center;
            goal.areanum = bestareanum;
            let mra = bot.midrangeareas.add(bestareanum as usize);
            goal.starttraveltime = (*mra).starttime;
            goal.goaltraveltime = (*mra).goaltime;
            goal.extratraveltime =
                (((*mra).starttime as c_int + (*mra).goaltime as c_int) - goaltraveltime) as u16;
            numaltroutegoals += 1;
            // don't return more than the maximum alternative route goals
            if numaltroutegoals >= maxaltroutegoals {
                break;
            }
        }
        numaltroutegoals
    }
}

/// Raven `AAS_InitAlternativeRouting`.
///
/// Source: `oracle/codemp/botlib/be_aas_routealt.cpp:199-207`
pub fn AAS_InitAlternativeRouting(bot: &mut BotLib) {
    if !ENABLE_ALTROUTING {
        return;
    }
    unsafe {
        if !bot.midrangeareas.is_null() {
            FreeMemory(bot, bot.midrangeareas as *mut ());
        }
        bot.midrangeareas = GetMemory(
            bot,
            (bot.aasworld.numareas
                * core::mem::size_of::<crate::be_aas_routealt::midrangearea_t>() as c_int)
                as u64,
        ) as *mut crate::be_aas_routealt::midrangearea_t;
        if !bot.clusterareas.is_null() {
            FreeMemory(bot, bot.clusterareas as *mut ());
        }
        bot.clusterareas = GetMemory(
            bot,
            (bot.aasworld.numareas * core::mem::size_of::<c_int>() as c_int) as u64,
        ) as *mut c_int;
    }
}

/// Raven `AAS_ShutdownAlternativeRouting`.
///
/// Source: `oracle/codemp/botlib/be_aas_routealt.cpp:214-223`
pub fn AAS_ShutdownAlternativeRouting(bot: &mut BotLib) {
    if !ENABLE_ALTROUTING {
        return;
    }
    unsafe {
        if !bot.midrangeareas.is_null() {
            FreeMemory(bot, bot.midrangeareas as *mut ());
        }
        bot.midrangeareas = core::ptr::null_mut();
        if !bot.clusterareas.is_null() {
            FreeMemory(bot, bot.clusterareas as *mut ());
        }
        bot.clusterareas = core::ptr::null_mut();
        bot.numclusterareas = 0;
    }
}
