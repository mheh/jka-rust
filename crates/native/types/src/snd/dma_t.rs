#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `dma_t` — digital-audio (DMA) sound device description.
///
/// Type definition source: `oracle/codemp/client/snd_local.h:67-74`
/// Type definition source: `oracle/code/client/snd_local.h:67-74`
#[repr(C)]
pub struct dma_t {
    pub channels: c_int,
    /// mono samples in buffer
    pub samples: c_int,
    /// don't mix less than this #
    pub submission_chunk: c_int,
    pub samplebits: c_int,
    pub speed: c_int,
    pub buffer: *mut u8,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<dma_t>() == 32);
const _: () = assert!(core::mem::offset_of!(dma_t, channels) == 0);
const _: () = assert!(core::mem::offset_of!(dma_t, samples) == 4);
const _: () = assert!(core::mem::offset_of!(dma_t, submission_chunk) == 8);
const _: () = assert!(core::mem::offset_of!(dma_t, samplebits) == 12);
const _: () = assert!(core::mem::offset_of!(dma_t, speed) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(dma_t, buffer) == 24);
