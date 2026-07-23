#![allow(non_camel_case_types, non_snake_case, clippy::missing_safety_doc)]

//! MP botlib `be_ai_weight.cpp` — fuzzy-logic inventory weight configs.
//!
//! Redesigned (porting-rules §F17): the malloc'd `fuzzyseperator_t` decision
//! trees, `weight_t` names, and `weightconfig_t` blocks become owned Rust shapes
//! (`FuzzySeperator` with `Box` child/next, `Weight { String, Box tree }`,
//! `WeightConfig { String, Vec<Weight> }`) held in the `BotLib.weightconfigs`
//! arena and reached by `WeightConfigHandle`. `FreeFuzzySeperators_r` /
//! `FreeWeightConfig2`'s manual frees dissolve into `Drop` (clearing an arena
//! slot drops its whole tree). The `weightFileList` cache and its
//! `bot_reloadcharacters` retain/free semantics are preserved.
//!
//! Source: `oracle/codemp/botlib/be_ai_weight.cpp`

use core::ffi::{c_char, c_int};
use std::ffi::CString;

use mp_engine_qcommon::common::Common;
use mp_qshared::common::mp::botlib::botlib_misc::BOTFILESBASEFOLDER;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_MESSAGE};
use mp_qshared::shared::{qfalse, qtrue};

use crate::be_ai_weight::fuzzyseperator_s::FuzzySeperator;
use crate::be_ai_weight::weight_s::Weight;
use crate::be_ai_weight::weightconfig_s::{
    WeightConfig, WeightConfigHandle, MAX_INVENTORYVALUE, MAX_WEIGHTS, MAX_WEIGHT_FILES, WT_BALANCE,
};
use crate::l_libvar_fns::LibVarGetValue;
use crate::l_precomp::source_s::Source;
use crate::l_precomp_fns::{
    FreeSource, LoadSourceFile, PC_CheckTokenString, PC_ExpectAnyToken, PC_ExpectTokenString,
    PC_ExpectTokenType, PC_ReadToken, PC_SetBaseFolder, SourceError, SourceWarning,
};
use crate::l_script::consts::{TT_INTEGER, TT_NUMBER, TT_STRING};
use crate::l_script::token_s::Token;
use crate::l_script_fns::StripDoubleQuotes;
use crate::BotLib;

/// Raven `FindFuzzyWeight` — index of the named weight in `wc`, or `-1`.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:540-552`
pub fn FindFuzzyWeight(wc: &WeightConfig, name: &str) -> c_int {
    for (i, w) in wc.weights.iter().enumerate() {
        if w.name == name {
            return i as c_int;
        }
    }
    -1
}

/// Raven `FuzzyWeight_r` — recursively evaluate a fuzzy separator tree
/// against the given inventory, interpolating between adjacent cases.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:559-586`
pub fn FuzzyWeight_r(inventory: *mut c_int, fs: &FuzzySeperator) -> f32 {
    unsafe {
        let inv = *inventory.offset(fs.index as isize);
        if inv < fs.value {
            match &fs.child {
                Some(child) => FuzzyWeight_r(inventory, child),
                None => fs.weight,
            }
        } else if let Some(next) = &fs.next {
            if inv < next.value {
                // first weight
                let w1 = match &fs.child {
                    Some(child) => FuzzyWeight_r(inventory, child),
                    None => fs.weight,
                };
                // second weight
                let w2 = match &next.child {
                    Some(child) => FuzzyWeight_r(inventory, child),
                    None => next.weight,
                };
                // the scale factor
                // C computes this with all-int operands: integer division
                // THEN widens to float. Since value <= inventory[index] <
                // next->value here, the quotient is always 0 (returns w2).
                // Source: `oracle/codemp/botlib/be_ai_weight.cpp:579`
                let scale = ((inv - fs.value) / (next.value - fs.value)) as f32;
                // scale between the two weights
                scale * w1 + (1.0 - scale) * w2
            } else {
                FuzzyWeight_r(inventory, next)
            }
        } else {
            fs.weight
        }
    }
}

