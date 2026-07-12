#![allow(non_snake_case, non_camel_case_types, clippy::missing_safety_doc)]

//! MP botlib `be_ai_goal.cpp` — bot goal AI (item/roam goal selection, goal
//! stack, level-item tracking, item weight config loading).
//!
//! DESTINATION NOTE: the packet order named
//! `crates/mp/engine/botlib/src/be_ai_goal.rs`, but `be_ai_goal` already
//! exists as a directory module (`be_ai_goal/mod.rs`, types-only) — `.rs` +
//! `/mod.rs` for the same module name cannot coexist, so this file lands at
//! the `_fns` escape per `_PREAMBLE.md`'s destination rule.
//!
//! Source: `oracle/codemp/botlib/be_ai_goal.cpp`

use core::ffi::{c_char, c_int, c_ulong};

use mp_engine_qcommon::common::Common;
use mp_engine_qcommon::common_fns::{Com_Memcpy, Com_Memset};
use mp_qshared::common::mp::botlib::aas_entityinfo_s::aas_entityinfo_t;
use mp_qshared::common::mp::botlib::botlib_error::{
    BLERR_CANNOTLOADITEMCONFIG, BLERR_CANNOTLOADITEMWEIGHTS, BLERR_NOERROR,
};
use mp_qshared::common::mp::botlib::botlib_misc::BOTFILESBASEFOLDER;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_FATAL, PRT_MESSAGE, PRT_WARNING};
use mp_qshared::common::mp::qcommon::bot_goal::{bot_goal_t, GFL_DROPPED, GFL_ITEM, GFL_ROAM};
use mp_qshared::shared::limits::MAX_CLIENTS;
use mp_qshared::shared::surface_flags::{CONTENTS_PLAYERCLIP, CONTENTS_SOLID, CONTENTS_WATER};
use mp_qshared::shared::vec3_t;
use mp_qshared::shared::{qfalse, qtrue};

use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::gametype::{GT_SINGLE_PLAYER, GT_TEAM};

use crate::aasfile::presence_type::PRESENCE_NORMAL;
use crate::be_aas_bsp::be_aas_bsp_consts::MAX_EPAIRKEY;
use crate::be_ai_goal::be_ai_goal_cpp_consts::{
    AVOID_DEFAULT_TIME, AVOID_DROPPED_TIME, AVOID_MINIMUM_TIME, IFL_NOTBOT, IFL_NOTFREE,
    IFL_NOTSINGLE, IFL_NOTTEAM, IFL_ROAM, TRAVELTIME_SCALE,
};
use crate::be_ai_goal::bot_goalstate_s::{bot_goalstate_t, MAX_AVOIDGOALS, MAX_GOALSTACK};
use crate::be_ai_goal::itemconfig_s::itemconfig_t;
use crate::be_ai_goal::iteminfo_s::iteminfo_t;
use crate::be_ai_goal::levelitem_s::levelitem_t;
use crate::be_ai_weight::weightconfig_s::weightconfig_t;
use crate::l_script::consts::TT_STRING;
use crate::l_script::token_s::token_t;
use crate::l_struct::structdef_s::structdef_t;
use crate::BotLib;
use crate::{campspot_t, maplocation_t};

use crate::be_aas_bspq3_fns::{
    AAS_FloatForBSPEpairKey, AAS_IntForBSPEpairKey, AAS_NextBSPEntity, AAS_PointContents,
    AAS_Trace, AAS_ValueForBSPEpairKey, AAS_VectorForBSPEpairKey,
};
use crate::be_aas_entity::{AAS_EntityInfo, AAS_EntityModelindex, AAS_EntityType, AAS_NextEntity};
use crate::be_aas_main::{AAS_Loaded, AAS_Time};
use crate::be_aas_move::AAS_DropToFloor;
use crate::be_aas_reach_fns::{
    AAS_AreaJumpPad, AAS_AreaReachability, AAS_BestReachableArea, AAS_BestReachableFromJumpPadArea,
};
use crate::be_aas_route_fns::AAS_AreaTravelTimeToGoalArea;
use crate::be_aas_sample_fns::{AAS_PointAreaNum, AAS_PresenceTypeBoundingBox};
use crate::be_ai_move_fns::BotReachabilityArea;
use crate::be_ai_weight_fns::{
    EvolveWeightConfig, FindFuzzyWeight, FreeWeightConfig, FuzzyWeightUndecided,
    InterbreedWeightConfigs, ReadWeightConfig,
};
use crate::l_libvar_fns::{LibVar, LibVarSet, LibVarString, LibVarValue};
use crate::l_log_fns::Log_Write;
use crate::l_memory_fns::{FreeMemory, GetClearedHunkMemory, GetClearedMemory};
use crate::l_precomp_fns::{
    FreeSource, LoadSourceFile, PC_ExpectTokenType, PC_ReadToken, PC_SetBaseFolder, SourceError,
};
use crate::l_script_fns::StripDoubleQuotes;
use crate::l_struct_fns::ReadStructure;
use mp_qshared::shared::q_math::VectorLength;

// helper: vector arithmetic used inline below (mirrors the qshared q_math
// primitives; ported bodies transcribed inline to avoid a spurious edge for
// simple 3-float ops not listed as callees by any packet).
#[inline]
fn vec_sub(a: vec3_t, b: vec3_t) -> vec3_t {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
#[inline]
fn vec_add(a: vec3_t, b: vec3_t) -> vec3_t {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
#[inline]
fn vec_scale(a: vec3_t, s: f32) -> vec3_t {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// Raven `BotGoalStateFromHandle`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:186-199`
pub fn BotGoalStateFromHandle(bot: &mut BotLib, handle: c_int) -> *mut bot_goalstate_t {
    if handle <= 0 || handle > MAX_CLIENTS as c_int {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"goal state handle %d out of range\n".as_ptr() as *mut c_char,
                handle,
            );
        }
        return core::ptr::null_mut();
    }
    if bot.botgoalstates[handle as usize].is_null() {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"invalid goal state %d\n".as_ptr() as *mut c_char,
                handle,
            );
        }
        return core::ptr::null_mut();
    }
    bot.botgoalstates[handle as usize]
}

/// Raven `FreeLevelItem`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:399-403`
pub fn FreeLevelItem(bot: &mut BotLib, li: *mut levelitem_t) {
    unsafe {
        (*li).next = bot.freelevelitems;
    }
    bot.freelevelitems = li;
}

/// Raven `AddLevelItemToList`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:410-416`
pub fn AddLevelItemToList(bot: &mut BotLib, li: *mut levelitem_t) {
    unsafe {
        if !bot.levelitems.is_null() {
            (*bot.levelitems).prev = li;
        }
        (*li).prev = core::ptr::null_mut();
        (*li).next = bot.levelitems;
    }
    bot.levelitems = li;
}

/// Raven `RemoveLevelItemFromList`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:423-428`
pub fn RemoveLevelItemFromList(bot: &mut BotLib, li: *mut levelitem_t) {
    unsafe {
        if !(*li).prev.is_null() {
            (*(*li).prev).next = (*li).next;
        } else {
            bot.levelitems = (*li).next;
        }
        if !(*li).next.is_null() {
            (*(*li).next).prev = (*li).prev;
        }
    }
}

/// Raven `BotGoalName`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:663-680`
pub fn BotGoalName(bot: &mut BotLib, number: c_int, name: *mut c_char, size: c_int) {
    if bot.itemconfig.is_null() {
        return;
    }
    unsafe {
        let mut li = bot.levelitems;
        while !li.is_null() {
            if (*li).number == number {
                let iteminfo_idx = (*li).iteminfo as usize;
                let src = (*bot.itemconfig).iteminfo.add(iteminfo_idx);
                libc::strncpy(name, (*src).name.as_ptr(), (size - 1) as usize);
                *name.add((size - 1) as usize) = 0;
                return;
            }
            li = (*li).next;
        }
        libc::strcpy(name, c"".as_ptr());
    }
}

