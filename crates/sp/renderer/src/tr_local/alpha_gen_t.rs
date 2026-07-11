#![allow(non_camel_case_types, non_snake_case)]

/// Raven `alphaGen_t` — Alpha generation type.
///
/// Type definition source: `oracle/code/renderer/tr_local.h:214-228`
#[repr(i32)]
pub enum alphaGen_t {
    AGEN_IDENTITY = 0,
    AGEN_SKIP = 1,
    AGEN_ENTITY = 2,
    AGEN_ONE_MINUS_ENTITY = 3,
    AGEN_VERTEX = 4,
    AGEN_ONE_MINUS_VERTEX = 5,
    AGEN_LIGHTING_SPECULAR = 6,
    AGEN_WAVEFORM = 7,
    AGEN_PORTAL = 8,
    AGEN_BLEND = 9,
    AGEN_CONST = 10,
    AGEN_DOT = 11,
    AGEN_ONE_MINUS_DOT = 12,
}
