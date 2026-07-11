#![allow(non_snake_case, non_camel_case_types)]

//! MP botlib `be_aas_route.cpp` — AAS routing-cache logic (route table
//! computation, portal/area routing caches, route prediction).
//!
//! DESTINATION NOTE: the packet order named `crates/mp/engine/botlib/src/be_aas_route.rs`,
//! but `be_aas_route` already exists as a directory module (`be_aas_route/mod.rs`,
//! constants-only) — `.rs` + `/mod.rs` for the same module name cannot coexist,
//! so this file lands at the `_fns` escape per `_PREAMBLE.md`'s destination rule.

use std::os::raw::{c_int, c_ulong};

use mp_qshared::common::mp::botlib::aas_predictroute_s::aas_predictroute_s;
use mp_qshared::common::mp::botlib::aas_route_stop_event::{
    RSE_ENTERAREA, RSE_ENTERCONTENTS, RSE_NONE, RSE_NOROUTE, RSE_USETRAVELTYPE,
};
use mp_qshared::common::mp::botlib::aas_trace_s::aas_trace_t;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_FATAL, PRT_MESSAGE, PRT_WARNING};
use mp_qshared::common::mp::botlib::travel_flags::{
    TFL_AIR, TFL_BARRIERJUMP, TFL_BFGJUMP, TFL_BRIDGE, TFL_CROUCH, TFL_DEFAULT, TFL_DONOTENTER,
    TFL_DOUBLEJUMP, TFL_ELEVATOR, TFL_FUNCBOB, TFL_GRAPPLEHOOK, TFL_INVALID, TFL_JUMP, TFL_JUMPPAD,
    TFL_LADDER, TFL_LAVA, TFL_NOTTEAM1, TFL_NOTTEAM2, TFL_RAMPJUMP, TFL_ROCKETJUMP, TFL_SLIME,
    TFL_STRAFEJUMP, TFL_SWIM, TFL_TELEPORT, TFL_WALK, TFL_WALKOFFLEDGE, TFL_WATER, TFL_WATERJUMP,
};
use mp_qshared::shared::file_mode::{FS_READ, FS_WRITE};
use mp_qshared::shared::{fileHandle_t, qboolean, qfalse, qtrue, vec3_t, MAX_QPATH};

use crate::aasfile::aas_area_s::aas_area_t;
use crate::aasfile::aas_areasettings_s::aas_areasettings_t;
use crate::aasfile::aas_cluster_s::aas_cluster_t;
use crate::aasfile::aas_portal_s::aas_portal_t;
use crate::aasfile::aas_reachability_s::{aas_reachability_s, aas_reachability_t};
use crate::aasfile::area_contents::{
    AREACONTENTS_DONOTENTER, AREACONTENTS_LAVA, AREACONTENTS_NOTTEAM1, AREACONTENTS_NOTTEAM2,
    AREACONTENTS_SLIME, AREACONTENTS_WATER,
};
use crate::aasfile::area_flags::{AREA_BRIDGE, AREA_DISABLED};
use crate::aasfile::presence_type::PRESENCE_CROUCH;
use crate::aasfile::travel_type::{
    MAX_TRAVELTYPES, TRAVELFLAG_NOTTEAM1, TRAVELFLAG_NOTTEAM2, TRAVELTYPE_MASK, TRAVEL_BARRIERJUMP,
    TRAVEL_BFGJUMP, TRAVEL_CROUCH, TRAVEL_DOUBLEJUMP, TRAVEL_ELEVATOR, TRAVEL_FUNCBOB,
    TRAVEL_GRAPPLEHOOK, TRAVEL_INVALID, TRAVEL_JUMP, TRAVEL_JUMPPAD, TRAVEL_LADDER,
    TRAVEL_RAMPJUMP, TRAVEL_ROCKETJUMP, TRAVEL_STRAFEJUMP, TRAVEL_SWIM, TRAVEL_TELEPORT,
    TRAVEL_WALK, TRAVEL_WALKOFFLEDGE, TRAVEL_WATERJUMP,
};
use crate::be_aas_def::aas_reachabilityareas_s::aas_reachabilityareas_t;
use crate::be_aas_def::aas_reversedlink_s::aas_reversedlink_t;
use crate::be_aas_def::aas_reversedreachability_s::aas_reversedreachability_t;
use crate::be_aas_def::aas_routingcache_s::{aas_routingcache_t, CACHETYPE_AREA, CACHETYPE_PORTAL};
use crate::be_aas_def::aas_routingupdate_s::aas_routingupdate_t;
use crate::BotLib;
use mp_engine_qcommon::common::Common;

// The `bot: &mut BotLib` / `common: &mut Common` receivers named in every
// signature below are the campaign's threaded-state aggregates (ruling 2).
// Every reference to `aasworld`/`botimport`/`bot_developer`/
// `numareacacheupdates`/`numportalcacheupdates`/`routingcachesize`/
// `max_routingcachesize` below is the exact Raven global name per house rule,
// reached as a field on `bot` (or `common` where the packet says so).

/// Raven `AAS_RoutingInfo` — prints routing cache statistics.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:77-82`
pub fn AAS_RoutingInfo(bot: &mut BotLib) {
    bot.botimport.Print(
        PRT_MESSAGE,
        &format!("{} area cache updates\n", bot.numareacacheupdates),
    );
    bot.botimport.Print(
        PRT_MESSAGE,
        &format!("{} portal cache updates\n", bot.numportalcacheupdates),
    );
    bot.botimport.Print(
        PRT_MESSAGE,
        &format!("{} bytes routing cache\n", bot.routingcachesize),
    );
}

/// Raven `AAS_ClusterAreaNum` — the area's number within its cluster.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:92-111`
pub fn AAS_ClusterAreaNum(bot: &mut BotLib, cluster: c_int, areanum: c_int) -> c_int {
    let areacluster = bot.aasworld.areasettings[areanum as usize].cluster;
    if areacluster > 0 {
        bot.aasworld.areasettings[areanum as usize].clusterareanum
    } else {
        let side = (bot.aasworld.portals[(-areacluster) as usize].frontcluster != cluster) as c_int;
        bot.aasworld.portals[(-areacluster) as usize].clusterareanum[side as usize]
    }
}

/// Raven `AAS_InitTravelFlagFromType` — builds the travel-type -> travel-flag table.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:118-145`
pub fn AAS_InitTravelFlagFromType(bot: &mut BotLib) {
    for i in 0..MAX_TRAVELTYPES {
        bot.aasworld.travelflagfortype[i as usize] = TFL_INVALID;
    }
    bot.aasworld.travelflagfortype[TRAVEL_INVALID as usize] = TFL_INVALID;
    bot.aasworld.travelflagfortype[TRAVEL_WALK as usize] = TFL_WALK;
    bot.aasworld.travelflagfortype[TRAVEL_CROUCH as usize] = TFL_CROUCH;
    bot.aasworld.travelflagfortype[TRAVEL_BARRIERJUMP as usize] = TFL_BARRIERJUMP;
    bot.aasworld.travelflagfortype[TRAVEL_JUMP as usize] = TFL_JUMP;
    bot.aasworld.travelflagfortype[TRAVEL_LADDER as usize] = TFL_LADDER;
    bot.aasworld.travelflagfortype[TRAVEL_WALKOFFLEDGE as usize] = TFL_WALKOFFLEDGE;
    bot.aasworld.travelflagfortype[TRAVEL_SWIM as usize] = TFL_SWIM;
    bot.aasworld.travelflagfortype[TRAVEL_WATERJUMP as usize] = TFL_WATERJUMP;
    bot.aasworld.travelflagfortype[TRAVEL_TELEPORT as usize] = TFL_TELEPORT;
    bot.aasworld.travelflagfortype[TRAVEL_ELEVATOR as usize] = TFL_ELEVATOR;
    bot.aasworld.travelflagfortype[TRAVEL_ROCKETJUMP as usize] = TFL_ROCKETJUMP;
    bot.aasworld.travelflagfortype[TRAVEL_BFGJUMP as usize] = TFL_BFGJUMP;
    bot.aasworld.travelflagfortype[TRAVEL_GRAPPLEHOOK as usize] = TFL_GRAPPLEHOOK;
    bot.aasworld.travelflagfortype[TRAVEL_DOUBLEJUMP as usize] = TFL_DOUBLEJUMP;
    bot.aasworld.travelflagfortype[TRAVEL_RAMPJUMP as usize] = TFL_RAMPJUMP;
    bot.aasworld.travelflagfortype[TRAVEL_STRAFEJUMP as usize] = TFL_STRAFEJUMP;
    bot.aasworld.travelflagfortype[TRAVEL_JUMPPAD as usize] = TFL_JUMPPAD;
    bot.aasworld.travelflagfortype[TRAVEL_FUNCBOB as usize] = TFL_FUNCBOB;
}

/// Raven `AAS_TravelFlagForType_inline`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:152-166`
pub fn AAS_TravelFlagForType_inline(bot: &mut BotLib, traveltype: c_int) -> c_int {
    let mut tfl: c_int = 0;
    if tfl & TRAVELFLAG_NOTTEAM1 != 0 {
        tfl |= TFL_NOTTEAM1;
    }
    if tfl & TRAVELFLAG_NOTTEAM2 != 0 {
        tfl |= TFL_NOTTEAM2;
    }
    let traveltype = traveltype & TRAVELTYPE_MASK;
    if traveltype < 0 || traveltype >= MAX_TRAVELTYPES {
        return TFL_INVALID;
    }
    tfl |= bot.aasworld.travelflagfortype[traveltype as usize];
    tfl
}

/// Raven `AAS_UnlinkCache` — unlinks a routing cache from the LRU list.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:183-191`
pub fn AAS_UnlinkCache(bot: &mut BotLib, cache: *mut aas_routingcache_t) {
    unsafe {
        if !(*cache).time_next.is_null() {
            (*(*cache).time_next).time_prev = (*cache).time_prev;
        } else {
            bot.aasworld.newestcache = (*cache).time_prev;
        }
        if !(*cache).time_prev.is_null() {
            (*(*cache).time_prev).time_next = (*cache).time_next;
        } else {
            bot.aasworld.oldestcache = (*cache).time_next;
        }
        (*cache).time_next = std::ptr::null_mut();
        (*cache).time_prev = std::ptr::null_mut();
    }
}

/// Raven `AAS_LinkCache` — links a routing cache into the LRU list.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:198-212`
pub fn AAS_LinkCache(bot: &mut BotLib, cache: *mut aas_routingcache_t) {
    unsafe {
        if !bot.aasworld.newestcache.is_null() {
            (*bot.aasworld.newestcache).time_next = cache;
            (*cache).time_prev = bot.aasworld.newestcache;
        } else {
            bot.aasworld.oldestcache = cache;
            (*cache).time_prev = std::ptr::null_mut();
        }
        (*cache).time_next = std::ptr::null_mut();
        bot.aasworld.newestcache = cache;
    }
}

