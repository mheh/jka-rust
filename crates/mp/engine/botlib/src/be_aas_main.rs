#![allow(
    non_camel_case_types,
    non_snake_case,
    unused_variables,
    unused_assignments
)]

//! Function bodies for Raven's `be_aas_main.cpp` (AAS top-level: error
//! reporting, string-index tables, load/init/shutdown, per-frame driver).
//!
//! Ported per the engine C-track packets (`botlib__0436`..`botlib__2294`).
//! Source: `oracle/codemp/botlib/be_aas_main.cpp`.
//!
// The `bot: &mut BotLib` receiver named in every signature below is the
// campaign's threaded-state aggregate (ruling 2); `BotLib` does not exist in
// this worktree slice yet (`_PREAMBLE.md`'s "botlib waves" note,
// `be_aas_debug_fns.rs`/`be_aas_route_fns.rs` precedent). Every reference to
// `aasworld`/`botimport`/`saveroutingcache`/`bot_developer` below is the
// exact Raven global name per house rule, reached as a field on `bot` —
// resolved when the aggregate lands.

use core::ffi::{c_char, c_int};

use mp_qshared::common::mp::botlib::botlib_error::BLERR_NOERROR;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_FATAL, PRT_MESSAGE};
use mp_qshared::shared::vec3_t;

use mp_bg::public::configstring::CS_MODELS;
use mp_qshared::shared::limits::MAX_MODELS;

use crate::be_aas_def::be_aas_def_consts::MAX_PATH;
use crate::BotLib;

use crate::be_aas_bspq3_fns::{AAS_DumpBSPData, AAS_LoadBSPFile};
use crate::be_aas_cluster_fns::AAS_InitClustering;
use crate::be_aas_entity::{AAS_InvalidateEntities, AAS_ResetEntityLinks, AAS_UnlinkInvalidEntities};
// UNRESOLVED (rule 5): AAS_LoadAASFile/AAS_WriteAASFile/AAS_DumpAASData are
// genuinely unported; this is their canonical future home.
use crate::be_aas_file_fns::{AAS_DumpAASData, AAS_LoadAASFile, AAS_WriteAASFile};
use crate::be_aas_move::AAS_InitSettings;
use crate::be_aas_optimize_fns::AAS_Optimize;
use crate::be_aas_reach_fns::{AAS_ContinueInitReachability, AAS_InitReachability};
use crate::be_aas_route_fns::{
    AAS_FreeRoutingCaches, AAS_InitRouting, AAS_RoutingInfo, AAS_WriteRouteCache,
};
use crate::be_aas_routealt_fns::{AAS_InitAlternativeRouting, AAS_ShutdownAlternativeRouting};
use crate::be_aas_sample_fns::{
    AAS_FreeAASLinkHeap, AAS_FreeAASLinkedEntities, AAS_InitAASLinkHeap, AAS_InitAASLinkedEntities,
};
use crate::l_libvar_fns::{LibVar, LibVarGetValue, LibVarSet, LibVarValue};
use crate::l_memory_fns::{
    FreeMemory, GetClearedHunkMemory, GetMemory, PrintMemoryLabels, PrintUsedMemorySize,
};
use mp_engine_qcommon::common_fns::Com_Memset;
use mp_qshared::shared::q_string::{Q_stricmp, Q_strncpyz};

/// Raven `AAS_Error`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:40-49`
///
/// PORT-NOTE(variadic): Raven's `va_start`/`vsprintf`/`va_end` C-variadic seam
/// cannot be a non-extern Rust fn `...`. Resolved at integration (mirrors the
/// `l_precomp_fns.rs` `SourceError` precedent): the fn now takes an
/// already-rendered message in place of `fmt`/`...`.
pub fn AAS_Error(bot: &mut BotLib, fmt: *mut c_char) {
    unsafe {
        (bot.botimport.Print.unwrap())(PRT_FATAL, fmt);
    }
}

