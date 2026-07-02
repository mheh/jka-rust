#![allow(non_camel_case_types, non_snake_case)]

/// Raven `modtype_t` — model type enumeration.
///
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:1103-1115`
#[repr(i32)]
pub enum modtype_t {
	MOD_BAD = 0,
	MOD_BRUSH = 1,
	MOD_MESH = 2,
	MOD_MDXM = 3,
	MOD_MDXA = 4,
}
