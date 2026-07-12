#![allow(non_camel_case_types, non_snake_case)]

/// Raven `leType_t` — local entity types.
///
/// Type definition source: `oracle/code/cgame/cg_local.h:195-206`
#[repr(i32)]
pub enum leType_t {
    LE_MARK = 0,
    LE_FADE_MODEL = 1,
    LE_FADE_SCALE_MODEL = 2,
    LE_FRAGMENT = 3,
    LE_PUFF = 4,
    LE_FADE_RGB = 5,
    LE_LIGHT = 6,
    LE_LINE = 7,
    LE_QUAD = 8,
    LE_SPRITE = 9,
}
