//! MP `bg_saber.c` per-force-defense-level parry debounce table.
//!
//! Source: `oracle/codemp/game/bg_saber.c:2777-2783`

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use core::ffi::c_int;

use mp_qshared::shared::force_powers::NUM_FORCE_POWER_LEVELS;

/// Raven `bg_parryDebounce[NUM_FORCE_POWER_LEVELS]` — minimum ms between parries,
/// indexed by force-defense level (level 0 = no defense).
///
/// Source: `oracle/codemp/game/bg_saber.c:2777-2783`
pub static bg_parryDebounce: [c_int; NUM_FORCE_POWER_LEVELS as usize] = [
    500, //if don't even have defense, can't use defense!
    300,
    150,
    50,
];
