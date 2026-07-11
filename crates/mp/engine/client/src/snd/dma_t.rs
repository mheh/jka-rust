#![allow(non_camel_case_types, non_snake_case)]

/// Raven `dma_t` — digital-audio (DMA) sound device description.
///
/// Type definition source: `oracle/codemp/client/snd_local.h:67-74`
#[repr(C)]
pub struct dma_t {
    pub channels: i32,
    pub samples: i32,          // mono samples in buffer
    pub submission_chunk: i32, // don't mix less than this #
    pub samplebits: i32,
    pub speed: i32,
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
