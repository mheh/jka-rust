#![allow(non_camel_case_types, non_snake_case, clippy::missing_safety_doc)]

//! MP botlib `be_ai_weight.cpp` — fuzzy-logic inventory weight configs.
//!
//! Source: `oracle/codemp/botlib/be_ai_weight.cpp`

use core::ffi::{c_char, c_int, c_ulong};
use std::ffi::{CStr, CString};

use mp_engine_qcommon::common::Common;
use mp_qshared::common::mp::botlib::botlib_misc::BOTFILESBASEFOLDER;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_MESSAGE};
use mp_qshared::shared::{qfalse, qtrue};

use crate::be_ai_weight::fuzzyseperator_s::fuzzyseperator_t;
use crate::be_ai_weight::weightconfig_s::{
    weightconfig_t, MAX_INVENTORYVALUE, MAX_WEIGHTS, MAX_WEIGHT_FILES, WT_BALANCE,
};
use crate::l_precomp::source_s::Source;
use crate::l_precomp_fns::{
    FreeSource, LoadSourceFile, PC_CheckTokenString, PC_ExpectAnyToken, PC_ExpectTokenString,
    PC_ExpectTokenType, PC_ReadToken, PC_SetBaseFolder, SourceError, SourceWarning,
};
use crate::l_script::consts::{TT_INTEGER, TT_NUMBER, TT_STRING};
use crate::l_script::token_s::Token;
use crate::l_script_fns::StripDoubleQuotes;
use crate::BotLib;

use crate::l_libvar_fns::LibVarGetValue;
use crate::l_memory_fns::{FreeMemory, GetClearedMemory};
use mp_qshared::shared::q_string::Q_strncpyz;

/// Raven `FindFuzzyWeight` — index of the named weight in `wc`, or `-1`.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:540-552`
pub fn FindFuzzyWeight(wc: *mut weightconfig_t, name: *mut c_char) -> c_int {
    unsafe {
        for i in 0..(*wc).numweights {
            if libc::strcmp((*wc).weights[i as usize].name, name) == 0 {
                return i;
            }
        }
        -1
    }
}

/// Raven `FuzzyWeight_r` — recursively evaluate a fuzzy separator tree
/// against the given inventory, interpolating between adjacent cases.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:559-586`
pub fn FuzzyWeight_r(inventory: *mut c_int, fs: *mut fuzzyseperator_t) -> f32 {
    unsafe {
        let scale: f32;
        let w1: f32;
        let w2: f32;

        if *inventory.offset((*fs).index as isize) < (*fs).value {
            if !(*fs).child.is_null() {
                return FuzzyWeight_r(inventory, (*fs).child);
            } else {
                return (*fs).weight;
            }
        } else if !(*fs).next.is_null() {
            if *inventory.offset((*fs).index as isize) < (*(*fs).next).value {
                // first weight
                if !(*fs).child.is_null() {
                    w1 = FuzzyWeight_r(inventory, (*fs).child);
                } else {
                    w1 = (*fs).weight;
                }
                // second weight
                if !(*(*fs).next).child.is_null() {
                    w2 = FuzzyWeight_r(inventory, (*(*fs).next).child);
                } else {
                    w2 = (*(*fs).next).weight;
                }
                // the scale factor
                // C computes this with all-int operands: integer division
                // THEN widens to float. Since value <= inventory[index] <
                // next->value here, the quotient is always 0 (returns w2).
                // Source: `oracle/codemp/botlib/be_ai_weight.cpp:579`
                scale = ((*inventory.offset((*fs).index as isize) - (*fs).value)
                    / ((*(*fs).next).value - (*fs).value)) as f32;
                // scale between the two weights
                return scale * w1 + (1.0 - scale) * w2;
            }
            return FuzzyWeight_r(inventory, (*fs).next);
        }
        (*fs).weight
    }
}