/// Raven `AAS_GetAreaContentsTravelFlags`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:335-358`
pub fn AAS_GetAreaContentsTravelFlags(bot: &mut BotLib, areanum: c_int) -> c_int {
    let contents = bot.aasworld.areasettings[areanum as usize].contents;
    let mut tfl: c_int = 0;
    if contents & AREACONTENTS_WATER != 0 {
        tfl |= TFL_WATER;
    } else if contents & AREACONTENTS_SLIME != 0 {
        tfl |= TFL_SLIME;
    } else if contents & AREACONTENTS_LAVA != 0 {
        tfl |= TFL_LAVA;
    } else {
        tfl |= TFL_AIR;
    }
    if contents & AREACONTENTS_DONOTENTER != 0 {
        tfl |= TFL_DONOTENTER;
    }
    if contents & AREACONTENTS_NOTTEAM1 != 0 {
        tfl |= TFL_NOTTEAM1;
    }
    if contents & AREACONTENTS_NOTTEAM2 != 0 {
        tfl |= TFL_NOTTEAM2;
    }
    if bot.aasworld.areasettings[areanum as usize].areaflags & AREA_BRIDGE != 0 {
        tfl |= TFL_BRIDGE;
    }
    tfl
}

/// Raven `AAS_AreaContentsTravelFlags_inline`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:365-368`
pub fn AAS_AreaContentsTravelFlags_inline(bot: &mut BotLib, areanum: c_int) -> c_int {
    bot.aasworld.areacontentstravelflags[areanum as usize]
}

/// Raven `AAS_AreaContentsTravelFlags`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:375-378`
pub fn AAS_AreaContentsTravelFlags(bot: &mut BotLib, areanum: c_int) -> c_int {
    bot.aasworld.areacontentstravelflags[areanum as usize]
}

/// Raven `AAS_PortalMaxTravelTime`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:549-576`
pub fn AAS_PortalMaxTravelTime(bot: &mut BotLib, portalnum: c_int) -> c_int {
    let portal: *const aas_portal_t = &bot.aasworld.portals[portalnum as usize];
    unsafe {
        let revreach: *const aas_reversedreachability_t =
            &bot.aasworld.reversedreachability[(*portal).areanum as usize];
        let settings: *const aas_areasettings_t =
            &bot.aasworld.areasettings[(*portal).areanum as usize];
        let mut maxt: c_int = 0;
        for _l in 0..(*settings).numreachableareas {
            let mut n = 0usize;
            let mut revlink = (*revreach).first;
            while !revlink.is_null() {
                let t = bot.aasworld.areatraveltimes[(*portal).areanum as usize][_l as usize][n]
                    as c_int;
                if t > maxt {
                    maxt = t;
                }
                revlink = (*revlink).next;
                n += 1;
            }
        }
        maxt
    }
}

/// Raven `AAS_BridgeWalkable` — always false; bridges are not (yet) walkable.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1892-1895`
pub fn AAS_BridgeWalkable(_areanum: c_int) -> c_int {
    qfalse
}

/// Raven `AAS_TravelFlagForType`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:173-176`
pub fn AAS_TravelFlagForType(bot: &mut BotLib, traveltype: c_int) -> c_int {
    AAS_TravelFlagForType_inline(bot, traveltype)
}

/// Raven `AAS_FreeRoutingCache` — unlinks + frees a routing cache block.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:219-224`
pub fn AAS_FreeRoutingCache(bot: &mut BotLib, cache: *mut aas_routingcache_t) {
    AAS_UnlinkCache(bot, cache);
    unsafe {
        bot.routingcachesize -= (*cache).size;
    }
    crate::l_memory_fns::FreeMemory(bot, cache as *mut ());
}

/// Raven `AAS_RoutingTime`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:325-328`
pub fn AAS_RoutingTime(bot: &mut BotLib) -> f32 {
    crate::be_aas_main::AAS_Time(bot)
}

/// Raven `AAS_AreaTravelTime`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:457-476`
pub fn AAS_AreaTravelTime(bot: &mut BotLib, areanum: c_int, start: vec3_t, end: vec3_t) -> u16 {
    let dir = [start[0] - end[0], start[1] - end[1], start[2] - end[2]];
    let mut dist = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    if crate::be_aas_reach_fns::AAS_AreaCrouch(bot, areanum) != 0 {
        dist *= crate::be_aas_route::DISTANCEFACTOR_CROUCH;
    } else if crate::be_aas_reach_fns::AAS_AreaSwim(bot, areanum) != 0 {
        dist *= crate::be_aas_route::DISTANCEFACTOR_SWIM;
    } else {
        dist *= crate::be_aas_route::DISTANCEFACTOR_WALK;
    }
    let mut intdist = dist as c_int;
    if intdist <= 0 {
        intdist = 1;
    }
    intdist as u16
}

/// Raven `AAS_ReadCache` — reads one routing-cache block from a file.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1014-1026`
pub fn AAS_ReadCache(bot: &mut BotLib, fp: fileHandle_t) -> *mut aas_routingcache_t {
    unsafe {
        let mut size: c_int = 0;
        bot.botimport.FS_Read(
            &mut size as *mut c_int as *mut std::ffi::c_void,
            std::mem::size_of::<c_int>() as c_int,
            fp,
        );
        let cache =
            crate::l_memory_fns::GetMemory(bot, size as c_ulong) as *mut aas_routingcache_t;
        (*cache).size = size;
        let size_field = std::mem::size_of::<c_int>() as c_int;
        bot.botimport.FS_Read(
            (cache as *mut u8).add(size_field as usize) as *mut std::ffi::c_void,
            size - size_field,
            fp,
        );
        let base = std::mem::size_of::<aas_routingcache_t>() as isize
            - std::mem::size_of::<u16>() as isize;
        let extra = (size as isize - std::mem::size_of::<aas_routingcache_t>() as isize
            + std::mem::size_of::<u16>() as isize)
            / 3
            * 2;
        (*cache).reachabilities = (cache as *mut u8).offset(base + extra);
        cache
    }
}

/// Raven `AAS_UpdateAreaRoutingCache` — Dijkstra-like area routing-cache fill.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1271-1375`
pub fn AAS_UpdateAreaRoutingCache(bot: &mut BotLib, areacache: *mut aas_routingcache_t) {
    unsafe {
        // §19: `startareatraveltimes` is a local array Raven reads via the
        // `curupdate->areatraveltimes` pointer before any of it is written by
        // this function; zero-init to match the `Com_Memset` immediately below.
        let mut startareatraveltimes: [u16; 128] = [0; 128];

        let numreachabilityareas =
            bot.aasworld.clusters[(*areacache).cluster as usize].numreachabilityareas;
        bot.aasworld.frameroutingupdates += 1;

        let badtravelflags = !(*areacache).travelflags;

        let mut clusterareanum =
            AAS_ClusterAreaNum(bot, (*areacache).cluster, (*areacache).areanum);
        if clusterareanum >= numreachabilityareas {
            return;
        }

        let curupdate: *mut aas_routingupdate_t =
            &mut bot.aasworld.areaupdate[clusterareanum as usize];
        (*curupdate).areanum = (*areacache).areanum;
        (*curupdate).areatraveltimes = startareatraveltimes.as_mut_ptr();
        (*curupdate).tmptraveltime = (*areacache).starttraveltime;

        *(*areacache)
            .traveltimes
            .as_mut_ptr()
            .add(clusterareanum as usize) = (*areacache).starttraveltime;

        (*curupdate).next = std::ptr::null_mut();
        (*curupdate).prev = std::ptr::null_mut();
        let mut updateliststart = curupdate;
        let mut updatelistend = curupdate;

        while !updateliststart.is_null() {
            let curupdate = updateliststart;

            if !(*curupdate).next.is_null() {
                (*(*curupdate).next).prev = std::ptr::null_mut();
            } else {
                updatelistend = std::ptr::null_mut();
            }
            updateliststart = (*curupdate).next;

            (*curupdate).inlist = qfalse;

            let revreach: *const aas_reversedreachability_t =
                &bot.aasworld.reversedreachability[(*curupdate).areanum as usize];
            let mut i = 0usize;
            let mut revlink = (*revreach).first;
            while !revlink.is_null() {
                let linknum = (*revlink).linknum;
                let reach: *const aas_reachability_t = &bot.aasworld.reachability[linknum as usize];

                if AAS_TravelFlagForType_inline(bot, (*reach).traveltype) & badtravelflags != 0 {
                    revlink = (*revlink).next;
                    i += 1;
                    continue;
                }
                if bot.aasworld.areasettings[(*reach).areanum as usize].areaflags & AREA_DISABLED
                    != 0
                {
                    revlink = (*revlink).next;
                    i += 1;
                    continue;
                }
                if AAS_AreaContentsTravelFlags_inline(bot, (*reach).areanum) & badtravelflags != 0 {
                    revlink = (*revlink).next;
                    i += 1;
                    continue;
                }
                let nextareanum = (*revlink).areanum;
                let cluster = bot.aasworld.areasettings[nextareanum as usize].cluster;
                if cluster > 0 && cluster != (*areacache).cluster {
                    revlink = (*revlink).next;
                    i += 1;
                    continue;
                }
                clusterareanum = AAS_ClusterAreaNum(bot, (*areacache).cluster, nextareanum);
                if clusterareanum >= numreachabilityareas {
                    revlink = (*revlink).next;
                    i += 1;
                    continue;
                }

                let t = (*curupdate).tmptraveltime
                    + *(*curupdate).areatraveltimes.add(i)
                    + (*reach).traveltime;

                let existing = *(*areacache)
                    .traveltimes
                    .as_ptr()
                    .add(clusterareanum as usize);
                if existing == 0 || existing > t {
                    *(*areacache)
                        .traveltimes
                        .as_mut_ptr()
                        .add(clusterareanum as usize) = t;
                    *(*areacache).reachabilities.add(clusterareanum as usize) = (linknum
                        - bot.aasworld.areasettings[nextareanum as usize].firstreachablearea)
                        as u8;
                    let nextupdate: *mut aas_routingupdate_t =
                        &mut bot.aasworld.areaupdate[clusterareanum as usize];
                    (*nextupdate).areanum = nextareanum;
                    (*nextupdate).tmptraveltime = t;
                    (*nextupdate).areatraveltimes = bot.aasworld.areatraveltimes
                        [nextareanum as usize][(linknum
                        - bot.aasworld.areasettings[nextareanum as usize].firstreachablearea)
                        as usize]
                        .as_mut_ptr();
                    if (*nextupdate).inlist == qfalse {
                        (*nextupdate).next = std::ptr::null_mut();
                        (*nextupdate).prev = updatelistend;
                        if !updatelistend.is_null() {
                            (*updatelistend).next = nextupdate;
                        } else {
                            updateliststart = nextupdate;
                        }
                        updatelistend = nextupdate;
                        (*nextupdate).inlist = qtrue;
                    }
                }
                revlink = (*revlink).next;
                i += 1;
            }
        }
    }
}

