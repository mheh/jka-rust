#![allow(non_camel_case_types, non_snake_case)]

use super::huff_t::huff_t;

/// Raven `huffman_t` — paired compressor/decompressor state for the net-channel
/// adaptive Huffman coder.
///
/// Type definition source: `oracle/codemp/qcommon/qcommon.h:1071-1074`
#[repr(C)]
pub struct huffman_t {
    pub compressor: huff_t,
    pub decompressor: huff_t,
}

const _: () = assert!(core::mem::offset_of!(huffman_t, compressor) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<huffman_t>() == 102496);
    assert!(core::mem::offset_of!(huffman_t, decompressor) == 51248);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<huffman_t>() == 57400);
    assert!(core::mem::offset_of!(huffman_t, decompressor) == 28700);
};
