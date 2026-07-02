#![allow(non_camel_case_types, non_snake_case)]

/// Raven `deform_t` — mesh deformation modes.
///
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:207-224`
#[repr(i32)]
pub enum deform_t {
	DEFORM_NONE,
	DEFORM_WAVE,
	DEFORM_NORMALS,
	DEFORM_BULGE,
	DEFORM_MOVE,
	DEFORM_PROJECTION_SHADOW,
	DEFORM_AUTOSPRITE,
	DEFORM_AUTOSPRITE2,
	DEFORM_TEXT0,
	DEFORM_TEXT1,
	DEFORM_TEXT2,
	DEFORM_TEXT3,
	DEFORM_TEXT4,
	DEFORM_TEXT5,
	DEFORM_TEXT6,
	DEFORM_TEXT7,
}