/// Raven `AAS_ReachabilityFromNum` — copies out one reachability by index.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1902-1915`
pub fn AAS_ReachabilityFromNum(bot: &mut BotLib, num: c_int, reach: *mut aas_reachability_s) {
    unsafe {
        if bot.aasworld.initialized == qfalse {
            std::ptr::write_bytes(reach, 0, 1);
            return;
        }
        if num < 0 || num >= bot.aasworld.reachabilitysize {
            std::ptr::write_bytes(reach, 0, 1);
            return;
        }
        *reach = bot.aasworld.reachability[num as usize];
    }
}

/// Raven `AAS_NextAreaReachability`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1922-1950`
pub fn AAS_NextAreaReachability(bot: &mut BotLib, areanum: c_int, reachnum: c_int) -> c_int {
    if bot.aasworld.initialized == qfalse {
        return 0;
    }
    if areanum <= 0 || areanum >= bot.aasworld.numareas {
        bot.botimport.Print(
            PRT_ERROR,
            &format!("AAS_NextAreaReachability: areanum {} out of range\n", areanum),
        );
        return 0;
    }
    let settings = bot.aasworld.areasettings[areanum as usize];
    if reachnum == 0 {
        return settings.firstreachablearea;
    }
    if reachnum < settings.firstreachablearea {
        bot.botimport.Print(
            PRT_FATAL,
            "AAS_NextAreaReachability: reachnum < settings->firstreachableara",
        );
        return 0;
    }
    let reachnum = reachnum + 1;
    if reachnum >= settings.firstreachablearea + settings.numreachableareas {
        return 0;
    }
    reachnum
}

/// Raven `AAS_NextModelReachability`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1957-1977`
pub fn AAS_NextModelReachability(bot: &mut BotLib, num: c_int, modelnum: c_int) -> c_int {
    let mut num = num;
    if num <= 0 {
        num = 1;
    } else if num >= bot.aasworld.reachabilitysize {
        return 0;
    } else {
        num += 1;
    }
    for i in num..bot.aasworld.reachabilitysize {
        let reach = &bot.aasworld.reachability[i as usize];
        if (reach.traveltype & TRAVELTYPE_MASK) == TRAVEL_ELEVATOR {
            if reach.facenum == modelnum {
                return i;
            }
        } else if (reach.traveltype & TRAVELTYPE_MASK) == TRAVEL_FUNCBOB {
            if (reach.facenum & 0x0000_FFFF) == modelnum {
                return i;
            }
        }
    }
    0
}

/// Raven `DistancePointToLine`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:2049-2056`
pub fn DistancePointToLine(v1: vec3_t, v2: vec3_t, point: vec3_t) -> f32 {
    // PORT-NOTE(vProj): `AAS_ProjectPointOntoVector`'s resolved signature takes
    // `vProj` by value (see its own PORT-NOTE), so it cannot write back through
    // this call; matches the documented shape mismatch there.
    let p2: vec3_t = [0.0, 0.0, 0.0];
    crate::be_aas_main::AAS_ProjectPointOntoVector(point, v1, v2, p2);
    let vec = [point[0] - p2[0], point[1] - p2[1], point[2] - p2[2]];
    (vec[0] * vec[0] + vec[1] * vec[1] + vec[2] * vec[2]).sqrt()
}

/// Raven `AAS_AreaVisible` — always false (stub; visarea data is never built).
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:2039-2042`
pub fn AAS_AreaVisible(_srcarea: c_int, _destarea: c_int) -> c_int {
    qfalse
}

/// Raven `AAS_RemoveRoutingCacheInCluster` — frees all per-area cache in a cluster.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:231-249`
pub fn AAS_RemoveRoutingCacheInCluster(bot: &mut BotLib, clusternum: c_int) {
    unsafe {
        if bot.aasworld.clusterareacache.is_null() {
            return;
        }
        let cluster: *const aas_cluster_t = &bot.aasworld.clusters[clusternum as usize];
        for i in 0..(*cluster).numareas {
            let mut cache =
                *(*bot.aasworld.clusterareacache.add(clusternum as usize)).add(i as usize);
            while !cache.is_null() {
                let nextcache = (*cache).next;
                AAS_FreeRoutingCache(bot, cache);
                cache = nextcache;
            }
            *(*bot.aasworld.clusterareacache.add(clusternum as usize)).add(i as usize) =
                std::ptr::null_mut();
        }
    }
}

/// Raven `AAS_InitAreaContentsTravelFlags` — rebuilds the per-area content
/// travel-flag lookup table.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:385-395`
pub fn AAS_InitAreaContentsTravelFlags(bot: &mut BotLib) {
    if !bot.aasworld.areacontentstravelflags.is_null() {
        crate::l_memory_fns::FreeMemory(
            bot,
            bot.aasworld.areacontentstravelflags as *mut (),
        );
    }
    bot.aasworld.areacontentstravelflags = crate::l_memory_fns::GetClearedMemory(
        bot,
        (bot.aasworld.numareas * std::mem::size_of::<c_int>() as c_int) as c_ulong,
    ) as *mut c_int;
    for i in 0..bot.aasworld.numareas {
        let flags = AAS_GetAreaContentsTravelFlags(bot, i);
        unsafe {
            *bot.aasworld.areacontentstravelflags.add(i as usize) = flags;
        }
    }
}

/// Raven `AAS_CreateReversedReachability` — builds reversed reachability links.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:402-450`
pub fn AAS_CreateReversedReachability(bot: &mut BotLib) {
    unsafe {
        if !bot.aasworld.reversedreachability.is_null() {
            crate::l_memory_fns::FreeMemory(
                bot,
                bot.aasworld.reversedreachability as *mut (),
            );
        }
        let total = bot.aasworld.numareas as usize
            * std::mem::size_of::<aas_reversedreachability_t>()
            + bot.aasworld.reachabilitysize as usize * std::mem::size_of::<aas_reversedlink_t>();
        let mut ptr = crate::l_memory_fns::GetClearedMemory(bot, total as c_ulong) as *mut u8;

        bot.aasworld.reversedreachability = ptr as *mut aas_reversedreachability_t;
        ptr = ptr.add(
            bot.aasworld.numareas as usize * std::mem::size_of::<aas_reversedreachability_t>(),
        );

        for i in 1..bot.aasworld.numareas {
            let settings: *const aas_areasettings_t = &bot.aasworld.areasettings[i as usize];
            if (*settings).numreachableareas >= 128 {
                bot.botimport.Print(
                    PRT_WARNING,
                    &format!("area {} has more than 128 reachabilities\n", i),
                );
            }
            let mut n = 0;
            while n < (*settings).numreachableareas && n < 128 {
                let reach: *const aas_reachability_t =
                    &bot.aasworld.reachability[((*settings).firstreachablearea + n) as usize];

                let revlink = ptr as *mut aas_reversedlink_t;
                ptr = ptr.add(std::mem::size_of::<aas_reversedlink_t>());

                (*revlink).areanum = i;
                (*revlink).linknum = (*settings).firstreachablearea + n;
                (*revlink).next = (*bot
                    .aasworld
                    .reversedreachability
                    .add((*reach).areanum as usize))
                .first;
                (*bot
                    .aasworld
                    .reversedreachability
                    .add((*reach).areanum as usize))
                .first = revlink;
                (*bot
                    .aasworld
                    .reversedreachability
                    .add((*reach).areanum as usize))
                .numlinks += 1;

                n += 1;
            }
        }
    }
}

/// Raven `AAS_CalculateAreaTravelTimes` — precomputes per-reachability travel times.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:483-542`
pub fn AAS_CalculateAreaTravelTimes(bot: &mut BotLib) {
    unsafe {
        if !bot.aasworld.areatraveltimes.is_null() {
            crate::l_memory_fns::FreeMemory(bot, bot.aasworld.areatraveltimes as *mut ());
        }
        let mut size = bot.aasworld.numareas as usize * std::mem::size_of::<*mut *mut u16>();
        for i in 0..bot.aasworld.numareas {
            let revreach: *const aas_reversedreachability_t =
                &bot.aasworld.reversedreachability[i as usize];
            let settings: *const aas_areasettings_t = &bot.aasworld.areasettings[i as usize];
            size += (*settings).numreachableareas as usize * std::mem::size_of::<*mut u16>();
            size += (*settings).numreachableareas as usize
                * (*revreach).numlinks as usize
                * std::mem::size_of::<u16>();
        }
        let mut ptr = crate::l_memory_fns::GetClearedMemory(bot, size as c_ulong) as *mut u8;
        bot.aasworld.areatraveltimes = ptr as *mut *mut *mut u16;
        ptr = ptr.add(bot.aasworld.numareas as usize * std::mem::size_of::<*mut *mut u16>());

        for i in 0..bot.aasworld.numareas {
            let revreach: *const aas_reversedreachability_t =
                &bot.aasworld.reversedreachability[i as usize];
            let settings: *const aas_areasettings_t = &bot.aasworld.areasettings[i as usize];

            *bot.aasworld.areatraveltimes.add(i as usize) = ptr as *mut *mut u16;
            ptr = ptr.add((*settings).numreachableareas as usize * std::mem::size_of::<*mut u16>());

            for l in 0..(*settings).numreachableareas {
                *(*bot.aasworld.areatraveltimes.add(i as usize)).add(l as usize) = ptr as *mut u16;
                ptr = ptr.add((*revreach).numlinks as usize * std::mem::size_of::<u16>());

                let reach: *const aas_reachability_t =
                    &bot.aasworld.reachability[((*settings).firstreachablearea + l) as usize];

                let mut n = 0usize;
                let mut revlink = (*revreach).first;
                while !revlink.is_null() {
                    let end = bot.aasworld.reachability[(*revlink).linknum as usize].end;
                    let tt = AAS_AreaTravelTime(bot, i, end, (*reach).start);
                    *(*(*bot.aasworld.areatraveltimes.add(i as usize)).add(l as usize)).add(n) = tt;
                    revlink = (*revlink).next;
                    n += 1;
                }
            }
        }
    }
}

/// Raven `AAS_InitPortalMaxTravelTimes`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:583-596`
pub fn AAS_InitPortalMaxTravelTimes(bot: &mut BotLib) {
    if !bot.aasworld.portalmaxtraveltimes.is_null() {
        crate::l_memory_fns::FreeMemory(
            bot,
            bot.aasworld.portalmaxtraveltimes as *mut (),
        );
    }
    bot.aasworld.portalmaxtraveltimes = crate::l_memory_fns::GetClearedMemory(
        bot,
        (bot.aasworld.numportals * std::mem::size_of::<c_int>() as c_int) as c_ulong,
    ) as *mut c_int;
    for i in 0..bot.aasworld.numportals {
        let t = AAS_PortalMaxTravelTime(bot, i);
        unsafe {
            *bot.aasworld.portalmaxtraveltimes.add(i as usize) = t;
        }
    }
}

