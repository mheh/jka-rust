#![allow(non_camel_case_types, non_snake_case)]

use super::weight_s::Weight;

/// Raven `WT_BALANCE` — fuzzy weight balance flag.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.h:14`
pub const WT_BALANCE: i32 = 1;

/// `MAX_WEIGHTS`.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.h:16`
pub const MAX_WEIGHTS: usize = 128;

/// Raven `MAX_INVENTORYVALUE` — clamp for fuzzy-weight inventory evaluation.
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:30`
pub const MAX_INVENTORYVALUE: i32 = 999999;

/// Raven `MAX_WEIGHT_FILES` — max concurrently loaded weight configs.
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:33`
pub const MAX_WEIGHT_FILES: usize = 128;

/// Raven `weightconfig_t` — a set of named fuzzy weights loaded from a file.
///
/// Redesigned (porting-rules §F17): Raven's `numweights` + fixed
/// `weight_t weights[MAX_WEIGHTS]` + `char filename[MAX_QPATH]` become an owned
/// `Vec<Weight>` (length is the old `numweights`; `MAX_WEIGHTS` is still
/// enforced as the load cap) and a `String`. Instances live in the `BotLib`
/// weight-config arena, reached by `WeightConfigHandle`.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_weight.h:39-44`
#[derive(Default)]
pub struct WeightConfig {
    pub filename: String,
    pub weights: Vec<Weight>,
}

/// Arena handle for a `WeightConfig` owned by `BotLib.weightconfigs` (§B5).
///
/// Replaces Raven's `weightconfig_t *` (held by `bot_goalstate_t`,
/// `bot_weaponstate_t`, and the `weightFileList` cache). A `None` slot in the
/// arena is Raven's null config; the handle is the slot index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WeightConfigHandle(pub usize);
