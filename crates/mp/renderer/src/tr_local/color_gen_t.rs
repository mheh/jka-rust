#![allow(non_camel_case_types, non_snake_case)]

/// Raven `colorGen_t` — color generation modes.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:242-257`
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum colorGen_t {
    CGEN_BAD,
    CGEN_IDENTITY_LIGHTING, // tr.identityLight
    CGEN_IDENTITY,          // always (1,1,1,1)
    CGEN_ENTITY,            // grabbed from entity's modulate field
    CGEN_ONE_MINUS_ENTITY,  // grabbed from 1 - entity.modulate
    CGEN_EXACT_VERTEX,      // tess.vertexColors
    CGEN_VERTEX,            // tess.vertexColors * tr.identityLight
    CGEN_ONE_MINUS_VERTEX,
    CGEN_WAVEFORM, // programmatically generated
    CGEN_LIGHTING_DIFFUSE,
    CGEN_LIGHTING_DIFFUSE_ENTITY, //diffuse lighting * entity
    CGEN_FOG,                     // standard fog
    CGEN_CONST,                   // fixed color
    CGEN_LIGHTMAPSTYLE,
}
