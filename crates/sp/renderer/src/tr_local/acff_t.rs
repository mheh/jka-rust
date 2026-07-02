#![allow(non_camel_case_types, non_snake_case)]

/// Raven `acff_t` — Alpha combine function.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:261-266`
#[repr(i32)]
pub enum acff_t {
	ACFF_NONE = 0,
	ACFF_MODULATE_RGB = 1,
	ACFF_MODULATE_RGBA = 2,
	ACFF_MODULATE_ALPHA = 3,
}
