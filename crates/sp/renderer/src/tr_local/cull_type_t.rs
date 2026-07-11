#![allow(non_camel_case_types, non_snake_case)]

/// Raven `cullType_t` — Face culling type.
///
/// Type definition source: `oracle/code/renderer/tr_local.h:422-426`
#[repr(i32)]
pub enum cullType_t {
    CT_FRONT_SIDED = 0,
    CT_BACK_SIDED = 1,
    CT_TWO_SIDED = 2,
}