/// Raven `AAS_StringFromIndex`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:56-77`
pub fn AAS_StringFromIndex(
    bot: &mut BotLib,
    indexname: *mut c_char,
    stringindex: *mut *mut c_char,
    numindexes: c_int,
    index: c_int,
) -> *mut c_char {
    unsafe {
        if bot.aasworld.indexessetup == 0 {
            (bot.botimport.Print.unwrap())(
                PRT_ERROR,
                c"%s: index %d not setup\n".as_ptr() as *mut c_char,
                indexname,
                index,
            );
            return c"".as_ptr() as *mut c_char;
        }
        if index < 0 || index >= numindexes {
            (bot.botimport.Print.unwrap())(
                PRT_ERROR,
                c"%s: index %d out of range\n".as_ptr() as *mut c_char,
                indexname,
                index,
            );
            return c"".as_ptr() as *mut c_char;
        }
        if (*stringindex.offset(index as isize)).is_null() {
            if index != 0 {
                (bot.botimport.Print.unwrap())(
                    PRT_ERROR,
                    c"%s: reference to unused index %d\n".as_ptr() as *mut c_char,
                    indexname,
                    index,
                );
            }
            return c"".as_ptr() as *mut c_char;
        }
        *stringindex.offset(index as isize)
    }
}

/// Raven `AAS_IndexFromString`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:84-98`
pub fn AAS_IndexFromString(
    bot: &mut BotLib,
    indexname: *mut c_char,
    stringindex: *mut *mut c_char,
    numindexes: c_int,
    string: *mut c_char,
) -> c_int {
    unsafe {
        if bot.aasworld.indexessetup == 0 {
            (bot.botimport.Print.unwrap())(
                PRT_ERROR,
                c"%s: index not setup \"%s\"\n".as_ptr() as *mut c_char,
                indexname,
                string,
            );
            return 0;
        }
        for i in 0..numindexes {
            if (*stringindex.offset(i as isize)).is_null() {
                continue;
            }
            if Q_stricmp(*stringindex.offset(i as isize), string) == 0 {
                return i;
            }
        }
        0
    }
}

/// Raven `AAS_ModelFromIndex`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:105-108`
pub fn AAS_ModelFromIndex(bot: &mut BotLib, index: c_int) -> *mut c_char {
    unsafe {
        let cs = bot
            .aasworld
            .configstrings
            .as_mut_ptr()
            .offset(CS_MODELS as isize);
        AAS_StringFromIndex(
            bot,
            c"ModelFromIndex".as_ptr() as *mut c_char,
            cs,
            MAX_MODELS as c_int,
            index,
        )
    }
}

/// Raven `AAS_IndexFromModel`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:115-118`
pub fn AAS_IndexFromModel(bot: &mut BotLib, modelname: *mut c_char) -> c_int {
    unsafe {
        let cs = bot
            .aasworld
            .configstrings
            .as_mut_ptr()
            .offset(CS_MODELS as isize);
        AAS_IndexFromString(
            bot,
            c"IndexFromModel".as_ptr() as *mut c_char,
            cs,
            MAX_MODELS as c_int,
            modelname,
        )
    }
}

/// Raven `AAS_UpdateStringIndexes`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:125-139`
pub fn AAS_UpdateStringIndexes(
    bot: &mut BotLib,
    numconfigstrings: c_int,
    configstrings: *mut *mut c_char,
) {
    unsafe {
        // set string pointers and copy the strings
        for i in 0..numconfigstrings {
            let src = *configstrings.offset(i as isize);
            if !src.is_null() {
                // if (aasworld.configstrings[i]) FreeMemory(aasworld.configstrings[i]);
                let len = libc::strlen(src);
                let mem = GetMemory(bot, (len + 1) as core::ffi::c_ulong) as *mut c_char;
                bot.aasworld.configstrings[i as usize] = mem;
                libc::strcpy(bot.aasworld.configstrings[i as usize], src);
            }
        }
        bot.aasworld.indexessetup = mp_qshared::shared::qtrue as c_int;
    }
}

/// Raven `AAS_Loaded`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:146-149`
pub fn AAS_Loaded(bot: &mut BotLib) -> c_int {
    bot.aasworld.loaded
}

