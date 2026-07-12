#![allow(non_camel_case_types, non_snake_case)]

/// Raven `leType_t` — local entity types.
///
/// Type definition source: `oracle/codemp/cgame/cg_local.h:481-496`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum leType_t {
    LE_MARK = 0,
    LE_EXPLOSION = 1,
    LE_SPRITE_EXPLOSION = 2,
    LE_FADE_SCALE_MODEL = 3,
    LE_FRAGMENT = 4,
    LE_PUFF = 5,
    LE_MOVE_SCALE_FADE = 6,
    LE_FALL_SCALE_FADE = 7,
    LE_FADE_RGB = 8,
    LE_SCALE_FADE = 9,
    LE_SCOREPLUM = 10,
    LE_OLINE = 11,
    LE_SHOWREFENTITY = 12,
    LE_LINE = 13,
}