/// Raven `EvolveFuzzySeperator_r` — mutate a fuzzy separator tree's
/// `WT_BALANCE` leaves, occasionally taking a larger evolutionary leap.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:689-705`
pub fn EvolveFuzzySeperator_r(common: &mut Common, fs: *mut fuzzyseperator_t) {
    unsafe {
        if !(*fs).child.is_null() {
            EvolveFuzzySeperator_r(common, (*fs).child);
        } else if (*fs).r#type == WT_BALANCE {
            // every once in a while an evolution leap occurs, mutation
            // `random()`/`crandom()` route through the engine LCG on `common`:
            // `random()` = flrand(0,1), `crandom()` = flrand(-1,1).
            if common.qrand.flrand(0.0, 1.0) < 0.01 {
                (*fs).weight +=
                    common.qrand.flrand(-1.0, 1.0) * ((*fs).maxweight - (*fs).minweight);
            } else {
                (*fs).weight +=
                    common.qrand.flrand(-1.0, 1.0) * ((*fs).maxweight - (*fs).minweight) * 0.5;
            }
            // modify bounds if necesary because of mutation
            if (*fs).weight < (*fs).minweight {
                (*fs).minweight = (*fs).weight;
            } else if (*fs).weight > (*fs).maxweight {
                (*fs).maxweight = (*fs).weight;
            }
        }
        if !(*fs).next.is_null() {
            EvolveFuzzySeperator_r(common, (*fs).next);
        }
    }
}

/// Raven `ScaleFuzzySeperator_r` — scale a fuzzy separator tree's
/// `WT_BALANCE` leaves toward `(min+max)*scale`, clamped between bounds.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:727-742`
pub fn ScaleFuzzySeperator_r(fs: *mut fuzzyseperator_t, scale: f32) {
    unsafe {
        if !(*fs).child.is_null() {
            ScaleFuzzySeperator_r((*fs).child, scale);
        } else if (*fs).r#type == WT_BALANCE {
            (*fs).weight = ((*fs).maxweight + (*fs).minweight) * scale;
            // get the weight between bounds
            if (*fs).weight < (*fs).minweight {
                (*fs).weight = (*fs).minweight;
            } else if (*fs).weight > (*fs).maxweight {
                (*fs).weight = (*fs).maxweight;
            }
        }
        if !(*fs).next.is_null() {
            ScaleFuzzySeperator_r((*fs).next, scale);
        }
    }
}

/// Raven `ScaleFuzzySeperatorBalanceRange_r` — scale a fuzzy separator
/// tree's `WT_BALANCE` leaves' min/max bounds around their midpoint.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:770-788`
pub fn ScaleFuzzySeperatorBalanceRange_r(fs: *mut fuzzyseperator_t, scale: f32) {
    unsafe {
        if !(*fs).child.is_null() {
            ScaleFuzzySeperatorBalanceRange_r((*fs).child, scale);
        } else if (*fs).r#type == WT_BALANCE {
            let mid: f32 = ((*fs).minweight + (*fs).maxweight) * 0.5;
            // get the weight between bounds
            (*fs).maxweight = mid + ((*fs).maxweight - mid) * scale;
            (*fs).minweight = mid + ((*fs).minweight - mid) * scale;
            if (*fs).maxweight < (*fs).minweight {
                (*fs).maxweight = (*fs).minweight;
            }
        }
        if !(*fs).next.is_null() {
            ScaleFuzzySeperatorBalanceRange_r((*fs).next, scale);
        }
    }
}

/// Raven `InterbreedFuzzySeperator_r` — combine two fuzzy separator trees
/// into `fsout`, failing (and reporting) on shape mismatch.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:812-851`
pub fn InterbreedFuzzySeperator_r(
    bot: &mut BotLib,
    fs1: *mut fuzzyseperator_t,
    fs2: *mut fuzzyseperator_t,
    fsout: *mut fuzzyseperator_t,
) -> c_int {
    unsafe {
        if !(*fs1).child.is_null() {
            if (*fs2).child.is_null() || (*fsout).child.is_null() {
                (bot.botimport.Print.unwrap())(
                    PRT_ERROR,
                    c"cannot interbreed weight configs, unequal child\n".as_ptr() as *mut c_char,
                );
                return qfalse;
            }
            if InterbreedFuzzySeperator_r(bot, (*fs2).child, (*fs2).child, (*fsout).child) == 0 {
                return qfalse;
            }
        } else if (*fs1).r#type == WT_BALANCE {
            if (*fs2).r#type != WT_BALANCE || (*fsout).r#type != WT_BALANCE {
                (bot.botimport.Print.unwrap())(
                    PRT_ERROR,
                    c"cannot interbreed weight configs, unequal balance\n".as_ptr() as *mut c_char,
                );
                return qfalse;
            }
            (*fsout).weight = ((*fs1).weight + (*fs2).weight) / 2.0;
            if (*fsout).weight > (*fsout).maxweight {
                (*fsout).maxweight = (*fsout).weight;
            }
            if (*fsout).weight > (*fsout).minweight {
                (*fsout).minweight = (*fsout).weight;
            }
        }
        if !(*fs1).next.is_null() {
            if (*fs2).next.is_null() || (*fsout).next.is_null() {
                (bot.botimport.Print.unwrap())(
                    PRT_ERROR,
                    c"cannot interbreed weight configs, unequal next\n".as_ptr() as *mut c_char,
                );
                return qfalse;
            }
            if InterbreedFuzzySeperator_r(bot, (*fs1).next, (*fs2).next, (*fsout).next) == 0 {
                return qfalse;
            }
        }
        qtrue
    }
}

