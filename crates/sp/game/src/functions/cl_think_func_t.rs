#![allow(non_camel_case_types, non_snake_case)]

/// Raven `clThinkFunc_t` — enumeration of client-side think callback function IDs.
///
/// Type definition source: `oracle/oracle/code/game/g_functions.h:232-240`
#[repr(i32)]
pub enum clThinkFunc_t {
    clThinkF_NULL = 0,
    //
    clThinkF_CG_DLightThink,
    clThinkF_CG_MatrixEffect,
    clThinkF_CG_Limb,
}