/// Raven `FuzzyWeightUndecided_r` — like `FuzzyWeight_r`, but leaves with no
/// decided weight yet return a random value within their bounds.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:593-620`
pub fn FuzzyWeightUndecided_r(common: &mut Common, inventory: *mut c_int, fs: &FuzzySeperator) -> f32 {
    unsafe {
        let inv = *inventory.offset(fs.index as isize);
        if inv < fs.value {
            match &fs.child {
                Some(child) => FuzzyWeightUndecided_r(common, inventory, child),
                // `random()` routes through the engine LCG on `common` — `flrand(0,1)`.
                None => fs.minweight + common.qrand.flrand(0.0, 1.0) * (fs.maxweight - fs.minweight),
            }
        } else if let Some(next) = &fs.next {
            if inv < next.value {
                // first weight
                let w1 = match &fs.child {
                    Some(child) => FuzzyWeightUndecided_r(common, inventory, child),
                    None => {
                        fs.minweight + common.qrand.flrand(0.0, 1.0) * (fs.maxweight - fs.minweight)
                    }
                };
                // second weight (Raven calls the decided `FuzzyWeight_r` here —
                // a faithful quirk of be_ai_weight.cpp:610).
                let w2 = match &next.child {
                    Some(child) => FuzzyWeight_r(inventory, child),
                    None => {
                        next.minweight
                            + common.qrand.flrand(0.0, 1.0) * (next.maxweight - next.minweight)
                    }
                };
                // the scale factor
                // C computes this with all-int operands: integer division
                // THEN widens to float. Since value <= inventory[index] <
                // next->value here, the quotient is always 0 (returns w2).
                // Source: `oracle/codemp/botlib/be_ai_weight.cpp:613`
                let scale = ((inv - fs.value) / (next.value - fs.value)) as f32;
                // scale between the two weights
                scale * w1 + (1.0 - scale) * w2
            } else {
                FuzzyWeightUndecided_r(common, inventory, next)
            }
        } else {
            fs.weight
        }
    }
}

/// Raven `EvolveFuzzySeperator_r` — mutate a fuzzy separator tree's
/// `WT_BALANCE` leaves, occasionally taking a larger evolutionary leap.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:689-705`
pub fn EvolveFuzzySeperator_r(common: &mut Common, fs: &mut FuzzySeperator) {
    if let Some(child) = fs.child.as_mut() {
        EvolveFuzzySeperator_r(common, child);
    } else if fs.type_ == WT_BALANCE {
        // every once in a while an evolution leap occurs, mutation
        // `random()`/`crandom()` route through the engine LCG on `common`:
        // `random()` = flrand(0,1), `crandom()` = flrand(-1,1).
        if common.qrand.flrand(0.0, 1.0) < 0.01 {
            fs.weight += common.qrand.flrand(-1.0, 1.0) * (fs.maxweight - fs.minweight);
        } else {
            fs.weight += common.qrand.flrand(-1.0, 1.0) * (fs.maxweight - fs.minweight) * 0.5;
        }
        // modify bounds if necesary because of mutation
        if fs.weight < fs.minweight {
            fs.minweight = fs.weight;
        } else if fs.weight > fs.maxweight {
            fs.maxweight = fs.weight;
        }
    }
    if let Some(next) = fs.next.as_mut() {
        EvolveFuzzySeperator_r(common, next);
    }
}

/// Raven `ScaleFuzzySeperator_r` — scale a fuzzy separator tree's
/// `WT_BALANCE` leaves toward `(min+max)*scale`, clamped between bounds.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:727-742`
pub fn ScaleFuzzySeperator_r(fs: &mut FuzzySeperator, scale: f32) {
    if let Some(child) = fs.child.as_mut() {
        ScaleFuzzySeperator_r(child, scale);
    } else if fs.type_ == WT_BALANCE {
        fs.weight = (fs.maxweight + fs.minweight) * scale;
        // get the weight between bounds
        if fs.weight < fs.minweight {
            fs.weight = fs.minweight;
        } else if fs.weight > fs.maxweight {
            fs.weight = fs.maxweight;
        }
    }
    if let Some(next) = fs.next.as_mut() {
        ScaleFuzzySeperator_r(next, scale);
    }
}

/// Raven `ScaleFuzzySeperatorBalanceRange_r` — scale a fuzzy separator
/// tree's `WT_BALANCE` leaves' min/max bounds around their midpoint.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:770-788`
pub fn ScaleFuzzySeperatorBalanceRange_r(fs: &mut FuzzySeperator, scale: f32) {
    if let Some(child) = fs.child.as_mut() {
        ScaleFuzzySeperatorBalanceRange_r(child, scale);
    } else if fs.type_ == WT_BALANCE {
        let mid: f32 = (fs.minweight + fs.maxweight) * 0.5;
        // get the weight between bounds
        fs.maxweight = mid + (fs.maxweight - mid) * scale;
        fs.minweight = mid + (fs.minweight - mid) * scale;
        if fs.maxweight < fs.minweight {
            fs.maxweight = fs.minweight;
        }
    }
    if let Some(next) = fs.next.as_mut() {
        ScaleFuzzySeperatorBalanceRange_r(next, scale);
    }
}

