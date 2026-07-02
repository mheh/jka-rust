#![allow(non_camel_case_types, non_snake_case)]

/// Raven `targetModel_t` — target model parts for animation/rendering.
///
/// Type definition source: `oracle/oracle/code/game/g_shared.h:118-129`
#[repr(i32)]
pub enum targetModel_t {
    MODEL_LEGS = 0,
    MODEL_TORSO = 1,
    MODEL_HEAD = 2,
    MODEL_WEAPON1 = 3,
    MODEL_WEAPON2 = 4,
    MODEL_WEAPON3 = 5,
    MODEL_EXTRA1 = 6,
    MODEL_EXTRA2 = 7,
    NUM_TARGET_MODELS = 8,
}