/// Raven `AAS_FreeOldestCache` — frees the LRU routing cache (never one
/// leading toward a portal), returns whether one was freed.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:678-710`
pub fn AAS_FreeOldestCache(bot: &mut BotLib) -> c_int {
    unsafe {
        let mut cache = bot.aasworld.oldestcache;
        while !cache.is_null() {
            if (*cache).r#type == CACHETYPE_AREA
                && bot.aasworld.areasettings[(*cache).areanum as usize].cluster < 0
            {
                cache = (*cache).time_next;
                continue;
            }
            break;
        }
        if !cache.is_null() {
            if (*cache).r#type == CACHETYPE_AREA {
                let clusterareanum = AAS_ClusterAreaNum(bot, (*cache).cluster, (*cache).areanum);
                if !(*cache).prev.is_null() {
                    (*(*cache).prev).next = (*cache).next;
                } else {
                    *(*bot.aasworld.clusterareacache.add((*cache).cluster as usize))
                        .add(clusterareanum as usize) = (*cache).next;
                }
                if !(*cache).next.is_null() {
                    (*(*cache).next).prev = (*cache).prev;
                }
            } else {
                if !(*cache).prev.is_null() {
                    (*(*cache).prev).next = (*cache).next;
                } else {
                    *bot.aasworld.portalcache.add((*cache).areanum as usize) = (*cache).next;
                }
                if !(*cache).next.is_null() {
                    (*(*cache).next).prev = (*cache).prev;
                }
            }
            AAS_FreeRoutingCache(bot, cache);
            qtrue
        } else {
            qfalse
        }
    }
}

/// Raven `AAS_AllocRoutingCache` — allocates a routing-cache block sized for
/// `numtraveltimes` entries.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:717-734`
pub fn AAS_AllocRoutingCache(bot: &mut BotLib, numtraveltimes: c_int) -> *mut aas_routingcache_t {
    let size = std::mem::size_of::<aas_routingcache_t>() as c_int
        + numtraveltimes * std::mem::size_of::<u16>() as c_int
        + numtraveltimes * std::mem::size_of::<u8>() as c_int;
    bot.routingcachesize += size;
    unsafe {
        let cache =
            crate::l_memory_fns::GetClearedMemory(bot, size as c_ulong) as *mut aas_routingcache_t;
        (*cache).reachabilities = (cache as *mut u8).add(
            std::mem::size_of::<aas_routingcache_t>()
                + numtraveltimes as usize * std::mem::size_of::<u16>(),
        );
        (*cache).size = size;
        cache
    }
}

/// Raven `AAS_FreeAllClusterAreaCache`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:741-766`
pub fn AAS_FreeAllClusterAreaCache(bot: &mut BotLib) {
    unsafe {
        if bot.aasworld.clusterareacache.is_null() {
            return;
        }
        for i in 0..bot.aasworld.numclusters {
            let cluster: *const aas_cluster_t = &bot.aasworld.clusters[i as usize];
            for j in 0..(*cluster).numareas {
                let mut cache = *(*bot.aasworld.clusterareacache.add(i as usize)).add(j as usize);
                while !cache.is_null() {
                    let nextcache = (*cache).next;
                    AAS_FreeRoutingCache(bot, cache);
                    cache = nextcache;
                }
                *(*bot.aasworld.clusterareacache.add(i as usize)).add(j as usize) =
                    std::ptr::null_mut();
            }
        }
        crate::l_memory_fns::FreeMemory(bot, bot.aasworld.clusterareacache as *mut ());
        bot.aasworld.clusterareacache = std::ptr::null_mut();
    }
}

/// Raven `AAS_InitClusterAreaCache` — allocates the per-cluster/per-area
/// routing-cache pointer table.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:773-795`
pub fn AAS_InitClusterAreaCache(bot: &mut BotLib) {
    let mut size = 0i32;
    for i in 0..bot.aasworld.numclusters {
        size += bot.aasworld.clusters[i as usize].numareas;
    }
    unsafe {
        let mut ptr = crate::l_memory_fns::GetClearedMemory(
            bot,
            (bot.aasworld.numclusters * std::mem::size_of::<*mut *mut aas_routingcache_t>() as c_int
                + size * std::mem::size_of::<*mut aas_routingcache_t>() as c_int) as c_ulong,
        ) as *mut u8;
        bot.aasworld.clusterareacache = ptr as *mut *mut *mut aas_routingcache_t;
        ptr = ptr.add(
            bot.aasworld.numclusters as usize * std::mem::size_of::<*mut *mut aas_routingcache_t>(),
        );
        for i in 0..bot.aasworld.numclusters {
            *bot.aasworld.clusterareacache.add(i as usize) = ptr as *mut *mut aas_routingcache_t;
            ptr = ptr.add(
                bot.aasworld.clusters[i as usize].numareas as usize
                    * std::mem::size_of::<*mut aas_routingcache_t>(),
            );
        }
    }
}

/// Raven `AAS_FreeAllPortalCache`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:802-821`
pub fn AAS_FreeAllPortalCache(bot: &mut BotLib) {
    unsafe {
        if bot.aasworld.portalcache.is_null() {
            return;
        }
        for i in 0..bot.aasworld.numareas {
            let mut cache = *bot.aasworld.portalcache.add(i as usize);
            while !cache.is_null() {
                let nextcache = (*cache).next;
                AAS_FreeRoutingCache(bot, cache);
                cache = nextcache;
            }
            *bot.aasworld.portalcache.add(i as usize) = std::ptr::null_mut();
        }
        crate::l_memory_fns::FreeMemory(bot, bot.aasworld.portalcache as *mut ());
        bot.aasworld.portalcache = std::ptr::null_mut();
    }
}

/// Raven `AAS_InitPortalCache`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:828-833`
pub fn AAS_InitPortalCache(bot: &mut BotLib) {
    bot.aasworld.portalcache = crate::l_memory_fns::GetClearedMemory(
        bot,
        (bot.aasworld.numareas * std::mem::size_of::<*mut aas_routingcache_t>() as c_int) as c_ulong,
    ) as *mut *mut aas_routingcache_t;
}

/// Raven `AAS_InitRoutingUpdate` — allocates the per-frame routing-update
/// scratch arrays sized to the widest cluster/portal set.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:840-863`
pub fn AAS_InitRoutingUpdate(bot: &mut BotLib) {
    if !bot.aasworld.areaupdate.is_null() {
        crate::l_memory_fns::FreeMemory(bot, bot.aasworld.areaupdate as *mut ());
    }
    let mut maxreachabilityareas = 0;
    for i in 0..bot.aasworld.numclusters {
        if bot.aasworld.clusters[i as usize].numreachabilityareas > maxreachabilityareas {
            maxreachabilityareas = bot.aasworld.clusters[i as usize].numreachabilityareas;
        }
    }
    bot.aasworld.areaupdate = crate::l_memory_fns::GetClearedMemory(
        bot,
        (maxreachabilityareas * std::mem::size_of::<aas_routingupdate_t>() as c_int) as c_ulong,
    ) as *mut aas_routingupdate_t;

    if !bot.aasworld.portalupdate.is_null() {
        crate::l_memory_fns::FreeMemory(bot, bot.aasworld.portalupdate as *mut ());
    }
    bot.aasworld.portalupdate = crate::l_memory_fns::GetClearedMemory(
        bot,
        ((bot.aasworld.numportals + 1) * std::mem::size_of::<aas_routingupdate_t>() as c_int)
            as c_ulong,
    ) as *mut aas_routingupdate_t;
}

/// Raven `AAS_WriteRouteCache` — writes the `.rcd` route-cache dump.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:917-1007`
pub fn AAS_WriteRouteCache(bot: &mut BotLib) {
    unsafe {
        let mut numportalcache = 0;
        for i in 0..bot.aasworld.numareas {
            let mut cache = *bot.aasworld.portalcache.add(i as usize);
            while !cache.is_null() {
                numportalcache += 1;
                cache = (*cache).next;
            }
        }
        let mut numareacache = 0;
        for i in 0..bot.aasworld.numclusters {
            let cluster: *const aas_cluster_t = &bot.aasworld.clusters[i as usize];
            for j in 0..(*cluster).numareas {
                let mut cache = *(*bot.aasworld.clusterareacache.add(i as usize)).add(j as usize);
                while !cache.is_null() {
                    numareacache += 1;
                    cache = (*cache).next;
                }
            }
        }

        let filename = format!("maps/{}.rcd", bot.aasworld.mapname);
        let mut fp: fileHandle_t = 0;
        bot.botimport.FS_FOpenFile(&filename, &mut fp, FS_WRITE);
        if fp == 0 {
            let msg = std::ffi::CString::new(format!("Unable to open file: {}\n", filename))
                .unwrap_or_default();
            crate::be_aas_main::AAS_Error(bot, msg.as_ptr() as *mut core::ffi::c_char);
            return;
        }

        let mut routecacheheader = crate::be_aas_route::routecacheheader_t::default();
        routecacheheader.ident = crate::be_aas_route::RCID;
        routecacheheader.version = crate::be_aas_route::RCVERSION;
        routecacheheader.numareas = bot.aasworld.numareas;
        routecacheheader.numclusters = bot.aasworld.numclusters;
        routecacheheader.areacrc = crate::l_crc_fns::CRC_ProcessString(
            bot,
            bot.aasworld.areas as *mut core::ffi::c_uchar,
            std::mem::size_of::<aas_area_t>() as c_int * bot.aasworld.numareas,
        ) as c_int;
        routecacheheader.clustercrc = crate::l_crc_fns::CRC_ProcessString(
            bot,
            bot.aasworld.clusters as *mut core::ffi::c_uchar,
            std::mem::size_of::<aas_cluster_t>() as c_int * bot.aasworld.numclusters,
        ) as c_int;
        routecacheheader.numportalcache = numportalcache;
        routecacheheader.numareacache = numareacache;
        bot.botimport.FS_Write(
            &routecacheheader as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<crate::be_aas_route::routecacheheader_t>() as c_int,
            fp,
        );

        let mut totalsize = 0;
        for i in 0..bot.aasworld.numareas {
            let mut cache = *bot.aasworld.portalcache.add(i as usize);
            while !cache.is_null() {
                bot.botimport
                    .FS_Write(cache as *const std::ffi::c_void, (*cache).size, fp);
                totalsize += (*cache).size;
                cache = (*cache).next;
            }
        }
        for i in 0..bot.aasworld.numclusters {
            let cluster: *const aas_cluster_t = &bot.aasworld.clusters[i as usize];
            for j in 0..(*cluster).numareas {
                let mut cache = *(*bot.aasworld.clusterareacache.add(i as usize)).add(j as usize);
                while !cache.is_null() {
                    bot.botimport
                        .FS_Write(cache as *const std::ffi::c_void, (*cache).size, fp);
                    totalsize += (*cache).size;
                    cache = (*cache).next;
                }
            }
        }
        // NOTE: the visarea write pass is commented out in the oracle
        // (be_aas_route.cpp:988-1002); not transcribed.
        bot.botimport.FS_FCloseFile(fp);
        bot.botimport.Print(
            PRT_MESSAGE,
            &format!("\nroute cache written to {}\n", filename),
        );
        bot.botimport.Print(
            PRT_MESSAGE,
            &format!("written {} bytes of routing cache\n", totalsize),
        );
    }
}