/// Raven `InterbreedFuzzySeperator_r` — combine two fuzzy separator trees
/// into `fsout`, failing (and reporting) on shape mismatch.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:812-851`
pub fn InterbreedFuzzySeperator_r(
    bot: &mut BotLib,
    fs1: &FuzzySeperator,
    fs2: &FuzzySeperator,
    fsout: &mut FuzzySeperator,
) -> c_int {
    if fs1.child.is_some() {
        if fs2.child.is_none() || fsout.child.is_none() {
            unsafe {
                (bot.botimport.Print.unwrap())(
                    PRT_ERROR,
                    c"cannot interbreed weight configs, unequal child\n".as_ptr() as *mut c_char,
                );
            }
            return qfalse;
        }
        // Raven passes `(fs2->child, fs2->child, fsout->child)` here — a faithful
        // bug (the first argument should be `fs1->child`).
        // Source: `oracle/codemp/botlib/be_ai_weight.cpp:823`
        let c2 = fs2.child.as_deref().unwrap();
        if InterbreedFuzzySeperator_r(bot, c2, c2, fsout.child.as_deref_mut().unwrap()) == 0 {
            return qfalse;
        }
    } else if fs1.type_ == WT_BALANCE {
        if fs2.type_ != WT_BALANCE || fsout.type_ != WT_BALANCE {
            unsafe {
                (bot.botimport.Print.unwrap())(
                    PRT_ERROR,
                    c"cannot interbreed weight configs, unequal balance\n".as_ptr() as *mut c_char,
                );
            }
            return qfalse;
        }
        fsout.weight = (fs1.weight + fs2.weight) / 2.0;
        if fsout.weight > fsout.maxweight {
            fsout.maxweight = fsout.weight;
        }
        if fsout.weight > fsout.minweight {
            fsout.minweight = fsout.weight;
        }
    }
    if fs1.next.is_some() {
        if fs2.next.is_none() || fsout.next.is_none() {
            unsafe {
                (bot.botimport.Print.unwrap())(
                    PRT_ERROR,
                    c"cannot interbreed weight configs, unequal next\n".as_ptr() as *mut c_char,
                );
            }
            return qfalse;
        }
        if InterbreedFuzzySeperator_r(
            bot,
            fs1.next.as_deref().unwrap(),
            fs2.next.as_deref().unwrap(),
            fsout.next.as_deref_mut().unwrap(),
        ) == 0
        {
            return qfalse;
        }
    }
    qtrue
}

// Raven `FreeFuzzySeperators_r` (be_ai_weight.cpp:95-101) is dissolved into
// `Drop`: dropping a `FuzzySeperator` recursively drops its `Box` child/next.

/// Raven `FuzzyWeight` — `EVALUATERECURSIVELY` evaluation of `wc`'s
/// `weightnum`th separator tree against `inventory`.
///
/// `EVALUATERECURSIVELY` is defined (be_ai_weight.cpp:31), so the `#ifdef` arm
/// (interpolating `FuzzyWeight_r`) is live.
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:627-651`
pub fn FuzzyWeight(inventory: *mut c_int, wc: &WeightConfig, weightnum: c_int) -> f32 {
    FuzzyWeight_r(
        inventory,
        wc.weights[weightnum as usize].firstseperator.as_deref().unwrap(),
    )
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
    wc: &WeightConfig,
    weightnum: c_int,
) -> f32 {
    FuzzyWeightUndecided_r(
        common,
        inventory,
        wc.weights[weightnum as usize].firstseperator.as_deref().unwrap(),
    )
}

/// Raven `EvolveWeightConfig` — evolve every fuzzy weight tree in `config`.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:712-720`
pub fn EvolveWeightConfig(common: &mut Common, config: &mut WeightConfig) {
    for w in config.weights.iter_mut() {
        if let Some(fs) = w.firstseperator.as_mut() {
            EvolveFuzzySeperator_r(common, fs);
        }
    }
}