/// Raven `BotGetLevelItemGoal`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:843-889`
pub fn BotGetLevelItemGoal(
    bot: &mut BotLib,
    index: c_int,
    name: *mut c_char,
    goal: *mut bot_goal_t,
) -> c_int {
    if bot.itemconfig.is_null() {
        return -1;
    }
    unsafe {
        let mut li = bot.levelitems;
        if index >= 0 {
            while !li.is_null() {
                if (*li).number == index {
                    li = (*li).next;
                    break;
                }
                li = (*li).next;
            }
        }
        while !li.is_null() {
            let flags = (*li).flags;
            if bot.g_gametype == mp_bg::public::gametype::GT_SINGLE_PLAYER {
                if flags & IFL_NOTSINGLE != 0 {
                    li = (*li).next;
                    continue;
                }
            } else if bot.g_gametype >= GT_TEAM {
                if flags & IFL_NOTTEAM != 0 {
                    li = (*li).next;
                    continue;
                }
            } else if flags & IFL_NOTFREE != 0 {
                li = (*li).next;
                continue;
            }
            if flags & IFL_NOTBOT != 0 {
                li = (*li).next;
                continue;
            }
            let iteminfo_idx = (*li).iteminfo as usize;
            let ii = (*bot.itemconfig).iteminfo.add(iteminfo_idx);
            if mp_game::q_shared::Q_stricmp(name, (*ii).name.as_ptr()) == 0 {
                (*goal).areanum = (*li).goalareanum;
                (*goal).origin = (*li).goalorigin;
                (*goal).entitynum = (*li).entitynum;
                (*goal).mins = (*ii).mins;
                (*goal).maxs = (*ii).maxs;
                (*goal).number = (*li).number;
                (*goal).flags = GFL_ITEM;
                if (*li).timeout != 0.0 {
                    (*goal).flags |= GFL_DROPPED;
                }
                return (*li).number;
            }
            li = (*li).next;
        }
    }
    -1
}

/// Raven `BotGetMapLocationGoal`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:896-914`
pub fn BotGetMapLocationGoal(bot: &mut BotLib, name: *mut c_char, goal: *mut bot_goal_t) -> c_int {
    let mins: vec3_t = [-8.0, -8.0, -8.0];
    let maxs: vec3_t = [8.0, 8.0, 8.0];
    unsafe {
        let mut ml = bot.maplocations;
        while !ml.is_null() {
            if mp_game::q_shared::Q_stricmp((*ml).name.as_ptr() as *mut c_char, name) == 0 {
                (*goal).areanum = (*ml).areanum;
                (*goal).origin = (*ml).origin;
                (*goal).entitynum = 0;
                (*goal).mins = mins;
                (*goal).maxs = maxs;
                return qtrue;
            }
            ml = (*ml).next;
        }
    }
    qfalse
}

/// Raven `BotGetNextCampSpotGoal`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:921-942`
pub fn BotGetNextCampSpotGoal(bot: &mut BotLib, num: c_int, goal: *mut bot_goal_t) -> c_int {
    let mins: vec3_t = [-8.0, -8.0, -8.0];
    let maxs: vec3_t = [8.0, 8.0, 8.0];
    let mut num = num;
    if num < 0 {
        num = 0;
    }
    let mut i = num;
    unsafe {
        let mut cs = bot.campspots;
        while !cs.is_null() {
            i -= 1;
            if i < 0 {
                (*goal).areanum = (*cs).areanum;
                (*goal).origin = (*cs).origin;
                (*goal).entitynum = 0;
                (*goal).mins = mins;
                (*goal).maxs = maxs;
                return num + 1;
            }
            cs = (*cs).next;
        }
    }
    0
}

/// Raven `BotSaveGoalFuzzyLogic`.
///
/// Raven: `//WriteWeightConfig(filename, gs->itemweightconfig);` — commented
/// out in the oracle, kept as-is (no-op body beyond resolving the handle).
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:223-230`
pub fn BotSaveGoalFuzzyLogic(bot: &mut BotLib, goalstate: c_int, _filename: *mut c_char) {
    let _gs = BotGoalStateFromHandle(bot, goalstate);
    //WriteWeightConfig(filename, gs->itemweightconfig);
}

/// Raven `AllocLevelItem`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:378-392`
pub fn AllocLevelItem(bot: &mut BotLib) -> *mut levelitem_t {
    let li = bot.freelevelitems;
    if li.is_null() {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"out of level items\n".as_ptr() as *mut c_char,
            );
        }
        return core::ptr::null_mut();
    }
    unsafe {
        bot.freelevelitems = (*li).next;
        Com_Memset(li as *mut (), 0, core::mem::size_of::<levelitem_t>());
    }
    li
}

/// Raven `BotFreeInfoEntities`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:435-452`
pub fn BotFreeInfoEntities(bot: &mut BotLib) {
    unsafe {
        let mut ml = bot.maplocations;
        while !ml.is_null() {
            let nextml = (*ml).next;
            FreeMemory(bot, ml as *mut ());
            ml = nextml;
        }
    }
    bot.maplocations = core::ptr::null_mut();
    unsafe {
        let mut cs = bot.campspots;
        while !cs.is_null() {
            let nextcs = (*cs).next;
            FreeMemory(bot, cs as *mut ());
            cs = nextcs;
        }
    }
    bot.campspots = core::ptr::null_mut();
}

/// Raven `BotResetAvoidGoals`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:687-695`
pub fn BotResetAvoidGoals(bot: &mut BotLib, goalstate: c_int) {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return;
    }
    unsafe {
        Com_Memset(
            (*gs).avoidgoals.as_mut_ptr() as *mut (),
            0,
            MAX_AVOIDGOALS * core::mem::size_of::<c_int>(),
        );
        Com_Memset(
            (*gs).avoidgoaltimes.as_mut_ptr() as *mut (),
            0,
            MAX_AVOIDGOALS * core::mem::size_of::<f32>(),
        );
    }
}

/// Raven `BotDumpAvoidGoals`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:702-719`
pub fn BotDumpAvoidGoals(bot: &mut BotLib, goalstate: c_int) {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return;
    }
    let mut name = [0 as c_char; 32];
    for i in 0..MAX_AVOIDGOALS {
        unsafe {
            if (*gs).avoidgoaltimes[i] >= AAS_Time(bot) {
                BotGoalName(bot, (*gs).avoidgoals[i], name.as_mut_ptr(), 32);
                let remaining = (*gs).avoidgoaltimes[i] - AAS_Time(bot);
                let __m = std::ffi::CString::new(format!(
                    "avoid goal {}, number {} for {} seconds",
                    std::ffi::CStr::from_ptr(name.as_ptr()).to_string_lossy(),
                    (*gs).avoidgoals[i],
                    remaining as f64,
                ))
                .unwrap_or_default();
                Log_Write(bot, __m.as_ptr() as *mut c_char);
            }
        }
    }
}

/// Raven `BotAddToAvoidGoals`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:726-751`
pub fn BotAddToAvoidGoals(
    bot: &mut BotLib,
    gs: *mut bot_goalstate_t,
    number: c_int,
    avoidtime: f32,
) {
    unsafe {
        for i in 0..MAX_AVOIDGOALS {
            if (*gs).avoidgoals[i] == number {
                (*gs).avoidgoals[i] = number;
                (*gs).avoidgoaltimes[i] = AAS_Time(bot) + avoidtime;
                return;
            }
        }
        for i in 0..MAX_AVOIDGOALS {
            if (*gs).avoidgoaltimes[i] < AAS_Time(bot) {
                (*gs).avoidgoals[i] = number;
                (*gs).avoidgoaltimes[i] = AAS_Time(bot) + avoidtime;
                return;
            }
        }
    }
}

/// Raven `BotRemoveFromAvoidGoals`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:758-774`
pub fn BotRemoveFromAvoidGoals(bot: &mut BotLib, goalstate: c_int, number: c_int) {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return;
    }
    unsafe {
        for i in 0..MAX_AVOIDGOALS {
            if (*gs).avoidgoals[i] == number && (*gs).avoidgoaltimes[i] >= AAS_Time(bot) {
                (*gs).avoidgoaltimes[i] = 0.0;
                return;
            }
        }
    }
}

/// Raven `BotAvoidGoalTime`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:781-797`
pub fn BotAvoidGoalTime(bot: &mut BotLib, goalstate: c_int, number: c_int) -> f32 {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return 0.0;
    }
    unsafe {
        for i in 0..MAX_AVOIDGOALS {
            if (*gs).avoidgoals[i] == number && (*gs).avoidgoaltimes[i] >= AAS_Time(bot) {
                return (*gs).avoidgoaltimes[i] - AAS_Time(bot);
            }
        }
    }
    0.0
}