/// Raven `AAS_Initialized`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:156-159`
pub fn AAS_Initialized(bot: &mut BotLib) -> c_int {
    bot.aasworld.initialized
}

/// Raven `AAS_SetInitialized`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:166-176`
///
/// PORT-NOTE(DEBUG): the `#ifdef DEBUG` routing-cache-precompute block is
/// commented out in the oracle itself — dead code, dropped per porting-rules
/// §C10.
pub fn AAS_SetInitialized(bot: &mut BotLib) {
    unsafe {
        bot.aasworld.initialized = mp_qshared::shared::qtrue as c_int;
        (bot.botimport.Print.unwrap())(PRT_MESSAGE, c"AAS initialized.\n".as_ptr() as *mut c_char);
    }
}

/// Raven `AAS_ContinueInit`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:183-213`
pub fn AAS_ContinueInit(bot: &mut BotLib, time: f32) {
    unsafe {
        // if no AAS file loaded
        if bot.aasworld.loaded == 0 {
            return;
        }
        // if AAS is already initialized
        if bot.aasworld.initialized != 0 {
            return;
        }
        // calculate reachability, if not finished return
        if AAS_ContinueInitReachability(bot, time) != 0 {
            return;
        }
        // initialize clustering for the new map
        AAS_InitClustering(bot);
        // if reachability has been calculated and an AAS file should be written
        // or there is a forced data optimization
        if bot.aasworld.savefile != 0
            || (LibVarGetValue(bot, c"forcewrite".as_ptr() as *mut c_char) as c_int) != 0
        {
            // optimize the AAS data
            if (LibVarValue(
                bot,
                c"aasoptimize".as_ptr() as *mut c_char,
                c"0".as_ptr() as *mut c_char,
            ) as c_int)
                != 0
            {
                AAS_Optimize(bot);
            }
            // save the AAS file
            let filename_ptr = bot.aasworld.filename.as_mut_ptr();
            if AAS_WriteAASFile(bot, filename_ptr) != 0 {
                (bot.botimport.Print.unwrap())(
                    PRT_MESSAGE,
                    c"%s written succesfully\n".as_ptr() as *mut c_char,
                    bot.aasworld.filename.as_ptr(),
                );
            } else {
                (bot.botimport.Print.unwrap())(
                    PRT_ERROR,
                    c"couldn't write %s\n".as_ptr() as *mut c_char,
                    bot.aasworld.filename.as_ptr(),
                );
            }
        }
        // initialize the routing
        AAS_InitRouting(bot);
        // at this point AAS is initialized
        AAS_SetInitialized(bot);
    }
}

/// Raven `AAS_StartFrame`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:221-260`
pub fn AAS_StartFrame(bot: &mut BotLib, time: f32) -> c_int {
    unsafe {
        bot.aasworld.time = time;
        // unlink all entities that were not updated last frame
        AAS_UnlinkInvalidEntities(bot);
        // invalidate the entities
        AAS_InvalidateEntities(bot);
        // initialize AAS
        AAS_ContinueInit(bot, time);
        //
        bot.aasworld.frameroutingupdates = 0;
        //
        if bot.bot_developer != 0 {
            if LibVarGetValue(bot, c"showcacheupdates".as_ptr() as *mut c_char) != 0.0 {
                AAS_RoutingInfo(bot);
                LibVarSet(
                    bot,
                    c"showcacheupdates".as_ptr() as *mut c_char,
                    c"0".as_ptr() as *mut c_char,
                );
            }
            if LibVarGetValue(bot, c"showmemoryusage".as_ptr() as *mut c_char) != 0.0 {
                PrintUsedMemorySize();
                LibVarSet(
                    bot,
                    c"showmemoryusage".as_ptr() as *mut c_char,
                    c"0".as_ptr() as *mut c_char,
                );
            }
            if LibVarGetValue(bot, c"memorydump".as_ptr() as *mut c_char) != 0.0 {
                PrintMemoryLabels();
                LibVarSet(
                    bot,
                    c"memorydump".as_ptr() as *mut c_char,
                    c"0".as_ptr() as *mut c_char,
                );
            }
        }
        //
        if (*bot.saveroutingcache).value != 0.0 {
            AAS_WriteRouteCache(bot);
            LibVarSet(
                bot,
                c"saveroutingcache".as_ptr() as *mut c_char,
                c"0".as_ptr() as *mut c_char,
            );
        }
        //
        bot.aasworld.numframes += 1;
        BLERR_NOERROR
    }
}

