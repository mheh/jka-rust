#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::MAX_QPATH;

use super::weight_s::weight_t;

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
/// Type definition source: `oracle/codemp/botlib/be_ai_weight.h:39-44`
#[repr(C)]
pub struct weightconfig_t {
    pub numweights: i32,
    pub weights: [weight_t; MAX_WEIGHTS],
    pub filename: [c_char; MAX_QPATH as usize],
}

pub type weightconfig_s = weightconfig_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<weightconfig_t>() == 2120);
    assert!(core::mem::offset_of!(weightconfig_t, numweights) == 0);
    assert!(core::mem::offset_of!(weightconfig_t, weights) == 8);
    assert!(core::mem::offset_of!(weightconfig_t, filename) == 2056);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<weightconfig_t>() == 1092);
    assert!(core::mem::offset_of!(weightconfig_t, numweights) == 0);
    assert!(core::mem::offset_of!(weightconfig_t, weights) == 4);
    assert!(core::mem::offset_of!(weightconfig_t, filename) == 1028);
};