/// Raven `BotDumpGoalStack`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1166-1179`
pub fn BotDumpGoalStack(bot: &mut BotLib, goalstate: c_int) {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return;
    }
    let mut name = [0 as c_char; 32];
    unsafe {
        let top = (*gs).goalstacktop;
        for i in 1..=top {
            BotGoalName(
                bot,
                (*gs).goalstack[i as usize].number,
                name.as_mut_ptr(),
                32,
            );
            let __m = std::ffi::CString::new(format!(
                "{}: {}",
                i,
                std::ffi::CStr::from_ptr(name.as_ptr()).to_string_lossy(),
            ))
            .unwrap_or_default();
            Log_Write(bot, __m.as_ptr() as *mut c_char);
        }
    }
}

/// Raven `BotPopGoal`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1207-1214`
pub fn BotPopGoal(bot: &mut BotLib, goalstate: c_int) {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return;
    }
    unsafe {
        if (*gs).goalstacktop > 0 {
            (*gs).goalstacktop -= 1;
        }
    }
}

/// Raven `BotEmptyGoalStack`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1221-1228`
pub fn BotEmptyGoalStack(bot: &mut BotLib, goalstate: c_int) {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return;
    }
    unsafe {
        (*gs).goalstacktop = 0;
    }
}

/// Raven `BotGetTopGoal`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1235-1244`
pub fn BotGetTopGoal(bot: &mut BotLib, goalstate: c_int, goal: *mut bot_goal_t) -> c_int {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return qfalse;
    }
    unsafe {
        if (*gs).goalstacktop == 0 {
            return qfalse;
        }
        Com_Memcpy(
            goal as *mut (),
            &(*gs).goalstack[(*gs).goalstacktop as usize] as *const bot_goal_t as *const (),
            core::mem::size_of::<bot_goal_t>(),
        );
    }
    qtrue
}

/// Raven `BotGetSecondGoal`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1251-1260`
pub fn BotGetSecondGoal(bot: &mut BotLib, goalstate: c_int, goal: *mut bot_goal_t) -> c_int {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return qfalse;
    }
    unsafe {
        if (*gs).goalstacktop <= 1 {
            return qfalse;
        }
        Com_Memcpy(
            goal as *mut (),
            &(*gs).goalstack[((*gs).goalstacktop - 1) as usize] as *const bot_goal_t as *const (),
            core::mem::size_of::<bot_goal_t>(),
        );
    }
    qtrue
}

/// Raven `BotTouchingGoal`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1592-1614`
pub fn BotTouchingGoal(bot: &mut BotLib, origin: vec3_t, goal: *mut bot_goal_t) -> c_int {
    let boxmins: vec3_t = [0.0; 3];
    let boxmaxs: vec3_t = [0.0; 3];
    let safety_maxs: vec3_t = [0.0, 0.0, 0.0]; //{4, 4, 10};
    let safety_mins: vec3_t = [0.0, 0.0, 0.0]; //{-4, -4, 0};

    unsafe {
        AAS_PresenceTypeBoundingBox(bot, PRESENCE_NORMAL, boxmins, boxmaxs);
        let mut absmins = vec_sub((*goal).mins, boxmaxs);
        let mut absmaxs = vec_sub((*goal).maxs, boxmins);
        absmins = vec_add(absmins, (*goal).origin);
        absmaxs = vec_add(absmaxs, (*goal).origin);
        //make the box a little smaller for safety
        absmaxs = vec_sub(absmaxs, safety_maxs);
        absmins = vec_sub(absmins, safety_mins);

        for i in 0..3 {
            if origin[i] < absmins[i] || origin[i] > absmaxs[i] {
                return qfalse;
            }
        }
    }
    qtrue
}

/// Raven `BotInterbreedGoalFuzzyLogic`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:206-216`
pub fn BotInterbreedGoalFuzzyLogic(bot: &mut BotLib, parent1: c_int, parent2: c_int, child: c_int) {
    let p1 = BotGoalStateFromHandle(bot, parent1);
    let p2 = BotGoalStateFromHandle(bot, parent2);
    let c = BotGoalStateFromHandle(bot, child);

    unsafe {
        InterbreedWeightConfigs(
            bot,
            (*p1).itemweightconfig,
            (*p2).itemweightconfig,
            (*c).itemweightconfig,
        );
    }
}

/// Raven `BotMutateGoalFuzzyLogic`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:237-244`
pub fn BotMutateGoalFuzzyLogic(
    common: &mut Common,
    bot: &mut BotLib,
    goalstate: c_int,
    _range: f32,
) {
    let gs = BotGoalStateFromHandle(bot, goalstate);

    unsafe {
        EvolveWeightConfig(common, (*gs).itemweightconfig);
    }
}

/// Raven `ItemWeightIndex`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:332-348`
pub fn ItemWeightIndex(
    bot: &mut BotLib,
    iwc: *mut weightconfig_t,
    ic: *mut itemconfig_t,
) -> *mut c_int {
    unsafe {
        let numiteminfo = (*ic).numiteminfo;
        //initialize item weight index
        let index = GetClearedMemory(
            bot,
            (core::mem::size_of::<c_int>() as c_int * numiteminfo) as c_ulong,
        ) as *mut c_int;

        for i in 0..numiteminfo {
            let classname = (*(*ic).iteminfo.add(i as usize)).classname.as_ptr() as *mut c_char;
            let w = FindFuzzyWeight(iwc, classname);
            *index.add(i as usize) = w;
            if w < 0 {
                let __m = std::ffi::CString::new(format!(
                    "item info {} \"{}\" has no fuzzy weight\r\n",
                    i,
                    std::ffi::CStr::from_ptr(classname).to_string_lossy(),
                ))
                .unwrap_or_default();
                Log_Write(bot, __m.as_ptr() as *mut c_char);
            }
        }
        index
    }
}

/// Raven `BotSetAvoidGoalTime`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:804-836`
pub fn BotSetAvoidGoalTime(bot: &mut BotLib, goalstate: c_int, number: c_int, avoidtime: f32) {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return;
    }
    let mut avoidtime = avoidtime;
    if avoidtime < 0.0 {
        if bot.itemconfig.is_null() {
            return;
        }
        unsafe {
            let mut li = bot.levelitems;
            while !li.is_null() {
                if (*li).number == number {
                    let iteminfo_idx = (*li).iteminfo as usize;
                    avoidtime = (*(*bot.itemconfig).iteminfo.add(iteminfo_idx)).respawntime;
                    if avoidtime == 0.0 {
                        avoidtime = AVOID_DEFAULT_TIME as f32;
                    }
                    if avoidtime < AVOID_MINIMUM_TIME as f32 {
                        avoidtime = AVOID_MINIMUM_TIME as f32;
                    }
                    BotAddToAvoidGoals(bot, gs, number, avoidtime);
                    return;
                }
                li = (*li).next;
            }
        }
    } else {
        BotAddToAvoidGoals(bot, gs, number, avoidtime);
    }
}

/// Raven `BotFindEntityForLevelItem`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:949-982`
pub fn BotFindEntityForLevelItem(bot: &mut BotLib, li: *mut levelitem_t) {
    let ic = bot.itemconfig;
    if bot.itemconfig.is_null() {
        return;
    }
    unsafe {
        let mut ent = AAS_NextEntity(bot, 0);
        while ent != 0 {
            //get the model index of the entity
            let modelindex = AAS_EntityModelindex(bot, ent);
            if modelindex != 0 {
                //get info about the entity
                let mut entinfo: aas_entityinfo_t = core::mem::zeroed();
                AAS_EntityInfo(bot, ent, &mut entinfo);
                //if the entity is still moving
                if entinfo.origin[0] == entinfo.lastvisorigin[0]
                    && entinfo.origin[1] == entinfo.lastvisorigin[1]
                    && entinfo.origin[2] == entinfo.lastvisorigin[2]
                {
                    let iteminfo_idx = (*li).iteminfo as usize;
                    if (*(*ic).iteminfo.add(iteminfo_idx)).modelindex == modelindex {
                        //check if the entity is very close
                        let dir = vec_sub((*li).origin, entinfo.origin);
                        if VectorLength(dir) < 30.0 {
                            //found an entity for this level item
                            (*li).entitynum = ent;
                        }
                    }
                }
            }
            ent = AAS_NextEntity(bot, ent);
        }
    }
}