/// Raven `AAS_ReadRouteCache` — reads back the `.rcd` route-cache dump if the
/// map/CRC/version match.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1033-1117`
pub fn AAS_ReadRouteCache(bot: &mut BotLib) -> c_int {
    unsafe {
        let filename = format!("maps/{}.rcd", bot.aasworld.mapname);
        let mut fp: fileHandle_t = 0;
        bot.botimport.FS_FOpenFile(&filename, &mut fp, FS_READ);
        if fp == 0 {
            return qfalse;
        }
        let mut routecacheheader = crate::be_aas_route::routecacheheader_t::default();
        bot.botimport.FS_Read(
            &mut routecacheheader as *mut _ as *mut std::ffi::c_void,
            std::mem::size_of::<crate::be_aas_route::routecacheheader_t>() as c_int,
            fp,
        );
        if routecacheheader.ident != crate::be_aas_route::RCID {
            let msg = c"%s is not a route cache dump\n";
            crate::be_aas_main::AAS_Error(bot, msg.as_ptr() as *mut core::ffi::c_char);
            return qfalse;
        }
        if routecacheheader.version != crate::be_aas_route::RCVERSION {
            let msg = std::ffi::CString::new(format!(
                "route cache dump has wrong version {}, should be {}",
                routecacheheader.version,
                crate::be_aas_route::RCVERSION
            ))
            .unwrap_or_default();
            crate::be_aas_main::AAS_Error(bot, msg.as_ptr() as *mut core::ffi::c_char);
            return qfalse;
        }
        if routecacheheader.numareas != bot.aasworld.numareas {
            return qfalse;
        }
        if routecacheheader.numclusters != bot.aasworld.numclusters {
            return qfalse;
        }
        if routecacheheader.areacrc
            != crate::l_crc_fns::CRC_ProcessString(
                bot,
                bot.aasworld.areas as *mut core::ffi::c_uchar,
                std::mem::size_of::<aas_area_t>() as c_int * bot.aasworld.numareas,
            ) as c_int
        {
            return qfalse;
        }
        if routecacheheader.clustercrc
            != crate::l_crc_fns::CRC_ProcessString(
                bot,
                bot.aasworld.clusters as *mut core::ffi::c_uchar,
                std::mem::size_of::<aas_cluster_t>() as c_int * bot.aasworld.numclusters,
            ) as c_int
        {
            return qfalse;
        }
        for _ in 0..routecacheheader.numportalcache {
            let cache = AAS_ReadCache(bot, fp);
            (*cache).next = *bot.aasworld.portalcache.add((*cache).areanum as usize);
            (*cache).prev = std::ptr::null_mut();
            if !(*bot.aasworld.portalcache.add((*cache).areanum as usize)).is_null() {
                (*(*bot.aasworld.portalcache.add((*cache).areanum as usize))).prev = cache;
            }
            *bot.aasworld.portalcache.add((*cache).areanum as usize) = cache;
        }
        for _ in 0..routecacheheader.numareacache {
            let cache = AAS_ReadCache(bot, fp);
            let clusterareanum = AAS_ClusterAreaNum(bot, (*cache).cluster, (*cache).areanum);
            (*cache).next = *(*bot.aasworld.clusterareacache.add((*cache).cluster as usize))
                .add(clusterareanum as usize);
            (*cache).prev = std::ptr::null_mut();
            if !(*(*bot.aasworld.clusterareacache.add((*cache).cluster as usize))
                .add(clusterareanum as usize))
            .is_null()
            {
                (*(*(*bot.aasworld.clusterareacache.add((*cache).cluster as usize))
                    .add(clusterareanum as usize)))
                .prev = cache;
            }
            *(*bot.aasworld.clusterareacache.add((*cache).cluster as usize))
                .add(clusterareanum as usize) = cache;
        }
        // NOTE: the visarea read pass is commented out in the oracle
        // (be_aas_route.cpp:1102-1113); not transcribed.
        bot.botimport.FS_FCloseFile(fp);
        qtrue
    }
}

/// Raven `AAS_InitReachabilityAreas` — builds, per reachability, the list of
/// areas its inter-area trace passes through.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1126-1191`
pub fn AAS_InitReachabilityAreas(bot: &mut BotLib) {
    // §19: `areas` is Raven's uninitialized local scratch buffer, only the
    // first `numareas` entries of which are ever read (written by
    // `AAS_TraceAreas` before use in every reachable branch); zero-init here.
    let mut areas: [c_int; crate::be_aas_route::MAX_REACHABILITYPASSAREAS as usize] =
        [0; crate::be_aas_route::MAX_REACHABILITYPASSAREAS as usize];

    if !bot.aasworld.reachabilityareas.is_null() {
        crate::l_memory_fns::FreeMemory(bot, bot.aasworld.reachabilityareas as *mut ());
    }
    if !bot.aasworld.reachabilityareaindex.is_null() {
        crate::l_memory_fns::FreeMemory(
            bot,
            bot.aasworld.reachabilityareaindex as *mut (),
        );
    }

    bot.aasworld.reachabilityareas = crate::l_memory_fns::GetClearedMemory(
        bot,
        (bot.aasworld.reachabilitysize * std::mem::size_of::<aas_reachabilityareas_t>() as c_int)
            as c_ulong,
    ) as *mut aas_reachabilityareas_t;
    bot.aasworld.reachabilityareaindex = crate::l_memory_fns::GetClearedMemory(
        bot,
        (bot.aasworld.reachabilitysize
            * crate::be_aas_route::MAX_REACHABILITYPASSAREAS as c_int
            * std::mem::size_of::<c_int>() as c_int) as c_ulong,
    ) as *mut c_int;

    let mut numreachareas = 0;
    for i in 0..bot.aasworld.reachabilitysize {
        let reach = bot.aasworld.reachability[i as usize];
        let mut numareas = 0;
        match reach.traveltype & TRAVELTYPE_MASK {
            t if t == TRAVEL_BARRIERJUMP || t == TRAVEL_WATERJUMP => {
                let mut end = reach.start;
                end[2] = reach.end[2];
                numareas = crate::be_aas_sample_fns::AAS_TraceAreas(
                    bot,
                    reach.start,
                    end,
                    areas.as_mut_ptr(),
                    std::ptr::null_mut(),
                    crate::be_aas_route::MAX_REACHABILITYPASSAREAS as c_int,
                );
            }
            t if t == TRAVEL_WALKOFFLEDGE => {
                let mut start = reach.end;
                start[2] = reach.start[2];
                numareas = crate::be_aas_sample_fns::AAS_TraceAreas(
                    bot,
                    start,
                    reach.end,
                    areas.as_mut_ptr(),
                    std::ptr::null_mut(),
                    crate::be_aas_route::MAX_REACHABILITYPASSAREAS as c_int,
                );
            }
            t if t == TRAVEL_GRAPPLEHOOK => {
                numareas = crate::be_aas_sample_fns::AAS_TraceAreas(
                    bot,
                    reach.start,
                    reach.end,
                    areas.as_mut_ptr(),
                    std::ptr::null_mut(),
                    crate::be_aas_route::MAX_REACHABILITYPASSAREAS as c_int,
                );
            }
            t if t == TRAVEL_JUMP
                || t == TRAVEL_ROCKETJUMP
                || t == TRAVEL_BFGJUMP
                || t == TRAVEL_JUMPPAD
                || t == TRAVEL_ELEVATOR
                || t == TRAVEL_FUNCBOB
                || t == TRAVEL_WALK
                || t == TRAVEL_CROUCH
                || t == TRAVEL_LADDER
                || t == TRAVEL_SWIM
                || t == TRAVEL_TELEPORT => {}
            _ => {}
        }
        unsafe {
            (*bot.aasworld.reachabilityareas.add(i as usize)).firstarea = numreachareas;
            (*bot.aasworld.reachabilityareas.add(i as usize)).numareas = numareas;
            for j in 0..numareas {
                *bot.aasworld
                    .reachabilityareaindex
                    .add(numreachareas as usize) = areas[j as usize];
                numreachareas += 1;
            }
        }
    }
}

