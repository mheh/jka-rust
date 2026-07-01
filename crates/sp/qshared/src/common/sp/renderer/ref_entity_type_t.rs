#![allow(non_camel_case_types, non_snake_case)]

/// Raven `refEntityType_t` — Entity rendering type discriminant.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_types.h:82-98`
#[repr(i32)]
pub enum refEntityType_t {
	RT_MODEL,
	RT_POLY,
	RT_SPRITE,
	RT_ORIENTED_QUAD,
	RT_LINE,
	RT_ELECTRICITY,
	RT_CYLINDER,
	RT_LATHE,
	RT_BEAM,
	RT_SABER_GLOW,
	/// Doesn't draw anything, just info for portals.
	RT_PORTALSURFACE,
	RT_CLOUDS,

	RT_MAX_REF_ENTITY_TYPE,
}
