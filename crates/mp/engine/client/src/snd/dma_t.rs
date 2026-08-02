//! Raven `dma_t` — the engine-owned mixer ring.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `dma_t` — the output format and the sample ring the paint chain writes.
///
/// The sound stack never crosses the ABI, so `byte *buffer` becomes an owned
/// `Vec<u8>` and an empty vector reads as Raven's NULL pointer.
/// Type definition source: `oracle/codemp/client/snd_local.h:67-74`
#[derive(Default)]
pub struct dma_t {
    pub channels: c_int,
    /// mono samples in buffer
    pub samples: c_int,
    /// don't mix less than this #
    pub submission_chunk: c_int,
    pub samplebits: c_int,
    pub speed: c_int,
    pub buffer: Vec<u8>,
}
