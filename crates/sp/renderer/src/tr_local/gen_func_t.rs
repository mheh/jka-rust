#![allow(non_camel_case_types, non_snake_case)]

/// Raven `genFunc_t` — Generator function type for waveforms.
///
/// Type definition source: `oracle/code/renderer/tr_local.h:180-192`
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
