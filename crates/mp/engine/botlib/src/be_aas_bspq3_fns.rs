#![allow(
    non_camel_case_types,
    non_snake_case,
    unused_variables,
    unused_assignments
)]

//! Function bodies for Raven's `be_aas_bspq3.cpp` (the BSP-side AAS shim:
//! trace/point-contents/PVS/PHS forwarding to `botimport`, the BSP entity
//! epair store, and the dead-stub light/box-entity queries).
//!
//! Ported per the engine C-track packets (`botlib__0395`..`botlib__1915`).
//! Source: `oracle/codemp/botlib/be_aas_bspq3.cpp`.
//!
//! DESTINATION NOTE: the packet order named
//! `crates/mp/engine/botlib/src/be_aas_bspq3.rs`, but `be_aas_bspq3` already
//! exists as a directory module (consts-only) — `_fns` escape per
//! `_PREAMBLE.md`'s destination rule, matching the sibling
//! `be_aas_cluster_fns.rs`/`be_aas_entity.rs` convention.
//!
//! PORT-NOTE(BotLib): `BotLib` is the synthesized botlib aggregate (per
//! `_PREAMBLE.md`'s state-receiver table) — not yet defined anywhere in the
//! tree, matching every sibling `*_fns.rs` file in this crate that already
//! references `bot: &mut BotLib` ahead of its landing. Reported in
//! missing_symbols.
//!
//! PORT-NOTE(bsp_t family): Raven's `bsp_t`/`bsp_entity_t`/`bsp_epair_t`
//! (be_aas_bspq3.cpp:38-70, the BSP-entity epair store backing `bspworld`)
//! have no rosetta row yet — referenced here as `bot.bspworld` /
//! `bsp_entity_t` / `bsp_epair_t` exactly as the packets resolve them.
//! Reported in missing_symbols.
//!
//! PORT-NOTE(unsafe): raw-pointer epair-list walks and `botimport` fn-ptr
//! calls are confined in `unsafe` per porting-rules §D11, matching the
//! sibling `be_aas_entity.rs`/`be_aas_cluster_fns.rs` convention. Input
//! `vec3_t` args are shadowed with `let mut x = x;` only to hand a `&mut`
//! to the `botimport` callbacks (whose fn-ptr type is `*mut vec3_t` though
//! they read the arg); true out-params take `*mut vec3_t`.
//!
//! PORT-NOTE(callee-signatures): `ScriptError`/`FreeScript`/
//! `GetClearedHunkMemory`/`GetHunkMemory`/`LoadScriptMemory`/
//! `PS_ExpectTokenType`/`PS_ReadToken`/`SetScriptFlags`/`StripDoubleQuotes`
//! are ported in the sibling `l_script_fns.rs`/`l_memory` packets outside
//! this shard; forward-declared below with their faithful resolved shapes,
//! matching the `l_script_fns.rs`/`be_aas_cluster_fns.rs` forward-decl
//! convention. Reported in missing_symbols (not linked into this file).

use core::ffi::c_char;
use core::ffi::c_int;
use core::ffi::c_ulong;

use mp_qshared::shared::{qboolean, qfalse, qtrue};

use mp_qshared::common::mp::botlib::bsp_trace_s::bsp_trace_t;
use mp_qshared::common::mp::botlib::print_type::PRT_MESSAGE;
use mp_qshared::shared::vec3_t;

use crate::be_aas_bsp::be_aas_bsp_consts::MAX_EPAIRKEY;
use crate::be_aas_bspq3::be_aas_bspq3_cpp_consts::MAX_BSPENTITIES;
use crate::be_aas_def::bsp_link_s::bsp_link_t;
use crate::l_script::consts::{SCFL_NOSTRINGESCAPECHARS, SCFL_NOSTRINGWHITESPACES, TT_STRING};
use crate::l_script::token_s::token_t;
use mp_qshared::common::mp::botlib::botlib_error::BLERR_NOERROR;

use crate::BotLib;

