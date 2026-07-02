#![allow(non_camel_case_types, non_snake_case)]

use crate::blockstream::cblock_stream::CBlockStream;

/// Raven `bstream_t` — a block-stream node with a link to the previous node.
///
/// Raven: (none).
/// Type definition source: `oracle/oracle/code/icarus/sequencer.h:13-17`
#[repr(C)]
pub struct bstream_t {
    pub stream: *mut CBlockStream,
    pub last: *mut bstream_t,
}

pub type bstream_s = bstream_t;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<bstream_t>() == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bstream_t, stream) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bstream_t, last) == 8);
