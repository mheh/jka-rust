#![allow(non_camel_case_types, non_snake_case)]

/// Raven `genFunc_t` — generator function type for waveforms.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:192-204`
// Fieldless C enum passed by value at Raven's own call sites
// (`TableForFunc( genFunc_t func )`); `Copy` keeps those reads out of a
// struct field from moving.
#[derive(Clone, Copy)]
#[repr(i32)]
pub enum genFunc_t {
    GF_NONE = 0,
    GF_SIN = 1,
    GF_SQUARE = 2,
    GF_TRIANGLE = 3,
    GF_SAWTOOTH = 4,
    GF_INVERSE_SAWTOOTH = 5,
    GF_NOISE = 6,
    GF_RAND = 7,
}