/// Raven `ScaleWeight` — find the named weight and scale its separator
/// tree, `scale` clamped to `[0, 1]`.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:749-763`
pub fn ScaleWeight(config: &mut WeightConfig, name: &str, mut scale: f32) {
    if scale < 0.0 {
        scale = 0.0;
    } else if scale > 1.0 {
        scale = 1.0;
    }
    for w in config.weights.iter_mut() {
        if w.name == name {
            if let Some(fs) = w.firstseperator.as_mut() {
                ScaleFuzzySeperator_r(fs, scale);
            }
            break;
        }
    }
}

/// Raven `ScaleFuzzyBalanceRange` — scale every `WT_BALANCE` leaf's bounds
/// in `config`, `scale` clamped to `[0, 100]`.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:795-805`
pub fn ScaleFuzzyBalanceRange(config: &mut WeightConfig, mut scale: f32) {
    if scale < 0.0 {
        scale = 0.0;
    } else if scale > 100.0 {
        scale = 100.0;
    }
    for w in config.weights.iter_mut() {
        if let Some(fs) = w.firstseperator.as_mut() {
            ScaleFuzzySeperatorBalanceRange_r(fs, scale);
        }
    }
}

/// Raven `InterbreedWeightConfigs` — interbreed two weight configs into
/// `configout`, failing (and reporting) on a `numweights` mismatch.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:859-876`
pub fn InterbreedWeightConfigs(
    bot: &mut BotLib,
    config1: &WeightConfig,
    config2: &WeightConfig,
    configout: &mut WeightConfig,
) {
    if config1.weights.len() != config2.weights.len()
        || config1.weights.len() != configout.weights.len()
    {
        unsafe {
            (bot.botimport.Print.unwrap())(
                PRT_ERROR,
                c"cannot interbreed weight configs, unequal numweights\n".as_ptr() as *mut c_char,
            );
        }
        return;
    }
    for i in 0..config1.weights.len() {
        // Every loaded weight has a `firstseperator`; unwrap matches Raven's
        // unconditional pointer deref.
        InterbreedFuzzySeperator_r(
            bot,
            config1.weights[i].firstseperator.as_deref().unwrap(),
            config2.weights[i].firstseperator.as_deref().unwrap(),
            configout.weights[i].firstseperator.as_deref_mut().unwrap(),
        );
    }
}

/// Raven `FreeWeightConfig2` — unconditionally free a weight config and all
/// of its owned strings/trees. Redesigned: clearing the arena slot drops the
/// `WeightConfig` (its `Vec<Weight>`, `String` names, and `Box` trees).
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:108-118`
pub fn FreeWeightConfig2(bot: &mut BotLib, config: WeightConfigHandle) {
    bot.weightconfigs[config.0] = None;
}

/// Raven `FreeWeightConfig` — free a weight config unless
/// `bot_reloadcharacters` is set (in which case it stays cached).
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:125-129`
pub fn FreeWeightConfig(bot: &mut BotLib, config: Option<WeightConfigHandle>) {
    if LibVarGetValue(bot, "bot_reloadcharacters") == 0.0 {
        return;
    }
    if let Some(h) = config {
        FreeWeightConfig2(bot, h);
    }
}

/// Raven `BotShutdownWeights` — free every cached weight config file.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:883-895`
pub fn BotShutdownWeights(bot: &mut BotLib) {
    for i in 0..MAX_WEIGHT_FILES {
        if let Some(h) = bot.weightFileList[i] {
            FreeWeightConfig2(bot, h);
            bot.weightFileList[i] = None;
        }
    }
}

