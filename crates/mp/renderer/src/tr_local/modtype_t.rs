#![allow(non_camel_case_types, non_snake_case)]

/// Raven `modtype_t` — model type enumeration.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:1103-1115`
// `Clone`/`Copy` added by W2-F8: `BModelTable` carries this discriminant by
// value across the frame package.
#[repr(i32)]
#[derive(Clone, Copy)]
pub enum modtype_t {
    MOD_BAD = 0,
    MOD_BRUSH = 1,
    MOD_MESH = 2,
    MOD_MDXM = 3,
    MOD_MDXA = 4,
}