/// Raven `BotPushGoal`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1186-1200`
pub fn BotPushGoal(bot: &mut BotLib, goalstate: c_int, goal: *mut bot_goal_t) {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return;
    }
    unsafe {
        if (*gs).goalstacktop >= MAX_GOALSTACK as c_int - 1 {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"goal heap overflow\n".as_ptr() as *mut c_char,
            );
            BotDumpGoalStack(bot, goalstate);
            return;
        }
        (*gs).goalstacktop += 1;
        Com_Memcpy(
            &mut (*gs).goalstack[(*gs).goalstacktop as usize] as *mut bot_goal_t as *mut (),
            goal as *const (),
            core::mem::size_of::<bot_goal_t>(),
        );
    }
}

/// Raven `BotItemGoalInVisButNotVisible`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1621-1651`
pub fn BotItemGoalInVisButNotVisible(
    bot: &mut BotLib,
    viewer: c_int,
    eye: vec3_t,
    _viewangles: vec3_t,
    goal: *mut bot_goal_t,
) -> c_int {
    unsafe {
        if (*goal).flags & GFL_ITEM == 0 {
            return qfalse;
        }
        let mut middle = vec_add((*goal).mins, (*goal).mins);
        middle = vec_scale(middle, 0.5);
        middle = vec_add((*goal).origin, middle);

        let trace = AAS_Trace(
            bot,
            eye,
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            middle,
            viewer,
            CONTENTS_SOLID,
        );
        //if the goal middle point is visible
        if trace.fraction >= 1.0 {
            //the goal entity number doesn't have to be valid
            //just assume it's valid
            if (*goal).entitynum <= 0 {
                return qfalse;
            }
            //if the entity data isn't valid
            let mut entinfo: aas_entityinfo_t = core::mem::zeroed();
            AAS_EntityInfo(bot, (*goal).entitynum, &mut entinfo);
            //NOTE: for some wacko reason entities are sometimes
            // not updated
            //if (!entinfo.valid) return qtrue;
            if entinfo.ltime < AAS_Time(bot) - 0.5 {
                return qtrue;
            }
        }
    }
    qfalse
}

/// Raven `BotResetGoalState`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1658-1667`
pub fn BotResetGoalState(bot: &mut BotLib, goalstate: c_int) {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return;
    }
    unsafe {
        Com_Memset(
            (*gs).goalstack.as_mut_ptr() as *mut (),
            0,
            MAX_GOALSTACK * core::mem::size_of::<bot_goal_t>(),
        );
        (*gs).goalstacktop = 0;
    }
    BotResetAvoidGoals(bot, goalstate);
}

/// Raven `BotAllocGoalState`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1715-1729`
pub fn BotAllocGoalState(bot: &mut BotLib, client: c_int) -> c_int {
    for i in 1..=MAX_CLIENTS as c_int {
        if bot.botgoalstates[i as usize].is_null() {
            unsafe {
                let mem = GetClearedMemory(bot, core::mem::size_of::<bot_goalstate_t>() as c_ulong)
                    as *mut bot_goalstate_t;
                bot.botgoalstates[i as usize] = mem;
                (*mem).client = client;
            }
            return i;
        }
    }
    0
}

/// Raven `BotInitInfoEntities`.
///
/// Note: Raven's `maplocation_t`/`campspot_t` (`be_ai_goal.cpp:79-91`) have
/// no rosetta row from this packet (its TYPE ROSETTA table was empty) —
/// referenced here by their exact Raven names per the no-stub/no-invented-
/// shim rule; escalated in missing_symbols.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:459-514`
pub fn BotInitInfoEntities(bot: &mut BotLib) {
    BotFreeInfoEntities(bot);

    let mut numlocations = 0;
    let mut numcampspots = 0;
    unsafe {
        let mut ent = AAS_NextBSPEntity(bot, 0);
        while ent != 0 {
            let mut classname = [0 as c_char; MAX_EPAIRKEY as usize];
            if AAS_ValueForBSPEpairKey(
                bot,
                ent,
                c"classname".as_ptr() as *mut c_char,
                classname.as_mut_ptr(),
                MAX_EPAIRKEY,
            ) != 0
            {
                //map locations
                if libc::strcmp(classname.as_ptr(), c"target_location".as_ptr()) == 0 {
                    let ml = GetClearedMemory(bot, core::mem::size_of::<maplocation_t>() as c_ulong)
                        as *mut maplocation_t;
                    AAS_VectorForBSPEpairKey(
                        bot,
                        ent,
                        c"origin".as_ptr() as *mut c_char,
                        (*ml).origin,
                    );
                    AAS_ValueForBSPEpairKey(
                        bot,
                        ent,
                        c"message".as_ptr() as *mut c_char,
                        (*ml).name.as_mut_ptr(),
                        core::mem::size_of_val(&(*ml).name) as c_int,
                    );
                    (*ml).areanum = AAS_PointAreaNum(bot, (*ml).origin);
                    (*ml).next = bot.maplocations;
                    bot.maplocations = ml;
                    numlocations += 1;
                }
                //camp spots
                else if libc::strcmp(classname.as_ptr(), c"info_camp".as_ptr()) == 0 {
                    let cs = GetClearedMemory(bot, core::mem::size_of::<campspot_t>() as c_ulong)
                        as *mut campspot_t;
                    AAS_VectorForBSPEpairKey(
                        bot,
                        ent,
                        c"origin".as_ptr() as *mut c_char,
                        (*cs).origin,
                    );
                    //cs->origin[2] += 16;
                    AAS_ValueForBSPEpairKey(
                        bot,
                        ent,
                        c"message".as_ptr() as *mut c_char,
                        (*cs).name.as_mut_ptr(),
                        core::mem::size_of_val(&(*cs).name) as c_int,
                    );
                    AAS_FloatForBSPEpairKey(
                        bot,
                        ent,
                        c"range".as_ptr() as *mut c_char,
                        &mut (*cs).range,
                    );
                    AAS_FloatForBSPEpairKey(
                        bot,
                        ent,
                        c"weight".as_ptr() as *mut c_char,
                        &mut (*cs).weight,
                    );
                    AAS_FloatForBSPEpairKey(
                        bot,
                        ent,
                        c"wait".as_ptr() as *mut c_char,
                        &mut (*cs).wait,
                    );
                    AAS_FloatForBSPEpairKey(
                        bot,
                        ent,
                        c"random".as_ptr() as *mut c_char,
                        &mut (*cs).random,
                    );
                    (*cs).areanum = AAS_PointAreaNum(bot, (*cs).origin);
                    if (*cs).areanum == 0 {
                        bot.botimport.Print.unwrap()(
                            PRT_MESSAGE,
                            c"camp spot at %1.1f %1.1f %1.1f in solid\n".as_ptr() as *mut c_char,
                            (*cs).origin[0] as f64,
                            (*cs).origin[1] as f64,
                            (*cs).origin[2] as f64,
                        );
                        FreeMemory(bot, cs as *mut ());
                        ent = AAS_NextBSPEntity(bot, ent);
                        continue;
                    }
                    (*cs).next = bot.campspots;
                    bot.campspots = cs;
                    //AAS_DrawPermanentCross(cs->origin, 4, LINECOLOR_YELLOW);
                    numcampspots += 1;
                }
            }
            ent = AAS_NextBSPEntity(bot, ent);
        }
    }
    if bot.bot_developer != 0 {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"%d map locations\n".as_ptr() as *mut c_char,
                numlocations,
            );
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"%d camp spots\n".as_ptr() as *mut c_char,
                numcampspots,
            );
        }
    }
}

/// Raven `InitLevelItemHeap`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:355-371`
pub fn InitLevelItemHeap(bot: &mut BotLib) {
    unsafe {
        if !bot.levelitemheap.is_null() {
            FreeMemory(bot, bot.levelitemheap as *mut ());
        }

        let max_levelitems = LibVarValue(
            bot,
            c"max_levelitems".as_ptr() as *mut c_char,
            c"256".as_ptr() as *mut c_char,
        ) as c_int;
        bot.levelitemheap = GetClearedMemory(
            bot,
            (max_levelitems as usize * core::mem::size_of::<levelitem_t>()) as c_ulong,
        ) as *mut levelitem_t;

        for i in 0..max_levelitems - 1 {
            (*bot.levelitemheap.add(i as usize)).next = bot.levelitemheap.add((i + 1) as usize);
        }
        (*bot.levelitemheap.add((max_levelitems - 1) as usize)).next = core::ptr::null_mut();
        //
        bot.freelevelitems = bot.levelitemheap;
    }
}