/// Raven `FreeFuzzySeperators_r` — free an entire fuzzy separator tree.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:95-101`
pub fn FreeFuzzySeperators_r(bot: &mut BotLib, fs: *mut fuzzyseperator_t) {
    unsafe {
        if fs.is_null() {
            return;
        }
        if !(*fs).child.is_null() {
            FreeFuzzySeperators_r(bot, (*fs).child);
        }
        if !(*fs).next.is_null() {
            FreeFuzzySeperators_r(bot, (*fs).next);
        }
        FreeMemory(bot, fs as *mut _);
    }
}

/// Raven `FuzzyWeightUndecided_r` — like `FuzzyWeight_r`, but leaves with no
/// decided weight yet return a random value within their bounds.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:593-620`
pub fn FuzzyWeightUndecided_r(
    common: &mut Common,
    inventory: *mut c_int,
    fs: *mut fuzzyseperator_t,
) -> f32 {
    unsafe {
        let scale: f32;
        let w1: f32;
        let w2: f32;

        if *inventory.offset((*fs).index as isize) < (*fs).value {
            if !(*fs).child.is_null() {
                return FuzzyWeightUndecided_r(common, inventory, (*fs).child);
            } else {
                // `random()` routes through the engine LCG on `common` — `flrand(0,1)`.
                return (*fs).minweight
                    + common.qrand.flrand(0.0, 1.0) * ((*fs).maxweight - (*fs).minweight);
            }
        } else if !(*fs).next.is_null() {
            if *inventory.offset((*fs).index as isize) < (*(*fs).next).value {
                // first weight
                if !(*fs).child.is_null() {
                    w1 = FuzzyWeightUndecided_r(common, inventory, (*fs).child);
                } else {
                    w1 = (*fs).minweight
                        + common.qrand.flrand(0.0, 1.0) * ((*fs).maxweight - (*fs).minweight);
                }
                // second weight
                if !(*(*fs).next).child.is_null() {
                    w2 = FuzzyWeight_r(inventory, (*(*fs).next).child);
                } else {
                    w2 = (*(*fs).next).minweight
                        + common.qrand.flrand(0.0, 1.0)
                            * ((*(*fs).next).maxweight - (*(*fs).next).minweight);
                }
                // the scale factor
                // C computes this with all-int operands: integer division
                // THEN widens to float. Since value <= inventory[index] <
                // next->value here, the quotient is always 0 (returns w2).
                // Source: `oracle/codemp/botlib/be_ai_weight.cpp:613`
                scale = ((*inventory.offset((*fs).index as isize) - (*fs).value)
                    / ((*(*fs).next).value - (*fs).value)) as f32;
                // scale between the two weights
                return scale * w1 + (1.0 - scale) * w2;
            }
            return FuzzyWeightUndecided_r(common, inventory, (*fs).next);
        }
        (*fs).weight
    }
}

/// Raven `FuzzyWeight` — `EVALUATERECURSIVELY` evaluation of `wc`'s
/// `weightnum`th separator tree against `inventory`.
///
/// `EVALUATERECURSIVELY` is defined (be_ai_weight.cpp:31), so the `#ifdef` arm
/// (interpolating `FuzzyWeight_r`) is live.
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:627-651`
pub fn FuzzyWeight(inventory: *mut c_int, wc: *mut weightconfig_t, weightnum: c_int) -> f32 {
    unsafe { FuzzyWeight_r(inventory, (*wc).weights[weightnum as usize].firstseperator) }
}

/// Raven `EvolveWeightConfig` — evolve every fuzzy weight tree in `config`.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:712-720`
pub fn EvolveWeightConfig(common: &mut Common, config: *mut weightconfig_t) {
    unsafe {
        for i in 0..(*config).numweights {
            EvolveFuzzySeperator_r(common, (*config).weights[i as usize].firstseperator);
        }
    }
}