// ---------------------------------------------------------------------
// Externally-ported callees this file reaches: these already have real
// (non-extern) definitions elsewhere in the crate (`l_script_fns.rs`/
// `l_memory_fns.rs`) and `mp_engine_qcommon::common_fns` (`Com_Memcpy`/
// `Com_Memset`), so they are imported rather than forward-declared via
// `extern "Rust"` — `ScriptError` in particular is a real variadic Rust fn,
// which `extern "Rust"` cannot express (E0045).
// ---------------------------------------------------------------------
use mp_engine_qcommon::common_fns::{Com_Memcpy, Com_Memset};

use crate::l_memory_fns::{FreeMemory, GetClearedHunkMemory, GetHunkMemory};
use crate::l_script_fns::{
    FreeScript, LoadScriptMemory, PS_ExpectTokenType, PS_ReadToken, ScriptError, SetScriptFlags,
    StripDoubleQuotes,
};

/// Raven `AAS_Trace` — forward a bbox trace to the engine's `botimport`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:130-135`
pub fn AAS_Trace(
    bot: &mut BotLib,
    start: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    end: vec3_t,
    passent: c_int,
    contentmask: c_int,
) -> bsp_trace_t {
    unsafe {
        let mut start = start;
        let mut mins = mins;
        let mut maxs = maxs;
        let mut end = end;
        let mut bsptrace = core::mem::zeroed::<bsp_trace_t>();
        (bot.botimport.Trace.unwrap())(
            &mut bsptrace,
            &mut start,
            &mut mins,
            &mut maxs,
            &mut end,
            passent,
            contentmask,
        );
        bsptrace
    }
}

/// Raven `AAS_PointContents` — forward to `botimport`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:143-146`
pub fn AAS_PointContents(bot: &mut BotLib, point: vec3_t) -> c_int {
    unsafe {
        let mut point = point;
        (bot.botimport.PointContents.unwrap())(&mut point)
    }
}

/// Raven `AAS_inPVS` (Raven's closing comment says `AAS_InPVS`) — forward to
/// `botimport`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:174-177`
pub fn AAS_inPVS(bot: &mut BotLib, p1: vec3_t, p2: vec3_t) -> qboolean {
    unsafe {
        let mut p1 = p1;
        let mut p2 = p2;
        (bot.botimport.inPVS.unwrap())(&mut p1, &mut p2) as qboolean
    }
}

/// Raven `AAS_inPHS` — always visible/audible (PHS is not implemented).
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:185-188`
pub fn AAS_inPHS(p1: vec3_t, p2: vec3_t) -> qboolean {
    qtrue
}

/// Raven `AAS_BSPModelMinsMaxsOrigin` — forward to `botimport`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:195-198`
pub fn AAS_BSPModelMinsMaxsOrigin(
    bot: &mut BotLib,
    modelnum: c_int,
    angles: vec3_t,
    mins: *mut vec3_t,
    maxs: *mut vec3_t,
    origin: *mut vec3_t,
) {
    unsafe {
        let mut angles = angles;
        (bot.botimport.BSPModelMinsMaxsOrigin.unwrap())(
            modelnum,
            &mut angles,
            mins,
            maxs,
            origin,
        );
    }
}

/// Raven `AAS_UnlinkFromBSPLeaves` — no-op (dead BSP-leaf linking stub).
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:206-208`
pub fn AAS_UnlinkFromBSPLeaves(leaves: *mut bsp_link_t) {}

/// Raven `AAS_BSPLinkEntity` — no-op stub, always returns `NULL`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:215-218`
pub fn AAS_BSPLinkEntity(
    absmins: vec3_t,
    absmaxs: vec3_t,
    entnum: c_int,
    modelnum: c_int,
) -> *mut bsp_link_t {
    core::ptr::null_mut()
}

/// Raven `AAS_BoxEntities` — no-op stub, always returns 0.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:225-228`
pub fn AAS_BoxEntities(
    absmins: vec3_t,
    absmaxs: vec3_t,
    list: *mut c_int,
    maxcount: c_int,
) -> c_int {
    0
}

