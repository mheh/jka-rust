#![allow(non_camel_case_types, non_snake_case)]

use super::nodetype::node_t;

/// Raven `huff_t` — adaptive Huffman coder state (tree, free-list bookkeeping,
/// and the fixed node/pointer pools it allocates from).
///
/// Type definition source: `oracle/codemp/qcommon/qcommon.h:1057-1069`
#[repr(C)]
pub struct huff_t {
    pub blocNode: i32,
    pub blocPtrs: i32,

    pub tree: *mut node_t,
    pub lhead: *mut node_t,
    pub ltail: *mut node_t,
    pub loc: [*mut node_t; 257],
    pub freelist: *mut *mut node_t,

    pub nodeList: [node_t; 768],
    pub nodePtrs: [*mut node_t; 768],
}

const _: () = assert!(core::mem::offset_of!(huff_t, blocNode) == 0);
const _: () = assert!(core::mem::offset_of!(huff_t, blocPtrs) == 4);
const _: () = assert!(core::mem::offset_of!(huff_t, tree) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<huff_t>() == 51248);
    assert!(core::mem::offset_of!(huff_t, lhead) == 16);
    assert!(core::mem::offset_of!(huff_t, ltail) == 24);
    assert!(core::mem::offset_of!(huff_t, loc) == 32);
    assert!(core::mem::offset_of!(huff_t, freelist) == 2088);
    assert!(core::mem::offset_of!(huff_t, nodeList) == 2096);
    assert!(core::mem::offset_of!(huff_t, nodePtrs) == 45104);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<huff_t>() == 28700);
    assert!(core::mem::offset_of!(huff_t, lhead) == 12);
    assert!(core::mem::offset_of!(huff_t, ltail) == 16);
    assert!(core::mem::offset_of!(huff_t, loc) == 20);
    assert!(core::mem::offset_of!(huff_t, freelist) == 1048);
    assert!(core::mem::offset_of!(huff_t, nodeList) == 1052);
    assert!(core::mem::offset_of!(huff_t, nodePtrs) == 25628);
};