/// Raven `BotFreeItemWeights`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1700-1708`
pub fn BotFreeItemWeights(bot: &mut BotLib, goalstate: c_int) {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return;
    }
    unsafe {
        if !(*gs).itemweightconfig.is_null() {
            FreeWeightConfig(bot, (*gs).itemweightconfig);
        }
        if !(*gs).itemweightindex.is_null() {
            FreeMemory(bot, (*gs).itemweightindex as *mut ());
        }
    }
}

/// Raven `BotUpdateEntityItems`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:993-1159`
pub fn BotUpdateEntityItems(bot: &mut BotLib) {
    //timeout current entity items if necessary
    unsafe {
        let mut li = bot.levelitems;
        while !li.is_null() {
            let nextli = (*li).next;
            //if it is a item that will time out
            if (*li).timeout != 0.0 {
                //timeout the item
                if (*li).timeout < AAS_Time(bot) {
                    RemoveLevelItemFromList(bot, li);
                    FreeLevelItem(bot, li);
                }
            }
            li = nextli;
        }
    }
    //find new entity items
    let ic = bot.itemconfig;
    if bot.itemconfig.is_null() {
        return;
    }
    unsafe {
        let mut ent = AAS_NextEntity(bot, 0);
        while ent != 0 {
            if AAS_EntityType(bot, ent) == entityType_t::ET_ITEM as c_int {
                //get the model index of the entity
                let modelindex = AAS_EntityModelindex(bot, ent);
                if modelindex != 0 {
                    //get info about the entity
                    let mut entinfo: aas_entityinfo_t = core::mem::zeroed();
                    AAS_EntityInfo(bot, ent, &mut entinfo);
                    //if the entity is still moving
                    if entinfo.origin[0] == entinfo.lastvisorigin[0]
                        && entinfo.origin[1] == entinfo.lastvisorigin[1]
                        && entinfo.origin[2] == entinfo.lastvisorigin[2]
                    {
                        //check if the entity is already stored as a level item
                        let mut li = bot.levelitems;
                        let mut found = false;
                        while !li.is_null() {
                            //if the level item is linked to an entity
                            if (*li).entitynum != 0 && (*li).entitynum == ent {
                                let iteminfo_idx = (*li).iteminfo as usize;
                                //the entity is re-used if the models are different
                                if (*(*ic).iteminfo.add(iteminfo_idx)).modelindex != modelindex {
                                    //remove this level item
                                    RemoveLevelItemFromList(bot, li);
                                    FreeLevelItem(bot, li);
                                } else {
                                    if entinfo.origin[0] != (*li).origin[0]
                                        || entinfo.origin[1] != (*li).origin[1]
                                        || entinfo.origin[2] != (*li).origin[2]
                                    {
                                        (*li).origin = entinfo.origin;
                                        //also update the goal area number
                                        let ii = (*ic).iteminfo.add(iteminfo_idx);
                                        (*li).goalareanum = AAS_BestReachableArea(
                                            bot,
                                            (*li).origin,
                                            (*ii).mins,
                                            (*ii).maxs,
                                            (*li).goalorigin,
                                        );
                                    }
                                }
                                found = true;
                                break;
                            }
                            li = (*li).next;
                        }
                        if !found {
                            //try to link the entity to a level item
                            let mut li2 = bot.levelitems;
                            let mut linked = false;
                            while !li2.is_null() {
                                //if this level item is already linked
                                if (*li2).entitynum == 0 {
                                    let flags = (*li2).flags;
                                    let skip = if bot.g_gametype == GT_SINGLE_PLAYER {
                                        flags & IFL_NOTSINGLE != 0
                                    } else if bot.g_gametype >= GT_TEAM {
                                        flags & IFL_NOTTEAM != 0
                                    } else {
                                        flags & IFL_NOTFREE != 0
                                    };
                                    if !skip {
                                        let iteminfo_idx = (*li2).iteminfo as usize;
                                        let ii = (*ic).iteminfo.add(iteminfo_idx);
                                        //if the model of the level item and the entity are the same
                                        if (*ii).modelindex == modelindex {
                                            //check if the entity is very close
                                            let dir = vec_sub((*li2).origin, entinfo.origin);
                                            if VectorLength(dir) < 30.0 {
                                                //found an entity for this level item
                                                (*li2).entitynum = ent;
                                                //if the origin is different
                                                if entinfo.origin[0] != (*li2).origin[0]
                                                    || entinfo.origin[1] != (*li2).origin[1]
                                                    || entinfo.origin[2] != (*li2).origin[2]
                                                {
                                                    //update the level item origin
                                                    (*li2).origin = entinfo.origin;
                                                    //also update the goal area number
                                                    (*li2).goalareanum = AAS_BestReachableArea(
                                                        bot,
                                                        (*li2).origin,
                                                        (*ii).mins,
                                                        (*ii).maxs,
                                                        (*li2).goalorigin,
                                                    );
                                                }
                                                linked = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                                li2 = (*li2).next;
                            }
                            if !linked {
                                //check if the model is from a known item
                                let numiteminfo = (*ic).numiteminfo;
                                let mut i = 0;
                                while i < numiteminfo {
                                    if (*(*ic).iteminfo.add(i as usize)).modelindex == modelindex {
                                        break;
                                    }
                                    i += 1;
                                }
                                //if the model is not from a known item
                                if i < numiteminfo {
                                    //allocate a new level item
                                    let newli = AllocLevelItem(bot);
                                    if !newli.is_null() {
                                        //entity number of the level item
                                        (*newli).entitynum = ent;
                                        //number for the level item
                                        (*newli).number = bot.numlevelitems + ent;
                                        //set the item info index for the level item
                                        (*newli).iteminfo = i;
                                        //origin of the item
                                        (*newli).origin = entinfo.origin;
                                        //get the item goal area and goal origin
                                        let ii = (*ic).iteminfo.add(i as usize);
                                        (*newli).goalareanum = AAS_BestReachableArea(
                                            bot,
                                            (*newli).origin,
                                            (*ii).mins,
                                            (*ii).maxs,
                                            (*newli).goalorigin,
                                        );
                                        //never go for items dropped into jumppads
                                        if AAS_AreaJumpPad(bot, (*newli).goalareanum) != 0 {
                                            FreeLevelItem(bot, newli);
                                        } else {
                                            //time this item out after 30 seconds
                                            //dropped items disappear after 30 seconds
                                            (*newli).timeout = AAS_Time(bot) + 30.0;
                                            //add the level item to the list
                                            AddLevelItemToList(bot, newli);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ent = AAS_NextEntity(bot, ent);
        }
    }
}

/// Raven `BotFreeGoalState`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1736-1751`
pub fn BotFreeGoalState(bot: &mut BotLib, handle: c_int) {
    if handle <= 0 || handle > MAX_CLIENTS as c_int {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"goal state handle %d out of range\n".as_ptr() as *mut c_char,
                handle,
            );
        }
        return;
    }
    if bot.botgoalstates[handle as usize].is_null() {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"invalid goal state handle %d\n".as_ptr() as *mut c_char,
                handle,
            );
        }
        return;
    }
    BotFreeItemWeights(bot, handle);
    FreeMemory(bot, bot.botgoalstates[handle as usize] as *mut ());
    bot.botgoalstates[handle as usize] = core::ptr::null_mut();
}

/// Raven `BotShutdownGoalAI`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1784-1805`
pub fn BotShutdownGoalAI(bot: &mut BotLib) {
    if !bot.itemconfig.is_null() {
        FreeMemory(bot, bot.itemconfig as *mut ());
    }
    bot.itemconfig = core::ptr::null_mut();
    if !bot.levelitemheap.is_null() {
        FreeMemory(bot, bot.levelitemheap as *mut ());
    }
    bot.levelitemheap = core::ptr::null_mut();
    bot.freelevelitems = core::ptr::null_mut();
    bot.levelitems = core::ptr::null_mut();
    bot.numlevelitems = 0;

    BotFreeInfoEntities(bot);

    for i in 1..=MAX_CLIENTS as c_int {
        if !bot.botgoalstates[i as usize].is_null() {
            BotFreeGoalState(bot, i);
        }
    }
}

/// Raven `BotInitLevelItems`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:521-656`
pub fn BotInitLevelItems(bot: &mut BotLib) {
    //initialize the map locations and camp spots
    BotInitInfoEntities(bot);

    //initialize the level item heap
    InitLevelItemHeap(bot);
    bot.levelitems = core::ptr::null_mut();
    bot.numlevelitems = 0;

    let ic = bot.itemconfig;
    if bot.itemconfig.is_null() {
        return;
    }

    //if there's no AAS file loaded
    unsafe {
        if AAS_Loaded(bot) == 0 {
            return;
        }

        //update the modelindexes of the item info
        let numiteminfo = (*ic).numiteminfo;
        for i in 0..numiteminfo {
            let ii = (*ic).iteminfo.add(i as usize);
            if (*ii).modelindex == 0 {
                let __m = std::ffi::CString::new(format!(
                    "item {} has modelindex 0",
                    std::ffi::CStr::from_ptr((*ii).classname.as_ptr()).to_string_lossy(),
                ))
                .unwrap_or_default();
                Log_Write(bot, __m.as_ptr() as *mut c_char);
            }
        }

        let mut ent = AAS_NextBSPEntity(bot, 0);
        while ent != 0 {
            let mut classname = [0 as c_char; MAX_EPAIRKEY as usize];
            if AAS_ValueForBSPEpairKey(
                bot,
                ent,
                c"classname".as_ptr() as *mut c_char,
                classname.as_mut_ptr(),
                MAX_EPAIRKEY,
            ) != 0
            {
                let mut spawnflags: c_int = 0;
                AAS_IntForBSPEpairKey(
                    bot,
                    ent,
                    c"spawnflags".as_ptr() as *mut c_char,
                    &mut spawnflags,
                );

                let mut i = 0;
                while i < numiteminfo {
                    if libc::strcmp(
                        classname.as_ptr(),
                        (*ic)
                            .iteminfo
                            .add(i as usize)
                            .as_ref()
                            .unwrap()
                            .classname
                            .as_ptr(),
                    ) == 0
                    {
                        break;
                    }
                    i += 1;
                }
                if i >= numiteminfo {
                    let __m = std::ffi::CString::new(format!(
                        "entity {} unknown item\r\n",
                        std::ffi::CStr::from_ptr(classname.as_ptr()).to_string_lossy(),
                    ))
                    .unwrap_or_default();
                    Log_Write(bot, __m.as_ptr() as *mut c_char);
                    ent = AAS_NextBSPEntity(bot, ent);
                    continue;
                }
                //get the origin of the item
                let origin: vec3_t = [0.0; 3];
                if AAS_VectorForBSPEpairKey(
                    bot,
                    ent,
                    c"origin".as_ptr() as *mut c_char,
                    origin,
                ) == 0
                {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"item %s without origin\n".as_ptr() as *mut c_char,
                        classname.as_ptr(),
                    );
                    ent = AAS_NextBSPEntity(bot, ent);
                    continue;
                }

                let ii = (*ic).iteminfo.add(i as usize);
                let mut goalareanum: c_int = 0;
                //if it is a floating item
                if spawnflags & 1 != 0 {
                    //if the item is not floating in water
                    if AAS_PointContents(bot, origin) & CONTENTS_WATER == 0 {
                        let mut end = origin;
                        end[2] -= 32.0;
                        let trace = AAS_Trace(
                            bot,
                            origin,
                            (*ii).mins,
                            (*ii).maxs,
                            end,
                            -1,
                            CONTENTS_SOLID | CONTENTS_PLAYERCLIP,
                        );
                        //if the item not near the ground
                        if trace.fraction >= 1.0 {
                            //if the item is not reachable from a jumppad
                            goalareanum = AAS_BestReachableFromJumpPadArea(
                                bot,
                                origin,
                                (*ii).mins,
                                (*ii).maxs,
                            );
                            let __m = std::ffi::CString::new(format!(
                                "item {} reachable from jumppad area {}\r\n",
                                std::ffi::CStr::from_ptr((*ii).classname.as_ptr())
                                    .to_string_lossy(),
                                goalareanum,
                            ))
                            .unwrap_or_default();
                            Log_Write(bot, __m.as_ptr() as *mut c_char);
                            if goalareanum == 0 {
                                ent = AAS_NextBSPEntity(bot, ent);
                                continue;
                            }
                        }
                    }
                }

                let li = AllocLevelItem(bot);
                if li.is_null() {
                    return;
                }
                bot.numlevelitems += 1;
                (*li).number = bot.numlevelitems;
                (*li).timeout = 0.0;
                (*li).entitynum = 0;

                (*li).flags = 0;
                let mut value: c_int = 0;
                AAS_IntForBSPEpairKey(bot, ent, c"notfree".as_ptr() as *mut c_char, &mut value);
                if value != 0 {
                    (*li).flags |= IFL_NOTFREE;
                }
                AAS_IntForBSPEpairKey(bot, ent, c"notteam".as_ptr() as *mut c_char, &mut value);
                if value != 0 {
                    (*li).flags |= IFL_NOTTEAM;
                }
                AAS_IntForBSPEpairKey(bot, ent, c"notsingle".as_ptr() as *mut c_char, &mut value);
                if value != 0 {
                    (*li).flags |= IFL_NOTSINGLE;
                }
                AAS_IntForBSPEpairKey(bot, ent, c"notbot".as_ptr() as *mut c_char, &mut value);
                if value != 0 {
                    (*li).flags |= IFL_NOTBOT;
                }
                if libc::strcmp(classname.as_ptr(), c"item_botroam".as_ptr()) == 0 {
                    (*li).flags |= IFL_ROAM;
                    AAS_FloatForBSPEpairKey(
                        bot,
                        ent,
                        c"weight".as_ptr() as *mut c_char,
                        &mut (*li).weight,
                    );
                }
                //if not a stationary item
                if spawnflags & 1 == 0 {
                    if AAS_DropToFloor(bot, origin, (*ii).mins, (*ii).maxs) == 0 {
                        bot.botimport.Print.unwrap()(
                            PRT_MESSAGE,
                            c"%s in solid at (%1.1f %1.1f %1.1f)\n".as_ptr() as *mut c_char,
                            classname.as_ptr(),
                            origin[0] as f64,
                            origin[1] as f64,
                            origin[2] as f64,
                        );
                    }
                }
                //item info of the level item
                (*li).iteminfo = i;
                //origin of the item
                (*li).origin = origin;

                if goalareanum != 0 {
                    (*li).goalareanum = goalareanum;
                    (*li).goalorigin = origin;
                } else {
                    //get the item goal area and goal origin
                    (*li).goalareanum = AAS_BestReachableArea(
                        bot,
                        origin,
                        (*ii).mins,
                        (*ii).maxs,
                        (*li).goalorigin,
                    );
                    if (*li).goalareanum == 0 {
                        bot.botimport.Print.unwrap()(
                            PRT_MESSAGE,
                            c"%s not reachable for bots at (%1.1f %1.1f %1.1f)\n".as_ptr()
                                as *mut c_char,
                            classname.as_ptr(),
                            origin[0] as f64,
                            origin[1] as f64,
                            origin[2] as f64,
                        );
                    }
                }

                AddLevelItemToList(bot, li);
            }
            ent = AAS_NextBSPEntity(bot, ent);
        }
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"found %d level items\n".as_ptr() as *mut c_char,
            bot.numlevelitems,
        );
    }
}

/// Raven `BotChooseLTGItem`.
///
/// Raven: `#ifdef UNDECIDEDFUZZY`/`#else` — `UNDECIDEDFUZZY` is
/// unconditionally true (see [`crate::be_ai_goal::be_ai_goal_cpp_consts`]),
/// so only the `FuzzyWeightUndecided` branch is live; `DROPPEDWEIGHT` is
/// likewise always-true so its guarded block transcribes unconditionally.
/// The commented-out roam-goal fallback (`#if 0`-style block in the
/// original, wrapped in a literal `/* */` comment) is dead and dropped.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1268-1428`
pub fn BotChooseLTGItem(
    common: &mut Common,
    bot: &mut BotLib,
    goalstate: c_int,
    origin: vec3_t,
    inventory: *mut c_int,
    travelflags: c_int,
) -> c_int {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return qfalse;
    }
    unsafe {
        if (*gs).itemweightconfig.is_null() {
            return qfalse;
        }
        //get the area the bot is in
        let mut areanum = BotReachabilityArea(bot, origin, (*gs).client);
        //if the bot is in solid or if the area the bot is in has no reachability links
        if areanum == 0 || AAS_AreaReachability(bot, areanum) == 0 {
            //use the last valid area the bot was in
            areanum = (*gs).lastreachabilityarea;
        }
        //remember the last area with reachabilities the bot was in
        (*gs).lastreachabilityarea = areanum;
        //if still in solid
        if areanum == 0 {
            return qfalse;
        }
        //the item configuration
        let ic = bot.itemconfig;
        if bot.itemconfig.is_null() {
            return qfalse;
        }
        //best weight and item so far
        let mut bestweight: f32 = 0.0;
        let mut bestitem: *mut levelitem_t = core::ptr::null_mut();
        let mut goal: bot_goal_t = core::mem::zeroed();
        //go through the items in the level
        let mut li = bot.levelitems;
        while !li.is_null() {
            let flags = (*li).flags;
            let skip = if bot.g_gametype == GT_SINGLE_PLAYER {
                flags & IFL_NOTSINGLE != 0
            } else if bot.g_gametype >= GT_TEAM {
                flags & IFL_NOTTEAM != 0
            } else {
                flags & IFL_NOTFREE != 0
            };
            if !skip
                && flags & IFL_NOTBOT == 0
                && (*li).goalareanum != 0
                && ((*li).entitynum != 0 || flags & IFL_ROAM != 0)
            {
                //get the fuzzy weight function for this item
                let iteminfo_idx = (*li).iteminfo as usize;
                let iteminfo = (*ic).iteminfo.add(iteminfo_idx);
                let weightnum = *(*gs).itemweightindex.add((*iteminfo).number as usize);
                if weightnum >= 0 {
                    let mut weight =
                        FuzzyWeightUndecided(common, inventory, (*gs).itemweightconfig, weightnum);
                    //HACK: to make dropped items more attractive
                    if (*li).timeout != 0.0 {
                        weight += (*bot.droppedweight).value;
                    }
                    //use weight scale for item_botroam
                    if flags & IFL_ROAM != 0 {
                        weight *= (*li).weight;
                    }
                    if weight > 0.0 {
                        //get the travel time towards the goal area
                        let t = AAS_AreaTravelTimeToGoalArea(
                            bot,
                            areanum,
                            origin,
                            (*li).goalareanum,
                            travelflags,
                        );
                        //if the goal is reachable
                        if t > 0 {
                            //if this item won't respawn before we get there
                            let avoidtime = BotAvoidGoalTime(bot, goalstate, (*li).number);
                            if avoidtime - t as f32 * 0.009 <= 0.0 {
                                let weight2 = weight / (t as f32 * TRAVELTIME_SCALE);
                                if weight2 > bestweight {
                                    bestweight = weight2;
                                    bestitem = li;
                                }
                            }
                        }
                    }
                }
            }
            li = (*li).next;
        }
        //if no goal item found
        if bestitem.is_null() {
            return qfalse;
        }
        //create a bot goal for this item
        let iteminfo = (*ic).iteminfo.add((*bestitem).iteminfo as usize);
        goal.origin = (*bestitem).goalorigin;
        goal.mins = (*iteminfo).mins;
        goal.maxs = (*iteminfo).maxs;
        goal.areanum = (*bestitem).goalareanum;
        goal.entitynum = (*bestitem).entitynum;
        goal.number = (*bestitem).number;
        goal.flags = GFL_ITEM;
        if (*bestitem).timeout != 0.0 {
            goal.flags |= GFL_DROPPED;
        }
        if (*bestitem).flags & IFL_ROAM != 0 {
            goal.flags |= GFL_ROAM;
        }
        goal.iteminfo = (*bestitem).iteminfo;
        //if it's a dropped item
        let avoidtime = if (*bestitem).timeout != 0.0 {
            AVOID_DROPPED_TIME as f32
        } else {
            let mut at = (*iteminfo).respawntime;
            if at == 0.0 {
                at = AVOID_DEFAULT_TIME as f32;
            }
            if at < AVOID_MINIMUM_TIME as f32 {
                at = AVOID_MINIMUM_TIME as f32;
            }
            at
        };
        //add the chosen goal to the goals to avoid for a while
        BotAddToAvoidGoals(bot, gs, (*bestitem).number, avoidtime);
        //push the goal on the stack
        BotPushGoal(bot, goalstate, &mut goal);
        qtrue
    }
}

/// Raven `BotChooseNBGItem`.
///
/// Raven: same always-true `UNDECIDEDFUZZY`/`DROPPEDWEIGHT` note as
/// [`BotChooseLTGItem`].
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1435-1585`
pub fn BotChooseNBGItem(
    common: &mut Common,
    bot: &mut BotLib,
    goalstate: c_int,
    origin: vec3_t,
    inventory: *mut c_int,
    travelflags: c_int,
    ltg: *mut bot_goal_t,
    maxtime: f32,
) -> c_int {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return qfalse;
    }
    unsafe {
        if (*gs).itemweightconfig.is_null() {
            return qfalse;
        }
        //get the area the bot is in
        let mut areanum = BotReachabilityArea(bot, origin, (*gs).client);
        //if the bot is in solid or if the area the bot is in has no reachability links
        if areanum == 0 || AAS_AreaReachability(bot, areanum) == 0 {
            //use the last valid area the bot was in
            areanum = (*gs).lastreachabilityarea;
        }
        //remember the last area with reachabilities the bot was in
        (*gs).lastreachabilityarea = areanum;
        //if still in solid
        if areanum == 0 {
            return qfalse;
        }
        let ltg_time = if !ltg.is_null() {
            AAS_AreaTravelTimeToGoalArea(bot, areanum, origin, (*ltg).areanum, travelflags)
        } else {
            99999
        };
        //the item configuration
        let ic = bot.itemconfig;
        if bot.itemconfig.is_null() {
            return qfalse;
        }
        //best weight and item so far
        let mut bestweight: f32 = 0.0;
        let mut bestitem: *mut levelitem_t = core::ptr::null_mut();
        let mut goal: bot_goal_t = core::mem::zeroed();
        //go through the items in the level
        let mut li = bot.levelitems;
        while !li.is_null() {
            let flags = (*li).flags;
            let skip = if bot.g_gametype == GT_SINGLE_PLAYER {
                flags & IFL_NOTSINGLE != 0
            } else if bot.g_gametype >= GT_TEAM {
                flags & IFL_NOTTEAM != 0
            } else {
                flags & IFL_NOTFREE != 0
            };
            if !skip
                && flags & IFL_NOTBOT == 0
                && (*li).goalareanum != 0
                && ((*li).entitynum != 0 || flags & IFL_ROAM != 0)
            {
                let iteminfo_idx = (*li).iteminfo as usize;
                let iteminfo = (*ic).iteminfo.add(iteminfo_idx);
                let weightnum = *(*gs).itemweightindex.add((*iteminfo).number as usize);
                if weightnum >= 0 {
                    let mut weight =
                        FuzzyWeightUndecided(common, inventory, (*gs).itemweightconfig, weightnum);
                    //HACK: to make dropped items more attractive
                    if (*li).timeout != 0.0 {
                        weight += (*bot.droppedweight).value;
                    }
                    //use weight scale for item_botroam
                    if flags & IFL_ROAM != 0 {
                        weight *= (*li).weight;
                    }
                    if weight > 0.0 {
                        //get the travel time towards the goal area
                        let mut t = AAS_AreaTravelTimeToGoalArea(
                            bot,
                            areanum,
                            origin,
                            (*li).goalareanum,
                            travelflags,
                        );
                        //if the goal is reachable
                        if t > 0 && (t as f32) < maxtime {
                            //if this item won't respawn before we get there
                            let avoidtime = BotAvoidGoalTime(bot, goalstate, (*li).number);
                            if avoidtime - t as f32 * 0.009 <= 0.0 {
                                let weight2 = weight / (t as f32 * TRAVELTIME_SCALE);
                                if weight2 > bestweight {
                                    t = 0;
                                    if !ltg.is_null() && (*li).timeout == 0.0 {
                                        //get the travel time from the goal to the long term goal
                                        t = AAS_AreaTravelTimeToGoalArea(
                                            bot,
                                            (*li).goalareanum,
                                            (*li).goalorigin,
                                            (*ltg).areanum,
                                            travelflags,
                                        );
                                    }
                                    //if the travel back is possible and doesn't take too long
                                    if t <= ltg_time {
                                        bestweight = weight2;
                                        bestitem = li;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            li = (*li).next;
        }
        //if no goal item found
        if bestitem.is_null() {
            return qfalse;
        }
        //create a bot goal for this item
        let iteminfo = (*ic).iteminfo.add((*bestitem).iteminfo as usize);
        goal.origin = (*bestitem).goalorigin;
        goal.mins = (*iteminfo).mins;
        goal.maxs = (*iteminfo).maxs;
        goal.areanum = (*bestitem).goalareanum;
        goal.entitynum = (*bestitem).entitynum;
        goal.number = (*bestitem).number;
        goal.flags = GFL_ITEM;
        if (*bestitem).timeout != 0.0 {
            goal.flags |= GFL_DROPPED;
        }
        if (*bestitem).flags & IFL_ROAM != 0 {
            goal.flags |= GFL_ROAM;
        }
        goal.iteminfo = (*bestitem).iteminfo;
        //if it's a dropped item
        let avoidtime = if (*bestitem).timeout != 0.0 {
            AVOID_DROPPED_TIME as f32
        } else {
            let mut at = (*iteminfo).respawntime;
            if at == 0.0 {
                at = AVOID_DEFAULT_TIME as f32;
            }
            if at < AVOID_MINIMUM_TIME as f32 {
                at = AVOID_MINIMUM_TIME as f32;
            }
            at
        };
        //add the chosen goal to the goals to avoid for a while
        BotAddToAvoidGoals(bot, gs, (*bestitem).number, avoidtime);
        //push the goal on the stack
        BotPushGoal(bot, goalstate, &mut goal);
        qtrue
    }
}

/// Raven `LoadItemConfig`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:251-324`
pub fn LoadItemConfig(bot: &mut BotLib, filename: *mut c_char) -> *mut itemconfig_t {
    unsafe {
        let mut max_iteminfo = LibVarValue(
            bot,
            c"max_iteminfo".as_ptr() as *mut c_char,
            c"256".as_ptr() as *mut c_char,
        ) as c_int;
        if max_iteminfo < 0 {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"max_iteminfo = %d\n".as_ptr() as *mut c_char,
                max_iteminfo,
            );
            max_iteminfo = 256;
            LibVarSet(
                bot,
                c"max_iteminfo".as_ptr() as *mut c_char,
                c"256".as_ptr() as *mut c_char,
            );
        }

        // §19: `path` is a fixed local buffer Raven writes via strncpy before
        // any read; zero-init to avoid reading uninitialized bytes if the
        // filename is shorter than the buffer and unterminated.
        const MAX_PATH: usize = 260;
        let mut path = [0 as c_char; MAX_PATH];
        libc::strncpy(path.as_mut_ptr(), filename, MAX_PATH);
        PC_SetBaseFolder(bot, BOTFILESBASEFOLDER.as_ptr() as *mut c_char);
        let source = LoadSourceFile(bot, path.as_ptr());
        if source.is_null() {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"counldn't load %s\n".as_ptr() as *mut c_char,
                path.as_ptr(),
            );
            return core::ptr::null_mut();
        }
        //initialize item config
        let ic = GetClearedHunkMemory(
            bot,
            (core::mem::size_of::<itemconfig_t>()
                + max_iteminfo as usize * core::mem::size_of::<iteminfo_t>())
                as c_ulong,
        ) as *mut itemconfig_t;
        (*ic).iteminfo =
            (ic as *mut u8).add(core::mem::size_of::<itemconfig_t>()) as *mut iteminfo_t;
        (*ic).numiteminfo = 0;
        //parse the item config file
        let mut token: token_t = core::mem::zeroed();
        while PC_ReadToken(bot, source, &mut token) != 0 {
            if libc::strcmp(token.string.as_ptr(), c"iteminfo".as_ptr()) == 0 {
                if (*ic).numiteminfo >= max_iteminfo {
                    let __m = std::ffi::CString::new(format!(
                        "more than {} item info defined\n",
                        max_iteminfo
                    ))
                    .unwrap_or_default();
                    SourceError(bot, source, __m.as_ptr());
                    FreeMemory(bot, ic as *mut ());
                    FreeSource(bot, source);
                    return core::ptr::null_mut();
                }
                let ii = (*ic).iteminfo.add((*ic).numiteminfo as usize);
                Com_Memset(ii as *mut (), 0, core::mem::size_of::<iteminfo_t>());
                if PC_ExpectTokenType(bot, source, TT_STRING, 0, &mut token) == 0 {
                    FreeMemory(bot, ic as *mut ());
                    FreeMemory(bot, source as *mut ());
                    return core::ptr::null_mut();
                }
                StripDoubleQuotes(token.string.as_mut_ptr());
                libc::strncpy(
                    (*ii).classname.as_mut_ptr(),
                    token.string.as_ptr(),
                    core::mem::size_of_val(&(*ii).classname) - 1,
                );
                let iteminfo_struct_ptr = &mut bot.iteminfo_struct as *mut structdef_t;
                if ReadStructure(bot, source, iteminfo_struct_ptr, ii as *mut c_char) == 0 {
                    FreeMemory(bot, ic as *mut ());
                    FreeSource(bot, source);
                    return core::ptr::null_mut();
                }
                (*ii).number = (*ic).numiteminfo;
                (*ic).numiteminfo += 1;
            } else {
                let __m = std::ffi::CString::new(format!(
                    "unknown definition {}\n",
                    std::ffi::CStr::from_ptr(token.string.as_ptr()).to_string_lossy(),
                ))
                .unwrap_or_default();
                SourceError(bot, source, __m.as_ptr());
                FreeMemory(bot, ic as *mut ());
                FreeSource(bot, source);
                return core::ptr::null_mut();
            }
        }
        FreeSource(bot, source);

        if (*ic).numiteminfo == 0 {
            bot.botimport.Print.unwrap()(
                PRT_WARNING,
                c"no item info loaded\n".as_ptr() as *mut c_char,
            );
        }
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"loaded %s\n".as_ptr() as *mut c_char,
            path.as_ptr(),
        );
        ic
    }
}

/// Raven `BotLoadItemWeights`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1674-1693`
pub fn BotLoadItemWeights(bot: &mut BotLib, goalstate: c_int, filename: *mut c_char) -> c_int {
    let gs = BotGoalStateFromHandle(bot, goalstate);
    if gs.is_null() {
        return BLERR_CANNOTLOADITEMWEIGHTS;
    }
    unsafe {
        //load the weight configuration
        (*gs).itemweightconfig = ReadWeightConfig(bot, filename);
        if (*gs).itemweightconfig.is_null() {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"couldn't load weights\n".as_ptr() as *mut c_char,
            );
            return BLERR_CANNOTLOADITEMWEIGHTS;
        }
        //if there's no item configuration
        if bot.itemconfig.is_null() {
            return BLERR_CANNOTLOADITEMWEIGHTS;
        }
        //create the item weight index
        (*gs).itemweightindex = ItemWeightIndex(bot, (*gs).itemweightconfig, bot.itemconfig);
    }
    //everything went ok
    BLERR_NOERROR
}

/// Raven `BotSetupGoalAI`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:1758-1777`
pub fn BotSetupGoalAI(bot: &mut BotLib) -> c_int {
    //check if teamplay is on
    unsafe {
        bot.g_gametype = LibVarValue(
            bot,
            c"g_gametype".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
        ) as c_int;
        //item configuration file
        let filename = LibVarString(
            bot,
            c"itemconfig".as_ptr() as *mut c_char,
            c"items.c".as_ptr() as *mut c_char,
        );
        //load the item configuration
        bot.itemconfig = LoadItemConfig(bot, filename);
        if bot.itemconfig.is_null() {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"couldn't load item config\n".as_ptr() as *mut c_char,
            );
            return BLERR_CANNOTLOADITEMCONFIG;
        }

        bot.droppedweight = LibVar(
            bot,
            c"droppedweight".as_ptr() as *mut c_char,
            c"1000".as_ptr() as *mut c_char,
        );
    }
    //everything went ok
    BLERR_NOERROR
}
