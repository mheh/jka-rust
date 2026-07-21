#![allow(non_camel_case_types, non_snake_case)]

/// Raven `footstep_t` — enumeration of footstep surface types.
///
/// Type definition source: `oracle/codemp/cgame/cg_local.h:90-118`
/// Type definition source: `oracle/code/cgame/cg_media.h:6-34`
#[repr(i32)]
pub enum footstep_t {
    FOOTSTEP_STONEWALK = 0,
    FOOTSTEP_STONERUN = 1,
    FOOTSTEP_METALWALK = 2,
    FOOTSTEP_METALRUN = 3,
    FOOTSTEP_PIPEWALK = 4,
    FOOTSTEP_PIPERUN = 5,
    FOOTSTEP_SPLASH = 6,
    FOOTSTEP_WADE = 7,
    FOOTSTEP_SWIM = 8,
    FOOTSTEP_SNOWWALK = 9,
    FOOTSTEP_SNOWRUN = 10,
    FOOTSTEP_SANDWALK = 11,
    FOOTSTEP_SANDRUN = 12,
    FOOTSTEP_GRASSWALK = 13,
    FOOTSTEP_GRASSRUN = 14,
    FOOTSTEP_DIRTWALK = 15,
    FOOTSTEP_DIRTRUN = 16,
    FOOTSTEP_MUDWALK = 17,
    FOOTSTEP_MUDRUN = 18,
    FOOTSTEP_GRAVELWALK = 19,
    FOOTSTEP_GRAVELRUN = 20,
    FOOTSTEP_RUGWALK = 21,
    FOOTSTEP_RUGRUN = 22,
    FOOTSTEP_WOODWALK = 23,
    FOOTSTEP_WOODRUN = 24,
    FOOTSTEP_TOTAL = 25,
}
