#![allow(non_camel_case_types, non_snake_case)]

/// Raven `fogPass_t` — Fog rendering pass type.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:428-432`
#[repr(i32)]
pub enum fogPass_t {
	FP_NONE = 0,		// surface is translucent and will just be adjusted properly
	FP_EQUAL = 1,		// surface is opaque but possibly alpha tested
	FP_LE = 2,			// surface is translucent, but still needs a fog pass (fog surface)
}