/// Raven `AAS_NextBSPEntity`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:235-240`
pub fn AAS_NextBSPEntity(bot: &mut BotLib, ent: c_int) -> c_int {
    let ent = ent + 1;
    if ent >= 1 && ent < bot.bspworld.numentities {
        return ent;
    }
    0
}

/// Raven `AAS_BSPEntityInRange`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:247-255`
pub fn AAS_BSPEntityInRange(bot: &mut BotLib, ent: c_int) -> c_int {
    unsafe {
        if ent <= 0 || ent >= bot.bspworld.numentities {
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"bsp entity out of range\n".as_ptr() as *mut c_char,
            );
            return qfalse;
        }
        qtrue
    }
}

/// Raven `AAS_BSPTraceLight` — no-op stub, always returns 0.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:433-436`
pub fn AAS_BSPTraceLight(
    start: vec3_t,
    end: vec3_t,
    endpos: vec3_t,
    red: *mut c_int,
    green: *mut c_int,
    blue: *mut c_int,
) -> c_int {
    0
}

/// Raven `AAS_EntityCollision`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:153-166`
pub fn AAS_EntityCollision(
    bot: &mut BotLib,
    entnum: c_int,
    start: vec3_t,
    boxmins: vec3_t,
    boxmaxs: vec3_t,
    end: vec3_t,
    contentmask: c_int,
    trace: *mut bsp_trace_t,
) -> qboolean {
    unsafe {
        let mut start = start;
        let mut boxmins = boxmins;
        let mut boxmaxs = boxmaxs;
        let mut end = end;
        let mut enttrace = core::mem::zeroed::<bsp_trace_t>();

        (bot.botimport.EntityTrace.unwrap())(
            &mut enttrace,
            &mut start,
            &mut boxmins,
            &mut boxmaxs,
            &mut end,
            entnum,
            contentmask,
        );
        if enttrace.fraction < (*trace).fraction {
            Com_Memcpy(
                trace as *mut (),
                &enttrace as *const bsp_trace_t as *const (),
                core::mem::size_of::<bsp_trace_t>(),
            );
            return qtrue;
        }
        qfalse
    }
}

/// Raven `AAS_ValueForBSPEpairKey` (Raven's closing comment says
/// `AAS_FindBSPEpair`).
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:262-278`
pub fn AAS_ValueForBSPEpairKey(
    bot: &mut BotLib,
    ent: c_int,
    key: *mut c_char,
    value: *mut c_char,
    size: c_int,
) -> c_int {
    unsafe {
        *value = 0;
        if AAS_BSPEntityInRange(bot, ent) == 0 {
            return qfalse;
        }
        let mut epair = bot.bspworld.entities[ent as usize].epairs;
        while !epair.is_null() {
            if libc::strcmp((*epair).key, key) == 0 {
                libc::strncpy(value, (*epair).value, (size - 1) as usize);
                *value.offset((size - 1) as isize) = 0;
                return qtrue;
            }
            epair = (*epair).next;
        }
        qfalse
    }
}

/// Raven `AAS_FreeBSPEntities`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:336-355`
pub fn AAS_FreeBSPEntities(bot: &mut BotLib) {
    unsafe {
        for i in 1..bot.bspworld.numentities {
            let ent =
                &mut bot.bspworld.entities[i as usize] as *mut crate::be_aas_bspq3::bsp_entity_t;
            let mut epair = (*ent).epairs;
            while !epair.is_null() {
                let nextepair = (*epair).next;
                if !(*epair).key.is_null() {
                    FreeMemory(bot, (*epair).key as *mut ());
                }
                if !(*epair).value.is_null() {
                    FreeMemory(bot, (*epair).value as *mut ());
                }
                FreeMemory(bot, epair as *mut ());
                epair = nextepair;
            }
        }
        bot.bspworld.numentities = 0;
    }
}