/// Raven `AAS_NearestHideArea` — best reachable area out of enemy sight, near
/// the source area, avoiding the enemy's position.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:2063-2192`
pub fn AAS_NearestHideArea(
    bot: &mut BotLib,
    _srcnum: c_int,
    origin: vec3_t,
    areanum: c_int,
    _enemynum: c_int,
    enemyorigin: vec3_t,
    enemyareanum: c_int,
    travelflags: c_int,
) -> c_int {
    unsafe {
        // Function-scope static `hidetraveltimes` (fork-3 rule): genuine
        // cross-frame state → a field on the owning host struct.
        if bot.hidetraveltimes.is_null() {
            bot.hidetraveltimes = crate::l_memory_fns::GetClearedMemory(
                bot,
                (bot.aasworld.numareas * std::mem::size_of::<u16>() as c_int) as c_ulong,
            ) as *mut u16;
        } else {
            std::ptr::write_bytes(bot.hidetraveltimes, 0, bot.aasworld.numareas as usize);
        }
        let mut besttraveltime: u16 = 0;
        let mut bestarea: c_int = 0;
        let start_visible = qtrue;

        let badtravelflags = !travelflags;

        let curupdate: *mut aas_routingupdate_t = &mut bot.aasworld.areaupdate[areanum as usize];
        (*curupdate).areanum = areanum;
        (*curupdate).start = origin;
        (*curupdate).areatraveltimes =
            *(*bot.aasworld.areatraveltimes.add(areanum as usize)).add(0);
        (*curupdate).tmptraveltime = 0;

        (*curupdate).next = std::ptr::null_mut();
        (*curupdate).prev = std::ptr::null_mut();
        let mut updateliststart = curupdate;
        let mut updatelistend = curupdate;

        while !updateliststart.is_null() {
            let curupdate = updateliststart;

            if !(*curupdate).next.is_null() {
                (*(*curupdate).next).prev = std::ptr::null_mut();
            } else {
                updatelistend = std::ptr::null_mut();
            }
            updateliststart = (*curupdate).next;

            (*curupdate).inlist = qfalse;

            let numreach =
                bot.aasworld.areasettings[(*curupdate).areanum as usize].numreachableareas;
            let firstreach =
                bot.aasworld.areasettings[(*curupdate).areanum as usize].firstreachablearea;

            for i in 0..numreach {
                let reach: *const aas_reachability_t =
                    &bot.aasworld.reachability[(firstreach + i) as usize];

                if AAS_TravelFlagForType_inline(bot, (*reach).traveltype) & badtravelflags != 0 {
                    continue;
                }
                if AAS_AreaContentsTravelFlags_inline(bot, (*reach).areanum) & badtravelflags != 0 {
                    continue;
                }
                let nextareanum = (*reach).areanum;
                if nextareanum == enemyareanum {
                    continue;
                }
                let mut t = (*curupdate).tmptraveltime
                    + AAS_AreaTravelTime(
                        bot,
                        (*curupdate).areanum,
                        (*curupdate).start,
                        (*reach).start,
                    );

                // PORT-NOTE(vProj): see `DistancePointToLine` above — the resolved
                // signature cannot write back through this call.
                let p: vec3_t = [0.0, 0.0, 0.0];
                crate::be_aas_main::AAS_ProjectPointOntoVector(
                    enemyorigin,
                    (*curupdate).start,
                    (*reach).end,
                    p,
                );
                let mut j = 0;
                while j < 3 {
                    if (p[j] > (*curupdate).start[j] && p[j] > (*reach).end[j])
                        || (p[j] < (*curupdate).start[j] && p[j] < (*reach).end[j])
                    {
                        break;
                    }
                    j += 1;
                }
                let v2 = if j < 3 {
                    [
                        enemyorigin[0] - (*reach).end[0],
                        enemyorigin[1] - (*reach).end[1],
                        enemyorigin[2] - (*reach).end[2],
                    ]
                } else {
                    [
                        enemyorigin[0] - p[0],
                        enemyorigin[1] - p[1],
                        enemyorigin[2] - p[2],
                    ]
                };
                let dist2 = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2]).sqrt();
                if dist2 < 40.0 {
                    continue;
                }
                let v1 = [
                    enemyorigin[0] - (*curupdate).start[0],
                    enemyorigin[1] - (*curupdate).start[1],
                    enemyorigin[2] - (*curupdate).start[2],
                ];
                let dist1 = (v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2]).sqrt();

                if dist2 < dist1 {
                    t += ((dist1 - dist2) * 10.0) as u16;
                }
                if start_visible == qfalse && AAS_AreaVisible(enemyareanum, nextareanum) != 0 {
                    continue;
                }
                if besttraveltime != 0 && t >= besttraveltime {
                    continue;
                }
                let existing = *bot.hidetraveltimes.add(nextareanum as usize);
                if existing == 0 || existing > t {
                    if AAS_AreaVisible(enemyareanum, nextareanum) == 0 {
                        besttraveltime = t;
                        bestarea = nextareanum;
                    }
                    *bot.hidetraveltimes.add(nextareanum as usize) = t;
                    let nextupdate: *mut aas_routingupdate_t =
                        &mut bot.aasworld.areaupdate[nextareanum as usize];
                    (*nextupdate).areanum = nextareanum;
                    (*nextupdate).tmptraveltime = t;
                    (*nextupdate).start = (*reach).end;
                    if (*nextupdate).inlist == qfalse {
                        (*nextupdate).next = std::ptr::null_mut();
                        (*nextupdate).prev = updatelistend;
                        if !updatelistend.is_null() {
                            (*updatelistend).next = nextupdate;
                        } else {
                            updateliststart = nextupdate;
                        }
                        updatelistend = nextupdate;
                        (*nextupdate).inlist = qtrue;
                    }
                }
            }
        }
        bestarea
    }
}

/// Raven `AAS_RemoveRoutingCacheUsingArea` — invalidates all cache that could
/// route through `areanum` (its cluster's, or both sides if it's a portal;
/// always the portal cache).
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:256-284`
pub fn AAS_RemoveRoutingCacheUsingArea(bot: &mut BotLib, areanum: c_int) {
    let clusternum = bot.aasworld.areasettings[areanum as usize].cluster;
    if clusternum > 0 {
        AAS_RemoveRoutingCacheInCluster(bot, clusternum);
    } else {
        let front = bot.aasworld.portals[(-clusternum) as usize].frontcluster;
        let back = bot.aasworld.portals[(-clusternum) as usize].backcluster;
        AAS_RemoveRoutingCacheInCluster(bot, front);
        AAS_RemoveRoutingCacheInCluster(bot, back);
    }
    unsafe {
        for i in 0..bot.aasworld.numareas {
            let mut cache = *bot.aasworld.portalcache.add(i as usize);
            while !cache.is_null() {
                let nextcache = (*cache).next;
                AAS_FreeRoutingCache(bot, cache);
                cache = nextcache;
            }
            *bot.aasworld.portalcache.add(i as usize) = std::ptr::null_mut();
        }
    }
}

/// Raven `AAS_FreeRoutingCaches` — tears down all routing-cache-related state.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1234-1263`
pub fn AAS_FreeRoutingCaches(bot: &mut BotLib) {
    AAS_FreeAllClusterAreaCache(bot);
    AAS_FreeAllPortalCache(bot);
    if !bot.aasworld.areatraveltimes.is_null() {
        crate::l_memory_fns::FreeMemory(bot, bot.aasworld.areatraveltimes as *mut ());
    }
    bot.aasworld.areatraveltimes = std::ptr::null_mut();
    if !bot.aasworld.portalmaxtraveltimes.is_null() {
        crate::l_memory_fns::FreeMemory(
            bot,
            bot.aasworld.portalmaxtraveltimes as *mut (),
        );
    }
    bot.aasworld.portalmaxtraveltimes = std::ptr::null_mut();
    if !bot.aasworld.reversedreachability.is_null() {
        crate::l_memory_fns::FreeMemory(
            bot,
            bot.aasworld.reversedreachability as *mut (),
        );
    }
    bot.aasworld.reversedreachability = std::ptr::null_mut();
    if !bot.aasworld.areaupdate.is_null() {
        crate::l_memory_fns::FreeMemory(bot, bot.aasworld.areaupdate as *mut ());
    }
    bot.aasworld.areaupdate = std::ptr::null_mut();
    if !bot.aasworld.portalupdate.is_null() {
        crate::l_memory_fns::FreeMemory(bot, bot.aasworld.portalupdate as *mut ());
    }
    bot.aasworld.portalupdate = std::ptr::null_mut();
    if !bot.aasworld.reachabilityareas.is_null() {
        crate::l_memory_fns::FreeMemory(bot, bot.aasworld.reachabilityareas as *mut ());
    }
    bot.aasworld.reachabilityareas = std::ptr::null_mut();
    if !bot.aasworld.reachabilityareaindex.is_null() {
        crate::l_memory_fns::FreeMemory(
            bot,
            bot.aasworld.reachabilityareaindex as *mut (),
        );
    }
    bot.aasworld.reachabilityareaindex = std::ptr::null_mut();
    if !bot.aasworld.areacontentstravelflags.is_null() {
        crate::l_memory_fns::FreeMemory(
            bot,
            bot.aasworld.areacontentstravelflags as *mut (),
        );
    }
    bot.aasworld.areacontentstravelflags = std::ptr::null_mut();
}

/// Raven `AAS_GetAreaRoutingCache` — finds or builds the per-cluster area
/// routing cache for the given travel flags.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1382-1421`
pub fn AAS_GetAreaRoutingCache(
    bot: &mut BotLib,
    clusternum: c_int,
    areanum: c_int,
    travelflags: c_int,
) -> *mut aas_routingcache_t {
    unsafe {
        let clusterareanum = AAS_ClusterAreaNum(bot, clusternum, areanum);
        let clustercache =
            *(*bot.aasworld.clusterareacache.add(clusternum as usize)).add(clusterareanum as usize);
        let mut cache = clustercache;
        while !cache.is_null() {
            if (*cache).travelflags == travelflags {
                break;
            }
            cache = (*cache).next;
        }
        if cache.is_null() {
            cache = AAS_AllocRoutingCache(
                bot,
                bot.aasworld.clusters[clusternum as usize].numreachabilityareas,
            );
            (*cache).cluster = clusternum;
            (*cache).areanum = areanum;
            (*cache).origin = bot.aasworld.areas[areanum as usize].center;
            (*cache).starttraveltime = 1.0;
            (*cache).travelflags = travelflags;
            (*cache).prev = std::ptr::null_mut();
            (*cache).next = clustercache;
            if !clustercache.is_null() {
                (*clustercache).prev = cache;
            }
            *(*bot.aasworld.clusterareacache.add(clusternum as usize))
                .add(clusterareanum as usize) = cache;
            AAS_UpdateAreaRoutingCache(bot, cache);
        } else {
            AAS_UnlinkCache(bot, cache);
        }
        (*cache).time = AAS_RoutingTime(bot);
        (*cache).r#type = CACHETYPE_AREA;
        AAS_LinkCache(bot, cache);
        cache
    }
}

/// Raven `AAS_EnableRoutingArea` — enables/disables/queries an area for
/// routing.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:291-318`
pub fn AAS_EnableRoutingArea(bot: &mut BotLib, areanum: c_int, enable: c_int) -> c_int {
    if areanum <= 0 || areanum >= bot.aasworld.numareas {
        if bot.bot_developer != 0 {
            bot.botimport.Print(
                PRT_ERROR,
                &format!("AAS_EnableRoutingArea: areanum {} out of range\n", areanum),
            );
        }
        return 0;
    }
    let flags = bot.aasworld.areasettings[areanum as usize].areaflags & AREA_DISABLED;
    if enable < 0 {
        return (flags == 0) as c_int;
    }
    if enable != 0 {
        bot.aasworld.areasettings[areanum as usize].areaflags &= !AREA_DISABLED;
    } else {
        bot.aasworld.areasettings[areanum as usize].areaflags |= AREA_DISABLED;
    }
    if (flags & AREA_DISABLED)
        != (bot.aasworld.areasettings[areanum as usize].areaflags & AREA_DISABLED)
    {
        AAS_RemoveRoutingCacheUsingArea(bot, areanum);
    }
    (flags == 0) as c_int
}

/// Raven `AAS_InitRouting` — one-time (per-map) routing subsystem init.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1198-1227`
pub fn AAS_InitRouting(bot: &mut BotLib) {
    AAS_InitTravelFlagFromType(bot);
    AAS_InitAreaContentsTravelFlags(bot);
    AAS_InitRoutingUpdate(bot);
    AAS_CreateReversedReachability(bot);
    AAS_InitClusterAreaCache(bot);
    AAS_InitPortalCache(bot);
    AAS_CalculateAreaTravelTimes(bot);
    AAS_InitPortalMaxTravelTimes(bot);
    AAS_InitReachabilityAreas(bot);
    bot.routingcachesize = 0;
    bot.max_routingcachesize =
        1024 * crate::l_libvar::LibVarValue(bot, "max_routingcache", "4096") as c_int;
    AAS_ReadRouteCache(bot);
}

