#![allow(non_camel_case_types, non_snake_case)]

/// Raven `EGLFogOverride` — fog override modes.
///
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:279-285`
#[repr(i32)]
pub enum EGLFogOverride {
	GLFOGOVERRIDE_NONE = 0,
	GLFOGOVERRIDE_BLACK,
	GLFOGOVERRIDE_WHITE,
	GLFOGOVERRIDE_MAX,
}
