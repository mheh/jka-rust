//! MP `bg_public.h` effect type definitions.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:627-649`

#![allow(non_camel_case_types)]

/// Raven `effectTypes_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:627-649`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum effectTypes_t {
    EFFECT_NONE = 0,
    EFFECT_SMOKE = 1,
    EFFECT_EXPLOSION = 2,
    EFFECT_EXPLOSION_PAS = 3,
    EFFECT_SPARK_EXPLOSION = 4,
    EFFECT_EXPLOSION_TRIPMINE = 5,
    EFFECT_EXPLOSION_DETPACK = 6,
    EFFECT_EXPLOSION_FLECHETTE = 7,
    EFFECT_STUNHIT = 8,
    EFFECT_EXPLOSION_DEMP2ALT = 9,
    EFFECT_EXPLOSION_TURRET = 10,
    EFFECT_SPARKS = 11,
    EFFECT_WATER_SPLASH = 12,
    EFFECT_ACID_SPLASH = 13,
    EFFECT_LAVA_SPLASH = 14,
    EFFECT_LANDING_MUD = 15,
    EFFECT_LANDING_SAND = 16,
    EFFECT_LANDING_DIRT = 17,
    EFFECT_LANDING_SNOW = 18,
    EFFECT_LANDING_GRAVEL = 19,
    EFFECT_MAX = 20,
}