/// Raven `ReadValue` — read a (non-negative) numeric token as a float
/// weight value.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:42-59`
pub fn ReadValue(bot: &mut BotLib, source: &mut Source, value: &mut f32) -> c_int {
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

/// Raven `ReadFuzzyWeight` — read a `balance(w, min, max)` or bare-weight
/// `return` clause into `fs`.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:66-88`
pub fn ReadFuzzyWeight(bot: &mut BotLib, source: &mut Source, fs: &mut FuzzySeperator) -> c_int {
    if PC_CheckTokenString(bot, source, "balance") != 0 {
        fs.type_ = WT_BALANCE;
        if PC_ExpectTokenString(bot, source, "(") == 0 {
            return qfalse;
        }
        if ReadValue(bot, source, &mut fs.weight) == 0 {
            return qfalse;
        }
        if PC_ExpectTokenString(bot, source, ",") == 0 {
            return qfalse;
        }
        if ReadValue(bot, source, &mut fs.minweight) == 0 {
            return qfalse;
        }
        if PC_ExpectTokenString(bot, source, ",") == 0 {
            return qfalse;
        }
        if ReadValue(bot, source, &mut fs.maxweight) == 0 {
            return qfalse;
        }
        if PC_ExpectTokenString(bot, source, ")") == 0 {
            return qfalse;
        }
    } else {
        fs.type_ = 0;
        if ReadValue(bot, source, &mut fs.weight) == 0 {
            return qfalse;
        }
        fs.minweight = fs.weight;
        fs.maxweight = fs.weight;
    }
    if PC_ExpectTokenString(bot, source, ";") == 0 {
        return qfalse;
    }
    qtrue
}

/// Raven `ReadFuzzySeperators_r` — parse a `switch (index) { case n: ... }`
/// block into a linked fuzzy separator tree.
///
/// The malloc'd sibling chain becomes a `Vec` of built nodes linked back-to-
/// front into an owned `next` chain; on any parse error the collected nodes
/// drop (Raven's `FreeFuzzySeperators_r(firstfs)`).
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:136-255`
pub fn ReadFuzzySeperators_r(bot: &mut BotLib, source: &mut Source) -> Option<Box<FuzzySeperator>> {
    let index: c_int;
    let mut founddefault = qfalse;
    let mut token = Token::default();
    let mut nodes: Vec<Box<FuzzySeperator>> = Vec::new();

    if PC_ExpectTokenString(bot, source, "(") == 0 {
        return None;
    }
    if PC_ExpectTokenType(bot, source, TT_NUMBER, TT_INTEGER, &mut token) == 0 {
        return None;
    }
    index = token.intvalue as c_int;
    if PC_ExpectTokenString(bot, source, ")") == 0 {
        return None;
    }
    if PC_ExpectTokenString(bot, source, "{") == 0 {
        return None;
    }
    if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
        return None;
    }
    loop {
        let def = (token.string == "default") as c_int;
        if def != 0 || token.string == "case" {
            let mut fs = Box::new(FuzzySeperator {
                index,
                ..Default::default()
            });
            if def != 0 {
                if founddefault != 0 {
                    SourceError(bot, source, "switch already has a default\n");
                    return None;
                }
                fs.value = MAX_INVENTORYVALUE;
                founddefault = qtrue;
            } else {
                if PC_ExpectTokenType(bot, source, TT_NUMBER, TT_INTEGER, &mut token) == 0 {
                    return None;
                }
                fs.value = token.intvalue as c_int;
            }
            if PC_ExpectTokenString(bot, source, ":") == 0
                || PC_ExpectAnyToken(bot, source, &mut token) == 0
            {
                return None;
            }
            let mut newindent = qfalse;
            if token.string == "{" {
                newindent = qtrue;
                if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
                    return None;
                }
            }
            if token.string == "return" {
                if ReadFuzzyWeight(bot, source, &mut fs) == 0 {
                    return None;
                }
            } else if token.string == "switch" {
                fs.child = ReadFuzzySeperators_r(bot, source);
                if fs.child.is_none() {
                    return None;
                }
            } else {
                SourceError(bot, source, &format!("invalid name {}\n", token.string));
                return None;
            }
            if newindent != 0 && PC_ExpectTokenString(bot, source, "}") == 0 {
                return None;
            }
            nodes.push(fs);
        } else {
            SourceError(bot, source, &format!("invalid name {}\n", token.string));
            return None;
        }
        if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
            return None;
        }
        if token.string == "}" {
            break;
        }
    }
    //
    if founddefault == 0 {
        SourceWarning(bot, source, "switch without default\n");
        nodes.push(Box::new(FuzzySeperator {
            index,
            value: MAX_INVENTORYVALUE,
            ..Default::default()
        }));
    }
    // link the sibling chain (front-to-back order preserved)
    let mut chain: Option<Box<FuzzySeperator>> = None;
    while let Some(mut node) = nodes.pop() {
        node.next = chain;
        chain = Some(node);
    }
    chain
}

/// Raven `ReadWeightConfig` — load (or, absent `bot_reloadcharacters`,
/// return the cached copy of) the named weight config file.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:262-420`
///
/// `DEBUG` is not defined in this build; the `#ifdef DEBUG` timing-print arms
/// are dropped per §C10.
pub fn ReadWeightConfig(bot: &mut BotLib, filename: &str) -> Option<WeightConfigHandle> {
    let mut avail: c_int = -1;
    let mut token = Token::default();

    if LibVarGetValue(bot, "bot_reloadcharacters") == 0.0 {
        avail = -1;
        let mut n: c_int = 0;
        while n < MAX_WEIGHT_FILES as c_int {
            match bot.weightFileList[n as usize] {
                None => {
                    if avail == -1 {
                        avail = n;
                    }
                }
                Some(h) => {
                    if bot.weightconfig(h).filename == filename {
                        // botimport.Print( PRT_MESSAGE, "retained %s\n", filename );
                        return Some(h);
                    }
                }
            }
            n += 1;
        }
        if avail == -1 {
            let filename_c = CString::new(filename).unwrap_or_default();
            unsafe {
                (bot.botimport.Print.unwrap())(
                    PRT_ERROR,
                    c"weightFileList was full trying to load %s\n".as_ptr() as *mut c_char,
                    filename_c.as_ptr(),
                );
            }
            return None;
        }
    }

    PC_SetBaseFolder(bot, BOTFILESBASEFOLDER);
    let mut source = match LoadSourceFile(bot, filename) {
        Some(s) => s,
        None => {
            let filename_c = CString::new(filename).unwrap_or_default();
            unsafe {
                (bot.botimport.Print.unwrap())(
                    PRT_ERROR,
                    c"counldn't load %s\n".as_ptr() as *mut c_char,
                    filename_c.as_ptr(),
                );
            }
            return None;
        }
    };
    // initialize item config (Raven Q_strncpyz's filename into a MAX_QPATH
    // buffer; weight filenames are always short so the owned String is exact).
    let mut config = WeightConfig {
        filename: filename.to_string(),
        weights: Vec::new(),
    };
    // parse the item config file
    while PC_ReadToken(bot, &mut source, &mut token) != 0 {
        if token.string == "weight" {
            if config.weights.len() >= MAX_WEIGHTS {
                SourceWarning(bot, &source, "too many fuzzy weights\n");
                break;
            }
            if PC_ExpectTokenType(bot, &mut source, TT_STRING, 0, &mut token) == 0 {
                // config drops here (Raven leaks it in non-reload mode; the
                // owned model frees unconditionally — an error path either way).
                FreeSource(source);
                return None;
            }
            StripDoubleQuotes(&mut token.string);
            let mut weight = Weight {
                name: token.string.clone(),
                firstseperator: None,
            };
            if PC_ExpectAnyToken(bot, &mut source, &mut token) == 0 {
                FreeSource(source);
                return None;
            }
            let mut newindent = qfalse;
            if token.string == "{" {
                newindent = qtrue;
                if PC_ExpectAnyToken(bot, &mut source, &mut token) == 0 {
                    FreeSource(source);
                    return None;
                }
            }
            if token.string == "switch" {
                let fs = ReadFuzzySeperators_r(bot, &mut source);
                if fs.is_none() {
                    FreeSource(source);
                    return None;
                }
                weight.firstseperator = fs;
            } else if token.string == "return" {
                let mut fs = Box::new(FuzzySeperator {
                    index: 0,
                    value: MAX_INVENTORYVALUE,
                    ..Default::default()
                });
                if ReadFuzzyWeight(bot, &mut source, &mut fs) == 0 {
                    FreeSource(source);
                    return None;
                }
                weight.firstseperator = Some(fs);
            } else {
                SourceError(bot, &source, &format!("invalid name {}\n", token.string));
                FreeSource(source);
                return None;
            }
            if newindent != 0 && PC_ExpectTokenString(bot, &mut source, "}") == 0 {
                FreeSource(source);
                return None;
            }
            config.weights.push(weight);
        } else {
            SourceError(bot, &source, &format!("invalid name {}\n", token.string));
            FreeSource(source);
            return None;
        }
    }
    // free the source at the end of a pass
    FreeSource(source);
    // if the file was located in a pak file
    let filename_c = CString::new(filename).unwrap_or_default();
    unsafe {
        (bot.botimport.Print.unwrap())(
            PRT_MESSAGE,
            c"loaded %s\n".as_ptr() as *mut c_char,
            filename_c.as_ptr(),
        );
    }
    // #ifdef DEBUG (dropped — not defined by the oracle build, §C10)
    //
    let handle = bot.alloc_weightconfig(config);
    if LibVarGetValue(bot, "bot_reloadcharacters") == 0.0 {
        bot.weightFileList[avail as usize] = Some(handle);
    }
    //
    Some(handle)
}
