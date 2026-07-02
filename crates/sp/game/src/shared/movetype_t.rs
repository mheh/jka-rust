#![allow(non_camel_case_types, non_snake_case)]

/// Raven `movetype_t` — entity movement type.
///
/// Type definition source: `oracle/oracle/code/game/g_shared.h:374-381`
#[repr(i32)]
pub enum movetype_t {
    MT_STATIC = 0,
    MT_WALK = 1,
    MT_RUNJUMP = 2,
    MT_FLYSWIM = 3,
    NUM_MOVETYPES = 4,
}
