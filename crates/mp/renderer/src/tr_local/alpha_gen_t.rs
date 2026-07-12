#![allow(non_camel_case_types, non_snake_case)]

/// Raven `alphaGen_t` — alpha generation modes.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:226-240`
#[repr(i32)]
pub enum alphaGen_t {
    AGEN_IDENTITY,
    AGEN_SKIP,
    AGEN_ENTITY,
    AGEN_ONE_MINUS_ENTITY,
    AGEN_VERTEX,
    AGEN_ONE_MINUS_VERTEX,
    AGEN_LIGHTING_SPECULAR,
    AGEN_WAVEFORM,
    AGEN_PORTAL,
    AGEN_BLEND,
    AGEN_CONST,
    AGEN_DOT,
    AGEN_ONE_MINUS_DOT,
}