/// Raven `AAS_VectorForBSPEpairKey`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:285-299`
pub fn AAS_VectorForBSPEpairKey(
    bot: &mut BotLib,
    ent: c_int,
    key: *mut c_char,
    v: *mut vec3_t,
) -> c_int {
    unsafe {
        let mut buf = [0 as c_char; MAX_EPAIRKEY as usize];

        (*v)[0] = 0.0;
        (*v)[1] = 0.0;
        (*v)[2] = 0.0;
        if AAS_ValueForBSPEpairKey(bot, ent, key, buf.as_mut_ptr(), MAX_EPAIRKEY) == 0 {
            return qfalse;
        }
        //scanf into doubles, then assign, so it is vec_t size independent
        let (mut v1, mut v2, mut v3): (f64, f64, f64) = (0.0, 0.0, 0.0);
        libc::sscanf(
            buf.as_ptr(),
            c"%lf %lf %lf".as_ptr(),
            &mut v1,
            &mut v2,
            &mut v3,
        );
        (*v)[0] = v1 as f32;
        (*v)[1] = v2 as f32;
        (*v)[2] = v3 as f32;
        qtrue
    }
}

/// Raven `AAS_FloatForBSPEpairKey`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:306-314`
pub fn AAS_FloatForBSPEpairKey(
    bot: &mut BotLib,
    ent: c_int,
    key: *mut c_char,
    value: *mut f32,
) -> c_int {
    unsafe {
        let mut buf = [0 as c_char; MAX_EPAIRKEY as usize];

        *value = 0.0;
        if AAS_ValueForBSPEpairKey(bot, ent, key, buf.as_mut_ptr(), MAX_EPAIRKEY) == 0 {
            return qfalse;
        }
        *value = libc::atof(buf.as_ptr()) as f32;
        qtrue
    }
}

/// Raven `AAS_IntForBSPEpairKey`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:321-329`
pub fn AAS_IntForBSPEpairKey(
    bot: &mut BotLib,
    ent: c_int,
    key: *mut c_char,
    value: *mut c_int,
) -> c_int {
    unsafe {
        let mut buf = [0 as c_char; MAX_EPAIRKEY as usize];

        *value = 0;
        if AAS_ValueForBSPEpairKey(bot, ent, key, buf.as_mut_ptr(), MAX_EPAIRKEY) == 0 {
            return qfalse;
        }
        *value = libc::atoi(buf.as_ptr());
        qtrue
    }
}

/// Raven `AAS_DumpBSPData`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:443-453`
pub fn AAS_DumpBSPData(bot: &mut BotLib) {
    AAS_FreeBSPEntities(bot);

    if !bot.bspworld.dentdata.is_null() {
        FreeMemory(bot, bot.bspworld.dentdata as *mut ());
    }
    bot.bspworld.dentdata = core::ptr::null_mut();
    bot.bspworld.entdatasize = 0;
    //
    bot.bspworld.loaded = qfalse;
    Com_Memset(
        &mut bot.bspworld as *mut _ as *mut (),
        0,
        core::mem::size_of_val(&bot.bspworld),
    );
}

