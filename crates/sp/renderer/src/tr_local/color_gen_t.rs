#![allow(non_camel_case_types, non_snake_case)]

/// Raven `colorGen_t` — Color generation type.
///
/// Type definition source: `oracle/code/renderer/tr_local.h:230-246`
#[repr(i32)]
pub enum colorGen_t {
    CGEN_BAD = 0,
    CGEN_IDENTITY_LIGHTING = 1, // tr.identityLight
    CGEN_IDENTITY = 2,          // always (1,1,1,1)
    CGEN_SKIP = 3,
    CGEN_ENTITY = 4,           // grabbed from entity's modulate field
    CGEN_ONE_MINUS_ENTITY = 5, // grabbed from 1 - entity.modulate
    CGEN_EXACT_VERTEX = 6,     // tess.vertexColors
    CGEN_VERTEX = 7,           // tess.vertexColors * tr.identityLight
    CGEN_ONE_MINUS_VERTEX = 8,
    CGEN_WAVEFORM = 9, // programmatically generated
    CGEN_LIGHTING_DIFFUSE = 10,
    CGEN_LIGHTING_DIFFUSE_ENTITY = 11, //diffuse lighting * entity
    CGEN_FOG = 12,                     // standard fog
    CGEN_CONST = 13,                   // fixed color
    CGEN_LIGHTMAPSTYLE = 14,
}
