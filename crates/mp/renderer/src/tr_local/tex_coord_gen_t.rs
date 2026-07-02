#![allow(non_camel_case_types, non_snake_case)]

/// Raven `texCoordGen_t` — texture coordinate generation type.
///
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:259-270`
#[repr(i32)]
pub enum texCoordGen_t {
	TCGEN_BAD = 0,
	TCGEN_IDENTITY = 1,			// clear to 0,0
	TCGEN_LIGHTMAP = 2,
	TCGEN_LIGHTMAP1 = 3,
	TCGEN_LIGHTMAP2 = 4,
	TCGEN_LIGHTMAP3 = 5,
	TCGEN_TEXTURE = 6,
	TCGEN_ENVIRONMENT_MAPPED = 7,
	TCGEN_FOG = 8,
	TCGEN_VECTOR = 9,			// S and T from world coordinates
}