/// Raven `ScaleWeight` — find the named weight and scale its separator
/// tree, `scale` clamped to `[0, 1]`.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:749-763`
pub fn ScaleWeight(config: *mut weightconfig_t, name: *mut c_char, mut scale: f32) {
    unsafe {
        if scale < 0.0 {
            scale = 0.0;
        } else if scale > 1.0 {
            scale = 1.0;
        }
        for i in 0..(*config).numweights {
            if libc::strcmp(name, (*config).weights[i as usize].name) == 0 {
                ScaleFuzzySeperator_r((*config).weights[i as usize].firstseperator, scale);
                break;
            }
        }
    }
}

/// Raven `ScaleFuzzyBalanceRange` — scale every `WT_BALANCE` leaf's bounds
/// in `config`, `scale` clamped to `[0, 100]`.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:795-805`
pub fn ScaleFuzzyBalanceRange(config: *mut weightconfig_t, mut scale: f32) {
    unsafe {
        if scale < 0.0 {
            scale = 0.0;
        } else if scale > 100.0 {
            scale = 100.0;
        }
        for i in 0..(*config).numweights {
            ScaleFuzzySeperatorBalanceRange_r((*config).weights[i as usize].firstseperator, scale);
        }
    }
}

/// Raven `InterbreedWeightConfigs` — interbreed two weight configs into
/// `configout`, failing (and reporting) on a `numweights` mismatch.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:859-876`
pub fn InterbreedWeightConfigs(
    bot: &mut BotLib,
    config1: *mut weightconfig_t,
    config2: *mut weightconfig_t,
    configout: *mut weightconfig_t,
) {
    unsafe {
        if (*config1).numweights != (*config2).numweights
            || (*config1).numweights != (*configout).numweights
        {
            (bot.botimport.Print.unwrap())(
                PRT_ERROR,
                c"cannot interbreed weight configs, unequal numweights\n".as_ptr() as *mut c_char,
            );
            return;
        }
        for i in 0..(*config1).numweights {
            InterbreedFuzzySeperator_r(
                bot,
                (*config1).weights[i as usize].firstseperator,
                (*config2).weights[i as usize].firstseperator,
                (*configout).weights[i as usize].firstseperator,
            );
        }
    }
}

/// Raven `FreeWeightConfig2` — unconditionally free a weight config and all
/// of its owned strings/trees.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:108-118`
pub fn FreeWeightConfig2(bot: &mut BotLib, config: *mut weightconfig_t) {
    unsafe {
        for i in 0..(*config).numweights {
            FreeFuzzySeperators_r(bot, (*config).weights[i as usize].firstseperator);
            if !(*config).weights[i as usize].name.is_null() {
                FreeMemory(bot, (*config).weights[i as usize].name as *mut _);
            }
        }
        FreeMemory(bot, config as *mut _);
    }
}

/// Raven `FuzzyWeightUndecided` — `EVALUATERECURSIVELY` evaluation, undecided
/// leaves resolving to a random value within their bounds.
///
/// `EVALUATERECURSIVELY` is defined (be_ai_weight.cpp:31), so the `#ifdef` arm
/// (interpolating `FuzzyWeightUndecided_r`) is live.
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:658-682`
pub fn FuzzyWeightUndecided(
    common: &mut Common,
    inventory: *mut c_int,
    wc: *mut weightconfig_t,
    weightnum: c_int,
) -> f32 {
    unsafe {
        FuzzyWeightUndecided_r(
            common,
            inventory,
            (*wc).weights[weightnum as usize].firstseperator,
        )
    }
}

/// Raven `FreeWeightConfig` — free a weight config unless
/// `bot_reloadcharacters` is set (in which case it stays cached).
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:125-129`
pub fn FreeWeightConfig(bot: &mut BotLib, config: *mut weightconfig_t) {
    if LibVarGetValue(bot, "bot_reloadcharacters") == 0.0 {
        return;
    }
    FreeWeightConfig2(bot, config);
}

/// Raven `BotShutdownWeights` — free every cached weight config file.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:883-895`
pub fn BotShutdownWeights(bot: &mut BotLib) {
    for i in 0..MAX_WEIGHT_FILES {
        if !bot.weightFileList[i].is_null() {
            FreeWeightConfig2(bot, bot.weightFileList[i]);
            bot.weightFileList[i] = core::ptr::null_mut();
        }
    }
}

