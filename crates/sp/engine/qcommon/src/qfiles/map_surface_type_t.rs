#![allow(non_camel_case_types, non_snake_case)]

/// Raven `mapSurfaceType_t` — BSP map surface types.
///
/// Type definition source: `oracle/code/qcommon/qfiles.h:540-546`
#[repr(i32)]
pub enum mapSurfaceType_t {
    MST_BAD = 0,
    MST_PLANAR = 1,
    MST_PATCH = 2,
    MST_TRIANGLE_SOUP = 3,
    MST_FLARE = 4,
}