/// Raven `AAS_ParseBSPEntities`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:362-426`
pub fn AAS_ParseBSPEntities(bot: &mut BotLib) {
    unsafe {
        let mut token = core::mem::zeroed::<token_t>();

        let script = LoadScriptMemory(
            bot,
            bot.bspworld.dentdata,
            bot.bspworld.entdatasize,
            c"entdata".as_ptr() as *mut c_char,
        );
        SetScriptFlags(
            script,
            SCFL_NOSTRINGWHITESPACES | SCFL_NOSTRINGESCAPECHARS, //SCFL_PRIMITIVE);
        );

        bot.bspworld.numentities = 1;

        while PS_ReadToken(bot, script, &mut token) != 0 {
            if libc::strcmp(token.string.as_ptr(), c"{".as_ptr()) != 0 {
                // PORT-NOTE(variadic): reproduces Raven's `vsprintf`-into-buffer step
                // before forwarding to `ScriptError` (see l_script_fns.rs script_error!).
                let mut __se_text = [0 as c_char; 1024];
                libc::sprintf(
                    __se_text.as_mut_ptr(),
                    c"invalid %s\n".as_ptr(),
                    token.string.as_ptr(),
                );
                ScriptError(bot, script, __se_text.as_ptr());
                AAS_FreeBSPEntities(bot);
                FreeScript(bot, script);
                return;
            }
            if bot.bspworld.numentities >= MAX_BSPENTITIES {
                bot.botimport.Print.unwrap()(
                    PRT_MESSAGE,
                    c"too many entities in BSP file\n".as_ptr() as *mut c_char,
                );
                break;
            }
            let ent = &mut bot.bspworld.entities[bot.bspworld.numentities as usize]
                as *mut crate::be_aas_bspq3::bsp_entity_t;
            bot.bspworld.numentities += 1;
            (*ent).epairs = core::ptr::null_mut();
            while PS_ReadToken(bot, script, &mut token) != 0 {
                if libc::strcmp(token.string.as_ptr(), c"}".as_ptr()) == 0 {
                    break;
                }
                let epair = GetClearedHunkMemory(
                    bot,
                    core::mem::size_of::<crate::be_aas_bspq3::bsp_epair_t>() as c_ulong,
                ) as *mut crate::be_aas_bspq3::bsp_epair_t;
                (*epair).next = (*ent).epairs;
                (*ent).epairs = epair;
                if token.r#type != TT_STRING {
                    let mut __se_text = [0 as c_char; 1024];
                    libc::sprintf(
                        __se_text.as_mut_ptr(),
                        c"invalid %s\n".as_ptr(),
                        token.string.as_ptr(),
                    );
                    ScriptError(bot, script, __se_text.as_ptr());
                    AAS_FreeBSPEntities(bot);
                    FreeScript(bot, script);
                    return;
                }
                StripDoubleQuotes(token.string.as_mut_ptr());
                (*epair).key =
                    GetHunkMemory(bot, (libc::strlen(token.string.as_ptr()) + 1) as c_ulong)
                        as *mut c_char;
                libc::strcpy((*epair).key, token.string.as_ptr());
                if PS_ExpectTokenType(bot, script, TT_STRING, 0, &mut token) == 0 {
                    AAS_FreeBSPEntities(bot);
                    FreeScript(bot, script);
                    return;
                }
                StripDoubleQuotes(token.string.as_mut_ptr());
                (*epair).value =
                    GetHunkMemory(bot, (libc::strlen(token.string.as_ptr()) + 1) as c_ulong)
                        as *mut c_char;
                libc::strcpy((*epair).value, token.string.as_ptr());
            }
            if libc::strcmp(token.string.as_ptr(), c"}".as_ptr()) != 0 {
                ScriptError(bot, script, c"missing }\n".as_ptr() as *mut c_char);
                AAS_FreeBSPEntities(bot);
                FreeScript(bot, script);
                return;
            }
        }
        FreeScript(bot, script);
    }
}

/// Raven `AAS_LoadBSPFile`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:461-470`
pub fn AAS_LoadBSPFile(bot: &mut BotLib) -> c_int {
    unsafe {
        AAS_DumpBSPData(bot);
        bot.bspworld.entdatasize =
            libc::strlen(bot.botimport.BSPEntityData.unwrap()()) as c_int + 1;
        bot.bspworld.dentdata =
            GetClearedHunkMemory(bot, bot.bspworld.entdatasize as c_ulong) as *mut c_char;
        Com_Memcpy(
            bot.bspworld.dentdata as *mut (),
            bot.botimport.BSPEntityData.unwrap()() as *const (),
            bot.bspworld.entdatasize as usize,
        );
        AAS_ParseBSPEntities(bot);
        bot.bspworld.loaded = qtrue;
        BLERR_NOERROR
    }
}
