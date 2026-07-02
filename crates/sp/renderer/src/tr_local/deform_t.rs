#![allow(non_camel_case_types, non_snake_case)]

/// Raven `deform_t` — Geometry deformation type.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:195-212`
#[repr(i32)]
pub enum deform_t {
	DEFORM_NONE = 0,
	DEFORM_WAVE = 1,
	DEFORM_NORMALS = 2,
	DEFORM_BULGE = 3,
	DEFORM_MOVE = 4,
	DEFORM_PROJECTION_SHADOW = 5,
	DEFORM_AUTOSPRITE = 6,
	DEFORM_AUTOSPRITE2 = 7,
	DEFORM_TEXT0 = 8,
	DEFORM_TEXT1 = 9,
	DEFORM_TEXT2 = 10,
	DEFORM_TEXT3 = 11,
	DEFORM_TEXT4 = 12,
	DEFORM_TEXT5 = 13,
	DEFORM_TEXT6 = 14,
	DEFORM_TEXT7 = 15,
}
