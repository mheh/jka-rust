#![allow(non_camel_case_types, non_snake_case)]

/// Raven `texMod_t` — texture modification type.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:287-296`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum texMod_t {
    TMOD_NONE = 0,
    TMOD_TRANSFORM = 1,
    TMOD_TURBULENT = 2,
    TMOD_SCROLL = 3,
    TMOD_SCALE = 4,
    TMOD_STRETCH = 5,
    TMOD_ROTATE = 6,
    TMOD_ENTITY_TRANSLATE = 7,
}
