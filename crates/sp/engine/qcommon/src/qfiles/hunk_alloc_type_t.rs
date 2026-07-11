#![allow(non_camel_case_types, non_snake_case)]

/// Raven `hunkAllocType_t` — hunk allocation type categories.
///
/// Type definition source: `oracle/code/qcommon/qfiles.h:573-585`
#[repr(i32)]
pub enum hunkAllocType_t {
    HA_MISC = 0,
    HA_MAP = 1,
    HA_SHADERS = 2,
    HA_LIGHTING = 3,
    HA_FOG = 4,
    HA_PATCHES = 5,
    HA_VIS = 6,
    HA_SUBMODELS = 7,
    HA_MODELS = 8,
    MAX_HA_TYPES = 9,
}