/// Raven `AAS_UpdatePortalRoutingCache` — Dijkstra-like portal routing-cache fill.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1428-1519`
pub fn AAS_UpdatePortalRoutingCache(bot: &mut BotLib, portalcache: *mut aas_routingcache_t) {
    unsafe {
        let curupdate: *mut aas_routingupdate_t =
            &mut bot.aasworld.portalupdate[bot.aasworld.numportals as usize];
        (*curupdate).cluster = (*portalcache).cluster;
        (*curupdate).areanum = (*portalcache).areanum;
        (*curupdate).tmptraveltime = (*portalcache).starttraveltime;

        let clusternum = bot.aasworld.areasettings[(*portalcache).areanum as usize].cluster;
        if clusternum < 0 {
            *(*portalcache)
                .traveltimes
                .as_mut_ptr()
                .add((-clusternum) as usize) = (*portalcache).starttraveltime;
        }

        (*curupdate).next = std::ptr::null_mut();
        (*curupdate).prev = std::ptr::null_mut();
        let mut updateliststart = curupdate;
        let mut updatelistend = curupdate;

        while !updateliststart.is_null() {
            let curupdate = updateliststart;
            if !(*curupdate).next.is_null() {
                (*(*curupdate).next).prev = std::ptr::null_mut();
            } else {
                updatelistend = std::ptr::null_mut();
            }
            updateliststart = (*curupdate).next;
            (*curupdate).inlist = qfalse;

            let cluster: *const aas_cluster_t =
                &bot.aasworld.clusters[(*curupdate).cluster as usize];

            let cache = AAS_GetAreaRoutingCache(
                bot,
                (*curupdate).cluster,
                (*curupdate).areanum,
                (*portalcache).travelflags,
            );

            for i in 0..(*cluster).numportals {
                let portalnum = bot.aasworld.portalindex[((*cluster).firstportal + i) as usize];
                let portal: *const aas_portal_t = &bot.aasworld.portals[portalnum as usize];
                if (*portal).areanum == (*curupdate).areanum {
                    continue;
                }
                let clusterareanum =
                    AAS_ClusterAreaNum(bot, (*curupdate).cluster, (*portal).areanum);
                if clusterareanum >= (*cluster).numreachabilityareas {
                    continue;
                }
                let t0 = *(*cache).traveltimes.as_ptr().add(clusterareanum as usize);
                if t0 == 0 {
                    continue;
                }
                let t = t0 + (*curupdate).tmptraveltime;

                let existing = *(*portalcache).traveltimes.as_ptr().add(portalnum as usize);
                if existing == 0 || existing > t {
                    *(*portalcache)
                        .traveltimes
                        .as_mut_ptr()
                        .add(portalnum as usize) = t;
                    let nextupdate: *mut aas_routingupdate_t =
                        &mut bot.aasworld.portalupdate[portalnum as usize];
                    if (*portal).frontcluster == (*curupdate).cluster {
                        (*nextupdate).cluster = (*portal).backcluster;
                    } else {
                        (*nextupdate).cluster = (*portal).frontcluster;
                    }
                    (*nextupdate).areanum = (*portal).areanum;
                    (*nextupdate).tmptraveltime =
                        t + *bot.aasworld.portalmaxtraveltimes.add(portalnum as usize) as u16;
                    if (*nextupdate).inlist == qfalse {
                        (*nextupdate).next = std::ptr::null_mut();
                        (*nextupdate).prev = updatelistend;
                        if !updatelistend.is_null() {
                            (*updatelistend).next = nextupdate;
                        } else {
                            updateliststart = nextupdate;
                        }
                        updatelistend = nextupdate;
                        (*nextupdate).inlist = qtrue;
                    }
                }
            }
        }
    }
}

/// Raven `AAS_GetPortalRoutingCache` — finds or builds the portal routing
/// cache for the given travel flags.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1526-1561`
pub fn AAS_GetPortalRoutingCache(
    bot: &mut BotLib,
    clusternum: c_int,
    areanum: c_int,
    travelflags: c_int,
) -> *mut aas_routingcache_t {
    unsafe {
        let mut cache = *bot.aasworld.portalcache.add(areanum as usize);
        while !cache.is_null() {
            if (*cache).travelflags == travelflags {
                break;
            }
            cache = (*cache).next;
        }
        if cache.is_null() {
            cache = AAS_AllocRoutingCache(bot, bot.aasworld.numportals);
            (*cache).cluster = clusternum;
            (*cache).areanum = areanum;
            (*cache).origin = bot.aasworld.areas[areanum as usize].center;
            (*cache).starttraveltime = 1.0;
            (*cache).travelflags = travelflags;
            (*cache).prev = std::ptr::null_mut();
            (*cache).next = *bot.aasworld.portalcache.add(areanum as usize);
            if !(*bot.aasworld.portalcache.add(areanum as usize)).is_null() {
                (*(*bot.aasworld.portalcache.add(areanum as usize))).prev = cache;
            }
            *bot.aasworld.portalcache.add(areanum as usize) = cache;
            AAS_UpdatePortalRoutingCache(bot, cache);
        } else {
            AAS_UnlinkCache(bot, cache);
        }
        (*cache).time = AAS_RoutingTime(bot);
        (*cache).r#type = CACHETYPE_PORTAL;
        AAS_LinkCache(bot, cache);
        cache
    }
}

/// Raven `AAS_AreaRouteToGoalArea` — the core routing query: travel time +
/// first reachability from `areanum` toward `goalareanum`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1568-1743`
pub fn AAS_AreaRouteToGoalArea(
    bot: &mut BotLib,
    areanum: c_int,
    origin: *const vec3_t,
    goalareanum: c_int,
    mut travelflags: c_int,
    traveltime: *mut c_int,
    reachnum: *mut c_int,
) -> c_int {
    unsafe {
        if bot.aasworld.initialized == qfalse {
            return qfalse;
        }
        if areanum == goalareanum {
            *traveltime = 1;
            *reachnum = 0;
            return qtrue;
        }
        if areanum <= 0 || areanum >= bot.aasworld.numareas {
            if bot.bot_developer != 0 {
                bot.botimport.Print(
                    PRT_ERROR,
                    &format!(
                        "AAS_AreaTravelTimeToGoalArea: areanum {} out of range\n",
                        areanum
                    ),
                );
            }
            return qfalse;
        }
        if goalareanum <= 0 || goalareanum >= bot.aasworld.numareas {
            if bot.bot_developer != 0 {
                bot.botimport.Print(
                    PRT_ERROR,
                    &format!(
                        "AAS_AreaTravelTimeToGoalArea: goalareanum {} out of range\n",
                        goalareanum
                    ),
                );
            }
            return qfalse;
        }
        while crate::l_memory_fns::AvailableMemory(bot) < 1 * 1024 * 1024 {
            if AAS_FreeOldestCache(bot) == 0 {
                break;
            }
        }
        if crate::be_aas_reach_fns::AAS_AreaDoNotEnter(bot, areanum) != 0
            || crate::be_aas_reach_fns::AAS_AreaDoNotEnter(bot, goalareanum) != 0
        {
            travelflags |= TFL_DONOTENTER;
        }

        let mut clusternum = bot.aasworld.areasettings[areanum as usize].cluster;
        let mut goalclusternum = bot.aasworld.areasettings[goalareanum as usize].cluster;
        if clusternum < 0 && goalclusternum > 0 {
            let portal: *const aas_portal_t = &bot.aasworld.portals[(-clusternum) as usize];
            if (*portal).frontcluster == goalclusternum || (*portal).backcluster == goalclusternum {
                clusternum = goalclusternum;
            }
        } else if clusternum > 0 && goalclusternum < 0 {
            let portal: *const aas_portal_t = &bot.aasworld.portals[(-goalclusternum) as usize];
            if (*portal).frontcluster == clusternum || (*portal).backcluster == clusternum {
                goalclusternum = clusternum;
            }
        }
        if clusternum > 0 && goalclusternum > 0 && clusternum == goalclusternum {
            let areacache = AAS_GetAreaRoutingCache(bot, clusternum, goalareanum, travelflags);
            let clusterareanum = AAS_ClusterAreaNum(bot, clusternum, areanum);
            let cluster: *const aas_cluster_t = &bot.aasworld.clusters[clusternum as usize];
            if clusterareanum >= (*cluster).numreachabilityareas {
                return 0;
            }
            let tt = *(*areacache)
                .traveltimes
                .as_ptr()
                .add(clusterareanum as usize);
            if tt != 0 {
                *reachnum = bot.aasworld.areasettings[areanum as usize].firstreachablearea
                    + *(*areacache).reachabilities.add(clusterareanum as usize) as c_int;
                if origin.is_null() {
                    *traveltime = tt as c_int;
                    return qtrue;
                }
                let reach = &bot.aasworld.reachability[*reachnum as usize];
                *traveltime =
                    tt as c_int + AAS_AreaTravelTime(bot, areanum, *origin, reach.start) as c_int;
                return qtrue;
            }
        }

        clusternum = bot.aasworld.areasettings[areanum as usize].cluster;
        goalclusternum = bot.aasworld.areasettings[goalareanum as usize].cluster;
        if goalclusternum < 0 {
            let portal: *const aas_portal_t = &bot.aasworld.portals[(-goalclusternum) as usize];
            goalclusternum = (*portal).frontcluster;
        }
        let portalcache = AAS_GetPortalRoutingCache(bot, goalclusternum, goalareanum, travelflags);
        if clusternum < 0 {
            *traveltime = *(*portalcache)
                .traveltimes
                .as_ptr()
                .add((-clusternum) as usize) as c_int;
            *reachnum = bot.aasworld.areasettings[areanum as usize].firstreachablearea
                + *(*portalcache).reachabilities.add((-clusternum) as usize) as c_int;
            return qtrue;
        }

        let mut besttime: u16 = 0;
        let mut bestreachnum: c_int = -1;
        let cluster: *const aas_cluster_t = &bot.aasworld.clusters[clusternum as usize];
        for i in 0..(*cluster).numportals {
            let portalnum = bot.aasworld.portalindex[((*cluster).firstportal + i) as usize];
            if *(*portalcache).traveltimes.as_ptr().add(portalnum as usize) == 0 {
                continue;
            }
            let portal: *const aas_portal_t = &bot.aasworld.portals[portalnum as usize];
            let areacache =
                AAS_GetAreaRoutingCache(bot, clusternum, (*portal).areanum, travelflags);
            let clusterareanum = AAS_ClusterAreaNum(bot, clusternum, areanum);
            if clusterareanum >= (*cluster).numreachabilityareas {
                continue;
            }
            if *(*areacache)
                .traveltimes
                .as_ptr()
                .add(clusterareanum as usize)
                == 0
            {
                continue;
            }
            let mut t = *(*portalcache).traveltimes.as_ptr().add(portalnum as usize)
                + *(*areacache)
                    .traveltimes
                    .as_ptr()
                    .add(clusterareanum as usize);
            t += *bot.aasworld.portalmaxtraveltimes.add(portalnum as usize) as u16;

            if !origin.is_null() {
                *reachnum = bot.aasworld.areasettings[areanum as usize].firstreachablearea
                    + *(*areacache).reachabilities.add(clusterareanum as usize) as c_int;
                let reach = &bot.aasworld.reachability[*reachnum as usize];
                t += AAS_AreaTravelTime(bot, areanum, *origin, reach.start);
            }
            if besttime == 0 || t < besttime {
                bestreachnum = *reachnum;
                besttime = t;
            }
        }
        if bestreachnum < 0 {
            return qfalse;
        }
        *reachnum = bestreachnum;
        *traveltime = besttime as c_int;
        qtrue
    }
}