/// Raven `ReadValue` — read a (non-negative) numeric token as a float
/// weight value.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:42-59`
pub fn ReadValue(bot: &mut BotLib, source: &mut Source, value: *mut f32) -> c_int {
    unsafe {
        let mut token = Token::default();

        if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
            return qfalse;
        }
        if token.string == "-" {
            SourceWarning(bot, source, "negative value set to zero\n");
            if PC_ExpectTokenType(bot, source, TT_NUMBER, 0, &mut token) == 0 {
                return qfalse;
            }
        }
        if token.type_ != TT_NUMBER {
            SourceError(
                bot,
                source,
                &format!("invalid return value {}\n", token.string),
            );
            return qfalse;
        }
        *value = token.floatvalue as f32;
        qtrue
    }
}

/// Raven `ReadFuzzyWeight` — read a `balance(w, min, max)` or bare-weight
/// `return` clause into `fs`.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:66-88`
pub fn ReadFuzzyWeight(
    bot: &mut BotLib,
    source: &mut Source,
    fs: *mut fuzzyseperator_t,
) -> c_int {
    unsafe {
        if PC_CheckTokenString(bot, source, "balance") != 0 {
            (*fs).r#type = WT_BALANCE;
            if PC_ExpectTokenString(bot, source, "(") == 0 {
                return qfalse;
            }
            if ReadValue(bot, source, &mut (*fs).weight) == 0 {
                return qfalse;
            }
            if PC_ExpectTokenString(bot, source, ",") == 0 {
                return qfalse;
            }
            if ReadValue(bot, source, &mut (*fs).minweight) == 0 {
                return qfalse;
            }
            if PC_ExpectTokenString(bot, source, ",") == 0 {
                return qfalse;
            }
            if ReadValue(bot, source, &mut (*fs).maxweight) == 0 {
                return qfalse;
            }
            if PC_ExpectTokenString(bot, source, ")") == 0 {
                return qfalse;
            }
        } else {
            (*fs).r#type = 0;
            if ReadValue(bot, source, &mut (*fs).weight) == 0 {
                return qfalse;
            }
            (*fs).minweight = (*fs).weight;
            (*fs).maxweight = (*fs).weight;
        }
        if PC_ExpectTokenString(bot, source, ";") == 0 {
            return qfalse;
        }
        qtrue
    }
}

