#![allow(non_camel_case_types, non_snake_case)]

use crate::blockstream::cblock_stream::CBlockStream;

/// Raven `bstream_t` — a node in the block-stream linked list used by the
/// Icarus sequencer.
///
/// Raven: none.
/// Type definition source: `oracle/codemp/game/../icarus/sequencer.h:42-46`
#[repr(C)]
pub struct bstream_t {
    pub stream: *mut CBlockStream,
    pub last: *mut bstream_t,
}

pub type bstream_s = bstream_t;

const _: () = assert!(core::mem::size_of::<bstream_t>() == 16);
const _: () = assert!(core::mem::offset_of!(bstream_t, stream) == 0);
const _: () = assert!(core::mem::offset_of!(bstream_t, last) == 8);
