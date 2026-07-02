#![allow(non_camel_case_types, non_snake_case)]

/// Raven `modtype_t` — model type enumeration.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:955-968`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum modtype_t {
    MOD_BAD = 0,
    MOD_BRUSH = 1,
    MOD_MESH = 2,
    MOD_MDXM = 3,
    MOD_MDXA = 4,
}