/// Raven `ReadFuzzySeperators_r` — parse a `switch (index) { case n: ... }`
/// block into a linked fuzzy separator tree.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:136-255`
pub fn ReadFuzzySeperators_r(bot: &mut BotLib, source: &mut Source) -> *mut fuzzyseperator_t {
    unsafe {
        let mut newindent;
        let index: c_int;
        let mut def;
        let mut founddefault = qfalse;
        let mut token = Token::default();
        let mut fs: *mut fuzzyseperator_t;
        let mut lastfs: *mut fuzzyseperator_t = core::ptr::null_mut();
        let mut firstfs: *mut fuzzyseperator_t = core::ptr::null_mut();

        if PC_ExpectTokenString(bot, source, "(") == 0 {
            return core::ptr::null_mut();
        }
        if PC_ExpectTokenType(bot, source, TT_NUMBER, TT_INTEGER, &mut token) == 0 {
            return core::ptr::null_mut();
        }
        index = token.intvalue as c_int;
        if PC_ExpectTokenString(bot, source, ")") == 0 {
            return core::ptr::null_mut();
        }
        if PC_ExpectTokenString(bot, source, "{") == 0 {
            return core::ptr::null_mut();
        }
        if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
            return core::ptr::null_mut();
        }
        loop {
            def = (token.string == "default") as c_int;
            if def != 0 || token.string == "case" {
                fs = GetClearedMemory(bot, core::mem::size_of::<fuzzyseperator_t>() as c_ulong)
                    as *mut fuzzyseperator_t;
                (*fs).index = index;
                if !lastfs.is_null() {
                    (*lastfs).next = fs;
                } else {
                    firstfs = fs;
                }
                lastfs = fs;
                if def != 0 {
                    if founddefault != 0 {
                        SourceError(bot, source, "switch already has a default\n");
                        FreeFuzzySeperators_r(bot, firstfs);
                        return core::ptr::null_mut();
                    }
                    (*fs).value = MAX_INVENTORYVALUE;
                    founddefault = qtrue;
                } else {
                    if PC_ExpectTokenType(bot, source, TT_NUMBER, TT_INTEGER, &mut token) == 0 {
                        FreeFuzzySeperators_r(bot, firstfs);
                        return core::ptr::null_mut();
                    }
                    (*fs).value = token.intvalue as c_int;
                }
                if PC_ExpectTokenString(bot, source, ":") == 0
                    || PC_ExpectAnyToken(bot, source, &mut token) == 0
                {
                    FreeFuzzySeperators_r(bot, firstfs);
                    return core::ptr::null_mut();
                }
                newindent = qfalse;
                if token.string == "{" {
                    newindent = qtrue;
                    if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
                        FreeFuzzySeperators_r(bot, firstfs);
                        return core::ptr::null_mut();
                    }
                }
                if token.string == "return" {
                    if ReadFuzzyWeight(bot, source, fs) == 0 {
                        FreeFuzzySeperators_r(bot, firstfs);
                        return core::ptr::null_mut();
                    }
                } else if token.string == "switch" {
                    (*fs).child = ReadFuzzySeperators_r(bot, source);
                    if (*fs).child.is_null() {
                        FreeFuzzySeperators_r(bot, firstfs);
                        return core::ptr::null_mut();
                    }
                } else {
                    SourceError(bot, source, &format!("invalid name {}\n", token.string));
                    return core::ptr::null_mut();
                }
                if newindent != 0 {
                    if PC_ExpectTokenString(bot, source, "}") == 0 {
                        FreeFuzzySeperators_r(bot, firstfs);
                        return core::ptr::null_mut();
                    }
                }
            } else {
                FreeFuzzySeperators_r(bot, firstfs);
                SourceError(bot, source, &format!("invalid name {}\n", token.string));
                return core::ptr::null_mut();
            }
            if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
                FreeFuzzySeperators_r(bot, firstfs);
                return core::ptr::null_mut();
            }
            if token.string == "}" {
                break;
            }
        }
        //
        if founddefault == 0 {
            SourceWarning(bot, source, "switch without default\n");
            fs = GetClearedMemory(bot, core::mem::size_of::<fuzzyseperator_t>() as c_ulong)
                as *mut fuzzyseperator_t;
            (*fs).index = index;
            (*fs).value = MAX_INVENTORYVALUE;
            (*fs).weight = 0.0;
            (*fs).next = core::ptr::null_mut();
            (*fs).child = core::ptr::null_mut();
            if !lastfs.is_null() {
                (*lastfs).next = fs;
            } else {
                firstfs = fs;
            }
        }
        //
        firstfs
    }
}

