#![allow(non_camel_case_types, non_snake_case)]

/// Raven `SoundCompressionMethod_t` — sound compression method enumeration.
///
/// Type definition source: `oracle/code/client/snd_local.h:38-45`
#[repr(i32)]
pub enum SoundCompressionMethod_t {
    /// Formerly ct_NONE in EF1, now indicates 16-bit samples (the default)
    ct_16 = 0,
    /// MP3 compression
    ct_MP3 = 1,
    /// Used only for array sizing
    ct_NUMBEROF = 2,
}