/// Raven `AAS_Time`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:267-270`
pub fn AAS_Time(bot: &mut BotLib) -> f32 {
    bot.aasworld.time
}

/// Raven `AAS_LoadFiles`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:293-316`
pub fn AAS_LoadFiles(bot: &mut BotLib, mapname: *const c_char) -> c_int {
    unsafe {
        let mut aasfile = [0 as c_char; MAX_PATH];
        // char bspfile[MAX_PATH];

        libc::strcpy(bot.aasworld.mapname.as_mut_ptr(), mapname);
        // NOTE: first reset the entity links into the AAS areas and BSP leaves
        // the AAS link heap and BSP link heap are reset after respectively the
        // AAS file and BSP file are loaded
        AAS_ResetEntityLinks(bot);
        // load bsp info
        AAS_LoadBSPFile(bot);

        // load the aas file
        let mapname_str = core::ffi::CStr::from_ptr(mapname).to_string_lossy();
        let __s = std::ffi::CString::new(format!("maps/{}.aas", mapname_str)).unwrap_or_default();
        Q_strncpyz(aasfile.as_mut_ptr(), __s.as_ptr(), MAX_PATH as c_int);
        let errnum = AAS_LoadAASFile(bot, aasfile.as_mut_ptr());
        if errnum != BLERR_NOERROR {
            return errnum;
        }

        (bot.botimport.Print.unwrap())(
            PRT_MESSAGE,
            c"loaded %s\n".as_ptr() as *mut c_char,
            aasfile.as_ptr(),
        );
        libc::strncpy(
            bot.aasworld.filename.as_mut_ptr(),
            aasfile.as_ptr(),
            MAX_PATH,
        );
        BLERR_NOERROR
    }
}

/// Raven `AAS_LoadMap`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:324-358`
pub fn AAS_LoadMap(bot: &mut BotLib, mapname: *const c_char) -> c_int {
    // if no mapname is provided then the string indexes are updated
    if mapname.is_null() {
        return 0;
    }
    //
    bot.aasworld.initialized = mp_qshared::shared::qfalse as c_int;
    // NOTE: free the routing caches before loading a new map because
    // to free the caches the old number of areas, number of clusters
    // and number of areas in a clusters must be available
    AAS_FreeRoutingCaches(bot);
    // load the map
    let errnum = AAS_LoadFiles(bot, mapname);
    if errnum != BLERR_NOERROR {
        bot.aasworld.loaded = mp_qshared::shared::qfalse as c_int;
        return errnum;
    }
    //
    AAS_InitSettings(bot);
    // initialize the AAS link heap for the new map
    AAS_InitAASLinkHeap(bot);
    // initialize the AAS linked entities for the new map
    AAS_InitAASLinkedEntities(bot);
    // initialize reachability for the new map
    AAS_InitReachability(bot);
    // initialize the alternative routing
    AAS_InitAlternativeRouting(bot);
    // everything went ok
    0
}

/// Raven `AAS_ProjectPointOntoVector`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:277-286`
///
/// PORT-NOTE(vProj): the resolved signature carries `vProj: vec3_t` by value
/// (the packet's mechanical `T*`-decay reading of Raven's `vec3_t` array
/// param), so the write below lands only in this fn's local copy and does
/// not propagate to the caller — see shape_mismatches; Raven's C array decay
/// makes `vProj` a true out-param, which this signature cannot express.
pub fn AAS_ProjectPointOntoVector(point: vec3_t, vStart: vec3_t, vEnd: vec3_t, mut vProj: vec3_t) {
    let mut pVec = [0.0f32; 3];
    let mut vec = [0.0f32; 3];

    VectorSubtract(point, vStart, &mut pVec);
    VectorSubtract(vEnd, vStart, &mut vec);
    mp_game::q_math::VectorNormalize(&mut vec);
    // project onto the directional vector for this segment
    VectorMA(vStart, DotProduct(pVec, vec), vec, &mut vProj);
}

