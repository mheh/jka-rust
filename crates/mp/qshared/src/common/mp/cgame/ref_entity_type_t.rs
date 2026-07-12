//! MP `tr_types.h` reference entity type enumeration.

#![allow(non_camel_case_types)]

/// Raven `refEntityType_t` — reference entity type codes for the renderer.
///
/// Type definition source: `oracle/codemp/cgame/tr_types.h:83-98`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum refEntityType_t {
    RT_MODEL = 0,
    RT_POLY = 1,
    RT_SPRITE = 2,
    RT_ORIENTED_QUAD = 3,
    RT_BEAM = 4,
    RT_SABER_GLOW = 5,
    RT_ELECTRICITY = 6,
    RT_PORTALSURFACE = 7, // doesn't draw anything, just info for portals
    RT_LINE = 8,
    RT_ORIENTEDLINE = 9,
    RT_CYLINDER = 10,
    RT_ENT_CHAIN = 11,
    RT_MAX_REF_ENTITY_TYPE = 12,
}