/// Raven `ReadWeightConfig` — load (or, absent `bot_reloadcharacters`,
/// return the cached copy of) the named weight config file.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:262-420`
///
/// `DEBUG` is not defined in this build; the `#ifdef DEBUG` timing-print arms
/// are dropped per §C10.
pub fn ReadWeightConfig(bot: &mut BotLib, filename: *mut c_char) -> *mut weightconfig_t {
    unsafe {
        let mut newindent;
        let mut avail: c_int = 0;
        let mut n: c_int;
        let mut token = Token::default();
        let mut fs: *mut fuzzyseperator_t;
        let mut config: *mut weightconfig_t;

        if LibVarGetValue(bot, "bot_reloadcharacters") == 0.0 {
            avail = -1;
            n = 0;
            while n < MAX_WEIGHT_FILES as c_int {
                config = bot.weightFileList[n as usize];
                if config.is_null() {
                    if avail == -1 {
                        avail = n;
                    }
                    n += 1;
                    continue;
                }
                if libc::strcmp(filename, (*config).filename.as_ptr() as *mut c_char) == 0 {
                    // botimport.Print( PRT_MESSAGE, "retained %s\n", filename );
                    return config;
                }
                n += 1;
            }

            if avail == -1 {
                (bot.botimport.Print.unwrap())(
                    PRT_ERROR,
                    c"weightFileList was full trying to load %s\n".as_ptr() as *mut c_char,
                    filename,
                );
                return core::ptr::null_mut();
            }
        }

        PC_SetBaseFolder(bot, BOTFILESBASEFOLDER);
        let filename_str = CStr::from_ptr(filename).to_string_lossy().into_owned();
        let mut source = match LoadSourceFile(bot, &filename_str) {
            Some(s) => s,
            None => {
                (bot.botimport.Print.unwrap())(
                    PRT_ERROR,
                    c"counldn't load %s\n".as_ptr() as *mut c_char,
                    filename,
                );
                return core::ptr::null_mut();
            }
        };
        //
        config = GetClearedMemory(bot, core::mem::size_of::<weightconfig_t>() as c_ulong)
            as *mut weightconfig_t;
        (*config).numweights = 0;
        Q_strncpyz(
            (*config).filename.as_mut_ptr(),
            filename,
            core::mem::size_of_val(&(*config).filename) as c_int,
        );
        // parse the item config file
        while PC_ReadToken(bot, &mut source, &mut token) != 0 {
            if token.string == "weight" {
                if (*config).numweights >= MAX_WEIGHTS as c_int {
                    SourceWarning(bot, &source, "too many fuzzy weights\n");
                    break;
                }
                if PC_ExpectTokenType(bot, &mut source, TT_STRING, 0, &mut token) == 0 {
                    FreeWeightConfig(bot, config);
                    FreeSource(source);
                    return core::ptr::null_mut();
                }
                StripDoubleQuotes(&mut token.string);
                let name_c = CString::new(token.string.as_str()).unwrap_or_default();
                (*config).weights[(*config).numweights as usize].name =
                    GetClearedMemory(bot, token.string.len() as c_ulong + 1) as *mut c_char;
                libc::strcpy(
                    (*config).weights[(*config).numweights as usize].name,
                    name_c.as_ptr(),
                );
                if PC_ExpectAnyToken(bot, &mut source, &mut token) == 0 {
                    FreeWeightConfig(bot, config);
                    FreeSource(source);
                    return core::ptr::null_mut();
                }
                newindent = qfalse;
                if token.string == "{" {
                    newindent = qtrue;
                    if PC_ExpectAnyToken(bot, &mut source, &mut token) == 0 {
                        FreeWeightConfig(bot, config);
                        FreeSource(source);
                        return core::ptr::null_mut();
                    }
                }
                if token.string == "switch" {
                    fs = ReadFuzzySeperators_r(bot, &mut source);
                    if fs.is_null() {
                        FreeWeightConfig(bot, config);
                        FreeSource(source);
                        return core::ptr::null_mut();
                    }
                    (*config).weights[(*config).numweights as usize].firstseperator = fs;
                } else if token.string == "return" {
                    fs = GetClearedMemory(bot, core::mem::size_of::<fuzzyseperator_t>() as c_ulong)
                        as *mut fuzzyseperator_t;
                    (*fs).index = 0;
                    (*fs).value = MAX_INVENTORYVALUE;
                    (*fs).next = core::ptr::null_mut();
                    (*fs).child = core::ptr::null_mut();
                    if ReadFuzzyWeight(bot, &mut source, fs) == 0 {
                        FreeMemory(bot, fs as *mut _);
                        FreeWeightConfig(bot, config);
                        FreeSource(source);
                        return core::ptr::null_mut();
                    }
                    (*config).weights[(*config).numweights as usize].firstseperator = fs;
                } else {
                    SourceError(bot, &source, &format!("invalid name {}\n", token.string));
                    FreeWeightConfig(bot, config);
                    FreeSource(source);
                    return core::ptr::null_mut();
                }
                if newindent != 0 {
                    if PC_ExpectTokenString(bot, &mut source, "}") == 0 {
                        FreeWeightConfig(bot, config);
                        FreeSource(source);
                        return core::ptr::null_mut();
                    }
                }
                (*config).numweights += 1;
            } else {
                SourceError(bot, &source, &format!("invalid name {}\n", token.string));
                FreeWeightConfig(bot, config);
                FreeSource(source);
                return core::ptr::null_mut();
            }
        }
        // free the source at the end of a pass
        FreeSource(source);
        // if the file was located in a pak file
        (bot.botimport.Print.unwrap())(
            PRT_MESSAGE,
            c"loaded %s\n".as_ptr() as *mut c_char,
            filename,
        );
        // #ifdef DEBUG (dropped — not defined by the oracle build, §C10)
        //
        if LibVarGetValue(bot, "bot_reloadcharacters") == 0.0 {
            bot.weightFileList[avail as usize] = config;
        }
        //
        config
    }
}