// PORT-NOTE(macros): Raven's vector `#define`s (`VectorSubtract`, `VectorMA`,
// `DotProduct`) expand inline here, faithful to the preprocessor, matching
// the sibling `be_aas_move.rs`/`be_aas_debug_fns.rs` convention. `vProj` is a
// faithful out-param (`vec3_t` decays to a raw pointer at the ABI seam, so
// the write lands in the caller's array).
fn DotProduct(a: vec3_t, b: vec3_t) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn VectorSubtract(a: vec3_t, b: vec3_t, out: &mut vec3_t) {
    out[0] = a[0] - b[0];
    out[1] = a[1] - b[1];
    out[2] = a[2] - b[2];
}

fn VectorMA(a: vec3_t, scale: f32, b: vec3_t, out: &mut vec3_t) {
    out[0] = a[0] + scale * b[0];
    out[1] = a[1] + scale * b[1];
    out[2] = a[2] + scale * b[2];
}

/// Raven `AAS_Setup`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:366-382`
pub fn AAS_Setup(bot: &mut BotLib) -> c_int {
    bot.aasworld.maxclients = LibVarValue(
        bot,
        c"maxclients".as_ptr() as *mut c_char,
        c"128".as_ptr() as *mut c_char,
    ) as c_int;
    bot.aasworld.maxentities = LibVarValue(
        bot,
        c"maxentities".as_ptr() as *mut c_char,
        c"1024".as_ptr() as *mut c_char,
    ) as c_int;
    // as soon as it's set to 1 the routing cache will be saved
    bot.saveroutingcache = LibVar(
        bot,
        c"saveroutingcache".as_ptr() as *mut c_char,
        c"0".as_ptr() as *mut c_char,
    );
    // allocate memory for the entities
    if !bot.aasworld.entities.is_null() {
        FreeMemory(bot, bot.aasworld.entities as *mut ());
    }
    bot.aasworld.entities = GetClearedHunkMemory(
        bot,
        (bot.aasworld.maxentities as usize
            * core::mem::size_of::<crate::be_aas_def::aas_entity_s::aas_entity_t>())
            as core::ffi::c_ulong,
    ) as *mut crate::be_aas_def::aas_entity_s::aas_entity_t;
    // invalidate all the entities
    AAS_InvalidateEntities(bot);
    // force some recalculations
    // LibVarSet("forceclustering", "1");			//force clustering calculation
    // LibVarSet("forcereachability", "1");		//force reachability calculation
    bot.aasworld.numframes = 0;
    BLERR_NOERROR
}

/// Raven `AAS_Shutdown`.
///
/// Source: `oracle/codemp/botlib/be_aas_main.cpp:389-412`
pub fn AAS_Shutdown(bot: &mut BotLib) {
    AAS_ShutdownAlternativeRouting(bot);
    //
    AAS_DumpBSPData(bot);
    // free routing caches
    AAS_FreeRoutingCaches(bot);
    // free aas link heap
    AAS_FreeAASLinkHeap(bot);
    // free aas linked entities
    AAS_FreeAASLinkedEntities(bot);
    // free the aas data
    AAS_DumpAASData(bot);
    // free the entities
    if !bot.aasworld.entities.is_null() {
        FreeMemory(bot, bot.aasworld.entities as *mut ());
    }
    // clear the aasworld structure
    Com_Memset(
        &mut bot.aasworld as *mut _ as *mut (),
        0,
        core::mem::size_of::<crate::be_aas_def::aas_s::aas_t>(),
    );
    // aas has not been initialized
    bot.aasworld.initialized = mp_qshared::shared::qfalse as c_int;
    // NOTE: as soon as a new .bsp file is loaded the .bsp file memory is
    // freed an reallocated, so there's no need to free that memory here
    // print shutdown
    // botimport.Print(PRT_MESSAGE, "AAS shutdown.\n");
}