/// Raven `AAS_AreaTravelTimeToGoalArea`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1750-1759`
pub fn AAS_AreaTravelTimeToGoalArea(
    bot: &mut BotLib,
    areanum: c_int,
    origin: vec3_t,
    goalareanum: c_int,
    travelflags: c_int,
) -> c_int {
    let mut traveltime: c_int = 0;
    let mut reachnum: c_int = 0;
    if AAS_AreaRouteToGoalArea(
        bot,
        areanum,
        &origin,
        goalareanum,
        travelflags,
        &mut traveltime,
        &mut reachnum,
    ) != 0
    {
        return traveltime;
    }
    0
}

/// Raven `AAS_AreaReachabilityToGoalArea`.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1766-1775`
pub fn AAS_AreaReachabilityToGoalArea(
    bot: &mut BotLib,
    areanum: c_int,
    origin: vec3_t,
    goalareanum: c_int,
    travelflags: c_int,
) -> c_int {
    let mut traveltime: c_int = 0;
    let mut reachnum: c_int = 0;
    if AAS_AreaRouteToGoalArea(
        bot,
        areanum,
        &origin,
        goalareanum,
        travelflags,
        &mut traveltime,
        &mut reachnum,
    ) != 0
    {
        return reachnum;
    }
    0
}

/// Raven `AAS_CreateAllRoutingCache` — exhaustively pre-warms the routing
/// cache between all reachable area pairs.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:870-888`
pub fn AAS_CreateAllRoutingCache(bot: &mut BotLib) {
    bot.aasworld.initialized = qtrue;
    bot.botimport
        .Print(PRT_MESSAGE, "AAS_CreateAllRoutingCache\n");
    for i in 1..bot.aasworld.numareas {
        if crate::be_aas_reach_fns::AAS_AreaReachability(bot, i) == 0 {
            continue;
        }
        for j in 1..bot.aasworld.numareas {
            if i == j {
                continue;
            }
            if crate::be_aas_reach_fns::AAS_AreaReachability(bot, j) == 0 {
                continue;
            }
            let _t = AAS_AreaTravelTimeToGoalArea(
                bot,
                i,
                bot.aasworld.areas[i as usize].center,
                j,
                TFL_DEFAULT,
            );
        }
    }
    bot.aasworld.initialized = qfalse;
}

/// Raven `AAS_PredictRoute` — walks the reachability chain from `areanum`
/// toward `goalareanum`, stopping early on any of the requested stop events.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1783-1885`
pub fn AAS_PredictRoute(
    bot: &mut BotLib,
    route: *mut aas_predictroute_s,
    areanum: c_int,
    origin: vec3_t,
    goalareanum: c_int,
    travelflags: c_int,
    maxareas: c_int,
    maxtime: c_int,
    stopevent: c_int,
    stopcontents: c_int,
    stoptfl: c_int,
    stopareanum: c_int,
) -> c_int {
    unsafe {
        (*route).stopevent = RSE_NONE;
        (*route).endarea = goalareanum;
        (*route).endcontents = 0;
        (*route).endtravelflags = 0;
        (*route).endpos = origin;
        (*route).time = 0;

        let mut curareanum = areanum;
        let mut curorigin = origin;

        let mut i = 0;
        while curareanum != goalareanum
            && (maxareas == 0 || i < maxareas)
            && i < bot.aasworld.numareas
        {
            let reachnum = AAS_AreaReachabilityToGoalArea(
                bot,
                curareanum,
                curorigin,
                goalareanum,
                travelflags,
            );
            if reachnum == 0 {
                (*route).stopevent = RSE_NOROUTE;
                return qfalse;
            }
            let reach = bot.aasworld.reachability[reachnum as usize];

            if stopevent & RSE_USETRAVELTYPE != 0 {
                if AAS_TravelFlagForType_inline(bot, reach.traveltype) & stoptfl != 0 {
                    (*route).stopevent = RSE_USETRAVELTYPE;
                    (*route).endarea = curareanum;
                    (*route).endcontents = bot.aasworld.areasettings[curareanum as usize].contents;
                    (*route).endtravelflags = AAS_TravelFlagForType_inline(bot, reach.traveltype);
                    (*route).endpos = reach.start;
                    return qtrue;
                }
                if AAS_AreaContentsTravelFlags_inline(bot, reach.areanum) & stoptfl != 0 {
                    (*route).stopevent = RSE_USETRAVELTYPE;
                    (*route).endarea = reach.areanum;
                    (*route).endcontents =
                        bot.aasworld.areasettings[reach.areanum as usize].contents;
                    (*route).endtravelflags =
                        AAS_AreaContentsTravelFlags_inline(bot, reach.areanum);
                    (*route).endpos = reach.end;
                    (*route).time += AAS_AreaTravelTime(bot, areanum, origin, reach.start) as c_int;
                    (*route).time += reach.traveltime as c_int;
                    return qtrue;
                }
            }
            let reachareas = bot.aasworld.reachabilityareas[reachnum as usize];
            for j in 0..(reachareas.numareas + 1) {
                let testareanum = if j >= reachareas.numareas {
                    reach.areanum
                } else {
                    *bot.aasworld
                        .reachabilityareaindex
                        .add((reachareas.firstarea + j) as usize)
                };
                if stopevent & RSE_ENTERCONTENTS != 0
                    && bot.aasworld.areasettings[testareanum as usize].contents & stopcontents != 0
                {
                    (*route).stopevent = RSE_ENTERCONTENTS;
                    (*route).endarea = testareanum;
                    (*route).endcontents = bot.aasworld.areasettings[testareanum as usize].contents;
                    (*route).endpos = reach.end;
                    (*route).time += AAS_AreaTravelTime(bot, areanum, origin, reach.start) as c_int;
                    (*route).time += reach.traveltime as c_int;
                    return qtrue;
                }
                if stopevent & RSE_ENTERAREA != 0 && testareanum == stopareanum {
                    (*route).stopevent = RSE_ENTERAREA;
                    (*route).endarea = testareanum;
                    (*route).endcontents = bot.aasworld.areasettings[testareanum as usize].contents;
                    (*route).endpos = reach.start;
                    return qtrue;
                }
            }

            (*route).time += AAS_AreaTravelTime(bot, areanum, origin, reach.start) as c_int;
            (*route).time += reach.traveltime as c_int;
            (*route).endarea = reach.areanum;
            (*route).endcontents = bot.aasworld.areasettings[reach.areanum as usize].contents;
            (*route).endtravelflags = AAS_TravelFlagForType_inline(bot, reach.traveltype);
            (*route).endpos = reach.end;

            curareanum = reach.areanum;
            curorigin = reach.end;

            if maxtime != 0 && (*route).time > maxtime {
                break;
            }
            i += 1;
        }
        if curareanum != goalareanum {
            return qfalse;
        }
        qtrue
    }
}

/// Raven `AAS_RandomGoalArea` — picks a random reachable goal area at least a
/// minimum ground-face size away, not too close to a wall drop-off.
///
/// Source: `oracle/codemp/botlib/be_aas_route.cpp:1984-2032`
pub fn AAS_RandomGoalArea(
    common: &mut Common,
    bot: &mut BotLib,
    areanum: c_int,
    travelflags: c_int,
    goalareanum: *mut c_int,
    goalorigin: *mut vec3_t,
) -> c_int {
    unsafe {
        if crate::be_aas_reach_fns::AAS_AreaReachability(bot, areanum) == 0 {
            return qfalse;
        }
        // `random()` (libc, [0,1) via ruling 21's rand family): the engine LCG
        // on `common`.
        // PORT-NOTE(qrand-field): the `QRand` field name on `Common` is
        // pinned when the type lands (ruling 21); `common.qrand` stands in.
        let mut n = (bot.aasworld.numareas as f32 * common.qrand.flrand(0.0, 1.0)) as c_int;
        for _ in 0..bot.aasworld.numareas {
            if n <= 0 {
                n = 1;
            }
            if n >= bot.aasworld.numareas {
                n = 1;
            }
            if crate::be_aas_reach_fns::AAS_AreaReachability(bot, n) != 0 {
                let t = AAS_AreaTravelTimeToGoalArea(
                    bot,
                    areanum,
                    bot.aasworld.areas[areanum as usize].center,
                    n,
                    travelflags,
                );
                if t > 0 {
                    if crate::be_aas_reach_fns::AAS_AreaSwim(bot, n) != 0 {
                        *goalareanum = n;
                        *goalorigin = bot.aasworld.areas[n as usize].center;
                        return qtrue;
                    }
                    let mut start = bot.aasworld.areas[n as usize].center;
                    if crate::be_aas_sample_fns::AAS_PointAreaNum(bot, start) == 0 {
                        let msg = std::ffi::CString::new(format!(
                            "area {} center {} {} {} in solid?",
                            n, start[0], start[1], start[2]
                        ))
                        .unwrap_or_default();
                        crate::l_log_fns::Log_Write(bot, msg.as_ptr() as *mut std::os::raw::c_char);
                    }
                    let mut end = start;
                    end[2] -= 300.0;
                    let trace = crate::be_aas_sample_fns::AAS_TraceClientBBox(
                        bot,
                        start,
                        end,
                        PRESENCE_CROUCH,
                        -1,
                    );
                    if trace.startsolid == qfalse
                        && trace.fraction < 1.0
                        && crate::be_aas_sample_fns::AAS_PointAreaNum(bot, trace.endpos) == n
                    {
                        if crate::be_aas_reach_fns::AAS_AreaGroundFaceArea(bot, n) > 300.0 {
                            *goalareanum = n;
                            *goalorigin = trace.endpos;
                            return qtrue;
                        }
                    }
                    let _ = start;
                }
            }
            n += 1;
        }
        qfalse
    }
}
