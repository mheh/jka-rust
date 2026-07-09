#![allow(non_camel_case_types, non_snake_case)]

/// Raven `EGLFogOverride` — GL fog override type.
///
/// Type definition source: `oracle/code/renderer/tr_local.h:268-274`
#[repr(i32)]
pub enum EGLFogOverride {
	GLFOGOVERRIDE_NONE = 0,
	GLFOGOVERRIDE_BLACK = 1,
	GLFOGOVERRIDE_WHITE = 2,
	GLFOGOVERRIDE_MAX = 3,
}
