#![allow(non_camel_case_types, non_snake_case)]

/// Raven `mapSurfaceType_t` — map surface type enumeration.
///
/// Type definition source: `oracle/oracle/codemp/qcommon/../qcommon/qfiles.h:530-536`
#[repr(i32)]
pub enum mapSurfaceType_t {
	MST_BAD,
	MST_PLANAR,
	MST_PATCH,
	MST_TRIANGLE_SOUP,
	MST_FLARE,
}
